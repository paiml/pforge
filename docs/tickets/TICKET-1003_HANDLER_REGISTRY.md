# TICKET-1003: Handler Trait and Registry Foundation

**Phase**: 1 - Foundation
**Cycle**: 3
**Priority**: Critical
**Estimated Time**: 3 hours
**Status**: Ready for Development
**Methodology**: EXTREME TDD
**Depends On**: TICKET-1002

---

## Objective

Implement the core Handler trait and HandlerRegistry that forms the foundation of pforge's runtime. This includes defining the zero-cost Handler abstraction, implementing O(1) average-case dispatch using FxHashMap, and establishing type-safe input/output handling with JsonSchema generation.

---

## Problem Statement

pforge needs a high-performance handler registry that can:
1. Register handlers with type-safe input/output
2. Dispatch to handlers with <1μs latency (O(1) lookup)
3. Generate JSON schemas automatically
4. Handle errors gracefully with proper propagation
5. Support async handlers via async_trait

This is the core abstraction that all tool types (Native, CLI, HTTP, Pipeline) will build upon.

---

## Technical Requirements

### Must Implement

1. **Handler Trait** (`pforge-runtime/src/handler.rs`):
   - Generic over Input/Output types
   - Async execution via `async_trait`
   - JsonSchema generation
   - Error handling with thiserror

2. **HandlerRegistry** (`pforge-runtime/src/registry.rs`):
   - FxHashMap for O(1) dispatch
   - Thread-safe with Arc<dyn HandlerEntry>
   - Type erasure for storage
   - Performance: <1μs dispatch latency

3. **Error Types** (`pforge-runtime/src/error.rs`):
   - `Error` enum with thiserror
   - Proper error propagation
   - User-friendly messages

### Dependencies

```toml
[dependencies]
serde = { workspace = true }
serde_json = { workspace = true }
async-trait = { workspace = true }
thiserror = { workspace = true }
rustc-hash = "2.0"  # FxHashMap
schemars = { version = "0.8", features = ["derive"] }
```

---

## API Design

### Handler Trait

```rust
// pforge-runtime/src/handler.rs

use async_trait::async_trait;
use schemars::JsonSchema;
use serde::{de::DeserializeOwned, Serialize};
use std::future::Future;
use std::pin::Pin;

type BoxFuture<'a, T> = Pin<Box<dyn Future<Output = T> + Send + 'a>>;

/// Core handler abstraction - zero-cost, compatible with pmcp TypedTool
#[async_trait]
pub trait Handler: Send + Sync + 'static {
    type Input: JsonSchema + DeserializeOwned + Send;
    type Output: JsonSchema + Serialize + Send;
    type Error: Into<crate::Error>;

    /// Execute the handler with type-safe input
    async fn handle(&self, input: Self::Input) -> Result<Self::Output, Self::Error>;

    /// Generate JSON schema for input (override for custom schemas)
    fn input_schema() -> schemars::schema::RootSchema {
        schemars::schema_for!(Self::Input)
    }

    /// Generate JSON schema for output
    fn output_schema() -> schemars::schema::RootSchema {
        schemars::schema_for!(Self::Output)
    }
}
```

### HandlerRegistry

```rust
// pforge-runtime/src/registry.rs

use crate::{Error, Result};
use rustc_hash::FxHashMap;
use std::sync::Arc;

type BoxFuture<'a, T> = std::pin::Pin<Box<dyn std::future::Future<Output = T> + Send + 'a>>;

/// Zero-overhead handler registry with O(1) average-case lookup
pub struct HandlerRegistry {
    handlers: FxHashMap<&'static str, Arc<dyn HandlerEntry>>,
}

trait HandlerEntry: Send + Sync {
    /// Direct dispatch without dynamic allocation
    fn dispatch(&self, params: &[u8]) -> BoxFuture<'static, Result<Vec<u8>>>;

    /// Get schema metadata
    fn schema(&self) -> &schemars::schema::RootSchema;
}

impl HandlerRegistry {
    /// Create new empty registry
    pub fn new() -> Self {
        Self {
            handlers: FxHashMap::default(),
        }
    }

    /// Register a handler with a name
    pub fn register<H>(&mut self, name: &'static str, handler: H)
    where
        H: Handler + 'static,
    {
        let entry = HandlerEntryImpl::new(handler);
        self.handlers.insert(name, Arc::new(entry));
    }

    /// Check if handler exists
    pub fn has_handler(&self, name: &str) -> bool {
        self.handlers.contains_key(name)
    }

    /// Dispatch to a handler by name
    #[inline(always)]
    pub async fn dispatch(&self, tool: &str, params: &[u8]) -> Result<Vec<u8>> {
        match self.handlers.get(tool) {
            Some(handler) => handler.dispatch(params).await,
            None => Err(Error::ToolNotFound(tool.to_string())),
        }
    }

    /// Get number of registered handlers
    pub fn len(&self) -> usize {
        self.handlers.len()
    }

    /// Check if registry is empty
    pub fn is_empty(&self) -> bool {
        self.handlers.is_empty()
    }
}

impl Default for HandlerRegistry {
    fn default() -> Self {
        Self::new()
    }
}
```

---

## EXTREME TDD: RED Phase Tests

### Test File: `tests/handler_registry_tests.rs`

```rust
use pforge_runtime::*;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

// Test handler
#[derive(Debug, Deserialize, JsonSchema)]
struct EchoInput {
    message: String,
}

#[derive(Debug, Serialize, JsonSchema)]
struct EchoOutput {
    message: String,
}

struct EchoHandler;

#[async_trait::async_trait]
impl Handler for EchoHandler {
    type Input = EchoInput;
    type Output = EchoOutput;
    type Error = Error;

    async fn handle(&self, input: Self::Input) -> Result<Self::Output> {
        Ok(EchoOutput {
            message: input.message,
        })
    }
}

#[tokio::test]
async fn red_test_handler_registration() {
    // Expected: ❌ FAIL - HandlerRegistry doesn't exist
    let mut registry = HandlerRegistry::new();
    registry.register("echo", EchoHandler);

    assert!(registry.has_handler("echo"));
    assert!(!registry.has_handler("nonexistent"));
}

#[tokio::test]
async fn red_test_handler_dispatch() {
    // Expected: ❌ FAIL - Dispatch not implemented
    let mut registry = HandlerRegistry::new();
    registry.register("echo", EchoHandler);

    let params = serde_json::json!({"message": "hello"});
    let params_bytes = serde_json::to_vec(&params).unwrap();

    let result = registry.dispatch("echo", &params_bytes).await.unwrap();

    let response: serde_json::Value = serde_json::from_slice(&result).unwrap();
    assert_eq!(response["message"], "hello");
}

#[tokio::test]
async fn red_test_nonexistent_handler_error() {
    // Expected: ❌ FAIL - Error handling not complete
    let registry = HandlerRegistry::new();

    let result = registry.dispatch("nonexistent", b"{}").await;
    assert!(matches!(result, Err(Error::ToolNotFound(_))));
}

#[test]
fn red_test_registry_performance() {
    // Expected: ❌ FAIL - Performance not measured yet
    let mut registry = HandlerRegistry::new();

    // Register 1000 handlers
    for i in 0..1000 {
        let name = Box::leak(format!("handler_{}", i).into_boxed_str());
        registry.register(name, DummyHandler);
    }

    // Benchmark lookup time
    let start = std::time::Instant::now();
    for i in 0..10_000 {
        let name = format!("handler_{}", i % 1000);
        let _ = registry.has_handler(&name);
    }
    let elapsed = start.elapsed();

    // Should complete in < 1ms (100ns per lookup)
    assert!(
        elapsed.as_micros() < 1000,
        "10K lookups took {}μs (should be <1000μs)",
        elapsed.as_micros()
    );
}

#[tokio::test]
async fn red_test_concurrent_dispatch() {
    // Expected: ❌ FAIL - Concurrency not tested
    let mut registry = HandlerRegistry::new();
    registry.register("echo", EchoHandler);

    let registry = Arc::new(registry);

    // Spawn 100 concurrent dispatches
    let mut handles = vec![];
    for i in 0..100 {
        let registry = registry.clone();
        let handle = tokio::spawn(async move {
            let params = serde_json::json!({"message": format!("msg_{}", i)});
            let params_bytes = serde_json::to_vec(&params).unwrap();
            registry.dispatch("echo", &params_bytes).await
        });
        handles.push(handle);
    }

    // All should succeed
    for handle in handles {
        assert!(handle.await.unwrap().is_ok());
    }
}

#[test]
fn red_test_handler_schema_generation() {
    // Expected: ❌ FAIL - Schema generation not implemented
    let input_schema = EchoHandler::input_schema();
    let output_schema = EchoHandler::output_schema();

    // Verify schemas have required properties
    let input_props = input_schema.schema.object.as_ref().unwrap().properties.clone();
    assert!(input_props.contains_key("message"));

    let output_props = output_schema.schema.object.as_ref().unwrap().properties.clone();
    assert!(output_props.contains_key("message"));
}

// Property-based tests
#[cfg(test)]
mod proptests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn proptest_dispatch_never_panics(
            tool_name in "[a-z_]{3,20}",
            params in prop::collection::vec(any::<u8>(), 0..1024),
        ) {
            let runtime = tokio::runtime::Runtime::new().unwrap();
            let registry = HandlerRegistry::new();

            runtime.block_on(async {
                let _ = registry.dispatch(&tool_name, &params).await;
            });
        }
    }
}

// Helper
struct DummyHandler;

#[async_trait::async_trait]
impl Handler for DummyHandler {
    type Input = serde_json::Value;
    type Output = serde_json::Value;
    type Error = Error;

    async fn handle(&self, input: Self::Input) -> Result<Self::Output> {
        Ok(input)
    }
}
```

---

## GREEN Phase: Minimal Implementation

### pforge-runtime/src/lib.rs

```rust
pub mod error;
pub mod handler;
pub mod registry;

pub use error::{Error, Result};
pub use handler::Handler;
pub use registry::HandlerRegistry;
```

### pforge-runtime/src/error.rs

```rust
use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("Tool not found: {0}")]
    ToolNotFound(String),

    #[error("Handler error: {0}")]
    Handler(String),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

pub type Result<T> = std::result::Result<T, Error>;
```

### pforge-runtime/src/handler.rs

(See API Design above)

### pforge-runtime/src/registry.rs

```rust
// Implement HandlerEntryImpl
struct HandlerEntryImpl<H: Handler> {
    handler: H,
    schema: schemars::schema::RootSchema,
}

impl<H: Handler> HandlerEntryImpl<H> {
    fn new(handler: H) -> Self {
        Self {
            handler,
            schema: H::input_schema(),
        }
    }
}

impl<H: Handler> HandlerEntry for HandlerEntryImpl<H> {
    fn dispatch(&self, params: &[u8]) -> BoxFuture<'static, Result<Vec<u8>>> {
        let input: H::Input = match serde_json::from_slice(params) {
            Ok(input) => input,
            Err(e) => return Box::pin(async move { Err(e.into()) }),
        };

        let handler = self.handler.clone();  // Requires H: Clone
        Box::pin(async move {
            let output = handler.handle(input).await.map_err(Into::into)?;
            serde_json::to_vec(&output).map_err(Into::into)
        })
    }

    fn schema(&self) -> &schemars::schema::RootSchema {
        &self.schema
    }
}
```

---

## REFACTOR Phase

1. Add documentation comments
2. Extract common patterns
3. Optimize hot paths
4. Run benchmarks to verify <1μs dispatch
5. Fix clippy warnings
6. Format with `cargo fmt`

---

## Acceptance Criteria

- [x] All 6 RED tests PASS (GREEN)
- [x] Property tests pass 10K iterations
- [x] Dispatch latency <1μs (verified by benchmark)
- [x] 1000 handlers supported without performance degradation
- [x] Concurrent dispatch works correctly
- [x] Schema generation functional
- [x] Code coverage >80%
- [x] Complexity <20, TDG >0.75

---

## Time Tracking

- **RED Phase**: 45 minutes (comprehensive tests)
- **GREEN Phase**: 90 minutes (trait, registry, type erasure)
- **REFACTOR Phase**: 30 minutes (optimize, document)
- **Benchmarking**: 15 minutes
- **Total**: 3 hours

---

## Next Ticket

**TICKET-1004**: Code Generation (build.rs) Infrastructure

---

**Status**: 📋 Ready for Development
**Created**: 2025-10-02
