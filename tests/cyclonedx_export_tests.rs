mod common;

use clap::Parser;
use common::{touch, write_manifest};
use serde_json::Value;
use svccat::cli::{Cli, Commands, ExportFormat};
use tempfile::TempDir;

// ── SBOM render pipeline ──────────────────────────────────────────────────────
//
// Mirrors `tests/spdx_export_tests.rs::spdx_export_renders_catalog_with_dependencies`:
// same discovered manifest, same assertions translated to CycloneDX's shape
// (components/dependencies instead of packages/relationships).

#[test]
fn cyclonedx_export_renders_catalog_with_dependencies() {
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
    let out = svccat::output::cyclonedx::render_export(&m).unwrap();
    let v: Value = serde_json::from_str(&out).unwrap();

    assert_eq!(v["bomFormat"], "CycloneDX");
    assert_eq!(v["specVersion"], "1.7");
    assert_eq!(v["components"].as_array().unwrap().len(), 2);

    // Every component gets a dependency-graph entry (declared explicitly,
    // per CycloneDX's own recommendation), so there are 2 entries even
    // though only one edge exists.
    let dependencies = v["dependencies"].as_array().unwrap();
    assert_eq!(dependencies.len(), 2);

    let gateway_dep = dependencies
        .iter()
        .find(|d| d["ref"] == "component-api-gateway")
        .unwrap();
    let depends_on = gateway_dep["dependsOn"].as_array().unwrap();
    assert_eq!(depends_on.len(), 1);
    assert_eq!(depends_on[0], "component-auth-service");

    let auth_dep = dependencies
        .iter()
        .find(|d| d["ref"] == "component-auth-service")
        .unwrap();
    assert!(auth_dep.get("dependsOn").is_none());
}

// ── CLI surface ───────────────────────────────────────────────────────────────
//
// Mirrors `tests/spdx_export_tests.rs`'s CLI-surface tests: the CycloneDX
// value is a sibling of `spdx-json` on the same `ExportFormat` enum, wired
// through the same `export --format` flag rather than a new subcommand.

#[test]
fn export_accepts_cyclonedx_json_format() {
    let cli = Cli::try_parse_from(["svccat", "export", "--format", "cyclonedx-json"]).unwrap();
    match cli.command {
        Commands::Export { format, .. } => assert_eq!(format, ExportFormat::CyclonedxJson),
        other => panic!("unexpected command: {:?}", other),
    }
}

#[test]
fn export_rejects_cyclonedx_xml_format() {
    let err = Cli::try_parse_from(["svccat", "export", "--format", "cyclonedx-xml"]).unwrap_err();
    assert_eq!(err.kind(), clap::error::ErrorKind::InvalidValue);
}
