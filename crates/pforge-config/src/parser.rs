use crate::{ConfigError, ForgeConfig, Result};
use std::path::Path;

pub fn parse_config(path: &Path) -> Result<ForgeConfig> {
    let content = std::fs::read_to_string(path)
        .map_err(|e| ConfigError::IoError(path.to_path_buf(), e))?;

    parse_config_from_str(&content)
}

pub fn parse_config_from_str(yaml: &str) -> Result<ForgeConfig> {
    serde_yml::from_str(yaml).map_err(|e| ConfigError::ParseError(e.to_string()))
}
