use pforge_runtime::{Error, Handler, Result};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, JsonSchema)]
pub struct CalculateInput {
    pub operation: String,
    pub a: f64,
    pub b: f64,
}

#[derive(Debug, Serialize, JsonSchema)]
pub struct CalculateOutput {
    pub result: f64,
}

pub struct CalculateHandler;

#[async_trait::async_trait]
impl Handler for CalculateHandler {
    type Input = CalculateInput;
    type Output = CalculateOutput;
    type Error = Error;

    async fn handle(&self, input: Self::Input) -> Result<Self::Output> {
        let result = match input.operation.as_str() {
            "add" => input.a + input.b,
            "subtract" => input.a - input.b,
            "multiply" => input.a * input.b,
            "divide" => {
                if input.b == 0.0 {
                    return Err(Error::Handler("Division by zero".to_string()));
                }
                input.a / input.b
            }
            _ => {
                return Err(Error::Handler(format!(
                    "Unknown operation: {}. Supported: add, subtract, multiply, divide",
                    input.operation
                )))
            }
        };

        Ok(CalculateOutput { result })
    }
}

// Re-export for easier access
pub use CalculateHandler as calculate_handler;

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_add() {
        let handler = CalculateHandler;
        let input = CalculateInput {
            operation: "add".to_string(),
            a: 5.0,
            b: 3.0,
        };

        let output = handler.handle(input).await.unwrap();
        assert_eq!(output.result, 8.0);
    }

    #[tokio::test]
    async fn test_subtract() {
        let handler = CalculateHandler;
        let input = CalculateInput {
            operation: "subtract".to_string(),
            a: 10.0,
            b: 3.0,
        };

        let output = handler.handle(input).await.unwrap();
        assert_eq!(output.result, 7.0);
    }

    #[tokio::test]
    async fn test_multiply() {
        let handler = CalculateHandler;
        let input = CalculateInput {
            operation: "multiply".to_string(),
            a: 4.0,
            b: 5.0,
        };

        let output = handler.handle(input).await.unwrap();
        assert_eq!(output.result, 20.0);
    }

    #[tokio::test]
    async fn test_divide() {
        let handler = CalculateHandler;
        let input = CalculateInput {
            operation: "divide".to_string(),
            a: 15.0,
            b: 3.0,
        };

        let output = handler.handle(input).await.unwrap();
        assert_eq!(output.result, 5.0);
    }

    #[tokio::test]
    async fn test_divide_by_zero() {
        let handler = CalculateHandler;
        let input = CalculateInput {
            operation: "divide".to_string(),
            a: 10.0,
            b: 0.0,
        };

        let result = handler.handle(input).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Division by zero"));
    }

    #[tokio::test]
    async fn test_unknown_operation() {
        let handler = CalculateHandler;
        let input = CalculateInput {
            operation: "modulo".to_string(),
            a: 10.0,
            b: 3.0,
        };

        let result = handler.handle(input).await;
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Unknown operation"));
    }
}
