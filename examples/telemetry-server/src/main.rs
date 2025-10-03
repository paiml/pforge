mod handlers;

use pforge_config::parse_config;
use pforge_runtime::{HealthCheck, McpServer, MetricsCollector};
use std::path::Path;
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize structured logging
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .json()
        .init();

    tracing::info!("Starting telemetry-server");

    // Parse configuration
    let config = parse_config(Path::new("pforge.yaml"))?;

    // Create shared metrics collector and health check
    let metrics = Arc::new(MetricsCollector::new());
    let health = Arc::new(HealthCheck::new());

    // Register initial health status
    health.register_component("server", pforge_runtime::HealthStatus::Healthy);

    // Create MCP server
    let server = McpServer::new(config);
    let registry = server.registry();

    // Register handlers with shared observability components
    {
        let mut reg = registry.write().await;

        reg.register(
            "get_metrics",
            handlers::metrics::GetMetricsHandler {
                collector: metrics.clone(),
            },
        );

        reg.register(
            "get_health",
            handlers::health::GetHealthHandler {
                health: health.clone(),
            },
        );

        reg.register(
            "set_component_health",
            handlers::health::SetComponentHealthHandler {
                health: health.clone(),
            },
        );

        reg.register("echo", handlers::echo::EchoHandler);
        reg.register("error_test", handlers::echo::ErrorTestHandler);
    }

    tracing::info!("Handlers registered, starting server");

    // TODO: Integrate metrics collection into dispatch loop
    // This would require middleware support in the server

    server.run().await?;

    Ok(())
}
