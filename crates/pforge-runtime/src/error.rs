use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("Tool not found: {0}")]
    ToolNotFound(String),

    #[error("Handler error: {0}")]
    Handler(String),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("HTTP error: {0}")]
    Http(String),

    #[error("Timeout error")]
    Timeout,
}

pub type Result<T> = std::result::Result<T, Error>;
