use pforge_runtime::{Handler, Result};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Deserialize, JsonSchema)]
pub struct ProcessorInput {
    pub data: Value,
    #[serde(default = "default_format")]
    pub format: String,
}

fn default_format() -> String {
    "json".to_string()
}

#[derive(Debug, Serialize, JsonSchema)]
pub struct ProcessorOutput {
    pub processed_data: String,
    pub format: String,
    pub size_bytes: usize,
    pub validation: ValidationResult,
}

#[derive(Debug, Serialize, JsonSchema)]
pub struct ValidationResult {
    pub valid: bool,
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
}

pub struct DataProcessor;

#[async_trait::async_trait]
impl Handler for DataProcessor {
    type Input = ProcessorInput;
    type Output = ProcessorOutput;
    type Error = pforge_runtime::Error;

    async fn handle(&self, input: Self::Input) -> Result<Self::Output> {
        // Validate format
        let valid_formats = vec!["json", "yaml", "toml"];
        if !valid_formats.contains(&input.format.as_str()) {
            return Err(pforge_runtime::Error::Handler(format!(
                "Invalid format: {}. Supported: {:?}",
                input.format, valid_formats
            )));
        }

        // Process based on format
        let processed_data = match input.format.as_str() {
            "json" => serde_json::to_string_pretty(&input.data)
                .map_err(|e| pforge_runtime::Error::Handler(format!("JSON error: {}", e)))?,
            "yaml" => serde_yaml::to_string(&input.data)
                .map_err(|e| pforge_runtime::Error::Handler(format!("YAML error: {}", e)))?,
            "toml" => {
                // TOML requires specific structure, simplified for example
                format!("# TOML format\ndata = {}", serde_json::to_string(&input.data)?)
            }
            _ => unreachable!(),
        };

        // Validate data
        let validation = validate_data(&input.data);
        let size_bytes = processed_data.len();

        Ok(ProcessorOutput {
            processed_data,
            format: input.format,
            size_bytes,
            validation,
        })
    }
}

fn validate_data(data: &Value) -> ValidationResult {
    let errors = Vec::new();
    let mut warnings = Vec::new();

    // Example validation rules
    match data {
        Value::Null => {
            warnings.push("Data is null".to_string());
        }
        Value::Object(map) if map.is_empty() => {
            warnings.push("Object is empty".to_string());
        }
        Value::Array(arr) if arr.is_empty() => {
            warnings.push("Array is empty".to_string());
        }
        Value::Object(map) if map.len() > 100 => {
            warnings.push(format!("Large object with {} fields", map.len()));
        }
        _ => {}
    }

    ValidationResult {
        valid: errors.is_empty(),
        errors,
        warnings,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[tokio::test]
    async fn test_json_processing() {
        let handler = DataProcessor;
        let input = ProcessorInput {
            data: json!({"key": "value"}),
            format: "json".to_string(),
        };

        let result = handler.handle(input).await.unwrap();
        assert_eq!(result.format, "json");
        assert!(result.validation.valid);
        assert!(result.processed_data.contains("key"));
    }

    #[tokio::test]
    async fn test_yaml_processing() {
        let handler = DataProcessor;
        let input = ProcessorInput {
            data: json!({"test": "data"}),
            format: "yaml".to_string(),
        };

        let result = handler.handle(input).await.unwrap();
        assert_eq!(result.format, "yaml");
        assert!(result.processed_data.contains("test:"));
    }

    #[tokio::test]
    async fn test_invalid_format() {
        let handler = DataProcessor;
        let input = ProcessorInput {
            data: json!({}),
            format: "xml".to_string(),
        };

        let result = handler.handle(input).await;
        assert!(result.is_err());
    }
}
