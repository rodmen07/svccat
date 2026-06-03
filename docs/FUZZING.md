# Fuzzing svccat

This document describes how to set up and run fuzz testing for svccat to find edge cases and potential panics.

## Overview

Fuzzing tests svccat's parsers and validators with randomly generated inputs to:
- Find parsing bugs
- Detect panic/unwrap calls
- Discover resource exhaustion vulnerabilities
- Test edge cases that unit tests might miss

## Setup

### Prerequisites

Install `cargo-fuzz`:

```bash
cargo install cargo-fuzz
```

### Create Fuzz Targets

Initialize the fuzzing infrastructure:

```bash
cargo fuzz init
```

This creates a `fuzz/` directory with fuzz target templates.

## Fuzz Targets

Create targeted fuzz tests for key components:

### 1. Manifest YAML Parsing

**File:** `fuzz/fuzz_targets/fuzz_manifest.rs`

```rust
#![no_main]
use libfuzzer_sys::fuzz_target;
use svccat::manifest::Manifest;

fuzz_target!(|data: &[u8]| {
    if let Ok(text) = std::str::from_utf8(data) {
        let _ = serde_yaml::from_str::<Manifest>(text);
    }
});
```

Tests manifest loading with arbitrary YAML input.

### 2. URL Validation

**File:** `fuzz/fuzz_targets/fuzz_url.rs`

```rust
#![no_main]
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    if let Ok(url) = std::str::from_utf8(data) {
        let _ = svccat::urlvalidation::validate_url(url, false);
        let _ = svccat::urlvalidation::validate_url(url, true);
    }
});
```

Tests URL validation with arbitrary string input.

### 3. Glob Pattern Processing

**File:** `fuzz/fuzz_targets/fuzz_glob.rs`

```rust
#![no_main]
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    if let Ok(pattern) = std::str::from_utf8(data) {
        // Test glob compilation with arbitrary patterns
        let _ = glob::Pattern::new(pattern);
    }
});
```

Tests glob pattern parsing with arbitrary input.

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

Add fuzzing to GitHub Actions for continuous fuzz testing:

```yaml
name: Continuous Fuzzing

on:
  schedule:
    - cron: '0 2 * * *'  # Daily at 2 AM UTC

jobs:
  fuzz:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4.1.7
      - uses: dtolnay/rust-toolchain@nightly
      - name: Run fuzzers
        run: |
          cargo install cargo-fuzz
          for target in fuzz/fuzz_targets/*.rs; do
            name=$(basename "$target" .rs)
            cargo fuzz run "$name" -- -max_total_time=600
          done
```

## What Gets Fuzzed

### ✅ Current Coverage

- [x] Manifest YAML parsing
- [x] URL validation
- [x] Glob pattern compilation

### 📋 Recommended Coverage

- [ ] Git reference validation (unit tests currently cover this)
- [ ] Path validation (unit tests currently cover this)
- [ ] Dependency graph cycle detection

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

**Last Updated:** May 28, 2026 (v0.19.0)
