//! First coverage for `src/output/terminal.rs` — the oldest untested surface
//! in this repo: its content dates to v0.1.0 (`48ce7a1`) and was last touched
//! for v0.7.0 (`d183889`), yet it had no `mod tests` and no integration test
//! referenced it directly. This follows the `tests/sarif_output_tests.rs` /
//! `tests/cli_binary_tests.rs` precedent exactly: in-process tests for the
//! one function with an observable return value (`render_since_diff`), and
//! binary-level tests (spawn the real compiled `svccat`, assert on real
//! stdout) for the renderers that only print, so a wiring regression — a
//! format match arm falling through, the wrong report threaded into a render
//! call, output computed but never printed — fails here even though it
//! compiles clean.
//!
//! The `--since` identity contract pinned below is load-bearing beyond this
//! module: `main.rs`'s `--baseline` filter builds the same
//! `kind|service|detail` key by hand (a textual duplicate of
//! `terminal::drift_key`), so these tests document the semantics both sites
//! must keep agreeing on: message and severity changes are NOT new drift;
//! only a kind, service, or detail change is.

use assert_cmd::Command;
use predicates::prelude::*;
use std::path::Path;
use svccat::drift::DriftReport;
use svccat::output::terminal::render_since_diff;
use tempfile::TempDir;

// ── In-process: render_since_diff return values ─────────────────────────────

/// Build a `DriftReport` through serde, the same route
/// `tests/diff_drift_order_tests.rs` uses: every drift type here is
/// `#[non_exhaustive]`, so JSON is the supported out-of-crate constructor.
fn report(drifts: serde_json::Value) -> DriftReport {
    serde_json::from_value(serde_json::json!({
        "manifest": "services.yaml",
        "declared": 2,
        "discovered": 2,
        "drifts": drifts,
    }))
    .expect("valid DriftReport JSON")
}

fn drift(kind: &str, severity: &str, service: &str, message: &str) -> serde_json::Value {
    serde_json::json!({
        "kind": kind,
        "severity": severity,
        "service": service,
        "message": message,
    })
}

fn drift_with_detail(
    kind: &str,
    severity: &str,
    service: &str,
    message: &str,
    detail: &str,
) -> serde_json::Value {
    serde_json::json!({
        "kind": kind,
        "severity": severity,
        "service": service,
        "message": message,
        "detail": detail,
    })
}

#[test]
fn since_identical_reports_count_nothing() {
    let old = report(serde_json::json!([drift(
        "declared_missing_from_repo",
        "error",
        "billing",
        "declared but not discovered"
    )]));
    let new = report(serde_json::json!([drift(
        "declared_missing_from_repo",
        "error",
        "billing",
        "declared but not discovered"
    )]));
    assert_eq!(render_since_diff(&old, &new, "HEAD~1"), (0, 0));
}

#[test]
fn since_counts_new_and_resolved_drift() {
    let old = report(serde_json::json!([drift(
        "declared_missing_from_repo",
        "error",
        "billing",
        "declared but not discovered"
    )]));
    let new = report(serde_json::json!([drift(
        "undeclared_in_repo",
        "warning",
        "shipping",
        "discovered but not declared"
    )]));
    // billing resolved, shipping new.
    assert_eq!(render_since_diff(&old, &new, "HEAD~1"), (1, 1));
}

#[test]
fn since_message_and_severity_changes_are_not_new_drift() {
    // Same kind + service + (absent) detail on both sides; only the message
    // text and the severity differ. The identity key deliberately ignores
    // both, so this must count as unchanged — the same contract the
    // `--baseline` filter in `main.rs` implements with its own copy of the
    // key. If this test starts failing after an intentional key change,
    // main.rs's baseline key must change in the same commit.
    let old = report(serde_json::json!([drift(
        "missing_field",
        "warning",
        "billing",
        "missing url"
    )]));
    let new = report(serde_json::json!([drift(
        "missing_field",
        "error",
        "billing",
        "url is required"
    )]));
    assert_eq!(render_since_diff(&old, &new, "v1.0.0"), (0, 0));
}

#[test]
fn since_detail_change_is_new_plus_resolved() {
    let old = report(serde_json::json!([drift_with_detail(
        "missing_field",
        "warning",
        "billing",
        "missing field",
        "url"
    )]));
    let new = report(serde_json::json!([drift_with_detail(
        "missing_field",
        "warning",
        "billing",
        "missing field",
        "docs"
    )]));
    assert_eq!(render_since_diff(&old, &new, "HEAD~1"), (1, 1));
}

#[test]
fn since_same_kind_same_service_distinct_details_all_counted() {
    // Two MissingField items for the same service must not collapse into one
    // identity: detail is what keeps them distinct.
    let old = report(serde_json::json!([]));
    let new = report(serde_json::json!([
        drift_with_detail(
            "missing_field",
            "warning",
            "billing",
            "missing field",
            "url"
        ),
        drift_with_detail(
            "missing_field",
            "warning",
            "billing",
            "missing field",
            "docs"
        ),
    ]));
    assert_eq!(render_since_diff(&old, &new, "HEAD~1"), (2, 0));
}

// ── Binary-level: check (terminal default) and check --format compact ───────

fn write_manifest(root: &Path, body: &str) {
    std::fs::write(root.join("services.yaml"), body).expect("write services.yaml");
}

fn touch(root: &Path, rel: &str) {
    let full = root.join(rel);
    std::fs::create_dir_all(full.parent().expect("parent")).expect("mkdir");
    std::fs::write(full, "").expect("touch");
}

/// Two declared services, both present on disk: no drift. Field set copied
/// from `tests/integration_test.rs::no_drift_when_all_services_found`, which
/// proves this exact shape analyzes clean.
const CLEAN_MANIFEST: &str = r#"
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
"#;

fn clean_repo() -> TempDir {
    let dir = TempDir::new().expect("tempdir");
    touch(dir.path(), "services/api-gateway/Cargo.toml");
    touch(dir.path(), "services/auth-service/Dockerfile");
    write_manifest(dir.path(), CLEAN_MANIFEST);
    dir
}

/// The clean repo plus a third declared service with no directory on disk:
/// exactly one DeclaredMissingFromRepo error.
fn drifted_repo() -> TempDir {
    let dir = TempDir::new().expect("tempdir");
    touch(dir.path(), "services/api-gateway/Cargo.toml");
    touch(dir.path(), "services/auth-service/Dockerfile");
    write_manifest(
        dir.path(),
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
  - name: ghost-service
    language: Go
    role: Phantom
    platform: Cloud Run
"#,
    );
    dir
}

fn check_cmd(root: &Path, extra_args: &[&str]) -> Command {
    let mut cmd = Command::cargo_bin("svccat").expect("binary");
    cmd.arg("--root")
        .arg(root)
        .arg("check")
        .arg("--manifest")
        .arg(root.join("services.yaml"));
    for arg in extra_args {
        cmd.arg(arg);
    }
    // Inside GitHub Actions, `check` defaults to github-annotation format when
    // no --format is given (see `tests/sarif_output_tests.rs`): remove the
    // variable so the default-format tests assert the same thing locally and
    // on the CI runners. NO_COLOR pins `colored` off regardless of tty
    // detection, so the substrings below match raw text on every runner.
    cmd.env_remove("GITHUB_ACTIONS");
    cmd.env("NO_COLOR", "1");
    cmd
}

#[test]
fn check_default_terminal_ok_path_prints_summary_via_binary() {
    check_cmd(clean_repo().path(), &[])
        .assert()
        .success()
        .stdout(predicate::str::contains("2 declared, 2 discovered"))
        .stdout(predicate::str::contains("OK  No drift detected"));
}

#[test]
fn check_default_terminal_drift_path_prints_items_and_counts_via_binary() {
    check_cmd(drifted_repo().path(), &[])
        .assert()
        .success() // no --fail-on-drift: drift still exits 0
        .stdout(predicate::str::contains("3 declared, 2 discovered"))
        // Singular/plural handling is part of the pinned surface.
        .stdout(predicate::str::contains(
            "DRIFT DETECTED  (1 error, 0 warnings)",
        ))
        .stdout(predicate::str::contains("[MISSING]"))
        .stdout(predicate::str::contains("ghost-service"))
        .stdout(predicate::str::contains("1 error(s)"));
}

#[test]
fn check_drift_with_fail_on_drift_exits_one_via_binary() {
    check_cmd(drifted_repo().path(), &["--fail-on-drift"])
        .assert()
        .code(1)
        .stdout(predicate::str::contains("DRIFT DETECTED"));
}

#[test]
fn check_format_compact_ok_path_lists_every_service_via_binary() {
    check_cmd(clean_repo().path(), &["--format", "compact"])
        .assert()
        .success()
        .stdout(predicate::str::contains("api-gateway"))
        .stdout(predicate::str::contains("auth-service"))
        .stdout(predicate::str::contains("2 ok, 0 errors, 0 warnings"));
}

#[test]
fn check_format_compact_drift_path_marks_service_and_counts_via_binary() {
    check_cmd(drifted_repo().path(), &["--format", "compact"])
        .assert()
        .success()
        .stdout(predicate::str::contains("[MISSING]"))
        .stdout(predicate::str::contains("ghost-service"))
        .stdout(predicate::str::contains("2 ok"))
        .stdout(predicate::str::contains("1 error(s)"))
        .stdout(predicate::str::contains("0 warning(s)"));
}

#[test]
fn check_format_compact_undeclared_service_is_a_warning_via_binary() {
    let dir = clean_repo();
    // A discovered directory the manifest does not declare.
    touch(dir.path(), "services/stowaway/Dockerfile");
    check_cmd(dir.path(), &["--format", "compact"])
        .assert()
        .success()
        .stdout(predicate::str::contains("[UNDECLARED]"))
        .stdout(predicate::str::contains("stowaway"))
        .stdout(predicate::str::contains("2 ok"))
        .stdout(predicate::str::contains("1 warning(s)"));
}
