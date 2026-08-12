//! `svccat audit --cost-estimate` must print the same `By platform:` block every
//! time it is given the same manifest.
//!
//! `cost::CostBreakdown::by_platform` is a `HashMap`, whose iteration order is
//! randomised per process. The terminal renderer used to order it with a STABLE
//! sort on a truncated key — `(cost as i32).wrapping_neg()` — so every pair of
//! platforms whose costs compared equal kept whatever order the hash map handed
//! over, and the block came out differently on almost every run. A ten-run sample
//! of a three-platform catalog produced six distinct orderings.
//!
//! These are binary-level tests on purpose. The defect is a property of separate
//! PROCESSES: a single process seeds its maps once, so calling the renderer twice
//! in one program is a weaker probe than running the real binary twice. `render`
//! also communicates only by printing, so the shipped behaviour *is* the stdout
//! (the `tests/stats_output_tests.rs` precedent).
//!
//! `NO_COLOR` is set on every invocation so `colored` emits plain text whatever the
//! runner's terminal detection decides.

use assert_cmd::Command;
use std::collections::BTreeSet;
use std::fs;
use tempfile::TempDir;

/// How many separate processes each determinism assertion samples.
///
/// Pre-fix, three equal-cost platforms landed in one of 3! = 6 orders per run, so
/// the chance of ten runs agreeing by luck is 6^-9 — about one in ten million.
/// One run proves nothing at all, which is why this is a repeat-run assertion and
/// not a snapshot.
const RUNS: usize = 10;

/// A catalog whose platforms all estimate to the same monthly cost.
///
/// `Vercel` is in the estimate table at $10; `Netlify` and `Railway` are not and
/// take the $10 unknown-platform default. So all three tie, which is exactly the
/// class the truncating sort key left in hash order.
const EQUAL_COST_CATALOG: &str = "\
version: \"1\"

services:
  - name: web
    platform: Vercel
  - name: docs
    platform: Netlify
  - name: worker
    platform: Railway
";

/// A catalog whose platforms have distinct estimates: Kubernetes $200,
/// AWS EC2 $100, Vercel $10.
const DISTINCT_COST_CATALOG: &str = "\
version: \"1\"

services:
  - name: web
    platform: Vercel
  - name: cluster
    platform: Kubernetes
  - name: box
    platform: AWS EC2
";

/// Run `svccat audit --cost-estimate` in a temp dir holding `manifest_yaml`.
fn audit_stdout(manifest_yaml: &str) -> String {
    let tmp = TempDir::new().unwrap();
    fs::write(tmp.path().join("services.yaml"), manifest_yaml).unwrap();

    let out = Command::cargo_bin("svccat")
        .unwrap()
        .current_dir(tmp.path())
        .env("NO_COLOR", "1")
        .arg("audit")
        .arg("--cost-estimate")
        .output()
        .unwrap();

    String::from_utf8(out.stdout).unwrap()
}

/// The `By platform:` block: every indented line following the header, in order.
///
/// Panics rather than returning empty when the header is missing, so a run that
/// printed no cost section at all can never masquerade as a stable one — an empty
/// block would compare equal to another empty block and pass this whole file
/// vacuously.
fn by_platform_block(stdout: &str) -> Vec<String> {
    let mut lines = stdout.lines();
    lines
        .find(|l| l.trim_end() == "  By platform:")
        .unwrap_or_else(|| panic!("no `By platform:` header in audit output:\n{stdout}"));

    let block: Vec<String> = lines
        .take_while(|l| l.starts_with("    ") && l.contains('$'))
        .map(|l| l.trim_end().to_string())
        .collect();

    assert!(
        !block.is_empty(),
        "`By platform:` header present but no platform rows under it:\n{stdout}"
    );
    block
}

/// The platform names of a block, in printed order.
fn platform_names(block: &[String]) -> Vec<String> {
    block
        .iter()
        .map(|l| {
            l.trim()
                .split(':')
                .next()
                .expect("a platform row is `name: $cost`")
                .to_string()
        })
        .collect()
}

// ── Determinism: the defect itself ──────────────────────────────────────────

#[test]
fn the_by_platform_block_is_identical_across_repeated_runs() {
    let orderings: BTreeSet<Vec<String>> = (0..RUNS)
        .map(|_| by_platform_block(&audit_stdout(EQUAL_COST_CATALOG)))
        .collect();

    assert_eq!(
        orderings.len(),
        1,
        "{RUNS} runs over one unchanged manifest produced {} distinct `By platform:` blocks: {:#?}",
        orderings.len(),
        orderings
    );
}

#[test]
fn distinct_cost_platforms_are_also_stable_across_runs() {
    let orderings: BTreeSet<Vec<String>> = (0..RUNS)
        .map(|_| by_platform_block(&audit_stdout(DISTINCT_COST_CATALOG)))
        .collect();

    assert_eq!(
        orderings.len(),
        1,
        "{RUNS} runs over one unchanged manifest produced {} distinct `By platform:` blocks: {:#?}",
        orderings.len(),
        orderings
    );
}

// ── Which order, not merely that there is one ───────────────────────────────

#[test]
fn platforms_are_listed_most_expensive_first() {
    let block = by_platform_block(&audit_stdout(DISTINCT_COST_CATALOG));
    assert_eq!(
        platform_names(&block),
        vec![
            "Kubernetes".to_string(),
            "AWS EC2".to_string(),
            "Vercel".to_string()
        ],
        "descending cost order expected, got:\n{block:#?}"
    );
}

#[test]
fn platforms_that_cost_the_same_are_listed_by_name() {
    // Sampled over RUNS processes rather than one: with the pre-fix key a single
    // run landed on the alphabetical order about one time in six, so a one-shot
    // assertion here would have been a coin flip rather than a control.
    let expected = vec![
        "Netlify".to_string(),
        "Railway".to_string(),
        "Vercel".to_string(),
    ];
    for run in 0..RUNS {
        let block = by_platform_block(&audit_stdout(EQUAL_COST_CATALOG));
        assert_eq!(
            platform_names(&block),
            expected,
            "run {run}: equal costs should fall back to platform name, got:\n{block:#?}"
        );
    }
}

// ── The rest of the section is untouched ────────────────────────────────────

#[test]
fn the_cost_section_still_reports_the_total_and_every_platform() {
    let stdout = audit_stdout(EQUAL_COST_CATALOG);
    assert!(
        stdout.contains("Estimated monthly: $30.00"),
        "cost total missing or wrong:\n{stdout}"
    );

    let block = by_platform_block(&stdout);
    assert_eq!(block.len(), 3, "expected one row per platform:\n{block:#?}");
    for row in &block {
        assert!(
            row.ends_with("$10.00"),
            "each platform in this catalog costs $10.00, got `{row}`"
        );
    }
}
