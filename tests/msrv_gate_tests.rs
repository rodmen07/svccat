//! PR-time drift guards for the `MSRV <version>` job in
//! `.github/workflows/ci.yml`.
//!
//! The declared `rust-version` in `Cargo.toml` was fiction from before v1.6.0
//! until the PR that added these tests: it said 1.85 while the committed
//! lockfile carried idna_adapter 1.2.2 and the icu_* 2.2.0 crates, every one
//! declaring `rust-version = "1.86"`, so `cargo +1.85 check --locked` exited
//! 101 and no consumer on the declared toolchain could ever have built the
//! crate. A declared floor that nothing builds on is exactly the class of
//! prose-vs-reality drift this repo pins from the test suite (the lint-gate
//! and workflow-permissions guards are the precedents).
//!
//! Three values must agree and none may drift alone:
//!
//! 1. `Cargo.toml`'s `rust-version` — what the published metadata promises;
//! 2. the `toolchain:` the `msrv` job installs — what CI actually proves;
//! 3. the job's `name:` (`MSRV <version>`) — the branch-protection
//!    required-context string on `main`, which would gate merges on a stale
//!    number if it lagged, or detach the gate entirely if renamed carelessly.
//!
//! Raising the floor is legal only as one PR editing all three (plus the
//! required-context rename on `main`'s protection); these tests make any
//! lone edit a red check instead of a silent lie.

use std::fs;

const CI_WORKFLOW: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/.github/workflows/ci.yml");
const CARGO_TOML: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/Cargo.toml");

/// The exact check command the job must run. `--all-targets` reaches the
/// dev-dependency floor (criterion declares its own `rust-version`), and
/// `--locked` makes a lockfile the MSRV cannot build a loud failure instead
/// of a silent re-resolve to different versions than the ones that ship.
const MSRV_CHECK_COMMAND: &str = "cargo check --all-targets --all-features --locked";

/// Reads a file with CRLF normalised: this repo's workflow files are CRLF in
/// a default checkout, so LF-anchored matching silently finds nothing.
fn read_normalised(path: &str) -> String {
    fs::read_to_string(path)
        .unwrap_or_else(|e| panic!("failed to read {path}: {e}"))
        .replace("\r\n", "\n")
}

/// The declared MSRV, extracted from `Cargo.toml`'s one `rust-version` line.
/// Hard failure on zero or multiple matches, so a reshuffled manifest can
/// never make this guard pass vacuously.
fn declared_rust_version() -> String {
    let text = read_normalised(CARGO_TOML);
    let values: Vec<&str> = text
        .lines()
        .filter_map(|line| {
            line.strip_prefix("rust-version = \"")
                .and_then(|rest| rest.strip_suffix('"'))
        })
        .collect();
    assert_eq!(
        values.len(),
        1,
        "expected exactly one `rust-version = \"...\"` line in Cargo.toml, \
         found {}: {values:?} — this extractor (and the MSRV gate it feeds) \
         needs updating if the manifest changed shape",
        values.len()
    );
    values[0].to_string()
}

/// Returns the body of the top-level job whose key is `key`, i.e. every line
/// from `  <key>:` up to the next two-space-indented key. Same shape as the
/// lint-gate guard's extractor.
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
fn msrv_job_name_carries_the_declared_rust_version() {
    let version = declared_rust_version();
    let text = read_normalised(CI_WORKFLOW);
    let block = job_block(&text, "msrv");

    let name_line = block
        .lines()
        .find(|line| line.trim_start().starts_with("name:"))
        .unwrap_or_else(|| panic!("the `msrv` job has no `name:` line:\n{block}"));

    assert_eq!(
        name_line.trim(),
        format!("name: MSRV {version}"),
        "the `msrv` job's name must be `MSRV <rust-version>` for the \
         rust-version Cargo.toml declares ({version}); this string is the \
         branch-protection required context on `main`, so a floor raise must \
         rename it here AND in the protection settings, in that order"
    );
}

#[test]
fn msrv_job_installs_exactly_the_declared_rust_version() {
    let version = declared_rust_version();
    let text = read_normalised(CI_WORKFLOW);
    let block = job_block(&text, "msrv");

    assert!(
        block
            .lines()
            .any(|line| line.trim() == format!("toolchain: \"{version}\"")),
        "the `msrv` job must install `toolchain: \"{version}\"`, the exact \
         rust-version Cargo.toml declares — a job proving any other toolchain \
         makes the published metadata a promise nothing checks; job body \
         was:\n{block}"
    );
}

#[test]
fn msrv_job_checks_every_target_against_the_committed_lockfile() {
    let text = read_normalised(CI_WORKFLOW);
    let block = job_block(&text, "msrv");

    assert!(
        block
            .lines()
            .any(|line| line.trim() == format!("run: {MSRV_CHECK_COMMAND}")),
        "the `msrv` job must run `{MSRV_CHECK_COMMAND}` verbatim (dropping \
         `--all-targets` skips the dev-dependency floor, dropping `--locked` \
         lets cargo re-resolve past the exact versions that ship); job body \
         was:\n{block}"
    );
}
