use std::path::PathBuf;
use svccat::workspace;

#[test]
fn test_workspace_config_loading() {
    let config_path = PathBuf::from("tests/fixtures/workspace/svccat.toml");

    let (workspace_config, workspace_root) =
        workspace::load_workspace_config(&config_path).expect("Failed to load workspace config");

    // Verify workspace has 2 repos
    assert_eq!(workspace_config.repos.len(), 2);

    // Verify first repo
    assert_eq!(workspace_config.repos[0].name, "backend");
    assert_eq!(workspace_config.repos[0].path, PathBuf::from("repo1"));
    assert_eq!(
        workspace_config.repos[0].manifest,
        PathBuf::from("services.yaml")
    );
    assert!(workspace_config.repos[0].enabled);

    // Verify second repo
    assert_eq!(workspace_config.repos[1].name, "frontend");
    assert_eq!(workspace_config.repos[1].path, PathBuf::from("repo2"));
    assert!(workspace_config.repos[1].enabled);

    // Verify workspace root is correct
    assert!(workspace_root.ends_with("tests/fixtures/workspace"));
}

#[test]
fn test_workspace_analysis() {
    let config_path = PathBuf::from("tests/fixtures/workspace/svccat.toml");

    let (workspace_config, workspace_root) =
        workspace::load_workspace_config(&config_path).expect("Failed to load workspace config");

    let report = workspace::analyze_workspace(&workspace_config, &workspace_root, &[], 1)
        .expect("Failed to analyze workspace");

    // Verify we analyzed both repos
    assert_eq!(report.repos.len(), 2);

    // Backend repo: 2 services declared (api, auth)
    assert_eq!(report.repos[0].name, "backend");
    assert_eq!(report.repos[0].drift.declared, 2);
    assert_eq!(report.repos[0].drift.discovered, 2); // Both api and auth dirs exist

    // Frontend repo: 2 services declared (web, admin-panel)
    assert_eq!(report.repos[1].name, "frontend");
    assert_eq!(report.repos[1].drift.declared, 2);
    assert_eq!(report.repos[1].drift.discovered, 2); // Both web and admin-panel dirs exist

    // Total aggregation
    assert_eq!(report.total_declared, 4);
    assert_eq!(report.total_discovered, 4);
    assert_eq!(report.total_errors, 0); // No drift - all declared services exist
}

#[test]
fn test_workspace_drift_detection() {
    let config_path = PathBuf::from("tests/fixtures/workspace/svccat.toml");

    let (workspace_config, workspace_root) =
        workspace::load_workspace_config(&config_path).expect("Failed to load workspace config");

    let report = workspace::analyze_workspace(&workspace_config, &workspace_root, &[], 1)
        .expect("Failed to analyze workspace");

    // Verify no errors (all services are in sync)
    assert!(
        !report.has_errors(),
        "Should have no errors when all services exist"
    );

    // Verify the aggregated metrics are correct
    assert_eq!(report.total_declared, 4);
    assert_eq!(report.total_discovered, 4);
}

#[test]
fn test_workspace_disabled_repo() {
    // Create a config with one disabled repo
    let config = svccat::workspace::WorkspaceConfig {
        name: None,
        description: None,
        repos: vec![
            svccat::workspace::RepositoryConfig {
                name: "backend".to_string(),
                path: PathBuf::from("repo1"),
                manifest: PathBuf::from("services.yaml"),
                enabled: true,
            },
            svccat::workspace::RepositoryConfig {
                name: "disabled".to_string(),
                path: PathBuf::from("repo2"),
                manifest: PathBuf::from("services.yaml"),
                enabled: false, // This repo should be skipped
            },
        ],
    };

    let workspace_root = PathBuf::from("tests/fixtures/workspace");
    let report = svccat::workspace::analyze_workspace(&config, &workspace_root, &[], 1)
        .expect("Failed to analyze workspace");

    // Only the backend repo should be analyzed
    assert_eq!(report.repos.len(), 1);
    assert_eq!(report.repos[0].name, "backend");
    assert_eq!(report.total_declared, 2);
}

#[test]
fn test_workspace_find_config() {
    let workspace_root = PathBuf::from("tests/fixtures/workspace");

    let config_path = workspace::find_workspace_config(&workspace_root);

    assert!(config_path.is_some(), "Should find workspace config");
    assert!(config_path.unwrap().ends_with("svccat.toml"));
}

#[test]
fn test_workspace_fixture_has_no_metadata() {
    // The shared fixture declares no workspace name or description, so both
    // parse as None and the report carries no workspace name.
    let config_path = PathBuf::from("tests/fixtures/workspace/svccat.toml");

    let (workspace_config, workspace_root) =
        workspace::load_workspace_config(&config_path).expect("Failed to load workspace config");

    assert_eq!(workspace_config.name, None);
    assert_eq!(workspace_config.description, None);

    let report = workspace::analyze_workspace(&workspace_config, &workspace_root, &[], 1)
        .expect("Failed to analyze workspace");
    assert_eq!(report.workspace_name, None);

    let json = svccat::output::workspace::render_json(&report).expect("Failed to render JSON");
    let parsed: serde_json::Value = serde_json::from_str(&json).expect("Invalid JSON");
    assert!(parsed["workspace_name"].is_null());
}

#[test]
fn test_workspace_name_propagates_to_report_and_renderers() {
    let config_path = PathBuf::from("tests/fixtures/workspace/svccat.toml");

    let (mut workspace_config, workspace_root) =
        workspace::load_workspace_config(&config_path).expect("Failed to load workspace config");
    workspace_config.name = Some("Fixture Platform".to_string());

    let report = workspace::analyze_workspace(&workspace_config, &workspace_root, &[], 1)
        .expect("Failed to analyze workspace");
    assert_eq!(report.workspace_name.as_deref(), Some("Fixture Platform"));

    let json = svccat::output::workspace::render_json(&report).expect("Failed to render JSON");
    let parsed: serde_json::Value = serde_json::from_str(&json).expect("Invalid JSON");
    assert_eq!(parsed["workspace_name"], "Fixture Platform");

    let md = svccat::output::workspace::render_markdown(&report);
    assert!(
        md.contains("**Workspace:** Fixture Platform"),
        "markdown should include the workspace name: {md}"
    );
}

#[test]
fn test_workspace_filter_restricts_analysis() {
    let config_path = PathBuf::from("tests/fixtures/workspace/svccat.toml");

    let (workspace_config, workspace_root) =
        workspace::load_workspace_config(&config_path).expect("Failed to load workspace config");

    let filtered =
        workspace::filter_repos(&workspace_config, "frontend").expect("Failed to filter repos");
    assert_eq!(filtered.repos.len(), 1);

    let report = workspace::analyze_workspace(&filtered, &workspace_root, &[], 1)
        .expect("Failed to analyze workspace");

    assert_eq!(report.repos.len(), 1);
    assert_eq!(report.repos[0].name, "frontend");
    assert_eq!(report.total_declared, 2);
    assert_eq!(report.total_discovered, 2);
}

#[test]
fn test_workspace_filter_unknown_name_is_an_error() {
    let config_path = PathBuf::from("tests/fixtures/workspace/svccat.toml");

    let (workspace_config, _) =
        workspace::load_workspace_config(&config_path).expect("Failed to load workspace config");

    let err = workspace::filter_repos(&workspace_config, "backend,missing-repo")
        .expect_err("Unknown repo name should be rejected");
    let msg = err.to_string();
    assert!(
        msg.contains("missing-repo"),
        "error should name the unknown repo: {msg}"
    );
    assert!(
        msg.contains("backend") && msg.contains("frontend"),
        "error should list available repos: {msg}"
    );
}

#[test]
fn test_workspace_filter_does_not_override_disabled() {
    // Selecting a disabled repo with --filter keeps it in the selection but
    // analysis still skips it: enabled = false always wins.
    let config = svccat::workspace::WorkspaceConfig {
        name: None,
        description: None,
        repos: vec![
            svccat::workspace::RepositoryConfig {
                name: "backend".to_string(),
                path: PathBuf::from("repo1"),
                manifest: PathBuf::from("services.yaml"),
                enabled: true,
            },
            svccat::workspace::RepositoryConfig {
                name: "disabled".to_string(),
                path: PathBuf::from("repo2"),
                manifest: PathBuf::from("services.yaml"),
                enabled: false,
            },
        ],
    };

    let filtered = svccat::workspace::filter_repos(&config, "backend,disabled")
        .expect("Failed to filter repos");
    assert_eq!(filtered.repos.len(), 2, "filter selects both repos");

    let workspace_root = PathBuf::from("tests/fixtures/workspace");
    let report = svccat::workspace::analyze_workspace(&filtered, &workspace_root, &[], 1)
        .expect("Failed to analyze workspace");

    assert_eq!(
        report.repos.len(),
        1,
        "analysis still skips the disabled repo"
    );
    assert_eq!(report.repos[0].name, "backend");
}
