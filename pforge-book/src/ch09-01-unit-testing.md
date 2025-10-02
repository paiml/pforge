# Unit Testing

Unit tests are the foundation of pforge's testing pyramid. With **74 fast, focused tests** distributed across all crates, unit testing ensures individual components work correctly in isolation before integration. Each unit test completes in under 1 millisecond, enabling rapid feedback during development.

## Unit Test Philosophy

pforge's unit testing follows five core principles:

1. **Fast**: <1ms per test for instant feedback
2. **Focused**: Test one behavior per test function
3. **Isolated**: No dependencies on external state or other tests
4. **Deterministic**: Same input always produces same output
5. **Clear**: Test name clearly describes what's being tested

These principles enable the 5-minute TDD cycle that drives pforge development.

## Test Organization

Unit tests are co-located with source code using Rust's `#[cfg(test)]` module pattern:

```rust
// crates/pforge-runtime/src/registry.rs

pub struct HandlerRegistry {
    handlers: FxHashMap<String, Arc<dyn HandlerEntry>>,
}

impl HandlerRegistry {
    pub fn new() -> Self {
        Self {
            handlers: FxHashMap::default(),
        }
    }

    pub fn register<H>(&mut self, name: impl Into<String>, handler: H)
    where
        H: Handler,
        H::Input: 'static,
        H::Output: 'static,
    {
        let entry = HandlerEntryImpl::new(handler);
        self.handlers.insert(name.into(), Arc::new(entry));
    }

    pub fn has_handler(&self, name: &str) -> bool {
        self.handlers.contains_key(name)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_registry_new() {
        let registry = HandlerRegistry::new();
        assert!(registry.is_empty());
        assert_eq!(registry.len(), 0);
    }

    #[test]
    fn test_registry_register() {
        let mut registry = HandlerRegistry::new();
        registry.register("test_handler", TestHandler);

        assert!(!registry.is_empty());
        assert_eq!(registry.len(), 1);
        assert!(registry.has_handler("test_handler"));
        assert!(!registry.has_handler("nonexistent"));
    }
}
```

### Benefits of Inline Tests

- **Proximity**: Tests are next to the code they test
- **Visibility**: Easy to see what's tested and what's missing
- **Refactoring**: Tests update naturally when code changes
- **Compilation**: Tests only compile in test mode (no production overhead)

## Test Naming Conventions

pforge uses descriptive test names that form readable sentences:

```rust
#[test]
fn test_registry_returns_error_for_unknown_tool() {
    // Clear intent: what's being tested and expected outcome
}

#[test]
fn test_config_validation_rejects_duplicate_tool_names() {
    // Describes both the action and expected result
}

#[test]
fn test_handler_dispatch_preserves_async_context() {
    // Documents important behavior
}
```

### Naming Pattern

**Format**: `test_<component>_<behavior>_<condition>`

Examples:
- `test_registry_new_creates_empty_registry`
- `test_validator_rejects_invalid_handler_paths`
- `test_codegen_generates_correct_struct_for_native_tool`

## Common Unit Testing Patterns

### Testing State Transitions

```rust
#[test]
fn test_registry_tracks_handler_count_correctly() {
    let mut registry = HandlerRegistry::new();

    // Initial state
    assert_eq!(registry.len(), 0);
    assert!(registry.is_empty());

    // After first registration
    registry.register("handler1", TestHandler);
    assert_eq!(registry.len(), 1);
    assert!(!registry.is_empty());

    // After second registration
    registry.register("handler2", TestHandler);
    assert_eq!(registry.len(), 2);
}
```

### Testing Error Conditions

All error paths must be tested explicitly:

```rust
#[test]
fn test_validator_rejects_duplicate_tool_names() {
    let config = ForgeConfig {
        forge: create_test_metadata(),
        tools: vec![
            create_native_tool("duplicate"),
            create_native_tool("duplicate"),  // Intentional duplicate
        ],
        resources: vec![],
        prompts: vec![],
        state: None,
    };

    let result = validate_config(&config);

    assert!(result.is_err());
    assert!(matches!(
        result.unwrap_err(),
        ConfigError::DuplicateToolName(_)
    ));
}

#[test]
fn test_validator_rejects_invalid_handler_paths() {
    let config = create_config_with_handler_path("invalid_path");

    let result = validate_config(&config);

    assert!(result.is_err());
    match result.unwrap_err() {
        ConfigError::InvalidHandlerPath(msg) => {
            assert!(msg.contains("expected format: module::function"));
        }
        _ => panic!("Expected InvalidHandlerPath error"),
    }
}
```

### Testing Boundary Conditions

Test edge cases explicitly:

```rust
#[test]
fn test_registry_handles_empty_state() {
    let registry = HandlerRegistry::new();
    assert_eq!(registry.len(), 0);
    assert!(registry.is_empty());
}

#[test]
fn test_config_validation_accepts_zero_tools() {
    let config = ForgeConfig {
        forge: create_test_metadata(),
        tools: vec![],  // Empty tools list
        resources: vec![],
        prompts: vec![],
        state: None,
    };

    let result = validate_config(&config);
    assert!(result.is_ok());
}

#[test]
fn test_handler_path_validation_rejects_empty_string() {
    let result = validate_handler_path("");

    assert!(result.is_err());
    assert!(matches!(
        result.unwrap_err(),
        ConfigError::InvalidHandlerPath(_)
    ));
}
```

### Testing Async Functions

Use `#[tokio::test]` for async unit tests:

```rust
#[tokio::test]
async fn test_registry_dispatch_succeeds_for_registered_handler() {
    let mut registry = HandlerRegistry::new();
    registry.register("double", DoubleHandler);

    let input = TestInput { value: 21 };
    let input_bytes = serde_json::to_vec(&input).unwrap();

    let result = registry.dispatch("double", &input_bytes).await;

    assert!(result.is_ok());
    let output: TestOutput = serde_json::from_slice(&result.unwrap()).unwrap();
    assert_eq!(output.result, 42);
}

#[tokio::test]
async fn test_registry_dispatch_returns_tool_not_found_error() {
    let registry = HandlerRegistry::new();

    let result = registry.dispatch("nonexistent", b"{}").await;

    assert!(result.is_err());
    assert!(matches!(
        result.unwrap_err(),
        Error::ToolNotFound(_)
    ));
}
```

### Testing With Test Fixtures

Use helper functions to reduce boilerplate:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    // Test fixtures
    fn create_test_metadata() -> ForgeMetadata {
        ForgeMetadata {
            name: "test_server".to_string(),
            version: "1.0.0".to_string(),
            transport: TransportType::Stdio,
            optimization: OptimizationLevel::Debug,
        }
    }

    fn create_native_tool(name: &str) -> ToolDef {
        ToolDef::Native {
            name: name.to_string(),
            description: format!("Test tool: {}", name),
            handler: HandlerRef {
                path: format!("handlers::{}", name),
                inline: None,
            },
            params: ParamSchema {
                fields: HashMap::new(),
            },
            timeout_ms: None,
        }
    }

    fn create_valid_config() -> ForgeConfig {
        ForgeConfig {
            forge: create_test_metadata(),
            tools: vec![create_native_tool("test_tool")],
            resources: vec![],
            prompts: vec![],
            state: None,
        }
    }

    #[test]
    fn test_with_fixtures() {
        let config = create_valid_config();
        assert!(validate_config(&config).is_ok());
    }
}
```

## Real Unit Test Examples

### Example 1: Handler Registry Tests

From `crates/pforge-runtime/src/registry.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, Serialize, Deserialize, JsonSchema)]
    struct TestInput {
        value: i32,
    }

    #[derive(Debug, Serialize, Deserialize, JsonSchema)]
    struct TestOutput {
        result: i32,
    }

    struct TestHandler;

    #[async_trait]
    impl crate::Handler for TestHandler {
        type Input = TestInput;
        type Output = TestOutput;
        type Error = crate::Error;

        async fn handle(&self, input: Self::Input) -> Result<Self::Output> {
            Ok(TestOutput {
                result: input.value * 2,
            })
        }
    }

    #[tokio::test]
    async fn test_registry_new() {
        let registry = HandlerRegistry::new();
        assert!(registry.is_empty());
        assert_eq!(registry.len(), 0);
    }

    #[tokio::test]
    async fn test_registry_register() {
        let mut registry = HandlerRegistry::new();
        registry.register("test", TestHandler);

        assert!(!registry.is_empty());
        assert_eq!(registry.len(), 1);
        assert!(registry.has_handler("test"));
        assert!(!registry.has_handler("nonexistent"));
    }

    #[tokio::test]
    async fn test_registry_dispatch() {
        let mut registry = HandlerRegistry::new();
        registry.register("test", TestHandler);

        let input = TestInput { value: 21 };
        let input_bytes = serde_json::to_vec(&input).unwrap();

        let result = registry.dispatch("test", &input_bytes).await;
        assert!(result.is_ok());

        let output: TestOutput = serde_json::from_slice(&result.unwrap()).unwrap();
        assert_eq!(output.result, 42);
    }

    #[tokio::test]
    async fn test_registry_dispatch_missing_tool() {
        let registry = HandlerRegistry::new();

        let result = registry.dispatch("nonexistent", b"{}").await;

        assert!(result.is_err());
        match result.unwrap_err() {
            Error::ToolNotFound(name) => {
                assert_eq!(name, "nonexistent");
            }
            _ => panic!("Expected ToolNotFound error"),
        }
    }

    #[tokio::test]
    async fn test_registry_get_schemas() {
        let mut registry = HandlerRegistry::new();
        registry.register("test", TestHandler);

        let input_schema = registry.get_input_schema("test");
        assert!(input_schema.is_some());

        let output_schema = registry.get_output_schema("test");
        assert!(output_schema.is_some());

        let missing_schema = registry.get_input_schema("nonexistent");
        assert!(missing_schema.is_none());
    }
}
```

### Example 2: Config Validation Tests

From `crates/pforge-config/src/validator.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_config_success() {
        let config = ForgeConfig {
            forge: ForgeMetadata {
                name: "test".to_string(),
                version: "1.0.0".to_string(),
                transport: TransportType::Stdio,
                optimization: OptimizationLevel::Debug,
            },
            tools: vec![ToolDef::Native {
                name: "tool1".to_string(),
                description: "Tool 1".to_string(),
                handler: HandlerRef {
                    path: "module::handler".to_string(),
                    inline: None,
                },
                params: ParamSchema {
                    fields: HashMap::new(),
                },
                timeout_ms: None,
            }],
            resources: vec![],
            prompts: vec![],
            state: None,
        };

        assert!(validate_config(&config).is_ok());
    }

    #[test]
    fn test_validate_config_duplicate_tools() {
        let config = ForgeConfig {
            forge: create_test_metadata(),
            tools: vec![
                create_tool("duplicate"),
                create_tool("duplicate"),
            ],
            resources: vec![],
            prompts: vec![],
            state: None,
        };

        let result = validate_config(&config);

        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            ConfigError::DuplicateToolName(_)
        ));
    }

    #[test]
    fn test_validate_handler_path_empty() {
        let result = validate_handler_path("");
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_handler_path_no_separator() {
        let result = validate_handler_path("invalid_path");

        assert!(result.is_err());
        match result.unwrap_err() {
            ConfigError::InvalidHandlerPath(msg) => {
                assert!(msg.contains("expected format: module::function"));
            }
            _ => panic!("Wrong error type"),
        }
    }

    #[test]
    fn test_validate_handler_path_valid() {
        assert!(validate_handler_path("module::function").is_ok());
        assert!(validate_handler_path("crate::module::function").is_ok());
    }
}
```

### Example 3: Code Generation Tests

From `crates/pforge-codegen/src/lib.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_config() -> ForgeConfig {
        ForgeConfig {
            forge: ForgeMetadata {
                name: "test_server".to_string(),
                version: "1.0.0".to_string(),
                transport: TransportType::Stdio,
                optimization: OptimizationLevel::Debug,
            },
            tools: vec![ToolDef::Native {
                name: "test_tool".to_string(),
                description: "Test tool".to_string(),
                handler: HandlerRef {
                    path: "handlers::test_handler".to_string(),
                    inline: None,
                },
                params: ParamSchema {
                    fields: {
                        let mut map = HashMap::new();
                        map.insert("input".to_string(), ParamType::Simple(SimpleType::String));
                        map
                    },
                },
                timeout_ms: None,
            }],
            resources: vec![],
            prompts: vec![],
            state: None,
        }
    }

    #[test]
    fn test_generate_all() {
        let config = create_test_config();
        let result = generate_all(&config);

        assert!(result.is_ok());
        let code = result.unwrap();

        // Verify generated header
        assert!(code.contains("// Auto-generated by pforge"));
        assert!(code.contains("// DO NOT EDIT"));

        // Verify imports
        assert!(code.contains("use pforge_runtime::*"));
        assert!(code.contains("use serde::{Deserialize, Serialize}"));
        assert!(code.contains("use schemars::JsonSchema"));

        // Verify param struct generation
        assert!(code.contains("pub struct TestToolParams"));

        // Verify registration function
        assert!(code.contains("pub fn register_handlers"));
    }

    #[test]
    fn test_generate_all_empty_tools() {
        let config = ForgeConfig {
            forge: create_test_metadata(),
            tools: vec![],
            resources: vec![],
            prompts: vec![],
            state: None,
        };

        let result = generate_all(&config);
        assert!(result.is_ok());

        let code = result.unwrap();
        assert!(code.contains("pub fn register_handlers"));
    }

    #[test]
    fn test_write_generated_code() {
        let config = create_test_config();
        let temp_dir = std::env::temp_dir();
        let output_path = temp_dir.join("test_generated.rs");

        let result = write_generated_code(&config, &output_path);
        assert!(result.is_ok());

        // Verify file exists
        assert!(output_path.exists());

        // Verify content
        let content = std::fs::read_to_string(&output_path).unwrap();
        assert!(content.contains("pub struct TestToolParams"));

        // Cleanup
        std::fs::remove_file(&output_path).ok();
    }

    #[test]
    fn test_write_generated_code_invalid_path() {
        let config = create_test_config();
        let invalid_path = Path::new("/nonexistent/directory/test.rs");

        let result = write_generated_code(&config, invalid_path);

        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), CodegenError::IoError(_, _)));
    }
}
```

## Performance Considerations

### Keep Tests Fast

```rust
// Good: Fast, focused test (<1ms)
#[test]
fn test_config_has_unique_tool_names() {
    let mut names = HashSet::new();
    for tool in config.tools {
        assert!(names.insert(tool.name()));
    }
}

// Bad: Slow test (>10ms) - move to integration test
#[test]
fn test_full_server_startup() {
    // This belongs in integration tests, not unit tests
    let server = Server::new(config);
    server.start().await;
    // ... many operations ...
}
```

### Avoid I/O in Unit Tests

```rust
// Good: No I/O, fast
#[test]
fn test_serialization() {
    let config = create_test_config();
    let yaml = serde_yml::to_string(&config).unwrap();
    assert!(yaml.contains("test_server"));
}

// Bad: File I/O slows down tests
#[test]
fn test_config_from_file() {
    let config = load_config_from_file("test.yaml");  // Slow!
    assert!(config.is_ok());
}
```

## Test Coverage

pforge enforces ≥80% line coverage. View coverage with:

```bash
# Generate coverage report
make coverage

# View HTML report
make coverage-open
```

### Ensuring Coverage

```rust
// Cover all match arms
#[test]
fn test_error_display() {
    let errors = vec![
        Error::ToolNotFound("test".to_string()),
        Error::InvalidConfig("test".to_string()),
        Error::Validation("test".to_string()),
        Error::Handler("test".to_string()),
        Error::Timeout("test".to_string()),
    ];

    for error in errors {
        let msg = error.to_string();
        assert!(!msg.is_empty());
    }
}

// Cover all enum variants
#[test]
fn test_transport_serialization() {
    let transports = vec![
        TransportType::Stdio,
        TransportType::Sse,
        TransportType::WebSocket,
    ];

    for transport in transports {
        let yaml = serde_yml::to_string(&transport).unwrap();
        let parsed: TransportType = serde_yml::from_str(&yaml).unwrap();
        assert_eq!(transport, parsed);
    }
}
```

## Running Unit Tests

### Quick Commands

```bash
# Run all unit tests
cargo test --lib

# Run specific crate's unit tests
cargo test --lib -p pforge-runtime

# Run specific test
cargo test test_registry_new

# Run with output
cargo test --lib -- --nocapture

# Run with threads for debugging
cargo test --lib -- --test-threads=1
```

### Watch Mode

For TDD, use watch mode:

```bash
# Auto-run tests on file changes
make watch

# Or with cargo-watch
cargo watch -x 'test --lib --quiet' -x 'clippy --quiet'
```

## Best Practices Summary

1. **Keep tests fast**: Target <1ms per test
2. **Test one thing**: Single behavior per test
3. **Use descriptive names**: `test_component_behavior_condition`
4. **Test error paths**: Every error variant needs a test
5. **Avoid I/O**: No file/network operations in unit tests
6. **Use fixtures**: Helper functions reduce boilerplate
7. **Test boundaries**: Empty, zero, max values
8. **Isolate tests**: No shared state between tests
9. **Make tests readable**: Clear setup, action, assertion
10. **Maintain coverage**: Keep ≥80% line coverage

## Common Pitfalls

### Avoid Test Dependencies

```rust
// Bad: Tests depend on each other
static mut COUNTER: i32 = 0;

#[test]
fn test_one() {
    unsafe { COUNTER += 1; }
    assert_eq!(unsafe { COUNTER }, 1);  // Fails if run out of order!
}

// Good: Each test is independent
#[test]
fn test_one() {
    let counter = 0;
    let result = counter + 1;
    assert_eq!(result, 1);
}
```

### Avoid Unwrap in Tests

```rust
// Bad: Unwrap hides error details
#[test]
fn test_parsing() {
    let config = parse_config(yaml).unwrap();  // What error occurred?
    assert_eq!(config.name, "test");
}

// Good: Explicit error handling
#[test]
fn test_parsing() {
    let config = parse_config(yaml)
        .expect("Failed to parse valid config");
    assert_eq!(config.name, "test");
}

// Even better: Test the Result
#[test]
fn test_parsing() {
    let result = parse_config(yaml);
    assert!(result.is_ok(), "Parse failed: {:?}", result.unwrap_err());
    assert_eq!(result.unwrap().name, "test");
}
```

### Test Negative Cases

```rust
// Incomplete: Only tests happy path
#[test]
fn test_validate_config() {
    let config = create_valid_config();
    assert!(validate_config(&config).is_ok());
}

// Complete: Tests both success and failure
#[test]
fn test_validate_config_success() {
    let config = create_valid_config();
    assert!(validate_config(&config).is_ok());
}

#[test]
fn test_validate_config_rejects_duplicates() {
    let config = create_config_with_duplicates();
    assert!(validate_config(&config).is_err());
}

#[test]
fn test_validate_config_rejects_invalid_paths() {
    let config = create_config_with_invalid_path();
    assert!(validate_config(&config).is_err());
}
```

## Summary

Unit tests form the foundation of pforge's quality assurance:

- **74 fast tests** distributed across all crates
- **<1ms per test** enabling rapid TDD cycles
- **Co-located** with source code for easy maintenance
- **Comprehensive coverage** of all error paths
- **Part of quality gates** blocking commits on failure

Well-written unit tests provide instant feedback, document expected behavior, and catch regressions before they reach production. Combined with integration tests (Chapter 9.2), property-based tests (Chapter 9.3), and mutation testing (Chapter 9.4), they ensure pforge maintains the highest quality standards.
