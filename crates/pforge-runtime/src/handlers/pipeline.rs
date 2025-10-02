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
