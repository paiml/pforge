//! # pforge-runtime
//!
//! Core runtime library for pforge - a zero-boilerplate MCP server framework.
//!
//! This crate provides the execution engine for MCP servers defined via YAML configuration,
//! including handler registration, dispatch, middleware, state management, and fault tolerance.
//!
//! ## Quick Start
//!
//! ```rust
//! use pforge_runtime::{Handler, HandlerRegistry, Result};
//! use serde::{Deserialize, Serialize};
//! use schemars::JsonSchema;
//!
//! // Define input/output types
//! #[derive(Debug, Deserialize, JsonSchema)]
//! struct GreetInput {
//!     name: String,
//! }
//!
//! #[derive(Debug, Serialize, JsonSchema)]
//! struct GreetOutput {
//!     message: String,
//! }
//!
//! // Implement handler
//! struct GreetHandler;
//!
//! #[async_trait::async_trait]
//! impl Handler for GreetHandler {
//!     type Input = GreetInput;
//!     type Output = GreetOutput;
//!     type Error = pforge_runtime::Error;
//!
//!     async fn handle(&self, input: Self::Input) -> Result<Self::Output> {
//!         Ok(GreetOutput {
//!             message: format!("Hello, {}!", input.name),
//!         })
//!     }
//! }
//!
//! # #[tokio::main]
//! # async fn main() -> Result<()> {
//! // Register and dispatch
//! let mut registry = HandlerRegistry::new();
//! registry.register("greet", GreetHandler);
//!
//! let input = serde_json::json!({"name": "World"});
//! let input_bytes = serde_json::to_vec(&input)?;
//! let output_bytes = registry.dispatch("greet", &input_bytes).await?;
//! let output: serde_json::Value = serde_json::from_slice(&output_bytes)?;
//!
//! assert_eq!(output["message"], "Hello, World!");
//! # Ok(())
//! # }
//! ```
//!
//! ## Features
//!
//! - **Zero-overhead dispatch**: O(1) average-case handler lookup with FxHash
//! - **Type safety**: Full compile-time type checking with Serde + JsonSchema
//! - **Async-first**: Built on tokio for high-performance async execution
//! - **Fault tolerance**: Circuit breaker, retry with exponential backoff, timeouts
//! - **State management**: Persistent (Sled) and in-memory backends with TTL support
//! - **Middleware**: Composable request/response processing chain
//! - **MCP protocol**: Full support for resources, prompts, and tools

pub mod error;
pub mod handler;
pub mod handlers;
pub mod middleware;
pub mod prompt;
pub mod recovery;
pub mod registry;
pub mod resource;
pub mod server;
pub mod state;
pub mod telemetry;
pub mod timeout;
pub mod transport;

pub use error::{Error, Result};
pub use handler::Handler;
pub use handlers::{CliHandler, HttpHandler, PipelineHandler};
pub use middleware::{LoggingMiddleware, Middleware, MiddlewareChain, ValidationMiddleware};
pub use prompt::{PromptManager, PromptMetadata};
pub use recovery::{
    CircuitBreaker, CircuitBreakerConfig, CircuitState, ErrorTracker, FallbackHandler,
    RecoveryMiddleware,
};
pub use registry::HandlerRegistry;
pub use resource::{ResourceHandler, ResourceManager};
pub use server::McpServer;
pub use state::{MemoryStateManager, SledStateManager, StateManager};
pub use telemetry::{
    ComponentHealth, HealthCheck, HealthStatus, MetricsCollector, TelemetryMiddleware,
};
pub use timeout::{
    retry_with_policy, with_timeout, RetryMiddleware, RetryPolicy, TimeoutMiddleware,
};
