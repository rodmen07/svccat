//! `svccat workspace check` must emit the same bytes every time it is given the
//! same workspace.
//!
//! The JSON and Markdown workspace reports are what a CI step diffs, uploads or
//! commits, so this is a byte-determinism contract rather than a display-order
//! preference: if the bytes move on their own, a `git diff --exit-code` or an
//! artifact comparison reports a change that is not one, on a report that names
//! REAL findings a user is meant to act on.
//!
//! Three separate producers fed hash order into that output, and all three were
//! reproduced on the pre-fix binary over the fixture below (ten runs each, one
//! process per run):
//!
//! 1. `DependencyGraph::validate_all_dependencies` walked `self.nodes`, a
//!    `HashMap`, so `unresolvable_dependencies` came out in a different order
//!    almost every run — **10 distinct orderings in 10 runs**.
//! 2. `DependencyGraph::detect_cycles_in_graph` picked its DFS start nodes from
//!    `nodes.keys()`. Which node the search enters a cycle FROM decides where
//!    that cycle's path is cut, so this rotated the cycle's own member list and
//!    its `description` string: **all three rotations of one three-service cycle
//!    appeared across ten runs**.
//! 3. `cross_repo_analysis` collected `graph.nodes.values()` into the HTML
//!    report's D3 payload, so both its `nodes` array and the `links` array
//!    derived from it wandered.
//!
//! Whole-document sha256 over ten processes was **10 distinct of 10** for
//! `--format json`, `--format html` AND `--format markdown`.
//!
//! These are binary-level tests on purpose: separate PROCESSES are what a user
//! actually experiences (`svccat workspace check` today, the same command in
//! tomorrow's CI run), and `RandomState` is re-seeded per process, which is the
//! exact axis the defect lived on. A determinism test on the graph functions
//! alone would not be a property of the binary's output; the unit tests in
//! `src/deps_graph.rs` cover the producers, and these cover the artifact.

use assert_cmd::Command;
use std::collections::BTreeSet;
use std::fs;
use std::path::Path;
use tempfile::TempDir;

/// How many separate processes each byte-identity assertion samples.
///
/// Seven unresolvable dependencies give 7! orderings and the pre-fix binary
/// really did wander over them (10 distinct in 10 runs), so ten runs agreeing by
/// luck is not a thing that happens. One run would prove nothing whatever.
const RUNS: usize = 10;

/// A cheaper sample for the sweep over the formats the defect was NOT reproduced
/// in, which is a completeness check rather than the headline contract.
const SWEEP_RUNS: usize = 3;

/// Every `--format` value `workspace check` accepts, read off its `--help`.
///
/// The three formats the defect was reproduced in get `RUNS` samples of their
/// own below; this list exists so a future producer that leaks hash order into
/// `csv` or `slack` is caught by the same guard instead of by a user.
const ALL_FORMATS: &[&str] = &[
    "terminal",
    "compact",
    "json",
    "sarif",
    "markdown",
    "junit",
    "github-annotation",
    "csv",
    "slack",
    "teams",
    "datadog",
    "html",
];

const WORKSPACE_CONFIG: &str = "\
[workspace]
name = \"Order Fixture\"
repos = [
  { name = \"r1\", path = \"r1\" },
  { name = \"r2\", path = \"r2\" },
  { name = \"r3\", path = \"r3\" },
]
";

/// `a1` reaches `r2:b1`, which reaches `r3:c1`, which reaches back to `r1:a1`:
/// one three-service cycle spanning all three repos, which is what makes the
/// rotation observable. Each repo also names services that exist nowhere, which
/// is what fills `unresolvable_dependencies`.
const R1_MANIFEST: &str = "\
version: \"1\"
services:
  - name: a1
    depends_on: [ghost1, ghost2, \"r2:b1\"]
  - name: a2
    depends_on: [ghost3]
  - name: a3
    depends_on: [ghost4]
";

const R2_MANIFEST: &str = "\
version: \"1\"
services:
  - name: b1
    depends_on: [ghost5, \"r3:c1\"]
  - name: b2
    depends_on: [ghost6]
";

const R3_MANIFEST: &str = "\
version: \"1\"
services:
  - name: c1
    depends_on: [\"r1:a1\"]
  - name: c2
    depends_on: [ghost7]
";

/// Lay the three-repo workspace down once. Every sample of a given assertion
/// runs against this ONE directory, so the temp path is held constant and the
/// only thing varying between samples is the process.
fn workspace() -> TempDir {
    let tmp = TempDir::new().unwrap();
    for (repo, manifest) in [
        ("r1", R1_MANIFEST),
        ("r2", R2_MANIFEST),
        ("r3", R3_MANIFEST),
    ] {
        fs::create_dir(tmp.path().join(repo)).unwrap();
        fs::write(tmp.path().join(repo).join("services.yaml"), manifest).unwrap();
    }
    fs::write(tmp.path().join("svccat.toml"), WORKSPACE_CONFIG).unwrap();
    tmp
}

/// One `svccat workspace check --format <format>` process; returns stdout.
fn check_stdout(dir: &Path, format: &str) -> String {
    let out = Command::cargo_bin("svccat")
        .unwrap()
        .current_dir(dir)
        .args(["workspace", "check", "--config", "svccat.toml", "--format"])
        .arg(format)
        .output()
        .unwrap();

    String::from_utf8(out.stdout).unwrap()
}

/// Run `format` in `runs` separate processes and return the distinct outputs.
fn distinct_outputs(dir: &Path, format: &str, runs: usize) -> BTreeSet<String> {
    (0..runs).map(|_| check_stdout(dir, format)).collect()
}

fn assert_byte_identical(format: &str, runs: usize) {
    let dir = workspace();
    let distinct = distinct_outputs(dir.path(), format, runs);
    assert_eq!(
        distinct.len(),
        1,
        "`workspace check --format {format}` produced {} distinct documents in \
         {runs} processes; it must produce exactly 1",
        distinct.len()
    );
}

/// The JSON array named by `key`, sliced out by bracket matching rather than by
/// assuming where serde puts it among the sibling keys.
fn json_array(doc: &str, key: &str) -> String {
    let marker = format!("\"{key}\": [");
    let start = doc
        .find(&marker)
        .unwrap_or_else(|| panic!("no `{key}` array in the JSON report"))
        + marker.len()
        - 1;

    let mut depth = 0usize;
    for (offset, ch) in doc[start..].char_indices() {
        match ch {
            '[' => depth += 1,
            ']' => {
                depth -= 1;
                if depth == 0 {
                    return doc[start..start + offset + 1].to_string();
                }
            }
            _ => {}
        }
    }
    panic!("unterminated `{key}` array in the JSON report");
}

/// Every `ghostN` name in `text`, in the order it first appears.
fn ghosts_in_order(text: &str) -> Vec<String> {
    let mut seen = BTreeSet::new();
    let mut order = Vec::new();
    let bytes = text.as_bytes();
    let mut cursor = 0usize;

    while let Some(hit) = text[cursor..].find("ghost") {
        let start = cursor + hit;
        let mut end = start + "ghost".len();
        while end < bytes.len() && bytes[end].is_ascii_digit() {
            end += 1;
        }
        let name = text[start..end].to_string();
        if seen.insert(name.clone()) {
            order.push(name);
        }
        cursor = end;
    }

    order
}

/// The sorted order the fixture must produce: by depending service
/// (`r1:a1` < `r1:a2` < `r1:a3` < `r2:b1` < `r2:b2` < `r3:c2`), then by the
/// order that service's own manifest declares its `depends_on` entries — which
/// is why `ghost1` precedes `ghost2` inside `a1` rather than being sorted too.
const EXPECTED_GHOST_ORDER: [&str; 7] = [
    "ghost1", "ghost2", "ghost3", "ghost4", "ghost5", "ghost6", "ghost7",
];

/// The cycle, cut at its lowest member. `r1:a1` sorts below `r2:b1` and
/// `r3:c1`, so a sorted DFS start order always enters this cycle at `r1:a1`.
const EXPECTED_CYCLE: &str = "Circular dependency: r1:a1 → r2:b1 → r3:c1";

#[test]
fn json_report_is_byte_identical_across_ten_processes() {
    assert_byte_identical("json", RUNS);
}

#[test]
fn html_report_is_byte_identical_across_ten_processes() {
    assert_byte_identical("html", RUNS);
}

#[test]
fn markdown_report_is_byte_identical_across_ten_processes() {
    assert_byte_identical("markdown", RUNS);
}

#[test]
fn every_output_format_is_byte_identical_across_processes() {
    let dir = workspace();
    let unstable: Vec<&str> = ALL_FORMATS
        .iter()
        .copied()
        .filter(|format| distinct_outputs(dir.path(), format, SWEEP_RUNS).len() != 1)
        .collect();

    assert!(
        unstable.is_empty(),
        "these `--format` values produced more than one document over \
         {SWEEP_RUNS} processes on one unchanged workspace: {unstable:?}"
    );
}

#[test]
fn json_unresolvable_dependencies_are_ordered_by_depending_service() {
    let dir = workspace();
    let doc = check_stdout(dir.path(), "json");
    let array = json_array(&doc, "unresolvable_dependencies");

    assert_eq!(
        ghosts_in_order(&array),
        EXPECTED_GHOST_ORDER,
        "`unresolvable_dependencies` is not in (depending service, declaration) \
         order; array was:\n{array}"
    );
}

#[test]
fn markdown_unresolvable_dependencies_are_ordered_by_depending_service() {
    let dir = workspace();
    let doc = check_stdout(dir.path(), "markdown");
    let section = doc
        .split("### Unresolvable Dependencies")
        .nth(1)
        .expect("no unresolvable-dependency section in the Markdown report");

    assert_eq!(
        ghosts_in_order(section),
        EXPECTED_GHOST_ORDER,
        "the Markdown unresolvable list is not in (depending service, \
         declaration) order; section was:\n{section}"
    );
}

#[test]
fn the_reported_cycle_is_cut_at_its_lowest_member() {
    let dir = workspace();
    for format in ["json", "markdown", "html"] {
        let doc = check_stdout(dir.path(), format);
        assert!(
            doc.contains(EXPECTED_CYCLE),
            "`--format {format}` did not report the cycle as \
             `{EXPECTED_CYCLE}`; a rotation of it means the DFS start order is \
             back to hash order"
        );
    }
}

#[test]
fn html_graph_payload_lists_its_nodes_in_service_key_order() {
    let dir = workspace();
    let doc = check_stdout(dir.path(), "html");

    let mut ids = Vec::new();
    let mut cursor = 0usize;
    while let Some(hit) = doc[cursor..].find("\"id\":\"") {
        let start = cursor + hit + "\"id\":\"".len();
        let end = start + doc[start..].find('"').expect("unterminated graph id");
        ids.push(doc[start..end].to_string());
        cursor = end;
    }

    assert_eq!(
        ids,
        ["r1:a1", "r1:a2", "r1:a3", "r2:b1", "r2:b2", "r3:c1", "r3:c2"],
        "the D3 graph payload's nodes are not in ServiceKey order"
    );
}

/// Anti-vacuity guard: every assertion above compares documents to each other or
/// to an expected order, and all of them would pass on a workspace that found
/// NOTHING. If the fixture ever stops producing the findings these tests order —
/// a manifest key renamed, `depends_on` parsing changed, the sections dropped
/// from a renderer — this test fails loudly instead of the suite going quietly
/// green while it guards an empty report.
#[test]
fn the_fixture_really_produces_the_findings_these_tests_order() {
    let dir = workspace();
    let doc = check_stdout(dir.path(), "json");

    assert_eq!(
        ghosts_in_order(&json_array(&doc, "unresolvable_dependencies")).len(),
        7,
        "expected 7 unresolvable dependencies in the fixture; report was:\n{doc}"
    );
    assert_eq!(
        doc.matches("Circular dependency:").count(),
        1,
        "expected exactly 1 circular dependency in the fixture; report was:\n{doc}"
    );
    assert!(
        doc.contains("\"unresolvable_dependencies\": 7"),
        "the dependency summary disagrees with the unresolvable list; report \
         was:\n{doc}"
    );

    let html = check_stdout(dir.path(), "html");
    assert_eq!(
        html.matches("\"id\":\"").count(),
        7,
        "expected 7 nodes in the D3 graph payload"
    );
}
