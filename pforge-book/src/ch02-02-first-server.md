# Your First Server

Let's build your first MCP server using pforge. We'll create a simple greeting server that demonstrates the core concepts.

## Scaffold a New Project

Create a new pforge project with the `new` command:

```bash
pforge new hello-server
cd hello-server
```

This creates a complete project structure:

```
hello-server/
├── pforge.yaml          # Server configuration
├── Cargo.toml           # Rust dependencies
├── .gitignore           # Git ignore rules
└── src/
    ├── lib.rs           # Library root
    └── handlers/
        ├── mod.rs       # Handler module exports
        └── greet.rs     # Example greeting handler
```

The scaffolded project includes:
- A working example handler
- Pre-configured dependencies
- Sensible defaults
- Git integration

## Explore the Configuration

Open `pforge.yaml` to see the server configuration:

```yaml
forge:
  name: hello-server
  version: 0.1.0
  transport: stdio

tools:
  - type: native
    name: greet
    description: "Greet a person by name"
    handler:
      path: handlers::greet::say_hello
    params:
      name:
        type: string
        required: true
        description: "Name of the person to greet"
```

Let's break this down:

### The `forge` Section

```yaml
forge:
  name: hello-server      # Server identifier
  version: 0.1.0          # Semantic version
  transport: stdio        # Communication channel (stdio, sse, websocket)
```

The `forge` section defines server metadata. The `stdio` transport means the server communicates via standard input/output, perfect for local development.

### The `tools` Section

```yaml
tools:
  - type: native                           # Handler type
    name: greet                            # Tool identifier
    description: "Greet a person by name"  # Human-readable description
    handler:
      path: handlers::greet::say_hello     # Rust function path
    params:
      name:                                # Parameter name
        type: string                       # Data type
        required: true                     # Validation rule
        description: "Name of the person to greet"
```

Each tool defines:
- **type**: How the tool executes (native, cli, http, pipeline)
- **name**: Unique identifier for the tool
- **description**: What the tool does
- **handler**: Where to find the implementation
- **params**: Input schema with type validation

## Understand the Handler

Open `src/handlers/greet.rs`:

```rust
use pforge_runtime::{Handler, Result};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, JsonSchema)]
pub struct GreetInput {
    pub name: String,
}

#[derive(Debug, Serialize, JsonSchema)]
pub struct GreetOutput {
    pub message: String,
}

pub struct GreetHandler;

#[async_trait::async_trait]
impl Handler for GreetHandler {
    type Input = GreetInput;
    type Output = GreetOutput;
    type Error = pforge_runtime::Error;

    async fn handle(&self, input: Self::Input) -> Result<Self::Output> {
        Ok(GreetOutput {
            message: format!("Hello, {}!", input.name),
        })
    }
}

// Alias for YAML reference
pub use GreetHandler as say_hello;
```

Let's examine each component:

### Input Type

```rust
#[derive(Debug, Deserialize, JsonSchema)]
pub struct GreetInput {
    pub name: String,
}
```

- `Deserialize`: Converts JSON to Rust struct
- `JsonSchema`: Auto-generates schema for validation
- Matches the `params` in `pforge.yaml`

### Output Type

```rust
#[derive(Debug, Serialize, JsonSchema)]
pub struct GreetOutput {
    pub message: String,
}
```

- `Serialize`: Converts Rust struct to JSON
- `JsonSchema`: Documents the response format
- Type-safe response structure

### Handler Implementation

```rust
#[async_trait::async_trait]
impl Handler for GreetHandler {
    type Input = GreetInput;
    type Output = GreetOutput;
    type Error = pforge_runtime::Error;

    async fn handle(&self, input: Self::Input) -> Result<Self::Output> {
        Ok(GreetOutput {
            message: format!("Hello, {}!", input.name),
        })
    }
}
```

The `Handler` trait requires:
- **Input**: Request parameters
- **Output**: Response data
- **Error**: Error type (usually `pforge_runtime::Error`)
- **handle()**: Async function with your logic

### Export Alias

```rust
pub use GreetHandler as say_hello;
```

This creates an alias matching the YAML `handler.path: handlers::greet::say_hello`.

## Build the Project

Compile your server:

```bash
cargo build
```

Expected output:

```
   Compiling pforge-runtime v0.1.0
   Compiling hello-server v0.1.0
    Finished dev [unoptimized + debuginfo] target(s) in 12.34s
```

For production builds:

```bash
cargo build --release
```

This enables optimizations for maximum performance.

## Run the Server

Start your server:

```bash
pforge serve
```

You should see:

```
[INFO] Starting hello-server v0.1.0
[INFO] Transport: stdio
[INFO] Registered tools: greet
[INFO] Server ready
```

The server is now listening on stdin/stdout for MCP protocol messages.

To stop the server, press `Ctrl+C`.

## Customize Your Server

Let's add a custom greeting parameter. Update `pforge.yaml`:

```yaml
tools:
  - type: native
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
```

Update `src/handlers/greet.rs`:

```rust
#[derive(Debug, Deserialize, JsonSchema)]
pub struct GreetInput {
    pub name: String,
    #[serde(default = "default_greeting")]
    pub greeting: String,
}

fn default_greeting() -> String {
    "Hello".to_string()
}

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

Rebuild and test:

```bash
cargo build
pforge serve
```

Now your server accepts both `name` and an optional `greeting` parameter.

## Project Structure Deep Dive

### `Cargo.toml`

Generated dependencies:

```toml
[package]
name = "hello-server"
version = "0.1.0"
edition = "2021"

[dependencies]
pforge-runtime = "0.1"
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
schemars = { version = "0.8", features = ["derive"] }
async-trait = "0.1"
tokio = { version = "1", features = ["full"] }
```

All dependencies are added automatically by `pforge new`.

### `src/lib.rs`

Module structure:

```rust
pub mod handlers;
```

This exports your handlers so pforge can find them.

### `.gitignore`

Common Rust ignores:

```
/target
Cargo.lock
*.swp
.DS_Store
```

Ready for version control from day one.

## Common Customizations

### Add a New Tool

Edit `pforge.yaml`:

```yaml
tools:
  - type: native
    name: greet
    # ... existing greet tool

  - type: native
    name: farewell
    description: "Say goodbye"
    handler:
      path: handlers::farewell_handler
    params:
      name:
        type: string
        required: true
```

Create `src/handlers/farewell.rs`:

```rust
use pforge_runtime::{Handler, Result};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, JsonSchema)]
pub struct FarewellInput {
    pub name: String,
}

#[derive(Debug, Serialize, JsonSchema)]
pub struct FarewellOutput {
    pub message: String,
}

pub struct FarewellHandler;

#[async_trait::async_trait]
impl Handler for FarewellHandler {
    type Input = FarewellInput;
    type Output = FarewellOutput;
    type Error = pforge_runtime::Error;

    async fn handle(&self, input: Self::Input) -> Result<Self::Output> {
        Ok(FarewellOutput {
            message: format!("Goodbye, {}!", input.name),
        })
    }
}

pub use FarewellHandler as farewell_handler;
```

Update `src/handlers/mod.rs`:

```rust
pub mod greet;
pub mod farewell;
```

Rebuild and you have two tools.

### Change Transport

For HTTP-based communication, update `pforge.yaml`:

```yaml
forge:
  name: hello-server
  version: 0.1.0
  transport: sse  # Server-Sent Events
```

Or for WebSocket:

```yaml
forge:
  name: hello-server
  version: 0.1.0
  transport: websocket
```

Each transport has different deployment characteristics covered in Chapter 19.

## Development Workflow

The typical development cycle:

1. **Edit** `pforge.yaml` to define tools
2. **Implement** handlers in `src/handlers/`
3. **Build** with `cargo build`
4. **Test** with `cargo test`
5. **Run** with `pforge serve`

For rapid iteration, use watch mode:

```bash
cargo watch -x build -x test
```

This rebuilds and tests automatically on file changes.

## What's Next

You now have a working MCP server. In the next section, we'll test it thoroughly and learn debugging techniques.

---

Next: [Testing Your Server](ch02-03-testing.md)
