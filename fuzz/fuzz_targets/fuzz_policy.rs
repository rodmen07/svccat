#![no_main]
use libfuzzer_sys::fuzz_target;
use std::sync::OnceLock;
use svccat::manifest::Manifest;
use svccat::policy::{check, PolicyConfig};

/// A fixed, representative catalog the fuzzed policy is evaluated against.
///
/// Deliberately covers the three shapes `policy::has_field` discriminates
/// between: a service that declares every known field, a sparse one that
/// declares almost none, and one whose `name` is the empty string (the only
/// field `has_field` answers by emptiness rather than by `Option`).
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

fuzz_target!(|data: &[u8]| {
    // Drives the FILE-BASED policy config (`.svccat/policy.yaml`, `src/policy.rs`),
    // which no other target reaches: `fuzz_manifest` only exercises the INLINE
    // `policy.rules` list embedded in the service manifest (`src/manifest.rs`'s
    // `PolicyConfig`, compiled by `src/rules.rs`). Two different types share the
    // name `PolicyConfig`; this one is the flat required/recommended field lists
    // that `svccat policy`, `svccat ci` and `svccat scorecard` load from disk.
    //
    // The fuzzed entry point is the deserialization + evaluation pipeline
    // `PolicyConfig::load` delegates to, not `load` itself: `load` takes a
    // `&Path` and does its own file I/O, so fuzzing it directly would fuzz the
    // filesystem rather than the parser. Everything `load` can reach on
    // untrusted bytes is reached here.
    if let Ok(text) = std::str::from_utf8(data) {
        if let Ok(cfg) = serde_yaml::from_str::<PolicyConfig>(text) {
            // `is_empty` is the gate every caller checks before doing any work.
            let _ = cfg.is_empty();
            // Then the evaluation itself: arbitrary field names flow into
            // `has_field`'s match, and arbitrary service/field strings flow into
            // the violation `format!`s.
            let report = check(fixture_catalog(), &cfg);
            let _ = report.error_count();
            let _ = report.warning_count();
            let _ = report.passed();
        }
    }
});
