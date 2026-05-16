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

fn load(root: &Path) -> (svccat::manifest::Manifest, Vec<svccat::discovery::DiscoveredService>) {
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
    assert_eq!(report.drifts.len(), 0, "unexpected drift: {:?}", report.drifts);
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
    assert_eq!(undeclared.len(), 1, "expected exactly one undeclared service");
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
