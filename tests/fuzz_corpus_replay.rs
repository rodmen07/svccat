//! Replay of the committed fuzz seed / regression corpora through the exact
//! library entry points the cargo-fuzz targets in `fuzz/fuzz_targets/` exercise.
//!
//! Why this suite exists: the Continuous Fuzzing workflow
//! (`.github/workflows/fuzzing.yml`) only runs on push/schedule/dispatch, never
//! on `pull_request` — fuzzing is a background campaign, not a per-PR gate, so a
//! multi-minute-per-target budget does not slow anyone's merge. That leaves a
//! real gap: nothing on a PR proves that inputs a past fuzz run already found
//! interesting — crash reproducers in particular — are still handled gracefully
//! by the current code. This suite closes the gap by replaying every committed
//! seed under `fuzz/corpus_seeds/<target>/` through the same call its matching
//! fuzz target makes, asserting the process does not abort and pinning the known
//! crash reproducers to a graceful `Err` (never a stack overflow / SIGABRT).
//!
//! These files double as libFuzzer seed inputs: pointing a `cargo fuzz run` at
//! `fuzz/corpus_seeds/<target>` gives the fuzzer a meaningful starting corpus
//! instead of an empty one.
//!
//! Each `drive_*` helper is a byte-for-byte mirror of the body of the fuzz
//! target with the same name; keep them in sync when a target changes.

use std::fs;
use std::path::{Path, PathBuf};

use svccat::manifest::Manifest;
use svccat::rules::RuleEngine;
use svccat::urlvalidation::validate_url;

/// The three fuzz targets that have a committed seed corpus.
const TARGETS: [&str; 3] = ["fuzz_manifest", "fuzz_url", "fuzz_glob"];

fn corpus_dir(target: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("fuzz")
        .join("corpus_seeds")
        .join(target)
}

/// Read every regular file in a corpus target directory as raw bytes, returning
/// `(file_name, bytes)` pairs sorted by name so the replay order is stable.
fn read_seeds(target: &str) -> Vec<(String, Vec<u8>)> {
    let dir = corpus_dir(target);
    let mut seeds: Vec<(String, Vec<u8>)> = fs::read_dir(&dir)
        .unwrap_or_else(|e| panic!("corpus dir {} is unreadable: {e}", dir.display()))
        .filter_map(|entry| {
            let path = entry.ok()?.path();
            if path.is_file() {
                let name = path.file_name()?.to_string_lossy().into_owned();
                Some((name, fs::read(&path).ok()?))
            } else {
                None
            }
        })
        .collect();
    seeds.sort_by(|a, b| a.0.cmp(&b.0));
    seeds
}

/// Mirror of `fuzz/fuzz_targets/fuzz_manifest.rs`: parse arbitrary bytes as a
/// manifest, then drive the parsed inline `policy.rules` through the rule
/// compiler exactly as `svccat check` / `workspace check` do via `src/drift.rs`.
fn drive_manifest(data: &[u8]) {
    if let Ok(text) = std::str::from_utf8(data) {
        if let Ok(manifest) = serde_yaml::from_str::<Manifest>(text) {
            let _ = RuleEngine::compile(&manifest.policy.rules);
        }
    }
}

/// Mirror of `fuzz/fuzz_targets/fuzz_url.rs`.
fn drive_url(data: &[u8]) {
    if let Ok(url) = std::str::from_utf8(data) {
        let _ = validate_url(url, false);
        let _ = validate_url(url, true);
    }
}

/// Mirror of `fuzz/fuzz_targets/fuzz_glob.rs`.
fn drive_glob(data: &[u8]) {
    if let Ok(pattern) = std::str::from_utf8(data) {
        let _ = glob::Pattern::new(pattern);
    }
}

#[test]
fn every_target_has_a_nonempty_seed_corpus() {
    for target in TARGETS {
        let seeds = read_seeds(target);
        assert!(
            !seeds.is_empty(),
            "seed corpus for {target} is empty; committed seeds guard the fuzz \
             target against regressions and seed the daily fuzz run"
        );
    }
}

#[test]
fn manifest_seeds_replay_without_aborting() {
    // A panic fails this test; a stack overflow / SIGABRT (the pre-PR#16
    // base-cycle behaviour) aborts the whole process, which is exactly the
    // regression this replay is here to catch between the daily fuzz runs.
    for (name, bytes) in read_seeds("fuzz_manifest") {
        drive_manifest(&bytes);
        // Reference `name` so a future `--nocapture` run identifies each seed.
        let _ = name;
    }
}

#[test]
fn url_seeds_replay_without_aborting() {
    for (name, bytes) in read_seeds("fuzz_url") {
        drive_url(&bytes);
        let _ = name;
    }
}

#[test]
fn glob_seeds_replay_without_aborting() {
    for (name, bytes) in read_seeds("fuzz_glob") {
        drive_glob(&bytes);
        let _ = name;
    }
}

#[test]
fn base_cycle_manifest_seeds_compile_to_err_not_a_crash() {
    // These seeds reproduce the HIGH-severity crash fixed in svccat PR #16:
    // `RuleEngine::resolve_rule` recursed over each rule's `base` chain with no
    // cycle guard and stack-overflowed the process (STATUS_STACK_OVERFLOW /
    // SIGABRT) instead of returning an `Err`. The fix makes a cyclic chain
    // resolve to a normal error. Asserting `Err` here (not merely "did not
    // panic") is the non-tautological regression guard: reverting the fix makes
    // this abort the process rather than fail an assertion.
    for name in ["base_cycle_self", "base_cycle_mutual"] {
        let bytes = fs::read(corpus_dir("fuzz_manifest").join(name))
            .unwrap_or_else(|e| panic!("crash-reproducer seed {name} is missing: {e}"));
        let text = std::str::from_utf8(&bytes).expect("crash seed must be valid utf-8");
        let manifest: Manifest = serde_yaml::from_str(text)
            .unwrap_or_else(|e| panic!("crash seed {name} must parse as a manifest: {e}"));
        assert!(
            RuleEngine::compile(&manifest.policy.rules).is_err(),
            "cyclic policy base chain in {name} must compile to Err \
             (was a process-aborting stack overflow before PR #16)"
        );
    }
}

#[test]
fn url_corpus_rejects_private_and_loopback_targets() {
    // Pin the SSRF-relevant classifications the fuzz_url target explores: a
    // private (RFC 1918) or loopback IP literal must be rejected by
    // `validate_url`. A change that quietly started allowing them would fail
    // here even though the daily fuzz run (which only asserts "no panic") would
    // not notice.
    for name in ["private_ip_10", "loopback_127", "ipv6_loopback"] {
        let bytes = fs::read(corpus_dir("fuzz_url").join(name))
            .unwrap_or_else(|e| panic!("url seed {name} is missing: {e}"));
        let url = std::str::from_utf8(&bytes).unwrap();
        assert!(
            validate_url(url, false).is_err(),
            "{name} ({url}) is a private/loopback target and must be rejected"
        );
    }
}
