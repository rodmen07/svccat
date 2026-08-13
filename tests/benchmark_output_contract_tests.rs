//! Drift guards for `.github/workflows/benchmark.yml`, written after `main`
//! went red on `9ee581b` with the pull request that produced that exact tree
//! green.
//!
//! WHAT HAPPENED. `Store benchmark results` is the only step that PARSES
//! `output.txt`, and it was gated on `if: github.ref == 'refs/heads/main'`. A
//! pull request therefore ran the benchmarks, wrote `output.txt`, and never
//! looked at it -- so PR #46's benchmark job was green and could not have been
//! anything else, and the identical commit failed the instant it became `main`.
//! The gate could not fail before merge by construction.
//!
//! WHY IT FAILED. Criterion's default baseline directory is `base`. When that
//! directory exists but `sample.json` inside it does not -- the state a
//! partially restored `target/` cache leaves behind -- criterion reports it
//! through `println!` (`criterion-0.8.2/src/macros_private.rs:36`), i.e. onto
//! STDOUT, which is exactly what `| tee output.txt` captures. It lands mid-line,
//! between `test <name> ... ` and `bench: <n> ns/iter`, splitting the single
//! line the extractor matches, so the run parses ZERO benchmarks. The bench
//! process exits 0 throughout, so `cargo bench` succeeds and the pipeline
//! succeeds; only the downstream parse fails.
//!
//! That citation was originally taken on the criterion this repo used when the
//! failure happened, and it survived the dev-dependency bump: the macro is still
//! a `println!` at the same line of the same file, and all three states were
//! re-run on the current version rather than assumed. Which criterion the claim
//! describes is pinned by `tests/criterion_citation_tests.rs`, so the next bump
//! cannot leave this paragraph quietly describing a program nobody builds.
//!
//! THE FIX HAS TWO HALVES AND THIS FILE GUARDS BOTH. The cause is removed by
//! wiping `target/criterion` before the run, which forces the virgin state that
//! is provably clean. The blindness is removed by running the SAME action on
//! pull requests with `auto-push: false` and `save-data-file: false`, so the
//! parser that gates `main` is the parser a PR must satisfy. That it is the
//! same action rather than a hand-written grep is the point: a second parser
//! would be a second definition of "parseable" and would drift away from the
//! one that actually gates `main`.
//!
//! SIX mutations were run against the committed workflow and each reddened
//! EXACTLY ONE test, 4 passed / 1 failed every time, with the file restored
//! byte-identically after each: deleting the `rm -rf target/criterion` line;
//! gating the PR-side check back onto `main`; flipping its `auto-push` to true;
//! pointing it at a different output file; swapping its action for a
//! hand-written `grep`, which is the realistic way the same-parser property
//! would be lost; and putting `skip-fetch-gh-pages: true` back in place of
//! `external-data-json-path`, which is the specific misconfiguration this step
//! shipped with on its first run and caught on itself.
//!
//! Deleting the PR-side step outright is deliberately NOT disjoint -- it reddens
//! three, because three of these tests are each entitled to assume it exists.
//! That is stated rather than engineered away: a guard set whose every member
//! survived the deletion of the thing it guards would be the more suspicious
//! shape.

use std::fs;
use std::path::PathBuf;

/// `.github/workflows/*.yml` in this repo is CRLF, so LF-anchored matching
/// silently finds nothing. Normalise before any comparison.
fn benchmark_workflow() -> String {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join(".github")
        .join("workflows")
        .join("benchmark.yml");
    fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("failed to read {}: {e}", path.display()))
        .replace("\r\n", "\n")
}

/// The block of a `- name: <name>` step, up to the next step at the same
/// indentation. Returns `None` when no such step exists, which is what the
/// deletion mutations produce.
fn step_block<'a>(workflow: &'a str, name: &str) -> Option<&'a str> {
    let header = format!("- name: {name}\n");
    let start = workflow.find(&header)?;
    let rest = &workflow[start + header.len()..];
    let end = rest.find("\n      - name: ").map_or(rest.len(), |i| i + 1);
    Some(&rest[..end])
}

#[test]
fn benchmarks_run_against_a_wiped_criterion_directory() {
    let workflow = benchmark_workflow();
    let block = step_block(&workflow, "Run benchmarks")
        .expect("benchmark.yml still has a `Run benchmarks` step");

    let wipe = block.find("rm -rf target/criterion");
    let bench = block.find("cargo bench");
    assert!(
        wipe.is_some(),
        "`Run benchmarks` no longer wipes target/criterion, so a partially \
         restored cache can put criterion's `base` directory back without its \
         sample.json -- the exact state that split the bencher line on 9ee581b \
         and made the run parse zero benchmarks. Step body was:\n{block}"
    );
    assert!(
        wipe < bench,
        "target/criterion is wiped AFTER cargo bench rather than before, which \
         removes only the evidence and leaves the failure. Step body was:\n{block}"
    );
}

#[test]
fn a_pull_request_parses_output_txt_with_the_same_action_that_gates_main() {
    let workflow = benchmark_workflow();
    let block = step_block(&workflow, "Verify benchmark output parses (no publish)").expect(
        "the PR-side verification step is gone. Without it nothing parses \
             output.txt before merge, which is precisely how a green pull \
             request handed main a red Performance Benchmarks on 9ee581b",
    );

    assert!(
        block.contains("uses: benchmark-action/github-action-benchmark@"),
        "the PR-side check no longer uses github-action-benchmark. A \
         hand-written parser here would be a SECOND definition of \"parseable\" \
         and would drift from the one that gates main, which is the whole \
         reason this step is the action itself. Step body was:\n{block}"
    );
    assert!(
        !block.contains("github.ref == 'refs/heads/main'"),
        "the PR-side check has been gated back onto main, which restores the \
         original defect: the only step that reads output.txt would again be \
         unreachable from a pull request. Step body was:\n{block}"
    );
}

#[test]
fn the_pull_request_check_publishes_nothing() {
    let workflow = benchmark_workflow();
    let block = step_block(&workflow, "Verify benchmark output parses (no publish)")
        .expect("the PR-side verification step is gone");

    for (key, why) in [
        (
            "auto-push: false",
            "a pull request would push tracking data to gh-pages, which for a \
             fork PR is both wrong and unauthorised",
        ),
        (
            "save-data-file: false",
            "a pull request would write a data file whose only purpose is to be \
             published, leaving state behind on a check that must be read-only",
        ),
        (
            "external-data-json-path:",
            "the step would go back to using the gh-pages BRANCH for its history, \
             and a PR checkout has no local gh-pages -- the action would run \
             `git switch gh-pages` and die with `fatal: invalid reference: \
             gh-pages` before ever parsing anything, which is what its first \
             run on PR #47 did. `skip-fetch-gh-pages: true` does NOT prevent \
             this: it skips the fetch and still performs the switch",
        ),
    ] {
        assert!(
            block.contains(key),
            "the PR-side verification step no longer declares `{key}`, so {why}. \
             Step body was:\n{block}"
        );
    }
}

#[test]
fn main_still_publishes_and_reads_the_same_file_the_pr_check_reads() {
    let workflow = benchmark_workflow();
    let store = step_block(&workflow, "Store benchmark results")
        .expect("benchmark.yml still has a `Store benchmark results` step");
    let verify = step_block(&workflow, "Verify benchmark output parses (no publish)")
        .expect("the PR-side verification step is gone");

    assert!(
        store.contains("auto-push: true"),
        "main no longer publishes benchmark tracking data, so the PR-side check \
         would be verifying a pipeline that no longer has a downstream. Step \
         body was:\n{store}"
    );
    assert!(
        store.contains("github.ref == 'refs/heads/main'"),
        "the publishing step lost its main gate and would now push from pull \
         requests. Step body was:\n{store}"
    );

    // The same-parser property is only real if both steps feed the extractor
    // the same tool and the same file. Either one drifting turns the PR check
    // into a check of something else.
    for key in ["tool: 'cargo'", "output-file-path: output.txt"] {
        assert!(
            store.contains(key) && verify.contains(key),
            "`{key}` is no longer declared by BOTH the PR-side check and the \
             main-side store, so the PR is no longer proving the thing main \
             will do.\nverify:\n{verify}\nstore:\n{store}"
        );
    }
}

/// Characterisation of criterion's output, not a gate -- the gate is the action
/// itself, in CI. This pins the two byte shapes that were captured locally on
/// criterion 0.8.2 while reproducing the failure, so the mechanism stays legible
/// without anyone having to re-derive it from a red CI run.
///
/// Both captures were RE-TAKEN on the current dependency rather than carried
/// over, and the clean one moved: the earlier version printed the timing with no
/// digit grouping (`17703`), this one groups it (`26,035`). The action's cargo
/// extractor accepts either -- its capture group is `[0-9,.]+` and it strips the
/// separators -- so the change is invisible to the parse, which is exactly why a
/// stale fixture would have kept looking right. `tests/criterion_citation_tests.rs`
/// is what forces this paragraph to be re-derived on the next bump.
#[test]
fn the_polluted_line_shape_is_the_one_that_stops_parsing() {
    // Captured verbatim from a virgin CRITERION_HOME.
    let clean = "test load_manifest_small ... bench:      26,035 ns/iter (+/- 5,012)";
    // Captured verbatim after deleting base/sample.json and keeping base/.
    let polluted = concat!(
        "test load_manifest_small ... Criterion.rs ERROR: error: Failed to ",
        "access file \"...\\\\load_manifest_small\\\\base\\\\sample.json\": ",
        "The system cannot find the file specified. (os error 2)\n",
        "bench:      19,242 ns/iter (+/- 4,322)"
    );

    let is_bencher_line = |line: &str| {
        line.starts_with("test ") && line.contains(" ... bench:") && line.contains(" ns/iter")
    };

    assert!(
        is_bencher_line(clean),
        "the clean capture stopped looking like a bencher line, so this fixture \
         no longer describes criterion's healthy output"
    );
    assert!(
        polluted.lines().all(|l| !is_bencher_line(l)),
        "the polluted capture now parses, which would mean criterion stopped \
         splitting the line -- re-derive the failure before trusting this file"
    );
    assert!(
        polluted.contains("Criterion.rs ERROR"),
        "the polluted fixture lost the marker that identifies the cause"
    );
}
