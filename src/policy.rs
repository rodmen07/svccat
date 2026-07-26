use crate::manifest::{Manifest, ServiceEntry};
use anyhow::Result;
use colored::Colorize;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::io::Read as _;
use std::path::{Path, PathBuf};

// ── Deserialization limits ─────────────────────────────────────────────────────
// `Manifest::load` (`src/manifest.rs`) caps the file size before parsing and
// bounds the parsed document afterwards, naming YAML-bomb resource exhaustion
// as the reason. A policy file is read from the same repository, with the same
// deserializer, across the same trust boundary, so it gets the same posture.
//
// What these limits do and do not buy is worth stating plainly, because that is
// where the remaining risk lives:
//
// * The byte cap runs BEFORE the parser sees anything, so an oversized document
//   is never handed to serde and never held in memory whole.
// * The two shape limits run AFTER deserialization. They bound what a caller
//   can be handed, and they turn an absurd document into a named error instead
//   of a silently accepted one, but they cannot un-allocate what the parser
//   already built. Alias expansion that stays inside the byte cap (one big
//   anchor, many aliases) is therefore still possible here, exactly as it is in
//   `Manifest::load`. Capping it would need an alias budget in the YAML
//   deserializer, which `serde_yaml_ng` 0.10 does not expose.

/// Maximum policy file size: 1 MiB.
///
/// Deliberately far tighter than `Manifest`'s 10 MB cap, because the two
/// documents are not comparable: a manifest carries an arbitrary service
/// catalogue, while a policy file carries two lists of field names drawn from
/// the nine `has_field` knows about. 1 MiB is already four orders of magnitude
/// more than any real policy file needs.
const MAX_POLICY_SIZE: u64 = 1024 * 1024;

/// Maximum number of field entries across `required` and `recommended`
/// combined (mirrors `MAX_SERVICES`).
const MAX_POLICY_FIELDS: usize = 10_000;

/// Maximum length of a single field name (mirrors `MAX_SERVICE_NAME_LEN`).
const MAX_POLICY_FIELD_LEN: usize = 256;

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
    /// The file exists but breaches one of the resource limits above: it is
    /// larger than [`MAX_POLICY_SIZE`], declares more than
    /// [`MAX_POLICY_FIELDS`] fields, or names a field longer than
    /// [`MAX_POLICY_FIELD_LEN`].
    ///
    /// Distinct from [`PolicyLoadError::Parse`] on purpose: the document is
    /// well-formed YAML, so telling the user it "failed to parse" would send
    /// them hunting for a syntax error that is not there.
    Limit { path: PathBuf, detail: String },
}

impl PolicyLoadError {
    /// The policy file the failure refers to.
    pub fn path(&self) -> &Path {
        match self {
            PolicyLoadError::Read { path, .. }
            | PolicyLoadError::Parse { path, .. }
            | PolicyLoadError::Limit { path, .. } => path,
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
            PolicyLoadError::Limit { path, detail } => {
                write!(
                    f,
                    "policy file '{}' exceeds a resource limit: {detail}",
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
            PolicyLoadError::Limit { .. } => None,
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

    /// Read and parse one specific policy file, under the resource limits
    /// declared at the top of this module.
    ///
    /// # Security
    /// - Refuses files over [`MAX_POLICY_SIZE`] before the parser runs, so an
    ///   oversized document cannot be turned into events at all.
    /// - Bounds the field count and field-name length of what did parse.
    fn load_from(path: &Path) -> Result<Self, PolicyLoadError> {
        // A bounded read rather than stat-then-read: the cap is enforced on the
        // bytes actually taken from the file, so a file that grows between the
        // two syscalls cannot slip past it, and nothing over the cap is ever
        // held in memory.
        let file = std::fs::File::open(path).map_err(|source| read_error(path, source))?;
        let mut bytes = Vec::new();
        file.take(MAX_POLICY_SIZE + 1)
            .read_to_end(&mut bytes)
            .map_err(|source| read_error(path, source))?;

        if bytes.len() as u64 > MAX_POLICY_SIZE {
            return Err(PolicyLoadError::Limit {
                path: path.to_path_buf(),
                detail: format!(
                    "file is larger than the {MAX_POLICY_SIZE} byte maximum, so it was not parsed"
                ),
            });
        }

        let text = String::from_utf8(bytes).map_err(|e| {
            read_error(
                path,
                std::io::Error::new(std::io::ErrorKind::InvalidData, e),
            )
        })?;

        let config = serde_yaml::from_str::<PolicyConfig>(&text).map_err(|source| {
            PolicyLoadError::Parse {
                path: path.to_path_buf(),
                source,
            }
        })?;
        config.validate_limits(path)?;
        Ok(config)
    }

    /// Bound a parsed policy document, the way `Manifest::validate_limits`
    /// bounds a parsed manifest.
    fn validate_limits(&self, path: &Path) -> Result<(), PolicyLoadError> {
        let limit = |detail: String| PolicyLoadError::Limit {
            path: path.to_path_buf(),
            detail,
        };

        let declared = self.required.len() + self.recommended.len();
        if declared > MAX_POLICY_FIELDS {
            return Err(limit(format!(
                "declares too many fields ({declared}, max {MAX_POLICY_FIELDS})"
            )));
        }

        for (list, names) in [
            ("required", &self.required),
            ("recommended", &self.recommended),
        ] {
            for name in names {
                if name.len() > MAX_POLICY_FIELD_LEN {
                    return Err(limit(format!(
                        "a '{}' field name is too long ({} bytes, max {})",
                        list,
                        name.len(),
                        MAX_POLICY_FIELD_LEN
                    )));
                }
            }
        }

        Ok(())
    }

    pub fn is_empty(&self) -> bool {
        self.required.is_empty() && self.recommended.is_empty()
    }
}

/// Every I/O failure in [`PolicyConfig::load_from`] reports the same way.
fn read_error(path: &Path, source: std::io::Error) -> PolicyLoadError {
    PolicyLoadError::Read {
        path: path.to_path_buf(),
        source,
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

    // ── Resource limits ────────────────────────────────────────────────────
    //
    // Each limit gets a pair: a document exactly AT the limit that must still
    // load, and one a single unit OVER it that must be refused. The pairs are
    // what make these tests limit tests rather than "the error path exists"
    // tests - a guard that refuses everything passes only the second half.

    /// A body of exactly `len` bytes that is a valid policy document, padded
    /// with a YAML comment.
    fn valid_policy_of_exactly(len: usize) -> String {
        let mut body = String::from(VALID);
        body.push_str("# ");
        assert!(body.len() <= len, "cannot pad down to {len} bytes");
        while body.len() < len {
            body.push('x');
        }
        body
    }

    #[test]
    fn a_policy_file_at_exactly_the_size_cap_still_loads() {
        let body = valid_policy_of_exactly(MAX_POLICY_SIZE as usize);
        assert_eq!(body.len() as u64, MAX_POLICY_SIZE);
        let dir = repo_with(&[(".svccat/policy.yaml", &body)]);

        let cfg = PolicyConfig::load_checked(dir.path())
            .expect("a file exactly at the cap is not over it")
            .expect("a policy file exists");
        assert_eq!(cfg.required, vec!["team".to_string()]);
    }

    #[test]
    fn one_byte_over_the_size_cap_is_refused() {
        let body = valid_policy_of_exactly(MAX_POLICY_SIZE as usize + 1);
        assert_eq!(body.len() as u64, MAX_POLICY_SIZE + 1);
        let dir = repo_with(&[(".svccat/policy.yaml", &body)]);

        let err = PolicyConfig::load_checked(dir.path())
            .expect_err("a file over the size cap must be refused");

        assert!(
            matches!(err, PolicyLoadError::Limit { .. }),
            "expected a limit failure, got {err:?}"
        );
        assert_eq!(err.path(), dir.path().join(".svccat").join("policy.yaml"));
        let rendered = err.to_string();
        assert!(
            rendered.contains("exceeds a resource limit") && rendered.contains("larger than"),
            "message should say the file is too big: {rendered}"
        );
        // The content itself is a perfectly good policy document, so nothing
        // but the size could have refused it.
        assert!(
            PolicyConfig::load_checked(repo_with(&[(".svccat/policy.yaml", VALID)]).path()).is_ok()
        );
    }

    /// The discriminator between "there is a size limit" and "the size limit
    /// protects the parser": an oversized document whose content would ALSO
    /// fail to deserialize must be refused as a limit breach, which can only
    /// happen if the cap is checked before `serde_yaml` is handed the text.
    #[test]
    fn the_size_cap_runs_before_the_parser() {
        let mut body = String::from(SCALAR_WHERE_LIST_EXPECTED);
        body.push_str("# ");
        while body.len() as u64 <= MAX_POLICY_SIZE {
            body.push('x');
        }
        let dir = repo_with(&[(".svccat/policy.yaml", &body)]);

        let err =
            PolicyConfig::load_checked(dir.path()).expect_err("oversized file must be refused");

        assert!(
            matches!(err, PolicyLoadError::Limit { .. }),
            "an oversized file must never reach the parser, got {err:?}"
        );
    }

    #[test]
    fn a_policy_at_exactly_the_field_limit_still_loads() {
        let mut body = String::from("required:\n");
        for i in 0..MAX_POLICY_FIELDS {
            body.push_str(&format!("  - f{i}\n"));
        }
        let dir = repo_with(&[(".svccat/policy.yaml", &body)]);

        let cfg = PolicyConfig::load_checked(dir.path())
            .expect("exactly the maximum number of fields is allowed")
            .expect("a policy file exists");
        assert_eq!(cfg.required.len(), MAX_POLICY_FIELDS);
    }

    /// The two lists are bounded together, not separately: 5_000 + 5_001 is
    /// over the limit even though neither list is.
    #[test]
    fn too_many_declared_fields_are_refused_across_both_lists() {
        let half = MAX_POLICY_FIELDS / 2;
        let mut body = String::from("required:\n");
        for i in 0..half {
            body.push_str(&format!("  - r{i}\n"));
        }
        body.push_str("recommended:\n");
        for i in 0..=half {
            body.push_str(&format!("  - c{i}\n"));
        }
        let dir = repo_with(&[(".svccat/policy.yaml", &body)]);

        let err = PolicyConfig::load_checked(dir.path())
            .expect_err("more fields than the limit must be refused");

        assert!(
            matches!(err, PolicyLoadError::Limit { .. }),
            "expected a limit failure, got {err:?}"
        );
        let rendered = err.to_string();
        assert!(
            rendered.contains("declares too many fields")
                && rendered.contains(&(MAX_POLICY_FIELDS + 1).to_string()),
            "message should report the count it saw: {rendered}"
        );
    }

    #[test]
    fn a_field_name_at_exactly_the_length_cap_still_loads() {
        let name = "n".repeat(MAX_POLICY_FIELD_LEN);
        let dir = repo_with(&[(".svccat/policy.yaml", &format!("required:\n  - {name}\n"))]);

        let cfg = PolicyConfig::load_checked(dir.path())
            .expect("a name exactly at the cap is not over it")
            .expect("a policy file exists");
        assert_eq!(cfg.required, vec![name]);
    }

    #[test]
    fn a_field_name_one_byte_over_the_length_cap_is_refused() {
        let name = "n".repeat(MAX_POLICY_FIELD_LEN + 1);
        let dir = repo_with(&[(
            ".svccat/policy.yaml",
            &format!("recommended:\n  - {name}\n"),
        )]);

        let err = PolicyConfig::load_checked(dir.path())
            .expect_err("an over-long field name must be refused");

        assert!(
            matches!(err, PolicyLoadError::Limit { .. }),
            "expected a limit failure, got {err:?}"
        );
        let rendered = err.to_string();
        assert!(
            rendered.contains("'recommended' field name is too long"),
            "message should name the list it came from: {rendered}"
        );
    }

    /// The limit failures join the existing first-error/fall-through logic
    /// rather than short-circuiting it: an over-limit candidate is skipped the
    /// same way an unparseable one is.
    #[test]
    fn an_over_limit_first_candidate_does_not_hide_a_valid_later_one() {
        let oversized = valid_policy_of_exactly(MAX_POLICY_SIZE as usize + 1);
        let dir = repo_with(&[
            (".svccat/policy.yaml", oversized.as_str()),
            ("svccat.policy.yaml", VALID),
        ]);

        let cfg = PolicyConfig::load_checked(dir.path())
            .expect("a later valid candidate must still win")
            .expect("a policy file exists");
        assert_eq!(cfg.required, vec!["team".to_string()]);
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
