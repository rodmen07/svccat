#![no_main]
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    if let Ok(pattern) = std::str::from_utf8(data) {
        // Test glob pattern compilation with arbitrary patterns
        // This will find panics or resource exhaustion in glob parsing
        let _ = glob::Pattern::new(pattern);
    }
});
