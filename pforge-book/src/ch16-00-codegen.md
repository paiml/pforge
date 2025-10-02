# Chapter 16: Code Generation Internals

pforge's code generation transforms declarative YAML configuration into optimized Rust code. This chapter explores the internals of `pforge-codegen`, the Abstract Syntax Tree (AST) transformations, and how type-safe handlers are generated at compile time.

## Code Generation Philosophy

**Key Principles**:
1. **Type Safety**: Generate compile-time checked code
2. **Zero Runtime Cost**: No dynamic dispatch where avoidable
3. **Readable Output**: Generated code should be maintainable
4. **Error Preservation**: Clear error messages pointing to YAML source

## Code Generation Pipeline

```
┌─────────────┐      ┌──────────────┐      ┌─────────────┐      ┌──────────┐
│ forge.yaml  │─────>│ Parse & Val  │─────>│ AST Trans   │─────>│ Rust Gen │
│             │      │ idate Config │      │ formation   │      │          │
└─────────────┘      └──────────────┘      └─────────────┘      └──────────┘
                            │                       │                   │
                            v                       v                   v
                     Error Location         Type Inference      main.rs
                     Line/Column            Schema Gen          handlers.rs
```

**Stages**:
1. **Parse**: YAML → `ForgeConfig` struct
2. **Validate**: Check semantics (tool name uniqueness, etc.)
3. **Transform**: Config → Rust AST
4. **Generate**: AST → formatted Rust code

## YAML Parsing and Validation

### Configuration Structures

From `crates/pforge-config/src/types.rs`:

```rust
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]  // Catch typos early
pub struct ForgeConfig {
    pub forge: ForgeMetadata,
    #[serde(default)]
    pub tools: Vec<ToolDef>,
    #[serde(default)]
    pub resources: Vec<ResourceDef>,
    #[serde(default)]
    pub prompts: Vec<PromptDef>,
    #[serde(default)]
    pub state: Option<StateDef>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ToolDef {
    Native {
        name: String,
        description: String,
        handler: HandlerRef,
        params: ParamSchema,
        #[serde(default)]
        timeout_ms: Option<u64>,
    },
    Cli {
        name: String,
        description: String,
        command: String,
        args: Vec<String>,
        // ...
    },
    Http { /* ... */ },
    Pipeline { /* ... */ },
}
```

**Key Design Decisions**:
- **`#[serde(deny_unknown_fields)]`**: Catch configuration errors at parse time
- **`#[serde(tag = "type")]`**: Discriminated union for tool types
- **`#[serde(default)]`**: Optional fields with sensible defaults

### Validation Pass

```rust
// crates/pforge-config/src/validator.rs
pub fn validate_config(config: &ForgeConfig) -> Result<(), ValidationError> {
    // Check for duplicate tool names
    let mut names = HashSet::new();
    for tool in &config.tools {
        if !names.insert(tool.name()) {
            return Err(ValidationError::DuplicateTool(tool.name().to_string()));
        }
    }

    // Validate handler references
    for tool in &config.tools {
        if let ToolDef::Native { handler, .. } = tool {
            validate_handler_path(&handler.path)?;
        }
    }

    // Validate parameter schemas
    for tool in &config.tools {
        if let ToolDef::Native { params, .. } = tool {
            validate_param_schema(params)?;
        }
    }

    // Validate pipeline references
    for tool in &config.tools {
        if let ToolDef::Pipeline { steps, .. } = tool {
            for step in steps {
                if !names.contains(&step.tool) {
                    return Err(ValidationError::UnknownTool(step.tool.clone()));
                }
            }
        }
    }

    Ok(())
}

fn validate_handler_path(path: &str) -> Result<(), ValidationError> {
    // Check format: module::submodule::function_name
    if !path.contains("::") {
        return Err(ValidationError::InvalidHandlerPath(path.to_string()));
    }

    // Ensure valid Rust identifier
    for segment in path.split("::") {
        if !is_valid_identifier(segment) {
            return Err(ValidationError::InvalidIdentifier(segment.to_string()));
        }
    }

    Ok(())
}
```

## AST Generation

### Generating Parameter Structs

From `crates/pforge-codegen/src/generator.rs`:

```rust
pub fn generate_param_struct(tool_name: &str, params: &ParamSchema) -> Result<String> {
    let struct_name = to_pascal_case(tool_name) + "Params";
    let mut output = String::new();

    // Derive traits
    output.push_str("#[derive(Debug, Deserialize, JsonSchema)]\n");
    output.push_str(&format!("pub struct {} {{\n", struct_name));

    // Generate fields
    for (field_name, param_type) in &params.fields {
        generate_field(&mut output, field_name, param_type)?;
    }

    output.push_str("}\n");

    Ok(output)
}

fn generate_field(
    output: &mut String,
    field_name: &str,
    param_type: &ParamType,
) -> Result<()> {
    let (ty, required, description) = match param_type {
        ParamType::Simple(simple_ty) => (rust_type_from_simple(simple_ty), true, None),
        ParamType::Complex {
            ty,
            required,
            description,
            ..
        } => (rust_type_from_simple(ty), *required, description.clone()),
    };

    // Add doc comment
    if let Some(desc) = description {
        output.push_str(&format!("    /// {}\n", desc));
    }

    // Add field
    if required {
        output.push_str(&format!("    pub {}: {},\n", field_name, ty));
    } else {
        output.push_str(&format!("    pub {}: Option<{}>,\n", field_name, ty));
    }

    Ok(())
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
```

**Example Output**:

```yaml
# Input (forge.yaml)
tools:
  - type: native
    name: calculate
    params:
      operation:
        type: string
        required: true
        description: "Operation: add, subtract, multiply, divide"
      a:
        type: float
        required: true
      b:
        type: float
        required: true
```

```rust
// Generated output
#[derive(Debug, Deserialize, JsonSchema)]
pub struct CalculateParams {
    /// Operation: add, subtract, multiply, divide
    pub operation: String,
    pub a: f64,
    pub b: f64,
}
```

### Generating Handler Registration

```rust
pub fn generate_handler_registration(config: &ForgeConfig) -> Result<String> {
    let mut output = String::new();

    output.push_str("pub fn register_handlers(registry: &mut HandlerRegistry) {\n");

    for tool in &config.tools {
        match tool {
            ToolDef::Native { name, handler, .. } => {
                generate_native_registration(&mut output, name, handler)?;
            }
            ToolDef::Cli {
                name,
                command,
                args,
                cwd,
                env,
                stream,
                ..
            } => {
                generate_cli_registration(&mut output, name, command, args, cwd, env, *stream)?;
            }
            ToolDef::Http {
                name,
                endpoint,
                method,
                headers,
                auth,
                ..
            } => {
                generate_http_registration(&mut output, name, endpoint, method, headers, auth)?;
            }
            ToolDef::Pipeline { name, steps, .. } => {
                generate_pipeline_registration(&mut output, name, steps)?;
            }
        }
    }

    output.push_str("}\n");

    Ok(output)
}

fn generate_native_registration(
    output: &mut String,
    name: &str,
    handler: &HandlerRef,
) -> Result<()> {
    output.push_str(&format!(
        "    registry.register(\"{}\", {});\n",
        name, handler.path
    ));
    Ok(())
}

fn generate_cli_registration(
    output: &mut String,
    name: &str,
    command: &str,
    args: &[String],
    cwd: &Option<String>,
    env: &HashMap<String, String>,
    stream: bool,
) -> Result<()> {
    output.push_str(&format!("    registry.register(\"{}\", CliHandler::new(\n", name));
    output.push_str(&format!("        \"{}\".to_string(),\n", command));
    output.push_str(&format!("        vec![{}],\n", format_string_vec(args)));

    if let Some(cwd_val) = cwd {
        output.push_str(&format!("        Some(\"{}\".to_string()),\n", cwd_val));
    } else {
        output.push_str("        None,\n");
    }

    output.push_str(&format!("        {{\n"));
    for (key, value) in env {
        output.push_str(&format!("            (\"{}\".to_string(), \"{}\".to_string()),\n", key, value));
    }
    output.push_str(&format!("        }}.into_iter().collect(),\n"));

    output.push_str("        None,\n"); // timeout
    output.push_str(&format!("        {},\n", stream));
    output.push_str("    ));\n");

    Ok(())
}
```

### Generating Main Function

```rust
pub fn generate_main(config: &ForgeConfig) -> Result<String> {
    let mut output = String::new();

    output.push_str("use pforge_runtime::HandlerRegistry;\n");
    output.push_str("use tokio;\n\n");

    output.push_str("#[tokio::main]\n");

    // Select runtime flavor based on transport
    match config.forge.transport {
        TransportType::Stdio => {
            output.push_str("#[tokio::main(flavor = \"current_thread\")]\n");
        }
        TransportType::Sse | TransportType::WebSocket => {
            output.push_str("#[tokio::main(flavor = \"multi_thread\")]\n");
        }
    }

    output.push_str("async fn main() -> Result<(), Box<dyn std::error::Error>> {\n");
    output.push_str("    let mut registry = HandlerRegistry::new();\n");
    output.push_str("    register_handlers(&mut registry);\n\n");

    // Generate transport-specific server start
    match config.forge.transport {
        TransportType::Stdio => {
            output.push_str("    pforge_runtime::serve_stdio(registry).await?;\n");
        }
        TransportType::Sse => {
            output.push_str("    pforge_runtime::serve_sse(registry, 3000).await?;\n");
        }
        TransportType::WebSocket => {
            output.push_str("    pforge_runtime::serve_websocket(registry, 3000).await?;\n");
        }
    }

    output.push_str("    Ok(())\n");
    output.push_str("}\n");

    Ok(output)
}
```

## Schema Generation

### JSON Schema from Types

pforge uses `schemars` to generate JSON schemas at compile time:

```rust
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, JsonSchema)]
pub struct CalculateParams {
    pub operation: String,
    pub a: f64,
    pub b: f64,
}

// At runtime, schema is available via:
let schema = schemars::schema_for!(CalculateParams);
```

**Generated JSON Schema**:

```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "title": "CalculateParams",
  "type": "object",
  "required": ["operation", "a", "b"],
  "properties": {
    "operation": {
      "type": "string",
      "description": "Operation: add, subtract, multiply, divide"
    },
    "a": {
      "type": "number"
    },
    "b": {
      "type": "number"
    }
  }
}
```

### Custom Schema Attributes

```rust
use schemars::JsonSchema;

#[derive(JsonSchema)]
pub struct AdvancedParams {
    #[schemars(regex(pattern = r"^\w+$"))]
    pub username: String,

    #[schemars(range(min = 0, max = 100))]
    pub age: u8,

    #[schemars(length(min = 8, max = 64))]
    pub password: String,

    #[schemars(default)]
    pub optional_field: Option<String>,
}
```

## Build Integration

### build.rs Script

```rust
// build.rs
use pforge_codegen::{generate_main, generate_handler_registration, generate_param_struct};
use pforge_config::ForgeConfig;
use std::fs;
use std::path::Path;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("cargo:rerun-if-changed=forge.yaml");

    // Load configuration
    let config_str = fs::read_to_string("forge.yaml")?;
    let config: ForgeConfig = serde_yaml::from_str(&config_str)?;

    // Validate
    pforge_config::validate_config(&config)?;

    // Generate code
    let out_dir = std::env::var("OUT_DIR")?;
    let dest_path = Path::new(&out_dir).join("generated.rs");

    let mut output = String::new();

    // Generate parameter structs
    for tool in &config.tools {
        if let pforge_config::ToolDef::Native { name, params, .. } = tool {
            output.push_str(&generate_param_struct(name, params)?);
            output.push_str("\n\n");
        }
    }

    // Generate handler registration
    output.push_str(&generate_handler_registration(&config)?);
    output.push_str("\n\n");

    // Generate main function
    output.push_str(&generate_main(&config)?);

    // Write to file
    fs::write(&dest_path, output)?;

    // Format with rustfmt
    std::process::Command::new("rustfmt")
        .arg(&dest_path)
        .status()?;

    Ok(())
}
```

### Including Generated Code

```rust
// src/main.rs or src/lib.rs
include!(concat!(env!("OUT_DIR"), "/generated.rs"));
```

## Error Handling and Diagnostics

### Source Location Tracking

```rust
use serde_yaml::{Mapping, Value};

#[derive(Debug)]
pub struct Spanned<T> {
    pub node: T,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct Span {
    pub start: Position,
    pub end: Position,
}

#[derive(Debug, Clone)]
pub struct Position {
    pub line: usize,
    pub column: usize,
}

impl Spanned<ForgeConfig> {
    pub fn parse(yaml_str: &str) -> Result<Self, ParseError> {
        let value: serde_yaml::Value = serde_yaml::from_str(yaml_str)?;

        // Track spans during deserialization
        let config = Self::from_value(value)?;

        Ok(config)
    }
}
```

### Pretty Error Messages

```rust
pub fn format_error(error: &CodegenError, yaml_source: &str) -> String {
    match error {
        CodegenError::DuplicateTool { name, first_location, second_location } => {
            format!(
                "Error: Duplicate tool name '{}'\n\n\
                 First defined at:  {}:{}:{}\n\
                 Also defined at:   {}:{}:{}\n",
                name,
                "forge.yaml", first_location.line, first_location.column,
                "forge.yaml", second_location.line, second_location.column
            )
        }
        CodegenError::InvalidHandlerPath { path, location } => {
            let line = yaml_source.lines().nth(location.line - 1).unwrap_or("");

            format!(
                "Error: Invalid handler path '{}'\n\n\
                 {}:{}:{}\n\
                 {}\n\
                 {}^\n\
                 Expected format: module::submodule::function_name\n",
                path,
                "forge.yaml", location.line, location.column,
                line,
                " ".repeat(location.column - 1)
            )
        }
        _ => format!("{:?}", error),
    }
}
```

## Advanced Code Generation

### Macro Generation

For repetitive patterns, pforge can generate proc macros:

```rust
// Generated macro for tool invocation
#[macro_export]
macro_rules! call_tool {
    ($registry:expr, calculate, $operation:expr, $a:expr, $b:expr) => {{
        let input = CalculateParams {
            operation: $operation.to_string(),
            a: $a,
            b: $b,
        };
        $registry.dispatch("calculate", &serde_json::to_vec(&input)?)
    }};
}

// Usage in tests
#[test]
fn test_calculate() {
    let mut registry = HandlerRegistry::new();
    register_handlers(&mut registry);

    let result = call_tool!(registry, calculate, "add", 5.0, 3.0)?;
    assert_eq!(result, 8.0);
}
```

### Optimization: Static Dispatch

For known tool sets, pforge can generate compile-time dispatch tables:

```rust
// Generated code with static dispatch
pub mod generated {
    use once_cell::sync::Lazy;
    use phf::phf_map;

    // Perfect hash map for O(1) worst-case lookup
    static HANDLER_MAP: phf::Map<&'static str, usize> = phf_map! {
        "calculate" => 0,
        "search" => 1,
        "transform" => 2,
    };

    static HANDLERS: Lazy<Vec<Box<dyn Handler>>> = Lazy::new(|| {
        vec![
            Box::new(CalculateHandler),
            Box::new(SearchHandler),
            Box::new(TransformHandler),
        ]
    });

    #[inline(always)]
    pub fn dispatch_static(tool: &str) -> Option<&dyn Handler> {
        HANDLER_MAP.get(tool)
            .and_then(|&idx| HANDLERS.get(idx))
            .map(|h| h.as_ref())
    }
}
```

## Testing Generated Code

### Snapshot Testing

```rust
// tests/codegen_test.rs
use insta::assert_snapshot;

#[test]
fn test_generate_param_struct() {
    let mut params = ParamSchema::new();
    params.add_field("name", ParamType::Simple(SimpleType::String));
    params.add_field("age", ParamType::Simple(SimpleType::Integer));

    let output = generate_param_struct("test_tool", &params).unwrap();

    assert_snapshot!(output);
}
```

```rust
// Snapshot stored in tests/snapshots/codegen_test__test_generate_param_struct.snap
---
source: tests/codegen_test.rs
expression: output
---
#[derive(Debug, Deserialize, JsonSchema)]
pub struct TestToolParams {
    pub name: String,
    pub age: i64,
}
```

### Round-Trip Testing

```rust
#[test]
fn test_config_roundtrip() {
    let yaml = include_str!("fixtures/calculator.yaml");

    // Parse YAML
    let config: ForgeConfig = serde_yaml::from_str(yaml).unwrap();

    // Generate code
    let generated = generate_all(&config).unwrap();

    // Compile generated code
    let temp_dir = TempDir::new().unwrap();
    let src_path = temp_dir.path().join("lib.rs");
    fs::write(&src_path, generated).unwrap();

    // Verify compilation
    let output = Command::new("rustc")
        .arg("--crate-type=lib")
        .arg(&src_path)
        .output()
        .unwrap();

    assert!(output.status.success());
}
```

## CLI Integration

### pforge build Command

```rust
// crates/pforge-cli/src/commands/build.rs
use pforge_codegen::Generator;
use pforge_config::ForgeConfig;

pub fn cmd_build(args: &BuildArgs) -> Result<()> {
    // Load config
    let config = ForgeConfig::load("forge.yaml")?;

    // Validate
    config.validate()?;

    // Generate code
    let generator = Generator::new(&config);
    let output = generator.generate_all()?;

    // Write to src/generated/
    let dest_dir = Path::new("src/generated");
    fs::create_dir_all(dest_dir)?;

    fs::write(dest_dir.join("mod.rs"), output)?;

    // Format
    Command::new("cargo")
        .args(&["fmt", "--", "src/generated/mod.rs"])
        .status()?;

    // Build project
    let profile = if args.release { "release" } else { "debug" };
    Command::new("cargo")
        .args(&["build", "--profile", profile])
        .status()?;

    println!("Build successful!");

    Ok(())
}
```

## Debugging Generated Code

### Preserving Generated Code

```toml
# .cargo/config.toml
[build]
# Keep generated code for inspection
target-dir = "target"

[env]
CARGO_BUILD_KEEP_GENERATED = "1"
```

```bash
# View generated code
cat target/debug/build/pforge-*/out/generated.rs | bat -l rust

# Or with syntax highlighting
rustfmt target/debug/build/pforge-*/out/generated.rs
```

### Debug Logging

```rust
// In build.rs
fn main() {
    if std::env::var("DEBUG_CODEGEN").is_ok() {
        eprintln!("=== Generated Code ===");
        eprintln!("{}", output);
        eprintln!("=== End Generated Code ===");
    }

    // ... rest of build script
}
```

```bash
# Enable debug logging
DEBUG_CODEGEN=1 cargo build
```

## Summary

pforge's code generation:

1. **Parses YAML** with full span tracking for error messages
2. **Validates** configuration for semantic correctness
3. **Transforms** config into Rust AST
4. **Generates** type-safe parameter structs, handler registration, and main function
5. **Optimizes** with static dispatch and compile-time perfect hashing
6. **Formats** with rustfmt for readable output
7. **Integrates** seamlessly with Cargo build system

**Key Benefits**:
- Type safety at compile time
- Zero runtime overhead
- Clear error messages
- Maintainable generated code

**Next chapter**: CI/CD with GitHub Actions - automating quality gates and deployment.
