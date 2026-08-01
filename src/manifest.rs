use crate::rules::Rule;
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

// ── Deserialization Limits ────────────────────────────────────────────────────
// These limits prevent resource exhaustion attacks using YAML/TOML bombs
// (e.g., exponential expansion via anchors, deep nesting, large collections)

/// Maximum manifest file size: 10 MB
/// This prevents resource exhaustion from YAML anchor expansion and deep nesting
const MAX_MANIFEST_SIZE: u64 = 10 * 1024 * 1024;

/// Maximum number of services in a manifest (reasonable upper bound for large monorepos)
const MAX_SERVICES: usize = 10_000;

/// Maximum service name length to prevent string bombs
const MAX_SERVICE_NAME_LEN: usize = 256;

// ── Manifest ─────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[non_exhaustive]
pub struct Manifest {
    #[serde(default = "default_version")]
    pub version: String,

    #[serde(default)]
    pub discovery: DiscoveryConfig,

    #[serde(default)]
    pub policy: PolicyConfig,

    pub services: Vec<ServiceEntry>,
}

impl Manifest {
    /// Load manifest from file with resource limits to prevent deserialization attacks.
    ///
    /// # Security
    /// - Enforces maximum file size (10 MB) to prevent YAML bomb attacks
    /// - Validates service count and field lengths
    /// - Rejects manifests with excessive nesting or expansion
    pub fn load(path: &Path) -> Result<Self> {
        // Check file size to prevent deserialization bombs (YAML anchors, deep nesting, etc)
        let metadata = std::fs::metadata(path)
            .with_context(|| format!("cannot stat manifest: {}", path.display()))?;

        if metadata.len() > MAX_MANIFEST_SIZE {
            anyhow::bail!(
                "manifest file is too large ({} bytes, max {} bytes). This check prevents resource exhaustion from YAML expansion attacks.",
                metadata.len(),
                MAX_MANIFEST_SIZE
            );
        }

        let text = std::fs::read_to_string(path)
            .with_context(|| format!("cannot read manifest: {}", path.display()))?;

        let manifest: Self = serde_yaml::from_str(&text)
            .with_context(|| format!("cannot parse manifest: {}", path.display()))?;

        // Validate loaded manifest
        Self::validate_limits(&manifest, path)?;

        Ok(manifest)
    }

    /// Validate manifest for resource exhaustion limits and security constraints.
    fn validate_limits(manifest: &Manifest, path: &Path) -> Result<()> {
        if manifest.services.len() > MAX_SERVICES {
            anyhow::bail!(
                "manifest has too many services ({}, max {})",
                manifest.services.len(),
                MAX_SERVICES
            );
        }

        // Sanity checks on service entries to catch expansions early
        for svc in &manifest.services {
            if svc.name.len() > MAX_SERVICE_NAME_LEN {
                anyhow::bail!(
                    "service name too long in {}: '{}' ({} bytes, max {})",
                    path.display(),
                    &svc.name[..MAX_SERVICE_NAME_LEN.min(50)],
                    svc.name.len(),
                    MAX_SERVICE_NAME_LEN
                );
            }

            // Check depends_on list isn't absurdly large
            if svc.depends_on.len() > 1000 {
                anyhow::bail!(
                    "service '{}' has too many dependencies ({}, max 1000)",
                    svc.name,
                    svc.depends_on.len()
                );
            }

            // Validate service paths to prevent directory traversal
            svc.validate()
                .with_context(|| format!("service '{}' has invalid paths", svc.name))?;
        }

        Ok(())
    }

    /// Effective discovery glob patterns, falling back to common monorepo conventions.
    pub fn effective_discovery_paths(&self) -> Vec<String> {
        if self.discovery.paths.is_empty() {
            DEFAULT_DISCOVERY_PATHS
                .iter()
                .map(|s| s.to_string())
                .collect()
        } else {
            self.discovery.paths.clone()
        }
    }
}

// ── Policy config ─────────────────────────────────────────────────────────────

/// Declarative policy rules enforced during `svccat check`.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PolicyConfig {
    /// Fields that every service entry must declare.
    /// Missing fields become error-level drift items.
    /// Example: ["url", "language", "platform"]
    #[serde(default)]
    pub require_fields: Vec<String>,

    /// Custom validation rules for services.
    /// Example: { id: "naming_convention", description: "...", expression: "name matches ^service-", severity: "error" }
    #[serde(default)]
    pub rules: Vec<Rule>,
}

// ── Discovery config ─────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DiscoveryConfig {
    /// Glob patterns (relative to repo root) that expand to candidate service
    /// directories.  Defaults to common monorepo conventions when empty.
    #[serde(default)]
    pub paths: Vec<String>,

    /// Filenames whose presence inside a directory marks it as a service.
    #[serde(default = "default_markers")]
    pub markers: Vec<String>,

    /// Glob patterns (relative to repo root) for directories to exclude from
    /// discovery.  E.g. `["examples/*", "vendor/*"]`.
    #[serde(default)]
    pub ignore: Vec<String>,
}

/// Glob patterns tried when `discovery.paths` is empty.
pub const DEFAULT_DISCOVERY_PATHS: &[&str] =
    &["services/*", "microservices/*", "apps/*", "packages/*"];

fn default_markers() -> Vec<String> {
    default_markers_pub()
}

/// Public version of the default markers list, usable outside this module.
pub fn default_markers_pub() -> Vec<String> {
    [
        "Cargo.toml",
        "Dockerfile",
        "go.mod",
        "package.json",
        "pyproject.toml",
        "requirements.txt",
        // JVM
        "build.gradle",
        "build.gradle.kts",
        "pom.xml",
        // C / C++
        "CMakeLists.txt",
        // .NET
        "Directory.Build.props",
        // Ruby
        "Gemfile",
        // Elixir
        "mix.exs",
        // Dart / Flutter
        "pubspec.yaml",
    ]
    .iter()
    .map(|s| s.to_string())
    .collect()
}

// ── Service entry ─────────────────────────────────────────────────────────────

/// One entry in the `services:` list.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
#[non_exhaustive]
pub struct ServiceEntry {
    pub name: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub language: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub platform: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub role: Option<String>,

    /// Owning team name (e.g. "platform", "growth").
    #[serde(skip_serializing_if = "Option::is_none")]
    pub team: Option<String>,

    /// On-call contact: a user handle, email, or PagerDuty service name.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub oncall: Option<String>,

    /// Portfolio-compatible: git submodule path that owns the source.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub submodule: Option<String>,

    /// Explicit filesystem path to the service root (overrides name-based matching).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,

    /// Path to the service's documentation file.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub docs: Option<String>,

    /// Path to the service's CI workflow file.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ci: Option<String>,

    /// Arbitrary labels for grouping and filtering (e.g. "critical", "beta").
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub tags: Vec<String>,

    /// Names of other services this service depends on (used for graph edges).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub depends_on: Vec<String>,
}

impl ServiceEntry {
    /// Returns the canonical relative path for existence checks.
    /// Prefers `path`, then `submodule`, then `None` (name-based matching).
    pub fn declared_path(&self) -> Option<&str> {
        self.path.as_deref().or(self.submodule.as_deref())
    }

    /// Look up a string metadata field by its manifest key name.
    ///
    /// This is the ONE place that maps a field *name* (as written in a policy
    /// file, a `require_fields` list, or a coverage table) onto the field's
    /// declared value. Every surface that asks "does this service declare
    /// `team`?" routes through here, so a field can never mean one thing to
    /// `svccat policy` and another to `svccat scorecard`.
    ///
    /// Returns `None` for a name this struct has no field for; callers decide
    /// whether an unknown name is an error or is simply ignored.
    pub fn field_value(&self, field: &str) -> Option<&str> {
        match field {
            "name" => Some(self.name.as_str()),
            "language" => self.language.as_deref(),
            "platform" => self.platform.as_deref(),
            "url" => self.url.as_deref(),
            "role" => self.role.as_deref(),
            "team" => self.team.as_deref(),
            "oncall" => self.oncall.as_deref(),
            "submodule" => self.submodule.as_deref(),
            "path" => self.path.as_deref(),
            "docs" => self.docs.as_deref(),
            "ci" => self.ci.as_deref(),
            _ => None,
        }
    }

    /// True when `field` is declared AND carries a non-empty value.
    ///
    /// **An empty string is not a value.** `team: ""` is treated exactly like
    /// an absent `team:`, because a field present-but-blank tells a reader of
    /// the catalog nothing, and crediting it lets a service pass an ownership
    /// requirement while naming no owner. This matches what `svccat lint` has
    /// always reported and what [`ServiceEntry::validate`] already enforces for
    /// the path-like fields, which it rejects outright when empty.
    pub fn has_field(&self, field: &str) -> bool {
        self.field_value(field).is_some_and(|v| !v.is_empty())
    }

    /// Validate the service entry for path traversal and other security issues.
    ///
    /// # Security
    /// Rejects paths that could escape the repo root (containing "..", absolute paths, etc)
    pub fn validate(&self) -> Result<()> {
        validate_optional_path(&self.path, "path")?;
        validate_optional_path(&self.submodule, "submodule")?;
        validate_optional_path(&self.docs, "docs")?;
        validate_optional_path(&self.ci, "ci")?;
        Ok(())
    }
}

/// Validate an optional relative path to prevent directory traversal attacks.
///
/// Rejects:
/// - Absolute paths (starting with "/" or "C:\")
/// - Paths containing ".." (parent directory traversal)
/// - Paths with null bytes
/// - Empty strings
fn validate_optional_path(path_opt: &Option<String>, field_name: &str) -> Result<()> {
    let path = match path_opt {
        Some(p) => p,
        None => return Ok(()),
    };

    if path.is_empty() {
        anyhow::bail!("{} field cannot be empty", field_name);
    }

    // Reject absolute paths
    if path.starts_with('/') || path.starts_with('\\') {
        anyhow::bail!(
            "{}: absolute paths not allowed (must be relative to repo root): {}",
            field_name,
            path
        );
    }

    // Reject parent directory traversal
    if path.contains("..") {
        anyhow::bail!(
            "{}: path traversal not allowed (contains '..'): {}",
            field_name,
            path
        );
    }

    // Reject null bytes
    if path.contains('\0') {
        anyhow::bail!("{}: path contains null bytes", field_name);
    }

    // Reject Windows drive letters (C:, D:, etc)
    if path.len() >= 2 && path.chars().nth(1) == Some(':') {
        anyhow::bail!(
            "{}: absolute paths (Windows drive letters) not allowed: {}",
            field_name,
            path
        );
    }

    Ok(())
}

// ── Helpers ───────────────────────────────────────────────────────────────────

fn default_version() -> String {
    "1".to_string()
}

/// Look for a manifest in `root`, trying common filenames.
pub fn find_default(root: &Path) -> PathBuf {
    for name in &["svccat.yaml", "svccat.yml", "services.yaml", "services.yml"] {
        let p = root.join(name);
        if p.exists() {
            return p;
        }
    }
    root.join("services.yaml")
}

#[cfg(test)]
mod tests {
    use super::ServiceEntry;

    /// Build an entry with every string field declared and non-empty.
    fn fully_populated() -> ServiceEntry {
        let mut svc = ServiceEntry {
            name: "alpha".to_string(),
            ..Default::default()
        };
        svc.language = Some("rust".to_string());
        svc.platform = Some("fly".to_string());
        svc.url = Some("https://alpha.example.com".to_string());
        svc.role = Some("api".to_string());
        svc.team = Some("platform".to_string());
        svc.oncall = Some("@alpha".to_string());
        svc.submodule = Some("vendor/alpha".to_string());
        svc.path = Some("services/alpha".to_string());
        svc.docs = Some("docs/alpha.md".to_string());
        svc.ci = Some(".github/workflows/alpha.yml".to_string());
        svc
    }

    /// Drift guard: reads the field-name list of EVERY surface that asks
    /// "is this field declared?" and checks each name against the one
    /// `field_value` map they all now route through.
    ///
    /// Without this, adding a field name to any of those lists without
    /// teaching `field_value` about it fails silently and in the worst
    /// direction: `field_value` returns `None`, `has_field` returns `false`,
    /// and the field reads as never-populated for every service in the
    /// catalog. Nothing errors, no test fails, and a coverage row or a
    /// completeness score simply drops.
    #[test]
    fn field_names_every_surface_checks_are_known() {
        let svc = fully_populated();

        let sources: &[(&str, Vec<&str>)] = &[
            (
                "scorecard::SCORED_FIELDS",
                crate::scorecard::SCORED_FIELDS.to_vec(),
            ),
            ("stats::FIELDS", crate::stats::FIELDS.to_vec()),
            (
                "policy::POLICY_FIELDS",
                crate::policy::POLICY_FIELDS.to_vec(),
            ),
            (
                "drift::REQUIRABLE_FIELDS",
                crate::drift::REQUIRABLE_FIELDS.to_vec(),
            ),
            (
                "drift::RECOMMENDED_FIELDS",
                crate::drift::RECOMMENDED_FIELDS
                    .iter()
                    .map(|(f, _)| *f)
                    .collect(),
            ),
        ];

        for (source, fields) in sources {
            assert!(
                !fields.is_empty(),
                "{source} is empty, so this guard would pass vacuously"
            );
            for field in fields {
                assert!(
                    svc.field_value(field).is_some(),
                    "{source} names '{field}', which ServiceEntry::field_value does not \
                     recognise; it would read as never-declared for every service"
                );
                assert!(
                    svc.has_field(field),
                    "{source} names '{field}', which is declared and non-empty on a fully \
                     populated entry yet has_field reports it unset"
                );
            }
        }
    }

    /// The predicate itself: declared-and-non-empty, not merely declared.
    #[test]
    fn an_empty_string_is_not_a_declared_field() {
        let mut svc = fully_populated();
        assert!(svc.has_field("team"));

        svc.team = Some(String::new());
        assert_eq!(svc.field_value("team"), Some(""));
        assert!(
            !svc.has_field("team"),
            "an empty `team: \"\"` must read exactly like an absent `team:`"
        );

        svc.team = None;
        assert_eq!(svc.field_value("team"), None);
        assert!(!svc.has_field("team"));
    }

    #[test]
    fn an_unknown_field_name_is_never_declared() {
        let svc = fully_populated();
        assert_eq!(svc.field_value("tier"), None);
        assert!(!svc.has_field("tier"));
    }

    /// `name` is a required `String`, not an `Option`, and must still answer
    /// the same question: policy files may require it.
    #[test]
    fn name_participates_in_the_same_map() {
        let mut svc = fully_populated();
        assert_eq!(svc.field_value("name"), Some("alpha"));
        assert!(svc.has_field("name"));

        svc.name = String::new();
        assert!(!svc.has_field("name"));
    }
}
