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
    output.push_str(&format!("#[derive(Debug, Deserialize, JsonSchema)]\n"));
    output.push_str(&format!("pub struct {} {{\n", struct_name));

    for (field_name, param_type) in &params.fields {
        // Generate field
        let (ty, required, description) = match param_type {
            ParamType::Simple(simple_ty) => {
                (rust_type_from_simple(simple_ty), true, None)
            }
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
                env,
                stream,
                description: _,
            } => {
                output.push_str(&format!(
                    "    registry.register(\"{}\", CliHandler::new(\n",
                    name
                ));
                output.push_str(&format!("        \"{}\".to_string(),\n", command));
                output.push_str(&format!("        vec![{}],\n", format_string_vec(args)));

                if let Some(cwd_val) = cwd {
                    output.push_str(&format!(
                        "        Some(\"{}\".to_string()),\n",
                        cwd_val
                    ));
                } else {
                    output.push_str("        None,\n");
                }

                output.push_str(&format!(
                    "        HashMap::new(), // env\n"
                ));
                output.push_str("        None, // timeout\n");
                output.push_str(&format!("        {},\n", stream));
                output.push_str("    ));\n");
            }
            pforge_config::ToolDef::Http {
                name,
                endpoint,
                method,
                headers,
                auth,
                description: _,
            } => {
                output.push_str(&format!(
                    "    registry.register(\"{}\", HttpHandler::new(\n",
                    name
                ));
                output.push_str(&format!("        \"{}\".to_string(),\n", endpoint));
                output.push_str(&format!("        HttpMethod::{:?},\n", method));
                output.push_str("        HashMap::new(), // headers\n");
                output.push_str("        None, // auth\n");
                output.push_str("    ));\n");
            }
            pforge_config::ToolDef::Pipeline { name, steps, description: _ } => {
                output.push_str("    // Pipeline handler TBD\n");
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
