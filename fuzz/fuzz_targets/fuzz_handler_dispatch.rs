#![no_main]

use libfuzzer_sys::fuzz_target;
use pforge_runtime::HandlerRegistry;

fuzz_target!(|data: &[u8]| {
    // Create a registry
    let registry = HandlerRegistry::new();

    // Try to convert data to a string (tool name)
    if let Ok(tool_name) = std::str::from_utf8(data) {
        // Test that looking up non-existent tools doesn't panic
        let _ = registry.get(tool_name);
    }

    // Test registry with empty name
    let _ = registry.get("");

    // Test registry with very long names
    let long_name = "a".repeat(10000);
    let _ = registry.get(&long_name);
});
