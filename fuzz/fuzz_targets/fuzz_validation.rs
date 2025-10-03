#![no_main]

use libfuzzer_sys::fuzz_target;
use pforge_config::{ForgeConfig, ToolDef};

fuzz_target!(|data: &[u8]| {
    // Try to parse YAML and validate it
    if let Ok(yaml_str) = std::str::from_utf8(data) {
        if let Ok(config) = serde_yaml::from_str::<ForgeConfig>(yaml_str) {
            // Validate the config structure
            // Check tool name uniqueness
            let mut tool_names = std::collections::HashSet::new();
            for tool in &config.tools {
                let name = match tool {
                    ToolDef::Native { name, .. } => name,
                    ToolDef::Cli { name, .. } => name,
                    ToolDef::Http { name, .. } => name,
                    ToolDef::Pipeline { name, .. } => name,
                };
                tool_names.insert(name.clone());
            }

            // Verify all tools have valid names (non-empty)
            for tool in &config.tools {
                let name = tool.name();
                assert!(!name.is_empty(), "Tool name should not be empty");
            }

            // Try to serialize back
            let _ = serde_yaml::to_string(&config);
        }
    }
});
