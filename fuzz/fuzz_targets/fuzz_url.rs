#![no_main]
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    if let Ok(url_str) = std::str::from_utf8(data) {
        // Test URL validation with arbitrary string input
        // This will find panics or crashes in the URL validation logic
        let _ = svccat::urlvalidation::validate_url(url_str, false);
        let _ = svccat::urlvalidation::validate_url(url_str, true);
    }
});
