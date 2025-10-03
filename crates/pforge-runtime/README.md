# pforge-runtime

[![crates.io](https://img.shields.io/crates/v/pforge-runtime.svg)](https://crates.io/crates/pforge-runtime)
[![Documentation](https://docs.rs/pforge-runtime/badge.svg)](https://docs.rs/pforge-runtime)

Core runtime for building high-performance Model Context Protocol (MCP) servers with pforge.

## Features

- **Type-Safe Handlers**: Define MCP tools with full Rust type safety
- **Multiple Handler Types**: Native Rust, CLI wrappers, HTTP proxies, and pipelines
- **State Management**: Built-in persistent state with memory and Sled backends
- **Middleware System**: Composable middleware for logging, validation, recovery, and metrics
- **Resource Management**: URI-based resource handling with template matching
- **Prompt Templates**: Mustache-style templating for dynamic prompts
- **Circuit Breaker**: Fault tolerance with configurable circuit breaking
- **Retry Policies**: Exponential backoff with jitter
- **Error Tracking**: Automatic error classification and monitoring

## Performance

Built on [pmcp](https://github.com/paiml/rust-mcp-sdk) (Pragmatic AI Labs MCP SDK):

- **< 1μs** hot handler dispatch (O(1) lookup with FxHash)
- **> 100K req/s** sequential throughput
- **> 500K req/s** concurrent throughput (8-core)
- **< 512KB** memory baseline
- **< 256B** memory per tool

## Installation

```bash
cargo add pforge-runtime
```

## Quick Example

```rust
use pforge_runtime::prelude::*;
use serde_json::{json, Value};

#[derive(Default)]
struct CalculatorHandler;

#[async_trait::async_trait]
impl Handler for CalculatorHandler {
    async fn handle(&self, params: Value) -> Result<Value> {
        let a = params["a"].as_i64().ok_or(Error::Validation("Missing 'a'".into()))?;
        let b = params["b"].as_i64().ok_or(Error::Validation("Missing 'b'".into()))?;
        let op = params["op"].as_str().ok_or(Error::Validation("Missing 'op'".into()))?;

        let result = match op {
            "add" => a + b,
            "sub" => a - b,
            "mul" => a * b,
            "div" if b != 0 => a / b,
            _ => return Err(Error::Validation("Invalid operation".into())),
        };

        Ok(json!({ "result": result }))
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let mut registry = HandlerRegistry::new();
    registry.register("calculator", Arc::new(CalculatorHandler::default()));

    // Use the registry...
    Ok(())
}
```

## Handler Types

### Native Handlers

Pure Rust handlers compiled into your binary:

```rust
#[async_trait::async_trait]
impl Handler for MyHandler {
    async fn handle(&self, params: Value) -> Result<Value> {
        // Your logic here
        Ok(json!({"status": "success"}))
    }
}
```

### CLI Handlers

Wrap command-line tools as MCP tools:

```yaml
- type: cli
  name: git_status
  description: "Check git status"
  command: git
  args: ["status", "--short"]
```

### HTTP Handlers

Proxy HTTP endpoints as MCP tools:

```yaml
- type: http
  name: fetch_user
  description: "Fetch user data"
  endpoint: "https://api.example.com/users/{id}"
  method: GET
```

### Pipeline Handlers

Chain multiple tools together:

```yaml
- type: pipeline
  name: process_data
  description: "Fetch and transform data"
  steps:
    - tool: fetch_data
    - tool: transform
    - tool: validate
```

## Middleware

Build composable middleware stacks:

```rust
use pforge_runtime::{MiddlewareChain, LoggingMiddleware, ValidationMiddleware, RecoveryMiddleware};

let mut chain = MiddlewareChain::new();

// Add logging
chain.add(Arc::new(LoggingMiddleware::new("my-server")));

// Add validation
chain.add(Arc::new(ValidationMiddleware::new(vec!["input".to_string()])));

// Add recovery with circuit breaker
chain.add(Arc::new(RecoveryMiddleware::new()
    .with_circuit_breaker(CircuitBreakerConfig {
        failure_threshold: 5,
        timeout: Duration::from_secs(60),
        success_threshold: 2,
    })));

// Execute through middleware
let result = chain.execute(params, |req| async move {
    // Handler logic
    Ok(json!({"output": req["input"]}))
}).await?;
```

## State Management

Persistent state for your handlers:

```rust
use pforge_runtime::{StateManager, MemoryStateManager};

let state = MemoryStateManager::new();

// Set a value
state.set("user:123", b"Alice".to_vec(), None).await?;

// Get a value
if let Some(value) = state.get("user:123").await? {
    println!("User: {}", String::from_utf8_lossy(&value));
}

// Delete a value
state.delete("user:123").await?;
```

## Circuit Breaker

Prevent cascading failures:

```rust
use pforge_runtime::{CircuitBreaker, CircuitBreakerConfig};

let cb = CircuitBreaker::new(CircuitBreakerConfig {
    failure_threshold: 5,      // Open after 5 failures
    timeout: Duration::from_secs(60),  // Try again after 60s
    success_threshold: 2,      // Close after 2 successes
});

let result = cb.call(|| async {
    // Potentially failing operation
    risky_operation().await
}).await?;
```

## Retry Policies

Resilient execution with exponential backoff:

```rust
use pforge_runtime::{retry_with_policy, RetryPolicy};

let policy = RetryPolicy::new(3)
    .with_backoff(Duration::from_millis(100), Duration::from_secs(5))
    .with_jitter(true);

let result = retry_with_policy(&policy, || async {
    unstable_operation().await
}).await?;
```

## Documentation

- [API Documentation](https://docs.rs/pforge-runtime)
- [User Guide](https://github.com/paiml/pforge/blob/main/docs/USER_GUIDE.md)
- [Architecture](https://github.com/paiml/pforge/blob/main/docs/ARCHITECTURE.md)
- [Examples](https://github.com/paiml/pforge/tree/main/examples)

## License

MIT
