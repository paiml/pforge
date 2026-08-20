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

    #[error("State error: {0}")]
    StateError(String),

    #[error("Timeout error")]
    Timeout,
}

impl Error {
    /// Error for functionality whose cargo feature was compiled out.
    ///
    /// `sse`, `websocket` and `http-handlers` are optional, but a config naming
    /// them still PARSES with them off — the config types are feature-independent,
    /// and they have to be, or the same `forge.yaml` would mean different things
    /// to different builds of the same version.
    ///
    /// So the failure belongs here, at construction, naming the missing feature
    /// and how to get it back. The tempting alternatives are worse: a panic turns
    /// a configuration mistake into a crash with no remedy in the message, and
    /// silently skipping the thing leaves a server that starts, reports healthy,
    /// and does not serve what its operator configured.
    pub fn feature_disabled(feature: &str, subject: &str) -> Self {
        Error::Handler(format!(
            "{subject} is unavailable: this binary was built without the `{feature}` \
             cargo feature. Rebuild with `--features {feature}` (or with default \
             features) to enable it."
        ))
    }
}

pub type Result<T> = std::result::Result<T, Error>;
