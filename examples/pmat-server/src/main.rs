mod handlers;

use pforge_config::parse_config;
use pforge_runtime::McpServer;
use std::path::Path;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load configuration from pforge.yaml
    let config = parse_config(Path::new("pforge.yaml"))?;

    eprintln!("╔═══════════════════════════════════════╗");
    eprintln!("║   PMAT Analysis MCP Server            ║");
    eprintln!("║   Code Quality & Technical Debt       ║");
    eprintln!("║   Powered by pforge v{}           ║", env!("CARGO_PKG_VERSION"));
    eprintln!("╚═══════════════════════════════════════╝");
    eprintln!();
    eprintln!("This example demonstrates:");
    eprintln!("  ✓ CLI tool integration (pmat commands)");
    eprintln!("  ✓ Native handler with complex logic");
    eprintln!("  ✓ Multiple analysis tools");
    eprintln!("  ✓ Structured quality metrics");
    eprintln!("  ✓ JSON output formatting");
    eprintln!();

    // Create MCP server from configuration
    let server = McpServer::new(config);

    // Get registry to register native handlers
    let registry = server.registry();
    {
        let mut reg = registry.write().await;
        reg.register("metrics_summary", handlers::metrics::MetricsSummary);
        eprintln!("Registered native handler: metrics_summary");
    }

    eprintln!();
    eprintln!("Available tools:");
    eprintln!("  • analyze_complexity() - Check cyclomatic complexity");
    eprintln!("  • analyze_satd() - Detect technical debt comments");
    eprintln!("  • analyze_tdg() - Calculate Technical Debt Grade");
    eprintln!("  • analyze_cognitive() - Check cognitive complexity");
    eprintln!("  • metrics_summary(path, include_history?) - Full metrics report");
    eprintln!();

    // Run the server
    server.run().await?;

    Ok(())
}
