//! Binary-level coverage for `svccat check --format github-annotation`.
//!
//! `src/output/github_annotation.rs` is the DEFAULT output format when
//! `check` runs inside GitHub Actions with no explicit `--format`
//! (`src/main.rs` switches to it when `GITHUB_ACTIONS=true`), and it had
//! never had a test of any shape: no `mod tests`, and no integration test
//! referenced it. This file is the binary half of its first coverage (the
//! unit half lives in `src/output/github_annotation.rs`), following the
//! `tests/sarif_output_tests.rs` precedent: spawn the real compiled binary
//! and assert on its real stdout, so a regression in the *wiring* (the match
//! arm passing `&[]` instead of the real ping results, or the default-format
//! switch breaking) fails here even though every in-process unit test would
//! keep passing.
//!
//! The `--ping` tests use an address the SSRF validator rejects before any
//! socket is opened, so they need no network, no listener, and no free-port
//! dance.

use assert_cmd::Command;
use std::path::Path;
use tempfile::TempDir;

/// A repo whose manifest declares one service that does not exist on disk
/// (so `check` finds drift) and whose `url` is a private address (so `--ping`
/// blocks it without touching the network).
fn repo_with_a_blocked_url() -> TempDir {
    let dir = TempDir::new().expect("tempdir");
    write_manifest(
        dir.path(),
        r#"version: "1"

discovery:
  paths: ["services/*"]

services:
  - name: billing
    language: Rust
    platform: Cloud Run
    url: http://10.0.0.1/health
"#,
    );
    dir
}

fn write_manifest(root: &Path, body: &str) {
    std::fs::write(root.join("services.yaml"), body).expect("write services.yaml");
}

/// Run `check` with an explicit `--format github-annotation` and return
/// stdout. `GITHUB_ACTIONS` is cleared so the default-format switch in
/// `main.rs` plays no part in these tests except the one that sets it.
fn annotation_stdout(root: &Path, extra_args: &[&str], github_actions_env: bool) -> String {
    let mut cmd = Command::cargo_bin("svccat").expect("binary");
    cmd.arg("--root")
        .arg(root)
        .arg("check")
        .arg("--manifest")
        .arg(root.join("services.yaml"));
    for arg in extra_args {
        cmd.arg(arg);
    }
    if github_actions_env {
        cmd.env("GITHUB_ACTIONS", "true");
    } else {
        cmd.env_remove("GITHUB_ACTIONS");
    }

    let out = cmd.assert().success();
    String::from_utf8(out.get_output().stdout.clone()).expect("utf-8 stdout")
}

#[test]
fn check_format_github_annotation_emits_workflow_commands() {
    let dir = repo_with_a_blocked_url();
    let stdout = annotation_stdout(dir.path(), &["--format", "github-annotation"], false);

    // The declared service has no directory on disk, so `check` really ran
    // and the drift comes out as a workflow command.
    assert!(
        stdout.contains("::error ") && stdout.contains("svccat [MISSING]"),
        "expected a drift annotation, got: {stdout:?}"
    );
}

/// The defect this file was written to pin: before the fix, `render_check`
/// took only the report, so `--ping` findings were invisible in the format
/// that GitHub Actions runs by DEFAULT — and invisibly twice over, since
/// ping never affects `check`'s exit code. The sarif renderer had the same
/// defect, fixed one increment earlier; this is its sibling (the json,
/// junit, markdown and terminal renderers all reported ping results).
#[test]
fn check_ping_reports_a_blocked_url_as_an_error_annotation() {
    let dir = repo_with_a_blocked_url();
    let stdout = annotation_stdout(
        dir.path(),
        &["--format", "github-annotation", "--ping"],
        false,
    );

    assert!(
        stdout.contains("svccat [INVALID-URL]"),
        "--ping finding missing from annotation output: {stdout:?}"
    );
    let ping_line = stdout
        .lines()
        .find(|l| l.contains("svccat [INVALID-URL]"))
        .expect("ping annotation line");
    assert!(
        ping_line.starts_with("::error "),
        "ping finding must be an error annotation: {ping_line:?}"
    );
    assert!(
        ping_line.contains("billing") && ping_line.contains("10.0.0.1"),
        "annotation must name the service and the url: {ping_line:?}"
    );
}

/// Without `--ping` the output must contain no ping annotations at all,
/// which is what keeps the test above from passing for the wrong reason.
#[test]
fn without_ping_no_ping_annotations_are_emitted() {
    let dir = repo_with_a_blocked_url();
    let stdout = annotation_stdout(dir.path(), &["--format", "github-annotation"], false);

    assert!(
        !stdout.contains("[UNREACHABLE]") && !stdout.contains("[INVALID-URL]"),
        "ping findings appeared without --ping: {stdout:?}"
    );
}

/// The default-format switch: inside GitHub Actions with no explicit
/// `--format`, `check` must emit workflow commands (this is the code path
/// that makes this module the format most users actually see), and the
/// `--ping` wiring must hold there too.
#[test]
fn github_actions_env_defaults_to_annotations_and_carries_ping_findings() {
    let dir = repo_with_a_blocked_url();
    let stdout = annotation_stdout(dir.path(), &["--ping"], true);

    assert!(
        stdout.contains("::error ") && stdout.contains("svccat [MISSING]"),
        "expected annotations as the Actions default format, got: {stdout:?}"
    );
    assert!(
        stdout.contains("svccat [INVALID-URL]"),
        "--ping finding missing from the Actions default format: {stdout:?}"
    );
}

/// Escaping at the binary level: a service name carrying `%` must come out
/// data-escaped (`%25`) so a literal `%0A` in manifest content can never
/// decode as a newline on the runner. The newline-injection shape itself is
/// pinned in the unit tests; YAML cannot smuggle a raw newline into a plain
/// scalar service name, but `%`-sequences pass through YAML untouched, so
/// this is the injection precursor a manifest can actually carry.
#[test]
fn percent_sequences_in_manifest_content_are_escaped_in_the_annotation() {
    let dir = TempDir::new().expect("tempdir");
    write_manifest(
        dir.path(),
        r#"version: "1"

discovery:
  paths: ["services/*"]

services:
  - name: "billing%0A::error title=fake::injected"
    language: Rust
    platform: Cloud Run
"#,
    );
    let stdout = annotation_stdout(dir.path(), &["--format", "github-annotation"], false);

    assert!(
        stdout.contains("billing%250A"),
        "a literal %0A in a service name must be escaped to %250A: {stdout:?}"
    );
    for line in stdout.lines().filter(|l| l.contains("billing")) {
        assert!(
            !line.trim_start().is_empty() && line.starts_with("::"),
            "annotation lines must be single workflow commands: {line:?}"
        );
    }
}
