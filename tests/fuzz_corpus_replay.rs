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

use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;

use svccat::manifest::Manifest;
use svccat::policy::{check, PolicyConfig};
use svccat::rules::RuleEngine;
use svccat::urlvalidation::validate_url;

/// The fuzz targets that have a committed seed corpus.
///
/// `fuzz_targets_agree_across_sources` proves this list is the same set as
/// `fuzz/fuzz_targets/`, `fuzz/Cargo.toml`'s `[[bin]]` entries and the
/// `Continuous Fuzzing` workflow matrix, so it cannot silently fall behind.
const TARGETS: [&str; 4] = ["fuzz_manifest", "fuzz_url", "fuzz_glob", "fuzz_policy"];

fn repo_root() -> &'static Path {
    Path::new(env!("CARGO_MANIFEST_DIR"))
}

fn corpus_dir(target: &str) -> PathBuf {
    repo_root().join("fuzz").join("corpus_seeds").join(target)
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

/// Mirror of `FIXTURE_CATALOG` in `fuzz/fuzz_targets/fuzz_policy.rs`: the fixed
/// catalog every fuzzed policy is evaluated against. Covers the three shapes
/// `policy::has_field` discriminates between (fully populated, sparse, and an
/// empty `name`, the only field answered by emptiness rather than by `Option`).
const FIXTURE_CATALOG: &str = r#"
version: "1"
services:
  - name: alpha
    language: rust
    platform: fly
    role: api
    url: https://alpha.example.com
    team: platform
    oncall: platform-oncall
    docs: docs/alpha.md
    ci: .github/workflows/alpha.yml
  - name: beta
    language: go
  - name: ""
"#;

fn fixture_catalog() -> &'static Manifest {
    static CATALOG: OnceLock<Manifest> = OnceLock::new();
    CATALOG.get_or_init(|| {
        serde_yaml::from_str::<Manifest>(FIXTURE_CATALOG).expect("fixture catalog must parse")
    })
}

/// Mirror of `fuzz/fuzz_targets/fuzz_policy.rs`: parse arbitrary bytes as the
/// FILE-BASED policy config (`.svccat/policy.yaml`, `src/policy.rs`) and drive
/// the parsed config through the same evaluation `svccat policy`, `svccat ci`
/// and `svccat scorecard` run. Note this is a different type from the INLINE
/// `manifest.policy` config that `drive_manifest` exercises; the two share the
/// name `PolicyConfig` but nothing else.
fn drive_policy(data: &[u8]) {
    if let Ok(text) = std::str::from_utf8(data) {
        if let Ok(cfg) = serde_yaml::from_str::<PolicyConfig>(text) {
            let _ = cfg.is_empty();
            let report = check(fixture_catalog(), &cfg);
            let _ = report.error_count();
            let _ = report.warning_count();
            let _ = report.passed();
        }
    }
}

/// Read one seed and parse it as a file-based policy config.
fn parse_policy_seed(name: &str) -> Result<PolicyConfig, serde_yaml::Error> {
    let bytes = fs::read(corpus_dir("fuzz_policy").join(name))
        .unwrap_or_else(|e| panic!("policy seed {name} is missing: {e}"));
    let text = std::str::from_utf8(&bytes).expect("policy seed must be valid utf-8");
    serde_yaml::from_str::<PolicyConfig>(text)
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
fn policy_seeds_replay_without_aborting() {
    for (name, bytes) in read_seeds("fuzz_policy") {
        drive_policy(&bytes);
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

#[test]
fn policy_corpus_evaluates_to_the_expected_violation_counts() {
    // Non-tautological companion to `policy_seeds_replay_without_aborting`
    // (which only proves nothing aborts): these counts pin `policy::has_field`'s
    // actual semantics against the fixture catalog, so a change to which field
    // names it recognises — or to the empty-`name` rule, the one field answered
    // by emptiness rather than by `Option::is_some` — fails here instead of
    // silently changing what `svccat policy` reports.
    //
    // Fixture: `alpha` declares every known field, `beta` declares only
    // `language`, and the third entry has an empty `name` and nothing else.
    for (seed, errors, warnings) in [
        // required [team, oncall]: alpha 0, beta 2, empty-name 2.
        // recommended [language, platform, docs]: alpha 0, beta 2, empty-name 3.
        ("valid_basic", 4, 5),
        // required = all 9 known fields: alpha 0, beta 7, empty-name 9
        // (`name` counts as missing because it is the empty string).
        ("all_known_fields", 16, 0),
        // Nothing `has_field` recognises: every unknown name is a violation on
        // every service (3 required x 3 services, 2 recommended x 3 services).
        ("unknown_field_names", 9, 6),
        // YAML aliases must expand, so `recommended` is the same [team, oncall]
        // list as `required`: 4 errors and the same 4 again as warnings.
        ("anchor_alias", 4, 4),
    ] {
        let cfg = parse_policy_seed(seed)
            .unwrap_or_else(|e| panic!("policy seed {seed} must parse: {e}"));
        let report = check(fixture_catalog(), &cfg);
        assert_eq!(
            (report.error_count(), report.warning_count()),
            (errors, warnings),
            "policy seed {seed} evaluated to unexpected (errors, warnings)"
        );
        assert_eq!(
            report.passed(),
            errors == 0,
            "policy seed {seed}: passed() must track error_count()"
        );
    }
}

#[test]
fn malformed_policy_seeds_fail_to_parse_rather_than_panicking() {
    // These three seeds are the reason a policy fuzz target is worth having:
    // each is a plausible hand-edit of `.svccat/policy.yaml` that the
    // deserializer rejects. The library contract is a returned `Err`, never a
    // panic. NOTE what the CALLERS then do with that `Err` is a separate,
    // filed defect: `PolicyConfig::load` swallows it and returns `None`, so
    // every one of these files is indistinguishable from having no policy file
    // at all (`svccat policy` prints "No policy file found", `svccat ci` skips
    // the policy step, `svccat scorecard` scores without policy). This test
    // pins the parser half only; it deliberately does not bless the swallow.
    for seed in ["malformed_yaml", "required_not_a_list", "null_values"] {
        assert!(
            parse_policy_seed(seed).is_err(),
            "policy seed {seed} is malformed and must deserialize to Err"
        );
    }
}

#[test]
fn lenient_policy_seeds_still_parse() {
    // The complement of the test above, so "rejects everything" could never
    // pass both: unknown top-level keys are ignored (there is no
    // `deny_unknown_fields`), and duplicate entries are kept verbatim.
    let cfg = parse_policy_seed("unknown_top_level_keys")
        .expect("unknown top-level keys are ignored, not rejected");
    assert_eq!(cfg.required, ["team"]);
    let cfg =
        parse_policy_seed("duplicate_entries").expect("duplicate field entries must still parse");
    assert_eq!(
        cfg.required.len(),
        3,
        "duplicates are preserved, not deduped"
    );
}

/// Target names declared as `[[bin]]` entries in `fuzz/Cargo.toml`.
fn targets_in_fuzz_manifest() -> BTreeSet<String> {
    let text = fs::read_to_string(repo_root().join("fuzz").join("Cargo.toml"))
        .expect("fuzz/Cargo.toml is readable");
    let mut in_bin = false;
    let mut found = BTreeSet::new();
    for line in text.lines() {
        let line = line.trim();
        if line.starts_with('[') {
            in_bin = line == "[[bin]]";
            continue;
        }
        if in_bin {
            if let Some(rest) = line.strip_prefix("name") {
                if let Some(value) = rest.split('"').nth(1) {
                    found.insert(value.to_string());
                }
            }
        }
    }
    found
}

/// Target names in the `Continuous Fuzzing` workflow's job matrix.
fn targets_in_fuzzing_workflow() -> BTreeSet<String> {
    let text = fs::read_to_string(
        repo_root()
            .join(".github")
            .join("workflows")
            .join("fuzzing.yml"),
    )
    .expect(".github/workflows/fuzzing.yml is readable");
    let list = text
        .lines()
        .map(str::trim)
        .find_map(|line| line.strip_prefix("target: [")?.strip_suffix(']'))
        .expect("fuzzing.yml must declare a `target: [...]` matrix");
    list.split(',')
        .map(|t| t.trim().to_string())
        .filter(|t| !t.is_empty())
        .collect()
}

#[test]
fn fuzz_targets_agree_across_sources() {
    // The drift guard. A fuzz target can exist as a file, build as a `[[bin]]`,
    // and still never be fuzzed by anything if it is missing from the workflow
    // matrix — an inert surface indistinguishable from a green run, which is
    // exactly what this repo's fuzzing setup was before PR #15. Reading all
    // four sources here means adding a target and forgetting any one of them
    // fails the PR gate, instead of quietly shipping a target nobody runs.
    let expected: BTreeSet<String> = TARGETS.iter().map(|t| t.to_string()).collect();

    let files: BTreeSet<String> = fs::read_dir(repo_root().join("fuzz").join("fuzz_targets"))
        .expect("fuzz/fuzz_targets is readable")
        .filter_map(|entry| {
            let path = entry.ok()?.path();
            if path.extension()? != "rs" {
                return None;
            }
            Some(path.file_stem()?.to_string_lossy().into_owned())
        })
        .collect();
    assert_eq!(files, expected, "fuzz/fuzz_targets/*.rs vs TARGETS");

    assert_eq!(
        targets_in_fuzz_manifest(),
        expected,
        "fuzz/Cargo.toml [[bin]] entries vs TARGETS"
    );
    assert_eq!(
        targets_in_fuzzing_workflow(),
        expected,
        "Continuous Fuzzing matrix vs TARGETS — a target missing here is never fuzzed"
    );

    let corpora: BTreeSet<String> = fs::read_dir(repo_root().join("fuzz").join("corpus_seeds"))
        .expect("fuzz/corpus_seeds is readable")
        .filter_map(|entry| {
            let path = entry.ok()?.path();
            if !path.is_dir() {
                return None;
            }
            Some(path.file_name()?.to_string_lossy().into_owned())
        })
        .collect();
    assert_eq!(corpora, expected, "fuzz/corpus_seeds/* vs TARGETS");
}
