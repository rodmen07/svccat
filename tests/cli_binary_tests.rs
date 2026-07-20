//! Binary-level integration tests: these spawn the actual compiled `svccat`
//! executable via `assert_cmd::Command::cargo_bin` and assert on its real
//! stdout and exit code, rather than calling internal render functions
//! (`render_workspace_check_output_to_string`, `render_check_output_to_string`,
//! `output::mermaid::render_html_graph`, ...) directly in-process.
//!
//! Why this file exists: two adversarial security reviews (PR #6, PR #7, both
//! about `workspace check --format html` / `graph --format html`) both flagged
//! that the codebase had zero tests of this shape. `main.rs` extracts its
//! format-dispatch `match` arms into small functions specifically so they are
//! unit-testable "without needing to spawn the actual binary" (see the doc
//! comment above `render_workspace_check_output_to_string`), and every
//! existing test for those arms calls the extracted function directly. That
//! is real, valid coverage of the render functions themselves, but it cannot
//! catch a regression in the wiring around them: a match arm silently falling
//! through to the wrong branch, `Cli::parse()` mis-routing a subcommand, the
//! rendered string being computed but never written to stdout, or the wrong
//! variable being threaded into a render call. All of those compile clean and
//! pass every existing unit test, because none of them ever runs `main()`.
//!
//! `Command::cargo_bin("svccat")` resolves the just-built binary from
//! Cargo's own target directory, so these tests need no path or extension
//! handling for `svccat` vs `svccat.exe` and behave identically on the
//! windows/macos/ubuntu runners in this repo's CI matrix.

use assert_cmd::Command;
use predicates::prelude::*;
use std::path::PathBuf;
use tempfile::TempDir;

/// The workspace fixture already used by `tests/workspace_integration_tests.rs`
/// (2 repos, "backend" and "frontend", 4 services total, no drift). Reused
/// here rather than inventing a parallel fixture scheme.
fn workspace_fixture_config() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/workspace/svccat.toml")
}

/// The single-repo manifest fixture already used by the rules integration
/// tests (4 services, no `depends_on` edges, which is fine here: these tests
/// only need to prove the binary renders real content, not exercise the
/// graph-layout logic that the `output::mermaid` unit tests already cover).
fn manifest_basic_fixture() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/rules/manifest-basic.yaml")
}

// ── workspace check ─────────────────────────────────────────────────────────

#[test]
fn workspace_check_format_html_renders_real_html_via_binary() {
    // This is the exact command both adversarial reviews (PR #6, PR #7)
    // worried had no binary-level coverage. A regression that made the
    // `Html` arm silently fall through to `None` (the terminal-format
    // branch) would print nothing to stdout here and this test would fail
    // on the `stdout(...)` predicates below, even though every existing
    // unit test (which calls `render_workspace_check_output_to_string`
    // directly) would keep passing.
    Command::cargo_bin("svccat")
        .unwrap()
        .arg("workspace")
        .arg("check")
        .arg("--config")
        .arg(workspace_fixture_config())
        .arg("--format")
        .arg("html")
        .assert()
        .success()
        .stdout(predicate::str::contains("<!DOCTYPE html>"))
        .stdout(predicate::str::contains("backend"))
        .stdout(predicate::str::contains("frontend"));
}

#[test]
fn workspace_check_format_json_emits_real_json_via_binary() {
    Command::cargo_bin("svccat")
        .unwrap()
        .arg("workspace")
        .arg("check")
        .arg("--config")
        .arg(workspace_fixture_config())
        .arg("--format")
        .arg("json")
        .assert()
        .success()
        .stdout(predicate::str::contains("\"total_declared\": 4"))
        .stdout(predicate::str::contains("\"total_errors\": 0"))
        .stdout(predicate::str::contains("\"name\": \"backend\""));
}

#[test]
fn workspace_check_format_markdown_emits_real_markdown_via_binary() {
    Command::cargo_bin("svccat")
        .unwrap()
        .arg("workspace")
        .arg("check")
        .arg("--config")
        .arg(workspace_fixture_config())
        .arg("--format")
        .arg("markdown")
        .assert()
        .success()
        .stdout(predicate::str::starts_with("# Workspace Drift Report"))
        .stdout(predicate::str::contains("backend"))
        .stdout(predicate::str::contains("frontend"));
}

/// A pure-unit test can assert `render_workspace_check_output_to_string`
/// returns `None` for the `Terminal` format, but it cannot prove the binary
/// then actually falls back to *printing* the terminal report (via
/// `output::workspace::render_terminal`'s own `println!` calls) instead of
/// silently discarding it. This runs the real process and reads real stdout.
#[test]
fn workspace_check_default_terminal_format_prints_to_stdout_via_binary() {
    Command::cargo_bin("svccat")
        .unwrap()
        .arg("workspace")
        .arg("check")
        .arg("--config")
        .arg(workspace_fixture_config())
        .assert()
        .success()
        .stdout(predicate::str::contains("WORKSPACE DRIFT REPORT"))
        .stdout(predicate::str::contains("backend"));
}

// ── svccat graph ────────────────────────────────────────────────────────────

#[test]
fn graph_format_html_renders_real_html_via_binary() {
    Command::cargo_bin("svccat")
        .unwrap()
        .arg("graph")
        .arg("--manifest")
        .arg(manifest_basic_fixture())
        .arg("--format")
        .arg("html")
        .assert()
        .success()
        .stdout(predicate::str::contains("<!DOCTYPE html>"))
        .stdout(predicate::str::contains("svccat graph - 4 service(s)"))
        .stdout(predicate::str::contains("id=\"graph-data\""));
}

/// The sibling command that had a real DOM-based XSS fixed today (PR #7,
/// `src/output/mermaid.rs::render_html_graph`): the old renderer embedded its
/// node/link JSON via raw `{:?}` Debug-format interpolation instead of
/// `json_script::embed`, so a literal `</script>` in a service name broke out
/// of the data island and injected live markup. `mermaid.rs`'s own unit test
/// (`malicious_service_name_in_graph_data_cannot_close_the_script_tag`) proves
/// the render function itself is safe; this proves the *binary* — real CLI
/// parsing, real manifest load off disk, real stdout write — produces the
/// same safe output for the same payload, closing exactly the gap both
/// reviews flagged: coverage that would fail if the wiring around a safe
/// renderer somehow reintroduced the raw payload on the way to stdout.
#[test]
fn graph_format_html_binary_neutralizes_script_breakout_payload() {
    let dir = TempDir::new().unwrap();
    let root = dir.path();

    std::fs::write(
        root.join("services.yaml"),
        r#"
discovery:
  paths: ["services/*"]
services:
  - name: "</script><script>alert(1)</script>"
    platform: "Cloud Run"
"#,
    )
    .unwrap();

    Command::cargo_bin("svccat")
        .unwrap()
        .arg("graph")
        .arg("--manifest")
        .arg(root.join("services.yaml"))
        .arg("--format")
        .arg("html")
        .assert()
        .success()
        // The raw, unescaped payload must never reach stdout: that literal
        // script-close/open sequence is what would terminate the JSON
        // `<script>` element early and start a new, live one.
        .stdout(predicate::str::contains("</script><script>alert").not())
        // The `json_script::embed`-escaped form survives intact but inert.
        .stdout(predicate::str::contains("\\u003cscript\\u003ealert(1)"));
}

#[test]
fn graph_format_mermaid_is_the_default_via_binary() {
    Command::cargo_bin("svccat")
        .unwrap()
        .arg("graph")
        .arg("--manifest")
        .arg(manifest_basic_fixture())
        .assert()
        .success()
        .stdout(predicate::str::contains("graph"));
}

#[test]
fn unknown_subcommand_exits_nonzero_via_binary() {
    // A cheap sanity check that clap's own error path (never spawned in any
    // existing test) still produces the expected nonzero exit code, not just
    // that argument structs parse correctly in isolation.
    Command::cargo_bin("svccat")
        .unwrap()
        .arg("not-a-real-subcommand")
        .assert()
        .failure();
}
