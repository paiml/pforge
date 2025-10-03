# pforge-runtime Examples

This directory contains runnable examples demonstrating pforge-runtime features.

## Running Examples

All examples can be run with `cargo run --example <name>`:

### Calculator - Basic Handler

Demonstrates basic handler registration and dispatch with type-safe input/output.

```bash
cargo run --example calculator
```

**Features**:
- Type-safe handler implementation
- Error handling (division by zero)
- Handler registry usage
- Multiple operations (add, subtract, multiply, divide)

### Middleware Demo - Middleware Chain

Shows how to use middleware for request/response processing.

```bash
cargo run --example middleware_demo
```

**Features**:
- LoggingMiddleware for request/response logging
- Custom TimingMiddleware for performance tracking
- Before/after middleware phases
- Middleware composition

## Example Output

### Calculator

```
🧮 Calculator Handler Example

✅ Registered 'calculate' handler

📊 Running test cases:

  5 add 3 = 8
  10 subtract 4 = 6
  6 multiply 7 = 42
  20 divide 4 = 5

🔍 Testing error handling:

  ✅ Correctly caught error: Handler error: Division by zero

✨ Example complete!
```

### Middleware Demo

```
🔗 Middleware Example

✅ Created middlewares:
   1. LoggingMiddleware - logs requests/responses
   2. TimingMiddleware - tracks request duration

📤 Processing request through middlewares:

1. LoggingMiddleware.before()
[echo] Request: {"message":"Hello from middleware!"}
2. TimingMiddleware.before()
  ⏱️  Request started
3. Execute handler
4. TimingMiddleware.after()
  ⏱️  Request completed in 68.834µs
5. LoggingMiddleware.after()
[echo] Response: {"echo":"Hello from middleware!","length":22}

📥 Final response:
{
  "echo": "Hello from middleware!",
  "length": 22
}

✨ Example complete!
```

## Creating Your Own Example

1. Create a new file in `crates/pforge-runtime/examples/your_example.rs`
2. Use the handler pattern from existing examples
3. Run with `cargo run --example your_example`

Example template:

```rust
use pforge_runtime::{Handler, HandlerRegistry, Result};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
struct MyInput {
    // Your input fields
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
struct MyOutput {
    // Your output fields
}

struct MyHandler;

#[async_trait::async_trait]
impl Handler for MyHandler {
    type Input = MyInput;
    type Output = MyOutput;
    type Error = pforge_runtime::Error;

    async fn handle(&self, input: Self::Input) -> Result<Self::Output> {
        // Your handler logic
        todo!()
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let mut registry = HandlerRegistry::new();
    registry.register("my_handler", MyHandler);

    // Your example code

    Ok(())
}
```

## More Examples

For complete MCP server examples, see the top-level `examples/` directory:

- `examples/hello-world/` - Minimal viable server
- `examples/pmat-server/` - PMAT code analysis integration
- `examples/polyglot-server/` - Multi-language bridge example
- `examples/production-server/` - Full-featured production server
- `examples/telemetry-server/` - Observability and metrics
