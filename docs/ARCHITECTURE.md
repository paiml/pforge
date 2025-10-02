# pforge Architecture

This document provides a deep dive into the pforge architecture, design decisions, and implementation details.

## Overview

pforge is built as a modular monolith with five core crates:

```
pforge/
├── pforge-cli          # Command-line interface
├── pforge-config       # Configuration parsing and validation
├── pforge-runtime      # Core runtime and handlers
├── pforge-codegen      # Code generation from config
└── pforge-macro        # Procedural macros (future)
```

## Design Principles

### 1. Zero-Cost Abstractions

- Handler dispatch uses O(1) FxHashMap lookups
- Async-first design with `async_trait`
- Type erasure with `Arc<dyn Handler>`
- Compile-time type safety

### 2. Type Safety

- Strong typing throughout the stack
- Serde for serialization/deserialization
- JsonSchema for schema generation
- No `unwrap()` in production code

### 3. Modularity

- Clear separation of concerns
- Loosely coupled crates
- Dependency injection for state and services
- Pluggable middleware system

### 4. Performance

- O(1) handler lookup with FxHashMap
- Connection pooling (reqwest)
- Lazy evaluation where possible
- Efficient error handling with thiserror

## Core Components

### Configuration System (pforge-config)

**Purpose**: Parse and validate YAML configurations

**Key Types**:

```rust
pub struct ForgeConfig {
    pub forge: ForgeMetadata,
    pub tools: Vec<ToolDef>,
    pub resources: Vec<ResourceDef>,
    pub prompts: Vec<PromptDef>,
    pub state: Option<StateDef>,
}

pub enum ToolDef {
    Native { ... },
    Cli { ... },
    Http { ... },
    Pipeline { ... },
}
```

**Design Decisions**:

- Tagged enums for tool types (type-safe matching)
- `serde` with `deny_unknown_fields` for strict validation
- Separate validator pass for cross-cutting concerns
- Default values for optional fields

**Validation**:

```rust
pub fn validate_config(config: &ForgeConfig) -> Result<()> {
    // Check for duplicate tool names
    let mut seen = HashSet::new();
    for tool in &config.tools {
        if !seen.insert(tool.name()) {
            return Err(Error::Validation(format!(
                "Duplicate tool name: {}", tool.name()
            )));
        }
    }
    Ok(())
}
```

### Runtime System (pforge-runtime)

**Purpose**: Handler execution, middleware, state management

**Architecture**:

```
┌─────────────────────────────────────────┐
│          MCP Server                     │
│  ┌───────────────────────────────────┐  │
│  │   MiddlewareChain                 │  │
│  │  ┌────────────────────────────┐   │  │
│  │  │   HandlerRegistry          │   │  │
│  │  │  ┌──────────────────────┐  │   │  │
│  │  │  │  Handler Execution  │  │   │  │
│  │  │  └──────────────────────┘  │   │  │
│  │  └────────────────────────────┘   │  │
│  └───────────────────────────────────┘  │
└─────────────────────────────────────────┘
```

**Handler Trait**:

```rust
#[async_trait]
pub trait Handler: Send + Sync + 'static {
    type Input: JsonSchema + DeserializeOwned + Send;
    type Output: JsonSchema + Serialize + Send;
    type Error: Into<Error>;

    async fn handle(&self, input: Self::Input)
        -> Result<Self::Output, Self::Error>;
}
```

**Handler Registry**:

```rust
pub struct HandlerRegistry {
    handlers: FxHashMap<String, Arc<dyn HandlerEntry>>,
}

impl HandlerRegistry {
    pub fn register<H: Handler>(&mut self, name: String, handler: H) {
        self.handlers.insert(
            name,
            Arc::new(TypedHandlerEntry::new(handler))
        );
    }

    pub async fn dispatch(&self, name: &str, input: Value)
        -> Result<Value> {
        let handler = self.handlers.get(name)
            .ok_or_else(|| Error::NotFound(name.to_string()))?;

        handler.call(input).await
    }
}
```

**Performance**: O(1) average-case lookup using FxHashMap (faster than std HashMap for small keys).

### State Management

**Trait**:

```rust
#[async_trait]
pub trait StateManager: Send + Sync {
    async fn get(&self, key: &str) -> Result<Option<Vec<u8>>>;
    async fn set(&self, key: &str, value: Vec<u8>, ttl: Option<Duration>) -> Result<()>;
    async fn delete(&self, key: &str) -> Result<()>;
    async fn exists(&self, key: &str) -> Result<bool>;
}
```

**Implementations**:

1. **SledStateManager**: Persistent storage with Sled embedded database
2. **MemoryStateManager**: In-memory storage with DashMap for concurrent access

**Design**: Abstract trait allows swapping backends without changing handler code.

### Middleware System

**Chain Execution**:

```rust
pub struct MiddlewareChain {
    middlewares: Vec<Arc<dyn Middleware>>,
}

impl MiddlewareChain {
    pub async fn execute<F, Fut>(&self, request: Value, handler: F)
        -> Result<Value>
    where
        F: FnOnce(Value) -> Fut,
        Fut: Future<Output = Result<Value>>,
    {
        // Before phase (in order)
        for mw in &self.middlewares {
            request = mw.before(request).await?;
        }

        // Handler execution
        let result = handler(request.clone()).await;

        // After phase (reverse order)
        match result {
            Ok(response) => {
                for mw in self.middlewares.iter().rev() {
                    response = mw.after(request.clone(), response).await?;
                }
                Ok(response)
            }
            Err(error) => {
                // Error recovery (reverse order)
                for mw in self.middlewares.iter().rev() {
                    match mw.on_error(request.clone(), error).await {
                        Ok(recovery) => return Ok(recovery),
                        Err(new_error) => error = new_error,
                    }
                }
                Err(error)
            }
        }
    }
}
```

**Built-in Middleware**:

- **LoggingMiddleware**: Request/response logging
- **ValidationMiddleware**: Input validation
- **RecoveryMiddleware**: Circuit breaker + error tracking
- **TimeoutMiddleware**: Time-limited execution
- **RetryMiddleware**: Automatic retries with backoff

### Resource System

**URI Template Matching**:

```rust
pub struct ResourceManager {
    resources: Vec<ResourceEntry>,
}

struct ResourceEntry {
    uri_template: String,
    pattern: Regex,          // Compiled regex
    param_names: Vec<String>, // Extracted parameters
    supports: Vec<ResourceOperation>,
    handler: Arc<dyn ResourceHandler>,
}
```

**Pattern Compilation**:

```
"file:///{path}"  -> ^file:///(.+)$
"api://{service}/{resource}" -> ^api://([^/]+)/(.+)$
```

**Smart Matching**:
- Parameters followed by `/` use segment matching `[^/]+`
- Parameters at end use greedy matching `.+`

### Prompt System

**Template Interpolation**:

```rust
pub struct PromptManager {
    prompts: HashMap<String, PromptEntry>,
}

impl PromptManager {
    pub fn render(&self, name: &str, args: HashMap<String, Value>)
        -> Result<String> {
        let entry = self.prompts.get(name)?;

        // Validate required arguments
        self.validate_arguments(entry, &args)?;

        // Interpolate {{variable}} syntax
        self.interpolate(&entry.template, &args)
    }
}
```

### Error Recovery

**Circuit Breaker States**:

```
   ┌─────────────┐
   │   CLOSED    │ (Normal operation)
   └──────┬──────┘
          │ failures >= threshold
          ▼
   ┌─────────────┐
   │    OPEN     │ (Reject requests)
   └──────┬──────┘
          │ timeout elapsed
          ▼
   ┌─────────────┐
   │  HALF-OPEN  │ (Testing recovery)
   └──────┬──────┘
          │ successes >= threshold
          │ or any failure
          ▼
    CLOSED or OPEN
```

**Implementation**:

```rust
pub struct CircuitBreaker {
    config: CircuitBreakerConfig,
    state: Arc<RwLock<CircuitState>>,
    failure_count: Arc<AtomicUsize>,
    success_count: Arc<AtomicUsize>,
    last_failure_time: Arc<RwLock<Option<Instant>>>,
}
```

**Thread Safety**: Uses atomic operations and RwLock for concurrent access.

### Timeout and Retry

**Retry with Exponential Backoff**:

```rust
pub async fn retry_with_policy<F, Fut, T>(
    policy: &RetryPolicy,
    mut operation: F,
) -> Result<T>
where
    F: FnMut() -> Fut,
    Fut: Future<Output = Result<T>>,
{
    let mut attempt = 0;

    while attempt < policy.max_attempts {
        match operation().await {
            Ok(result) => return Ok(result),
            Err(error) => {
                if !policy.is_retryable(&error) {
                    return Err(error);
                }

                attempt += 1;
                if attempt < policy.max_attempts {
                    let backoff = policy.backoff_duration(attempt - 1);
                    tokio::time::sleep(backoff).await;
                }
            }
        }
    }

    Err(Error::Handler("Max retries exceeded".to_string()))
}
```

**Backoff Calculation**:

```
backoff = initial * multiplier^attempt
capped_backoff = min(backoff, max_backoff)
final_backoff = capped_backoff + jitter
```

## Data Flow

### Request Processing

```
1. MCP Client sends request
   ↓
2. Server receives on transport (stdio/sse/websocket)
   ↓
3. Middleware chain: Before phase
   ↓
4. HandlerRegistry: Lookup handler by name
   ↓
5. Handler: Execute with typed input
   ↓
6. Middleware chain: After phase
   ↓
7. Response serialized and sent to client
```

### Error Handling

```
Handler Error
   ↓
Middleware: on_error (reverse order)
   ↓
Error Tracker: Record error type
   ↓
Circuit Breaker: Update state
   ↓
Retry Logic: Attempt recovery if retryable
   ↓
Final error or recovery response
```

## Performance Characteristics

### Handler Dispatch

- **Lookup**: O(1) average case with FxHashMap
- **Registration**: O(1) insertion
- **Memory**: O(n) where n = number of handlers

### Middleware Chain

- **Execution**: O(m) where m = number of middlewares
- **Memory**: O(m) for middleware references

### State Management

**Sled**:
- **Read**: O(log n) tree lookup
- **Write**: O(log n) tree insertion
- **Memory**: Cached with configurable capacity

**Memory**:
- **Read**: O(1) with DashMap
- **Write**: O(1) with DashMap
- **Memory**: O(k) where k = number of keys

### Resource Matching

- **Compilation**: O(p) where p = pattern length
- **Matching**: O(r * u) where r = resources, u = URI length
- **Optimization**: Early exit on first match

## Concurrency Model

### Thread Safety

- **HandlerRegistry**: Immutable after registration (Arc)
- **StateManager**: Internal synchronization (RwLock, DashMap)
- **Middleware**: Stateless or internally synchronized
- **Circuit Breaker**: Atomic operations + RwLock

### Async Execution

- Uses Tokio runtime
- Non-blocking I/O for HTTP, state, CLI
- Efficient task scheduling
- Bounded concurrency where needed

## Memory Management

### Ownership

- Handlers: `Arc<dyn Handler>` for shared ownership
- Configuration: Owned by server, cloned when needed
- Requests/Responses: Moved through pipeline
- State values: `Vec<u8>` for zero-copy when possible

### Lifetimes

- No explicit lifetimes in public API
- Owned values preferred over references
- `'static` bounds for async traits

## Testing Strategy

### Unit Tests

- Test each component in isolation
- Mock external dependencies
- Property-based testing for generators
- 33+ unit tests in pforge-runtime

### Integration Tests

- Test cross-crate functionality
- Validate end-to-end workflows
- Configuration parsing → handler execution
- 12 comprehensive integration tests

### Performance Tests

- Benchmarks for critical paths
- Memory profiling
- Flamegraphs for CPU profiling

## Security Considerations

### Input Validation

- Schema validation for all inputs
- Type checking via Serde
- Size limits on requests
- Timeout enforcement

### State Management

- Key sanitization
- Access control (future)
- Audit logging (future)
- Encryption at rest (future)

### Error Handling

- No sensitive data in error messages
- Structured error types
- Error tracking without leaking internals

## Future Enhancements

### Planned Features

1. **Multi-Transport**: SSE, WebSocket support
2. **Language Bridges**: Python, Go, JavaScript FFI
3. **Hot Reload**: Configuration updates without restart
4. **Distributed State**: Redis backend
5. **Metrics**: Prometheus integration
6. **Tracing**: OpenTelemetry support

### Performance Improvements

1. **Handler Compilation**: Ahead-of-time code generation
2. **Connection Pooling**: Per-endpoint pools
3. **Caching**: Configurable response caching
4. **Batching**: Request batching for efficiency

## References

- [MCP Specification](https://modelcontextprotocol.io)
- [Rust Async Book](https://rust-lang.github.io/async-book/)
- [Tokio Documentation](https://tokio.rs)
- [Serde Guide](https://serde.rs)
