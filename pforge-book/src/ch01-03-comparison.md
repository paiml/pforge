# Chapter 1.3: Side-by-Side Comparison

This chapter provides a comprehensive feature-by-feature comparison of **pforge** and **pmcp** to help you choose the right tool for your project.

## Quick Reference Matrix

| Feature | pforge | pmcp | Winner |
|---------|--------|------|--------|
| **Development Model** | Declarative YAML | Programmatic Rust | Depends |
| **Code Required** | ~10 lines YAML + handlers | ~100-500 lines Rust | pforge |
| **Learning Curve** | Low (YAML + basic Rust) | Medium (full Rust + MCP) | pforge |
| **Type Safety** | Compile-time (codegen) | Compile-time (native) | Tie |
| **Tool Dispatch** | <1μs (perfect hash) | <10μs (HashMap) | pforge |
| **Cold Start** | <100ms | <50ms | pmcp |
| **Memory/Tool** | <256B | <512B | pforge |
| **Throughput** | >100K req/s | >50K req/s | pforge |
| **Binary Size** | ~5-10MB | ~2-3MB | pmcp |
| **Flexibility** | 4 handler types | Unlimited | pmcp |
| **Quality Gates** | Built-in (PMAT) | Manual | pforge |
| **Iteration Speed** | Fast (YAML edit) | Medium (recompile) | pforge |
| **Custom Protocols** | Not supported | Full control | pmcp |
| **WebAssembly** | Not supported | Supported | pmcp |
| **State Management** | Built-in | Manual | pforge |
| **CLI Wrappers** | Built-in | Manual | pforge |
| **HTTP Proxies** | Built-in | Manual | pforge |
| **Pipelines** | Built-in | Manual | pforge |
| **Middleware** | Built-in | Manual | pforge |
| **Circuit Breakers** | Built-in | Manual | pforge |
| **Library Development** | Not ideal | Perfect | pmcp |
| **Custom Transports** | Not supported | Full control | pmcp |

## Detailed Comparison

### 1. Configuration Approach

#### pforge: Declarative YAML

```yaml
# forge.yaml
forge:
  name: calculator-server
  version: 1.0.0
  transport: stdio
  optimization: release

tools:
  - type: native
    name: calculate
    description: "Perform arithmetic operations"
    handler:
      path: handlers::calculate
    params:
      operation: { type: string, required: true }
      a: { type: float, required: true }
      b: { type: float, required: true }
    timeout_ms: 5000
```

**Pros:**
- Declarative, self-documenting
- Easy to read and modify
- No recompilation for config changes
- Version control friendly
- Non-programmers can understand

**Cons:**
- Limited to supported features
- Can't express complex logic
- Requires code generation step

#### pmcp: Programmatic Rust

```rust
use pmcp::{ServerBuilder, TypedTool};
use serde::{Deserialize, Serialize};
use schemars::JsonSchema;

#[derive(Debug, Deserialize, JsonSchema)]
struct CalculateInput {
    operation: String,
    a: f64,
    b: f64,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let server = ServerBuilder::new()
        .name("calculator-server")
        .version("1.0.0")
        .tool_typed("calculate", |input: CalculateInput, _extra| {
            Box::pin(async move {
                let result = match input.operation.as_str() {
                    "add" => input.a + input.b,
                    "subtract" => input.a - input.b,
                    "multiply" => input.a * input.b,
                    "divide" => {
                        if input.b == 0.0 {
                            return Err(pmcp::Error::Validation(
                                "Division by zero".into()
                            ));
                        }
                        input.a / input.b
                    }
                    _ => return Err(pmcp::Error::Validation(
                        "Unknown operation".into()
                    )),
                };
                Ok(serde_json::json!({ "result": result }))
            })
        })
        .build()?;

    server.run_stdio().await?;
    Ok(())
}
```

**Pros:**
- Unlimited flexibility
- Express complex logic directly
- Full Rust type system
- Better IDE support
- No code generation

**Cons:**
- More boilerplate
- Steeper learning curve
- Requires recompilation
- More verbose

### 2. Handler Types

#### pforge: Four Built-in Types

```yaml
tools:
  # 1. Native handlers - Pure Rust logic
  - type: native
    name: validate_email
    handler:
      path: handlers::validate_email
    params:
      email: { type: string, required: true }

  # 2. CLI handlers - Subprocess wrappers
  - type: cli
    name: run_git_status
    command: git
    args: ["status", "--porcelain"]
    cwd: /path/to/repo
    stream: true

  # 3. HTTP handlers - API proxies
  - type: http
    name: create_github_issue
    endpoint: "https://api.github.com/repos/{{owner}}/{{repo}}/issues"
    method: POST
    headers:
      Authorization: "Bearer {{GITHUB_TOKEN}}"

  # 4. Pipeline handlers - Tool composition
  - type: pipeline
    name: validate_and_save
    steps:
      - tool: validate_email
        output: validation
      - tool: save_to_db
        condition: "{{validation.valid}}"
```

**Coverage:** ~80% of common use cases

#### pmcp: Unlimited Custom Handlers

```rust
// Any Rust code you can imagine
server
    .tool_typed("custom", |input, _| {
        Box::pin(async move {
            // Complex database transactions
            let mut tx = pool.begin().await?;

            // Call external services
            let response = reqwest::get("https://api.example.com").await?;

            // Complex business logic
            let result = process_with_ml_model(input).await?;

            tx.commit().await?;
            Ok(serde_json::to_value(result)?)
        })
    })
    .tool_raw("zero_copy", |bytes, _| {
        Box::pin(async move {
            // Zero-copy byte processing
            process_in_place(bytes)
        })
    })
    .custom_method("custom/protocol", |params| {
        Box::pin(async move {
            // Custom protocol extension
            Ok(custom_handler(params).await?)
        })
    })
```

**Coverage:** 100% - anything Rust can do

### 3. Performance Comparison

#### Tool Dispatch Latency

```
pforge (perfect hash):     0.7μs ± 0.1μs
pmcp (HashMap):            8.2μs ± 0.3μs

Speedup: 11.7x faster
```

**Why pforge is faster:**
- Compile-time perfect hash function (FKS algorithm)
- Zero dynamic lookups
- Inlined handler calls
- No runtime registry traversal

**pmcp overhead:**
- HashMap lookup: ~5-10ns
- Dynamic dispatch: ~2-5μs
- Type erasure overhead: ~1-3μs

#### Cold Start Time

```
pforge:  95ms  (includes codegen cache load)
pmcp:    42ms  (minimal binary)

Startup: pmcp 2.3x faster
```

**Why pmcp is faster:**
- No code generation loading
- Smaller binary
- Simpler initialization

**pforge overhead:**
- Load generated code: ~40ms
- Initialize registry: ~15ms
- State backend init: ~10ms

#### Throughput Benchmarks

```
Sequential Execution (1 core):
pforge:  105,000 req/s
pmcp:     68,000 req/s

Concurrent Execution (8 cores):
pforge:  520,000 req/s
pmcp:    310,000 req/s

Throughput: pforge 1.5-1.7x faster
```

**Why pforge scales better:**
- Lock-free perfect hash
- Pre-allocated handler slots
- Optimized middleware chain

#### Memory Usage

```
Per-tool overhead:
pforge:  ~200B  (registry entry + metadata)
pmcp:    ~450B  (boxed closure + type info)

10-tool server:
pforge:  ~2MB   (including state backend)
pmcp:    ~1.5MB (minimal runtime)
```

### 4. Development Workflow

#### pforge: Edit → Restart

```bash
# 1. Edit configuration
vim forge.yaml

# 2. Restart server (no recompile needed)
pforge serve

# Total time: ~5 seconds
```

**Iteration cycle:**
- YAML changes: 0s compile time
- Handler changes: 2-10s compile time
- Config validation: instant feedback
- Hot reload: supported (experimental)

#### pmcp: Edit → Compile → Run

```bash
# 1. Edit code
vim src/main.rs

# 2. Recompile
cargo build --release

# 3. Run
./target/release/my-server

# Total time: 30-120 seconds
```

**Iteration cycle:**
- Any change: full recompile
- Release build: 30-120s
- Debug build: 5-20s
- Incremental: helps but still slower

### 5. Quality & Testing

#### pforge: Built-in Quality Gates

```yaml
# Quality gates enforced automatically
quality:
  pre_commit:
    - cargo fmt --check
    - cargo clippy -- -D warnings
    - cargo test --all
    - cargo tarpaulin --out Json  # ≥80% coverage
    - pmat analyze complexity --max 20
    - pmat analyze satd --max 0
    - pmat analyze tdg --min 0.75

  ci:
    - cargo mutants  # ≥90% mutation kill rate
```

**Enforced standards:**
- No `unwrap()` in production code
- No `panic!()` in production code
- Cyclomatic complexity ≤ 20
- Test coverage ≥ 80%
- Technical Debt Grade ≥ 0.75
- Zero SATD comments

**Testing:**
```bash
# Property-based tests generated automatically
pforge test --property

# Mutation testing integrated
pforge test --mutation

# Benchmark regression checks
pforge bench --check
```

#### pmcp: Manual Quality Setup

```rust
// You implement quality checks yourself
#[cfg(test)]
mod tests {
    // You write all tests manually

    #[test]
    fn test_calculator() {
        // Manual test implementation
    }

    // Property tests if you add proptest
    proptest! {
        #[test]
        fn prop_test(a: f64, b: f64) {
            // Manual property test
        }
    }
}
```

**Standards:**
- You decide what to enforce
- You configure CI/CD
- You set up coverage tools
- You integrate quality checks

### 6. State Management

#### pforge: Built-in State

```yaml
# Automatic state management
state:
  backend: sled       # or "memory" for testing
  path: /tmp/state
  cache_size: 1000
  ttl: 3600
```

```rust
// Use in handlers
async fn handle(&self, input: Input) -> Result<Output> {
    // Get state
    let counter = self.state
        .get("counter").await?
        .unwrap_or(0);

    // Update state
    self.state
        .set("counter", counter + 1, None).await?;

    Ok(Output { count: counter + 1 })
}
```

**Backends:**
- **Sled**: Persistent embedded DB (default)
- **Memory**: In-memory DashMap (testing)
- **Redis**: Distributed state (future)

#### pmcp: Manual State Implementation

```rust
use std::sync::Arc;
use tokio::sync::RwLock;

struct AppState {
    data: Arc<RwLock<HashMap<String, Value>>>,
    db: PgPool,
    cache: Cache,
}

#[tokio::main]
async fn main() -> Result<()> {
    let state = Arc::new(AppState {
        data: Arc::new(RwLock::new(HashMap::new())),
        db: create_pool().await?,
        cache: Cache::new(),
    });

    let server = ServerBuilder::new()
        .name("stateful-server")
        .tool_typed("get_data", {
            let state = state.clone();
            move |input: GetInput, _| {
                let state = state.clone();
                Box::pin(async move {
                    let data = state.data.read().await;
                    Ok(data.get(&input.key).cloned())
                })
            }
        })
        .build()?;

    server.run_stdio().await
}
```

**Flexibility:**
- Any state backend you want
- Custom synchronization
- Complex state patterns
- Full control over lifecycle

### 7. Error Handling

#### pforge: Standardized Errors

```rust
use pforge_runtime::{Error, Result};

// Standardized error types
pub enum Error {
    Handler(String),
    Validation(String),
    Timeout,
    ToolNotFound(String),
    InvalidConfig(String),
}

// Automatic error conversion
async fn handle(&self, input: Input) -> Result<Output> {
    let value = input.value
        .ok_or_else(|| Error::Validation("Missing value".into()))?;

    // All errors converted to JSON-RPC format
    Ok(Output { result: value * 2 })
}
```

**Features:**
- Consistent error format
- Automatic JSON-RPC conversion
- Stack trace preservation
- Error tracking built-in

#### pmcp: Custom Error Handling

```rust
use pmcp::Error as McpError;
use thiserror::Error;

// Custom error types
#[derive(Debug, Error)]
pub enum MyError {
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("API error: {0}")]
    Api(#[from] reqwest::Error),

    #[error("Custom error: {0}")]
    Custom(String),
}

// Manual conversion to MCP errors
impl From<MyError> for McpError {
    fn from(err: MyError) -> Self {
        McpError::Handler(err.to_string())
    }
}
```

**Flexibility:**
- Define your own error types
- Custom error conversion
- Error context preservation
- Full control over error responses

### 8. Use Case Fit Matrix

| Use Case | pforge Fit | pmcp Fit | Recommendation |
|----------|------------|----------|----------------|
| **CLI tool wrapper** | ⭐⭐⭐⭐⭐ | ⭐⭐ | pforge |
| **HTTP API proxy** | ⭐⭐⭐⭐⭐ | ⭐⭐ | pforge |
| **Simple CRUD** | ⭐⭐⭐⭐⭐ | ⭐⭐⭐ | pforge |
| **Tool pipelines** | ⭐⭐⭐⭐⭐ | ⭐⭐ | pforge |
| **Database server** | ⭐⭐ | ⭐⭐⭐⭐⭐ | pmcp |
| **Real-time collab** | ⭐ | ⭐⭐⭐⭐⭐ | pmcp |
| **Custom protocols** | ❌ | ⭐⭐⭐⭐⭐ | pmcp |
| **WebAssembly** | ❌ | ⭐⭐⭐⭐⭐ | pmcp |
| **Library/SDK** | ❌ | ⭐⭐⭐⭐⭐ | pmcp |
| **Rapid prototyping** | ⭐⭐⭐⭐⭐ | ⭐⭐⭐ | pforge |
| **Production CRUD** | ⭐⭐⭐⭐⭐ | ⭐⭐⭐⭐ | pforge |
| **Complex state** | ⭐⭐ | ⭐⭐⭐⭐⭐ | pmcp |
| **Multi-server** | ⭐ | ⭐⭐⭐⭐⭐ | pmcp |

### 9. Code Size Comparison

For a typical 10-tool MCP server:

#### pforge

```
forge.yaml:                  80 lines
src/handlers.rs:            200 lines
tests/:                     150 lines
--------------------------------
Total:                      430 lines

Generated code:            ~2000 lines (hidden)
```

#### pmcp

```
src/main.rs:                150 lines
src/handlers/:              400 lines
src/state.rs:               100 lines
src/errors.rs:               50 lines
tests/:                     200 lines
--------------------------------
Total:                      900 lines
```

**Code reduction: 52% with pforge**

### 10. Learning Curve

#### pforge

**What you need to know:**
- ✅ YAML syntax (30 minutes)
- ✅ Basic Rust structs (1 hour)
- ✅ `async/await` basics (1 hour)
- ✅ Result/Option types (1 hour)

**What you don't need to know:**
- ❌ MCP protocol details
- ❌ JSON-RPC internals
- ❌ pmcp API
- ❌ Transport implementation

**Time to productivity:** 3-4 hours

#### pmcp

**What you need to know:**
- ✅ Rust fundamentals (10-20 hours)
- ✅ Async programming (5 hours)
- ✅ MCP protocol (2 hours)
- ✅ pmcp API (2 hours)
- ✅ Error handling patterns (2 hours)

**What you don't need to know:**
- ❌ Nothing - full control requires full knowledge

**Time to productivity:** 20-30 hours

## Migration Strategies

### pmcp → pforge

```rust
// Before (pmcp)
ServerBuilder::new()
    .tool_typed("calculate", |input: CalcInput, _| {
        Box::pin(async move {
            Ok(serde_json::json!({ "result": input.a + input.b }))
        })
    })

// After (pforge)
// 1. Extract to handler
pub struct CalculateHandler;

#[async_trait::async_trait]
impl Handler for CalculateHandler {
    type Input = CalcInput;
    type Output = CalcOutput;

    async fn handle(&self, input: Input) -> Result<Output> {
        Ok(CalcOutput { result: input.a + input.b })
    }
}

// 2. Add to forge.yaml
// tools:
//   - type: native
//     name: calculate
//     handler:
//       path: handlers::CalculateHandler
```

### pforge → pmcp

```rust
// Reuse pforge handlers in pmcp!
use pforge_runtime::Handler;

// pforge handler (no changes needed)
pub struct MyHandler;

#[async_trait::async_trait]
impl Handler for MyHandler {
    // ... existing implementation
}

// Use in pmcp server
#[tokio::main]
async fn main() -> Result<()> {
    let server = ServerBuilder::new()
        .name("hybrid-server")
        .tool_typed("my_tool", |input: MyInput, _| {
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

## Decision Matrix

### Choose pforge if:

✅ You want minimal boilerplate
✅ You need fast iteration (YAML changes)
✅ You want built-in quality gates
✅ You're building standard MCP patterns
✅ You need CLI/HTTP wrappers
✅ You want sub-microsecond dispatch
✅ You're new to Rust
✅ You need state management out-of-the-box

### Choose pmcp if:

✅ You need custom protocol extensions
✅ You need complex stateful logic
✅ You need custom transports
✅ You're building a library/SDK
✅ You need WebAssembly support
✅ You want complete control
✅ You're building multi-server orchestration
✅ You need runtime configuration

### Use both if:

✅ You want pforge for 80% of tools
✅ You need pmcp for complex 20%
✅ You're evolving from simple to complex
✅ You want the best of both worlds

## Summary

Both pforge and pmcp are production-ready tools from the same team. The choice depends on your specific needs:

- **Quick standard server?** → **pforge** (faster, easier)
- **Complex custom logic?** → **pmcp** (flexible, powerful)
- **Not sure?** → **Start with pforge**, migrate to pmcp if needed

Remember: pforge handlers are compatible with pmcp, so you can always evolve your architecture as requirements change.

---

**Next:** [Migration Between pforge and pmcp](ch01-04-migration.md)
