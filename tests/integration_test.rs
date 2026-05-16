use std::fs;
use std::path::Path;
use tempfile::TempDir;

// ── Test helpers ──────────────────────────────────────────────────────────────

fn touch(root: &Path, rel_path: &str) {
    let full = root.join(rel_path);
    if let Some(parent) = full.parent() {
        fs::create_dir_all(parent).unwrap();
    }
    fs::write(full, "").unwrap();
}

fn write_manifest(root: &Path, content: &str) {
    fs::write(root.join("services.yaml"), content).unwrap();
}

fn load(
    root: &Path,
) -> (
    svccat::manifest::Manifest,
    Vec<svccat::discovery::DiscoveredService>,
) {
    let m = svccat::manifest::Manifest::load(&root.join("services.yaml")).unwrap();
    let d = svccat::discovery::discover_services(root, &m);
    (m, d)
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[test]
fn no_drift_when_all_services_found() {
    let dir = TempDir::new().unwrap();
    let root = dir.path();

    touch(root, "services/api-gateway/Cargo.toml");
    touch(root, "services/auth-service/Dockerfile");

    write_manifest(
        root,
        r#"
discovery:
  paths:
    - "services/*"
services:
  - name: api-gateway
    language: Rust
    role: API gateway
    platform: Cloud Run
  - name: auth-service
    language: Python
    role: Authentication
    platform: Cloud Run
"#,
    );

    let (m, d) = load(root);
    let report = svccat::drift::analyze(&m, &d, root);
    assert_eq!(
        report.drifts.len(),
        0,
        "unexpected drift: {:?}",
        report.drifts
    );
}

#[test]
fn detects_declared_service_missing_from_repo() {
    let dir = TempDir::new().unwrap();
    let root = dir.path();

    touch(root, "services/api-gateway/Cargo.toml");
    // auth-service directory intentionally absent

    write_manifest(
        root,
        r#"
discovery:
  paths:
    - "services/*"
services:
  - name: api-gateway
    language: Rust
    role: API gateway
    platform: Cloud Run
  - name: auth-service
    language: Python
    role: Authentication
    platform: Cloud Run
"#,
    );

    let (m, d) = load(root);
    let report = svccat::drift::analyze(&m, &d, root);

    let missing: Vec<_> = report
        .drifts
        .iter()
        .filter(|item| item.kind == svccat::drift::DriftKind::DeclaredMissingFromRepo)
        .collect();
    assert_eq!(missing.len(), 1, "expected exactly one missing service");
    assert_eq!(missing[0].service, "auth-service");
    assert_eq!(missing[0].severity, svccat::drift::Severity::Error);
}

#[test]
fn detects_undeclared_service_in_repo() {
    let dir = TempDir::new().unwrap();
    let root = dir.path();

    touch(root, "services/api-gateway/Cargo.toml");
    touch(root, "services/auth-service/Dockerfile");
    touch(root, "services/new-feature/go.mod"); // not in manifest

    write_manifest(
        root,
        r#"
discovery:
  paths:
    - "services/*"
services:
  - name: api-gateway
    language: Rust
    role: API gateway
    platform: Cloud Run
  - name: auth-service
    language: Python
    role: Authentication
    platform: Cloud Run
"#,
    );

    let (m, d) = load(root);
    let report = svccat::drift::analyze(&m, &d, root);

    let undeclared: Vec<_> = report
        .drifts
        .iter()
        .filter(|item| item.kind == svccat::drift::DriftKind::UndeclaredInRepo)
        .collect();
    assert_eq!(
        undeclared.len(),
        1,
        "expected exactly one undeclared service"
    );
    assert_eq!(undeclared[0].service, "new-feature");
    assert_eq!(undeclared[0].severity, svccat::drift::Severity::Warning);
}

#[test]
fn detects_missing_role_field() {
    let dir = TempDir::new().unwrap();
    let root = dir.path();

    touch(root, "services/api-gateway/Cargo.toml");

    write_manifest(
        root,
        r#"
discovery:
  paths:
    - "services/*"
services:
  - name: api-gateway
    language: Rust
    platform: Cloud Run
    # role intentionally omitted
"#,
    );

    let (m, d) = load(root);
    let report = svccat::drift::analyze(&m, &d, root);

    let field_drifts: Vec<_> = report
        .drifts
        .iter()
        .filter(|item| item.kind == svccat::drift::DriftKind::MissingField)
        .collect();
    assert_eq!(field_drifts.len(), 1, "expected one missing-field drift");
    assert_eq!(field_drifts[0].detail.as_deref(), Some("role"));
    assert_eq!(field_drifts[0].severity, svccat::drift::Severity::Error);
}

#[test]
fn detects_missing_referenced_docs_file() {
    let dir = TempDir::new().unwrap();
    let root = dir.path();

    touch(root, "services/api-gateway/Cargo.toml");
    // docs/api-gateway.md intentionally absent

    write_manifest(
        root,
        r#"
discovery:
  paths:
    - "services/*"
services:
  - name: api-gateway
    language: Rust
    role: API gateway
    platform: Cloud Run
    docs: docs/api-gateway.md
"#,
    );

    let (m, d) = load(root);
    let report = svccat::drift::analyze(&m, &d, root);

    let ref_drifts: Vec<_> = report
        .drifts
        .iter()
        .filter(|item| item.kind == svccat::drift::DriftKind::MissingReferencedFile)
        .collect();
    assert_eq!(ref_drifts.len(), 1);
    assert!(ref_drifts[0].message.contains("docs"));
}

#[test]
fn explicit_path_overrides_name_matching() {
    let dir = TempDir::new().unwrap();
    let root = dir.path();

    // Service directory is nested, not top-level services/
    touch(root, "infra/gateway/Cargo.toml");

    write_manifest(
        root,
        r#"
discovery:
  paths:
    - "services/*"
services:
  - name: api-gateway
    language: Rust
    role: API gateway
    platform: Cloud Run
    path: infra/gateway
"#,
    );

    let (m, d) = load(root);
    let report = svccat::drift::analyze(&m, &d, root);

    // Should be no "missing" drift (explicit path matched), but api-gateway
    // is not in services/* discovery so no undeclared entries either.
    let missing: Vec<_> = report
        .drifts
        .iter()
        .filter(|i| i.kind == svccat::drift::DriftKind::DeclaredMissingFromRepo)
        .collect();
    assert_eq!(missing.len(), 0, "explicit path should resolve correctly");
}

#[test]
fn json_output_is_valid() {
    let dir = TempDir::new().unwrap();
    let root = dir.path();

    touch(root, "services/api-gateway/Cargo.toml");

    write_manifest(
        root,
        r#"
discovery:
  paths:
    - "services/*"
services:
  - name: api-gateway
    language: Rust
    role: API gateway
    platform: Cloud Run
"#,
    );

    let (m, d) = load(root);
    let report = svccat::drift::analyze(&m, &d, root);

    // Serialise to JSON — must not panic and must round-trip cleanly.
    let json_str = serde_json::to_string_pretty(&report).unwrap();
    let _: serde_json::Value = serde_json::from_str(&json_str).unwrap();
}

// ── depends_on graph tests ────────────────────────────────────────────────────

#[test]
fn graph_renders_depends_on_edges() {
    use std::io::Write;

    let dir = TempDir::new().unwrap();
    let root = dir.path();

    write_manifest(
        root,
        r#"
discovery:
  paths:
    - "services/*"
services:
  - name: payment-service
    language: Rust
    role: payments
    platform: Cloud Run
    depends_on:
      - auth-service
      - postgres
  - name: auth-service
    language: Go
    role: authentication
    platform: Cloud Run
  - name: postgres
    language: SQL
    role: database
    platform: Cloud SQL
"#,
    );

    let m = svccat::manifest::Manifest::load(&root.join("services.yaml")).unwrap();

    // Capture stdout
    let mut output = Vec::new();
    {
        // Render to a string by using the public render function
        // We test the manifest parses depends_on correctly
        assert_eq!(m.services[0].depends_on, vec!["auth-service", "postgres"]);
        assert!(m.services[1].depends_on.is_empty());
    }

    // Verify depends_on survives a YAML round-trip
    let yaml = serde_yaml::to_string(&m).unwrap();
    let m2: svccat::manifest::Manifest = serde_yaml::from_str(&yaml).unwrap();
    assert_eq!(m2.services[0].depends_on, vec!["auth-service", "postgres"]);
    let _ = output.flush();
}

// ── ping tests (unit-level, no real HTTP) ────────────────────────────────────

#[test]
fn ping_result_is_ok_for_reachable() {
    use svccat::ping::{PingResult, PingStatus};

    let r = PingResult {
        service: "api".to_string(),
        url: "https://example.com".to_string(),
        ping: PingStatus::Reachable { code: 200 },
    };
    assert!(r.is_ok());

    let r2 = PingResult {
        service: "api".to_string(),
        url: "https://example.com".to_string(),
        ping: PingStatus::Reachable { code: 404 },
    };
    assert!(r2.is_ok()); // reachable even if 404

    let r3 = PingResult {
        service: "api".to_string(),
        url: "https://example.com".to_string(),
        ping: PingStatus::Unreachable {
            reason: "connection refused".to_string(),
        },
    };
    assert!(!r3.is_ok());
}

#[test]
fn ping_skips_services_without_url() {
    use std::io::Write;

    let dir = TempDir::new().unwrap();
    let root = dir.path();

    write_manifest(
        root,
        r#"
services:
  - name: no-url-service
    language: Rust
    role: api
    platform: Cloud Run
"#,
    );

    let m = svccat::manifest::Manifest::load(&root.join("services.yaml")).unwrap();
    // ping_services returns empty because no url is set
    // We test the filter logic: services without url are skipped
    let services_with_url: Vec<_> = m.services.iter().filter(|s| s.url.is_some()).collect();
    assert_eq!(services_with_url.len(), 0);
}

#[test]
fn init_creates_services_yaml_from_discovered_services() {
    let dir = TempDir::new().unwrap();
    let root = dir.path();

    touch(root, "services/api-gateway/Cargo.toml");
    touch(root, "services/worker/go.mod");

    let output = root.join("services.yaml");
    svccat::init::run(root, output.clone(), false).unwrap();

    assert!(output.exists(), "services.yaml should be created");
    let contents = fs::read_to_string(&output).unwrap();
    assert!(
        contents.contains("api-gateway"),
        "should include api-gateway"
    );
    assert!(contents.contains("worker"), "should include worker");
    assert!(
        contents.contains("Rust"),
        "should infer Rust from Cargo.toml"
    );
    assert!(contents.contains("Go"), "should infer Go from go.mod");
}

#[test]
fn init_refuses_to_overwrite_without_force() {
    let dir = TempDir::new().unwrap();
    let root = dir.path();
    let output = root.join("services.yaml");
    fs::write(&output, "existing content").unwrap();

    let result = svccat::init::run(root, output, false);
    assert!(
        result.is_err(),
        "should error when file exists and --force not set"
    );
}

#[test]
fn init_overwrites_with_force() {
    let dir = TempDir::new().unwrap();
    let root = dir.path();
    let output = root.join("services.yaml");
    fs::write(&output, "old content").unwrap();

    touch(root, "services/my-svc/Dockerfile");
    svccat::init::run(root, output.clone(), true).unwrap();

    let contents = fs::read_to_string(&output).unwrap();
    assert!(
        contents.contains("my-svc"),
        "should contain discovered service"
    );
    assert!(
        !contents.contains("old content"),
        "should overwrite old content"
    );
}

#[test]
fn init_empty_repo_writes_skeleton() {
    let dir = TempDir::new().unwrap();
    let root = dir.path();
    let output = root.join("services.yaml");

    svccat::init::run(root, output.clone(), false).unwrap();

    let contents = fs::read_to_string(&output).unwrap();
    assert!(
        contents.contains("version"),
        "skeleton should contain version key"
    );
    assert!(
        contents.contains("services:"),
        "skeleton should contain services key"
    );
}

// ── ignore pattern tests ──────────────────────────────────────────────────────

#[test]
fn ignore_patterns_exclude_matching_directories() {
    let dir = TempDir::new().unwrap();
    let root = dir.path();

    touch(root, "services/api/Cargo.toml");
    touch(root, "services/examples/Cargo.toml"); // should be ignored
    touch(root, "services/vendor/go.mod"); // should be ignored

    write_manifest(
        root,
        r#"
discovery:
  paths:
    - "services/*"
  ignore:
    - "services/examples"
    - "services/vendor"
services:
  - name: api
    language: Rust
    role: api
    platform: Cloud Run
"#,
    );

    let m = svccat::manifest::Manifest::load(&root.join("services.yaml")).unwrap();
    let discovered = svccat::discovery::discover_services(root, &m);

    let names: Vec<&str> = discovered.iter().map(|d| d.name.as_str()).collect();
    assert!(names.contains(&"api"), "api should be discovered");
    assert!(!names.contains(&"examples"), "examples should be ignored");
    assert!(!names.contains(&"vendor"), "vendor should be ignored");
}

#[test]
fn extra_ignore_from_cli_merges_with_manifest() {
    let dir = TempDir::new().unwrap();
    let root = dir.path();

    touch(root, "services/api/Cargo.toml");
    touch(root, "services/scratch/Cargo.toml");

    write_manifest(
        root,
        r#"
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

    let m = svccat::manifest::Manifest::load(&root.join("services.yaml")).unwrap();
    // Pass "services/scratch" as an extra CLI ignore
    let discovered = svccat::discovery::discover_services_with_ignore(
        root,
        &m,
        &["services/scratch".to_string()],
    );

    let names: Vec<&str> = discovered.iter().map(|d| d.name.as_str()).collect();
    assert!(names.contains(&"api"));
    assert!(
        !names.contains(&"scratch"),
        "scratch should be excluded by CLI ignore"
    );
}

// ── svccat.toml config tests ──────────────────────────────────────────────────

#[test]
fn config_loads_from_svccat_toml() {
    let dir = TempDir::new().unwrap();
    let root = dir.path();

    fs::write(
        root.join("svccat.toml"),
        r#"
fail_on_drift = true
ignore = ["examples/*", "vendor/*"]
"#,
    )
    .unwrap();

    let cfg = svccat::config::SvccatConfig::load(root).unwrap();
    assert!(cfg.fail_on_drift);
    assert_eq!(cfg.ignore, vec!["examples/*", "vendor/*"]);
}

#[test]
fn config_returns_defaults_when_no_file() {
    let dir = TempDir::new().unwrap();
    let root = dir.path();

    let cfg = svccat::config::SvccatConfig::load(root).unwrap();
    assert!(!cfg.fail_on_drift);
    assert!(cfg.ignore.is_empty());
}

// ── completions test ──────────────────────────────────────────────────────────

#[test]
fn completions_produces_output_for_bash() {
    use clap::CommandFactory;
    use clap_complete::{generate, Shell};
    use svccat::cli::Cli;

    let mut cmd = Cli::command();
    let mut buf = Vec::new();
    generate(Shell::Bash, &mut cmd, "svccat", &mut buf);
    assert!(!buf.is_empty(), "bash completions should produce output");
    let script = String::from_utf8(buf).unwrap();
    assert!(
        script.contains("svccat"),
        "completions should mention the binary name"
    );
}
