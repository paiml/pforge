# pforge User Guide

**pforge** is a declarative framework for building Model Context Protocol (MCP) servers with zero boilerplate.

## Table of Contents

- [Quick Start](#quick-start)
- [Installation](#installation)
- [Creating Your First Server](#creating-your-first-server)
- [Configuration](#configuration)
- [Handlers](#handlers)
- [Advanced Features](#advanced-features)
- [Examples](#examples)
- [Best Practices](#best-practices)

## Quick Start

```bash
# Create a new project
pforge new my-server

# Navigate to project
cd my-server

# Build the project
pforge build

# Run the server
pforge serve
```

## Installation

### From Source

```bash
git clone https://github.com/paiml/pforge
cd pforge
cargo install --path crates/pforge-cli
```

### From crates.io (coming soon)

```bash
cargo install pforge-cli
```

## Creating Your First Server

### 1. Initialize Project

```bash
pforge new hello-world
cd hello-world
```

This creates:
```
hello-world/
├── Cargo.toml
├── pforge.yaml          # Configuration
└── src/
    ├── main.rs
    └── handlers/
        ├── mod.rs
        └── hello.rs      # Example handler
```

### 2. Configure Tools

Edit `pforge.yaml`:

```yaml
forge:
  name: hello-world
  version: 0.1.0
  transport: stdio

tools:
  - type: native
    name: greet
    description: "Greet a person by name"
    handler:
      path: handlers::hello::greet
    params:
      name:
        type: string
        required: true
      greeting:
        type: string
        required: false
        default: "Hello"
```

### 3. Implement Handler

`src/handlers/hello.rs`:

```rust
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Deserialize)]
pub struct GreetInput {
    pub name: String,
    pub greeting: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct GreetOutput {
    pub message: String,
}

pub async fn greet(input: GreetInput) -> Result<GreetOutput, Box<dyn std::error::Error>> {
    let greeting = input.greeting.unwrap_or_else(|| "Hello".to_string());
    Ok(GreetOutput {
        message: format!("{}, {}!", greeting, input.name),
    })
}
```

### 4. Run Server

```bash
pforge serve
```

## Configuration

### Tool Types

#### Native Handlers

Pure Rust functions with full type safety:

```yaml
tools:
  - type: native
    name: process_data
    description: "Process incoming data"
    handler:
      path: handlers::process::process_data
    params:
      data:
        type: object
        required: true
      options:
        type: object
        required: false
    timeout_ms: 5000
```

#### CLI Tools

Execute shell commands:

```yaml
tools:
  - type: cli
    name: git_status
    description: "Get git status"
    command: git
    args: ["status", "--short"]
    cwd: /path/to/repo
    env:
      GIT_PAGER: ""
    stream: false
```

#### HTTP Tools

Make HTTP requests:

```yaml
tools:
  - type: http
    name: fetch_user
    description: "Fetch user data from API"
    endpoint: "https://api.example.com/users/{{user_id}}"
    method: GET
    auth:
      type: bearer
      token: ${API_TOKEN}
    headers:
      Content-Type: application/json
```

#### Pipeline Tools

Chain multiple tools:

```yaml
tools:
  - type: pipeline
    name: process_pipeline
    description: "Multi-step processing"
    steps:
      - tool: fetch_data
        output_var: raw_data

      - tool: transform_data
        input:
          data: "{{raw_data}}"
        output_var: transformed

      - tool: store_data
        input:
          data: "{{transformed}}"
        condition: "{{transformed.valid}}"
        error_policy: continue
```

### Resources

Expose files and data:

```yaml
resources:
  - uri_template: "file:///{path}"
    handler:
      path: handlers::file_resource
    supports:
      - read
      - write
      - subscribe

  - uri_template: "db://{table}/{id}"
    handler:
      path: handlers::db_resource
    supports:
      - read
```

### Prompts

Template-based prompts:

```yaml
prompts:
  - name: code_review
    description: "Generate code review prompt"
    template: |
      Review the following {{language}} code:

      {{code}}

      Focus on: {{focus_areas}}
    arguments:
      language:
        type: string
        required: true
      code:
        type: string
        required: true
      focus_areas:
        type: string
        required: false
        default: "correctness, performance, style"
```

### State Management

Persistent state with Sled:

```yaml
state:
  backend: sled
  path: ./data/state.db
  options:
    cache_capacity: 1073741824  # 1GB
```

In-memory state for testing:

```yaml
state:
  backend: memory
  path: ""
```

## Handlers

### Type-Safe Native Handlers

```rust
use serde::{Deserialize, Serialize};
use pforge_runtime::Result;

#[derive(Deserialize)]
pub struct CalculateInput {
    pub operation: String,
    pub a: f64,
    pub b: f64,
}

#[derive(Serialize)]
pub struct CalculateOutput {
    pub result: f64,
}

pub async fn calculate(input: CalculateInput) -> Result<CalculateOutput> {
    let result = match input.operation.as_str() {
        "add" => input.a + input.b,
        "subtract" => input.a - input.b,
        "multiply" => input.a * input.b,
        "divide" => input.a / input.b,
        _ => return Err(pforge_runtime::Error::Handler(
            "Unknown operation".to_string()
        )),
    };

    Ok(CalculateOutput { result })
}
```

### Using State in Handlers

```rust
use pforge_runtime::{StateManager, Result};

pub async fn get_counter(
    state: &dyn StateManager,
) -> Result<u64> {
    let value = state.get("counter").await?;

    match value {
        Some(bytes) => {
            let count = u64::from_le_bytes(bytes.try_into().unwrap());
            Ok(count)
        }
        None => Ok(0),
    }
}

pub async fn increment_counter(
    state: &dyn StateManager,
) -> Result<u64> {
    let current = get_counter(state).await?;
    let new_count = current + 1;

    state.set("counter", new_count.to_le_bytes().to_vec(), None).await?;
    Ok(new_count)
}
```

## Advanced Features

### Middleware

Add cross-cutting concerns:

```rust
use pforge_runtime::{MiddlewareChain, LoggingMiddleware, ValidationMiddleware};

let mut chain = MiddlewareChain::new();

// Add logging
chain.add(Arc::new(LoggingMiddleware::new("api")));

// Add validation
chain.add(Arc::new(ValidationMiddleware::new(vec![
    "user_id".to_string(),
    "action".to_string(),
])));

// Execute with middleware
chain.execute(request, handler).await?;
```

### Timeout and Retry

```rust
use pforge_runtime::{RetryPolicy, with_timeout, retry_with_policy};
use std::time::Duration;

// Simple timeout
let result = with_timeout(
    Duration::from_secs(5),
    expensive_operation()
).await?;

// Retry with exponential backoff
let policy = RetryPolicy::new(3)
    .with_backoff(
        Duration::from_millis(100),
        Duration::from_secs(5),
    )
    .with_multiplier(2.0);

let result = retry_with_policy(&policy, || async {
    api_call().await
}).await?;
```

### Circuit Breaker

Prevent cascading failures:

```rust
use pforge_runtime::{CircuitBreaker, CircuitBreakerConfig};
use std::time::Duration;

let config = CircuitBreakerConfig {
    failure_threshold: 5,
    timeout: Duration::from_secs(60),
    success_threshold: 2,
};

let cb = CircuitBreaker::new(config);

// Protected call
let result = cb.call(|| async {
    potentially_failing_service().await
}).await?;
```

### Error Recovery

```rust
use pforge_runtime::{RecoveryMiddleware, CircuitBreakerConfig};

let recovery = RecoveryMiddleware::new()
    .with_circuit_breaker(CircuitBreakerConfig::default());

let tracker = recovery.error_tracker();

// Use in middleware chain
chain.add(Arc::new(recovery));

// Check error stats
println!("Total errors: {}", tracker.total_errors());
```

## Examples

### REST API Server

```yaml
forge:
  name: rest-api
  version: 1.0.0

tools:
  - type: http
    name: list_users
    endpoint: "https://api.example.com/users"
    method: GET
    auth:
      type: bearer
      token: ${API_KEY}

  - type: http
    name: create_user
    endpoint: "https://api.example.com/users"
    method: POST
    auth:
      type: bearer
      token: ${API_KEY}
```

### Data Processing Pipeline

```yaml
tools:
  - type: pipeline
    name: etl_pipeline
    steps:
      - tool: extract_data
        output_var: raw

      - tool: transform_data
        input:
          data: "{{raw}}"
        output_var: clean

      - tool: load_data
        input:
          data: "{{clean}}"
```

## Best Practices

### 1. Error Handling

Always use proper error types:

```rust
use pforge_runtime::{Error, Result};

pub async fn my_handler(input: Input) -> Result<Output> {
    let data = fetch_data()
        .await
        .map_err(|e| Error::Handler(format!("Fetch failed: {}", e)))?;

    Ok(process(data))
}
```

### 2. Configuration Management

Use environment variables for secrets:

```yaml
tools:
  - type: http
    name: api_call
    endpoint: ${API_ENDPOINT}
    auth:
      type: bearer
      token: ${API_TOKEN}
```

### 3. Testing

Write comprehensive tests:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_handler() {
        let input = MyInput { value: 42 };
        let result = my_handler(input).await.unwrap();
        assert_eq!(result.value, 43);
    }
}
```

### 4. Performance

- Use connection pooling for HTTP tools
- Enable caching for expensive operations
- Set appropriate timeouts
- Use circuit breakers for external services

### 5. Security

- Never commit secrets to version control
- Use environment variables for sensitive data
- Validate all inputs
- Set resource limits
- Enable audit logging

## CLI Commands

```bash
# Create new project
pforge new <name>

# Build project
pforge build

# Run server
pforge serve

# Development mode with hot reload
pforge dev

# Show version
pforge --version

# Show help
pforge --help
```

## Troubleshooting

### Server Won't Start

1. Check configuration syntax: `pforge build`
2. Verify all handlers exist
3. Check environment variables
4. Review logs for errors

### Handler Errors

1. Verify input types match schema
2. Check error messages for details
3. Enable debug logging
4. Test handlers in isolation

### Performance Issues

1. Enable caching
2. Use connection pooling
3. Set appropriate timeouts
4. Profile with flamegraphs

## Next Steps

- Read the [Architecture Guide](./ARCHITECTURE.md)
- Explore [Examples](../examples/)
- Check the [API Documentation](https://docs.rs/pforge-runtime)
- Report issues on [GitHub](https://github.com/paiml/pforge/issues)

## Support

- GitHub Issues: https://github.com/paiml/pforge/issues
- Documentation: https://docs.rs/pforge-runtime
- Community: https://github.com/paiml/pforge/discussions
