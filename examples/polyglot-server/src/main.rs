mod handlers;

use pforge_config::parse_config;
use pforge_runtime::McpServer;
use std::path::Path;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load configuration from pforge.yaml
    let config = parse_config(Path::new("pforge.yaml"))?;

    eprintln!("╔═══════════════════════════════════════╗");
    eprintln!("║   Polyglot MCP Server                 ║");
    eprintln!("║   Rust + Python + Go                  ║");
    eprintln!("║   Powered by pforge v{}           ║", env!("CARGO_PKG_VERSION"));
    eprintln!("╚═══════════════════════════════════════╝");
    eprintln!();
    eprintln!("This example demonstrates:");
    eprintln!("  ✓ Rust native handler (Fibonacci)");
    eprintln!("  ✓ Python handler via bridge (Sentiment Analysis)");
    eprintln!("  ✓ Go handler via bridge (Cryptographic Hash)");
    eprintln!("  ✓ CLI handler (System Info)");
    eprintln!("  ✓ Polyglot pipeline");
    eprintln!();

    // Create MCP server from configuration
    let server = McpServer::new(config);

    // Get registry to register native handlers
    let registry = server.registry();
    {
        let mut reg = registry.write().await;
        reg.register("rust_fibonacci", handlers::fibonacci::FibonacciHandler);
        eprintln!("Registered Rust handler: rust_fibonacci");

        reg.register("python_sentiment", handlers::python_bridge::PythonSentimentHandler);
        eprintln!("Registered Python handler: python_sentiment");

        reg.register("go_hash", handlers::go_bridge::GoHashHandler);
        eprintln!("Registered Go handler: go_hash");
    }

    eprintln!();
    eprintln!("Available tools:");
    eprintln!("  • rust_fibonacci(n) - Calculate Fibonacci (Rust)");
    eprintln!("  • python_sentiment(text, language?) - Analyze sentiment (Python)");
    eprintln!("  • go_hash(data, algorithm?) - Calculate hash (Go)");
    eprintln!("  • system_info() - Get system info (CLI)");
    eprintln!("  • polyglot_pipeline() - Pipeline using all languages");
    eprintln!();

    eprintln!("Note: Python and Go handlers require:");
    eprintln!("  - Python 3: python3 available in PATH");
    eprintln!("  - Go binary: compile with 'cd src/go && go build hasher.go'");
    eprintln!();

    // Run the server
    server.run().await?;

    Ok(())
}
