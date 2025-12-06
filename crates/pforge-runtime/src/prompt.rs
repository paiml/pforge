use crate::{Error, Result};
use pforge_config::{ParamType, PromptDef};
use rustc_hash::FxHashMap;
use serde_json::Value;

/// Prompt manager handles prompt rendering with template interpolation
pub struct PromptManager {
    prompts: FxHashMap<String, PromptEntry>,
}

struct PromptEntry {
    description: String,
    template: String,
    arguments: FxHashMap<String, ParamType>,
}

impl PromptManager {
    pub fn new() -> Self {
        Self {
            prompts: FxHashMap::default(),
        }
    }

    /// Register a prompt definition
    pub fn register(&mut self, def: PromptDef) -> Result<()> {
        if self.prompts.contains_key(&def.name) {
            return Err(Error::Handler(format!(
                "Prompt '{}' already registered",
                def.name
            )));
        }

        self.prompts.insert(
            def.name.clone(),
            PromptEntry {
                description: def.description,
                template: def.template,
                arguments: def.arguments,
            },
        );

        Ok(())
    }

    /// Render a prompt with given arguments
    pub fn render(&self, name: &str, args: FxHashMap<String, Value>) -> Result<String> {
        let entry = self
            .prompts
            .get(name)
            .ok_or_else(|| Error::Handler(format!("Prompt '{}' not found", name)))?;

        // Validate arguments
        self.validate_arguments(entry, &args)?;

        // Perform template interpolation
        self.interpolate(&entry.template, &args)
    }

    /// Get prompt metadata
    pub fn get_prompt(&self, name: &str) -> Option<PromptMetadata> {
        self.prompts.get(name).map(|entry| PromptMetadata {
            description: entry.description.clone(),
            arguments: entry.arguments.clone(),
        })
    }

    /// List all registered prompts
    pub fn list_prompts(&self) -> Vec<String> {
        self.prompts.keys().cloned().collect()
    }

    /// Validate arguments against schema
    fn validate_arguments(
        &self,
        entry: &PromptEntry,
        args: &FxHashMap<String, Value>,
    ) -> Result<()> {
        // Check required arguments
        for (arg_name, param_type) in &entry.arguments {
            let is_required = match param_type {
                ParamType::Complex { required, .. } => *required,
                _ => false,
            };

            if is_required && !args.contains_key(arg_name) {
                return Err(Error::Handler(format!(
                    "Required argument '{}' not provided",
                    arg_name
                )));
            }
        }

        // Type validation could be added here
        Ok(())
    }

    /// Interpolate template with argument values
    /// Supports {{variable}} syntax
    fn interpolate(&self, template: &str, args: &FxHashMap<String, Value>) -> Result<String> {
        let mut result = template.to_string();

        for (key, value) in args {
            let placeholder = format!("{{{{{}}}}}", key);
            let replacement = match value {
                Value::String(s) => s.clone(),
                Value::Number(n) => n.to_string(),
                Value::Bool(b) => b.to_string(),
                Value::Null => String::new(),
                _ => serde_json::to_string(value)
                    .map_err(|e| Error::Handler(format!("Failed to serialize value: {}", e)))?,
            };

            result = result.replace(&placeholder, &replacement);
        }

        // Check for unresolved placeholders
        if result.contains("{{") && result.contains("}}") {
            // Extract unresolved variable names for better error message
            let unresolved: Vec<&str> = result
                .split("{{")
                .skip(1)
                .filter_map(|s| s.split("}}").next())
                .collect();

            if !unresolved.is_empty() {
                return Err(Error::Handler(format!(
                    "Unresolved template variables: {}",
                    unresolved.join(", ")
                )));
            }
        }

        Ok(result)
    }
}

impl Default for PromptManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Prompt metadata for discovery
#[derive(Debug, Clone)]
pub struct PromptMetadata {
    pub description: String,
    pub arguments: FxHashMap<String, ParamType>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use pforge_config::SimpleType;
    use serde_json::json;

    #[test]
    fn test_prompt_registration() {
        let mut manager = PromptManager::new();

        let def = PromptDef {
            name: "greeting".to_string(),
            description: "A simple greeting prompt".to_string(),
            template: "Hello, {{name}}!".to_string(),
            arguments: FxHashMap::default(),
        };

        manager.register(def).unwrap();
        assert_eq!(manager.list_prompts(), vec!["greeting"]);
    }

    #[test]
    fn test_duplicate_prompt_registration() {
        let mut manager = PromptManager::new();

        let def = PromptDef {
            name: "test".to_string(),
            description: "Test".to_string(),
            template: "{{x}}".to_string(),
            arguments: FxHashMap::default(),
        };

        manager.register(def.clone()).unwrap();
        let result = manager.register(def);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("already registered"));
    }

    #[test]
    fn test_simple_interpolation() {
        let mut manager = PromptManager::new();

        let def = PromptDef {
            name: "greeting".to_string(),
            description: "Greeting".to_string(),
            template: "Hello, {{name}}! You are {{age}} years old.".to_string(),
            arguments: FxHashMap::default(),
        };

        manager.register(def).unwrap();

        let mut args = FxHashMap::default();
        args.insert("name".to_string(), json!("Alice"));
        args.insert("age".to_string(), json!(30));

        let result = manager.render("greeting", args).unwrap();
        assert_eq!(result, "Hello, Alice! You are 30 years old.");
    }

    #[test]
    fn test_required_argument_validation() {
        let mut manager = PromptManager::new();

        let mut arguments = FxHashMap::default();
        arguments.insert(
            "name".to_string(),
            ParamType::Complex {
                ty: SimpleType::String,
                required: true,
                default: None,
                description: None,
                validation: None,
            },
        );

        let def = PromptDef {
            name: "greeting".to_string(),
            description: "Greeting".to_string(),
            template: "Hello, {{name}}!".to_string(),
            arguments,
        };

        manager.register(def).unwrap();

        let args = FxHashMap::default();
        let result = manager.render("greeting", args);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Required argument"));
    }

    #[test]
    fn test_unresolved_placeholder() {
        let mut manager = PromptManager::new();

        let def = PromptDef {
            name: "test".to_string(),
            description: "Test".to_string(),
            template: "Hello, {{name}}! Welcome to {{location}}.".to_string(),
            arguments: FxHashMap::default(),
        };

        manager.register(def).unwrap();

        let mut args = FxHashMap::default();
        args.insert("name".to_string(), json!("Alice"));
        // Missing 'location' argument

        let result = manager.render("test", args);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Unresolved template variables"));
    }

    #[test]
    fn test_get_prompt_metadata() {
        let mut manager = PromptManager::new();

        let mut arguments = FxHashMap::default();
        arguments.insert(
            "name".to_string(),
            ParamType::Complex {
                ty: SimpleType::String,
                required: true,
                default: None,
                description: Some("User name".to_string()),
                validation: None,
            },
        );

        let def = PromptDef {
            name: "greeting".to_string(),
            description: "A greeting prompt".to_string(),
            template: "Hello, {{name}}!".to_string(),
            arguments,
        };

        manager.register(def).unwrap();

        let metadata = manager.get_prompt("greeting").unwrap();
        assert_eq!(metadata.description, "A greeting prompt");
        assert!(metadata.arguments.contains_key("name"));
    }

    #[test]
    fn test_complex_value_interpolation() {
        let mut manager = PromptManager::new();

        let def = PromptDef {
            name: "test".to_string(),
            description: "Test".to_string(),
            template: "String: {{str}}, Number: {{num}}, Bool: {{bool}}".to_string(),
            arguments: FxHashMap::default(),
        };

        manager.register(def).unwrap();

        let mut args = FxHashMap::default();
        args.insert("str".to_string(), json!("hello"));
        args.insert("num".to_string(), json!(42));
        args.insert("bool".to_string(), json!(true));

        let result = manager.render("test", args).unwrap();
        assert_eq!(result, "String: hello, Number: 42, Bool: true");
    }

    #[test]
    fn test_required_argument_provided_succeeds() {
        // This test catches the && to || mutation in validate_arguments
        // When is_required=true AND arg IS provided, should succeed
        let mut manager = PromptManager::new();

        let mut arguments = FxHashMap::default();
        arguments.insert(
            "name".to_string(),
            ParamType::Complex {
                ty: SimpleType::String,
                required: true,
                default: None,
                description: None,
                validation: None,
            },
        );

        let def = PromptDef {
            name: "greeting".to_string(),
            description: "Greeting".to_string(),
            template: "Hello, {{name}}!".to_string(),
            arguments,
        };

        manager.register(def).unwrap();

        let mut args = FxHashMap::default();
        args.insert("name".to_string(), json!("Alice"));

        // This should succeed - required arg is provided
        let result = manager.render("greeting", args).unwrap();
        assert_eq!(result, "Hello, Alice!");
    }

    #[test]
    fn test_null_value_interpolation() {
        // This test catches the deletion of Value::Null match arm
        let mut manager = PromptManager::new();

        let def = PromptDef {
            name: "test".to_string(),
            description: "Test".to_string(),
            template: "Value is: {{val}}.".to_string(),
            arguments: FxHashMap::default(),
        };

        manager.register(def).unwrap();

        let mut args = FxHashMap::default();
        args.insert("val".to_string(), Value::Null);

        let result = manager.render("test", args).unwrap();
        // Null should be rendered as empty string
        assert_eq!(result, "Value is: .");
    }

    #[test]
    fn test_array_value_interpolation() {
        let mut manager = PromptManager::new();

        let def = PromptDef {
            name: "test".to_string(),
            description: "Test".to_string(),
            template: "Items: {{items}}".to_string(),
            arguments: FxHashMap::default(),
        };

        manager.register(def).unwrap();

        let mut args = FxHashMap::default();
        args.insert("items".to_string(), json!(["a", "b", "c"]));

        let result = manager.render("test", args).unwrap();
        assert_eq!(result, "Items: [\"a\",\"b\",\"c\"]");
    }
}
