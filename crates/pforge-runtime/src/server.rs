use crate::{Error, HandlerRegistry, Result};
use pforge_config::ForgeConfig;
use std::sync::Arc;
use tokio::sync::RwLock;

/// MCP Server implementation
pub struct McpServer {
    config: ForgeConfig,
    registry: Arc<RwLock<HandlerRegistry>>,
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
                    ..
                } => {
                    use crate::handlers::cli::CliHandler;
                    let handler = CliHandler::new(
                        command.clone(),
                        args.clone(),
                        cwd.clone(),
                        env.clone(),
                        None, // timeout from tool config
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
                    );
                    registry.register(name, handler);
                    eprintln!("Registered HTTP handler: {}", name);
                }
                pforge_config::ToolDef::Pipeline { name, .. } => {
                    eprintln!("Note: Pipeline handler '{}' pending implementation", name);
                }
            }
        }

        Ok(())
    }

    /// Run the MCP server
    pub async fn run(&self) -> Result<()> {
        eprintln!(
            "Starting MCP server: {} v{}",
            self.config.forge.name, self.config.forge.version
        );
        eprintln!("Transport: {:?}", self.config.forge.transport);
        eprintln!("Tools registered: {}", self.config.tools.len());

        // Register handlers
        self.register_handlers().await?;

        // TODO: Implement actual MCP protocol loop
        // For now, just keep the server alive
        eprintln!("\n⚠ MCP protocol loop not yet implemented");
        eprintln!("Server configuration loaded and handlers registered successfully");
        eprintln!("Press Ctrl+C to exit");

        // Wait indefinitely (will be replaced with actual MCP loop)
        tokio::signal::ctrl_c().await.map_err(Error::Io)?;

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
            env: std::collections::HashMap::new(),
            stream: false,
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
            headers: std::collections::HashMap::new(),
            auth: None,
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
                fields: std::collections::HashMap::new(),
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
    async fn test_register_handlers_pipeline() {
        let mut config = create_test_config();
        config.tools.push(ToolDef::Pipeline {
            name: "test_pipeline".to_string(),
            description: "Test pipeline handler".to_string(),
            steps: vec![],
        });

        let server = McpServer::new(config);
        let result = server.register_handlers().await;

        // Should succeed (pipeline handler pending)
        assert!(result.is_ok());
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
            env: std::collections::HashMap::new(),
            stream: false,
        });

        config.tools.push(ToolDef::Http {
            name: "http1".to_string(),
            description: "HTTP 1".to_string(),
            endpoint: "https://example.com".to_string(),
            method: pforge_config::HttpMethod::Get,
            headers: std::collections::HashMap::new(),
            auth: None,
        });

        let server = McpServer::new(config);
        assert_eq!(server.config.tools.len(), 2);

        let result = server.register_handlers().await;
        assert!(result.is_ok());
    }
}
