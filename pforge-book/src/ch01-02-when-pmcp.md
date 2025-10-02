# Chapter 1.2: When to Use pmcp Directly

This chapter explores scenarios where using **pmcp** (rust-mcp-sdk) directly is the better choice than pforge.

## The pmcp Sweet Spot

pmcp is a **low-level SDK** that gives you complete control over your MCP server. Use it when pforge's abstraction layer gets in the way of what you're trying to achieve.

## Use pmcp When...

### 1. **You Need Custom MCP Protocol Extensions**

pmcp lets you implement custom protocol features not in the standard MCP spec:

```rust
use pmcp::ServerBuilder;

#[tokio::main]
async fn main() -> Result<()> {
    let server = ServerBuilder::new()
        .name("custom-server")
        .version("1.0.0")

        // Custom JSON-RPC method
        .custom_method("custom/analyze", |params| {
            Box::pin(async move {
                // Your custom protocol logic
                let result = custom_analysis(params).await?;
                Ok(serde_json::to_value(result)?)
            })
        })

        // Custom notification handler
        .on_notification("custom/event", |params| {
            Box::pin(async move {
                handle_custom_event(params).await
            })
        })

        .build()?;

    server.run_stdio().await
}
```

**Why pmcp wins:**
- Full control over JSON-RPC messages
- Custom method registration
- Direct access to transport layer
- No framework constraints

### 2. **You Need Complex Stateful Logic**

pmcp gives you full control over server state and lifecycle:

```rust
use pmcp::ServerBuilder;
use std::sync::Arc;
use tokio::sync::RwLock;

// Complex application state
struct AppState {
    db_pool: sqlx::PgPool,
    cache: Arc<RwLock<HashMap<String, CachedValue>>>,
    query_planner: QueryPlanner,
    transaction_log: Arc<Mutex<Vec<Transaction>>>,
}

#[tokio::main]
async fn main() -> Result<()> {
    let state = Arc::new(AppState {
        db_pool: create_pool().await?,
        cache: Arc::new(RwLock::new(HashMap::new())),
        query_planner: QueryPlanner::new(),
        transaction_log: Arc::new(Mutex::new(Vec::new())),
    });

    let server = ServerBuilder::new()
        .name("database-server")
        .tool_typed("execute_query", {
            let state = state.clone();
            move |args: QueryArgs, _extra| {
                let state = state.clone();
                Box::pin(async move {
                    // Complex transactional logic
                    let mut tx = state.db_pool.begin().await?;

                    // Log transaction
                    state.transaction_log.lock().await.push(Transaction {
                        query: args.sql.clone(),
                        timestamp: Utc::now(),
                    });

                    // Execute with query planner
                    let plan = state.query_planner.plan(&args.sql)?;
                    let result = execute_plan(&mut tx, plan).await?;

                    // Update cache
                    state.cache.write().await.insert(
                        cache_key(&args),
                        CachedValue { result: result.clone(), ttl: Instant::now() }
                    );

                    tx.commit().await?;
                    Ok(serde_json::to_value(result)?)
                })
            }
        })
        .build()?;

    server.run_stdio().await
}
```

**Why pmcp wins:**
- Full lifecycle control
- Complex state management
- Custom transaction handling
- Direct database integration

### 3. **You Need Custom Transport Implementations**

pmcp supports custom transports beyond stdio/SSE/WebSocket:

```rust
use pmcp::{Server, Transport};

// Custom Unix domain socket transport
struct UnixSocketTransport {
    socket_path: PathBuf,
}

#[async_trait::async_trait]
impl Transport for UnixSocketTransport {
    async fn run(&self, server: Server) -> Result<()> {
        let listener = UnixListener::bind(&self.socket_path)?;

        loop {
            let (stream, _) = listener.accept().await?;
            let server = server.clone();

            tokio::spawn(async move {
                handle_connection(server, stream).await
            });
        }
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let server = ServerBuilder::new()
        .name("unix-socket-server")
        .tool_typed("process", |args, _| { /* ... */ })
        .build()?;

    let transport = UnixSocketTransport {
        socket_path: "/tmp/mcp.sock".into(),
    };

    transport.run(server).await
}
```

**Why pmcp wins:**
- Custom transport protocols
- Direct socket/network access
- Custom message framing
- Protocol optimization

### 4. **You're Building a Library/SDK**

pmcp is designed for building reusable components:

```rust
// Your reusable MCP server library
pub struct CodeAnalysisServer {
    analyzers: Vec<Box<dyn Analyzer>>,
}

impl CodeAnalysisServer {
    pub fn new() -> Self {
        Self {
            analyzers: vec![
                Box::new(ComplexityAnalyzer::new()),
                Box::new(SecurityAnalyzer::new()),
                Box::new(PerformanceAnalyzer::new()),
            ],
        }
    }

    pub fn add_analyzer(&mut self, analyzer: Box<dyn Analyzer>) {
        self.analyzers.push(analyzer);
    }

    pub fn build(self) -> Result<pmcp::Server> {
        let mut builder = ServerBuilder::new()
            .name("code-analysis")
            .version("1.0.0");

        // Register tools from analyzers
        for analyzer in self.analyzers {
            for tool in analyzer.tools() {
                builder = builder.tool_typed(&tool.name, tool.handler);
            }
        }

        builder.build()
    }
}

// Users can extend your library
fn main() -> Result<()> {
    let mut server = CodeAnalysisServer::new();

    // Add custom analyzer
    server.add_analyzer(Box::new(MyCustomAnalyzer::new()));

    let server = server.build()?;
    server.run_stdio().await
}
```

**Why pmcp wins:**
- Composable API
- Extensibility hooks
- Library-friendly design
- No framework lock-in

### 5. **You Need WebAssembly Compilation**

pmcp can compile to WASM for browser-based servers:

```rust
use pmcp::ServerBuilder;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub struct WasmMcpServer {
    server: pmcp::Server,
}

#[wasm_bindgen]
impl WasmMcpServer {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Result<WasmMcpServer, JsValue> {
        let server = ServerBuilder::new()
            .name("wasm-server")
            .tool_typed("process", |args: ProcessArgs, _| {
                Box::pin(async move {
                    // Pure Rust logic, runs in browser
                    Ok(serde_json::json!({ "result": process(args) }))
                })
            })
            .build()
            .map_err(|e| JsValue::from_str(&e.to_string()))?;

        Ok(WasmMcpServer { server })
    }

    #[wasm_bindgen]
    pub async fn handle_request(&self, request: JsValue) -> Result<JsValue, JsValue> {
        // Handle MCP requests from JavaScript
        let result = self.server.handle(request).await?;
        Ok(result)
    }
}
```

**Why pmcp wins:**
- WASM target support
- Browser compatibility
- Pure Rust execution
- JavaScript interop

### 6. **You Need Dynamic Server Configuration**

pmcp allows runtime configuration changes:

```rust
use pmcp::ServerBuilder;
use std::sync::Arc;

struct DynamicServer {
    builder: Arc<RwLock<ServerBuilder>>,
}

impl DynamicServer {
    pub async fn register_tool_at_runtime(&self, name: String, handler: impl Fn() -> Future) {
        let mut builder = self.builder.write().await;
        *builder = builder.clone().tool_typed(name, handler);
        // Rebuild and hot-swap server
    }

    pub async fn unregister_tool(&self, name: &str) {
        // Remove tool at runtime
    }
}
```

**Why pmcp wins:**
- Runtime tool registration
- Hot-swapping capabilities
- Dynamic configuration
- Plugin architecture

### 7. **You Need Fine-Grained Performance Control**

pmcp lets you optimize every aspect:

```rust
use pmcp::ServerBuilder;

#[tokio::main]
async fn main() -> Result<()> {
    let server = ServerBuilder::new()
        .name("optimized-server")

        // Custom executor
        .with_runtime(tokio::runtime::Builder::new_multi_thread()
            .worker_threads(16)
            .thread_name("mcp-worker")
            .thread_stack_size(4 * 1024 * 1024)
            .build()?)

        // Custom buffer sizes
        .with_buffer_size(65536)

        // Custom timeout strategy
        .with_timeout_strategy(CustomTimeoutStrategy::new())

        // Zero-copy tool handlers
        .tool_raw("process_bytes", |bytes: &[u8], _| {
            Box::pin(async move {
                // Process without allocations
                process_bytes_in_place(bytes)
            })
        })

        .build()?;

    server.run_stdio().await
}
```

**Why pmcp wins:**
- Custom runtime configuration
- Memory allocation control
- Zero-copy operations
- Performance tuning hooks

### 8. **You Need Multi-Server Orchestration**

pmcp allows running multiple servers in one process:

```rust
use pmcp::ServerBuilder;

#[tokio::main]
async fn main() -> Result<()> {
    // Server 1: Code analysis
    let analysis_server = ServerBuilder::new()
        .name("code-analysis")
        .tool_typed("analyze", |args, _| { /* ... */ })
        .build()?;

    // Server 2: File operations
    let file_server = ServerBuilder::new()
        .name("file-ops")
        .tool_typed("read_file", |args, _| { /* ... */ })
        .build()?;

    // Run both on different transports
    tokio::try_join!(
        analysis_server.run_stdio(),
        file_server.run_sse("0.0.0.0:8080"),
    )?;

    Ok(())
}
```

**Why pmcp wins:**
- Multi-server orchestration
- Different transports per server
- Process-level control
- Resource sharing

## Real-World Use Cases

### Case Study 1: Database Query Server

**Challenge:** Build a stateful database query server with transaction support

**Why pmcp:**
```rust
struct QueryServer {
    pool: PgPool,
    active_transactions: Arc<RwLock<HashMap<Uuid, Transaction>>>,
}

impl QueryServer {
    pub async fn build(self) -> Result<pmcp::Server> {
        ServerBuilder::new()
            .name("db-server")
            .tool_typed("begin_transaction", /* complex state logic */)
            .tool_typed("execute_query", /* transaction-aware */)
            .tool_typed("commit", /* finalize transaction */)
            .tool_typed("rollback", /* abort transaction */)
            .build()
    }
}
```

**Results:**
- Full control over connection pooling
- Custom transaction management
- Complex state coordination
- Optimized query execution

### Case Study 2: Real-Time Collaborative Server

**Challenge:** Build a server for real-time collaboration with WebSocket transport

**Why pmcp:**
```rust
struct CollaborationServer {
    rooms: Arc<RwLock<HashMap<String, Room>>>,
    connections: Arc<RwLock<HashMap<Uuid, WebSocket>>>,
}

impl CollaborationServer {
    pub async fn run(self) -> Result<()> {
        let server = ServerBuilder::new()
            .name("collab-server")
            .tool_typed("join_room", /* manage connections */)
            .tool_typed("send_message", /* broadcast to room */)
            .on_notification("user_typing", /* real-time events */)
            .build()?;

        // Custom WebSocket transport with broadcasting
        server.run_websocket("0.0.0.0:8080").await
    }
}
```

**Results:**
- WebSocket broadcast support
- Real-time event handling
- Custom connection management
- Room-based message routing

### Case Study 3: Browser-Based REPL

**Challenge:** Build an MCP server that runs entirely in the browser

**Why pmcp:**
```rust
#[wasm_bindgen]
pub struct BrowserRepl {
    server: pmcp::Server,
    history: Vec<String>,
}

#[wasm_bindgen]
impl BrowserRepl {
    pub fn new() -> Self {
        let server = ServerBuilder::new()
            .name("browser-repl")
            .tool_typed("eval", /* safe evaluation */)
            .tool_typed("history", /* return history */)
            .build()
            .unwrap();

        Self { server, history: vec![] }
    }

    pub async fn execute(&mut self, code: String) -> JsValue {
        self.history.push(code.clone());
        self.server.handle_tool("eval", serde_json::json!({ "code": code })).await
    }
}
```

**Results:**
- Runs entirely in browser
- No backend required
- JavaScript interoperability
- Secure sandboxed execution

## Performance Characteristics

| Metric | pmcp | Notes |
|--------|------|-------|
| **Tool Dispatch** | <10μs | HashMap lookup, very fast |
| **Cold Start** | <50ms | Minimal startup overhead |
| **Memory/Tool** | <512B | Flexible structure |
| **Throughput** | >50K req/s | Highly optimized |
| **Binary Size** | ~2MB | Minimal dependencies |

## When pmcp Might NOT Be the Best Choice

pmcp is **not** ideal when:

1. **You want zero boilerplate**
   - pmcp requires more code than pforge
   - Use pforge for standard patterns

2. **You want declarative configuration**
   - pmcp is programmatic, not declarative
   - Use pforge for YAML-based config

3. **You want built-in quality gates**
   - pmcp doesn't enforce quality standards
   - Use pforge for automatic PMAT integration

4. **You want CLI/HTTP handler types out of the box**
   - pmcp requires you to write these yourself
   - Use pforge for pre-built handler types

See [Chapter 1.1: When to Use pforge](ch01-01-when-pforge.md) for these cases.

## Combining pforge and pmcp

You can use both in the same project:

```rust
// Use pforge for simple tools
mod pforge_tools {
    include!(concat!(env!("OUT_DIR"), "/pforge_generated.rs"));
}

// Use pmcp for complex tools
use pmcp::ServerBuilder;

#[tokio::main]
async fn main() -> Result<()> {
    let mut builder = ServerBuilder::new()
        .name("hybrid-server")
        .version("1.0.0");

    // Add pforge-generated tools
    for (name, handler) in pforge_tools::handlers() {
        builder = builder.tool_typed(name, handler);
    }

    // Add custom pmcp tool with complex logic
    builder = builder.tool_typed("complex_analysis", |args: AnalysisArgs, _| {
        Box::pin(async move {
            // Complex custom logic here
            let result = perform_complex_analysis(args).await?;
            Ok(serde_json::to_value(result)?)
        })
    });

    let server = builder.build()?;
    server.run_stdio().await
}
```

## Summary

Use **pmcp** when you need:

✅ Custom MCP protocol extensions
✅ Complex stateful logic
✅ Custom transport implementations
✅ Library/SDK development
✅ WebAssembly compilation
✅ Runtime configuration
✅ Fine-grained performance control
✅ Multi-server orchestration

Use **pforge** when you want:

❌ Minimal boilerplate
❌ Declarative YAML configuration
❌ Built-in quality gates
❌ Pre-built handler types
❌ Fast iteration without recompilation

**Not sure?** Start with pforge. You can always integrate pmcp for complex features later.

---

**Next:** [Side-by-Side Comparison](ch01-03-comparison.md)
