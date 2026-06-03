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

#[derive(Debug, Clone, Serialize, Deserialize)]
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
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
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
