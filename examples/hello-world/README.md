# Hello World - pforge Example

The simplest possible MCP server using pforge.

## What It Does

Provides two tools:
- `greet` - Native Rust handler that greets a person
- `whoami` - CLI handler that runs the system `whoami` command

## Running

```bash
cd examples/hello-world
cargo run
```

## Configuration

See `pforge.yaml` for the declarative configuration.

## Handler Implementation

The native `greet` handler is implemented in `src/handlers/greet.rs` with:
- Type-safe input/output with Serde
- JsonSchema generation for MCP
- Async execution with async_trait
- Unit tests

## Testing the Handler

```bash
cargo test
```

## How It Works

1. **Configuration** (`pforge.yaml`):
   - Defines tools and their parameters
   - Specifies handler types (native vs CLI)

2. **Handler Implementation** (`src/handlers/greet.rs`):
   - Implements the `Handler` trait
   - Type-safe with Serde + JsonSchema
   - Async by default

3. **Server Setup** (`src/main.rs`):
   - Creates HandlerRegistry
   - Registers handlers
   - Starts MCP server

This is the foundation for any pforge server!
