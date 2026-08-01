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
//! These files double as libFuzzer seed inputs: the workflow's `cargo fuzz run`
//! step passes `fuzz/corpus_seeds/<target>` as a read-only seed directory, so
//! the daily campaign starts from this corpus instead of an empty one.
//! `fuzzing_workflow_runs_from_the_committed_seed_corpus` below is the PR-time
//! guard on that wiring, since the workflow never runs on `pull_request`.
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

/// The text of `.github/workflows/fuzzing.yml`.
fn fuzzing_workflow() -> String {
    fs::read_to_string(
        repo_root()
            .join(".github")
            .join("workflows")
            .join("fuzzing.yml"),
    )
    .expect(".github/workflows/fuzzing.yml is readable")
}

/// Target names in the `Continuous Fuzzing` workflow's job matrix.
fn targets_in_fuzzing_workflow() -> BTreeSet<String> {
    let text = fuzzing_workflow();
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

/// The `cargo fuzz run` invocation from the `Continuous Fuzzing` workflow, with
/// YAML line continuations and indentation collapsed so it reads as the single
/// shell command the runner actually executes.
fn fuzz_run_command_in_workflow() -> String {
    let text = fuzzing_workflow().replace('\\', " ");
    let flat = text.split_whitespace().collect::<Vec<_>>().join(" ");
    let start = flat
        .find("cargo fuzz run")
        .expect("fuzzing.yml must invoke `cargo fuzz run`");
    let rest = &flat[start..];
    // The command ends at the trailing YAML comment, the next workflow step, or
    // EOF -- whichever comes first. The command itself contains no `#`.
    let end = rest
        .find(" #")
        .or_else(|| rest.find("- name:"))
        .unwrap_or(rest.len());
    rest[..end].trim().to_string()
}

/// The text of `docs/FUZZING.md`.
fn fuzzing_docs() -> String {
    fs::read_to_string(repo_root().join("docs").join("FUZZING.md"))
        .expect("docs/FUZZING.md is readable")
}

/// The `cargo fuzz run` invocation quoted in `docs/FUZZING.md`'s CI Integration
/// section, flattened the same way, so the two can be compared directly.
fn fuzz_run_command_in_docs() -> String {
    let text = fuzzing_docs();
    let section = text
        .split("## CI Integration")
        .nth(1)
        .expect("docs/FUZZING.md must have a `## CI Integration` section");
    let section = section.split("\n## ").next().unwrap_or(section);
    let block = section
        .split("```")
        .nth(1)
        .expect("the CI Integration section must quote the command in a fenced block");
    let block = block.strip_prefix("bash").unwrap_or(block);
    block
        .replace('\\', " ")
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

#[test]
fn fuzzing_workflow_runs_from_the_committed_seed_corpus() {
    // Committing seeds is only half the job: libFuzzer reads a corpus only if
    // it is passed one. Before this guard, every leg of the workflow ran
    // `cargo fuzz run <target> -- -max_total_time=120` with NO corpus argument
    // and the job logs on main showed `INITED ... corp: 1/1b` for all four
    // targets — one synthetic empty input — so the committed regression seeds
    // (the PR #16 base-cycle crash reproducers among them) were replayed by
    // this suite and handed to the actual fuzzer never. That is the same
    // inert-surface shape as a target missing from the matrix, and since
    // `fuzzing.yml` does not run on `pull_request`, nothing at PR time would
    // notice it being undone. This test is that notice.
    let cmd = fuzz_run_command_in_workflow();
    let seeds = "fuzz/corpus_seeds/${{ matrix.target }}";
    let working = "fuzz/corpus/${{ matrix.target }}";

    let seeds_at = cmd.find(seeds).unwrap_or_else(|| {
        panic!(
            "the Continuous Fuzzing run step must pass `{seeds}` to `cargo fuzz run`, \
             or the daily campaign starts from an empty corpus and ignores every \
             committed seed. Command found: {cmd}"
        )
    });
    let working_at = cmd.find(working).unwrap_or_else(|| {
        panic!(
            "the run step must pass the gitignored working corpus `{working}` as well, \
             because libFuzzer WRITES newly discovered inputs into the first corpus \
             directory it is given. Command found: {cmd}"
        )
    });

    assert!(
        working_at < seeds_at,
        "`{working}` must come BEFORE `{seeds}`: libFuzzer writes new inputs into the \
         first corpus directory, so seeds-first would have a CI run mutate the committed \
         corpus this suite pins. Command found: {cmd}"
    );

    let separator = cmd.find(" -- ").unwrap_or(cmd.len());
    assert!(
        seeds_at < separator,
        "`{seeds}` must appear before the `--` separator so cargo-fuzz treats it as a \
         corpus directory; after `--` it would be passed to libFuzzer as a flag. \
         Command found: {cmd}"
    );
}

/// The steps of the `Continuous Fuzzing` job as `(name, body)` pairs in file
/// order, with comment lines dropped so a body is the step's own YAML and never
/// the prose sitting between it and the next step.
///
/// Split on the `- name:` lines rather than pulling a YAML parser into the test
/// suite for one file: the workflow is hand-written, its steps are uniformly
/// indented at six spaces, and nothing in it puts `- name:` inside a value.
fn fuzzing_workflow_steps() -> Vec<(String, String)> {
    let text = fuzzing_workflow().replace("\r\n", "\n");
    let mut steps: Vec<(String, String)> = Vec::new();
    for line in text.lines() {
        if let Some(name) = line.strip_prefix("      - name:") {
            steps.push((name.trim().to_string(), String::new()));
            continue;
        }
        if line.trim_start().starts_with('#') {
            continue;
        }
        if let Some((_, body)) = steps.last_mut() {
            body.push_str(line);
            body.push('\n');
        }
    }
    steps
}

/// The scalar value of `field:` inside a step body, e.g. `uses` or `key`.
fn step_field(body: &str, field: &str) -> Option<String> {
    let needle = format!("{field}:");
    body.lines()
        .find_map(|line| Some(line.trim().strip_prefix(&needle)?.trim().to_string()))
}

/// The entries of a step's `restore-keys: |` block scalar.
fn step_restore_keys(body: &str) -> Vec<String> {
    let mut lines = body
        .lines()
        .skip_while(|l| !l.trim().starts_with("restore-keys:"));
    let header = match lines.next() {
        Some(h) => h,
        None => return Vec::new(),
    };
    let indent = header.len() - header.trim_start().len();
    lines
        .take_while(|l| !l.trim().is_empty() && (l.len() - l.trim_start().len()) > indent)
        .map(|l| l.trim().to_string())
        .collect()
}

/// Index of the step whose body `uses:` the given action, or whose name matches.
fn step_index(steps: &[(String, String)], predicate: impl Fn(&str, &str) -> bool) -> Option<usize> {
    steps.iter().position(|(name, body)| predicate(name, body))
}

#[test]
fn fuzzing_workflow_carries_the_working_corpus_between_runs() {
    // The working corpus (`fuzz/corpus/<target>`) is gitignored scratch space
    // that libFuzzer writes every newly discovered coverage-increasing input
    // into. For as long as it was per-job scratch, the daily campaign threw all
    // of it away and re-explored the same ground from the same starting point:
    // the scheduled runs a week apart (2026-07-25 and 2026-07-31) report the
    // IDENTICAL `INITED cov:` figures for all four targets. That is an inert
    // surface of the same shape as the seeds nobody passed and the target
    // nobody matrixed -- green, expensive, and cumulatively worth nothing --
    // and since `fuzzing.yml` never runs on `pull_request`, this test is the
    // only thing at PR time that would notice it being undone.
    let steps = fuzzing_workflow_steps();
    let path = "fuzz/corpus/${{ matrix.target }}";

    let restore_at = step_index(&steps, |_, body| {
        step_field(body, "uses").is_some_and(|u| u.starts_with("actions/cache/restore@"))
    })
    .unwrap_or_else(|| {
        panic!(
            "the Continuous Fuzzing job must restore the working corpus with \
             `actions/cache/restore`, or every run starts from the committed seeds \
             alone and 120 seconds of campaign per target is discarded daily. \
             Steps found: {:?}",
            steps.iter().map(|(n, _)| n).collect::<Vec<_>>()
        )
    });
    let save_at = step_index(&steps, |_, body| {
        step_field(body, "uses").is_some_and(|u| u.starts_with("actions/cache/save@"))
    })
    .expect("the job must SAVE the working corpus too; restoring alone carries nothing forward");
    let run_at = step_index(&steps, |_, body| body.contains("cargo fuzz run"))
        .expect("fuzzing.yml must invoke `cargo fuzz run`");

    let restore = &steps[restore_at].1;
    let save = &steps[save_at].1;

    for (what, body) in [("restore", restore), ("save", save)] {
        assert_eq!(
            step_field(body, "path").as_deref(),
            Some(path),
            "the {what} step must cache exactly `{path}`, the gitignored WORKING corpus"
        );
        assert!(
            !body.contains("corpus_seeds"),
            "the {what} step must not touch `fuzz/corpus_seeds`: the committed corpus is the \
             reproducible floor this suite pins, and a cache entry restored over it would let \
             an evicted or poisoned cache silently replace the regression seeds"
        );
    }

    let restore_key = step_field(restore, "key").expect("the restore step must declare a `key`");
    let save_key = step_field(save, "key").expect("the save step must declare a `key`");
    assert_eq!(
        restore_key, save_key,
        "restore and save must use the SAME cache key, or what a run banks is never what the \
         next run looks for and the carry-over is a silent no-op"
    );
    assert!(
        restore_key.contains("${{ github.run_id }}"),
        "the cache key must be unique per run (`${{{{ github.run_id }}}}`): saving onto an \
         existing key is a no-op, so a fixed key would freeze the corpus at whatever the first \
         run happened to find. Key found: {restore_key}"
    );
    assert!(
        restore_key.contains("${{ matrix.target }}"),
        "the cache key must name the target, or four jobs would fight over one entry and each \
         would restore another target's corpus. Key found: {restore_key}"
    );

    let restore_keys = step_restore_keys(restore);
    let prefix = restore_keys.first().unwrap_or_else(|| {
        panic!(
            "the restore step must declare `restore-keys`: the exact key carries this run's id \
             and therefore never exists yet, so without a prefix fallback EVERY run is a cache \
             miss and the carry-over never happens"
        )
    });
    assert!(
        restore_key.starts_with(prefix) && prefix.len() < restore_key.len(),
        "the `restore-keys` prefix `{prefix}` must be a strict prefix of the key `{restore_key}`, \
         or it can never match a previous run's entry"
    );

    assert!(
        restore_at < run_at && run_at < save_at,
        "order must be restore -> fuzz -> save (found restore at {restore_at}, run at {run_at}, \
         save at {save_at}); restoring after the campaign, or saving before it, banks nothing"
    );
    assert_eq!(
        step_field(save, "if").as_deref(),
        Some("always()"),
        "the save step must run with `if: always()`, so a run that ends on a crash still banks \
         the inputs that led up to it instead of discarding the campaign that found the bug"
    );
}

#[test]
fn docs_quote_the_workflows_actual_fuzz_command() {
    // The drift guard for the prose half. `docs/FUZZING.md` quotes the command
    // CI runs, and it quoted the pre-seed-corpus one-liner for exactly as long
    // as the seeds went unused -- a reader following the doc would have
    // reproduced the empty-corpus run and seen nothing wrong. Reading BOTH
    // sources here means the next change to the run step has to bring the doc
    // with it instead of leaving a plausible, wrong command behind.
    let workflow = fuzz_run_command_in_workflow().replace("${{ matrix.target }}", "<target>");
    let docs = fuzz_run_command_in_docs();
    assert_eq!(
        docs, workflow,
        "docs/FUZZING.md's CI Integration section quotes a `cargo fuzz run` command that is \
         not the one .github/workflows/fuzzing.yml actually runs"
    );
}

/// Every path under `fuzz/` that `docs/FUZZING.md` names, reduced to its first
/// component below `fuzz/` (`fuzz/corpus_seeds/<target>/` contributes
/// `corpus_seeds`, `fuzz/Cargo.toml` contributes `Cargo.toml`).
///
/// A bare `fuzz/` only counts when it starts a path token, so `cargo-fuzz`,
/// `rust-fuzz.github.io` and any deeper `.../fuzz/` segment are skipped.
///
/// Segments carrying no alphanumeric character are prose, not paths: the doc
/// says "every `fuzz/...` path named below", and `...` is an ellipsis. This is
/// not a cosmetic filter — Windows normalizes away trailing dots, so
/// `fuzz/...` reports `exists() == true` there and `false` on Linux and macOS,
/// which is exactly how the first version of this guard passed locally and
/// failed on two of the three `Build & Test (This Checkout)` legs.
fn fuzz_paths_named_in_docs() -> BTreeSet<String> {
    let text = fuzzing_docs();
    let bytes = text.as_bytes();
    let mut found = BTreeSet::new();
    let mut from = 0;
    while let Some(rel) = text[from..].find("fuzz/") {
        let at = from + rel;
        from = at + "fuzz/".len();
        if at > 0 {
            let prev = bytes[at - 1] as char;
            if prev.is_ascii_alphanumeric() || matches!(prev, '-' | '_' | '.' | '/') {
                continue;
            }
        }
        let segment: String = text[from..]
            .chars()
            .take_while(|c| c.is_ascii_alphanumeric() || matches!(c, '_' | '-' | '.'))
            .collect();
        // Trailing dots are sentence punctuation, never part of a real path here.
        let segment = segment.trim_end_matches('.');
        if segment.chars().any(|c| c.is_ascii_alphanumeric()) {
            found.insert(segment.to_string());
        }
    }
    found
}

/// The real entries of the `fuzz/` directory.
///
/// Deliberately a `read_dir` listing rather than a `Path::exists()` probe per
/// name: `exists()` runs through the platform's path normalizer, and Windows
/// strips trailing dots, so a bogus `fuzz/...` "exists" there while failing on
/// Linux and macOS. Comparing against a listing behaves identically everywhere.
fn committed_fuzz_entries() -> BTreeSet<String> {
    fs::read_dir(repo_root().join("fuzz"))
        .expect("fuzz/ is readable")
        .filter_map(|entry| Some(entry.ok()?.file_name().to_string_lossy().into_owned()))
        .collect()
}

/// Entries `fuzz/.gitignore` excludes, i.e. the generated paths that legitimately
/// do not exist in a fresh checkout.
fn gitignored_fuzz_entries() -> BTreeSet<String> {
    let text = fs::read_to_string(repo_root().join("fuzz").join(".gitignore"))
        .expect("fuzz/.gitignore is readable");
    text.lines()
        .map(str::trim)
        .filter(|line| !line.is_empty() && !line.starts_with('#'))
        .map(|line| line.trim_matches('/').to_string())
        .collect()
}

#[test]
fn docs_only_reference_fuzz_paths_that_exist() {
    // The `Best Practices` section told readers to drop interesting inputs into
    // `fuzz/seeds/` for as long as this document existed. That directory has
    // never existed in this repo -- the seed corpus lives in
    // `fuzz/corpus_seeds/<target>/` -- so anyone following the advice added
    // files nothing reads: not cargo-fuzz, not the workflow, not this suite.
    // A wrong path in a doc is the quietest kind of inert surface, so pin every
    // `fuzz/...` path the prose names to something that is either committed or
    // deliberately generated (per `fuzz/.gitignore`, read here rather than
    // hardcoded so the two cannot drift).
    let gitignored = gitignored_fuzz_entries();
    let committed = committed_fuzz_entries();

    let named = fuzz_paths_named_in_docs();
    assert!(
        named.contains("corpus_seeds"),
        "docs/FUZZING.md must name the real seed corpus path fuzz/corpus_seeds/, \
         otherwise this guard is vacuous. Paths found: {named:?}"
    );

    for name in &named {
        assert!(
            committed.contains(name) || gitignored.contains(name),
            "docs/FUZZING.md points at `fuzz/{name}`, which is neither committed in \
             fuzz/ nor listed in fuzz/.gitignore as a generated path. Either the doc \
             names a directory that does not exist (the `fuzz/seeds/` mistake) or a \
             real path was removed without updating the doc. Generated paths: \
             {gitignored:?}"
        );
    }
}

/// Fuzz target names the `## Fuzz Targets` section of `docs/FUZZING.md`
/// documents, taken from the `fuzz/fuzz_targets/<name>.rs` file path each
/// subsection cites.
fn targets_documented_in_docs() -> BTreeSet<String> {
    let text = fuzzing_docs();
    let section = text
        .split("## Fuzz Targets")
        .nth(1)
        .expect("docs/FUZZING.md must have a `## Fuzz Targets` section");
    let section = section.split("\n## ").next().unwrap_or(section);

    let mut found = BTreeSet::new();
    let needle = "fuzz/fuzz_targets/";
    let mut from = 0;
    while let Some(rel) = section[from..].find(needle) {
        from = from + rel + needle.len();
        let name: String = section[from..]
            .chars()
            .take_while(|c| c.is_ascii_alphanumeric() || *c == '_')
            .collect();
        if !name.is_empty() {
            found.insert(name);
        }
    }
    found
}

#[test]
fn docs_describe_every_fuzz_target() {
    // `fuzz_targets_agree_across_sources` pins the four machine-readable
    // sources (files, [[bin]] entries, workflow matrix, seed corpora) to each
    // other, but the prose was not one of them: a fifth target could ship,
    // build, run in CI and carry seeds while `docs/FUZZING.md` still described
    // four, and nothing would fail. Reading the doc here makes the prose a
    // source like any other, so adding or removing a target has to bring its
    // documentation along.
    let expected: BTreeSet<String> = TARGETS.iter().map(|t| t.to_string()).collect();
    assert_eq!(
        targets_documented_in_docs(),
        expected,
        "the `## Fuzz Targets` section of docs/FUZZING.md must cite \
         fuzz/fuzz_targets/<name>.rs for exactly the targets that exist"
    );
}
