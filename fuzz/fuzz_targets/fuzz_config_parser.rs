#![no_main]

use libfuzzer_sys::fuzz_target;
use pforge_config::parse_config_from_str;

fuzz_target!(|data: &[u8]| {
    // Convert bytes to string, ignore invalid UTF-8
    if let Ok(yaml_str) = std::str::from_utf8(data) {
        // Try to parse the YAML config
        // We don't care if it fails, we just want to ensure it doesn't panic
        let _ = parse_config_from_str(yaml_str);
    }
});
