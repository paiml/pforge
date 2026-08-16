use anyhow::{Context, Result};
use pforge_config::parse_config;
use pforge_runtime::McpServer;
use std::path::Path;

pub async fn execute(config_path: &str) -> Result<()> {
    // Progress goes to STDERR, never stdout.
    //
    // On a stdio transport stdout IS the MCP channel: the client reads
    // JSON-RPC frames from it line by line. These five lines used to be
    // `println!`, so every `pforge serve` opened by writing
    //   Starting pforge server... / Config: ... / Server: ... / Transport: ... / Tools: N
    // into the protocol stream ahead of the first frame. A tolerant client
    // skips them; a strict one fails to parse and reports the server as
    // broken. `McpServer::run` already used `eprintln!` for exactly this
    // reason — this path did not.
    eprintln!("Starting pforge server...");
    eprintln!("  Config: {}", config_path);

    // Parse configuration
    let config = parse_config(Path::new(config_path)).context("Failed to parse configuration")?;

    eprintln!("  Server: {} v{}", config.forge.name, config.forge.version);
    eprintln!("  Transport: {:?}", config.forge.transport);
    eprintln!("  Tools: {}", config.tools.len());
    eprintln!();

    // Create and run MCP server
    let server = McpServer::new(config);
    server.run().await.context("Server error")?;

    Ok(())
}
