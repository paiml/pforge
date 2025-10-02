# Chapter 1.4: Migration Between pforge and pmcp

This chapter provides practical migration strategies for moving between pforge and pmcp, including real-world examples and best practices.

## Why Migrate?

### Common Migration Scenarios

**pmcp → pforge:**
- Reduce boilerplate code
- Standardize on declarative configuration
- Add built-in quality gates
- Improve iteration speed
- Simplify maintenance

**pforge → pmcp:**
- Need custom protocol extensions
- Require complex stateful logic
- Build library/SDK
- Need WebAssembly support
- Require custom transports

## Handler Compatibility

The good news: **pforge handlers are compatible with pmcp!**

Both frameworks share the same handler trait pattern, making migration straightforward.

```rust
// This handler works in BOTH pforge and pmcp
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use schemars::JsonSchema;

#[derive(Debug, Deserialize, JsonSchema)]
pub struct CalculateInput {
    a: f64,
    b: f64,
    operation: String,
}

#[derive(Debug, Serialize, JsonSchema)]
pub struct CalculateOutput {
    result: f64,
}

pub struct CalculateHandler;

#[async_trait]
impl pforge_runtime::Handler for CalculateHandler {
    type Input = CalculateInput;
    type Output = CalculateOutput;
    type Error = pforge_runtime::Error;

    async fn handle(&self, input: Self::Input) -> pforge_runtime::Result<Self::Output> {
        let result = match input.operation.as_str() {
            "add" => input.a + input.b,
            "subtract" => input.a - input.b,
            "multiply" => input.a * input.b,
            "divide" => {
                if input.b == 0.0 {
                    return Err(pforge_runtime::Error::Handler(
                        "Division by zero".into()
                    ));
                }
                input.a / input.b
            }
            _ => return Err(pforge_runtime::Error::Handler(
                "Unknown operation".into()
            )),
        };

        Ok(CalculateOutput { result })
    }
}
```

## Migrating from pmcp to pforge

### Step 1: Analyze Your pmcp Server

Identify your tools and their types:

```rust
// Existing pmcp server
let server = ServerBuilder::new()
    .name("my-server")
    .tool_typed("calculate", /* handler */)     // → Native handler
    .tool_typed("run_git", /* subprocess */)     // → CLI handler
    .tool_typed("fetch_api", /* HTTP call */)    // → HTTP handler
    .tool_typed("complex", /* custom logic */)   // → Keep in pmcp
    .build()?;
```

### Step 2: Extract Handlers

Convert tool closures to handler structs:

```rust
// Before (pmcp inline closure)
.tool_typed("calculate", |input: CalcInput, _| {
    Box::pin(async move {
        let result = input.a + input.b;
        Ok(serde_json::json!({ "result": result }))
    })
})

// After (pforge handler struct)
pub struct CalculateHandler;

#[async_trait]
impl Handler for CalculateHandler {
    type Input = CalcInput;
    type Output = CalcOutput;
    type Error = Error;

    async fn handle(&self, input: Self::Input) -> Result<Self::Output> {
        Ok(CalcOutput { result: input.a + input.b })
    }
}
```

### Step 3: Create forge.yaml

Map your tools to pforge configuration:

```yaml
forge:
  name: my-server
  version: 1.0.0
  transport: stdio

tools:
  # Native handlers (from pmcp tool_typed)
  - type: native
    name: calculate
    description: "Perform calculations"
    handler:
      path: handlers::CalculateHandler
    params:
      a: { type: float, required: true }
      b: { type: float, required: true }
      operation: { type: string, required: true }

  # CLI handlers (from subprocess calls)
  - type: cli
    name: run_git
    description: "Run git commands"
    command: git
    args: ["{{subcommand}}", "{{args}}"]
    cwd: /path/to/repo
    stream: true

  # HTTP handlers (from reqwest calls)
  - type: http
    name: fetch_api
    description: "Fetch from external API"
    endpoint: "https://api.example.com/{{path}}"
    method: GET
    headers:
      Authorization: "Bearer {{API_TOKEN}}"
```

### Step 4: Migrate State

```rust
// Before (pmcp manual state)
struct AppState {
    data: Arc<RwLock<HashMap<String, Value>>>,
}

let state = Arc::new(AppState {
    data: Arc::new(RwLock::new(HashMap::new())),
});

// After (pforge declarative state)
// In forge.yaml:
// state:
//   backend: sled
//   path: /tmp/my-server-state
//   cache_size: 1000

// In handler:
async fn handle(&self, input: Input) -> Result<Output> {
    let value = self.state.get("key").await?;
    self.state.set("key", value, None).await?;
    Ok(Output { value })
}
```

### Step 5: Test Migration

```bash
# Run existing pmcp tests
cargo test --all

# Generate pforge server
pforge build

# Run pforge tests
pforge test

# Compare behavior
diff <(echo '{"a": 5, "b": 3}' | ./pmcp-server) \
     <(echo '{"a": 5, "b": 3}' | pforge serve)
```

### Complete Example: pmcp → pforge

**Before (pmcp):**

```rust
// src/main.rs (120 lines)
use pmcp::{ServerBuilder, TypedTool};
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Debug, Deserialize, JsonSchema)]
struct CalcInput {
    a: f64,
    b: f64,
    operation: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    let state = Arc::new(RwLock::new(HashMap::new()));

    let server = ServerBuilder::new()
        .name("calculator")
        .version("1.0.0")
        .tool_typed("calculate", {
            let state = state.clone();
            move |input: CalcInput, _| {
                let state = state.clone();
                Box::pin(async move {
                    // 20 lines of logic
                    let result = match input.operation.as_str() {
                        "add" => input.a + input.b,
                        // ... more operations
                    };

                    // Update state
                    state.write().await.insert("last_result", result);

                    Ok(serde_json::json!({ "result": result }))
                })
            }
        })
        .tool_typed("run_command", |input: CmdInput, _| {
            Box::pin(async move {
                // 30 lines of subprocess handling
                let output = Command::new(&input.cmd)
                    .args(&input.args)
                    .output()
                    .await?;
                // ... error handling
                Ok(serde_json::json!({ "output": String::from_utf8(output.stdout)? }))
            })
        })
        .build()?;

    server.run_stdio().await
}
```

**After (pforge):**

```yaml
# forge.yaml (25 lines)
forge:
  name: calculator
  version: 1.0.0
  transport: stdio

state:
  backend: sled
  path: /tmp/calculator-state

tools:
  - type: native
    name: calculate
    description: "Perform arithmetic operations"
    handler:
      path: handlers::CalculateHandler
    params:
      a: { type: float, required: true }
      b: { type: float, required: true }
      operation: { type: string, required: true }

  - type: cli
    name: run_command
    description: "Run shell commands"
    command: "{{cmd}}"
    args: "{{args}}"
    stream: true
```

```rust
// src/handlers.rs (30 lines)
use pforge_runtime::{Handler, Result, Error};
use serde::{Deserialize, Serialize};
use schemars::JsonSchema;

#[derive(Debug, Deserialize, JsonSchema)]
pub struct CalcInput {
    a: f64,
    b: f64,
    operation: String,
}

#[derive(Debug, Serialize, JsonSchema)]
pub struct CalcOutput {
    result: f64,
}

pub struct CalculateHandler;

#[async_trait::async_trait]
impl Handler for CalculateHandler {
    type Input = CalcInput;
    type Output = CalcOutput;
    type Error = Error;

    async fn handle(&self, input: Self::Input) -> Result<Self::Output> {
        let result = match input.operation.as_str() {
            "add" => input.a + input.b,
            "subtract" => input.a - input.b,
            "multiply" => input.a * input.b,
            "divide" => {
                if input.b == 0.0 {
                    return Err(Error::Handler("Division by zero".into()));
                }
                input.a / input.b
            }
            _ => return Err(Error::Handler("Unknown operation".into())),
        };

        // State is managed automatically
        self.state.set("last_result", &result.to_string(), None).await?;

        Ok(CalcOutput { result })
    }
}
```

**Result:**
- **Code reduction:** 120 lines → 55 lines (54% reduction)
- **Complexity:** Manual state → Automatic state
- **Maintenance:** Easier to modify (YAML vs Rust)

## Migrating from pforge to pmcp

### Step 1: Keep Your Handlers

pforge handlers work directly in pmcp:

```rust
// handlers.rs - NO CHANGES NEEDED
pub struct MyHandler;

#[async_trait]
impl pforge_runtime::Handler for MyHandler {
    type Input = MyInput;
    type Output = MyOutput;
    type Error = pforge_runtime::Error;

    async fn handle(&self, input: Self::Input) -> pforge_runtime::Result<Self::Output> {
        // Handler logic stays the same
        Ok(MyOutput { result: process(input) })
    }
}
```

### Step 2: Convert YAML to pmcp Code

```yaml
# forge.yaml (pforge)
forge:
  name: my-server
  version: 1.0.0

tools:
  - type: native
    name: process
    handler:
      path: handlers::MyHandler
    params:
      input: { type: string, required: true }
```

Becomes:

```rust
// main.rs (pmcp)
use pmcp::ServerBuilder;

#[tokio::main]
async fn main() -> Result<()> {
    let server = ServerBuilder::new()
        .name("my-server")
        .version("1.0.0")
        .tool_typed("process", |input: MyInput, _| {
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

### Step 3: Add Custom Logic

Now you can extend beyond pforge's capabilities:

```rust
use pmcp::ServerBuilder;

#[tokio::main]
async fn main() -> Result<()> {
    let server = ServerBuilder::new()
        .name("advanced-server")
        .version("1.0.0")

        // Keep existing pforge handlers
        .tool_typed("basic", |input: BasicInput, _| {
            Box::pin(async move {
                let handler = BasicHandler;
                let output = handler.handle(input).await?;
                Ok(serde_json::to_value(output)?)
            })
        })

        // Add custom complex logic (not possible in pforge)
        .tool_typed("complex", |input: ComplexInput, _| {
            Box::pin(async move {
                // Custom database transactions
                let mut tx = db_pool.begin().await?;

                // Complex business logic
                let result = perform_analysis(&mut tx, input).await?;

                // Custom error handling
                match result {
                    Ok(data) => {
                        tx.commit().await?;
                        Ok(serde_json::to_value(data)?)
                    }
                    Err(e) => {
                        tx.rollback().await?;
                        Err(pmcp::Error::Handler(e.to_string()))
                    }
                }
            })
        })

        // Custom protocol extensions
        .custom_method("custom/analyze", |params| {
            Box::pin(async move {
                custom_protocol_handler(params).await
            })
        })

        .build()?;

    server.run_stdio().await
}
```

## Hybrid Approach: Using Both

You can use pforge and pmcp together in the same project:

### Strategy 1: pforge for Simple, pmcp for Complex

```rust
// Use pforge for 80% of simple tools
mod pforge_tools {
    include!(concat!(env!("OUT_DIR"), "/pforge_generated.rs"));
}

// Use pmcp for 20% of complex tools
use pmcp::ServerBuilder;

#[tokio::main]
async fn main() -> Result<()> {
    let mut builder = ServerBuilder::new()
        .name("hybrid-server")
        .version("1.0.0");

    // Add all pforge-generated tools
    for (name, handler) in pforge_tools::handlers() {
        builder = builder.tool_typed(name, handler);
    }

    // Add custom complex tools
    builder = builder
        .tool_typed("complex_analysis", |input: AnalysisInput, _| {
            Box::pin(async move {
                // Complex logic not expressible in pforge
                let result = ml_model.predict(input).await?;
                Ok(serde_json::to_value(result)?)
            })
        })
        .tool_typed("database_query", |input: QueryInput, _| {
            Box::pin(async move {
                // Complex transactional database operations
                let mut tx = pool.begin().await?;
                let result = execute_query(&mut tx, input).await?;
                tx.commit().await?;
                Ok(serde_json::to_value(result)?)
            })
        });

    let server = builder.build()?;
    server.run_stdio().await
}
```

### Strategy 2: Parallel Servers

Run pforge and pmcp servers side-by-side:

```bash
# Terminal 1: pforge server for standard tools
cd pforge-server
pforge serve

# Terminal 2: pmcp server for custom tools
cd pmcp-server
cargo run --release
```

```yaml
# Claude Desktop config
{
  "mcpServers": {
    "standard-tools": {
      "command": "pforge",
      "args": ["serve"],
      "cwd": "/path/to/pforge-server"
    },
    "custom-tools": {
      "command": "/path/to/pmcp-server/target/release/custom-server",
      "cwd": "/path/to/pmcp-server"
    }
  }
}
```

## Migration Checklist

### pmcp → pforge Migration

- [ ] Identify tool types (native/cli/http/pipeline)
- [ ] Extract handlers from closures
- [ ] Create forge.yaml configuration
- [ ] Convert state management to pforge state backend
- [ ] Set up quality gates (PMAT)
- [ ] Write tests for migrated handlers
- [ ] Benchmark performance (should improve)
- [ ] Update documentation
- [ ] Deploy and monitor

### pforge → pmcp Migration

- [ ] Keep existing handler implementations
- [ ] Convert forge.yaml to ServerBuilder code
- [ ] Add custom logic as needed
- [ ] Implement custom state management (if needed)
- [ ] Set up CI/CD (manual configuration)
- [ ] Write additional tests
- [ ] Update documentation
- [ ] Deploy and monitor

## Common Migration Pitfalls

### Pitfall 1: State Management Mismatch

**Problem:**
```rust
// pmcp: Manual Arc<RwLock>
let data = state.read().await.get("key").cloned();

// pforge: Async state backend
let data = self.state.get("key").await?;
```

**Solution:** Choose consistent state backend or use adapter pattern.

### Pitfall 2: Error Handling Differences

**Problem:**
```rust
// pmcp: Custom error types
Err(MyError::Database(e))

// pforge: Standardized errors
Err(Error::Handler(e.to_string()))
```

**Solution:** Map custom errors to pforge Error types:

```rust
impl From<MyError> for pforge_runtime::Error {
    fn from(err: MyError) -> Self {
        match err {
            MyError::Database(e) => Error::Handler(format!("DB: {}", e)),
            MyError::Validation(msg) => Error::Validation(msg),
            MyError::Timeout => Error::Timeout,
        }
    }
}
```

### Pitfall 3: Missing CLI/HTTP Wrappers

**Problem:** pmcp requires manual subprocess/HTTP handling.

**Solution:** Extract to separate pforge server or use libraries:

```rust
// Instead of reinventing CLI wrapper
use tokio::process::Command;

// Use pforge CLI handler type or simple wrapper
async fn run_command(cmd: &str, args: &[String]) -> Result<String> {
    let output = Command::new(cmd)
        .args(args)
        .output()
        .await?;

    String::from_utf8(output.stdout)
        .map_err(|e| Error::Handler(e.to_string()))
}
```

## Performance Considerations

### pmcp → pforge

**Expected improvements:**
- Tool dispatch: 11x faster (perfect hash vs HashMap)
- Throughput: 1.5-1.7x higher
- Memory per tool: ~50% reduction

**Trade-offs:**
- Cold start: ~2x slower (code generation)
- Binary size: 2-3x larger

### pforge → pmcp

**Expected changes:**
- More control over performance tuning
- Custom allocator options
- Zero-copy optimizations possible
- Manual optimization needed

## Testing Migration

### Compatibility Test

```rust
#[cfg(test)]
mod migration_tests {
    use super::*;

    #[tokio::test]
    async fn test_handler_compatibility() {
        // Test handler works in both pforge and pmcp
        let handler = MyHandler;

        let input = MyInput { value: 42 };
        let output = handler.handle(input).await.unwrap();

        assert_eq!(output.result, 84);
    }

    #[tokio::test]
    async fn test_behavior_equivalence() {
        // Compare pforge and pmcp server responses
        let pforge_response = test_pforge_server(input.clone()).await?;
        let pmcp_response = test_pmcp_server(input.clone()).await?;

        assert_eq!(pforge_response, pmcp_response);
    }
}
```

## Summary

Migration between pforge and pmcp is straightforward thanks to handler compatibility:

**Key Points:**
1. pforge handlers work in pmcp without changes
2. pmcp → pforge reduces code by ~50%
3. pforge → pmcp adds flexibility for complex cases
4. Hybrid approach combines benefits of both
5. Choose based on current needs, migrate as requirements evolve

**Migration Decision:**
- More tools becoming standard? → Migrate to pforge
- Need custom protocols? → Migrate to pmcp
- Mixed requirements? → Use hybrid approach

---

**Next:** [Architecture: How pforge Uses pmcp](ch01-05-architecture-pmcp.md)
