//! The `[reporting]` section of a workspace `svccat.toml`.
//!
//! A workspace config may carry reporting defaults alongside `[workspace]`:
//!
//! ```toml
//! [workspace]
//! name = "Platform Engineering"
//! repos = [{ name = "api", path = "api-repo" }]
//!
//! [reporting]
//! format = "json"                  # default output format
//! include_cross_repo_deps = true   # run cross-repo dependency analysis
//! exclude_patterns = ["vendor/*"]  # extra discovery ignore globs
//! ```
//!
//! Precedence is the same for every key: an explicit CLI flag wins, then the
//! `[reporting]` value, then the hard-coded default. `exclude_patterns` is the
//! deliberate exception, because "wins" makes no sense for it: it is additive
//! and merges into the ignore globs coming from `--ignore` and the root
//! `svccat.toml`, so no source of ignores can silently drop another's.
//!
//! Unknown keys inside `[reporting]` are ignored, matching how the rest of
//! `svccat.toml` is parsed (`SvccatConfig` uses plain serde derive, which skips
//! unrecognised fields). A key that *is* recognised but carries an unusable
//! value is rejected, following the same reasoning as `workspace check
//! --filter`: a typo must not silently change what gets reported.

use crate::cli::OutputFormat;
use anyhow::{anyhow, Result};
use clap::ValueEnum;
use serde::{Deserialize, Serialize};

/// Hard-coded default for [`ReportingConfig::include_cross_repo_deps`].
///
/// Cross-repo dependency analysis was unconditional before the `[reporting]`
/// section existed, so defaulting to `true` keeps existing workspaces intact.
pub const DEFAULT_INCLUDE_CROSS_REPO_DEPS: bool = true;

fn default_include_cross_repo_deps() -> bool {
    DEFAULT_INCLUDE_CROSS_REPO_DEPS
}

/// Reporting defaults parsed from `[reporting]`.
///
/// A missing section yields [`ReportingConfig::default()`]: no format override,
/// cross-repo dependency analysis on, and no extra exclude patterns.
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct ReportingConfig {
    /// Default output format for `workspace check`. `None` means "not
    /// configured", which leaves the hard-coded terminal default in play.
    #[serde(default)]
    pub format: Option<OutputFormat>,

    /// Whether to run cross-repo dependency analysis.
    ///
    /// When false, the dependency work is never started rather than computed
    /// and then hidden; see `workspace::analyze_cross_repo_dependencies`.
    #[serde(default = "default_include_cross_repo_deps")]
    pub include_cross_repo_deps: bool,

    /// Glob patterns merged into the discovery ignore globs for every repo.
    #[serde(default)]
    pub exclude_patterns: Vec<String>,
}

impl Default for ReportingConfig {
    fn default() -> Self {
        Self {
            format: None,
            include_cross_repo_deps: DEFAULT_INCLUDE_CROSS_REPO_DEPS,
            exclude_patterns: Vec::new(),
        }
    }
}

impl ReportingConfig {
    /// Merge [`Self::exclude_patterns`] into an existing set of discovery
    /// ignore globs.
    ///
    /// The result is fed straight into
    /// [`discovery::discover_services_with_opts`](crate::discovery::discover_services_with_opts),
    /// which compiles it, together with the manifest's own `discovery.ignore`
    /// list, into `glob::Pattern`s. There is no second glob engine here: this
    /// is only a concatenation, so an exclude pattern behaves exactly like a
    /// `--ignore` flag or a manifest ignore entry.
    ///
    /// `extra_ignore` comes first so the merged list reads
    /// CLI + root config, then workspace excludes; ordering is cosmetic
    /// because discovery treats the globs as an unordered "matches any" set.
    pub fn merged_ignore(&self, extra_ignore: &[String]) -> Vec<String> {
        let mut merged = Vec::with_capacity(extra_ignore.len() + self.exclude_patterns.len());
        merged.extend_from_slice(extra_ignore);
        merged.extend(self.exclude_patterns.iter().cloned());
        merged
    }
}

/// Resolve the effective output format for `workspace check`.
///
/// Precedence: `--format` on the CLI, then `[reporting].format`, then the
/// hard-coded [`OutputFormat::Terminal`] default. The config value was already
/// validated by [`parse`], so this cannot fail.
pub fn resolve_format(cli_format: Option<OutputFormat>, config: &ReportingConfig) -> OutputFormat {
    cli_format
        .or_else(|| config.format.clone())
        .unwrap_or(OutputFormat::Terminal)
}

/// Parse the optional `[reporting]` section of a workspace config document.
///
/// `section` is the value of the top-level `reporting` key, or `None` when the
/// section is absent. Unknown keys are ignored; recognised keys with an
/// unusable value are rejected with a message naming the key.
pub fn parse(section: Option<&toml::Value>) -> Result<ReportingConfig> {
    let Some(section) = section else {
        return Ok(ReportingConfig::default());
    };

    let table = section
        .as_table()
        .ok_or_else(|| anyhow!("[reporting] must be a table, got {}", section.type_str()))?;

    let format = match table.get("format") {
        None => None,
        Some(value) => {
            let name = value.as_str().ok_or_else(|| {
                anyhow!(
                    "reporting.format must be a string, got {}",
                    value.type_str()
                )
            })?;
            Some(parse_format(name)?)
        }
    };

    let include_cross_repo_deps = match table.get("include_cross_repo_deps") {
        None => DEFAULT_INCLUDE_CROSS_REPO_DEPS,
        Some(value) => value.as_bool().ok_or_else(|| {
            anyhow!(
                "reporting.include_cross_repo_deps must be a boolean, got {}",
                value.type_str()
            )
        })?,
    };

    let exclude_patterns = match table.get("exclude_patterns") {
        None => Vec::new(),
        Some(value) => {
            let array = value.as_array().ok_or_else(|| {
                anyhow!(
                    "reporting.exclude_patterns must be an array of strings, got {}",
                    value.type_str()
                )
            })?;
            array
                .iter()
                .enumerate()
                .map(|(idx, item)| {
                    item.as_str().map(String::from).ok_or_else(|| {
                        anyhow!(
                            "reporting.exclude_patterns[{}] must be a string, got {}",
                            idx,
                            item.type_str()
                        )
                    })
                })
                .collect::<Result<Vec<String>>>()?
        }
    };

    Ok(ReportingConfig {
        format,
        include_cross_repo_deps,
        exclude_patterns,
    })
}

/// Turn a `[reporting].format` string into an [`OutputFormat`].
///
/// Accepts exactly the values `--format` accepts, so the config and the flag
/// can never drift apart. An unrecognised value is an error listing every
/// valid name rather than a silent fallback to terminal output, which would
/// quietly emit the wrong format in a CI pipeline.
fn parse_format(name: &str) -> Result<OutputFormat> {
    OutputFormat::from_str(name, true).map_err(|_| {
        anyhow!(
            "unknown reporting.format '{}' (valid: {})",
            name,
            valid_format_names().join(", ")
        )
    })
}

/// Every accepted `--format` value, in declaration order.
fn valid_format_names() -> Vec<String> {
    OutputFormat::value_variants()
        .iter()
        .filter_map(|variant| {
            variant
                .to_possible_value()
                .map(|value| value.get_name().to_string())
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn section(toml_text: &str) -> toml::Value {
        toml_text.parse::<toml::Value>().unwrap()
    }

    fn parse_reporting(toml_text: &str) -> Result<ReportingConfig> {
        let doc = section(toml_text);
        parse(doc.get("reporting"))
    }

    // ── defaults ───────────────────────────────────────────────────────────

    #[test]
    fn absent_section_yields_defaults() {
        let config = parse_reporting("other = 1").unwrap();
        assert_eq!(config, ReportingConfig::default());
        assert_eq!(config.format, None);
        assert!(config.include_cross_repo_deps);
        assert!(config.exclude_patterns.is_empty());
    }

    #[test]
    fn empty_section_yields_defaults() {
        let config = parse_reporting("[reporting]").unwrap();
        assert_eq!(config, ReportingConfig::default());
    }

    #[test]
    fn parses_every_key() {
        let config = parse_reporting(
            r#"
[reporting]
format = "json"
include_cross_repo_deps = false
exclude_patterns = ["vendor/*", "examples/*"]
"#,
        )
        .unwrap();

        assert_eq!(config.format, Some(OutputFormat::Json));
        assert!(!config.include_cross_repo_deps);
        assert_eq!(config.exclude_patterns, vec!["vendor/*", "examples/*"]);
    }

    #[test]
    fn omitted_keys_fall_back_individually() {
        let config = parse_reporting(
            r#"
[reporting]
format = "markdown"
"#,
        )
        .unwrap();

        assert_eq!(config.format, Some(OutputFormat::Markdown));
        // The other two keys keep their hard-coded defaults.
        assert!(config.include_cross_repo_deps);
        assert!(config.exclude_patterns.is_empty());
    }

    // ── unknown keys ───────────────────────────────────────────────────────

    #[test]
    fn unknown_keys_are_ignored() {
        // Same treatment as unknown keys anywhere else in svccat.toml: serde's
        // derive silently skips them, so [reporting] does too.
        let config = parse_reporting(
            r#"
[reporting]
format = "json"
future_option = "ignored"
nested = { also = "ignored" }
"#,
        )
        .unwrap();

        assert_eq!(config.format, Some(OutputFormat::Json));
        assert!(config.include_cross_repo_deps);
    }

    // ── value validation ───────────────────────────────────────────────────

    #[test]
    fn unknown_format_value_is_rejected_and_lists_valid_names() {
        let err = parse_reporting(
            r#"
[reporting]
format = "jsonn"
"#,
        )
        .unwrap_err()
        .to_string();

        assert!(
            err.contains("jsonn"),
            "error should quote the bad value: {err}"
        );
        assert!(
            err.contains("json"),
            "error should list valid formats: {err}"
        );
        assert!(
            err.contains("markdown"),
            "error should list valid formats: {err}"
        );
    }

    #[test]
    fn format_must_be_a_string() {
        let err = parse_reporting(
            r#"
[reporting]
format = 42
"#,
        )
        .unwrap_err()
        .to_string();
        assert!(err.contains("reporting.format"), "{err}");
    }

    #[test]
    fn include_cross_repo_deps_must_be_a_boolean() {
        let err = parse_reporting(
            r#"
[reporting]
include_cross_repo_deps = "yes"
"#,
        )
        .unwrap_err()
        .to_string();
        assert!(err.contains("reporting.include_cross_repo_deps"), "{err}");
    }

    #[test]
    fn exclude_patterns_must_be_an_array_of_strings() {
        let err = parse_reporting(
            r#"
[reporting]
exclude_patterns = "vendor/*"
"#,
        )
        .unwrap_err()
        .to_string();
        assert!(err.contains("reporting.exclude_patterns"), "{err}");

        let err = parse_reporting(
            r#"
[reporting]
exclude_patterns = ["ok", 7]
"#,
        )
        .unwrap_err()
        .to_string();
        assert!(err.contains("reporting.exclude_patterns[1]"), "{err}");
    }

    #[test]
    fn reporting_must_be_a_table() {
        let err = parse_reporting(r#"reporting = "nope""#)
            .unwrap_err()
            .to_string();
        assert!(err.contains("[reporting]"), "{err}");
    }

    #[test]
    fn every_cli_format_value_is_accepted_in_config() {
        // The config accepts exactly what --format accepts; if a new variant is
        // added to OutputFormat this test covers it automatically.
        for name in valid_format_names() {
            let text = format!("[reporting]\nformat = \"{name}\"\n");
            assert!(
                parse_reporting(&text).is_ok(),
                "config should accept --format value '{name}'"
            );
        }
    }

    // ── format precedence ──────────────────────────────────────────────────

    #[test]
    fn format_precedence_cli_beats_config_beats_default() {
        let configured = ReportingConfig {
            format: Some(OutputFormat::Json),
            ..ReportingConfig::default()
        };
        let unset = ReportingConfig::default();

        // CLI wins over config.
        assert_eq!(
            resolve_format(Some(OutputFormat::Markdown), &configured),
            OutputFormat::Markdown
        );
        // Config wins over the hard-coded default.
        assert_eq!(resolve_format(None, &configured), OutputFormat::Json);
        // Hard-coded default when neither is set.
        assert_eq!(resolve_format(None, &unset), OutputFormat::Terminal);
        // CLI wins over the hard-coded default too.
        assert_eq!(
            resolve_format(Some(OutputFormat::Csv), &unset),
            OutputFormat::Csv
        );
    }

    #[test]
    fn cli_can_select_the_default_format_over_a_config_value() {
        // Passing --format terminal explicitly is not the same as passing
        // nothing: it must override a configured format.
        let configured = ReportingConfig {
            format: Some(OutputFormat::Json),
            ..ReportingConfig::default()
        };
        assert_eq!(
            resolve_format(Some(OutputFormat::Terminal), &configured),
            OutputFormat::Terminal
        );
    }

    // ── exclude pattern merge ──────────────────────────────────────────────

    #[test]
    fn merged_ignore_appends_to_existing_globs() {
        let config = ReportingConfig {
            exclude_patterns: vec!["vendor/*".to_string()],
            ..ReportingConfig::default()
        };
        let merged = config.merged_ignore(&["examples/*".to_string()]);
        assert_eq!(merged, vec!["examples/*", "vendor/*"]);
    }

    #[test]
    fn merged_ignore_is_additive_not_overriding() {
        // CLI/root-config ignores survive even when the workspace config also
        // supplies excludes: neither source can drop the other's patterns.
        let config = ReportingConfig {
            exclude_patterns: vec!["a/*".to_string(), "b/*".to_string()],
            ..ReportingConfig::default()
        };
        let merged = config.merged_ignore(&["cli/*".to_string()]);
        assert!(merged.contains(&"cli/*".to_string()));
        assert!(merged.contains(&"a/*".to_string()));
        assert!(merged.contains(&"b/*".to_string()));
        assert_eq!(merged.len(), 3);
    }

    #[test]
    fn merged_ignore_handles_empty_sides() {
        let empty = ReportingConfig::default();
        assert!(empty.merged_ignore(&[]).is_empty());
        assert_eq!(empty.merged_ignore(&["x".to_string()]), vec!["x"]);

        let config = ReportingConfig {
            exclude_patterns: vec!["y".to_string()],
            ..ReportingConfig::default()
        };
        assert_eq!(config.merged_ignore(&[]), vec!["y"]);
    }

    #[test]
    fn merged_ignore_keeps_duplicates_verbatim() {
        // Discovery compiles the list into a "matches any" glob set, so a
        // duplicate is harmless; dedup would only hide what the user wrote.
        let config = ReportingConfig {
            exclude_patterns: vec!["vendor/*".to_string()],
            ..ReportingConfig::default()
        };
        let merged = config.merged_ignore(&["vendor/*".to_string()]);
        assert_eq!(merged, vec!["vendor/*", "vendor/*"]);
    }
}
