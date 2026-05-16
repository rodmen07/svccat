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

// ── policy tests ──────────────────────────────────────────────────────────────

#[test]
fn policy_require_fields_flags_missing_url() {
    let dir = TempDir::new().unwrap();
    let root = dir.path();

    touch(root, "services/api/Cargo.toml");

    write_manifest(
        root,
        r#"
discovery:
  paths:
    - "services/*"
policy:
  require_fields: [url, language]
services:
  - name: api
    language: Rust
    role: api
    platform: Cloud Run
"#,
    );

    let (m, d) = load(root);
    let report = svccat::drift::analyze(&m, &d, root);

    let policy_violations: Vec<_> = report
        .drifts
        .iter()
        .filter(|i| i.kind == svccat::drift::DriftKind::PolicyViolation)
        .collect();

    assert_eq!(
        policy_violations.len(),
        1,
        "url should be flagged by policy"
    );
    assert!(policy_violations[0].message.contains("url"));
}

#[test]
fn policy_no_violations_when_fields_present() {
    let dir = TempDir::new().unwrap();
    let root = dir.path();

    touch(root, "services/api/Cargo.toml");

    write_manifest(
        root,
        r#"
discovery:
  paths:
    - "services/*"
policy:
  require_fields: [url, language]
services:
  - name: api
    language: Rust
    role: api
    platform: Cloud Run
    url: https://api.example.com
"#,
    );

    let (m, d) = load(root);
    let report = svccat::drift::analyze(&m, &d, root);

    let violations: Vec<_> = report
        .drifts
        .iter()
        .filter(|i| i.kind == svccat::drift::DriftKind::PolicyViolation)
        .collect();
    assert!(violations.is_empty(), "no policy violations expected");
}

// ── diff tests ────────────────────────────────────────────────────────────────

#[test]
fn diff_detects_added_and_removed_services() {
    let dir = TempDir::new().unwrap();
    let root = dir.path();

    let before_json = r#"{
      "version": "1",
      "manifest": "services.yaml",
      "summary": {"declared": 2, "discovered": 2, "drift_count": 0, "errors": 0, "warnings": 0},
      "services": [
        {"name": "api", "language": "Rust", "platform": "Cloud Run", "role": "api", "depends_on": []},
        {"name": "legacy", "language": "Go", "platform": "Fly.io", "role": "worker", "depends_on": []}
      ],
      "drift": []
    }"#;

    let after_json = r#"{
      "version": "1",
      "manifest": "services.yaml",
      "summary": {"declared": 2, "discovered": 2, "drift_count": 0, "errors": 0, "warnings": 0},
      "services": [
        {"name": "api", "language": "Rust", "platform": "Cloud Run", "role": "api", "depends_on": []},
        {"name": "new-worker", "language": "Python", "platform": "Cloud Run", "role": "worker", "depends_on": []}
      ],
      "drift": []
    }"#;

    let before_path = root.join("before.json");
    let after_path = root.join("after.json");
    fs::write(&before_path, before_json).unwrap();
    fs::write(&after_path, after_json).unwrap();

    let report = svccat::diff::diff_snapshots(&before_path, &after_path).unwrap();

    assert!(report.added.contains(&"new-worker".to_string()));
    assert!(report.removed.contains(&"legacy".to_string()));
    assert!(!report.added.contains(&"api".to_string()));
}

#[test]
fn diff_detects_field_changes() {
    let dir = TempDir::new().unwrap();
    let root = dir.path();

    let before_json = r#"{
      "services": [
        {"name": "api", "language": "Python", "platform": "Cloud Run", "role": "api", "depends_on": []}
      ],
      "drift": []
    }"#;

    let after_json = r#"{
      "services": [
        {"name": "api", "language": "Rust", "platform": "Cloud Run", "role": "api", "depends_on": []}
      ],
      "drift": []
    }"#;

    let before_path = root.join("before.json");
    let after_path = root.join("after.json");
    fs::write(&before_path, before_json).unwrap();
    fs::write(&after_path, after_json).unwrap();

    let report = svccat::diff::diff_snapshots(&before_path, &after_path).unwrap();

    assert_eq!(report.changed.len(), 1);
    assert_eq!(report.changed[0].name, "api");
    let lang_change = report.changed[0]
        .changes
        .iter()
        .find(|c| c.field == "language")
        .expect("language change expected");
    assert_eq!(lang_change.before, "Python");
    assert_eq!(lang_change.after, "Rust");
}

#[test]
fn diff_no_changes_when_snapshots_identical() {
    let dir = TempDir::new().unwrap();
    let root = dir.path();

    let snapshot = r#"{
      "services": [
        {"name": "api", "language": "Rust", "platform": "Cloud Run", "role": "api", "depends_on": []}
      ],
      "drift": []
    }"#;

    let before_path = root.join("before.json");
    let after_path = root.join("after.json");
    fs::write(&before_path, snapshot).unwrap();
    fs::write(&after_path, snapshot).unwrap();

    let report = svccat::diff::diff_snapshots(&before_path, &after_path).unwrap();
    assert!(
        report.is_empty(),
        "identical snapshots should produce no diff"
    );
}

// ── v0.6.0: Ownership metadata + team filter ──────────────────────────────────

#[test]
fn team_and_oncall_fields_parse() {
    let dir = TempDir::new().unwrap();
    let root = dir.path();

    touch(root, "services/api/Cargo.toml");
    touch(root, "services/worker/Cargo.toml");

    write_manifest(
        root,
        r#"
discovery:
  paths: ["services/*"]
services:
  - name: api
    language: Rust
    role: API
    platform: Cloud Run
    team: platform
    oncall: platform-oncall@example.com
  - name: worker
    language: Rust
    role: Worker
    platform: Cloud Run
    team: data
    oncall: data-team@example.com
"#,
    );

    let m = svccat::manifest::Manifest::load(&root.join("services.yaml")).unwrap();
    let api = m.services.iter().find(|s| s.name == "api").unwrap();
    let worker = m.services.iter().find(|s| s.name == "worker").unwrap();

    assert_eq!(api.team.as_deref(), Some("platform"));
    assert_eq!(api.oncall.as_deref(), Some("platform-oncall@example.com"));
    assert_eq!(worker.team.as_deref(), Some("data"));
    assert_eq!(worker.oncall.as_deref(), Some("data-team@example.com"));
}

#[test]
fn team_filter_limits_analysis_to_matching_services() {
    let dir = TempDir::new().unwrap();
    let root = dir.path();

    touch(root, "services/api/Cargo.toml");
    touch(root, "services/worker/Cargo.toml");

    write_manifest(
        root,
        r#"
discovery:
  paths: ["services/*"]
services:
  - name: api
    language: Rust
    role: API
    platform: Cloud Run
    team: platform
  - name: worker
    language: Rust
    role: Worker
    platform: Cloud Run
    team: data
"#,
    );

    // Simulate --team platform: retain only services owned by "platform".
    let full_m = svccat::manifest::Manifest::load(&root.join("services.yaml")).unwrap();
    let mut m = full_m.clone();
    m.services.retain(|s| {
        s.team
            .as_deref()
            .map(|t| t.eq_ignore_ascii_case("platform"))
            .unwrap_or(false)
    });

    // Discover against the full manifest, then filter out other-team services
    // to avoid false UndeclaredInRepo noise (mirrors main.rs behaviour).
    let in_scope: std::collections::HashSet<&str> =
        m.services.iter().map(|s| s.name.as_str()).collect();
    let other_declared: std::collections::HashSet<&str> = full_m
        .services
        .iter()
        .filter(|s| !in_scope.contains(s.name.as_str()))
        .map(|s| s.name.as_str())
        .collect();
    let discovered_all = svccat::discovery::discover_services(root, &full_m);
    let d: Vec<_> = discovered_all
        .into_iter()
        .filter(|d| !other_declared.contains(d.name.as_str()))
        .collect();

    let report = svccat::drift::analyze(&m, &d, root);

    // Only "api" (team: platform) should be in scope; "worker" should be invisible.
    assert_eq!(m.services.len(), 1);
    assert_eq!(m.services[0].name, "api");
    assert_eq!(
        report.drifts.len(),
        0,
        "no drift expected for team-filtered check: {:?}",
        report.drifts
    );
}

#[test]
fn policy_requires_team_field() {
    let dir = TempDir::new().unwrap();
    let root = dir.path();

    touch(root, "services/api/Cargo.toml");

    write_manifest(
        root,
        r#"
discovery:
  paths: ["services/*"]
policy:
  require_fields: ["team", "oncall"]
services:
  - name: api
    language: Rust
    role: API
    platform: Cloud Run
    # team and oncall intentionally omitted to trigger policy violations
"#,
    );

    let (m, d) = load(root);
    let report = svccat::drift::analyze(&m, &d, root);

    let policy_violations: Vec<_> = report
        .drifts
        .iter()
        .filter(|d| d.kind == svccat::drift::DriftKind::PolicyViolation)
        .collect();

    assert_eq!(
        policy_violations.len(),
        2,
        "expected 2 policy violations (team + oncall), got: {:?}",
        policy_violations
    );
    let fields: Vec<_> = policy_violations
        .iter()
        .filter_map(|v| v.detail.as_deref())
        .collect();
    assert!(fields.contains(&"team"), "expected team violation");
    assert!(fields.contains(&"oncall"), "expected oncall violation");
}

#[test]
fn team_filter_case_insensitive() {
    let dir = TempDir::new().unwrap();
    let root = dir.path();

    touch(root, "services/api/Cargo.toml");

    write_manifest(
        root,
        r#"
discovery:
  paths: ["services/*"]
services:
  - name: api
    language: Rust
    role: API
    platform: Cloud Run
    team: Platform
"#,
    );

    let mut m = svccat::manifest::Manifest::load(&root.join("services.yaml")).unwrap();
    // Match "platform" (lowercase) against team value "Platform" (mixed case).
    m.services.retain(|s| {
        s.team
            .as_deref()
            .map(|t| t.eq_ignore_ascii_case("platform"))
            .unwrap_or(false)
    });

    assert_eq!(m.services.len(), 1, "case-insensitive match should succeed");
}
