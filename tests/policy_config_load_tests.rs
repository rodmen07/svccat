//! Binary-level tests for how the CLI reports a `.svccat/policy.yaml` that
//! exists but cannot be loaded.
//!
//! Before this change, `PolicyConfig::load` swallowed both the read error and
//! the parse error and returned `None`, so a broken policy file was
//! indistinguishable from having no policy file at all. The three call sites
//! each reported that silence differently and all of them were wrong:
//!
//! * `svccat policy` printed "No policy file found. Create
//!   .svccat/policy.yaml ..." - factually untrue, the file is right there -
//!   and exited 0.
//! * `svccat ci` dropped the `policy` step from `steps_run` and reported
//!   "all checks passed", so a typo in the policy file silently disabled the
//!   policy gate in someone's pipeline.
//! * `svccat scorecard` scored the repo with no policy contribution and said
//!   nothing.
//!
//! These tests spawn the real binary (the `tests/cli_binary_tests.rs`
//! precedent) because the defect is in what the CLI *tells the user*, which
//! no in-process call to `policy::check` can observe.

use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use std::path::{Path, PathBuf};
use tempfile::TempDir;

/// The committed fuzz seed `fuzz/corpus_seeds/fuzz_policy/required_not_a_list`,
/// verbatim: a scalar where a sequence is expected, which is the single most
/// likely hand-edit typo in this file.
const BROKEN_POLICY: &str = "required: team\nrecommended: language\n";

const VALID_POLICY: &str = "required:\n  - team\nrecommended:\n  - docs\n";

/// A repo root whose manifest matches what is on disk, so lint and drift are
/// both clean and the only thing under test is the policy step.
fn repo(policy: Option<&str>) -> (TempDir, PathBuf) {
    let dir = TempDir::new().unwrap();
    let root = dir.path();
    fs::create_dir_all(root.join("services").join("api")).unwrap();
    fs::write(root.join("services").join("api").join("Cargo.toml"), "").unwrap();
    let manifest = root.join("services.yaml");
    fs::write(
        &manifest,
        r#"version: "1"

discovery:
  paths: ["services/*"]

services:
  - name: api
    language: Rust
    platform: Cloud Run
    role: Backend API
    team: backend
    docs: docs/api.md
"#,
    )
    .unwrap();
    if let Some(body) = policy {
        fs::create_dir_all(root.join(".svccat")).unwrap();
        fs::write(root.join(".svccat").join("policy.yaml"), body).unwrap();
    }
    (dir, manifest)
}

fn svccat(root: &Path) -> Command {
    let mut cmd = Command::cargo_bin("svccat").unwrap();
    cmd.arg("--root").arg(root);
    cmd
}

// ── svccat policy ───────────────────────────────────────────────────────────

#[test]
fn policy_command_names_the_broken_file_instead_of_claiming_none_exists() {
    let (dir, manifest) = repo(Some(BROKEN_POLICY));

    svccat(dir.path())
        .arg("policy")
        .arg("--manifest")
        .arg(&manifest)
        .assert()
        .failure()
        .stderr(predicate::str::contains("failed to parse policy file"))
        .stderr(predicate::str::contains("policy.yaml"))
        .stderr(predicate::str::contains("No policy file found").not());
}

#[test]
fn policy_command_still_reports_a_genuinely_absent_policy_file() {
    let (dir, manifest) = repo(None);

    svccat(dir.path())
        .arg("policy")
        .arg("--manifest")
        .arg(&manifest)
        .assert()
        .success()
        .stderr(predicate::str::contains("No policy file found"));
}

#[test]
fn policy_command_distinguishes_an_empty_policy_file_from_an_absent_one() {
    // Same wrong message, second flavour: the file exists and parses, it just
    // declares nothing. "No policy file found" was equally untrue here.
    let (dir, manifest) = repo(Some("required: []\nrecommended: []\n"));

    svccat(dir.path())
        .arg("policy")
        .arg("--manifest")
        .arg(&manifest)
        .assert()
        .success()
        .stderr(predicate::str::contains(
            "declares no required or recommended fields",
        ))
        .stderr(predicate::str::contains("No policy file found").not());
}

#[test]
fn policy_command_still_checks_a_valid_policy_file() {
    let (dir, manifest) = repo(Some(VALID_POLICY));

    svccat(dir.path())
        .arg("policy")
        .arg("--manifest")
        .arg(&manifest)
        .assert()
        .success()
        .stdout(predicate::str::contains("svccat policy check"))
        .stdout(predicate::str::contains("Required:"));
}

// ── svccat ci ───────────────────────────────────────────────────────────────

#[test]
fn ci_fails_the_policy_step_when_the_policy_file_cannot_be_loaded() {
    let (dir, manifest) = repo(Some(BROKEN_POLICY));

    let out = svccat(dir.path())
        .arg("ci")
        .arg("--manifest")
        .arg(&manifest)
        .arg("--format")
        .arg("json")
        .assert()
        .failure()
        .stderr(predicate::str::contains("failed to parse policy file"))
        .get_output()
        .stdout
        .clone();
    let json: serde_json::Value = serde_json::from_slice(&out).unwrap();

    assert_eq!(json["passed"], serde_json::json!(false));
    assert_eq!(json["policy"]["errors"], serde_json::json!(1));
    assert!(
        json["steps"]
            .as_array()
            .unwrap()
            .contains(&serde_json::json!("policy")),
        "the policy step must be reported as run-and-failed, got {}",
        json["steps"]
    );
}

#[test]
fn ci_still_passes_and_skips_policy_when_no_policy_file_exists() {
    // The control that keeps the test above honest: `ci` must not have simply
    // become stricter about everything. With no policy file the step is still
    // skipped and the run still passes.
    let (dir, manifest) = repo(None);

    let out = svccat(dir.path())
        .arg("ci")
        .arg("--manifest")
        .arg(&manifest)
        .arg("--format")
        .arg("json")
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let json: serde_json::Value = serde_json::from_slice(&out).unwrap();

    assert_eq!(json["passed"], serde_json::json!(true));
    assert_eq!(json["policy"]["errors"], serde_json::json!(0));
    assert!(
        !json["steps"]
            .as_array()
            .unwrap()
            .contains(&serde_json::json!("policy")),
        "an absent policy file must still skip the step, got {}",
        json["steps"]
    );
}

// ── svccat scorecard ────────────────────────────────────────────────────────

#[test]
fn scorecard_warns_rather_than_silently_scoring_without_policy() {
    let (dir, manifest) = repo(Some(BROKEN_POLICY));

    svccat(dir.path())
        .arg("scorecard")
        .arg("--manifest")
        .arg(&manifest)
        .assert()
        .success()
        .stderr(predicate::str::contains("failed to parse policy file"))
        .stderr(predicate::str::contains("scoring without policy checks"));
}
