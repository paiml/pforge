# Chapter 1: pforge vs pmcp (rust-mcp-sdk)

Both **pforge** and **pmcp** (Pragmatic Model Context Protocol SDK, also known as rust-mcp-sdk) are Rust implementations for building MCP servers, created by the same team at Pragmatic AI Labs. However, they serve **fundamentally different use cases**.

## The Key Difference

**pmcp** is a **library/SDK** - you write Rust code to build your MCP server.

**pforge** is a **framework** - you write YAML configuration and optional Rust handlers.

Think of it like this:
- **pmcp** ≈ Express.js (you write code)
- **pforge** ≈ Cargo Lambda (you write config + minimal code)

## Quick Comparison Table

| Feature | pforge | pmcp |
|---------|--------|------|
| **Approach** | Declarative YAML + handlers | Programmatic Rust SDK |
| **Code Required** | <10 lines YAML + handlers | 50-200+ lines Rust |
| **Type Safety** | Compile-time (via codegen) | Compile-time (native Rust) |
| **Performance** | <1μs dispatch (optimized) | <10μs (general purpose) |
| **Learning Curve** | Low (YAML + basic Rust) | Medium (full Rust + MCP) |
| **Flexibility** | 4 handler types (fixed) | Unlimited (write any code) |
| **Quality Gates** | Built-in (PMAT, TDD) | Optional (you implement) |
| **Build Process** | Code generation | Standard Rust |
| **Best For** | Standard MCP patterns | Custom complex logic |
| **Boilerplate** | Near-zero | Moderate |
| **Crates.io** | ✅ Publishable | ✅ Publishable |

## Side-by-Side Example

### The Same Calculator Tool

**With pmcp (rust-mcp-sdk):**

```rust
// main.rs (~60 lines)
use pmcp::{ServerBuilder, TypedTool};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, JsonSchema)]
struct CalculatorArgs {
    operation: String,
    a: f64,
    b: f64,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let server = ServerBuilder::new()
        .name("calculator-server")
        .version("1.0.0")
        .tool_typed("calculate", |args: CalculatorArgs, _extra| {
            Box::pin(async move {
                let result = match args.operation.as_str() {
                    "add" => args.a + args.b,
                    "subtract" => args.a - args.b,
                    "multiply" => args.a * args.b,
                    "divide" => {
                        if args.b == 0.0 {
                            return Err(pmcp::Error::Validation(
                                "Division by zero".into()
                            ));
                        }
                        args.a / args.b
                    }
                    _ => return Err(pmcp::Error::Validation(
                        "Unknown operation".into()
                    )),
                };
                Ok(serde_json::json!({ "result": result }))
            })
        })
        .build()?;

    // Run server with stdio transport
    server.run_stdio().await?;
    Ok(())
}
```

**With pforge:**

```yaml
# forge.yaml (8 lines)
forge:
  name: calculator-server
  version: 1.0.0

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
```

```rust
// src/handlers.rs (~25 lines)
use pforge_runtime::{Handler, Result, Error};
use serde::{Deserialize, Serialize};
use schemars::JsonSchema;

#[derive(Debug, Deserialize, JsonSchema)]
pub struct CalculateInput {
    operation: String,
    a: f64,
    b: f64,
}

#[derive(Debug, Serialize, JsonSchema)]
pub struct CalculateOutput {
    result: f64,
}

pub struct CalculateHandler;

#[async_trait::async_trait]
impl Handler for CalculateHandler {
    type Input = CalculateInput;
    type Output = CalculateOutput;
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
        Ok(CalculateOutput { result })
    }
}
```

```bash
# Run it
pforge serve
```

## When to Use Each

### Use **pforge** when:

✅ You're building standard MCP servers (tools, resources, prompts)
✅ You want minimal boilerplate
✅ You need fast iteration (change YAML, no recompile)
✅ You want built-in quality gates and TDD methodology
✅ You're wrapping CLIs, HTTP APIs, or simple logic
✅ You want sub-microsecond tool dispatch
✅ You're new to Rust (simpler to get started)
✅ You want enforced best practices

**Examples:**
- CLI tool wrappers (git, docker, kubectl)
- HTTP API proxies (GitHub, Slack, AWS)
- Simple data transformations
- Multi-tool pipelines

### Use **pmcp** when:

✅ You need complete control over server logic
✅ You're implementing complex stateful behavior
✅ You need custom transport implementations
✅ You're building a library/SDK for others
✅ You need features not in pforge's 4 handler types
✅ You want to publish a general-purpose MCP server
✅ You're comfortable with full Rust development

**Examples:**
- Database servers with custom query logic
- Real-time collaborative servers
- Custom protocol extensions
- Servers with complex state machines
- WebAssembly/browser-based servers

## Can I Use Both Together?

**Yes!** You can:

1. **Start with pforge**, then migrate complex tools to pmcp
2. **Use pmcp for the core**, pforge for simple wrappers
3. **Publish pmcp handlers** that pforge can use

Example: Use pforge for 90% of simple tools, drop down to pmcp for the 10% that need custom logic.

## Performance Comparison

| Metric | pforge | pmcp |
|--------|--------|------|
| **Tool Dispatch** | <1μs (perfect hash) | <10μs (hash map) |
| **Cold Start** | <100ms | <50ms |
| **Memory/Tool** | <256B | <512B |
| **Throughput** | >100K req/s | >50K req/s |
| **Binary Size** | Larger (includes codegen) | Smaller (minimal) |

**Why is pforge faster for dispatch?**
- Compile-time code generation with perfect hashing
- Zero dynamic lookups
- Inlined handler calls

**Why is pmcp faster for cold start?**
- No code generation step
- Simpler binary

## Code Size Comparison

For a typical 10-tool MCP server:

- **pforge**: ~50 lines YAML + ~200 lines handlers = **~250 lines total**
- **pmcp**: ~500-800 lines Rust (including boilerplate)

## Quality & Testing

| Aspect | pforge | pmcp |
|--------|--------|------|
| **Quality Gates** | Built-in pre-commit hooks | You implement |
| **TDD Methodology** | EXTREME TDD (5-min cycles) | Your choice |
| **Property Testing** | Built-in generators | You implement |
| **Mutation Testing** | cargo-mutants integrated | You configure |
| **Coverage Target** | 80%+ enforced | You set |
| **Complexity Limit** | Max 20 enforced | You set |

## Migration Path

### pmcp → pforge

If you have a pmcp server and want to try pforge:

1. Extract your tool logic into handlers
2. Create `forge.yaml` config
3. Test with `pforge serve`

### pforge → pmcp

If you need more flexibility:

1. Use your pforge handlers as-is
2. Replace YAML with `ServerBuilder` code
3. Add custom logic as needed

## Real-World Usage

**pforge in production:**
- PMAT code analysis server (pforge wraps pmat CLI)
- GitHub webhook server (pforge proxies GitHub API)
- Data pipeline orchestrator (pforge chains tools)

**pmcp in production:**
- Browser-based REPL (WebAssembly, custom logic)
- Database query server (complex state, transactions)
- Real-time collaboration (WebSocket, stateful)

## Summary

Choose based on your needs:

- **Quick standard MCP server?** → **pforge**
- **Complex custom logic?** → **pmcp**
- **Not sure?** → **Start with pforge**, migrate to pmcp if needed

Both are production-ready, both support crates.io publishing, and both are maintained by the same team.

---

Next: [When to Use pforge](ch01-01-when-pforge.md)
