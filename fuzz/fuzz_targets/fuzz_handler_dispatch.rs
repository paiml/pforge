#![no_main]

//! Fuzz the registry lookup and dispatch paths with arbitrary tool names.
//!
//! This target had been failing to BUILD since `HandlerRegistry::get` was
//! removed in favour of `has_handler`/`dispatch`, and nobody noticed: all three
//! fuzz steps in `.github/workflows/fuzz.yml` carried `continue-on-error: true`,
//! so the lane reported success every day while running two of its three
//! targets. Absence of crash artifacts was being read as absence of crashes,
//! when the fuzzer had never started (paiml/pforge#11).
//!
//! It now exercises `dispatch` as well as lookup. Dispatch is the interesting
//! surface — it takes attacker-shaped bytes as parameters, where lookup only
//! takes a name — and it is the path an MCP `tools/call` reaches.

use libfuzzer_sys::fuzz_target;
use pforge_runtime::HandlerRegistry;

fuzz_target!(|data: &[u8]| {
    let registry = HandlerRegistry::new();

    // Split the input so one half names a tool and the other is the payload,
    // rather than fuzzing only the name. An empty registry still exercises the
    // not-found path, which is the one an MCP client reaches on a bad call.
    let (name_bytes, params) = data.split_at(data.len() / 2);

    if let Ok(tool_name) = std::str::from_utf8(name_bytes) {
        // Lookup must never panic on arbitrary UTF-8.
        let _ = registry.has_handler(tool_name);

        // Dispatch is async; drive it to completion on a current-thread runtime.
        // A panic here is a real finding: this is what `tools/call` reaches.
        let rt = tokio::runtime::Builder::new_current_thread()
            .build()
            .expect("runtime");
        let _ = rt.block_on(registry.dispatch(tool_name, params));
    }

    // Boundary names that have historically broken lookups.
    let _ = registry.has_handler("");
    let _ = registry.has_handler(&"a".repeat(10_000));
    // Interior NUL and non-ASCII must be handled as ordinary strings.
    let _ = registry.has_handler("tool\0name");
    let _ = registry.has_handler("工具");
});
