use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

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
    pub fn load(path: &Path) -> Result<Self> {
        let text = std::fs::read_to_string(path)
            .with_context(|| format!("cannot read manifest: {}", path.display()))?;
        let manifest: Self = serde_yaml::from_str(&text)
            .with_context(|| format!("cannot parse manifest: {}", path.display()))?;
        Ok(manifest)
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
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceEntry {
    pub name: String,

    pub language: Option<String>,
    pub platform: Option<String>,
    pub url: Option<String>,
    pub role: Option<String>,

    /// Owning team name (e.g. "platform", "growth").
    pub team: Option<String>,

    /// On-call contact: a user handle, email, or PagerDuty service name.
    pub oncall: Option<String>,

    /// Portfolio-compatible: git submodule path that owns the source.
    pub submodule: Option<String>,

    /// Explicit filesystem path to the service root (overrides name-based matching).
    pub path: Option<String>,

    /// Path to the service's documentation file.
    pub docs: Option<String>,

    /// Path to the service's CI workflow file.
    pub ci: Option<String>,

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
