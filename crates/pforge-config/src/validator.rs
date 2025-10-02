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
