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
//! module: the markdown, junit, and github-annotation `--since` renderers
//! and `main.rs`'s `--baseline` filter all use the SAME
//! `kind|service|detail` key, and since the drift-identity extraction every
//! site calls the one shared definition, `terminal::drift_identity_key`.
//! The guard test at the bottom of this file scans the whole `src/` tree and
//! fails if any site regrows a hand-rolled copy of the format; the
//! binary-level `--baseline` tests prove the filter observes the identity
//! semantics (message and severity changes are NOT new drift; only a kind,
//! service, or detail change is) through the real compiled binary.

use assert_cmd::Command;
use predicates::prelude::*;
use std::path::Path;
use svccat::drift::{DriftItem, DriftReport};
use svccat::output::terminal::{drift_identity_key, render_since_diff};
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
    // `--baseline` filter in `main.rs` observes through the shared
    // `drift_identity_key` definition (guarded below).
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

// ── Drift identity: the shared key and its two call sites ───────────────────

/// Pins the exact key format. If this changes intentionally, every consumer
/// changes with it automatically (there is only one definition), but saved
/// baselines produced before the change stop matching — which is why the
/// format itself is part of the contract.
#[test]
fn drift_identity_key_is_kind_service_detail() {
    let with_detail: DriftItem = serde_json::from_value(drift_with_detail(
        "missing_field",
        "warning",
        "billing",
        "missing field",
        "url",
    ))
    .expect("valid DriftItem JSON");
    assert_eq!(drift_identity_key(&with_detail), "MissingField|billing|url");

    let without_detail: DriftItem = serde_json::from_value(drift(
        "declared_missing_from_repo",
        "error",
        "ghost-service",
        "'ghost-service' is declared in the manifest but not found in the repo",
    ))
    .expect("valid DriftItem JSON");
    assert_eq!(
        drift_identity_key(&without_detail),
        "DeclaredMissingFromRepo|ghost-service|"
    );
}

/// L-003 drift guard: the `{:?}|{}|{}` identity format must have exactly ONE
/// definition in the entire `src/` tree, in `src/output/terminal.rs`
/// (`drift_identity_key`), and `main.rs`'s `--baseline` filter must call it.
/// Before 2026-07-27 the format was hand-duplicated at SIX sites (terminal,
/// markdown, junit, and github-annotation renderers, plus two inline copies
/// in main.rs); if any file regrows a copy, two drift surfaces can silently
/// disagree about what counts as the same drift, and this test fails the
/// build. The scan enumerates `src/` with `read_dir` rather than probing
/// hardcoded paths, so a copy in a NEW module is caught too (and the
/// PR #23 `Path::exists()` cross-platform hazard never applies).
#[test]
fn drift_identity_format_is_defined_exactly_once_and_main_calls_it() {
    fn rust_sources(dir: &Path, out: &mut Vec<std::path::PathBuf>) {
        for entry in std::fs::read_dir(dir).expect("read_dir src") {
            let path = entry.expect("dir entry").path();
            if path.is_dir() {
                rust_sources(&path, out);
            } else if path.extension().is_some_and(|e| e == "rs") {
                out.push(path);
            }
        }
    }

    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let mut sources = Vec::new();
    rust_sources(&root.join("src"), &mut sources);
    assert!(
        sources.len() > 10,
        "the src/ scan stopped scanning anything: only {} files found",
        sources.len()
    );

    let needle = "{:?}|{}|{}";
    let mut defining_files: Vec<String> = Vec::new();
    for path in &sources {
        let text = std::fs::read_to_string(path).expect("read source file");
        for _ in 0..text.matches(needle).count() {
            defining_files.push(path.display().to_string());
        }
    }
    assert_eq!(
        defining_files.len(),
        1,
        "the drift identity format `{needle}` must appear exactly once in \
         src/ (the drift_identity_key definition); found it in: \
         {defining_files:?} — call output::terminal::drift_identity_key \
         instead of hand-rolling the format"
    );
    assert!(
        defining_files[0].ends_with("terminal.rs"),
        "the single definition must be src/output/terminal.rs's \
         drift_identity_key, found it in {}",
        defining_files[0]
    );

    let main_src = std::fs::read_to_string(root.join("src/main.rs")).expect("read src/main.rs");
    assert!(
        main_src.matches("drift_identity_key").count() >= 2,
        "main.rs's --baseline filter must call drift_identity_key for both \
         the baseline set and the retain predicate"
    );
}

// ── Binary-level: check --baseline (first coverage for the flag) ────────────

/// A baseline entry with the SAME kind|service|detail but a different message
/// and severity must still suppress the drift: the filter matches identity,
/// not bytes. This is the `--baseline` twin of
/// `since_message_and_severity_changes_are_not_new_drift`, run through the
/// real compiled binary so the main.rs wiring is what is under test.
#[test]
fn baseline_with_matching_identity_suppresses_drift_via_binary() {
    let dir = drifted_repo();
    let baseline = dir.path().join("baseline.json");
    std::fs::write(
        &baseline,
        serde_json::json!({
            "drift": [{
                "kind": "declared_missing_from_repo",
                "severity": "warning",
                "service": "ghost-service",
                "message": "message text recorded weeks ago, since reworded",
            }]
        })
        .to_string(),
    )
    .expect("write baseline.json");

    check_cmd(
        dir.path(),
        &[
            "--baseline",
            baseline.to_str().expect("utf8 path"),
            "--fail-on-drift",
        ],
    )
    .assert()
    .success()
    .stdout(predicate::str::contains("OK  No drift detected"));
}

/// A baseline whose only entry differs in identity (another service) must NOT
/// suppress the drift: the discriminator proving the filter is keyed, not a
/// suppress-everything switch.
#[test]
fn baseline_with_different_identity_does_not_suppress_via_binary() {
    let dir = drifted_repo();
    let baseline = dir.path().join("baseline.json");
    std::fs::write(
        &baseline,
        serde_json::json!({
            "drift": [{
                "kind": "declared_missing_from_repo",
                "severity": "error",
                "service": "some-other-service",
                "message": "'some-other-service' is declared in the manifest but not found in the repo",
            }]
        })
        .to_string(),
    )
    .expect("write baseline.json");

    check_cmd(
        dir.path(),
        &[
            "--baseline",
            baseline.to_str().expect("utf8 path"),
            "--fail-on-drift",
        ],
    )
    .assert()
    .code(1)
    .stdout(predicate::str::contains("DRIFT DETECTED"))
    .stdout(predicate::str::contains("ghost-service"));
}
