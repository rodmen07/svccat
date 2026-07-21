# Fuzzing svccat

This document describes how to set up and run fuzz testing for svccat to find edge cases and potential panics.

**Status (2026-07-20): this infrastructure is real and wired into CI.** Earlier
revisions of this document described a `cargo fuzz init` setup that had not
actually been done — `fuzz/Cargo.toml` did not exist (a `Cargo.fuzz.toml` sat
at the repo root instead, a path `cargo fuzz` never looks at, and it also
referenced a Cargo feature, `__fuzz_target`, that did not exist in
`Cargo.toml`), and `.github/workflows/fuzzing.yml`'s run step was a
placeholder (`echo "Executing ..."` with the real invocation commented out)
behind a matrix of engines (`libfuzzer`/`afl`/`honggfuzz`) none of which were
ever actually invoked. The workflow reported green on every run regardless,
because nothing in it could fail. Both gaps are now fixed: `fuzz/Cargo.toml`
exists at the path cargo-fuzz expects, and the CI job in "Fuzz Targets" and
"CI Integration" below runs for real. If you are reading a future revision of
this file and it drifts from `fuzz/Cargo.toml` or `fuzzing.yml` again, trust
those files, not this prose.

## Overview

Fuzzing tests svccat's parsers and validators with randomly generated inputs to:
- Find parsing bugs
- Detect panic/unwrap calls
- Discover resource exhaustion vulnerabilities
- Test edge cases that unit tests might miss

## Setup

### Prerequisites

Install `cargo-fuzz` (needs a nightly toolchain for libFuzzer instrumentation):

```bash
rustup toolchain install nightly
cargo install cargo-fuzz
```

### Fuzzing Infrastructure

Already set up in this repo: `fuzz/Cargo.toml` (a standalone, workspace-detached
crate depending on `svccat` via `path = ".."`) plus the target binaries below in
`fuzz/fuzz_targets/`. Nothing further to initialize; running
`cargo +nightly fuzz run <target>` from the repo root just works.

## Fuzz Targets

### 1. Manifest YAML parsing + inline policy rule compilation

**File:** `fuzz/fuzz_targets/fuzz_manifest.rs`

Parses arbitrary bytes as a `Manifest` (the same `serde_yaml`/`Deserialize`
path `Manifest::load` uses), and — when parsing succeeds — feeds the parsed
`policy.rules` straight into `RuleEngine::compile`, exactly as `svccat
check`/`workspace check` do via `src/drift.rs`. This second step is
deliberate, not incidental: `RuleEngine::resolve_rule` (`src/rules.rs`)
recurses over each rule's `base` chain with **no cycle guard**, so a
self- or mutually-referential `base` stack-overflows the process instead of
returning an `Err`. `svccat lint` gained a pre-compile cycle check
(`src/rule_schema.rs`) for this exact bug, but `check`/`workspace check` call
`RuleEngine::compile` directly and are not covered by that guard — so this
remains a real, reachable crash from an untrusted manifest file, and fuzzing
the parse-then-compile pipeline together is what lets this target find that
whole bug class automatically rather than only fuzzing YAML shape.

### 2. URL validation

**File:** `fuzz/fuzz_targets/fuzz_url.rs`

Feeds arbitrary strings into `svccat::urlvalidation::validate_url` (both the
`allow_localhost=false` and `=true` modes), the trust-boundary check used
before `--ping`/webhook requests go out (see `src/safe_http.rs` for the
redirect-hardening layered on top of this).

### 3. Glob pattern processing

**File:** `fuzz/fuzz_targets/fuzz_glob.rs`

Feeds arbitrary strings into `glob::Pattern::new`, the same pattern-compile
call `src/discovery.rs` makes for every `discovery.ignore` / `--ignore`
pattern read from a manifest or CLI flag.

## Running Fuzzing

### Run a Single Fuzzer

```bash
# Run fuzz_manifest for 1 hour
cargo fuzz run fuzz_manifest -- -max_total_time=3600

# Run with specific seed
cargo fuzz run fuzz_manifest -- fuzz/artifacts/fuzz_manifest/seed1

# Run with custom timeout (in seconds)
cargo fuzz run fuzz_manifest -- -timeout=5
```

### Run All Fuzzers

```bash
for target in fuzz/fuzz_targets/*.rs; do
    name=$(basename "$target" .rs)
    echo "Fuzzing $name..."
    cargo fuzz run "$name" -- -max_total_time=300
done
```

### Generate Coverage Report

```bash
# Build with coverage instrumentation
CARGO_FUZZ=1 cargo fuzz cov fuzz_manifest

# Generate HTML report
cargo fuzz cov fuzz_manifest -- --format=html
```

## Interpreting Results

### Crash Found

If fuzzing finds a crash:

1. **Examine the crash**: Check the crash message and stack trace
2. **Locate the artifact**: Saved in `fuzz/artifacts/[target]/crash-*`
3. **Reproduce**: Run the fuzzer with that artifact to reproduce
4. **Fix**: Update the code to handle the case gracefully
5. **Add regression test**: Add a unit test to prevent regression

Example:

```bash
# Crash found at:
ls fuzz/artifacts/fuzz_manifest/crash-3d41d6e74dcf6

# Reproduce with:
cargo fuzz run fuzz_manifest fuzz/artifacts/fuzz_manifest/crash-3d41d6e74dcf6

# After fixing, add a test case
```

### Leak Found

If fuzzing detects memory leaks or resource exhaustion:

1. **Check resource limits**: Ensure limits are enforced (manifest size, service count, etc.)
2. **Review allocations**: Look for unbounded Vec/HashMap/String growth
3. **Add limits**: Implement safeguards if missing
4. **Test fix**: Verify fuzzer no longer reports the leak

## CI Integration

`.github/workflows/fuzzing.yml` ("Continuous Fuzzing") runs this for real, on
`push` to `main`, once a day (`0 0 * * *`), and on manual
`workflow_dispatch`. It does **not** run on `pull_request` — fuzzing is a
push/schedule/manual concern, not a per-PR gate, so it never adds time to a
merge. It matrices the three real targets above, one job per target, each
running `cargo fuzz run <target> -- -max_total_time=120` (a 120-second
libFuzzer budget — enough to catch cheap, shallow crashes like the
unguarded recursive `base` chain above on every push and once a day, without
turning this into a multi-hour runner) against a nightly toolchain, and
uploads the crashing input as a build artifact on failure so it can be
downloaded and reproduced locally. Read the actual workflow file for the
current exact steps; it is the source of truth, not this paragraph.

## What Gets Fuzzed

### ✅ Current Coverage (and actually running in CI, not just described here)

- [x] Manifest YAML parsing
- [x] Inline policy rule compilation (`policy.rules` / `RuleEngine::compile`, including the `base`-chain cycle crash)
- [x] URL validation
- [x] Glob pattern compilation

### 📋 Recommended Coverage

- [ ] Git reference validation (unit tests currently cover this)
- [ ] Path validation (unit tests currently cover this)
- [ ] Dependency graph cycle detection (`src/deps_graph.rs` — a different cycle-detection surface than the policy-rule `base` chain above)
- [ ] A dedicated `fuzz_policy` target with structured (`arbitrary`-derived) `Rule`/`Vec<Rule>` generation and seed corpora, rather than reaching `RuleEngine::compile` only indirectly through YAML manifest text (tracked in the crate's roadmap as a follow-up, alongside seed corpora for the three existing targets)

## Best Practices

1. **Run regularly**: Schedule weekly or daily fuzzing runs
2. **Keep fuzzing long**: Longer fuzzing sessions find more bugs
3. **Monitor memory**: Watch for unbounded memory growth
4. **Add seed files**: Add interesting/edge-case inputs to `fuzz/seeds/`
5. **Review crashes**: Every crash deserves investigation and a regression test
6. **Update dependencies**: Regularly update libfuzzer-sys for better instrumentation

## Resources

- [cargo-fuzz documentation](https://rust-fuzz.github.io/book/cargo-fuzz.html)
- [libFuzzer guide](https://llvm.org/docs/LibFuzzer/)
- [Rust fuzzing book](https://rust-fuzz.github.io/book/)

## Troubleshooting

### "libfuzzer-sys: LLVM not found"

Install LLVM:

```bash
# macOS
brew install llvm

# Ubuntu
sudo apt-get install llvm

# Windows
# Download from https://releases.llvm.org/
```

### Fuzzer reports "timeout"

Increase the timeout or optimize slow code:

```bash
cargo fuzz run fuzz_manifest -- -timeout=10  # 10 seconds
```

### Out of memory during fuzzing

Limit memory usage:

```bash
cargo fuzz run fuzz_manifest -- -rss_limit_mb=512  # 512 MB limit
```

---

**Last Updated:** July 20, 2026 — `fuzz/Cargo.toml` and the CI workflow now
actually run cargo-fuzz; earlier revisions of this document described
infrastructure that did not exist yet.
