#![no_main]
use libfuzzer_sys::fuzz_target;
use svccat::manifest::Manifest;
use svccat::rules::RuleEngine;

fuzz_target!(|data: &[u8]| {
    // Try to parse arbitrary input as a manifest.
    // This will find panic! or unwrap() calls in the YAML parser.
    if let Ok(text) = std::str::from_utf8(data) {
        if let Ok(manifest) = serde_yaml::from_str::<Manifest>(text) {
            // Also drive the parsed manifest's inline policy.rules through the
            // rule compiler, exactly as `svccat check`/`workspace check` do via
            // `src/drift.rs`. `RuleEngine::compile`'s inheritance resolver
            // (`resolve_rule` in `src/rules.rs`) recurses over each rule's
            // `base` chain with no cycle guard, so a self- or mutually-
            // referential `base` (e.g. rule "a" has base "b", rule "b" has
            // base "a") stack-overflows the process instead of returning an
            // `Err`. `svccat lint` gained a pre-compile cycle check
            // (`src/rule_schema.rs`, added for that exact bug), but `check`/
            // `workspace check` call `RuleEngine::compile` directly and are
            // NOT covered by that guard, so this is still a live crash
            // surface reachable from an untrusted manifest file. Fuzzing the
            // parse-then-compile pipeline together, the way `check` actually
            // uses it, is what makes this target able to find that class of
            // bug automatically instead of only fuzzing YAML shape.
            let _ = RuleEngine::compile(&manifest.policy.rules);
        }
    }
});
