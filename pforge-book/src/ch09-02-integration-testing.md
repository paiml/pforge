# Integration Testing

Integration tests verify that pforge components work correctly together. With **26 comprehensive integration tests** covering cross-crate workflows, middleware chains, and end-to-end scenarios, integration testing ensures the system functions as a cohesive whole.

## Integration Test Philosophy

Integration tests differ from unit tests in scope and purpose:

| Aspect | Unit Tests | Integration Tests |
|--------|------------|-------------------|
| **Scope** | Single component | Multiple components |
| **Speed** | <1ms | <100ms target |
| **Dependencies** | None | Real implementations |
| **Location** | Inline `#[cfg(test)]` | `tests/` directory |
| **Purpose** | Verify isolation | Verify collaboration |

Integration tests answer the question: "Do these components work together correctly?"

## Test Organization

Integration tests live in dedicated test crates:

```
pforge/
├── crates/pforge-integration-tests/
│   ├── Cargo.toml
│   ├── integration_test.rs    # 18 integration tests
│   └── property_test.rs        # 12 property-based tests
└── crates/pforge-cli/tests/
    └── scaffold_tests.rs       # 8 CLI integration tests
```

### Integration Test Crate Structure

```toml
# crates/pforge-integration-tests/Cargo.toml
[package]
name = "pforge-integration-tests"
version = "0.1.0"
edition = "2021"
publish = false

[dependencies]
pforge-config = { path = "../pforge-config" }
pforge-runtime = { path = "../pforge-runtime" }
pforge-codegen = { path = "../pforge-codegen" }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
serde_yaml = "0.9"
tokio = { version = "1.0", features = ["full"] }
proptest = "1.0"  # For property-based tests
```

## Real Integration Test Examples

### Example 1: Config Parsing All Tool Types

Tests that all tool types parse correctly from YAML:

```rust
#[test]
fn test_config_parsing_all_tool_types() {
    let yaml = r#"
forge:
  name: test-server
  version: 0.1.0
  transport: stdio

tools:
  - type: native
    name: hello
    description: Say hello
    handler:
      path: handlers::hello
    params:
      name:
        type: string
        required: true

  - type: cli
    name: echo
    description: Echo command
    command: echo
    args: ["hello"]

  - type: http
    name: api_call
    description: API call
    endpoint: https://api.example.com
    method: GET
"#;

    let config: ForgeConfig = serde_yaml::from_str(yaml).unwrap();
    assert_eq!(config.forge.name, "test-server");
    assert_eq!(config.tools.len(), 3);

    // Verify each tool type parsed correctly
    assert!(matches!(config.tools[0], ToolDef::Native { .. }));
    assert!(matches!(config.tools[1], ToolDef::Cli { .. }));
    assert!(matches!(config.tools[2], ToolDef::Http { .. }));
}
```

**What this tests**:
- Cross-crate interaction: `pforge-config` types with `serde_yaml`
- All tool variants deserialize correctly
- Configuration structure is valid

### Example 2: Middleware Chain with Recovery

Tests that multiple middleware components work together:

```rust
#[tokio::test]
async fn test_middleware_chain_with_recovery() {
    let mut chain = MiddlewareChain::new();

    let recovery = RecoveryMiddleware::new().with_circuit_breaker(CircuitBreakerConfig {
        failure_threshold: 3,
        timeout: Duration::from_secs(60),
        success_threshold: 2,
    });

    let tracker = recovery.error_tracker();
    chain.add(Arc::new(recovery));

    // Successful execution
    let result = chain
        .execute(json!({"input": 42}), |req| async move {
            Ok(json!({"output": req["input"].as_i64().unwrap() * 2}))
        })
        .await
        .unwrap();

    assert_eq!(result["output"], 84);
    assert_eq!(tracker.total_errors(), 0);
}
```

**What this tests**:
- Middleware chain execution flow
- Recovery middleware integration
- Circuit breaker configuration
- Error tracking across components

### Example 3: Full Middleware Stack

Tests a realistic middleware stack with multiple layers:

```rust
#[tokio::test]
async fn test_full_middleware_stack() {
    use pforge_runtime::{LoggingMiddleware, ValidationMiddleware};

    let mut chain = MiddlewareChain::new();

    // Add validation
    chain.add(Arc::new(ValidationMiddleware::new(vec![
        "input".to_string(),
    ])));

    // Add logging
    chain.add(Arc::new(LoggingMiddleware::new("test")));

    // Add recovery
    chain.add(Arc::new(RecoveryMiddleware::new()));

    // Execute with valid request
    let result = chain
        .execute(json!({"input": 42}), |req| async move {
            Ok(json!({"output": req["input"].as_i64().unwrap() + 1}))
        })
        .await;

    assert!(result.is_ok());
    assert_eq!(result.unwrap()["output"], 43);

    // Execute with invalid request (missing field)
    let result = chain
        .execute(json!({"wrong": 42}), |req| async move {
            Ok(json!({"output": req["input"].as_i64().unwrap() + 1}))
        })
        .await;

    assert!(result.is_err());
}
```

**What this tests**:
- Multiple middleware components compose correctly
- Validation runs before handler execution
- Error propagation through middleware stack
- Both success and failure paths

### Example 4: State Management Persistence

Tests state management across operations:

```rust
#[tokio::test]
async fn test_state_management_persistence() {
    let state = MemoryStateManager::new();

    // Set and get
    state.set("key1", b"value1".to_vec(), None).await.unwrap();
    let value = state.get("key1").await.unwrap();
    assert_eq!(value, Some(b"value1".to_vec()));

    // Exists
    assert!(state.exists("key1").await.unwrap());
    assert!(!state.exists("key2").await.unwrap());

    // Delete
    state.delete("key1").await.unwrap();
    assert!(!state.exists("key1").await.unwrap());
}
```

**What this tests**:
- State operations work correctly in sequence
- Data persists across calls
- All CRUD operations integrate properly

### Example 5: Retry with Timeout Integration

Tests retry logic with timeouts:

```rust
#[tokio::test]
async fn test_retry_with_timeout() {
    let policy = RetryPolicy::new(3)
        .with_backoff(Duration::from_millis(10), Duration::from_millis(50))
        .with_jitter(false);

    let attempt_counter = Arc::new(AtomicUsize::new(0));
    let counter_clone = attempt_counter.clone();

    let result = retry_with_policy(&policy, || {
        let counter = counter_clone.clone();
        async move {
            let count = counter.fetch_add(1, Ordering::SeqCst);
            if count < 2 {
                with_timeout(Duration::from_millis(10), async {
                    tokio::time::sleep(Duration::from_secs(10)).await;
                    42
                })
                .await
            } else {
                Ok(100)
            }
        }
    })
    .await;

    assert!(result.is_ok());
    assert_eq!(result.unwrap(), 100);
    assert_eq!(attempt_counter.load(Ordering::SeqCst), 3);
}
```

**What this tests**:
- Retry policy execution
- Timeout integration
- Backoff behavior
- Success after multiple attempts

### Example 6: Circuit Breaker Integration

Tests circuit breaker state transitions:

```rust
#[tokio::test]
async fn test_circuit_breaker_integration() {
    let config = CircuitBreakerConfig {
        failure_threshold: 2,
        timeout: Duration::from_millis(100),
        success_threshold: 2,
    };

    let cb = CircuitBreaker::new(config);

    // Cause failures to open circuit
    for _ in 0..2 {
        let _ = cb
            .call(|| async { Err::<(), _>(Error::Handler("failure".to_string())) })
            .await;
    }

    // Circuit should be open
    let result = cb
        .call(|| async { Ok::<_, Error>(42) })
        .await;
    assert!(result.is_err());

    // Wait for timeout
    tokio::time::sleep(Duration::from_millis(150)).await;

    // Should transition to half-open and eventually close
    let _ = cb.call(|| async { Ok::<_, Error>(1) }).await;
    let _ = cb.call(|| async { Ok::<_, Error>(2) }).await;

    // Now should work
    let result = cb.call(|| async { Ok::<_, Error>(42) }).await;
    assert!(result.is_ok());
}
```

**What this tests**:
- Circuit breaker opens after threshold failures
- Half-open state after timeout
- Circuit closes after success threshold
- Complete state machine transitions

### Example 7: Prompt Manager Full Workflow

Tests template rendering with variable substitution:

```rust
#[tokio::test]
async fn test_prompt_manager_full_workflow() {
    let mut manager = PromptManager::new();

    // Register prompts
    let prompt = PromptDef {
        name: "greeting".to_string(),
        description: "Greet user".to_string(),
        template: "Hello {{name}}, you are {{age}} years old!".to_string(),
        arguments: HashMap::new(),
    };

    manager.register(prompt).unwrap();

    // Render prompt
    let mut args = HashMap::new();
    args.insert("name".to_string(), json!("Alice"));
    args.insert("age".to_string(), json!(30));

    let rendered = manager.render("greeting", args).unwrap();
    assert_eq!(rendered, "Hello Alice, you are 30 years old!");
}
```

**What this tests**:
- Prompt registration
- Template variable substitution
- JSON value integration with templates
- End-to-end prompt workflow

### Example 8: Config Validation Duplicate Tools

Tests validation across components:

```rust
#[test]
fn test_config_validation_duplicate_tools() {
    use pforge_config::validate_config;

    let yaml = r#"
forge:
  name: test
  version: 1.0.0

tools:
  - type: cli
    name: duplicate
    description: First
    command: echo
    args: []

  - type: cli
    name: duplicate
    description: Second
    command: echo
    args: []
"#;

    let config: ForgeConfig = serde_yaml::from_str(yaml).unwrap();
    let result = validate_config(&config);

    assert!(result.is_err());
    assert!(result
        .unwrap_err()
        .to_string()
        .contains("Duplicate tool name"));
}
```

**What this tests**:
- YAML parsing → config validation pipeline
- Error detection at validation layer
- Error message formatting

## Quality Gate Integration Tests

pforge includes 8 dedicated tests for PMAT quality gate integration:

### Example 9: PMAT Quality Gate Exists

```rust
#[test]
fn test_pmat_quality_gate_exists() {
    let output = Command::new("pmat")
        .arg("quality-gate")
        .arg("--help")
        .output()
        .expect("pmat should be installed");

    assert!(
        output.status.success(),
        "pmat quality-gate should be available"
    );
}
```

### Example 10: Complexity Enforcement

```rust
#[test]
fn test_complexity_enforcement() {
    let output = Command::new("pmat")
        .arg("analyze")
        .arg("complexity")
        .arg("--max-cyclomatic")
        .arg("20")
        .arg("--format")
        .arg("summary")
        .current_dir("../../")
        .output()
        .expect("pmat analyze complexity should work");

    assert!(
        output.status.success(),
        "Complexity should be under 20: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}
```

### Example 11: Coverage Tracking

```rust
#[test]
fn test_coverage_tracking() {
    let has_llvm_cov = Command::new("cargo")
        .arg("llvm-cov")
        .arg("--version")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false);

    let has_tarpaulin = Command::new("cargo")
        .arg("tarpaulin")
        .arg("--version")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false);

    assert!(
        has_llvm_cov || has_tarpaulin,
        "At least one coverage tool should be installed"
    );
}
```

## CLI Integration Tests

From `crates/pforge-cli/tests/scaffold_tests.rs`:

### Example 12: Workspace Compiles

```rust
#[test]
fn test_workspace_compiles() {
    let output = Command::new("cargo")
        .arg("build")
        .arg("--release")
        .output()
        .expect("Failed to run cargo build");

    assert!(output.status.success(), "Workspace should compile");
}
```

### Example 13: All Crates Exist

```rust
#[test]
fn test_all_crates_exist() {
    let root = workspace_root();
    let crates = vec![
        "crates/pforge-cli",
        "crates/pforge-runtime",
        "crates/pforge-codegen",
        "crates/pforge-config",
        "crates/pforge-macro",
    ];

    for crate_path in crates {
        let path = root.join(crate_path);
        assert!(path.exists(), "Crate {} should exist", crate_path);

        let cargo_toml = path.join("Cargo.toml");
        assert!(
            cargo_toml.exists(),
            "Cargo.toml should exist in {}",
            crate_path
        );
    }
}
```

## Integration Test Patterns

### Testing Async Workflows

```rust
#[tokio::test]
async fn test_async_workflow() {
    // Setup
    let registry = HandlerRegistry::new();
    let state = MemoryStateManager::new();

    // Execute workflow
    state.set("config", b"data".to_vec(), None).await.unwrap();
    let config = state.get("config").await.unwrap();

    // Verify
    assert!(config.is_some());
}
```

### Testing Error Propagation

```rust
#[tokio::test]
async fn test_error_propagation_through_middleware() {
    let mut chain = MiddlewareChain::new();
    chain.add(Arc::new(ValidationMiddleware::new(vec!["required".to_string()])));

    let result = chain
        .execute(json!({"wrong_field": 1}), |_| async { Ok(json!({})) })
        .await;

    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("Missing required field"));
}
```

### Testing State Transitions

```rust
#[tokio::test]
async fn test_circuit_breaker_state_transitions() {
    let cb = CircuitBreaker::new(config);

    // Initial: Closed
    assert_eq!(cb.state(), CircuitBreakerState::Closed);

    // After failures: Open
    for _ in 0..3 {
        let _ = cb.call(|| async { Err::<(), _>(Error::Handler("fail".into())) }).await;
    }
    assert_eq!(cb.state(), CircuitBreakerState::Open);

    // After timeout: HalfOpen
    tokio::time::sleep(timeout_duration).await;
    assert_eq!(cb.state(), CircuitBreakerState::HalfOpen);
}
```

## Running Integration Tests

### Quick Commands

```bash
# Run all integration tests
cargo test --test integration_test

# Run specific integration test
cargo test --test integration_test test_middleware_chain

# Run all tests in integration test crate
cargo test -p pforge-integration-tests

# Run with output
cargo test --test integration_test -- --nocapture
```

### Performance Monitoring

```bash
# Run with timing
cargo test --test integration_test -- --nocapture --test-threads=1

# Profile integration tests
cargo flamegraph --test integration_test
```

## Best Practices

### 1. Test Realistic Scenarios

```rust
// Good: Tests real workflow
#[tokio::test]
async fn test_complete_request_lifecycle() {
    let config = load_config();
    let registry = build_registry(&config);
    let middleware = setup_middleware();

    let result = process_request(&registry, &middleware, request).await;
    assert!(result.is_ok());
}
```

### 2. Use Real Dependencies

```rust
// Good: Uses real MemoryStateManager
#[tokio::test]
async fn test_state_integration() {
    let state = MemoryStateManager::new();
    // ... test with real implementation
}

// Avoid: Mock when testing integration
// let state = MockStateManager::new(); // Save mocks for unit tests
```

### 3. Test Error Recovery

```rust
#[tokio::test]
async fn test_recovery_from_transient_failures() {
    let policy = RetryPolicy::new(3);

    let mut attempts = 0;
    let result = retry_with_policy(&policy, || async {
        attempts += 1;
        if attempts < 2 {
            Err(Error::Handler("transient".into()))
        } else {
            Ok(42)
        }
    }).await;

    assert_eq!(result.unwrap(), 42);
    assert_eq!(attempts, 2);
}
```

### 4. Keep Tests Independent

```rust
#[tokio::test]
async fn test_a() {
    let state = MemoryStateManager::new();  // Fresh state
    // ... test logic
}

#[tokio::test]
async fn test_b() {
    let state = MemoryStateManager::new();  // Fresh state
    // ... test logic
}
```

### 5. Target <100ms Per Test

```rust
// Good: Fast integration test
#[tokio::test]
async fn test_handler_dispatch() {
    let registry = create_registry();
    let result = registry.dispatch("tool", params).await;
    assert!(result.is_ok());
}  // ~10-20ms

// If slower, consider:
// - Reducing setup complexity
// - Removing unnecessary waits
// - Moving to E2E tests if >100ms
```

## Common Pitfalls

### Avoid Shared State

```rust
// Bad: Global state causes test interference
static REGISTRY: Lazy<HandlerRegistry> = Lazy::new(|| {
    HandlerRegistry::new()
});

#[test]
fn test_a() {
    REGISTRY.register("test", handler);  // Affects other tests!
}

// Good: Each test creates its own instance
#[test]
fn test_a() {
    let mut registry = HandlerRegistry::new();
    registry.register("test", handler);
}
```

### Test Both Success and Failure

```rust
#[tokio::test]
async fn test_middleware_success_path() {
    let result = middleware.execute(valid_request, handler).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_middleware_failure_path() {
    let result = middleware.execute(invalid_request, handler).await;
    assert!(result.is_err());
}
```

### Clean Up Resources

```rust
#[test]
fn test_file_operations() {
    let temp_file = create_temp_file();

    // Test logic...

    // Cleanup
    std::fs::remove_file(&temp_file).ok();
}
```

## Debugging Integration Tests

### Enable Logging

```rust
#[tokio::test]
async fn test_with_logging() {
    let _ = env_logger::builder()
        .is_test(true)
        .try_init();

    // Test will now show RUST_LOG output
}
```

### Use Descriptive Assertions

```rust
// Bad: Unclear failure
assert!(result.is_ok());

// Good: Clear failure message
assert!(
    result.is_ok(),
    "Middleware chain failed: {:?}",
    result.unwrap_err()
);
```

### Test in Isolation

```bash
# Run single test to debug
cargo test --test integration_test test_specific_test -- --nocapture --test-threads=1
```

## Summary

Integration tests ensure pforge components work together correctly:

- **26 integration tests** covering cross-crate workflows
- **<100ms target** for fast feedback
- **Real dependencies** not mocks or stubs
- **Quality gates** verified through integration tests
- **Complete workflows** from config to execution

Integration tests sit between unit tests (Chapter 9.1) and property-based tests (Chapter 9.3), providing confidence that pforge's architecture enables robust, reliable MCP server development.

Key takeaways:
1. Test realistic scenarios with real dependencies
2. Keep tests fast (<100ms) and independent
3. Test both success and failure paths
4. Use integration tests to verify cross-crate workflows
5. Quality gates integration ensures PMAT enforcement works

Together with unit tests, property-based tests, and mutation testing, integration tests form a comprehensive quality assurance strategy that ensures pforge remains production-ready.
