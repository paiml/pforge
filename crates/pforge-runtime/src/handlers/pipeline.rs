use crate::{HandlerRegistry, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct PipelineHandler {
    pub steps: Vec<PipelineStep>,
}

#[derive(Debug, Clone)]
pub struct PipelineStep {
    pub tool: String,
    pub input: Option<serde_json::Value>,
    pub output_var: Option<String>,
    pub condition: Option<String>,
    pub error_policy: ErrorPolicy,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ErrorPolicy {
    FailFast,
    Continue,
}

#[derive(Debug, Deserialize)]
pub struct PipelineInput {
    #[serde(default)]
    pub variables: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Serialize)]
pub struct PipelineOutput {
    pub results: Vec<StepResult>,
    pub variables: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Serialize)]
pub struct StepResult {
    pub tool: String,
    pub success: bool,
    pub output: Option<serde_json::Value>,
    pub error: Option<String>,
}

impl PipelineHandler {
    pub fn new(steps: Vec<PipelineStep>) -> Self {
        Self { steps }
    }

    pub async fn execute(
        &self,
        input: PipelineInput,
        registry: &HandlerRegistry,
    ) -> Result<PipelineOutput> {
        let mut variables = input.variables;
        let mut results = Vec::new();

        for step in &self.steps {
            // Check condition if present
            if let Some(condition) = &step.condition {
                if !self.evaluate_condition(condition, &variables) {
                    continue;
                }
            }

            // Interpolate input with variables
            let step_input = if let Some(input_template) = &step.input {
                self.interpolate_variables(input_template, &variables)
            } else {
                serde_json::json!({})
            };

            // Execute step
            let step_result = match registry
                .dispatch(&step.tool, &serde_json::to_vec(&step_input)?)
                .await
            {
                Ok(output) => {
                    let output_value: serde_json::Value = serde_json::from_slice(&output)?;

                    // Store output in variable if specified
                    if let Some(var_name) = &step.output_var {
                        variables.insert(var_name.clone(), output_value.clone());
                    }

                    StepResult {
                        tool: step.tool.clone(),
                        success: true,
                        output: Some(output_value),
                        error: None,
                    }
                }
                Err(e) => {
                    let result = StepResult {
                        tool: step.tool.clone(),
                        success: false,
                        output: None,
                        error: Some(e.to_string()),
                    };

                    // Handle error based on policy
                    if step.error_policy == ErrorPolicy::FailFast {
                        results.push(result);
                        return Err(e);
                    }

                    result
                }
            };

            results.push(step_result);
        }

        Ok(PipelineOutput { results, variables })
    }

    fn evaluate_condition(
        &self,
        condition: &str,
        variables: &HashMap<String, serde_json::Value>,
    ) -> bool {
        // Simple variable existence check for MVP
        // Format: "variable_name" or "!variable_name"
        if let Some(var_name) = condition.strip_prefix('!') {
            !variables.contains_key(var_name)
        } else {
            variables.contains_key(condition)
        }
    }

    #[allow(clippy::only_used_in_recursion)]
    fn interpolate_variables(
        &self,
        template: &serde_json::Value,
        variables: &HashMap<String, serde_json::Value>,
    ) -> serde_json::Value {
        match template {
            serde_json::Value::String(s) => {
                // Replace {{var}} with variable value
                let mut result = s.clone();
                for (key, value) in variables {
                    let pattern = format!("{{{{{}}}}}", key);
                    if let Some(value_str) = value.as_str() {
                        result = result.replace(&pattern, value_str);
                    }
                }
                serde_json::Value::String(result)
            }
            serde_json::Value::Object(obj) => {
                let mut new_obj = serde_json::Map::new();
                for (k, v) in obj {
                    new_obj.insert(k.clone(), self.interpolate_variables(v, variables));
                }
                serde_json::Value::Object(new_obj)
            }
            serde_json::Value::Array(arr) => {
                let new_arr: Vec<_> = arr
                    .iter()
                    .map(|v| self.interpolate_variables(v, variables))
                    .collect();
                serde_json::Value::Array(new_arr)
            }
            other => other.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pipeline_handler_new() {
        let steps = vec![PipelineStep {
            tool: "test_tool".to_string(),
            input: None,
            output_var: None,
            condition: None,
            error_policy: ErrorPolicy::FailFast,
        }];

        let handler = PipelineHandler::new(steps);
        assert_eq!(handler.steps.len(), 1);
        assert_eq!(handler.steps[0].tool, "test_tool");
    }

    #[test]
    fn test_error_policy_equality() {
        assert_eq!(ErrorPolicy::FailFast, ErrorPolicy::FailFast);
        assert_eq!(ErrorPolicy::Continue, ErrorPolicy::Continue);
        assert_ne!(ErrorPolicy::FailFast, ErrorPolicy::Continue);
    }

    #[test]
    fn test_evaluate_condition_exists() {
        let handler = PipelineHandler::new(vec![]);
        let mut vars = HashMap::new();
        vars.insert("key".to_string(), serde_json::json!("value"));

        assert!(handler.evaluate_condition("key", &vars));
        assert!(!handler.evaluate_condition("missing", &vars));
    }

    #[test]
    fn test_evaluate_condition_not_exists() {
        let handler = PipelineHandler::new(vec![]);
        let mut vars = HashMap::new();
        vars.insert("key".to_string(), serde_json::json!("value"));

        assert!(!handler.evaluate_condition("!key", &vars));
        assert!(handler.evaluate_condition("!missing", &vars));
    }

    #[test]
    fn test_interpolate_variables_string() {
        let handler = PipelineHandler::new(vec![]);
        let mut vars = HashMap::new();
        vars.insert("name".to_string(), serde_json::json!("Alice"));

        let template = serde_json::json!("Hello {{name}}!");
        let result = handler.interpolate_variables(&template, &vars);

        assert_eq!(result, serde_json::json!("Hello Alice!"));
    }

    #[test]
    fn test_interpolate_variables_object() {
        let handler = PipelineHandler::new(vec![]);
        let mut vars = HashMap::new();
        vars.insert("user".to_string(), serde_json::json!("Bob"));

        let template = serde_json::json!({"greeting": "Hi {{user}}"});
        let result = handler.interpolate_variables(&template, &vars);

        assert_eq!(result["greeting"], "Hi Bob");
    }

    #[test]
    fn test_interpolate_variables_array() {
        let handler = PipelineHandler::new(vec![]);
        let mut vars = HashMap::new();
        vars.insert("item".to_string(), serde_json::json!("test"));

        let template = serde_json::json!(["{{item}}", "other"]);
        let result = handler.interpolate_variables(&template, &vars);

        assert_eq!(result[0], "test");
        assert_eq!(result[1], "other");
    }

    #[test]
    fn test_interpolate_variables_no_match() {
        let handler = PipelineHandler::new(vec![]);
        let vars = HashMap::new();

        let template = serde_json::json!("Hello {{missing}}!");
        let result = handler.interpolate_variables(&template, &vars);

        assert_eq!(result, serde_json::json!("Hello {{missing}}!"));
    }

    #[test]
    fn test_pipeline_input_deserialization() {
        let json = r#"{"variables": {"key": "value"}}"#;
        let input: PipelineInput = serde_json::from_str(json).unwrap();

        assert_eq!(input.variables.len(), 1);
        assert_eq!(input.variables["key"], "value");
    }

    #[test]
    fn test_pipeline_output_serialization() {
        let output = PipelineOutput {
            results: vec![StepResult {
                tool: "test".to_string(),
                success: true,
                output: Some(serde_json::json!({"result": "ok"})),
                error: None,
            }],
            variables: HashMap::new(),
        };

        let json = serde_json::to_string(&output).unwrap();
        assert!(json.contains("\"tool\":\"test\""));
        assert!(json.contains("\"success\":true"));
    }

    #[tokio::test]
    async fn test_pipeline_execute_simple() {
        use crate::{Handler, HandlerRegistry};
        use schemars::JsonSchema;

        // Create a simple test handler
        #[derive(Debug, serde::Deserialize, JsonSchema)]
        struct TestInput {
            value: String,
        }

        #[derive(Debug, serde::Serialize, JsonSchema)]
        struct TestOutput {
            result: String,
        }

        struct TestHandler;

        #[async_trait::async_trait]
        impl Handler for TestHandler {
            type Input = TestInput;
            type Output = TestOutput;
            type Error = crate::Error;

            async fn handle(&self, input: Self::Input) -> crate::Result<Self::Output> {
                Ok(TestOutput {
                    result: format!("processed: {}", input.value),
                })
            }
        }

        // Setup registry
        let mut registry = HandlerRegistry::new();
        registry.register("test_tool", TestHandler);

        // Create pipeline with one step
        let handler = PipelineHandler::new(vec![PipelineStep {
            tool: "test_tool".to_string(),
            input: Some(serde_json::json!({"value": "hello"})),
            output_var: Some("result".to_string()),
            condition: None,
            error_policy: ErrorPolicy::FailFast,
        }]);

        let input = PipelineInput {
            variables: HashMap::new(),
        };

        let output = handler.execute(input, &registry).await.unwrap();

        assert_eq!(output.results.len(), 1);
        assert!(output.results[0].success);
        assert!(output.variables.contains_key("result"));
    }

    #[tokio::test]
    async fn test_pipeline_execute_with_condition_skip() {
        use crate::HandlerRegistry;

        let registry = HandlerRegistry::new();

        let handler = PipelineHandler::new(vec![PipelineStep {
            tool: "nonexistent".to_string(),
            input: None,
            output_var: None,
            condition: Some("missing_var".to_string()),
            error_policy: ErrorPolicy::FailFast,
        }]);

        let input = PipelineInput {
            variables: HashMap::new(),
        };

        let output = handler.execute(input, &registry).await.unwrap();

        // Step should be skipped due to failed condition
        assert_eq!(output.results.len(), 0);
    }

    #[tokio::test]
    async fn test_pipeline_execute_continue_on_error() {
        use crate::HandlerRegistry;

        let registry = HandlerRegistry::new();

        let handler = PipelineHandler::new(vec![
            PipelineStep {
                tool: "nonexistent1".to_string(),
                input: None,
                output_var: None,
                condition: None,
                error_policy: ErrorPolicy::Continue,
            },
            PipelineStep {
                tool: "nonexistent2".to_string(),
                input: None,
                output_var: None,
                condition: None,
                error_policy: ErrorPolicy::Continue,
            },
        ]);

        let input = PipelineInput {
            variables: HashMap::new(),
        };

        let output = handler.execute(input, &registry).await.unwrap();

        // Both steps should fail but continue
        assert_eq!(output.results.len(), 2);
        assert!(!output.results[0].success);
        assert!(!output.results[1].success);
    }

    #[tokio::test]
    async fn test_pipeline_execute_fail_fast() {
        use crate::HandlerRegistry;

        let registry = HandlerRegistry::new();

        let handler = PipelineHandler::new(vec![
            PipelineStep {
                tool: "nonexistent1".to_string(),
                input: None,
                output_var: None,
                condition: None,
                error_policy: ErrorPolicy::FailFast,
            },
            PipelineStep {
                tool: "nonexistent2".to_string(),
                input: None,
                output_var: None,
                condition: None,
                error_policy: ErrorPolicy::FailFast,
            },
        ]);

        let input = PipelineInput {
            variables: HashMap::new(),
        };

        let result = handler.execute(input, &registry).await;

        // Should fail on first error
        assert!(result.is_err());
    }
}
