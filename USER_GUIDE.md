# pforge User Guide

**Version**: 0.1.0
**Last Updated**: 2025-10-03

---

## Table of Contents

1. [Introduction](#introduction)
2. [Installation](#installation)
3. [Quick Start](#quick-start)
4. [Core Concepts](#core-concepts)
5. [CLI Commands](#cli-commands)
6. [Configuration Reference](#configuration-reference)
7. [Handler Types](#handler-types)
8. [Advanced Features](#advanced-features)
9. [Best Practices](#best-practices)
10. [Troubleshooting](#troubleshooting)

---

## Introduction

### What is pforge?

**pforge** is a zero-boilerplate framework for building Model Context Protocol (MCP) servers through declarative YAML configuration. It combines the simplicity of configuration-driven development with the performance and safety of Rust.

**Design Philosophy**: Cargo Lambda simplicity × Flask ergonomics × Rust guarantees

### Key Features

- ✅ **Declarative Configuration**: Define MCP servers in < 10 lines of YAML
- ✅ **Type-Safe**: Automatic JSON Schema generation and validation
- ✅ **Zero Boilerplate**: No manual server setup, routing, or serialization
- ✅ **High Performance**: < 1μs handler dispatch, > 100K req/s throughput
- ✅ **Multi-Transport**: stdio, SSE, WebSocket support
- ✅ **Polyglot**: Native Rust, Python, Go, CLI, HTTP handlers
- ✅ **Production-Ready**: Built-in middleware, state management, error handling

### When to Use pforge

**Use pforge when you want to**:
- Build MCP servers quickly without boilerplate
- Integrate existing CLI tools or HTTP APIs as MCP tools
- Leverage Rust's performance and safety guarantees
- Scale from prototype to production seamlessly

**Don't use pforge if**:
- You need maximum control over every aspect of the server
- Your use case requires custom MCP protocol extensions
- You prefer writing server code manually

---

## Installation

### Prerequisites

- **Rust**: 1.75.0 or later (stable)
- **Cargo**: Comes with Rust
- **Platform**: Linux, macOS, or Windows

### Install from crates.io

```bash
cargo install pforge-cli
```

### Install from source

```bash
git clone https://github.com/paiml/pforge.git
cd pforge
cargo install --path crates/pforge-cli
```

### Verify Installation

```bash
pforge --version
# Output: pforge 0.1.0
```

---

## Quick Start

### Create Your First Server (< 2 minutes)

1. **Create a new project**:
   ```bash
   pforge new my-server
   cd my-server
   ```

2. **Your project structure**:
   ```
   my-server/
   ├── Cargo.toml       # Rust package metadata
   ├── pforge.yaml      # MCP server configuration
   └── src/
       └── handlers/    # Custom handler implementations
           └── hello.rs
   ```

3. **Run the server**:
   ```bash
   pforge serve
   ```

4. **Test with an MCP client**:
   ```bash
   # Example using stdio transport
   echo '{"jsonrpc": "2.0", "method": "tools/list", "id": 1}' | pforge serve
   ```

### Your First pforge.yaml

```yaml
forge:
  name: my-server
  version: 0.1.0
  transport: stdio

tools:
  - type: native
    name: hello
    description: "Say hello to someone"
    handler:
      path: handlers::hello::say_hello
    params:
      name:
        type: string
        required: true
        description: "Name to greet"
```

**That's it!** pforge generates the server, routing, and schema automatically.

---

## Core Concepts

### 1. Declarative Configuration

pforge uses YAML to define your MCP server. No code scaffolding, no manual routing.

**Traditional MCP server** (50+ lines):
```rust
// Manual server setup, routing, schema definitions...
struct HelloTool;
impl TypedTool for HelloTool { /* ... */ }
let server = McpServer::new();
server.register_tool(HelloTool);
// ...
```

**pforge** (5 lines):
```yaml
tools:
  - type: native
    name: hello
    handler:
      path: handlers::hello
```

### 2. Handler Types

pforge supports 4 handler types:

| Type | Use Case | Example |
|------|----------|---------|
| **Native** | Custom Rust logic | Data processing, algorithms |
| **CLI** | Wrap existing commands | `git`, `docker`, `curl` |
| **HTTP** | Proxy to REST APIs | OpenAI, Stripe, GitHub API |
| **Pipeline** | Compose multiple tools | Multi-step workflows |

### 3. Type Safety

All handlers have compile-time type checking and automatic JSON Schema generation:

```rust
#[derive(Deserialize, JsonSchema)]
struct HelloInput {
    name: String,
}

#[derive(Serialize, JsonSchema)]
struct HelloOutput {
    message: String,
}
```

pforge automatically:
- Validates input against schema
- Serializes output to JSON
- Returns typed errors

### 4. Zero-Cost Abstractions

pforge is built on Rust's zero-cost abstraction principle:
- Handler dispatch: < 1μs (faster than function call overhead in Python)
- Memory per tool: < 256 bytes
- Binary size: ~2MB (static linking)

---

## CLI Commands

### `pforge new <name>`

Create a new pforge project.

```bash
pforge new my-awesome-server
```

**Options**:
- `--template <name>`: Use a project template (default: basic)

**Templates**:
- `basic`: Single native handler (default)
- `full`: All handler types demonstrated
- `minimal`: Empty project

### `pforge build`

Build the pforge server binary.

```bash
pforge build           # Debug build
pforge build --release # Optimized build
```

**Options**:
- `--release`: Build with optimizations (recommended for production)
- `--target <triple>`: Cross-compile target

### `pforge serve`

Run the pforge server.

```bash
pforge serve                    # Use pforge.yaml in current directory
pforge serve --config custom.yaml  # Use custom config
```

**Options**:
- `--config <path>`: Path to pforge.yaml (default: ./pforge.yaml)
- `--log-level <level>`: Log level (error, warn, info, debug, trace)
- `--port <port>`: Port for SSE/WebSocket transports

**Example with logging**:
```bash
RUST_LOG=pforge=debug pforge serve
```

### `pforge dev`

Development mode with hot reload.

```bash
pforge dev
```

**Features**:
- Watches pforge.yaml for changes
- Automatically rebuilds and restarts
- Enhanced error messages

---

## Configuration Reference

### Forge Section

Top-level server configuration.

```yaml
forge:
  name: server-name             # Server identifier (required)
  version: 0.1.0                # Semantic version (required)
  transport: stdio              # Transport type (required)
  optimization: release         # Build optimization (optional)

  # Optional features
  state:
    backend: memory             # State backend: memory or sled
    path: /tmp/pforge-state     # Path for persistent state

  middleware:
    - type: logging
      enabled: true
    - type: metrics
      enabled: true
```

**Fields**:
- `name` (string, required): Server name (alphanumeric + hyphens)
- `version` (string, required): Semantic version (e.g., "0.1.0")
- `transport` (enum, required): `stdio` | `sse` | `websocket`
- `optimization` (enum): `debug` | `release` (default: debug)
- `state` (object): State management configuration
- `middleware` (array): Middleware chain configuration

### Tools Section

Define MCP tools.

#### Native Tool

```yaml
tools:
  - type: native
    name: my_tool
    description: "Tool description"
    handler:
      path: handlers::my_module::my_function
    params:
      input_field:
        type: string
        required: true
        description: "Field description"
    timeout_ms: 5000  # Optional timeout
```

**Fields**:
- `type`: `native`
- `name` (string): Tool name (snake_case recommended)
- `description` (string): Human-readable description
- `handler.path` (string): Rust function path (must implement Handler trait)
- `params` (object): Parameter schema (see Parameter Types)
- `timeout_ms` (number): Execution timeout in milliseconds

#### CLI Tool

```yaml
tools:
  - type: cli
    name: git_status
    description: "Check git repository status"
    command: git
    args: ["status", "--short"]
    cwd: /path/to/repo        # Optional working directory
    env:                       # Optional environment variables
      GIT_CONFIG: /path/to/config
    stream: true               # Stream output (optional)
```

**Fields**:
- `type`: `cli`
- `command` (string): Command to execute
- `args` (array): Command arguments
- `cwd` (string): Working directory (default: current)
- `env` (object): Environment variables
- `stream` (boolean): Stream stdout/stderr (default: false)

#### HTTP Tool

```yaml
tools:
  - type: http
    name: fetch_weather
    description: "Get weather data"
    endpoint: https://api.weather.com/v1/current
    method: GET               # GET, POST, PUT, DELETE, PATCH
    auth:                     # Optional authentication
      type: bearer
      token: ${WEATHER_API_KEY}
    headers:                  # Optional custom headers
      User-Agent: pforge/0.1.0
```

**Fields**:
- `type`: `http`
- `endpoint` (string): Full URL
- `method` (enum): HTTP method
- `auth` (object): Authentication config
- `headers` (object): Custom HTTP headers

#### Pipeline Tool

```yaml
tools:
  - type: pipeline
    name: process_workflow
    description: "Multi-step data processing"
    steps:
      - tool: fetch_data
        output_key: raw_data
      - tool: transform_data
        input_from: raw_data
        output_key: clean_data
      - tool: save_data
        input_from: clean_data
```

**Fields**:
- `type`: `pipeline`
- `steps` (array): Sequential tool invocations
  - `tool` (string): Tool name to invoke
  - `input_from` (string): Use output from previous step
  - `output_key` (string): Store output for later steps

### Parameter Types

Supported JSON Schema types for tool parameters:

```yaml
params:
  # String
  name:
    type: string
    required: true
    min_length: 1
    max_length: 100
    pattern: "^[a-zA-Z]+$"
    description: "User name"

  # Integer
  age:
    type: integer
    required: true
    min: 0
    max: 150
    description: "User age"

  # Float
  price:
    type: float
    required: true
    min: 0.0
    description: "Product price"

  # Boolean
  active:
    type: boolean
    required: false
    default: true
    description: "Is active"

  # Array
  tags:
    type: array
    required: false
    description: "Tags list"

  # Object
  metadata:
    type: object
    required: false
    description: "Additional metadata"
```

### Resources Section

Define MCP resources for file/data access.

```yaml
resources:
  - uri_template: "file:///{path}"
    handler:
      path: handlers::files::read_file
    supports:
      - read
      - write
```

### Prompts Section

Define MCP prompt templates.

```yaml
prompts:
  - name: code_review
    description: "Review code for issues"
    template: "Review the following code:\n\n{{code}}\n\nFocus on: {{focus}}"
    arguments:
      code:
        type: string
        required: true
      focus:
        type: string
        required: false
        default: "bugs and security"
```

---

## Handler Types

### Native Handlers

Native handlers are Rust functions that implement the `Handler` trait.

**Step 1**: Define input/output types

```rust
use serde::{Deserialize, Serialize};
use schemars::JsonSchema;

#[derive(Debug, Deserialize, JsonSchema)]
struct GreetInput {
    name: String,
    #[serde(default = "default_greeting")]
    greeting: String,
}

fn default_greeting() -> String {
    "Hello".to_string()
}

#[derive(Debug, Serialize, JsonSchema)]
struct GreetOutput {
    message: String,
}
```

**Step 2**: Implement Handler trait

```rust
use pforge_runtime::{Handler, Error};
use async_trait::async_trait;

struct GreetHandler;

#[async_trait]
impl Handler for GreetHandler {
    type Input = GreetInput;
    type Output = GreetOutput;
    type Error = Error;

    async fn handle(&self, input: Self::Input) -> Result<Self::Output, Self::Error> {
        Ok(GreetOutput {
            message: format!("{}, {}!", input.greeting, input.name),
        })
    }
}
```

**Step 3**: Configure in pforge.yaml

```yaml
tools:
  - type: native
    name: greet
    handler:
      path: handlers::greet::GreetHandler
    params:
      name:
        type: string
        required: true
      greeting:
        type: string
        required: false
```

### CLI Handlers

Wrap existing command-line tools as MCP tools.

```yaml
tools:
  # Simple command
  - type: cli
    name: date
    description: "Get current date and time"
    command: date
    args: ["+%Y-%m-%d %H:%M:%S"]

  # Command with environment
  - type: cli
    name: docker_ps
    description: "List Docker containers"
    command: docker
    args: ["ps", "--format", "{{.Names}}"]
    env:
      DOCKER_HOST: unix:///var/run/docker.sock

  # Streaming command
  - type: cli
    name: tail_logs
    description: "Stream log file"
    command: tail
    args: ["-f", "/var/log/app.log"]
    stream: true
```

**Use Cases**:
- System commands (`ls`, `grep`, `find`)
- Git operations (`git status`, `git log`)
- Docker management (`docker ps`, `docker logs`)
- Cloud CLI (`aws s3 ls`, `gcloud compute list`)

### HTTP Handlers

Proxy HTTP APIs as MCP tools.

```yaml
tools:
  # GET request
  - type: http
    name: get_user
    description: "Fetch user profile"
    endpoint: https://api.example.com/users/{user_id}
    method: GET
    headers:
      Accept: application/json

  # POST with authentication
  - type: http
    name: create_issue
    description: "Create GitHub issue"
    endpoint: https://api.github.com/repos/{owner}/{repo}/issues
    method: POST
    auth:
      type: bearer
      token: ${GITHUB_TOKEN}
    headers:
      Accept: application/vnd.github.v3+json
```

**Supported Auth Types**:
- `bearer`: Bearer token (OAuth 2.0)
- `basic`: Username + password
- `apikey`: API key in header

### Pipeline Handlers

Compose multiple tools into workflows.

```yaml
tools:
  # Step 1: Fetch data
  - type: http
    name: fetch_weather
    endpoint: https://api.weather.com/current
    method: GET

  # Step 2: Transform data
  - type: native
    name: parse_weather
    handler:
      path: handlers::weather::parse

  # Step 3: Send notification
  - type: http
    name: send_alert
    endpoint: https://api.slack.com/webhooks/xxx
    method: POST

  # Pipeline combining all steps
  - type: pipeline
    name: weather_alert
    description: "Fetch weather and send alert"
    steps:
      - tool: fetch_weather
        output_key: raw_weather
      - tool: parse_weather
        input_from: raw_weather
        output_key: parsed_weather
      - tool: send_alert
        input_from: parsed_weather
```

---

## Advanced Features

### State Management

Enable persistent state across tool invocations.

```yaml
forge:
  state:
    backend: sled           # or "memory"
    path: /var/lib/pforge
    ttl_seconds: 3600       # Optional: auto-expire keys

tools:
  - type: native
    name: set_value
    handler:
      path: handlers::state::set

  - type: native
    name: get_value
    handler:
      path: handlers::state::get
```

**Handler with state**:
```rust
use pforge_runtime::StateManager;

async fn handle(&self, input: Input) -> Result<Output> {
    let state = StateManager::global();

    // Set value
    state.set("key", "value").await?;

    // Get value
    let value: String = state.get("key").await?;

    Ok(Output { value })
}
```

### Middleware

Add cross-cutting concerns to all tools.

```yaml
forge:
  middleware:
    - type: logging
      enabled: true
      level: info

    - type: metrics
      enabled: true
      export_prometheus: true

    - type: recovery
      enabled: true
      max_retries: 3
```

**Available Middleware**:
- `logging`: Request/response logging
- `metrics`: Prometheus metrics export
- `recovery`: Auto-retry on transient failures
- `timeout`: Global timeout enforcement
- `ratelimit`: Request rate limiting

### Multi-Transport

Support different client connection types.

**stdio** (default):
```yaml
forge:
  transport: stdio
```

**SSE (Server-Sent Events)**:
```yaml
forge:
  transport: sse
  port: 8080
```

**WebSocket**:
```yaml
forge:
  transport: websocket
  port: 9000
  max_connections: 100
```

---

## Best Practices

### 1. Tool Naming

**DO**:
- Use `snake_case`: `get_user`, `fetch_data`
- Be descriptive: `calculate_tax` not `calc`
- Use verbs: `create_`, `get_`, `delete_`

**DON'T**:
- Use camelCase: `getUser`
- Use abbreviations: `calc`, `usr`
- Use numbers: `tool1`, `tool2`

### 2. Error Handling

**Return meaningful errors**:
```rust
if input.age < 0 {
    return Err(Error::Validation("Age cannot be negative".to_string()));
}
```

**Use Result propagation**:
```rust
let data = fetch_data().await?;  // Propagate errors
```

### 3. Testing

**Unit test handlers**:
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_greet_handler() {
        let handler = GreetHandler;
        let input = GreetInput {
            name: "Alice".to_string(),
            greeting: "Hi".to_string(),
        };

        let output = handler.handle(input).await.unwrap();
        assert_eq!(output.message, "Hi, Alice!");
    }
}
```

### 4. Performance

**Optimize hot paths**:
- Use `&str` instead of `String` when possible
- Avoid cloning large data structures
- Use `tokio::spawn` for parallel operations

**Monitor metrics**:
```bash
# Enable metrics
PFORGE_METRICS=1 pforge serve

# View metrics
curl http://localhost:9090/metrics
```

---

## Troubleshooting

### Common Issues

#### "Handler not found"

**Error**:
```
Error: Handler 'handlers::my_tool::MyHandler' not found
```

**Solution**:
- Verify handler path in pforge.yaml matches Rust module structure
- Ensure handler struct is public (`pub struct MyHandler`)
- Check `mod handlers;` in main.rs

#### "Parameter validation failed"

**Error**:
```
Error: Invalid input: field 'name' is required
```

**Solution**:
- Check JSON input matches parameter schema
- Verify required fields are present
- Check type constraints (min, max, pattern)

#### "Connection refused"

**Error**:
```
Error: Connection refused (port 8080)
```

**Solution**:
- Verify port is available: `lsof -i :8080`
- Check firewall settings
- Use `0.0.0.0` to bind all interfaces

### Debug Mode

Enable verbose logging:

```bash
RUST_LOG=pforge=trace pforge serve
```

**Log Levels**:
- `error`: Only errors
- `warn`: Warnings and errors
- `info`: General information
- `debug`: Detailed debugging
- `trace`: Very detailed (includes all requests)

### Getting Help

- **Documentation**: https://docs.pforge.dev
- **GitHub Issues**: https://github.com/paiml/pforge/issues
- **Discord**: https://discord.gg/pforge

---

## Next Steps

- Read [Architecture Documentation](./ARCHITECTURE.md)
- Explore [Examples](./examples/)
- Check [API Reference](https://docs.rs/pforge-runtime)
- Join the [Community](https://discord.gg/pforge)

---

**Last Updated**: 2025-10-03
**pforge Version**: 0.1.0
