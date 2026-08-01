//! First coverage for `src/stats.rs` (`svccat stats`).
//!
//! Chosen under the oldest-untested-surface rule: `src/stats.rs` was last
//! touched 2026-05-16 and had, until this file, zero inline `mod tests` and
//! zero references from `tests/` — the oldest zero-coverage module in the
//! crate once `output/sarif.rs` and `output/github_annotation.rs` (the other
//! two 2026-05-16 modules) got theirs in PR #28 and PR #35.
//!
//! These are binary-level tests: `stats::run` returns `()` and communicates
//! only by printing, so the shipped behaviour *is* the stdout. Calling it
//! in-process would prove nothing about what a user sees. `assert_cmd` runs
//! the real `svccat` binary, exactly as `tests/cli_binary_tests.rs` does.
//!
//! `NO_COLOR` is set on every invocation so the `colored` crate emits plain
//! text regardless of what the runner's terminal detection decides; without
//! it these assertions would be at the mercy of CI's tty configuration.

use assert_cmd::Command;
use std::fs;
use tempfile::TempDir;

/// Run `svccat stats` in a temp dir holding `manifest_yaml`, return stdout.
fn stats_stdout(manifest_yaml: &str) -> String {
    let tmp = TempDir::new().unwrap();
    fs::write(tmp.path().join("services.yaml"), manifest_yaml).unwrap();

    let out = Command::cargo_bin("svccat")
        .unwrap()
        .current_dir(tmp.path())
        .env("NO_COLOR", "1")
        .arg("stats")
        .output()
        .unwrap();

    String::from_utf8(out.stdout).unwrap()
}

/// The single output line for a field row, e.g. `  team  1/2  ██…  50%`.
fn field_row<'a>(stdout: &'a str, field: &str) -> &'a str {
    stdout
        .lines()
        .find(|l| l.trim_start().starts_with(field))
        .unwrap_or_else(|| panic!("no `{field}` row in stats output:\n{stdout}"))
}

/// (filled, empty) block counts in a rendered bar.
fn bar_blocks(row: &str) -> (usize, usize) {
    (
        row.chars().filter(|c| *c == '█').count(),
        row.chars().filter(|c| *c == '░').count(),
    )
}

// ── The empty and singular cases ────────────────────────────────────────────

#[test]
fn an_empty_catalog_says_so_instead_of_dividing_by_zero() {
    let out = stats_stdout("services: []\n");
    assert!(
        out.contains("no services declared"),
        "expected the empty-catalog message, got:\n{out}"
    );
    // The table is skipped entirely; no field rows, no health line.
    assert!(
        !out.contains("Field Coverage"),
        "got a table for 0 services:\n{out}"
    );
    assert!(
        !out.contains("Overall health"),
        "got a health score for 0 services:\n{out}"
    );
}

#[test]
fn one_service_is_reported_in_the_singular() {
    let out = stats_stdout("services:\n  - name: alpha\n");
    assert!(
        out.contains("[1 service]"),
        "expected singular, got:\n{out}"
    );
    assert!(
        !out.contains("[1 services]"),
        "pluralised a single service:\n{out}"
    );
}

#[test]
fn more_than_one_service_is_reported_in_the_plural() {
    let out = stats_stdout("services:\n  - name: alpha\n  - name: beta\n");
    assert!(out.contains("[2 services]"), "expected plural, got:\n{out}");
}

// ── The coverage table ──────────────────────────────────────────────────────

#[test]
fn every_tracked_field_gets_a_row() {
    let out = stats_stdout("services:\n  - name: alpha\n");
    for field in [
        "language", "platform", "team", "docs", "url", "role", "oncall",
    ] {
        assert!(
            out.lines().any(|l| l.trim_start().starts_with(field)),
            "no row for tracked field `{field}`:\n{out}"
        );
    }
}

#[test]
fn a_field_set_on_half_the_services_reads_half() {
    let out = stats_stdout("services:\n  - name: alpha\n    team: platform\n  - name: beta\n");

    let row = field_row(&out, "team");
    assert!(row.contains("1/2"), "expected 1 of 2 in `{row}`");
    assert!(row.contains("50%"), "expected 50% in `{row}`");

    // The bar is 20 blocks wide and half filled.
    let (filled, empty) = bar_blocks(row);
    assert_eq!((filled, empty), (10, 10), "bar for `{row}`");
}

#[test]
fn a_field_no_service_declares_reads_zero_and_an_empty_bar() {
    let out = stats_stdout("services:\n  - name: alpha\n  - name: beta\n");

    let row = field_row(&out, "oncall");
    assert!(row.contains("0/2"), "expected 0 of 2 in `{row}`");
    assert!(row.contains("0%"), "expected 0% in `{row}`");
    assert_eq!(bar_blocks(row), (0, 20), "bar for `{row}`");
}

#[test]
fn a_field_every_service_declares_reads_full() {
    let out = stats_stdout(
        "services:\n  - name: alpha\n    language: rust\n  - name: beta\n    language: go\n",
    );

    let row = field_row(&out, "language");
    assert!(row.contains("2/2"), "expected 2 of 2 in `{row}`");
    assert!(row.contains("100%"), "expected 100% in `{row}`");
    assert_eq!(bar_blocks(row), (20, 0), "bar for `{row}`");
}

#[test]
fn overall_health_is_the_mean_of_the_field_percentages() {
    // 7 tracked fields. Declaring 1 of them on the single service gives one
    // row at 100% and six at 0%, i.e. 100/7 = 14 after integer division.
    let out = stats_stdout("services:\n  - name: alpha\n    language: rust\n");
    assert!(
        out.contains("Overall health: 14%"),
        "expected 100/7 = 14%, got:\n{out}"
    );

    // All 7 declared is a clean 100%, which also proves the score is not
    // capped or averaged against some larger hidden field set.
    let out = stats_stdout(
        "services:\n  - name: alpha\n    language: rust\n    platform: fly\n    \
         team: platform\n    docs: docs/alpha.md\n    url: https://alpha.example.com\n    \
         role: api\n    oncall: \"@alpha\"\n",
    );
    assert!(
        out.contains("Overall health: 100%"),
        "expected 100%, got:\n{out}"
    );
}

// ── The regression this file was written for ────────────────────────────────

#[test]
fn a_field_declared_as_an_empty_string_does_not_count_as_declared() {
    // `stats` has always been right about this; the test pins it because the
    // predicate is now shared with `scorecard`, `policy` and `drift`, which
    // were not, and a future "simplification" back to `is_some()` would
    // silently re-inflate every one of them.
    let out = stats_stdout("services:\n  - name: alpha\n    team: \"\"\n    language: \"\"\n");

    let team = field_row(&out, "team");
    assert!(
        team.contains("0/1"),
        "`team: \"\"` counted as declared: `{team}`"
    );
    let language = field_row(&out, "language");
    assert!(
        language.contains("0/1"),
        "`language: \"\"` counted as declared: `{language}`"
    );
    assert!(
        out.contains("Overall health: 0%"),
        "empty strings inflated the health score:\n{out}"
    );
}
