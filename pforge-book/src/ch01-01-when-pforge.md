# Chapter 1.1: When to Use pforge

This chapter provides detailed guidance on when pforge is the right choice for your MCP server project.

## The pforge Sweet Spot

pforge is designed for **standard MCP server patterns** with **minimal boilerplate**. If you're building a server that fits common use cases, pforge will save you significant time and enforce best practices automatically.

## Use pforge When...

### 1. **You're Wrapping Existing Tools**

pforge excels at wrapping CLIs, HTTP APIs, and simple logic into MCP tools.

**Examples:**

```yaml
# Wrap Git commands
tools:
  - type: cli
    name: git_status
    description: "Get git repository status"
    command: git
    args: ["status", "--porcelain"]

  - type: cli
    name: git_commit
    description: "Commit changes"
    command: git
    args: ["commit", "-m", "{{message}}"]
```

```yaml
# Wrap HTTP APIs
tools:
  - type: http
    name: github_create_issue
    description: "Create a GitHub issue"
    endpoint: "https://api.github.com/repos/{{owner}}/{{repo}}/issues"
    method: POST
    headers:
      Authorization: "Bearer {{GITHUB_TOKEN}}"
```

**Why pforge wins here:**
- No need to write subprocess handling code
- No need to write HTTP client code
- Built-in error handling and retries
- Configuration changes don't require recompilation

### 2. **You Want Fast Iteration**

With pforge, changing your server is as simple as editing YAML:

```yaml
# Before: tool with 30s timeout
tools:
  - type: native
    name: slow_operation
    timeout_ms: 30000
```

```yaml
# After: increased to 60s - no code changes, no recompile
tools:
  - type: native
    name: slow_operation
    timeout_ms: 60000
```

**Development cycle:**
- **pmcp**: Edit code → Recompile → Test (2-5 minutes)
- **pforge**: Edit YAML → Restart (5-10 seconds)

### 3. **You Need Built-in Quality Gates**

pforge comes with PMAT integration and enforced quality standards:

```bash
# Automatically enforced pre-commit
$ git commit -m "Add new tool"

Running quality gates:
✓ cargo fmt --check
✓ cargo clippy -- -D warnings
✓ cargo test --all
✓ coverage ≥ 80%
✓ complexity ≤ 20
✓ no SATD comments
✓ TDG ≥ 0.75

Commit allowed ✓
```

**What you get:**
- Zero `unwrap()` in production code
- No functions with cyclomatic complexity > 20
- 80%+ test coverage enforced
- Mutation testing integrated
- Automatic code quality checks

### 4. **You're Building Standard CRUD Operations**

pforge's handler types cover most common patterns:

```yaml
tools:
  # Native handlers for business logic
  - type: native
    name: validate_user
    handler:
      path: handlers::validate_user
    params:
      email: { type: string, required: true }

  # CLI handlers for external tools
  - type: cli
    name: run_tests
    command: pytest
    args: ["tests/"]

  # HTTP handlers for API proxies
  - type: http
    name: fetch_user_data
    endpoint: "https://api.example.com/users/{{user_id}}"
    method: GET

  # Pipeline handlers for composition
  - type: pipeline
    name: validate_and_fetch
    steps:
      - tool: validate_user
        output: validation_result
      - tool: fetch_user_data
        condition: "{{validation_result.valid}}"
```

### 5. **You Want Sub-Microsecond Tool Dispatch**

pforge uses compile-time code generation with perfect hashing:

```
Benchmark: Tool Dispatch Latency
================================
pmcp (HashMap):     8.2μs ± 0.3μs
pforge (perfect hash): 0.7μs ± 0.1μs

Speedup: 11.7x faster
```

**How it works:**
- YAML configuration → Rust code generation
- Perfect hash function computed at compile time
- Zero dynamic lookups
- Inlined handler calls

### 6. **You're New to Rust**

pforge has a gentler learning curve:

**What you need to know:**

**Minimal:**
- YAML syntax (everyone knows this)
- Basic struct definitions for native handlers
- `async/await` for async handlers

**You don't need to know:**
- pmcp API details
- MCP protocol internals
- Transport layer implementation
- JSON-RPC message handling

**Example - Complete pforge server:**

```yaml
# forge.yaml - 10 lines
forge:
  name: my-server
  version: 0.1.0

tools:
  - type: native
    name: greet
    handler:
      path: handlers::greet
    params:
      name: { type: string, required: true }
```

```rust
// src/handlers.rs - 20 lines
use pforge_runtime::{Handler, Result, Error};
use serde::{Deserialize, Serialize};
use schemars::JsonSchema;

#[derive(Debug, Deserialize, JsonSchema)]
pub struct GreetInput {
    name: String,
}

#[derive(Debug, Serialize, JsonSchema)]
pub struct GreetOutput {
    message: String,
}

pub struct GreetHandler;

#[async_trait::async_trait]
impl Handler for GreetHandler {
    type Input = GreetInput;
    type Output = GreetOutput;
    type Error = Error;

    async fn handle(&self, input: Self::Input) -> Result<Self::Output> {
        Ok(GreetOutput {
            message: format!("Hello, {}!", input.name)
        })
    }
}

pub use GreetHandler as greet;
```

```bash
# Run it
$ pforge serve
```

### 7. **You Need Multi-Tool Pipelines**

pforge supports declarative tool composition:

```yaml
tools:
  - type: pipeline
    name: analyze_and_report
    description: "Analyze code and generate report"
    steps:
      - tool: run_linter
        output: lint_results

      - tool: run_tests
        output: test_results

      - tool: generate_report
        condition: "{{lint_results.passed}} && {{test_results.passed}}"
        inputs:
          lint: "{{lint_results}}"
          tests: "{{test_results}}"

      - tool: send_notification
        condition: "{{lint_results.passed}}"
        on_error: continue
```

**Benefits:**
- Declarative composition
- Conditional execution
- Error handling strategies
- Output passing between steps

### 8. **You Want State Management Out of the Box**

pforge provides persistent state with zero configuration:

```yaml
state:
  backend: sled
  path: /tmp/my-server-state
  cache_size: 1000
```

```rust
// In your handler
async fn handle(&self, input: Self::Input) -> Result<Self::Output> {
    // Get state
    let counter = self.state
        .get("counter")
        .await?
        .and_then(|bytes| String::from_utf8(bytes).ok())
        .and_then(|s| s.parse::<u64>().ok())
        .unwrap_or(0);

    // Increment
    let new_counter = counter + 1;

    // Save state
    self.state
        .set("counter", new_counter.to_string().into_bytes(), None)
        .await?;

    Ok(MyOutput { counter: new_counter })
}
```

**State backends:**
- **Sled**: Persistent embedded database (default)
- **Memory**: In-memory with DashMap (testing)
- **Redis**: Distributed state (future)

### 9. **You Want Enforced Best Practices**

pforge enforces patterns from day one:

**Error handling:**
```rust
// ❌ Not allowed in pforge
let value = map.get("key").unwrap();  // Compile error!

// ✅ Required pattern
let value = map.get("key")
    .ok_or_else(|| Error::Handler("Key not found".into()))?;
```

**Async by default:**
```rust
// All handlers are async - no blocking allowed
#[async_trait::async_trait]
impl Handler for MyHandler {
    async fn handle(&self, input: Self::Input) -> Result<Self::Output> {
        // Non-blocking I/O enforced
        let data = tokio::fs::read_to_string("data.txt").await?;
        Ok(MyOutput { data })
    }
}
```

**Type safety:**
```yaml
params:
  age: { type: integer, required: true }  # Compile-time checked
```

```rust
pub struct Input {
    age: i64,  // Not Option<i64> - required enforced at compile time
}
```

## Real-World Use Cases

### Case Study 1: PMAT Code Analysis Server

**Challenge:** Wrap the PMAT CLI tool as an MCP server

**Solution:**
```yaml
tools:
  - type: cli
    name: analyze_complexity
    command: pmat
    args: ["analyze", "complexity", "--file", "{{file_path}}"]

  - type: cli
    name: analyze_satd
    command: pmat
    args: ["analyze", "satd", "--file", "{{file_path}}"]
```

**Results:**
- 10 lines of YAML (vs ~200 lines of Rust with pmcp)
- No subprocess handling code
- Automatic error handling
- Built-in retry logic

### Case Study 2: GitHub API Proxy

**Challenge:** Expose GitHub API operations as MCP tools

**Solution:**
```yaml
tools:
  - type: http
    name: create_issue
    endpoint: "https://api.github.com/repos/{{owner}}/{{repo}}/issues"
    method: POST
    headers:
      Authorization: "Bearer {{GITHUB_TOKEN}}"
      Accept: "application/vnd.github.v3+json"

  - type: http
    name: list_pull_requests
    endpoint: "https://api.github.com/repos/{{owner}}/{{repo}}/pulls"
    method: GET
```

**Results:**
- No HTTP client code
- Automatic connection pooling (reqwest)
- Built-in authentication
- Retry on network errors

### Case Study 3: Data Pipeline Orchestrator

**Challenge:** Chain multiple data processing tools

**Solution:**
```yaml
tools:
  - type: pipeline
    name: process_data
    steps:
      - tool: extract_data
        output: raw_data
      - tool: transform_data
        inputs:
          data: "{{raw_data}}"
        output: transformed
      - tool: load_data
        inputs:
          data: "{{transformed}}"
```

**Results:**
- Declarative pipeline definition
- Automatic error recovery
- Step-by-step logging
- Conditional execution

## Performance Characteristics

| Metric | pforge | Notes |
|--------|--------|-------|
| **Tool Dispatch** | <1μs | Perfect hash, compile-time optimized |
| **Cold Start** | <100ms | Code generation adds startup time |
| **Memory/Tool** | <256B | Minimal overhead per handler |
| **Throughput** | >100K req/s | Sequential execution |
| **Config Reload** | ~10ms | Hot reload without restart |

## When pforge Might NOT Be the Best Choice

pforge is **not** ideal when:

1. **You need custom MCP protocol extensions**
   - pforge uses standard MCP features only
   - Drop down to pmcp for custom protocol work

2. **You need complex stateful logic**
   - Example: Database query planner with transaction management
   - pmcp gives you full control

3. **You need custom transport implementations**
   - pforge supports stdio/SSE/WebSocket
   - Custom transports require pmcp

4. **You're building a library/SDK**
   - pforge is for applications, not libraries
   - Use pmcp for reusable components

5. **You need WebAssembly compilation**
   - pforge targets native binaries
   - pmcp can compile to WASM

See [Chapter 1.2: When to Use pmcp](ch01-02-when-pmcp.md) for these cases.

## Migration Path

Start with pforge, migrate to pmcp when needed:

```rust
// Start with pforge handlers
pub struct MyHandler;

#[async_trait::async_trait]
impl pforge_runtime::Handler for MyHandler {
    // ... pforge handler impl
}

// Later, use same handler in pmcp
use pmcp::ServerBuilder;

#[tokio::main]
async fn main() -> Result<()> {
    let server = ServerBuilder::new()
        .name("my-server")
        .tool_typed("my_tool", |input: MyInput, _extra| {
            Box::pin(async move {
                let handler = MyHandler;
                let output = handler.handle(input).await?;
                Ok(serde_json::to_value(output)?)
            })
        })
        .build()?;

    server.run_stdio().await
}
```

**Key insight:** pforge handlers are compatible with pmcp!

## Summary

Use **pforge** when you want:

✅ Minimal boilerplate
✅ Fast iteration (YAML changes)
✅ Built-in quality gates
✅ CLI/HTTP/Pipeline handlers
✅ Sub-microsecond dispatch
✅ Gentle learning curve
✅ State management included
✅ Enforced best practices

Use **pmcp** when you need:

❌ Custom protocol extensions
❌ Complex stateful logic
❌ Custom transports
❌ Library/SDK development
❌ WebAssembly compilation

**Not sure?** Start with pforge. You can always drop down to pmcp later.

---

**Next:** [When to Use pmcp Directly](ch01-02-when-pmcp.md)
