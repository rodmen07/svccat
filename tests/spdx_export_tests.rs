mod common;

use clap::Parser;
use common::{touch, write_manifest};
use serde_json::Value;
use svccat::cli::{Cli, Commands, ExportFormat, SnapshotAction};
use tempfile::TempDir;

// ── SBOM render pipeline ──────────────────────────────────────────────────────

#[test]
fn spdx_export_renders_catalog_with_dependencies() {
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
    team: platform
    depends_on:
      - auth-service
  - name: auth-service
    language: Python
    role: Authentication
    platform: Cloud Run
"#,
    );

    let m = svccat::manifest::Manifest::load(&root.join("services.yaml")).unwrap();
    let out = svccat::output::spdx::render_export(&m).unwrap();
    let v: Value = serde_json::from_str(&out).unwrap();

    assert_eq!(v["spdxVersion"], "SPDX-2.3");
    assert_eq!(v["packages"].as_array().unwrap().len(), 2);

    let relationships = v["relationships"].as_array().unwrap();
    let describes = relationships
        .iter()
        .filter(|r| r["relationshipType"] == "DESCRIBES")
        .count();
    assert_eq!(describes, 2);

    let depends: Vec<&Value> = relationships
        .iter()
        .filter(|r| r["relationshipType"] == "DEPENDS_ON")
        .collect();
    assert_eq!(depends.len(), 1);
    assert_eq!(depends[0]["spdxElementId"], "SPDXRef-Package-api-gateway");
    assert_eq!(
        depends[0]["relatedSpdxElement"],
        "SPDXRef-Package-auth-service"
    );
}

// ── Snapshot sidecar ──────────────────────────────────────────────────────────

#[test]
fn snapshot_sbom_sidecar_roundtrip() {
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
"#,
    );

    let m = svccat::manifest::Manifest::load(&root.join("services.yaml")).unwrap();
    let d = svccat::discovery::discover_services(root, &m);
    let report = svccat::drift::analyze(&m, &d, root);

    svccat::snapshot::save(root, "pre-migration", &m, &report).unwrap();
    let sidecar = svccat::snapshot::save_sbom(root, "pre-migration", &m).unwrap();
    assert!(sidecar.ends_with("pre-migration.spdx.json"));
    assert!(sidecar.exists());

    let text = std::fs::read_to_string(&sidecar).unwrap();
    let v: Value = serde_json::from_str(&text).unwrap();
    assert_eq!(v["spdxVersion"], "SPDX-2.3");

    svccat::snapshot::delete(root, "pre-migration").unwrap();
    assert!(!sidecar.exists());
}

// ── CLI surface ───────────────────────────────────────────────────────────────

#[test]
fn export_accepts_spdx_json_format() {
    let cli = Cli::try_parse_from(["svccat", "export", "--format", "spdx-json"]).unwrap();
    match cli.command {
        Commands::Export { format, .. } => assert_eq!(format, ExportFormat::SpdxJson),
        other => panic!("unexpected command: {:?}", other),
    }
}

#[test]
fn export_rejects_spdx_xml_format() {
    let err = Cli::try_parse_from(["svccat", "export", "--format", "spdx-xml"]).unwrap_err();
    assert_eq!(err.kind(), clap::error::ErrorKind::InvalidValue);
}

#[test]
fn snapshot_save_parses_sbom_flag() {
    let cli = Cli::try_parse_from(["svccat", "snapshot", "save", "x", "--sbom"]).unwrap();
    match cli.command {
        Commands::Snapshot {
            action: SnapshotAction::Save { sbom, .. },
        } => assert!(sbom),
        other => panic!("unexpected command: {:?}", other),
    }
}

#[test]
fn snapshot_save_defaults_sbom_off() {
    let cli = Cli::try_parse_from(["svccat", "snapshot", "save", "x"]).unwrap();
    match cli.command {
        Commands::Snapshot {
            action: SnapshotAction::Save { sbom, .. },
        } => assert!(!sbom),
        other => panic!("unexpected command: {:?}", other),
    }
}
