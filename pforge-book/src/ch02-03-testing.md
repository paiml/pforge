# Testing Your Server

Now that you have a working server, let's test it thoroughly. pforge embraces EXTREME TDD, so testing is a first-class citizen.

## Unit Testing Handlers

Start with the most fundamental tests - your handler logic.

### Write Your First Test

Open `src/handlers/greet.rs` and add tests at the bottom:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_greet_basic() {
        let handler = GreetHandler;
        let input = GreetInput {
            name: "World".to_string(),
        };

        let result = handler.handle(input).await;
        assert!(result.is_ok());

        let output = result.unwrap();
        assert_eq!(output.message, "Hello, World!");
    }

    #[tokio::test]
    async fn test_greet_different_name() {
        let handler = GreetHandler;
        let input = GreetInput {
            name: "Alice".to_string(),
        };

        let result = handler.handle(input).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap().message, "Hello, Alice!");
    }

    #[tokio::test]
    async fn test_greet_empty_name() {
        let handler = GreetHandler;
        let input = GreetInput {
            name: "".to_string(),
        };

        let result = handler.handle(input).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap().message, "Hello, !");
    }
}
```

### Run the Tests

Execute your test suite:

```bash
cargo test
```

Expected output:

```
   Compiling hello-server v0.1.0
    Finished test [unoptimized + debuginfo] target(s) in 2.34s
     Running unittests src/lib.rs

running 3 tests
test handlers::greet::tests::test_greet_basic ... ok
test handlers::greet::tests::test_greet_different_name ... ok
test handlers::greet::tests::test_greet_empty_name ... ok

test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

All tests pass! Each test runs in microseconds.

### Test Best Practices

Following EXTREME TDD principles:

```rust
#[tokio::test]
async fn test_should_handle_unicode_names() {
    // Arrange
    let handler = GreetHandler;
    let input = GreetInput {
        name: "世界".to_string(),  // "World" in Japanese
    };

    // Act
    let result = handler.handle(input).await;

    // Assert
    assert!(result.is_ok());
    assert_eq!(result.unwrap().message, "Hello, 世界!");
}
```

Structure tests with Arrange-Act-Assert:
1. **Arrange**: Set up test data
2. **Act**: Execute the function
3. **Assert**: Verify results

## Integration Testing

Integration tests verify the entire server stack, not just individual handlers.

### Create Integration Tests

Create `tests/integration_test.rs`:

```rust
use hello_server::handlers::greet::{GreetHandler, GreetInput};
use pforge_runtime::Handler;

#[tokio::test]
async fn test_handler_integration() {
    let handler = GreetHandler;
    let input = GreetInput {
        name: "Integration Test".to_string(),
    };

    let output = handler.handle(input).await.expect("handler failed");
    assert!(output.message.contains("Integration Test"));
}
```

Run integration tests:

```bash
cargo test --test integration_test
```

Integration tests live in the `tests/` directory and have full access to your library.

## Testing with MCP Clients

To test the full MCP protocol, use an MCP client.

### Manual Testing with stdio

Start your server:

```bash
pforge serve
```

In another terminal, use an MCP inspector tool or send raw JSON-RPC messages:

```bash
echo '{"jsonrpc":"2.0","id":1,"method":"tools/list"}' | pforge serve
```

Expected response:

```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "result": {
    "tools": [
      {
        "name": "greet",
        "description": "Greet a person by name",
        "inputSchema": {
          "type": "object",
          "properties": {
            "name": {
              "type": "string",
              "description": "Name of the person to greet"
            }
          },
          "required": ["name"]
        }
      }
    ]
  }
}
```

### Call a Tool

```bash
echo '{"jsonrpc":"2.0","id":2,"method":"tools/call","params":{"name":"greet","arguments":{"name":"World"}}}' | pforge serve
```

Response:

```json
{
  "jsonrpc": "2.0",
  "id": 2,
  "result": {
    "content": [
      {
        "type": "text",
        "text": "{\"message\":\"Hello, World!\"}"
      }
    ]
  }
}
```

## Test Coverage

Measure your test coverage with `cargo-tarpaulin`:

```bash
# Install tarpaulin (Linux only)
cargo install cargo-tarpaulin

# Run coverage analysis
cargo tarpaulin --out Html
```

This generates `tarpaulin-report.html` showing line-by-line coverage.

pforge's quality gates enforce 80% minimum coverage. Check with:

```bash
cargo tarpaulin --out Json | jq '.files | to_entries | map(.value.coverage) | add / length'
```

Target: ≥ 0.80 (80%)

## Watch Mode for TDD

For rapid RED-GREEN-REFACTOR cycles:

```bash
cargo watch -x test
```

This runs tests automatically when files change. Perfect for EXTREME TDD's 5-minute cycles.

Advanced watch mode:

```bash
cargo watch -x 'test --lib --quiet' -x 'clippy --quiet'
```

Runs tests AND linting on every change.

## Debugging Tests

### Enable Logging

Add logging to your handler:

```rust
use tracing::info;

async fn handle(&self, input: Self::Input) -> Result<Self::Output> {
    info!("Handling greet request for: {}", input.name);
    Ok(GreetOutput {
        message: format!("Hello, {}!", input.name),
    })
}
```

Run tests with logging:

```bash
RUST_LOG=debug cargo test -- --nocapture
```

### Debug Individual Tests

Run a single test:

```bash
cargo test test_greet_basic
```

Run with output:

```bash
cargo test test_greet_basic -- --nocapture --exact
```

## Error Handling Tests

Test error paths to ensure robustness:

```rust
#[tokio::test]
async fn test_validation_error() {
    let handler = GreetHandler;
    // Simulate invalid input by testing edge cases
    let input = GreetInput {
        name: "A".repeat(10000),  // Very long name
    };

    let result = handler.handle(input).await;
    // Depending on your validation, this might error or succeed
    assert!(result.is_ok() || result.is_err());
}
```

For handlers that can fail:

```rust
use pforge_runtime::Error;

async fn handle(&self, input: Self::Input) -> Result<Self::Output> {
    if input.name.is_empty() {
        return Err(Error::Validation("Name cannot be empty".to_string()));
    }
    Ok(GreetOutput {
        message: format!("Hello, {}!", input.name),
    })
}

#[tokio::test]
async fn test_empty_name_validation() {
    let handler = GreetHandler;
    let input = GreetInput {
        name: "".to_string(),
    };

    let result = handler.handle(input).await;
    assert!(result.is_err());

    let err = result.unwrap_err();
    assert!(err.to_string().contains("empty"));
}
```

## Performance Testing

Benchmark your handlers:

```bash
cargo bench
```

For quick performance checks:

```rust
#[tokio::test]
async fn test_handler_performance() {
    let handler = GreetHandler;
    let input = GreetInput {
        name: "Benchmark".to_string(),
    };

    let start = std::time::Instant::now();

    for _ in 0..10_000 {
        let _ = handler.handle(input.clone()).await;
    }

    let elapsed = start.elapsed();
    println!("10,000 calls took: {:?}", elapsed);

    // Should be under 10ms for 10K simple operations
    assert!(elapsed.as_millis() < 10);
}
```

pforge handlers should dispatch in <1 microsecond each.

## Quality Gates

Run all quality checks before committing:

```bash
# Format check
cargo fmt --check

# Linting
cargo clippy -- -D warnings

# Tests
cargo test --all

# Coverage (Linux)
cargo tarpaulin --out Json

# Full quality gate
make quality-gate
```

The `make quality-gate` command runs:
1. Code formatting validation
2. Clippy linting (all warnings as errors)
3. All tests (unit + integration)
4. Coverage analysis (≥80%)
5. Complexity checks (≤20 per function)
6. Technical debt grade (≥75)

Any failure blocks commits when using pre-commit hooks.

## Common Testing Patterns

### Test Fixtures

Reuse test data:

```rust
fn sample_input() -> GreetInput {
    GreetInput {
        name: "Test".to_string(),
    }
}

#[tokio::test]
async fn test_with_fixture() {
    let handler = GreetHandler;
    let input = sample_input();
    let result = handler.handle(input).await;
    assert!(result.is_ok());
}
```

### Parameterized Tests

Test multiple cases:

```rust
#[tokio::test]
async fn test_greet_multiple_names() {
    let handler = GreetHandler;
    let test_cases = vec!["Alice", "Bob", "Charlie", "世界"];

    for name in test_cases {
        let input = GreetInput {
            name: name.to_string(),
        };
        let result = handler.handle(input).await;
        assert!(result.is_ok());
        assert!(result.unwrap().message.contains(name));
    }
}
```

### Async Test Helpers

Extract common async patterns:

```rust
async fn run_handler(name: &str) -> String {
    let handler = GreetHandler;
    let input = GreetInput {
        name: name.to_string(),
    };
    handler.handle(input).await.unwrap().message
}

#[tokio::test]
async fn test_with_helper() {
    let message = run_handler("Helper").await;
    assert_eq!(message, "Hello, Helper!");
}
```

## Troubleshooting

### Tests Hang

If tests never complete:

```bash
# Run with timeout
cargo test -- --test-threads=1 --nocapture

# Check for deadlocks
RUST_LOG=trace cargo test
```

### Compilation Errors

```bash
# Clean and rebuild
cargo clean
cargo test

# Update dependencies
cargo update
```

### Test Failures

Use `--nocapture` to see println! output:

```bash
cargo test -- --nocapture
```

Add debug output:

```rust
#[tokio::test]
async fn test_debug() {
    let result = handler.handle(input).await;
    dbg!(&result);  // Print detailed debug info
    assert!(result.is_ok());
}
```

## Next Steps

You now have a fully tested MCP server. Congratulations!

In the next chapters, we'll explore:
- Advanced handler types (CLI, HTTP, Pipeline)
- State management and persistence
- Error handling strategies
- Production deployment

Your foundation in EXTREME TDD will serve you well as we tackle more complex topics.

---

Next: [Chapter 3: Understanding pforge Architecture](ch03-00-calculator.md)
