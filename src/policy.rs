use crate::manifest::{Manifest, ServiceEntry};
use anyhow::Result;
use colored::Colorize;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::path::{Path, PathBuf};

// ── Config ─────────────────────────────────────────────────────────────────────

/// Policy definition loaded from `.svccat/policy.yaml` or `svccat.policy.yaml`.
///
/// Example file:
/// ```yaml
/// required:
///   - team
///   - oncall
/// recommended:
///   - language
///   - platform
///   - docs
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
pub struct PolicyConfig {
    /// Fields every service MUST declare.  Violations are errors.
    #[serde(default)]
    pub required: Vec<String>,

    /// Fields every service SHOULD declare.  Violations are warnings.
    #[serde(default)]
    pub recommended: Vec<String>,
}

/// Why a policy file that *exists* could not be turned into a [`PolicyConfig`].
///
/// Both variants carry the path, because the whole point of reporting them is
/// that the user is looking at a file they believe is in effect.
#[derive(Debug)]
pub enum PolicyLoadError {
    /// The file exists but could not be read (permissions, a dangling symlink,
    /// a directory named `policy.yaml`, ...).
    Read {
        path: PathBuf,
        source: std::io::Error,
    },
    /// The file was read but is not a valid policy document.
    Parse {
        path: PathBuf,
        source: serde_yaml::Error,
    },
}

impl PolicyLoadError {
    /// The policy file the failure refers to.
    pub fn path(&self) -> &Path {
        match self {
            PolicyLoadError::Read { path, .. } | PolicyLoadError::Parse { path, .. } => path,
        }
    }
}

impl fmt::Display for PolicyLoadError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PolicyLoadError::Read { path, source } => {
                write!(
                    f,
                    "failed to read policy file '{}': {source}",
                    path.display()
                )
            }
            PolicyLoadError::Parse { path, source } => {
                write!(
                    f,
                    "failed to parse policy file '{}': {source}",
                    path.display()
                )
            }
        }
    }
}

impl std::error::Error for PolicyLoadError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            PolicyLoadError::Read { source, .. } => Some(source),
            PolicyLoadError::Parse { source, .. } => Some(source),
        }
    }
}

impl PolicyConfig {
    /// Try to load a policy file from the repo root.
    /// Returns `None` when no policy file exists (not an error).
    ///
    /// A policy file that exists but cannot be read or parsed is also reported
    /// as `None`, which makes a broken policy file indistinguishable from an
    /// absent one. Prefer [`PolicyConfig::load_checked`] on any path that
    /// reports to the user; this method is kept for callers that genuinely
    /// want "policy or nothing" and it delegates to `load_checked`, so the two
    /// can never disagree about which file wins.
    pub fn load(root: &Path) -> Option<Self> {
        Self::load_checked(root).ok().flatten()
    }

    /// Load a policy file from the repo root, reporting *why* an existing file
    /// did not load.
    ///
    /// - `Ok(None)`      - no policy file exists at any candidate path.
    /// - `Ok(Some(cfg))` - a candidate loaded cleanly.
    /// - `Err(e)`        - at least one candidate exists and none loaded; `e`
    ///   describes the first broken candidate.
    ///
    /// Candidate order and the "first one that loads wins" rule are identical
    /// to [`PolicyConfig::load`], so switching a call site over never changes
    /// which configuration is used - it only turns silence into a message.
    pub fn load_checked(root: &Path) -> Result<Option<Self>, PolicyLoadError> {
        let candidates = [
            root.join(".svccat").join("policy.yaml"),
            root.join(".svccat").join("policy.yml"),
            root.join("svccat.policy.yaml"),
            root.join("svccat.policy.yml"),
        ];
        let mut first_error: Option<PolicyLoadError> = None;
        for path in &candidates {
            if path.exists() {
                match Self::load_from(path) {
                    Ok(cfg) => return Ok(Some(cfg)),
                    Err(e) => {
                        if first_error.is_none() {
                            first_error = Some(e);
                        }
                    }
                }
            }
        }
        match first_error {
            Some(e) => Err(e),
            None => Ok(None),
        }
    }

    /// Read and parse one specific policy file.
    fn load_from(path: &Path) -> Result<Self, PolicyLoadError> {
        let text = std::fs::read_to_string(path).map_err(|source| PolicyLoadError::Read {
            path: path.to_path_buf(),
            source,
        })?;
        serde_yaml::from_str::<PolicyConfig>(&text).map_err(|source| PolicyLoadError::Parse {
            path: path.to_path_buf(),
            source,
        })
    }

    pub fn is_empty(&self) -> bool {
        self.required.is_empty() && self.recommended.is_empty()
    }
}

// ── Report ─────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum PolicySeverity {
    Error,
    Warning,
}

#[derive(Debug, Clone, Serialize)]
pub struct PolicyViolation {
    pub service: String,
    pub field: String,
    pub severity: PolicySeverity,
    pub message: String,
}

pub struct PolicyReport {
    pub violations: Vec<PolicyViolation>,
    pub services_checked: usize,
}

impl PolicyReport {
    pub fn error_count(&self) -> usize {
        self.violations
            .iter()
            .filter(|v| matches!(v.severity, PolicySeverity::Error))
            .count()
    }

    pub fn warning_count(&self) -> usize {
        self.violations
            .iter()
            .filter(|v| matches!(v.severity, PolicySeverity::Warning))
            .count()
    }

    pub fn passed(&self) -> bool {
        self.error_count() == 0
    }
}

// ── Engine ─────────────────────────────────────────────────────────────────────

/// Run policy checks against every service in the manifest.
pub fn check(manifest: &Manifest, config: &PolicyConfig) -> PolicyReport {
    let mut violations = Vec::new();

    for svc in &manifest.services {
        for field in &config.required {
            if !has_field(svc, field) {
                violations.push(PolicyViolation {
                    service: svc.name.clone(),
                    field: field.clone(),
                    severity: PolicySeverity::Error,
                    message: format!(
                        "service '{}' is missing required field '{}'",
                        svc.name, field
                    ),
                });
            }
        }
        for field in &config.recommended {
            if !has_field(svc, field) {
                violations.push(PolicyViolation {
                    service: svc.name.clone(),
                    field: field.clone(),
                    severity: PolicySeverity::Warning,
                    message: format!(
                        "service '{}' is missing recommended field '{}'",
                        svc.name, field
                    ),
                });
            }
        }
    }

    PolicyReport {
        violations,
        services_checked: manifest.services.len(),
    }
}

fn has_field(svc: &ServiceEntry, field: &str) -> bool {
    match field {
        "name" => !svc.name.is_empty(),
        "language" => svc.language.is_some(),
        "platform" => svc.platform.is_some(),
        "role" => svc.role.is_some(),
        "url" => svc.url.is_some(),
        "team" => svc.team.is_some(),
        "oncall" => svc.oncall.is_some(),
        "docs" => svc.docs.is_some(),
        "ci" => svc.ci.is_some(),
        _ => false,
    }
}

// ── Renderers ──────────────────────────────────────────────────────────────────

pub fn render_terminal(report: &PolicyReport, config: &PolicyConfig) {
    let errors = report.error_count();
    let warnings = report.warning_count();

    println!("{}", "svccat policy check".bold());
    println!();

    if !config.required.is_empty() {
        println!("  {}  {}", "Required:".bold(), config.required.join(", "));
    }
    if !config.recommended.is_empty() {
        println!(
            "  {}  {}",
            "Recommended:".bold(),
            config.recommended.join(", ")
        );
    }
    println!("  {}  {}", "Services:".bold(), report.services_checked);
    println!();

    if report.violations.is_empty() {
        println!(
            "  {} All {} service{} comply with policy",
            "✓".green().bold(),
            report.services_checked,
            if report.services_checked == 1 {
                ""
            } else {
                "s"
            }
        );
        return;
    }

    for v in &report.violations {
        match v.severity {
            PolicySeverity::Error => {
                println!("  {}  {}", "✗".red().bold(), v.message.red())
            }
            PolicySeverity::Warning => {
                println!("  {}  {}", "⚠".yellow(), v.message.yellow())
            }
        }
    }
    println!();
    println!(
        "  {} error{}, {} warning{}",
        errors,
        plural(errors),
        warnings,
        plural(warnings)
    );
}

pub fn render_json(report: &PolicyReport) -> Result<()> {
    let json = serde_json::to_string_pretty(&report.violations)?;
    println!("{json}");
    Ok(())
}

fn plural(n: usize) -> &'static str {
    if n == 1 {
        ""
    } else {
        "s"
    }
}

// ── Tests ──────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    /// A repo root with the given policy files written verbatim.
    /// Each entry is `(relative path, contents)`.
    fn repo_with(files: &[(&str, &str)]) -> TempDir {
        let dir = TempDir::new().unwrap();
        for (rel, body) in files {
            let path = dir.path().join(rel);
            fs::create_dir_all(path.parent().unwrap()).unwrap();
            fs::write(&path, body).unwrap();
        }
        dir
    }

    const VALID: &str = "required:\n  - team\nrecommended:\n  - docs\n";
    /// The committed fuzz seed `fuzz/corpus_seeds/fuzz_policy/required_not_a_list`:
    /// a scalar where a sequence belongs, the single most likely hand-edit typo.
    const SCALAR_WHERE_LIST_EXPECTED: &str = "required: team\nrecommended: language\n";

    #[test]
    fn load_checked_names_the_file_and_the_reason_when_it_cannot_be_parsed() {
        let dir = repo_with(&[(".svccat/policy.yaml", SCALAR_WHERE_LIST_EXPECTED)]);

        let err = PolicyConfig::load_checked(dir.path())
            .expect_err("a malformed policy file must not be reported as success");

        assert!(
            matches!(err, PolicyLoadError::Parse { .. }),
            "expected a parse failure, got {err:?}"
        );
        assert_eq!(err.path(), dir.path().join(".svccat").join("policy.yaml"));
        let rendered = err.to_string();
        assert!(
            rendered.contains("failed to parse policy file"),
            "message should say what went wrong: {rendered}"
        );
        assert!(
            rendered.contains("policy.yaml"),
            "message should name the file: {rendered}"
        );
    }

    #[test]
    fn load_checked_returns_ok_none_only_when_no_policy_file_exists() {
        let dir = repo_with(&[("services.yaml", "version: \"1\"\n")]);

        assert_eq!(PolicyConfig::load_checked(dir.path()).unwrap(), None);
    }

    #[test]
    fn load_checked_returns_the_config_when_the_file_is_valid() {
        let dir = repo_with(&[(".svccat/policy.yaml", VALID)]);

        let cfg = PolicyConfig::load_checked(dir.path())
            .unwrap()
            .expect("a valid policy file must load");
        assert_eq!(cfg.required, vec!["team".to_string()]);
        assert_eq!(cfg.recommended, vec!["docs".to_string()]);
    }

    /// The compatibility contract that lets every call site migrate freely:
    /// `load` is exactly `load_checked` with the error dropped, for every
    /// arrangement of policy files - including the one where a broken
    /// candidate is followed by a good one. Reads BOTH functions, so the two
    /// cannot drift apart later (only one of them is covered by the tests
    /// above).
    #[test]
    fn load_is_load_checked_with_the_error_dropped() {
        let cases: Vec<(&str, Vec<(&str, &str)>)> = vec![
            (
                "no policy file",
                vec![("services.yaml", "version: \"1\"\n")],
            ),
            ("valid policy file", vec![(".svccat/policy.yaml", VALID)]),
            (
                "broken policy file",
                vec![(".svccat/policy.yaml", SCALAR_WHERE_LIST_EXPECTED)],
            ),
            (
                "broken first candidate, valid later candidate",
                vec![
                    (".svccat/policy.yaml", SCALAR_WHERE_LIST_EXPECTED),
                    ("svccat.policy.yaml", VALID),
                ],
            ),
            (
                "valid first candidate, broken later candidate",
                vec![
                    (".svccat/policy.yaml", VALID),
                    ("svccat.policy.yaml", SCALAR_WHERE_LIST_EXPECTED),
                ],
            ),
        ];

        for (label, files) in cases {
            let dir = repo_with(&files);
            let via_load = PolicyConfig::load(dir.path());
            let via_checked = PolicyConfig::load_checked(dir.path()).ok().flatten();
            assert_eq!(via_load, via_checked, "diverged on case: {label}");
        }
    }

    /// A broken candidate must not shadow a good one: this is the pre-existing
    /// `load` behaviour that `load_checked` deliberately preserves, so
    /// upgrading a call site cannot change which file is in force.
    #[test]
    fn a_broken_first_candidate_does_not_hide_a_valid_later_one() {
        let dir = repo_with(&[
            (".svccat/policy.yaml", SCALAR_WHERE_LIST_EXPECTED),
            ("svccat.policy.yaml", VALID),
        ]);

        let cfg = PolicyConfig::load_checked(dir.path())
            .expect("a later valid candidate must still win")
            .expect("a policy file exists");
        assert_eq!(cfg.required, vec!["team".to_string()]);
    }
}
