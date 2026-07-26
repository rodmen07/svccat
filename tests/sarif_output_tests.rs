//! Binary-level coverage for `svccat check --format sarif`.
//!
//! `src/output/sarif.rs` shipped in the first release of this format and had
//! never had a test of any shape: no `mod tests`, and no integration test
//! referenced it. This file is the binary half of its first coverage (the
//! unit half lives in `src/output/sarif.rs`), following the
//! `tests/cli_binary_tests.rs` precedent: spawn the real compiled binary and
//! assert on its real stdout, so a regression in the *wiring* (a match arm
//! falling through, the wrong variable threaded into the render call, the
//! document computed but never printed) fails here even though every
//! in-process unit test would keep passing.
//!
//! The `--ping` test deliberately uses an address the SSRF validator rejects
//! before any socket is opened, so it needs no network, no listener, and no
//! free-port dance: it is the same outcome a real unreachable service
//! produces from the renderer's point of view, minus the flakiness.

use assert_cmd::Command;
use serde_json::Value;
use std::path::Path;
use tempfile::TempDir;

/// A repo whose manifest declares one service that does not exist on disk
/// (so `check` finds drift) and whose `url` is a private address (so `--ping`
/// blocks it without touching the network).
fn repo_with_a_blocked_url() -> TempDir {
    let dir = TempDir::new().expect("tempdir");
    write_manifest(
        dir.path(),
        r#"version: "1"

discovery:
  paths: ["services/*"]

services:
  - name: billing
    language: Rust
    platform: Cloud Run
    url: http://10.0.0.1/health
"#,
    );
    dir
}

fn write_manifest(root: &Path, body: &str) {
    std::fs::write(root.join("services.yaml"), body).expect("write services.yaml");
}

fn sarif_stdout(root: &Path, extra_args: &[&str]) -> Value {
    let mut cmd = Command::cargo_bin("svccat").expect("binary");
    cmd.arg("--root")
        .arg(root)
        .arg("check")
        .arg("--manifest")
        .arg(root.join("services.yaml"))
        .arg("--format")
        .arg("sarif");
    for arg in extra_args {
        cmd.arg(arg);
    }
    // `check` defaults to github-annotation inside Actions when no format is
    // given; an explicit --format wins, but clear the variable anyway so the
    // test asserts the same thing locally and on the CI runners.
    cmd.env_remove("GITHUB_ACTIONS");

    let out = cmd.assert().success();
    let stdout = String::from_utf8(out.get_output().stdout.clone()).expect("utf-8 stdout");
    serde_json::from_str(&stdout).unwrap_or_else(|e| panic!("stdout is not JSON: {e}\n{stdout}"))
}

fn rule_ids(doc: &Value) -> Vec<String> {
    doc["runs"][0]["results"]
        .as_array()
        .expect("results array")
        .iter()
        .map(|r| r["ruleId"].as_str().expect("ruleId").to_string())
        .collect()
}

#[test]
fn check_format_sarif_prints_a_sarif_document_to_stdout() {
    let dir = repo_with_a_blocked_url();
    let doc = sarif_stdout(dir.path(), &[]);

    assert_eq!(doc["version"], "2.1.0");
    assert_eq!(doc["runs"][0]["tool"]["driver"]["name"], "svccat");
    // The declared service has no directory on disk, so `check` really ran.
    assert!(
        rule_ids(&doc).contains(&"declared_missing_from_repo".to_string()),
        "expected drift in {:?}",
        rule_ids(&doc)
    );
}

/// The defect this file was written to pin: before the fix, `render_check`
/// took `_ping_results` and dropped it, so `--ping` findings were invisible in
/// SARIF output *and* in the exit code (ping never affects it), while the
/// json/junit/markdown/terminal renderers all reported them.
#[test]
fn check_ping_format_sarif_reports_a_blocked_url_as_a_finding() {
    let dir = repo_with_a_blocked_url();
    let doc = sarif_stdout(dir.path(), &["--ping"]);

    let ids = rule_ids(&doc);
    assert!(
        ids.contains(&"invalid_service_url".to_string()),
        "--ping finding missing from SARIF results: {ids:?}"
    );

    let finding = doc["runs"][0]["results"]
        .as_array()
        .unwrap()
        .iter()
        .find(|r| r["ruleId"] == "invalid_service_url")
        .expect("invalid_service_url result");
    assert_eq!(finding["level"], "error");
    assert_eq!(
        finding["locations"][0]["logicalLocations"][0]["name"],
        "billing"
    );
    assert!(finding["message"]["text"]
        .as_str()
        .unwrap()
        .contains("10.0.0.1"));
}

/// Every emitted `ruleId` must be declared in `tool.driver.rules`, asserted
/// against the real binary's output rather than the in-process document, so a
/// rule dropped from the shipped build is caught here too.
#[test]
fn every_emitted_rule_id_is_declared_by_the_real_binary() {
    let dir = repo_with_a_blocked_url();
    let doc = sarif_stdout(dir.path(), &["--ping"]);

    let declared: Vec<String> = doc["runs"][0]["tool"]["driver"]["rules"]
        .as_array()
        .expect("rules array")
        .iter()
        .map(|r| r["id"].as_str().expect("rule id").to_string())
        .collect();

    let emitted = rule_ids(&doc);
    assert!(!emitted.is_empty(), "fixture produced no findings at all");
    for id in &emitted {
        assert!(
            declared.contains(id),
            "result ruleId {id:?} is not declared in tool.driver.rules {declared:?}"
        );
    }
}

/// The absolute-path URI defect, pinned against the real binary with the
/// exact repro from the bug report: `svccat --root <abs path> check --format
/// sarif` used to write the bare absolute path into every
/// `artifactLocation.uri`, which on Windows is a URI whose scheme is a drive
/// letter and on POSIX a root-relative reference. This fixture already runs
/// with an absolute `--root` (the tempdir) and an absolute `--manifest`
/// under it, so the URI must now come out relativised against the root: the
/// SARIF-preferred form, identical on every OS in the CI matrix.
#[test]
fn an_absolute_root_yields_a_root_relative_artifact_uri_not_a_bare_path() {
    let dir = repo_with_a_blocked_url();
    let doc = sarif_stdout(dir.path(), &["--ping"]);

    let artifact = doc["runs"][0]["artifacts"][0]["location"]["uri"]
        .as_str()
        .expect("artifact uri");
    assert_eq!(
        artifact, "services.yaml",
        "expected the manifest relativised against --root, got {artifact:?}"
    );

    let results = doc["runs"][0]["results"].as_array().expect("results array");
    assert!(!results.is_empty(), "fixture produced no findings at all");
    for result in results {
        assert_eq!(
            result["locations"][0]["physicalLocation"]["artifactLocation"]["uri"]
                .as_str()
                .expect("result uri"),
            artifact,
            "every result must name the same artifact the artifacts array declares"
        );
    }
}

/// Without `--ping` the document must contain no ping rule ids at all, which
/// is what keeps the test above from passing for the wrong reason.
#[test]
fn without_ping_no_ping_findings_are_emitted() {
    let dir = repo_with_a_blocked_url();
    let doc = sarif_stdout(dir.path(), &[]);

    let ids = rule_ids(&doc);
    assert!(
        !ids.contains(&"invalid_service_url".to_string())
            && !ids.contains(&"unreachable_service".to_string()),
        "ping findings appeared without --ping: {ids:?}"
    );
}
