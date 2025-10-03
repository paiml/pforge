use pforge_runtime::{Handler, Result};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, JsonSchema)]
pub struct EchoInput {
    pub message: String,
    #[serde(default)]
    pub delay_ms: Option<u64>,
}

#[derive(Debug, Serialize, JsonSchema)]
pub struct EchoOutput {
    pub message: String,
    pub length: usize,
    pub delayed_ms: u64,
}

pub struct EchoHandler;

#[async_trait::async_trait]
impl Handler for EchoHandler {
    type Input = EchoInput;
    type Output = EchoOutput;
    type Error = pforge_runtime::Error;

    async fn handle(&self, input: Self::Input) -> Result<Self::Output> {
        let delay = input.delay_ms.unwrap_or(0);

        if delay > 0 {
            tokio::time::sleep(tokio::time::Duration::from_millis(delay)).await;
        }

        Ok(EchoOutput {
            length: input.message.len(),
            message: input.message,
            delayed_ms: delay,
        })
    }
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct ErrorTestInput {}

#[derive(Debug, Serialize, JsonSchema)]
pub struct ErrorTestOutput {
    pub message: String,
}

pub struct ErrorTestHandler;

#[async_trait::async_trait]
impl Handler for ErrorTestHandler {
    type Input = ErrorTestInput;
    type Output = ErrorTestOutput;
    type Error = pforge_runtime::Error;

    async fn handle(&self, _input: Self::Input) -> Result<Self::Output> {
        Err(pforge_runtime::Error::Handler(
            "This handler always fails for testing error metrics".to_string(),
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_echo_handler() {
        let handler = EchoHandler;
        let input = EchoInput {
            message: "Hello".to_string(),
            delay_ms: None,
        };

        let output = handler.handle(input).await.unwrap();
        assert_eq!(output.message, "Hello");
        assert_eq!(output.length, 5);
        assert_eq!(output.delayed_ms, 0);
    }

    #[tokio::test]
    async fn test_echo_with_delay() {
        let handler = EchoHandler;
        let input = EchoInput {
            message: "Test".to_string(),
            delay_ms: Some(10),
        };

        let start = std::time::Instant::now();
        let output = handler.handle(input).await.unwrap();
        let elapsed = start.elapsed();

        assert_eq!(output.delayed_ms, 10);
        assert!(elapsed.as_millis() >= 10);
    }

    #[tokio::test]
    async fn test_error_handler() {
        let handler = ErrorTestHandler;
        let result = handler.handle(ErrorTestInput {}).await;
        assert!(result.is_err());
    }
}
