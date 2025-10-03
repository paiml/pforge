mod handlers;

use pforge_config::parse_config;
use pforge_runtime::McpServer;
use std::path::Path;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load configuration from pforge.yaml
    let config = parse_config(Path::new("pforge.yaml"))?;

    eprintln!("╔═══════════════════════════════════════╗");
    eprintln!("║   Hello World MCP Server              ║");
    eprintln!("║   Powered by pforge v{}           ║", env!("CARGO_PKG_VERSION"));
    eprintln!("╚═══════════════════════════════════════╝");
    eprintln!();
    eprintln!("This example demonstrates:");
    eprintln!("  ✓ Native Rust handler (greet)");
    eprintln!("  ✓ CLI handler (whoami)");
    eprintln!("  ✓ Type-safe input/output");
    eprintln!("  ✓ Async execution");
    eprintln!("  ✓ YAML configuration");
    eprintln!();

    // Create MCP server from configuration
    let server = McpServer::new(config);

    // Get registry to register native handlers
    let registry = server.registry();
    {
        let mut reg = registry.write().await;
        reg.register("greet", handlers::greet::GreetHandler);
        eprintln!("Registered native handler: greet");
    }

    eprintln!();
    eprintln!("Available tools:");
    eprintln!("  • greet(name, greeting?) - Greet a person");
    eprintln!("  • whoami() - Get current username");
    eprintln!();

    // Run the server
    server.run().await?;

    Ok(())
}
