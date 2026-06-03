use crate::deps_graph;
use crate::discovery;
use crate::drift;
use crate::manifest::Manifest;
use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

// ── Configuration ──────────────────────────────────────────────────────────

/// Configuration for a single repository in a workspace.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct RepositoryConfig {
    /// Human-readable name for the repository.
    pub name: String,

    /// Path to the repository root (relative to workspace config location).
    pub path: PathBuf,

    /// Path to the manifest file within the repo (relative to repo root).
    #[serde(default = "default_manifest_path")]
    pub manifest: PathBuf,

    /// Whether to include this repo in checks (default: true).
    #[serde(default = "default_enabled")]
    pub enabled: bool,
}

fn default_manifest_path() -> PathBuf {
    PathBuf::from("services.yaml")
}

fn default_enabled() -> bool {
    true
}

/// Workspace configuration containing multiple repositories.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct WorkspaceConfig {
    /// List of repositories in this workspace.
    pub repos: Vec<RepositoryConfig>,
}

// ── Analysis Results ───────────────────────────────────────────────────────

/// Result of analyzing a single repository.
#[derive(Debug, Clone, Serialize)]
pub struct RepositoryAnalysis {
    pub name: String,
    pub path: PathBuf,
    pub drift: drift::DriftReport,
}

/// Aggregated workspace drift report combining results from all repos.
#[derive(Debug, Clone, Serialize)]
pub struct WorkspaceDriftReport {
    /// Analyses for each repository (repo_name, analysis).
    pub repos: Vec<RepositoryAnalysis>,

    /// Total declared services across all repos.
    pub total_declared: usize,

    /// Total discovered services across all repos.
    pub total_discovered: usize,

    /// Total errors across all repos.
    pub total_errors: usize,

    /// Total warnings across all repos.
    pub total_warnings: usize,

    /// Dependency graph summary and analysis.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dependency_summary: Option<deps_graph::DependencySummary>,

    /// Circular dependencies detected in the workspace.
    #[serde(default)]
    pub circular_dependencies: Vec<deps_graph::CircularDependency>,

    /// Unresolvable dependencies detected in the workspace.
    #[serde(default)]
    pub unresolvable_dependencies: Vec<deps_graph::UnresolvableDependency>,
}

impl WorkspaceDriftReport {
    /// Check if any errors were found across all repos.
    pub fn has_errors(&self) -> bool {
        self.total_errors > 0
    }

    /// Check if any warnings were found across all repos.
    pub fn has_warnings(&self) -> bool {
        self.total_warnings > 0
    }
}

// ── Workspace Context ──────────────────────────────────────────────────────

/// Runtime context for workspace operations.
pub struct WorkspaceContext {
    /// Workspace configuration.
    pub config: WorkspaceConfig,

    /// Root directory where workspace config is located.
    pub workspace_root: PathBuf,

    /// Per-repo manifests and discovered services.
    pub analyses: Vec<RepositoryAnalysis>,
}

// ── Loading & Analysis ────────────────────────────────────────────────────

/// Load workspace configuration from a TOML file.
pub fn load_workspace_config(config_path: &Path) -> Result<(WorkspaceConfig, PathBuf)> {
    let workspace_root = config_path
        .parent()
        .ok_or_else(|| anyhow!("cannot determine workspace root from config path"))?
        .to_path_buf();

    let content = std::fs::read_to_string(config_path).map_err(|e| {
        anyhow!(
            "cannot read workspace config {}: {}",
            config_path.display(),
            e
        )
    })?;

    // Parse TOML and extract workspace section
    let toml: toml::Value =
        toml::from_str(&content).map_err(|e| anyhow!("cannot parse workspace config: {}", e))?;

    let workspace = toml
        .get("workspace")
        .ok_or_else(|| anyhow!("no [workspace] section in config"))?;

    let repos: Vec<RepositoryConfig> = workspace
        .get("repos")
        .and_then(|r| r.as_array())
        .ok_or_else(|| anyhow!("workspace.repos must be an array of tables"))?
        .iter()
        .enumerate()
        .map(|(idx, repo)| {
            let repo_table = repo
                .as_table()
                .ok_or_else(|| anyhow!("workspace.repos[{}] must be a table", idx))?;

            let name = repo_table
                .get("name")
                .and_then(|v| v.as_str())
                .ok_or_else(|| anyhow!("workspace.repos[{}].name is required", idx))?
                .to_string();

            let path = repo_table
                .get("path")
                .and_then(|v| v.as_str())
                .ok_or_else(|| anyhow!("workspace.repos[{}].path is required", idx))
                .map(PathBuf::from)?;

            let manifest = repo_table
                .get("manifest")
                .and_then(|v| v.as_str())
                .map(PathBuf::from)
                .unwrap_or_else(default_manifest_path);

            let enabled = repo_table
                .get("enabled")
                .and_then(|v| v.as_bool())
                .unwrap_or_else(default_enabled);

            Ok(RepositoryConfig {
                name,
                path,
                manifest,
                enabled,
            })
        })
        .collect::<Result<Vec<_>>>()?;

    if repos.is_empty() {
        return Err(anyhow!("workspace must have at least one repository"));
    }

    Ok((WorkspaceConfig { repos }, workspace_root))
}

/// Analyze a single repository.
fn analyze_repository(
    repo_config: &RepositoryConfig,
    workspace_root: &Path,
    extra_ignore: &[String],
    depth: u32,
) -> Result<RepositoryAnalysis> {
    // Resolve repo path relative to workspace root
    let repo_root = if repo_config.path.is_absolute() {
        repo_config.path.clone()
    } else {
        workspace_root.join(&repo_config.path)
    };

    if !repo_root.exists() {
        return Err(anyhow!(
            "repository '{}' path does not exist: {}",
            repo_config.name,
            repo_root.display()
        ));
    }

    // Resolve manifest path relative to repo root
    let manifest_path = repo_root.join(&repo_config.manifest);

    // Load manifest
    let manifest = Manifest::load(&manifest_path).map_err(|e| {
        anyhow!(
            "failed to load manifest for repository '{}': {}",
            repo_config.name,
            e
        )
    })?;

    // Discover services
    let discovered =
        discovery::discover_services_with_opts(&repo_root, &manifest, extra_ignore, depth);

    // Analyze drift
    let mut drift_report = drift::analyze(&manifest, &discovered, &repo_root);
    drift_report.manifest = manifest_path.display().to_string();

    Ok(RepositoryAnalysis {
        name: repo_config.name.clone(),
        path: repo_root,
        drift: drift_report,
    })
}

/// Load and analyze all repositories in a workspace.
pub fn analyze_workspace(
    config: &WorkspaceConfig,
    workspace_root: &Path,
    extra_ignore: &[String],
    depth: u32,
) -> Result<WorkspaceDriftReport> {
    let mut analyses = Vec::new();
    let mut total_declared = 0;
    let mut total_discovered = 0;
    let mut total_errors = 0;
    let mut total_warnings = 0;
    let mut manifests = Vec::new();

    // Analyze each enabled repository
    for repo_config in &config.repos {
        if !repo_config.enabled {
            eprintln!("⏭️  Skipping disabled repository: {}", repo_config.name);
            continue;
        }

        match analyze_repository(repo_config, workspace_root, extra_ignore, depth) {
            Ok(analysis) => {
                total_declared += analysis.drift.declared;
                total_discovered += analysis.drift.discovered;
                total_errors += analysis.drift.error_count();
                total_warnings += analysis.drift.warning_count();

                // Load manifest for dependency analysis
                let repo_root = workspace_root.join(&repo_config.path);
                let manifest_path = repo_root.join(&repo_config.manifest);
                if let Ok(manifest) = Manifest::load(&manifest_path) {
                    manifests.push((repo_config.name.clone(), manifest));
                }

                analyses.push(analysis);
            }
            Err(e) => {
                eprintln!(
                    "❌ Error analyzing repository '{}': {}",
                    repo_config.name, e
                );
                return Err(e);
            }
        }
    }

    // Analyze cross-repo dependencies
    let mut dependency_summary = None;
    let mut circular_dependencies = Vec::new();
    let mut unresolvable_dependencies = Vec::new();

    if !manifests.is_empty() {
        let manifest_refs: Vec<(String, &Manifest)> = manifests
            .iter()
            .map(|(name, manifest)| (name.clone(), manifest))
            .collect();

        match deps_graph::DependencyGraph::build(manifest_refs) {
            Ok(graph) => {
                dependency_summary = Some(graph.summary());
                circular_dependencies = graph.circular_dependencies.clone();
                unresolvable_dependencies = graph.validate_all_dependencies();
            }
            Err(e) => {
                eprintln!("⚠️  Warning: Failed to analyze dependencies: {}", e);
            }
        }
    }

    Ok(WorkspaceDriftReport {
        repos: analyses,
        total_declared,
        total_discovered,
        total_errors,
        total_warnings,
        dependency_summary,
        circular_dependencies,
        unresolvable_dependencies,
    })
}

// ── Utilities ──────────────────────────────────────────────────────────────

/// Find workspace configuration starting from a given directory.
/// Searches for svccat.toml and checks if it contains a [workspace] section.
pub fn find_workspace_config(root: &Path) -> Option<PathBuf> {
    let config_file = root.join("svccat.toml");

    if config_file.exists() {
        if let Ok(content) = std::fs::read_to_string(&config_file) {
            if let Ok(toml) = content.parse::<toml::Value>() {
                if toml.get("workspace").is_some() {
                    return Some(config_file);
                }
            }
        }
    }

    None
}
