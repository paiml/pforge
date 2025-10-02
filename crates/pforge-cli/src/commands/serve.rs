use anyhow::{Context, Result};
use pforge_config::parse_config;
use pforge_runtime::McpServer;
use std::path::Path;

pub async fn execute(config_path: &str) -> Result<()> {
    println!("Starting pforge server...");
    println!("  Config: {}", config_path);

    // Parse configuration
    let config = parse_config(Path::new(config_path))
        .context("Failed to parse configuration")?;

    println!("  Server: {} v{}", config.forge.name, config.forge.version);
    println!("  Transport: {:?}", config.forge.transport);
    println!("  Tools: {}", config.tools.len());
    println!();

    // Create and run MCP server
    let server = McpServer::new(config);
    server.run().await.context("Server error")?;

    Ok(())
}
