// Property-based tests for pforge
//
// These tests use proptest to generate thousands of random test cases,
// verifying that system properties hold across a wide range of inputs.
//
// Run with: cargo test --test property --release -- --test-threads=1

use pforge_config::*;
use proptest::prelude::*;
use std::collections::HashMap;

// ============================================================================
// Arbitrary Generators
// ============================================================================

/// Generate arbitrary simple types
fn arb_simple_type() -> impl Strategy<Value = SimpleType> {
    prop_oneof![
        Just(SimpleType::String),
        Just(SimpleType::Integer),
        Just(SimpleType::Float),
        Just(SimpleType::Boolean),
        Just(SimpleType::Array),
        Just(SimpleType::Object),
    ]
}

/// Generate arbitrary transport types
fn arb_transport_type() -> impl Strategy<Value = TransportType> {
    prop_oneof![
        Just(TransportType::Stdio),
        Just(TransportType::Sse),
        Just(TransportType::WebSocket),
    ]
}

/// Generate arbitrary optimization levels
fn arb_optimization_level() -> impl Strategy<Value = OptimizationLevel> {
    prop_oneof![
        Just(OptimizationLevel::Debug),
        Just(OptimizationLevel::Release),
    ]
}

/// Generate arbitrary HTTP methods
fn arb_http_method() -> impl Strategy<Value = HttpMethod> {
    prop_oneof![
        Just(HttpMethod::Get),
        Just(HttpMethod::Post),
        Just(HttpMethod::Put),
        Just(HttpMethod::Delete),
        Just(HttpMethod::Patch),
    ]
}

/// Generate arbitrary forge metadata
fn arb_forge_metadata() -> impl Strategy<Value = ForgeMetadata> {
    (
        "[a-z][a-z0-9_-]{2,20}",
        "[0-9]\\.[0-9]\\.[0-9]",
        arb_transport_type(),
        arb_optimization_level(),
    )
        .prop_map(|(name, version, transport, optimization)| ForgeMetadata {
            name,
            version,
            transport,
            optimization,
        })
}

/// Generate arbitrary handler references
fn arb_handler_ref() -> impl Strategy<Value = HandlerRef> {
    "[a-z][a-z0-9_]{2,10}::[a-z][a-z0-9_]{2,10}".prop_map(|path| HandlerRef { path, inline: None })
}

/// Generate arbitrary parameter schemas
fn arb_param_schema() -> impl Strategy<Value = ParamSchema> {
    prop::collection::hash_map(
        "[a-z][a-z0-9_]{2,15}",
        arb_simple_type().prop_map(ParamType::Simple),
        0..5,
    )
    .prop_map(|fields| ParamSchema { fields })
}

/// Generate arbitrary native tool definitions
fn arb_native_tool() -> impl Strategy<Value = ToolDef> {
    (
        "[a-z][a-z0-9_-]{2,20}",
        "[A-Z][A-Za-z0-9 ]{5,50}",
        arb_handler_ref(),
        arb_param_schema(),
    )
        .prop_map(|(name, description, handler, params)| ToolDef::Native {
            name,
            description,
            handler,
            params,
            timeout_ms: None,
        })
}

/// Generate arbitrary CLI tool definitions
fn arb_cli_tool() -> impl Strategy<Value = ToolDef> {
    (
        "[a-z][a-z0-9_-]{2,20}",
        "[A-Z][A-Za-z0-9 ]{5,50}",
        "[a-z]{2,10}",
        prop::collection::vec("[a-z-]{2,10}", 0..5),
    )
        .prop_map(|(name, description, command, args)| ToolDef::Cli {
            name,
            description,
            command,
            args,
            cwd: None,
            env: HashMap::new(),
            stream: false,
        })
}

/// Generate arbitrary HTTP tool definitions
fn arb_http_tool() -> impl Strategy<Value = ToolDef> {
    (
        "[a-z][a-z0-9_-]{2,20}",
        "[A-Z][A-Za-z0-9 ]{5,50}",
        "https://[a-z]{3,10}\\.[a-z]{2,5}/[a-z]{2,10}",
        arb_http_method(),
    )
        .prop_map(|(name, description, endpoint, method)| ToolDef::Http {
            name,
            description,
            endpoint,
            method,
            headers: HashMap::new(),
            auth: None,
        })
}

/// Generate arbitrary tool definitions
fn arb_tool_def() -> impl Strategy<Value = ToolDef> {
    prop_oneof![arb_native_tool(), arb_cli_tool(), arb_http_tool(),]
}

/// Generate arbitrary forge configs with unique tool names
fn arb_forge_config() -> impl Strategy<Value = ForgeConfig> {
    (
        arb_forge_metadata(),
        prop::collection::vec(arb_tool_def(), 1..10),
    )
        .prop_map(|(forge, tools)| {
            // Ensure unique tool names
            let mut unique_tools = Vec::new();
            let mut seen_names = std::collections::HashSet::new();

            for tool in tools {
                let name = tool.name();
                if seen_names.insert(name.to_string()) {
                    unique_tools.push(tool);
                }
            }

            ForgeConfig {
                forge,
                tools: unique_tools,
                resources: vec![],
                prompts: vec![],
                state: None,
            }
        })
}

// ============================================================================
// Property Tests: Configuration
// ============================================================================

proptest! {
    /// Property: Config serialization roundtrip preserves structure
    #[test]
    fn config_serialization_roundtrip(config in arb_forge_config()) {
        // Serialize to YAML
        let yaml = serde_yml::to_string(&config).unwrap();

        // Deserialize back
        let parsed: ForgeConfig = serde_yml::from_str(&yaml).unwrap();

        // Key properties should be preserved
        prop_assert_eq!(&config.forge.name, &parsed.forge.name);
        prop_assert_eq!(&config.forge.version, &parsed.forge.version);
        prop_assert_eq!(config.tools.len(), parsed.tools.len());
    }

    /// Property: Tool names are always unique after validation
    #[test]
    fn tool_names_unique(config in arb_forge_config()) {
        // All tool names should be unique
        let mut names = std::collections::HashSet::new();
        for tool in &config.tools {
            prop_assert!(names.insert(tool.name()));
        }
    }

    /// Property: Valid configs always pass validation
    #[test]
    fn valid_configs_pass_validation(config in arb_forge_config()) {
        let result = validate_config(&config);
        prop_assert!(result.is_ok(), "Valid config failed validation: {:?}", result);
    }

    /// Property: Handler paths always contain ::
    #[test]
    fn native_handler_paths_valid(config in arb_forge_config()) {
        for tool in &config.tools {
            if let ToolDef::Native { handler, .. } = tool {
                prop_assert!(handler.path.contains("::"),
                    "Handler path '{}' doesn't contain ::", handler.path);
            }
        }
    }

    /// Property: Transport types are always valid
    #[test]
    fn transport_types_valid(config in arb_forge_config()) {
        // All transport types should serialize/deserialize correctly
        let yaml = serde_yml::to_string(&config.forge.transport).unwrap();
        let parsed: TransportType = serde_yml::from_str(&yaml).unwrap();
        prop_assert_eq!(config.forge.transport, parsed);
    }

    /// Property: Tool names follow naming conventions
    #[test]
    fn tool_names_follow_conventions(config in arb_forge_config()) {
        for tool in &config.tools {
            let name = tool.name();
            // Should be lowercase alphanumeric with hyphens/underscores
            prop_assert!(name.chars().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-' || c == '_'),
                "Tool name '{}' doesn't follow conventions", name);
            // Should have reasonable length
            prop_assert!(name.len() >= 3 && name.len() <= 50,
                "Tool name '{}' length {} not in range 3-50", name, name.len());
        }
    }

    /// Property: HTTP methods are always valid
    #[test]
    fn http_methods_valid(method in arb_http_method()) {
        // Should serialize and deserialize correctly
        let yaml = serde_yml::to_string(&method).unwrap();
        let parsed: HttpMethod = serde_yml::from_str(&yaml).unwrap();
        prop_assert_eq!(method, parsed);
    }

    /// Property: Optimization levels are consistent
    #[test]
    fn optimization_levels_consistent(level in arb_optimization_level()) {
        let yaml = serde_yml::to_string(&level).unwrap();
        let parsed: OptimizationLevel = serde_yml::from_str(&yaml).unwrap();
        prop_assert_eq!(level, parsed);
    }
}

// ============================================================================
// Property Tests: Validation
// ============================================================================

proptest! {
    /// Property: Duplicate tool names always cause validation error
    #[test]
    fn duplicate_tool_names_rejected(name in "[a-z][a-z0-9_-]{2,20}") {
        let config = ForgeConfig {
            forge: ForgeMetadata {
                name: "test".to_string(),
                version: "1.0.0".to_string(),
                transport: TransportType::Stdio,
                optimization: OptimizationLevel::Debug,
            },
            tools: vec![
                ToolDef::Native {
                    name: name.clone(),
                    description: "Tool 1".to_string(),
                    handler: HandlerRef { path: "mod1::handler".to_string(), inline: None },
                    params: ParamSchema { fields: HashMap::new() },
                    timeout_ms: None,
                },
                ToolDef::Native {
                    name: name.clone(),
                    description: "Tool 2".to_string(),
                    handler: HandlerRef { path: "mod2::handler".to_string(), inline: None },
                    params: ParamSchema { fields: HashMap::new() },
                    timeout_ms: None,
                },
            ],
            resources: vec![],
            prompts: vec![],
            state: None,
        };

        let result = validate_config(&config);
        prop_assert!(result.is_err(), "Duplicate tool names should fail validation");
        prop_assert!(matches!(result.unwrap_err(), ConfigError::DuplicateToolName(_)));
    }

    /// Property: Invalid handler paths are rejected
    #[test]
    fn invalid_handler_paths_rejected(path in "[a-z]{3,20}") {
        // Path without :: should fail
        let config = ForgeConfig {
            forge: ForgeMetadata {
                name: "test".to_string(),
                version: "1.0.0".to_string(),
                transport: TransportType::Stdio,
                optimization: OptimizationLevel::Debug,
            },
            tools: vec![
                ToolDef::Native {
                    name: "test_tool".to_string(),
                    description: "Test".to_string(),
                    handler: HandlerRef { path, inline: None },
                    params: ParamSchema { fields: HashMap::new() },
                    timeout_ms: None,
                },
            ],
            resources: vec![],
            prompts: vec![],
            state: None,
        };

        let result = validate_config(&config);
        prop_assert!(result.is_err(), "Invalid handler path should fail validation");
    }
}

// ============================================================================
// Property Tests: Edge Cases
// ============================================================================

proptest! {
    /// Property: Empty configs are valid (just metadata, no tools)
    #[test]
    fn empty_config_valid(forge in arb_forge_metadata()) {
        let config = ForgeConfig {
            forge,
            tools: vec![],
            resources: vec![],
            prompts: vec![],
            state: None,
        };

        let result = validate_config(&config);
        prop_assert!(result.is_ok(), "Empty config should be valid");
    }

    /// Property: Single tool configs are always valid
    #[test]
    fn single_tool_valid(forge in arb_forge_metadata(), tool in arb_tool_def()) {
        let config = ForgeConfig {
            forge,
            tools: vec![tool],
            resources: vec![],
            prompts: vec![],
            state: None,
        };

        let result = validate_config(&config);
        prop_assert!(result.is_ok(), "Single tool config should be valid");
    }
}
