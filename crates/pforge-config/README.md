# pforge-config

[![crates.io](https://img.shields.io/crates/v/pforge-config.svg)](https://crates.io/crates/pforge-config)
[![Documentation](https://docs.rs/pforge-config/badge.svg)](https://docs.rs/pforge-config)

Configuration parsing and validation for pforge MCP servers.

## Features

- **YAML Configuration**: Declarative MCP server definition
- **Type-Safe Parsing**: Serde-based deserialization with validation
- **Tool Definitions**: Support for Native, CLI, HTTP, and Pipeline handlers
- **Resource Templates**: URI-based resource configuration
- **Prompt Templates**: Mustache-style prompt definitions
- **Schema Validation**: Automatic validation of configuration structure

## Installation

```bash
cargo add pforge-config
```

## Usage

### Parse Configuration

```rust
use pforge_config::ForgeConfig;

let yaml = r#"
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
"#;

let config: ForgeConfig = serde_yaml::from_str(yaml)?;
println!("Server: {} v{}", config.forge.name, config.forge.version);
```

### Validate Configuration

```rust
use pforge_config::validate_config;

let config: ForgeConfig = serde_yaml::from_str(yaml)?;

// Validates:
// - No duplicate tool names
// - Valid handler references
// - Correct parameter schemas
// - URI template syntax
validate_config(&config)?;
```

## Configuration Structure

### Forge Metadata

```yaml
forge:
  name: server-name        # Required
  version: 0.1.0          # Required
  transport: stdio        # stdio, sse, or websocket (default: stdio)
  optimization: release   # debug or release (default: debug)
```

### Tool Definitions

#### Native Handler

```yaml
tools:
  - type: native
    name: my_tool
    description: "Tool description"
    handler:
      path: handlers::my_handler  # Rust module path
    params:
      input:
        type: string
        required: true
        description: "Input parameter"
```

#### CLI Handler

```yaml
tools:
  - type: cli
    name: git_status
    description: "Check git status"
    command: git
    args: ["status", "--short"]
    timeout_ms: 5000
```

#### HTTP Handler

```yaml
tools:
  - type: http
    name: fetch_user
    description: "Fetch user data"
    endpoint: "https://api.example.com/users/{id}"
    method: GET
    headers:
      Authorization: "Bearer {token}"
    timeout_ms: 10000
```

#### Pipeline Handler

```yaml
tools:
  - type: pipeline
    name: process_workflow
    description: "Multi-step workflow"
    steps:
      - tool: fetch_data
        condition: "input.source == 'api'"
      - tool: transform
      - tool: validate
```

### Resource Definitions

```yaml
resources:
  - uri_template: "file:///{path}"
    handler:
      path: handlers::file_resource
    supports:
      - read
      - write
      - list
```

### Prompt Templates

```yaml
prompts:
  - name: greeting
    description: "Generate a greeting"
    template: "Hello {{name}}, welcome to {{location}}!"
    arguments:
      name:
        type: string
        required: true
      location:
        type: string
        required: true
        default: "pforge"
```

## Types

### TransportType

```rust
pub enum TransportType {
    Stdio,      // Standard input/output
    Sse,        // Server-Sent Events
    WebSocket,  // WebSocket
}
```

### ToolDef

```rust
pub enum ToolDef {
    Native {
        name: String,
        description: String,
        handler: HandlerRef,
        params: HashMap<String, ParamSchema>,
        timeout_ms: Option<u64>,
    },
    Cli {
        name: String,
        description: String,
        command: String,
        args: Vec<String>,
        timeout_ms: Option<u64>,
    },
    Http {
        name: String,
        description: String,
        endpoint: String,
        method: HttpMethod,
        headers: Option<HashMap<String, String>>,
        timeout_ms: Option<u64>,
    },
    Pipeline {
        name: String,
        description: String,
        steps: Vec<PipelineStep>,
    },
}
```

### ParamSchema

```rust
pub struct ParamSchema {
    pub param_type: String,      // e.g., "string", "number", "boolean"
    pub required: bool,
    pub description: Option<String>,
    pub default: Option<serde_json::Value>,
}
```

## Validation

The `validate_config` function checks:

1. **Unique Names**: No duplicate tool, resource, or prompt names
2. **Handler References**: Valid Rust module paths
3. **URI Templates**: Correct URI template syntax
4. **Parameter Schemas**: Valid JSON Schema types
5. **Pipeline Steps**: Referenced tools exist

Example:

```rust
use pforge_config::{ForgeConfig, validate_config};

let config: ForgeConfig = serde_yaml::from_str(yaml)?;

match validate_config(&config) {
    Ok(_) => println!("Configuration is valid!"),
    Err(e) => eprintln!("Validation error: {}", e),
}
```

## Documentation

- [API Documentation](https://docs.rs/pforge-config)
- [Configuration Guide](https://github.com/paiml/pforge/blob/main/docs/USER_GUIDE.md#configuration)
- [Examples](https://github.com/paiml/pforge/tree/main/examples)

## License

MIT
