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
