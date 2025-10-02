mod handlers;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Hello World MCP Server v0.1.0");
    println!("This example demonstrates:");
    println!("- Native Rust handler (greet)");
    println!("- Type-safe input/output");
    println!("- Async execution");
    println!();
    println!("Available tools:");
    println!("  greet(name, greeting?) - Greet a person");
    println!();
    println!("Try it with an MCP client!");

    // In a real server, you would:
    // 1. Create HandlerRegistry
    // 2. Register handlers
    // 3. Create McpServer with config
    // 4. Run server.run().await

    // For now, demonstrate handler works
    let handler = handlers::greet::GreetHandler;
    let input = handlers::greet::GreetInput {
        name: "pforge".to_string(),
        greeting: "Welcome to".to_string(),
    };

    let result = pforge_runtime::Handler::handle(&handler, input).await?;
    println!("Test: {}", result.message);

    Ok(())
}
