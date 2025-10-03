// Example: Middleware Chain Demonstration
//
// Run with: cargo run --example middleware_demo

use pforge_runtime::{Handler, LoggingMiddleware, Middleware, Result};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
struct EchoInput {
    message: String,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
struct EchoOutput {
    echo: String,
    length: usize,
}

struct EchoHandler;

#[async_trait::async_trait]
impl Handler for EchoHandler {
    type Input = EchoInput;
    type Output = EchoOutput;
    type Error = pforge_runtime::Error;

    async fn handle(&self, input: Self::Input) -> Result<Self::Output> {
        Ok(EchoOutput {
            length: input.message.len(),
            echo: input.message,
        })
    }
}

// Custom timing middleware
struct TimingMiddleware {
    start: std::time::Instant,
}

impl TimingMiddleware {
    fn new() -> Self {
        Self {
            start: std::time::Instant::now(),
        }
    }
}

#[async_trait::async_trait]
impl Middleware for TimingMiddleware {
    async fn before(&self, request: Value) -> Result<Value> {
        println!("  ⏱️  Request started");
        Ok(request)
    }

    async fn after(&self, _request: Value, response: Value) -> Result<Value> {
        let elapsed = self.start.elapsed();
        println!("  ⏱️  Request completed in {:?}", elapsed);
        Ok(response)
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    println!("🔗 Middleware Example\n");

    // Create middlewares
    let logging_mw = LoggingMiddleware::new("echo");
    let timing_mw = TimingMiddleware::new();

    println!("✅ Created middlewares:");
    println!("   1. LoggingMiddleware - logs requests/responses");
    println!("   2. TimingMiddleware - tracks request duration\n");

    // Test request
    println!("📤 Processing request through middlewares:\n");

    let input = EchoInput {
        message: "Hello from middleware!".to_string(),
    };

    let mut request = serde_json::to_value(&input)?;

    // Before phase
    println!("1. LoggingMiddleware.before()");
    request = logging_mw.before(request).await?;

    println!("2. TimingMiddleware.before()");
    request = timing_mw.before(request).await?;

    // Handler execution
    println!("3. Execute handler");
    let handler_input: EchoInput = serde_json::from_value(request.clone())?;
    let handler = EchoHandler;
    let handler_output = handler.handle(handler_input).await?;
    let mut response = serde_json::to_value(&handler_output)?;

    // After phase (reverse order)
    println!("4. TimingMiddleware.after()");
    response = timing_mw.after(request.clone(), response).await?;

    println!("5. LoggingMiddleware.after()");
    response = logging_mw.after(request, response).await?;

    println!("\n📥 Final response:");
    println!("{}", serde_json::to_string_pretty(&response)?);

    println!("\n✨ Example complete!\n");

    Ok(())
}
