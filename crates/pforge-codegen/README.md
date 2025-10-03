# pforge-codegen

[![crates.io](https://img.shields.io/crates/v/pforge-codegen.svg)](https://crates.io/crates/pforge-codegen)
[![Documentation](https://docs.rs/pforge-codegen/badge.svg)](https://docs.rs/pforge-codegen)

Code generation engine for pforge - converts YAML configuration into optimized Rust code.

## Features

- **YAML → Rust**: Transform declarative configuration into type-safe Rust code
- **Handler Registry**: Generate optimized handler registration code
- **Schema Generation**: Automatic JSON Schema generation for tool parameters
- **Type Safety**: Compile-time validation of generated code
- **Zero Runtime Cost**: All code generation happens at build time
- **Optimized Output**: Clean, idiomatic Rust code with minimal overhead

## Installation

```bash
cargo add pforge-codegen
```

## Usage

### Generate from Configuration

```rust
use pforge_codegen::generate_server;
use pforge_config::ForgeConfig;

let config: ForgeConfig = serde_yaml::from_str(yaml_config)?;

// Generate Rust code
let generated_code = generate_server(&config)?;

// Write to file
std::fs::write("src/generated.rs", generated_code)?;
```

### What Gets Generated

For this YAML configuration:

```yaml
forge:
  name: calculator
  version: 0.1.0

tools:
  - type: native
    name: add
    description: "Add two numbers"
    handler:
      path: handlers::add_handler
    params:
      a: { type: number, required: true }
      b: { type: number, required: true }
```

The codegen generates:

```rust
// Handler registration
pub fn register_handlers(registry: &mut HandlerRegistry) -> Result<()> {
    registry.register("add", Arc::new(handlers::add_handler()))?;
    Ok(())
}

// JSON Schema for parameters
pub fn add_schema() -> schemars::schema::RootSchema {
    schemars::schema_for!(AddParams)
}

#[derive(Serialize, Deserialize, JsonSchema)]
pub struct AddParams {
    pub a: f64,
    pub b: f64,
}

// Server initialization
pub async fn start_server() -> Result<()> {
    let mut registry = HandlerRegistry::new();
    register_handlers(&mut registry)?;

    // Start MCP server with stdio transport
    pmcp::start_stdio_server(registry).await
}
```

## Code Generation Pipeline

1. **Parse Configuration**: Load and validate YAML using `pforge-config`
2. **AST Generation**: Build Rust Abstract Syntax Tree
3. **Optimization**: Apply compiler optimizations
4. **Code Emission**: Generate clean, formatted Rust code

## Generated Components

### Handler Registration

Generates optimized registry initialization:

```rust
let mut registry = HandlerRegistry::new();
registry.register("tool1", Arc::new(Tool1Handler::default()))?;
registry.register("tool2", Arc::new(Tool2Handler::default()))?;
```

### Type Definitions

Creates type-safe parameter structs:

```rust
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ToolParams {
    pub input: String,
    #[serde(default)]
    pub optional: Option<i64>,
}
```

### Schema Generation

Automatic JSON Schema for MCP protocol:

```rust
pub fn tool_schema() -> schemars::schema::RootSchema {
    schemars::schema_for!(ToolParams)
}
```

### Transport Setup

Generates transport-specific initialization:

```rust
// For stdio transport
pmcp::start_stdio_server(registry).await?;

// For SSE transport
pmcp::start_sse_server("0.0.0.0:3000", registry).await?;

// For WebSocket transport
pmcp::start_ws_server("0.0.0.0:3001", registry).await?;
```

## CLI Integration

The code generator is used by `pforge-cli` during the build process:

```bash
# Automatically runs codegen
pforge build

# Or explicitly
pforge codegen
```

## Advanced Usage

### Custom Templates

Provide custom code generation templates:

```rust
use pforge_codegen::{CodegenConfig, generate_with_config};

let config = CodegenConfig {
    template_dir: Some("templates/".into()),
    output_format: OutputFormat::Formatted,
    optimize: true,
};

let code = generate_with_config(&forge_config, &config)?;
```

### Inline Handlers

Generate handlers with inline logic:

```yaml
tools:
  - type: native
    name: echo
    description: "Echo input"
    handler:
      inline: |
        Ok(json!({ "output": params["input"] }))
    params:
      input: { type: string, required: true }
```

Generates:

```rust
#[async_trait::async_trait]
impl Handler for EchoHandler {
    async fn handle(&self, params: Value) -> Result<Value> {
        Ok(json!({ "output": params["input"] }))
    }
}
```

## Generated Code Quality

All generated code follows Rust best practices:

- ✅ **Formatted**: Uses `rustfmt` for consistent style
- ✅ **Clippy Clean**: Passes `clippy` with no warnings
- ✅ **Type Safe**: Full type inference and checking
- ✅ **Documented**: Includes doc comments from YAML descriptions
- ✅ **Optimized**: Dead code elimination and inlining hints

## Performance

The code generator is designed for fast build times:

- **< 10ms** for typical configurations (< 10 tools)
- **< 100ms** for large configurations (100+ tools)
- **Incremental**: Only regenerates when configuration changes

## Documentation

- [API Documentation](https://docs.rs/pforge-codegen)
- [Code Generation Guide](https://github.com/paiml/pforge/blob/main/docs/ARCHITECTURE.md#code-generation)
- [Examples](https://github.com/paiml/pforge/tree/main/examples)

## License

MIT
