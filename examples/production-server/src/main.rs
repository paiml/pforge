mod handlers;

use pforge_config::parse_config;
use pforge_runtime::{McpServer, MemoryStateManager};
use std::path::Path;
use std::sync::Arc;
use tracing::{info, warn};
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize structured logging
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| EnvFilter::new("production_server=info"))
        )
        .init();

    info!("Starting Production MCP Server v1.0.0");

    // Load configuration
    let config = parse_config(Path::new("pforge.yaml"))?;
    info!("Configuration loaded successfully");

    eprintln!("╔═══════════════════════════════════════╗");
    eprintln!("║   Production MCP Server v1.0.0        ║");
    eprintln!("║   Full Feature Showcase               ║");
    eprintln!("║   Powered by pforge v{}           ║", env!("CARGO_PKG_VERSION"));
    eprintln!("╚═══════════════════════════════════════╝");
    eprintln!();
    eprintln!("Production Features:");
    eprintln!("  ✓ State management (memory backend)");
    eprintln!("  ✓ Native handlers with validation");
    eprintln!("  ✓ CLI handlers with streaming");
    eprintln!("  ✓ HTTP handlers with auth");
    eprintln!("  ✓ Pipeline workflows");
    eprintln!("  ✓ Resources and prompts");
    eprintln!("  ✓ Structured logging");
    eprintln!("  ✓ Comprehensive error handling");
    eprintln!();

    // Create MCP server
    let server = McpServer::new(config);

    // Initialize state manager
    let state = Arc::new(MemoryStateManager::new());
    info!("State manager initialized");

    // Register native handlers
    let registry = server.registry();
    {
        let mut reg = registry.write().await;

        reg.register(
            "counter_increment",
            handlers::counter::CounterHandler::new(state.clone())
        );
        info!("Registered handler: counter_increment");

        reg.register("data_processor", handlers::processor::DataProcessor);
        info!("Registered handler: data_processor");
    }

    eprintln!("Available tools:");
    eprintln!("  • counter_increment(name, increment?) - Stateful counter");
    eprintln!("  • data_processor(data, format?) - Data validation & formatting");
    eprintln!("  • log_stream() - Real-time log streaming (CLI)");
    eprintln!("  • api_fetch() - GitHub API integration (HTTP)");
    eprintln!("  • full_workflow() - Complete pipeline");
    eprintln!();
    eprintln!("Resources:");
    eprintln!("  • server_documentation - README.md");
    eprintln!("  • server_config - config.json");
    eprintln!();
    eprintln!("Prompts:");
    eprintln!("  • generate_report - Server status report");
    eprintln!("  • troubleshoot - Error troubleshooting");
    eprintln!();

    warn!("Note: Some handlers require specific setup:");
    eprintln!("  - log_stream: requires journalctl (Linux only)");
    eprintln!("  - api_fetch: requires internet connection");
    eprintln!();

    info!("Server ready, starting MCP protocol loop");

    // Run the server
    server.run().await?;

    info!("Server shutdown complete");
    Ok(())
}
