use pforge_config::{ForgeConfig, ParamSchema, ParamType, SimpleType};
use std::path::PathBuf;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum CodegenError {
    #[error("IO error: {0}: {1}")]
    IoError(PathBuf, #[source] std::io::Error),

    #[error("Generation error: {0}")]
    GenerationError(String),
}

pub type Result<T> = std::result::Result<T, CodegenError>;

/// Generate a parameter struct from ParamSchema
pub fn generate_param_struct(tool_name: &str, params: &ParamSchema) -> Result<String> {
    let struct_name = format!("{}Params", to_pascal_case(tool_name));
    let mut output = String::new();

    // Generate struct
    output.push_str("#[derive(Debug, Deserialize, JsonSchema)]\n");
    output.push_str(&format!("pub struct {} {{\n", struct_name));

    for (field_name, param_type) in &params.fields {
        // Generate field
        let (ty, required, description) = match param_type {
            ParamType::Simple(simple_ty) => (rust_type_from_simple(simple_ty), true, None),
            ParamType::Complex {
                ty,
                required,
                description,
                ..
            } => (rust_type_from_simple(ty), *required, description.clone()),
        };

        // Add documentation if present
        if let Some(desc) = description {
            output.push_str(&format!("    /// {}\n", desc));
        }

        // Add field
        if required {
            output.push_str(&format!("    pub {}: {},\n", field_name, ty));
        } else {
            output.push_str(&format!("    pub {}: Option<{}>,\n", field_name, ty));
        }
    }

    output.push_str("}\n");

    Ok(output)
}

/// Generate handler registration code
pub fn generate_handler_registration(config: &ForgeConfig) -> Result<String> {
    let mut output = String::new();

    output.push_str("pub fn register_handlers(registry: &mut HandlerRegistry) {\n");

    for tool in &config.tools {
        match tool {
            pforge_config::ToolDef::Native { name, handler, .. } => {
                // Extract handler path
                let handler_path = &handler.path;
                output.push_str(&format!(
                    "    registry.register(\"{}\", {});\n",
                    name, handler_path
                ));
            }
            pforge_config::ToolDef::Cli {
                name,
                command,
                args,
                cwd,
                env: _,
                stream,
                ..
            } => {
                output.push_str(&format!(
                    "    registry.register(\"{}\", CliHandler::new(\n",
                    name
                ));
                output.push_str(&format!("        \"{}\".to_string(),\n", command));
                output.push_str(&format!("        vec![{}],\n", format_string_vec(args)));

                if let Some(cwd_val) = cwd {
                    output.push_str(&format!("        Some(\"{}\".to_string()),\n", cwd_val));
                } else {
                    output.push_str("        None,\n");
                }

                output.push_str("        FxHashMap::default(), // env\n");
                output.push_str("        None, // timeout\n");
                output.push_str(&format!("        {},\n", stream));
                output.push_str("    ));\n");
            }
            pforge_config::ToolDef::Http {
                name,
                endpoint,
                method,
                ..
            } => {
                output.push_str(&format!(
                    "    registry.register(\"{}\", HttpHandler::new(\n",
                    name
                ));
                output.push_str(&format!("        \"{}\".to_string(),\n", endpoint));
                output.push_str(&format!("        HttpMethod::{:?},\n", method));
                output.push_str("        FxHashMap::default(), // headers\n");
                output.push_str("        None, // auth\n");
                output.push_str("    ));\n");
            }
            pforge_config::ToolDef::Pipeline {
                name,
                steps,
                description: _,
            } => {
                output.push_str(&format!(
                    "    registry.register(\"{}\", PipelineHandler::new(vec![\n",
                    name
                ));
                for step in steps {
                    output.push_str("        PipelineStep {\n");
                    output.push_str(&format!(
                        "            tool: \"{}\".to_string(),\n",
                        step.tool
                    ));
                    // Generate input
                    if let Some(input_val) = &step.input {
                        output.push_str(&format!(
                            "            input: Some(serde_json::json!({})),\n",
                            serde_json::to_string(input_val).unwrap_or_else(|_| "{}".to_string())
                        ));
                    } else {
                        output.push_str("            input: None,\n");
                    }
                    // Generate output_var
                    if let Some(var) = &step.output_var {
                        output.push_str(&format!(
                            "            output_var: Some(\"{}\".to_string()),\n",
                            var
                        ));
                    } else {
                        output.push_str("            output_var: None,\n");
                    }
                    // Generate condition
                    if let Some(cond) = &step.condition {
                        output.push_str(&format!(
                            "            condition: Some(\"{}\".to_string()),\n",
                            cond
                        ));
                    } else {
                        output.push_str("            condition: None,\n");
                    }
                    // Generate error policy
                    let policy = match step.error_policy {
                        pforge_config::ErrorPolicy::FailFast => "ErrorPolicy::FailFast",
                        pforge_config::ErrorPolicy::Continue => "ErrorPolicy::Continue",
                    };
                    output.push_str(&format!("            error_policy: {},\n", policy));
                    output.push_str("        },\n");
                }
                output.push_str("    ]));\n");
            }
        }
    }

    output.push_str("}\n");

    Ok(output)
}

fn to_pascal_case(s: &str) -> String {
    s.split('_')
        .map(|word| {
            let mut chars = word.chars();
            match chars.next() {
                None => String::new(),
                Some(first) => first.to_uppercase().chain(chars).collect(),
            }
        })
        .collect()
}

fn rust_type_from_simple(ty: &SimpleType) -> &'static str {
    match ty {
        SimpleType::String => "String",
        SimpleType::Integer => "i64",
        SimpleType::Float => "f64",
        SimpleType::Boolean => "bool",
        SimpleType::Array => "Vec<serde_json::Value>",
        SimpleType::Object => "serde_json::Value",
    }
}

fn format_string_vec(vec: &[String]) -> String {
    vec.iter()
        .map(|s| format!("\"{}\".to_string()", s))
        .collect::<Vec<_>>()
        .join(", ")
}

#[cfg(test)]
mod tests {
    use super::*;
    use pforge_config::*;
    use rustc_hash::FxHashMap;

    #[test]
    fn test_to_pascal_case() {
        assert_eq!(to_pascal_case("hello_world"), "HelloWorld");
        assert_eq!(to_pascal_case("test"), "Test");
        assert_eq!(to_pascal_case("foo_bar_baz"), "FooBarBaz");
    }

    #[test]
    fn test_rust_type_from_simple() {
        assert_eq!(rust_type_from_simple(&SimpleType::String), "String");
        assert_eq!(rust_type_from_simple(&SimpleType::Integer), "i64");
        assert_eq!(rust_type_from_simple(&SimpleType::Float), "f64");
        assert_eq!(rust_type_from_simple(&SimpleType::Boolean), "bool");
        assert_eq!(
            rust_type_from_simple(&SimpleType::Array),
            "Vec<serde_json::Value>"
        );
        assert_eq!(
            rust_type_from_simple(&SimpleType::Object),
            "serde_json::Value"
        );
    }

    #[test]
    fn test_format_string_vec() {
        assert_eq!(
            format_string_vec(&["foo".to_string(), "bar".to_string()]),
            "\"foo\".to_string(), \"bar\".to_string()"
        );
        assert_eq!(format_string_vec(&[]), "");
    }

    #[test]
    fn test_generate_param_struct_simple() {
        let mut fields = FxHashMap::default();
        fields.insert("name".to_string(), ParamType::Simple(SimpleType::String));
        fields.insert("age".to_string(), ParamType::Simple(SimpleType::Integer));

        let params = ParamSchema { fields };
        let result = generate_param_struct("test_tool", &params);

        assert!(result.is_ok());
        let code = result.unwrap();
        assert!(code.contains("pub struct TestToolParams"));
        assert!(code.contains("pub name: String"));
        assert!(code.contains("pub age: i64"));
    }

    #[test]
    fn test_generate_param_struct_complex() {
        let mut fields = FxHashMap::default();
        fields.insert(
            "optional_field".to_string(),
            ParamType::Complex {
                ty: SimpleType::String,
                required: false,
                description: Some("An optional field".to_string()),
                default: None,
                validation: None,
            },
        );

        let params = ParamSchema { fields };
        let result = generate_param_struct("my_tool", &params);

        assert!(result.is_ok());
        let code = result.unwrap();
        assert!(code.contains("/// An optional field"));
        assert!(code.contains("pub optional_field: Option<String>"));
    }

    #[test]
    fn test_generate_handler_registration_native() {
        let config = ForgeConfig {
            forge: ForgeMetadata {
                name: "test".to_string(),
                version: "1.0.0".to_string(),
                transport: TransportType::Stdio,
                optimization: OptimizationLevel::Debug,
            },
            tools: vec![ToolDef::Native {
                name: "test_tool".to_string(),
                description: "Test".to_string(),
                handler: HandlerRef {
                    path: "handlers::test_handler".to_string(),
                    inline: None,
                },
                params: ParamSchema {
                    fields: FxHashMap::default(),
                },
                timeout_ms: None,
            }],
            resources: vec![],
            prompts: vec![],
            state: None,
        };

        let result = generate_handler_registration(&config);
        assert!(result.is_ok());
        let code = result.unwrap();
        assert!(code.contains("pub fn register_handlers"));
        assert!(code.contains("registry.register(\"test_tool\", handlers::test_handler)"));
    }

    #[test]
    fn test_generate_handler_registration_cli() {
        let config = ForgeConfig {
            forge: ForgeMetadata {
                name: "test".to_string(),
                version: "1.0.0".to_string(),
                transport: TransportType::Stdio,
                optimization: OptimizationLevel::Debug,
            },
            tools: vec![ToolDef::Cli {
                name: "cli_tool".to_string(),
                description: "CLI Test".to_string(),
                command: "echo".to_string(),
                args: vec!["hello".to_string()],
                cwd: None,
                env: FxHashMap::default(),
                stream: false,
                timeout_ms: None,
            }],
            resources: vec![],
            prompts: vec![],
            state: None,
        };

        let result = generate_handler_registration(&config);
        assert!(result.is_ok());
        let code = result.unwrap();
        assert!(code.contains("CliHandler::new"));
        assert!(code.contains("\"echo\""));
        assert!(code.contains("\"hello\""));
    }

    #[test]
    fn test_generate_handler_registration_http() {
        let config = ForgeConfig {
            forge: ForgeMetadata {
                name: "test".to_string(),
                version: "1.0.0".to_string(),
                transport: TransportType::Stdio,
                optimization: OptimizationLevel::Debug,
            },
            tools: vec![ToolDef::Http {
                name: "http_tool".to_string(),
                description: "HTTP Test".to_string(),
                endpoint: "https://api.example.com".to_string(),
                method: HttpMethod::Get,
                headers: FxHashMap::default(),
                auth: None,
                timeout_ms: None,
            }],
            resources: vec![],
            prompts: vec![],
            state: None,
        };

        let result = generate_handler_registration(&config);
        assert!(result.is_ok());
        let code = result.unwrap();
        assert!(code.contains("HttpHandler::new"));
        assert!(code.contains("https://api.example.com"));
        assert!(code.contains("HttpMethod::Get"));
    }

    #[test]
    fn test_generate_handler_registration_pipeline() {
        let config = ForgeConfig {
            forge: ForgeMetadata {
                name: "test".to_string(),
                version: "1.0.0".to_string(),
                transport: TransportType::Stdio,
                optimization: OptimizationLevel::Debug,
            },
            tools: vec![ToolDef::Pipeline {
                name: "pipeline_tool".to_string(),
                description: "Pipeline Test".to_string(),
                steps: vec![
                    pforge_config::PipelineStep {
                        tool: "step1".to_string(),
                        input: Some(serde_json::json!({"key": "value"})),
                        output_var: Some("result".to_string()),
                        condition: None,
                        error_policy: pforge_config::ErrorPolicy::FailFast,
                    },
                    pforge_config::PipelineStep {
                        tool: "step2".to_string(),
                        input: None,
                        output_var: None,
                        condition: Some("result".to_string()),
                        error_policy: pforge_config::ErrorPolicy::Continue,
                    },
                ],
            }],
            resources: vec![],
            prompts: vec![],
            state: None,
        };

        let result = generate_handler_registration(&config);
        assert!(result.is_ok());
        let code = result.unwrap();
        assert!(code.contains("PipelineHandler::new"));
        assert!(code.contains("tool: \"step1\""));
        assert!(code.contains("tool: \"step2\""));
        assert!(code.contains("output_var: Some(\"result\""));
        assert!(code.contains("condition: Some(\"result\""));
        assert!(code.contains("ErrorPolicy::FailFast"));
        assert!(code.contains("ErrorPolicy::Continue"));
    }
}
