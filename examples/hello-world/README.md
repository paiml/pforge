# Hello World - pforge Example

**The simplest production-ready MCP server using pforge.**

This example demonstrates the core concepts of pforge in under 100 lines of code:
- ✅ Native Rust handlers with type safety
- ✅ CLI tool integration
- ✅ Declarative YAML configuration
- ✅ Async execution with tokio
- ✅ Comprehensive testing

---

## Quick Start

**Prerequisites**: Rust 1.75+ and Cargo

```bash
# Navigate to the example directory
cd examples/hello-world

# Run the server
cargo run

# Expected output:
# ╔═══════════════════════════════════════╗
# ║   Hello World MCP Server              ║
# ║   Powered by pforge v0.1.0            ║
# ╚═══════════════════════════════════════╝
#
# This example demonstrates:
#   ✓ Native Rust handler (greet)
#   ✓ CLI handler (whoami)
#   ✓ Type-safe input/output
#   ✓ Async execution
#   ✓ YAML configuration
#
# Available tools:
#   • greet(name, greeting?) - Greet a person
#   • whoami() - Get current username
```

---

## What It Does

### Tools Provided

1. **greet** - Native Rust handler
   - **Type**: Native handler (compiled Rust code)
   - **Input**: `name` (required), `greeting` (optional, default: "Hello")
   - **Output**: Formatted greeting message
   - **Performance**: < 1μs execution time

2. **whoami** - CLI handler
   - **Type**: CLI wrapper
   - **Input**: None
   - **Output**: Current system username
   - **Performance**: ~5ms (system call overhead)

### Example Usage with MCP Client

```json
// Request to greet tool
{
  "jsonrpc": "2.0",
  "id": 1,
  "method": "tools/call",
  "params": {
    "name": "greet",
    "arguments": {
      "name": "Alice",
      "greeting": "Hi"
    }
  }
}

// Response
{
  "jsonrpc": "2.0",
  "id": 1,
  "result": {
    "message": "Hi, Alice!"
  }
}
```

---

## Architecture

### Project Structure

```
hello-world/
├── pforge.yaml              # Declarative configuration
├── Cargo.toml               # Rust dependencies
├── src/
│   ├── main.rs              # Server entry point
│   └── handlers/
│       ├── mod.rs           # Handler module exports
│       └── greet.rs         # Native greet handler implementation
└── README.md                # This file
```

### How It Works

#### 1. Configuration (`pforge.yaml`)

The YAML configuration declaratively defines the MCP server:

```yaml
forge:
  name: hello-world
  version: 0.1.0
  transport: stdio         # MCP transport (stdio, SSE, WebSocket)

tools:
  - type: native            # Native Rust handler
    name: greet
    description: "Greet a person by name"
    handler:
      path: handlers::greet::say_hello
    params:
      name:
        type: string
        required: true
        description: "Name of the person to greet"
      greeting:
        type: string
        required: false
        default: "Hello"
        description: "Custom greeting word"

  - type: cli               # CLI wrapper handler
    name: whoami
    description: "Get current username"
    command: whoami
    args: []
```

#### 2. Handler Implementation (`src/handlers/greet.rs`)

The native handler implements pforge's `Handler` trait:

```rust
use pforge_runtime::{Handler, Result};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

// Input type with JsonSchema for MCP protocol
#[derive(Debug, Deserialize, JsonSchema)]
pub struct GreetInput {
    pub name: String,
    #[serde(default = "default_greeting")]
    pub greeting: String,
}

fn default_greeting() -> String {
    "Hello".to_string()
}

// Output type with JsonSchema
#[derive(Debug, Serialize, JsonSchema)]
pub struct GreetOutput {
    pub message: String,
}

// Handler implementation
pub struct GreetHandler;

#[async_trait::async_trait]
impl Handler for GreetHandler {
    type Input = GreetInput;
    type Output = GreetOutput;
    type Error = pforge_runtime::Error;

    async fn handle(&self, input: Self::Input) -> Result<Self::Output> {
        Ok(GreetOutput {
            message: format!("{}, {}!", input.greeting, input.name),
        })
    }
}
```

**Key Features**:
- ✅ Type-safe: Input/output validated at compile time
- ✅ JsonSchema: Automatic schema generation for MCP
- ✅ Async: Non-blocking execution with `async_trait`
- ✅ Testable: Pure logic, easy to unit test

#### 3. Server Setup (`src/main.rs`)

The main function bootstraps the MCP server:

```rust
use pforge_config::parse_config;
use pforge_runtime::McpServer;
use std::path::Path;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1. Load YAML configuration
    let config = parse_config(Path::new("pforge.yaml"))?;

    // 2. Create MCP server from config
    let server = McpServer::new(config);

    // 3. Register native handlers
    let registry = server.registry();
    {
        let mut reg = registry.write().await;
        reg.register("greet", handlers::greet::GreetHandler);
    }

    // 4. Run the server (starts MCP protocol loop)
    server.run().await?;

    Ok(())
}
```

**What Happens**:
1. Parse `pforge.yaml` configuration
2. Create `McpServer` instance
3. Register native handlers (CLI handlers auto-registered from config)
4. Start MCP protocol loop (stdio/SSE/WebSocket based on config)

---

## Testing

### Run Unit Tests

```bash
cargo test
```

### Test Coverage

```bash
# Install tarpaulin (if needed)
cargo install cargo-tarpaulin

# Run coverage
cargo tarpaulin --out Html
# Open tarpaulin-report.html in browser
```

### Unit Test Examples

```rust
#[tokio::test]
async fn test_greet_default() {
    let handler = GreetHandler;
    let input = GreetInput {
        name: "World".to_string(),
        greeting: "Hello".to_string(),
    };

    let result = handler.handle(input).await.unwrap();
    assert_eq!(result.message, "Hello, World!");
}

#[tokio::test]
async fn test_greet_custom() {
    let handler = GreetHandler;
    let input = GreetInput {
        name: "Alice".to_string(),
        greeting: "Hi".to_string(),
    };

    let result = handler.handle(input).await.unwrap();
    assert_eq!(result.message, "Hi, Alice!");
}
```

---

## Development Workflow

### 1. Add a New Handler

**Option A: Native Rust Handler**

```bash
# Create new handler file
cat > src/handlers/goodbye.rs << 'EOF'
use pforge_runtime::{Handler, Result};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, JsonSchema)]
pub struct GoodbyeInput {
    pub name: String,
}

#[derive(Debug, Serialize, JsonSchema)]
pub struct GoodbyeOutput {
    pub message: String,
}

pub struct GoodbyeHandler;

#[async_trait::async_trait]
impl Handler for GoodbyeHandler {
    type Input = GoodbyeInput;
    type Output = GoodbyeOutput;
    type Error = pforge_runtime::Error;

    async fn handle(&self, input: Self::Input) -> Result<Self::Output> {
        Ok(GoodbyeOutput {
            message: format!("Goodbye, {}!", input.name),
        })
    }
}
EOF

# Export from handlers/mod.rs
echo "pub mod goodbye;" >> src/handlers/mod.rs

# Register in main.rs
# Add to registration block:
# reg.register("goodbye", handlers::goodbye::GoodbyeHandler);
```

**Option B: CLI Handler (no code needed!)**

Just add to `pforge.yaml`:
```yaml
tools:
  - type: cli
    name: date
    description: "Get current date"
    command: date
    args: []
```

### 2. Run in Development Mode

```bash
# Run with debug logging
RUST_LOG=pforge=debug cargo run

# Run with hot reload (cargo-watch)
cargo install cargo-watch
cargo watch -x run
```

### 3. Build for Production

```bash
# Optimized release build
cargo build --release

# Binary location
./target/release/hello-world
```

---

## Extending This Example

### Add State Management

Update `pforge.yaml`:
```yaml
state:
  backend: memory
  ttl_seconds: 3600
  max_size: 1048576  # 1MB
```

Use in handlers:
```rust
use pforge_runtime::StateManager;

async fn handle(&self, input: Self::Input) -> Result<Self::Output> {
    let state = self.state_manager.get("greeting_count").await?;
    // ... use state
}
```

### Add Middleware

```rust
use pforge_runtime::{LoggingMiddleware, TimeoutMiddleware};

let server = McpServer::new(config)
    .with_middleware(LoggingMiddleware::new())
    .with_middleware(TimeoutMiddleware::new(Duration::from_secs(30)));
```

### Add Resources

Update `pforge.yaml`:
```yaml
resources:
  - uri: "file://greetings.txt"
    name: "greetings"
    description: "List of available greetings"
    mime_type: "text/plain"
```

### Add Prompts

Update `pforge.yaml`:
```yaml
prompts:
  - name: "greeting_prompt"
    description: "Generate a creative greeting"
    arguments:
      - name: "style"
        description: "Greeting style (formal, casual, funny)"
        required: true
```

---

## Troubleshooting

### Server Not Starting

**Problem**: `Error: failed to parse config`

**Solution**: Validate YAML syntax
```bash
# Check YAML is valid
cat pforge.yaml | yaml-validator

# Common issues:
# - Indentation (use 2 spaces, not tabs)
# - Missing quotes around strings with special chars
# - Incorrect nesting
```

### Handler Not Found

**Problem**: `Error: handler 'greet' not found`

**Solution**: Ensure handler is registered in main.rs
```rust
// Check this line exists:
reg.register("greet", handlers::greet::GreetHandler);
```

### Type Mismatch Errors

**Problem**: `Error: failed to deserialize input`

**Solution**: Verify input matches JsonSchema
```bash
# Generate schema for debugging
cargo run --example dump-schema
```

---

## Performance

### Benchmarks (on M1 Mac, 2023)

| Metric | Value |
|--------|-------|
| **Cold start** | < 50ms |
| **Handler dispatch** | 83-90ns |
| **Greet handler** | ~500ns |
| **CLI handler (whoami)** | ~5ms |
| **Memory usage** | ~8MB |
| **Throughput** | > 100K req/s |

### Optimization Tips

1. **Use release builds** (`cargo build --release`)
2. **Enable LTO** in `Cargo.toml`:
   ```toml
   [profile.release]
   lto = true
   codegen-units = 1
   ```
3. **Profile with flamegraph**:
   ```bash
   cargo install flamegraph
   cargo flamegraph -- serve
   ```

---

## Next Steps

After understanding this example:

1. **Explore Advanced Examples**:
   - `examples/pmat-server/` - Code analysis integration
   - `examples/polyglot-server/` - Multi-language handlers (Python, Go, Node.js)
   - `examples/production-server/` - Full production setup

2. **Read Documentation**:
   - [User Guide](../../USER_GUIDE.md) - Complete feature reference
   - [Architecture](../../ARCHITECTURE.md) - Deep dive into internals
   - [pforge Specification](../../docs/specifications/pforge-specification.md)

3. **Build Your Own Server**:
   ```bash
   pforge new my-server
   cd my-server
   pforge serve
   ```

---

## Learn More

- **MCP Protocol**: https://spec.modelcontextprotocol.io/
- **pforge Repository**: https://github.com/paiml/pforge
- **pmcp SDK**: https://github.com/paiml/pmcp
- **Issue Tracker**: https://github.com/paiml/pforge/issues

---

**License**: MIT

**Maintained by**: pforge team

**Version**: 0.1.0
