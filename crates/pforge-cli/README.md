# pforge-cli

[![crates.io](https://img.shields.io/crates/v/pforge-cli.svg)](https://crates.io/crates/pforge-cli)
[![Documentation](https://docs.rs/pforge-cli/badge.svg)](https://docs.rs/pforge-cli)

Command-line interface for the pforge framework - build Model Context Protocol (MCP) servers from declarative YAML configuration.

## Installation

```bash
cargo install pforge-cli
```

## Quick Start

```bash
# Create a new MCP server project
pforge new my-server
cd my-server

# Run the server
pforge serve

# Build for production
pforge build --release
```

## Commands

- **`pforge new <name>`** - Scaffold a new MCP server project
- **`pforge serve`** - Run the MCP server in development mode
- **`pforge build`** - Build the server binary
- **`pforge dev`** - Development mode with hot reload (planned)
- **`pforge test`** - Run tests for your handlers
- **`pforge quality`** - Run quality gates (formatting, linting, tests, coverage)

## Features

- **Zero Boilerplate**: Define your MCP server entirely in YAML
- **Type Safety**: Automatically generated Rust code with full type checking
- **Hot Reload**: Fast development cycle with automatic reloading (coming soon)
- **Quality Built-in**: Integrated quality gates and PMAT enforcement
- **Multiple Transports**: stdio, SSE, and WebSocket support

## Example

Create a `pforge.yaml`:

```yaml
forge:
  name: my-server
  version: 0.1.0
  transport: stdio

tools:
  - type: native
    name: greet
    description: "Greet someone"
    handler:
      path: handlers::greet_handler
    params:
      name: { type: string, required: true }
```

Implement `src/handlers/mod.rs`:

```rust
use pforge_runtime::prelude::*;
use serde_json::{json, Value};

#[async_trait::async_trait]
impl Handler for GreetHandler {
    async fn handle(&self, params: Value) -> Result<Value> {
        let name = params["name"].as_str().unwrap_or("World");
        Ok(json!({ "message": format!("Hello, {}!", name) }))
    }
}
```

Then run: `pforge serve`

## Documentation

- [User Guide](https://github.com/paiml/pforge/blob/main/docs/USER_GUIDE.md)
- [API Documentation](https://docs.rs/pforge-runtime)
- [Examples](https://github.com/paiml/pforge/tree/main/examples)

## License

MIT
