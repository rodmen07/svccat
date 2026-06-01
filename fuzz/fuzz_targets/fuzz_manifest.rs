#![no_main]
use libfuzzer_sys::fuzz_target;
use svccat::manifest::Manifest;

fuzz_target!(|data: &[u8]| {
    // Try to parse arbitrary input as a manifest
    // This will find panic! or unwrap() calls in the parser
    if let Ok(text) = std::str::from_utf8(data) {
        let _ = serde_yaml::from_str::<Manifest>(text);
    }
});
