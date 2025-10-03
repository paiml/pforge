use pforge_runtime::{Handler, Result};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::process::Command;

#[derive(Debug, Deserialize, JsonSchema)]
pub struct GoHashInput {
    pub data: String,
    #[serde(default = "default_algorithm")]
    pub algorithm: String,
}

fn default_algorithm() -> String {
    "sha256".to_string()
}

#[derive(Debug, Serialize, JsonSchema)]
pub struct GoHashOutput {
    pub data: String,
    pub algorithm: String,
    pub hash: String,
    pub length: usize,
    pub implementation: String,
}

pub struct GoHashHandler;

#[async_trait::async_trait]
impl Handler for GoHashHandler {
    type Input = GoHashInput;
    type Output = GoHashOutput;
    type Error = pforge_runtime::Error;

    async fn handle(&self, input: Self::Input) -> Result<Self::Output> {
        // In production, this would call the Go bridge via CGO
        // For this example, we'll use a Go subprocess
        let binary_path = "src/go/hasher";

        let output = Command::new(binary_path)
            .arg(&input.algorithm)
            .arg(&input.data)
            .output()
            .map_err(|e| pforge_runtime::Error::Handler(format!("Go execution failed: {}", e)))?;

        if !output.status.success() {
            let error = String::from_utf8_lossy(&output.stderr);
            return Err(pforge_runtime::Error::Handler(format!("Go error: {}", error)));
        }

        let result: serde_json::Value = serde_json::from_slice(&output.stdout)
            .map_err(|e| pforge_runtime::Error::Handler(format!("JSON parse error: {}", e)))?;

        let hash = result["hash"].as_str().unwrap_or("").to_string();
        let length = hash.len();

        Ok(GoHashOutput {
            data: input.data,
            algorithm: input.algorithm,
            hash,
            length,
            implementation: "Go".to_string(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_algorithm() {
        assert_eq!(default_algorithm(), "sha256");
    }

    #[tokio::test]
    async fn test_handler_structure() {
        // Note: This test requires Go binary to be compiled
        // In a real scenario, we'd mock the Go execution
        let _handler = GoHashHandler;
        let input = GoHashInput {
            data: "test".to_string(),
            algorithm: "sha256".to_string(),
        };

        // Test would call handler.handle(input).await
        // For now, just verify structure compiles
        assert_eq!(input.algorithm, "sha256");
    }
}
