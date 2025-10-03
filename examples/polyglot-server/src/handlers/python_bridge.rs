use pforge_runtime::{Handler, Result};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::process::Command;

#[derive(Debug, Deserialize, JsonSchema)]
pub struct PythonSentimentInput {
    pub text: String,
    #[serde(default = "default_language")]
    pub language: String,
}

fn default_language() -> String {
    "en".to_string()
}

#[derive(Debug, Serialize, JsonSchema)]
pub struct PythonSentimentOutput {
    pub text: String,
    pub sentiment: String,
    pub score: f64,
    pub language: String,
    pub implementation: String,
}

pub struct PythonSentimentHandler;

#[async_trait::async_trait]
impl Handler for PythonSentimentHandler {
    type Input = PythonSentimentInput;
    type Output = PythonSentimentOutput;
    type Error = pforge_runtime::Error;

    async fn handle(&self, input: Self::Input) -> Result<Self::Output> {
        // In production, this would call the Python bridge FFI
        // For this example, we'll use a Python subprocess
        let script_path = "src/python/sentiment_analyzer.py";

        let output = Command::new("python3")
            .arg(script_path)
            .arg(&input.text)
            .arg(&input.language)
            .output()
            .map_err(|e| pforge_runtime::Error::Handler(format!("Python execution failed: {}", e)))?;

        if !output.status.success() {
            let error = String::from_utf8_lossy(&output.stderr);
            return Err(pforge_runtime::Error::Handler(format!("Python error: {}", error)));
        }

        let result: serde_json::Value = serde_json::from_slice(&output.stdout)
            .map_err(|e| pforge_runtime::Error::Handler(format!("JSON parse error: {}", e)))?;

        Ok(PythonSentimentOutput {
            text: input.text,
            sentiment: result["sentiment"].as_str().unwrap_or("neutral").to_string(),
            score: result["score"].as_f64().unwrap_or(0.0),
            language: input.language,
            implementation: "Python".to_string(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_language() {
        assert_eq!(default_language(), "en");
    }

    #[tokio::test]
    async fn test_handler_structure() {
        // Note: This test requires Python and the script to be present
        // In a real scenario, we'd mock the Python execution
        let _handler = PythonSentimentHandler;
        let input = PythonSentimentInput {
            text: "This is wonderful!".to_string(),
            language: "en".to_string(),
        };

        // Test would call handler.handle(input).await
        // For now, just verify structure compiles
        assert_eq!(input.language, "en");
    }
}
