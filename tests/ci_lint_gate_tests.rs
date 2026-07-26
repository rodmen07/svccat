//! PR-time drift guards for the `Lint (fmt + clippy)` job in `.github/workflows/ci.yml`.
//!
//! A workflow file is not exercised by `cargo test`, and this repo has shipped
//! inert CI before: the pre-PR-#15 "Automated Fuzz Testing" workflow was green
//! for months while fuzzing nothing. The lint gate is therefore pinned from the
//! test suite, which every PR runs, so that deleting the job, renaming it out
//! from under its branch-protection required context, or weakening it by
//! dropping `-D warnings` fails a check instead of passing silently.
//!
//! The two tests fail on disjoint mutations on purpose (see the negative
//! controls in the PR that added them): weakening a command fails only the
//! command test, renaming the job fails only the name test.

use std::fs;

const CI_WORKFLOW: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/.github/workflows/ci.yml");

/// The exact `name:` of the lint job. This string is also the branch-protection
/// required-status-check context on `main`, so a rename here without the
/// matching protection update either detaches the gate or blocks every PR on a
/// context that never posts.
const LINT_JOB_NAME: &str = "Lint (fmt + clippy)";

/// Reads the workflow with CRLF normalised: `.github/workflows/*.yml` in this
/// repo is CRLF, so LF-anchored matching silently finds nothing.
fn workflow_text() -> String {
    fs::read_to_string(CI_WORKFLOW)
        .unwrap_or_else(|e| panic!("failed to read {CI_WORKFLOW}: {e}"))
        .replace("\r\n", "\n")
}

/// Returns the body of the top-level job whose key is `key`, i.e. every line
/// from `  <key>:` up to the next two-space-indented key.
fn job_block(text: &str, key: &str) -> String {
    let header = format!("  {key}:");
    let mut lines = text.lines().skip_while(|line| line.trim_end() != header);
    let Some(first) = lines.next() else {
        panic!(
            "`.github/workflows/ci.yml` has no top-level job `{key}`; \
             jobs found: {:?}",
            job_keys(text)
        );
    };

    let mut block = String::from(first);
    for line in lines {
        let is_next_job = line.starts_with("  ") && !line.starts_with("   ") && line.trim() != "";
        if is_next_job {
            break;
        }
        block.push('\n');
        block.push_str(line);
    }
    block
}

/// Every top-level job key in the workflow, for readable failure messages.
fn job_keys(text: &str) -> Vec<String> {
    text.lines()
        .filter(|line| line.starts_with("  ") && !line.starts_with("   ") && line.ends_with(':'))
        .map(|line| line.trim().trim_end_matches(':').to_string())
        .collect()
}

#[test]
fn ci_workflow_defines_a_lint_job_under_its_protected_context_name() {
    let text = workflow_text();
    let block = job_block(&text, "lint");

    let name_line = block
        .lines()
        .find(|line| line.trim_start().starts_with("name:"))
        .unwrap_or_else(|| panic!("the `lint` job has no `name:` line:\n{block}"));

    assert_eq!(
        name_line.trim(),
        format!("name: {LINT_JOB_NAME}"),
        "the `lint` job's name is the branch-protection required context on \
         `main`; renaming it here without updating protection either detaches \
         the gate or blocks every PR on a context that never posts"
    );
}

#[test]
fn lint_job_runs_fmt_and_clippy_with_warnings_denied() {
    let text = workflow_text();
    let block = job_block(&text, "lint");

    for command in [
        "cargo fmt --all -- --check",
        "cargo clippy --all-targets --all-features -- -D warnings",
    ] {
        assert!(
            block
                .lines()
                .any(|line| line.trim() == format!("run: {command}")),
            "the `lint` job must run `{command}` verbatim (dropping \
             `--all-targets`, `--all-features` or `-D warnings` narrows the \
             gate without failing anything); job body was:\n{block}"
        );
    }
}
