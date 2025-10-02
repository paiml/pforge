use std::path::PathBuf;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("IO error reading {0}: {1}")]
    IoError(PathBuf, #[source] std::io::Error),

    #[error("Parse error: {0}")]
    ParseError(String),

    #[error("Duplicate tool name: {0}")]
    DuplicateToolName(String),

    #[error("Invalid handler path: {0}")]
    InvalidHandlerPath(String),

    #[error("Validation error: {0}")]
    ValidationError(String),
}

pub type Result<T> = std::result::Result<T, ConfigError>;
