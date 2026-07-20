//! Integration tests for policy rule schema validation in `svccat lint`
//! (see `src/rule_schema.rs`).
//!
//! Kept in a focused file of its own rather than grown into the 2000+ line
//! `integration_test.rs`, matching the precedent set by
//! `reporting_config_tests.rs` and `cli_binary_tests.rs`.
//!
//! Before this feature, `lint::run` never looked at `manifest.policy` at
//! all (confirmed by reading the full pre-change file: none of its ten
//! numbered checks reference `policy`), so `svccat lint` accepted every one
//! of the malformed manifests below with exit code 0 and no mention of
//! `policy.rules` whatsoever. A duplicate rule id or a dangling `base`
//! reference only ever surfaced later, as a swallowed
//! `eprintln!("Warning: Failed to compile custom rules: ...")` inside
//! `svccat check` — and a *cyclic* `base` reference (a rule naming itself,
//! or two rules naming each other) didn't even get that: it crashed the
//! process outright with a stack overflow, because
//! `RuleEngine::compile`'s inheritance resolver recurses through the
//! `base` chain with no cycle guard. That crash was reproduced directly
//! (a throwaway example calling `RuleEngine::compile` on a single
//! self-referencing rule terminated the process with
//! `STATUS_STACK_OVERFLOW`, 0xc00000fd on Windows) while designing this
//! fix; it is deliberately NOT re-encoded as a test here, since a stack
//! overflow aborts the whole test binary process rather than failing one
//! test. `structural_errors_short_circuit_before_reaching_compile` in
//! `src/rule_schema.rs` instead proves — safely — that a cyclic input is
//! rejected before `RuleEngine::compile` is ever called.

use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use std::path::Path;
use tempfile::TempDir;

/// Write a manifest with the given `policy:` YAML block spliced in verbatim,
/// alongside a couple of services. Returns the TempDir (kept alive by the
/// caller) and the manifest path.
fn manifest_with_policy(policy_yaml: &str) -> (TempDir, std::path::PathBuf) {
    let dir = TempDir::new().unwrap();
    let root = dir.path();
    let manifest_path = root.join("services.yaml");
    fs::write(
        &manifest_path,
        format!(
            r#"
version: "1"
discovery:
  paths: ["services/*"]
{policy_yaml}
services:
  - name: service-api
    language: Rust
    platform: Cloud Run
    role: api
    team: platform
  - name: service-web
    language: TypeScript
    platform: Cloud Run
    role: web
    team: frontend
"#
        ),
    )
    .unwrap();
    (dir, manifest_path)
}

fn run_lint(manifest: &Path) -> assert_cmd::assert::Assert {
    Command::cargo_bin("svccat")
        .unwrap()
        .arg("lint")
        .arg("--manifest")
        .arg(manifest)
        .assert()
}

// ── a valid policy rule passes ──────────────────────────────────────────────

#[test]
fn lint_binary_passes_manifest_with_well_formed_policy_rules() {
    let (_dir, manifest) = manifest_with_policy(
        r#"
policy:
  rules:
    - id: naming_convention
      description: "Services must match pattern service-*"
      expression: "name matches ^service-[a-z-]+$"
      severity: error
    - id: critical_naming_convention
      description: "Derives from naming_convention with stricter severity"
      expression: "name matches ^service-[a-z-]+$"
      severity: error
      base: naming_convention
"#,
    );

    run_lint(&manifest)
        .success()
        .stdout(predicate::str::contains("policy rule").not());
}

// ── a specific, realistic malformed rule fails with a clear message ────────

#[test]
fn lint_binary_rejects_duplicate_policy_rule_ids_with_a_specific_message() {
    // A realistic mistake: copy-pasting an existing rule to start a new one
    // and forgetting to change the `id`. Before this change, `svccat lint`
    // said nothing about this manifest at all (exit 0); the duplicate only
    // caused ambiguous `base` resolution and doubled violation reporting
    // once `svccat check` ran, with no error surfaced anywhere.
    let (_dir, manifest) = manifest_with_policy(
        r#"
policy:
  rules:
    - id: required_team
      description: "Services must have a team"
      expression: "team exists"
      severity: error
    - id: required_team
      description: "Services must have docs"
      expression: "docs exists"
      severity: warning
"#,
    );

    run_lint(&manifest)
        .failure()
        .code(1)
        .stdout(predicate::str::contains("required_team"))
        .stdout(predicate::str::contains("unique"));
}

// ── edge case that was previously silently accepted (or worse) ─────────────

#[test]
fn lint_binary_rejects_self_referencing_base_before_it_can_crash_downstream() {
    // See the module doc comment: a rule whose `base` is its own `id` was
    // not merely unvalidated before this change, it hard-crashed the
    // process when `svccat check` tried to compile it (verified via a
    // throwaway repro: STATUS_STACK_OVERFLOW). `svccat lint` must catch
    // this with a normal error, not let the user find out via a crash.
    let (_dir, manifest) = manifest_with_policy(
        r#"
policy:
  rules:
    - id: self_ref
      description: "Refers to itself as its own base"
      expression: "team exists"
      severity: error
      base: self_ref
"#,
    );

    run_lint(&manifest)
        .failure()
        .code(1)
        .stdout(predicate::str::contains("cycle"))
        .stdout(predicate::str::contains("self_ref"));
}

#[test]
fn lint_binary_rejects_mutual_two_rule_base_cycle() {
    let (_dir, manifest) = manifest_with_policy(
        r#"
policy:
  rules:
    - id: rule_a
      description: "A"
      expression: "team exists"
      severity: error
      base: rule_b
    - id: rule_b
      description: "B"
      expression: "team exists"
      severity: error
      base: rule_a
"#,
    );

    run_lint(&manifest)
        .failure()
        .code(1)
        .stdout(predicate::str::contains("cycle"));
}

#[test]
fn lint_binary_rejects_dangling_base_reference() {
    let (_dir, manifest) = manifest_with_policy(
        r#"
policy:
  rules:
    - id: orphan_rule
      description: "References a base that does not exist"
      expression: "team exists"
      severity: error
      base: nonexistent_base
"#,
    );

    run_lint(&manifest)
        .failure()
        .code(1)
        .stdout(predicate::str::contains("orphan_rule"))
        .stdout(predicate::str::contains("nonexistent_base"));
}

#[test]
fn lint_binary_rejects_invalid_severity_value() {
    let (_dir, manifest) = manifest_with_policy(
        r#"
policy:
  rules:
    - id: bad_severity_rule
      description: "Has a typo'd severity"
      expression: "team exists"
      severity: critical
"#,
    );

    run_lint(&manifest)
        .failure()
        .code(1)
        .stdout(predicate::str::contains("bad_severity_rule"))
        .stdout(predicate::str::contains("critical"));
}

#[test]
fn lint_binary_reports_no_policy_rules_as_clean() {
    // No `policy:` block at all: the common case, must not regress.
    let (_dir, manifest) = manifest_with_policy("");
    run_lint(&manifest).success();
}
