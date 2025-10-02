use crate::{ConfigError, ForgeConfig, Result, ToolDef};
use std::collections::HashSet;

pub fn validate_config(config: &ForgeConfig) -> Result<()> {
    // Check for duplicate tool names
    let mut tool_names = HashSet::new();
    for tool in &config.tools {
        let name = tool.name();
        if !tool_names.insert(name) {
            return Err(ConfigError::DuplicateToolName(name.to_string()));
        }
    }

    // Validate handler paths for native tools
    for tool in &config.tools {
        if let ToolDef::Native { handler, .. } = tool {
            validate_handler_path(&handler.path)?;
        }
    }

    Ok(())
}

fn validate_handler_path(path: &str) -> Result<()> {
    if path.is_empty() {
        return Err(ConfigError::InvalidHandlerPath("empty path".to_string()));
    }

    // Basic validation: should contain ::
    if !path.contains("::") {
        return Err(ConfigError::InvalidHandlerPath(format!(
            "invalid format: {} (expected format: module::function)",
            path
        )));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::*;

    #[test]
    fn test_validate_config_success() {
        let config = ForgeConfig {
            forge: ForgeMetadata {
                name: "test".to_string(),
                version: "1.0.0".to_string(),
                transport: TransportType::Stdio,
                optimization: OptimizationLevel::Debug,
            },
            tools: vec![ToolDef::Native {
                name: "tool1".to_string(),
                description: "Tool 1".to_string(),
                handler: HandlerRef {
                    path: "module::handler".to_string(),
                    inline: None,
                },
                params: ParamSchema {
                    fields: std::collections::HashMap::new(),
                },
                timeout_ms: None,
            }],
            resources: vec![],
            prompts: vec![],
            state: None,
        };

        assert!(validate_config(&config).is_ok());
    }

    #[test]
    fn test_validate_config_duplicate_tools() {
        let config = ForgeConfig {
            forge: ForgeMetadata {
                name: "test".to_string(),
                version: "1.0.0".to_string(),
                transport: TransportType::Stdio,
                optimization: OptimizationLevel::Debug,
            },
            tools: vec![
                ToolDef::Native {
                    name: "duplicate".to_string(),
                    description: "Tool 1".to_string(),
                    handler: HandlerRef {
                        path: "module::handler1".to_string(),
                        inline: None,
                    },
                    params: ParamSchema {
                        fields: std::collections::HashMap::new(),
                    },
                    timeout_ms: None,
                },
                ToolDef::Native {
                    name: "duplicate".to_string(),
                    description: "Tool 2".to_string(),
                    handler: HandlerRef {
                        path: "module::handler2".to_string(),
                        inline: None,
                    },
                    params: ParamSchema {
                        fields: std::collections::HashMap::new(),
                    },
                    timeout_ms: None,
                },
            ],
            resources: vec![],
            prompts: vec![],
            state: None,
        };

        let result = validate_config(&config);
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            ConfigError::DuplicateToolName(_)
        ));
    }

    #[test]
    fn test_validate_handler_path_empty() {
        let result = validate_handler_path("");
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            ConfigError::InvalidHandlerPath(_)
        ));
    }

    #[test]
    fn test_validate_handler_path_invalid_format() {
        let result = validate_handler_path("invalid_path");
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            ConfigError::InvalidHandlerPath(_)
        ));
    }

    #[test]
    fn test_validate_handler_path_valid() {
        let result = validate_handler_path("module::handler");
        assert!(result.is_ok());
    }
}
