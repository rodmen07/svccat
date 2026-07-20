//! Integration tests for the `[reporting]` section of a workspace `svccat.toml`
//! (multi-repo slice 2).
//!
//! These exercise the three keys end to end through the real loader and the
//! real `analyze_workspace` path against on-disk fixtures, in the same style as
//! `workspace_integration_tests.rs`. Kept in a focused module of its own rather
//! than grown into the 2000-line `integration_test.rs`.

use std::fs;
use std::path::{Path, PathBuf};

use svccat::cli::OutputFormat;
use svccat::reporting;
use svccat::workspace;

/// Write a file, creating parent directories as needed.
fn write(root: &Path, rel: &str, content: &str) {
    let full = root.join(rel);
    fs::create_dir_all(full.parent().unwrap()).unwrap();
    fs::write(full, content).unwrap();
}

/// Build a single-repo workspace with a declared `api` service and an
/// undeclared `scratch` directory, returning the TempDir and config path.
fn scratch_workspace(reporting_section: &str) -> (tempfile::TempDir, PathBuf) {
    let dir = tempfile::TempDir::new().unwrap();
    let root = dir.path();

    write(
        root,
        "svccat.toml",
        &format!(
            r#"
[workspace]
repos = [{{ name = "api", path = "repo1" }}]
{reporting_section}
"#
        ),
    );

    write(
        root,
        "repo1/services.yaml",
        r#"
version: "1"
discovery:
  paths:
    - "services/*"
services:
  - name: api
    language: Rust
    role: api
    platform: Cloud Run
"#,
    );
    write(root, "repo1/services/api/Cargo.toml", "");
    write(root, "repo1/services/scratch/Cargo.toml", "");

    let config_path = root.join("svccat.toml");
    (dir, config_path)
}

// ── exclude_patterns ───────────────────────────────────────────────────────

#[test]
fn without_excludes_the_scratch_dir_is_discovered() {
    // Baseline: both api and scratch are discovered, so scratch shows up as an
    // undeclared service. This is the state the exclude pattern will change.
    let (_dir, config_path) = scratch_workspace("");
    let (config, root) = workspace::load_workspace_config(&config_path).unwrap();

    let report = workspace::analyze_workspace(&config, &root, &[], 1).unwrap();
    assert_eq!(report.total_discovered, 2, "api + scratch are discovered");
    assert!(
        report.total_warnings >= 1,
        "the undeclared scratch dir should warn"
    );
}

#[test]
fn exclude_patterns_flow_into_discovery_ignore_globs() {
    // The same fixture, but [reporting].exclude_patterns hides scratch. Because
    // the pattern is merged into the existing discovery ignore machinery, the
    // scratch directory is never discovered and its undeclared-service warning
    // disappears.
    let (_dir, config_path) = scratch_workspace(
        r#"
[reporting]
exclude_patterns = ["services/scratch"]
"#,
    );
    let (config, root) = workspace::load_workspace_config(&config_path).unwrap();

    let report = workspace::analyze_workspace(&config, &root, &[], 1).unwrap();
    assert_eq!(
        report.total_discovered, 1,
        "scratch is excluded, only api remains"
    );
    assert_eq!(
        report.total_warnings, 0,
        "no undeclared-service warning once scratch is excluded"
    );
}

#[test]
fn exclude_patterns_are_additive_with_cli_ignore() {
    // A CLI/root-config ignore (passed as extra_ignore) and a [reporting]
    // exclude both apply: neither source drops the other. Here the CLI ignore
    // hides api and the config exclude hides scratch, leaving nothing.
    let (_dir, config_path) = scratch_workspace(
        r#"
[reporting]
exclude_patterns = ["services/scratch"]
"#,
    );
    let (config, root) = workspace::load_workspace_config(&config_path).unwrap();

    let report =
        workspace::analyze_workspace(&config, &root, &["services/api".to_string()], 1).unwrap();
    assert_eq!(
        report.total_discovered, 0,
        "CLI ignore and config exclude both applied"
    );
}

// ── include_cross_repo_deps toggle ─────────────────────────────────────────

#[test]
fn cross_repo_deps_enabled_builds_the_dependency_summary() {
    // With the toggle on (the default), the dependency graph is built and a
    // summary is attached to the report.
    let config_path = PathBuf::from("tests/fixtures/workspace/svccat.toml");
    let (config, root) = workspace::load_workspace_config(&config_path).unwrap();
    assert!(config.reporting.include_cross_repo_deps, "default is on");

    let report = workspace::analyze_workspace(&config, &root, &[], 1).unwrap();
    assert!(
        report.dependency_summary.is_some(),
        "dependency analysis ran, so a summary is present"
    );
}

#[test]
fn cross_repo_deps_disabled_skips_the_dependency_work() {
    // With the toggle off, the graph is never built: there is no summary to
    // hide because the work never ran. `dependency_summary == None` is the
    // observable evidence that the graph build was skipped, not merely omitted
    // from rendering (a hidden-output design would still populate the summary).
    let config_path = PathBuf::from("tests/fixtures/workspace/svccat.toml");
    let (mut config, root) = workspace::load_workspace_config(&config_path).unwrap();
    config.reporting.include_cross_repo_deps = false;

    let report = workspace::analyze_workspace(&config, &root, &[], 1).unwrap();
    assert!(
        report.dependency_summary.is_none(),
        "no summary is produced when the toggle is off"
    );
    assert!(report.circular_dependencies.is_empty());
    assert!(report.unresolvable_dependencies.is_empty());

    // The per-repo drift results are unaffected: only the cross-repo work is
    // skipped, so the same repos are still analyzed.
    assert_eq!(report.repos.len(), 2);
    assert_eq!(report.total_declared, 4);
}

// ── format default ─────────────────────────────────────────────────────────

#[test]
fn configured_format_drives_resolution_but_cli_still_wins() {
    // A [reporting].format is parsed and, absent a --format flag, selected.
    let dir = tempfile::TempDir::new().unwrap();
    write(
        dir.path(),
        "svccat.toml",
        r#"
[workspace]
repos = [{ name = "api", path = "repo1" }]

[reporting]
format = "markdown"
"#,
    );
    let (config, _) = workspace::load_workspace_config(&dir.path().join("svccat.toml")).unwrap();
    assert_eq!(config.reporting.format, Some(OutputFormat::Markdown));

    // No --format: the configured value is used.
    assert_eq!(
        reporting::resolve_format(None, &config.reporting),
        OutputFormat::Markdown
    );
    // --format wins over the configured value.
    assert_eq!(
        reporting::resolve_format(Some(OutputFormat::Json), &config.reporting),
        OutputFormat::Json
    );
}

#[test]
fn absent_reporting_format_falls_back_to_terminal() {
    let dir = tempfile::TempDir::new().unwrap();
    write(
        dir.path(),
        "svccat.toml",
        r#"
[workspace]
repos = [{ name = "api", path = "repo1" }]
"#,
    );
    let (config, _) = workspace::load_workspace_config(&dir.path().join("svccat.toml")).unwrap();
    assert_eq!(config.reporting.format, None);
    assert_eq!(
        reporting::resolve_format(None, &config.reporting),
        OutputFormat::Terminal
    );
}
