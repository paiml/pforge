use crate::{Error, HandlerRegistry, Result};
use async_trait::async_trait;
use pforge_config::ForgeConfig;
use pmcp::server::ToolHandler;
use serde_json::Value;
use std::sync::Arc;
use tokio::sync::RwLock;

/// MCP Server implementation
pub struct McpServer {
    config: ForgeConfig,
    registry: Arc<RwLock<HandlerRegistry>>,
}

/// Adapter to wrap pforge handlers as pmcp ToolHandler
struct PforgeToolAdapter {
    registry: Arc<RwLock<HandlerRegistry>>,
    tool_name: String,
    description: Option<String>,
}

#[async_trait]
impl ToolHandler for PforgeToolAdapter {
    async fn handle(
        &self,
        args: Value,
        _extra: pmcp::server::cancellation::RequestHandlerExtra,
    ) -> pmcp::Result<Value> {
        // Serialize args to bytes for pforge dispatch
        let params = serde_json::to_vec(&args)
            .map_err(|e| pmcp::Error::protocol_msg(format!("Failed to serialize args: {}", e)))?;

        let registry = self.registry.read().await;
        let result_bytes = registry
            .dispatch(&self.tool_name, &params)
            .await
            .map_err(|e| pmcp::Error::protocol_msg(e.to_string()))?;

        // Deserialize result back to Value
        let result: Value = serde_json::from_slice(&result_bytes).map_err(|e| {
            pmcp::Error::protocol_msg(format!("Failed to deserialize result: {}", e))
        })?;

        Ok(result)
    }

    fn metadata(&self) -> Option<pmcp::types::ToolInfo> {
        // Try to get actual schema from registry (may fail if lock is held)
        let input_schema = if let Ok(guard) = self.registry.try_read() {
            if let Some(schema) = guard.get_input_schema(&self.tool_name) {
                // Convert RootSchema to serde_json::Value
                serde_json::to_value(&schema).unwrap_or_else(|_| {
                    serde_json::json!({
                        "type": "object",
                        "properties": {}
                    })
                })
            } else {
                serde_json::json!({
                    "type": "object",
                    "properties": {}
                })
            }
        } else {
            // Fallback if lock unavailable
            serde_json::json!({
                "type": "object",
                "properties": {}
            })
        };

        Some(pmcp::types::ToolInfo::new(
            self.tool_name.clone(),
            self.description.clone(),
            input_schema,
        ))
    }
}

impl McpServer {
    /// Create a new MCP server from configuration
    pub fn new(config: ForgeConfig) -> Self {
        Self {
            config,
            registry: Arc::new(RwLock::new(HandlerRegistry::new())),
        }
    }

    /// Register all handlers from configuration
    pub async fn register_handlers(&self) -> Result<()> {
        let mut registry = self.registry.write().await;

        for tool in &self.config.tools {
            match tool {
                pforge_config::ToolDef::Native { name, .. } => {
                    // Native handlers will be registered by generated code
                    eprintln!(
                        "Note: Native handler '{}' requires handler implementation",
                        name
                    );
                }
                pforge_config::ToolDef::Cli {
                    name,
                    command,
                    args,
                    cwd,
                    env,
                    stream,
                    timeout_ms,
                    ..
                } => {
                    use crate::handlers::cli::CliHandler;
                    let handler = CliHandler::new(
                        command.clone(),
                        args.clone(),
                        cwd.clone(),
                        env.clone(),
                        *timeout_ms,
                        *stream,
                    );
                    registry.register(name, handler);
                    eprintln!("Registered CLI handler: {}", name);
                }
                pforge_config::ToolDef::Http {
                    name,
                    endpoint,
                    method,
                    headers,
                    auth,
                    timeout_ms,
                    ..
                } => {
                    use crate::handlers::http::{
                        AuthConfig as HttpAuthConfig, HttpHandler, HttpMethod as HandlerHttpMethod,
                    };

                    let handler_method = match method {
                        pforge_config::HttpMethod::Get => HandlerHttpMethod::Get,
                        pforge_config::HttpMethod::Post => HandlerHttpMethod::Post,
                        pforge_config::HttpMethod::Put => HandlerHttpMethod::Put,
                        pforge_config::HttpMethod::Delete => HandlerHttpMethod::Delete,
                        pforge_config::HttpMethod::Patch => HandlerHttpMethod::Patch,
                    };

                    let handler_auth = auth.as_ref().map(|a| match a {
                        pforge_config::AuthConfig::Bearer { token } => HttpAuthConfig::Bearer {
                            token: token.clone(),
                        },
                        pforge_config::AuthConfig::Basic { username, password } => {
                            HttpAuthConfig::Basic {
                                username: username.clone(),
                                password: password.clone(),
                            }
                        }
                        pforge_config::AuthConfig::ApiKey { key, header } => {
                            HttpAuthConfig::ApiKey {
                                key: key.clone(),
                                header: header.clone(),
                            }
                        }
                    });

                    let handler = HttpHandler::new(
                        endpoint.clone(),
                        handler_method,
                        headers.clone(),
                        handler_auth,
                        *timeout_ms,
                    );
                    registry.register(name, handler);
                    eprintln!("Registered HTTP handler: {}", name);
                }
                pforge_config::ToolDef::Pipeline { name, steps, .. } => {
                    use crate::handlers::pipeline::PipelineHandlerAdapter;
                    let handler =
                        PipelineHandlerAdapter::from_config_steps(steps, self.registry.clone());
                    registry.register(name, handler);
                    eprintln!("Registered Pipeline handler: {}", name);
                }
            }
        }

        Ok(())
    }

    /// Run the MCP server using pmcp protocol implementation
    pub async fn run(&self) -> Result<()> {
        eprintln!(
            "Starting MCP server: {} v{}",
            self.config.forge.name, self.config.forge.version
        );
        eprintln!("Transport: {:?}", self.config.forge.transport);
        eprintln!("Tools registered: {}", self.config.tools.len());

        // Register handlers in pforge registry
        self.register_handlers().await?;

        // Build pmcp server with tool adapters
        let mut builder = pmcp::Server::builder()
            .name(&self.config.forge.name)
            .version(&self.config.forge.version);

        // Add tool adapters for each registered tool
        for tool in &self.config.tools {
            let (tool_name, description) = match tool {
                pforge_config::ToolDef::Native {
                    name, description, ..
                } => (name.clone(), Some(description.clone())),
                pforge_config::ToolDef::Cli {
                    name, description, ..
                } => (name.clone(), Some(description.clone())),
                pforge_config::ToolDef::Http {
                    name, description, ..
                } => (name.clone(), Some(description.clone())),
                pforge_config::ToolDef::Pipeline {
                    name, description, ..
                } => (name.clone(), Some(description.clone())),
            };

            let adapter = PforgeToolAdapter {
                registry: self.registry.clone(),
                tool_name: tool_name.clone(),
                description,
            };
            builder = builder.tool(&tool_name, adapter);
        }

        let server = builder
            .build()
            .map_err(|e| Error::Handler(format!("Failed to build MCP server: {}", e)))?;

        eprintln!("MCP server ready, starting protocol loop...");

        // Run the server with appropriate transport
        match self.config.forge.transport {
            pforge_config::TransportType::Stdio => {
                server
                    .run_stdio()
                    .await
                    .map_err(|e| Error::Handler(format!("MCP server error: {}", e)))?;
            }
            pforge_config::TransportType::Sse => {
                // See transport.rs for why this is not migrated to
                // StreamableHttpTransport: it is a wire-protocol change, not a
                // rename.
                #[allow(deprecated)]
                use pmcp::shared::{OptimizedSseConfig, OptimizedSseTransport};
                use std::time::Duration;

                let config = OptimizedSseConfig {
                    url: "http://localhost:8080/sse".to_string(),
                    connection_timeout: Duration::from_secs(30),
                    keepalive_interval: Duration::from_secs(15),
                    max_reconnects: 5,
                    reconnect_delay: Duration::from_secs(1),
                    buffer_size: 100,
                    flush_interval: Duration::from_millis(100),
                    enable_pooling: true,
                    max_connections: 10,
                    enable_compression: false,
                };
                #[allow(deprecated)]
                // see transport.rs: SSE -> StreamableHttp is a wire-protocol change
                let transport = OptimizedSseTransport::new(config);
                server
                    .run(transport)
                    .await
                    .map_err(|e| Error::Handler(format!("MCP server error: {}", e)))?;
            }
            pforge_config::TransportType::WebSocket => {
                use pmcp::shared::{WebSocketConfig, WebSocketTransport};
                use std::time::Duration;

                let url = "ws://localhost:8080/ws"
                    .parse()
                    .map_err(|e| Error::Handler(format!("Invalid WebSocket URL: {}", e)))?;
                let config = WebSocketConfig {
                    url,
                    auto_reconnect: true,
                    reconnect_delay: Duration::from_secs(1),
                    max_reconnect_delay: Duration::from_secs(30),
                    max_reconnect_attempts: Some(5),
                    ping_interval: Some(Duration::from_secs(30)),
                    request_timeout: Duration::from_secs(10),
                };
                let transport = WebSocketTransport::new(config);
                server
                    .run(transport)
                    .await
                    .map_err(|e| Error::Handler(format!("MCP server error: {}", e)))?;
            }
        }

        eprintln!("\nShutting down...");
        Ok(())
    }

    /// Get the handler registry (for testing)
    pub fn registry(&self) -> Arc<RwLock<HandlerRegistry>> {
        self.registry.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pforge_config::{ForgeMetadata, ParamSchema, ToolDef, TransportType};

    fn create_test_config() -> ForgeConfig {
        ForgeConfig {
            forge: ForgeMetadata {
                name: "test-server".to_string(),
                version: "0.1.0".to_string(),
                transport: TransportType::Stdio,
                optimization: pforge_config::OptimizationLevel::Debug,
            },
            tools: vec![],
            resources: vec![],
            prompts: vec![],
            state: None,
        }
    }

    #[test]
    fn test_server_new() {
        let config = create_test_config();
        let server = McpServer::new(config);

        assert_eq!(server.config.forge.name, "test-server");
        assert_eq!(server.config.forge.version, "0.1.0");
    }

    #[tokio::test]
    async fn test_register_handlers_cli() {
        let mut config = create_test_config();
        config.tools.push(ToolDef::Cli {
            name: "test_cli".to_string(),
            description: "Test CLI handler".to_string(),
            command: "echo".to_string(),
            args: vec!["hello".to_string()],
            cwd: None,
            env: rustc_hash::FxHashMap::default(),
            stream: false,
            timeout_ms: None,
        });

        let server = McpServer::new(config);
        let result = server.register_handlers().await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_register_handlers_http() {
        let mut config = create_test_config();
        config.tools.push(ToolDef::Http {
            name: "test_http".to_string(),
            description: "Test HTTP handler".to_string(),
            endpoint: "https://api.example.com".to_string(),
            method: pforge_config::HttpMethod::Get,
            headers: rustc_hash::FxHashMap::default(),
            auth: None,
            timeout_ms: None,
        });

        let server = McpServer::new(config);
        let result = server.register_handlers().await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_register_handlers_native() {
        let mut config = create_test_config();
        config.tools.push(ToolDef::Native {
            name: "test_native".to_string(),
            description: "Test native handler".to_string(),
            handler: pforge_config::HandlerRef {
                path: "handlers::test::TestHandler".to_string(),
                inline: None,
            },
            params: ParamSchema {
                fields: rustc_hash::FxHashMap::default(),
            },
            timeout_ms: Some(5000),
        });

        let server = McpServer::new(config);
        let result = server.register_handlers().await;

        // Should succeed (native handlers registered by generated code)
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_registry_access() {
        let config = create_test_config();
        let server = McpServer::new(config);

        let registry = server.registry();
        let _lock = registry.read().await;

        // Registry is accessible (test passes if no panic)
    }

    #[tokio::test]
    async fn test_registry_returns_actual_registry() {
        // This test catches mutation: registry() returning a new empty registry
        let mut config = create_test_config();
        config.tools.push(ToolDef::Cli {
            name: "test_cli".to_string(),
            description: "Test CLI".to_string(),
            command: "echo".to_string(),
            args: vec!["test".to_string()],
            cwd: None,
            env: rustc_hash::FxHashMap::default(),
            stream: false,
            timeout_ms: None,
        });

        let server = McpServer::new(config);
        server.register_handlers().await.unwrap();

        // Get registry and verify the handler is registered
        let registry = server.registry();
        let reg = registry.read().await;

        // The CLI handler should be registered - verify via len
        assert_eq!(reg.len(), 1, "Registry should contain registered handler");
    }

    #[tokio::test]
    async fn test_register_handlers_pipeline() {
        let mut config = create_test_config();
        config.tools.push(ToolDef::Pipeline {
            name: "test_pipeline".to_string(),
            description: "Test pipeline handler".to_string(),
            steps: vec![],
        });

        let server = McpServer::new(config);
        let result = server.register_handlers().await;
        assert!(result.is_ok());

        // Verify the pipeline is actually registered
        let registry = server.registry();
        let reg = registry.read().await;
        assert_eq!(reg.len(), 1, "Pipeline handler should be registered");
    }

    #[tokio::test]
    async fn test_server_with_multiple_tools() {
        let mut config = create_test_config();

        config.tools.push(ToolDef::Cli {
            name: "cli1".to_string(),
            description: "CLI 1".to_string(),
            command: "echo".to_string(),
            args: vec![],
            cwd: None,
            env: rustc_hash::FxHashMap::default(),
            stream: false,
            timeout_ms: None,
        });

        config.tools.push(ToolDef::Http {
            name: "http1".to_string(),
            description: "HTTP 1".to_string(),
            endpoint: "https://example.com".to_string(),
            method: pforge_config::HttpMethod::Get,
            headers: rustc_hash::FxHashMap::default(),
            auth: None,
            timeout_ms: None,
        });

        let server = McpServer::new(config);
        assert_eq!(server.config.tools.len(), 2);

        let result = server.register_handlers().await;
        assert!(result.is_ok());
    }
}
