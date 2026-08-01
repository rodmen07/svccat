//! Regression coverage for the empty-string field bypass.
//!
//! Found while adding the first tests for `src/stats.rs`: on one and the same
//! manifest, `svccat stats` reported a field at 0% while `svccat scorecard`
//! credited it as populated. The sibling sweep that followed showed the split
//! ran through the whole binary — six surfaces asked "is this field
//! declared?", and they did not agree:
//!
//! | surface | before | `team: ""` meant |
//! |---|---|---|
//! | `stats` | `!s.is_empty()` | not declared |
//! | `lint` | `map(str::is_empty)` | not declared |
//! | `scorecard::field_set` | `.is_some()` | **declared** |
//! | `policy::has_field` | `.is_some()` | **declared** |
//! | `drift` recommended fields | `.is_none()` | **declared** |
//! | `drift` `require_fields` | `.is_none()` | **declared** |
//!
//! The consequence was not cosmetic: a policy demanding `team` on every
//! service was *satisfied* by `team: ""`, so `svccat policy` printed "All
//! services comply" and the `policy` step of `svccat ci` passed, for a catalog
//! that named no owner at all. All six now route through
//! `ServiceEntry::has_field`, and these tests are what fails if any of them
//! stops doing so.
//!
//! Everything here drives the real binary, because the claim being defended is
//! about what the shipped gates do, not about what an internal predicate
//! returns.

use assert_cmd::Command;
use std::fs;
use std::path::Path;
use tempfile::TempDir;

/// Every optional string field, declared as an empty string.
const ALL_FIELDS_EMPTY: &str = "\
services:
  - name: alpha
    language: \"\"
    platform: \"\"
    url: \"\"
    role: \"\"
    team: \"\"
    oncall: \"\"
";

/// The same service with nothing declared at all. Every gate must treat this
/// and `ALL_FIELDS_EMPTY` identically; that equivalence is the whole fix.
const ALL_FIELDS_ABSENT: &str = "\
services:
  - name: alpha
";

fn run(dir: &Path, args: &[&str]) -> (String, i32) {
    let out = Command::cargo_bin("svccat")
        .unwrap()
        .current_dir(dir)
        .env("NO_COLOR", "1")
        .args(args)
        .output()
        .unwrap();

    let mut combined = String::from_utf8(out.stdout).unwrap();
    combined.push_str(&String::from_utf8(out.stderr).unwrap());
    (combined, out.status.code().unwrap_or(-1))
}

/// A repo whose `.svccat/policy.yaml` requires `team` and `oncall`.
fn repo_with_policy(manifest: &str) -> TempDir {
    let tmp = TempDir::new().unwrap();
    fs::write(tmp.path().join("services.yaml"), manifest).unwrap();
    fs::create_dir_all(tmp.path().join(".svccat")).unwrap();
    fs::write(
        tmp.path().join(".svccat/policy.yaml"),
        "required:\n  - team\n  - oncall\n",
    )
    .unwrap();
    tmp
}

// ── `svccat policy`: the required-field gate ────────────────────────────────

#[test]
fn a_required_field_declared_as_an_empty_string_is_a_violation() {
    let tmp = repo_with_policy(ALL_FIELDS_EMPTY);
    let (out, _) = run(tmp.path(), &["policy"]);

    assert!(
        out.contains("missing required field 'team'"),
        "`team: \"\"` satisfied `required: [team]`:\n{out}"
    );
    assert!(
        out.contains("missing required field 'oncall'"),
        "`oncall: \"\"` satisfied `required: [oncall]`:\n{out}"
    );
    assert!(
        !out.contains("comply with policy"),
        "reported compliance for a service that names no owner:\n{out}"
    );
}

#[test]
fn empty_and_absent_required_fields_report_identically() {
    let empty = repo_with_policy(ALL_FIELDS_EMPTY);
    let absent = repo_with_policy(ALL_FIELDS_ABSENT);

    let (out_empty, _) = run(empty.path(), &["policy"]);
    let (out_absent, _) = run(absent.path(), &["policy"]);

    assert_eq!(
        out_empty, out_absent,
        "a blank field and an absent field must be the same finding"
    );
}

#[test]
fn a_populated_required_field_still_complies() {
    // The guard against over-correcting: the gate must not have become one
    // that nothing can satisfy.
    let tmp = repo_with_policy(
        "services:\n  - name: alpha\n    team: platform\n    oncall: \"@alpha\"\n",
    );
    let (out, _) = run(tmp.path(), &["policy"]);

    assert!(
        out.contains("comply with policy"),
        "a fully declared service failed the policy gate:\n{out}"
    );
    assert!(!out.contains("missing required field"), "{out}");
}

// ── `svccat ci`: the step that gates a pipeline ─────────────────────────────

#[test]
fn the_ci_policy_step_fails_on_empty_required_fields() {
    let tmp = repo_with_policy(ALL_FIELDS_EMPTY);
    let (out, _) = run(tmp.path(), &["ci"]);

    let policy_line = out
        .lines()
        .find(|l| l.trim_start().starts_with("policy"))
        .unwrap_or_else(|| panic!("no policy step in `svccat ci` output:\n{out}"));

    assert!(
        policy_line.contains("FAIL"),
        "the ci policy step passed on a catalog with no declared owner: `{policy_line}`"
    );
}

// ── `svccat check`: the manifest's own `policy.require_fields` ──────────────

#[test]
fn manifest_require_fields_are_not_satisfied_by_an_empty_string() {
    let tmp = TempDir::new().unwrap();
    fs::write(
        tmp.path().join("services.yaml"),
        "policy:\n  require_fields:\n    - team\n    - oncall\n\
         services:\n  - name: alpha\n    team: \"\"\n    oncall: \"\"\n",
    )
    .unwrap();

    let (out, _) = run(tmp.path(), &["check"]);
    assert!(
        out.contains("required field 'team' is missing"),
        "`team: \"\"` satisfied `require_fields: [team]`:\n{out}"
    );
    assert!(
        out.contains("required field 'oncall' is missing"),
        "`oncall: \"\"` satisfied `require_fields: [oncall]`:\n{out}"
    );
}

#[test]
fn a_recommended_field_declared_as_an_empty_string_is_reported_missing() {
    let tmp = TempDir::new().unwrap();
    fs::write(
        tmp.path().join("services.yaml"),
        "services:\n  - name: alpha\n    role: \"\"\n    language: \"\"\n",
    )
    .unwrap();

    let (out, _) = run(tmp.path(), &["check"]);
    assert!(
        out.contains("missing recommended field: role"),
        "`role: \"\"` counted as a declared recommended field:\n{out}"
    );
    assert!(
        out.contains("missing recommended field: language"),
        "`language: \"\"` counted as a declared recommended field:\n{out}"
    );
}

// ── `svccat scorecard`: the completeness score ──────────────────────────────

#[test]
fn completeness_does_not_credit_empty_fields() {
    let tmp = TempDir::new().unwrap();
    fs::write(tmp.path().join("services.yaml"), ALL_FIELDS_EMPTY).unwrap();

    let (out, _) = run(tmp.path(), &["scorecard", "--format", "json"]);
    let json: serde_json::Value = serde_json::from_str(out.trim()).unwrap();

    assert_eq!(
        json["services"][0]["completeness"], 0,
        "six empty strings scored as populated metadata:\n{out}"
    );
    assert_eq!(json["avg_completeness"], 0, "{out}");
}

#[test]
fn completeness_is_identical_for_empty_and_absent_fields() {
    let empty = TempDir::new().unwrap();
    fs::write(empty.path().join("services.yaml"), ALL_FIELDS_EMPTY).unwrap();
    let absent = TempDir::new().unwrap();
    fs::write(absent.path().join("services.yaml"), ALL_FIELDS_ABSENT).unwrap();

    let (out_empty, _) = run(empty.path(), &["scorecard", "--format", "json"]);
    let (out_absent, _) = run(absent.path(), &["scorecard", "--format", "json"]);

    assert_eq!(out_empty, out_absent);
}

#[test]
fn completeness_still_rises_when_fields_are_actually_declared() {
    let tmp = TempDir::new().unwrap();
    fs::write(
        tmp.path().join("services.yaml"),
        "services:\n  - name: alpha\n    team: platform\n    oncall: \"@alpha\"\n",
    )
    .unwrap();

    let (out, _) = run(tmp.path(), &["scorecard", "--format", "json"]);
    let json: serde_json::Value = serde_json::from_str(out.trim()).unwrap();

    // 2 of 9 scored fields.
    assert_eq!(json["services"][0]["completeness"], 22, "{out}");
}

// ── The cross-surface agreement guard ───────────────────────────────────────

/// The guard that reads every surface at once.
///
/// One manifest, one blank `team:`, and all four commands that have an opinion
/// about it. Before the fix, `stats` and `lint` said "not declared" while
/// `scorecard` and `policy` said "declared" — the drift this asserts can never
/// come back. It intentionally checks observable command output rather than
/// the shared predicate, so it still fails if some surface grows its own copy
/// of the check again.
#[test]
fn every_surface_agrees_that_a_blank_field_is_not_declared() {
    let tmp = repo_with_policy(ALL_FIELDS_EMPTY);

    let (stats, _) = run(tmp.path(), &["stats"]);
    let team_row = stats
        .lines()
        .find(|l| l.trim_start().starts_with("team"))
        .unwrap_or_else(|| panic!("no team row:\n{stats}"));
    assert!(
        team_row.contains("0/1"),
        "stats credited `team: \"\"`: `{team_row}`"
    );

    let (lint, _) = run(tmp.path(), &["lint"]);
    assert!(
        lint.contains("has no team owner"),
        "lint credited `team: \"\"`:\n{lint}"
    );

    let (policy, _) = run(tmp.path(), &["policy"]);
    assert!(
        policy.contains("missing required field 'team'"),
        "policy credited `team: \"\"`:\n{policy}"
    );

    let (scorecard, _) = run(tmp.path(), &["scorecard", "--format", "json"]);
    let json: serde_json::Value = serde_json::from_str(scorecard.trim()).unwrap();
    assert_eq!(
        json["services"][0]["completeness"], 0,
        "scorecard credited `team: \"\"`:\n{scorecard}"
    );
}
