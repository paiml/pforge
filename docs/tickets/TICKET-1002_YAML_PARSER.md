# TICKET-1002: YAML Configuration Schema and Parser

**Phase**: 1 - Foundation
**Cycle**: 2
**Priority**: Critical
**Estimated Time**: 3 hours
**Status**: Ready for Development
**Methodology**: EXTREME TDD
**Depends On**: TICKET-1001

---

## Objective

Implement the complete YAML configuration schema and parser for pforge. This includes defining all configuration types (`ForgeConfig`, `ToolDef`, `ParamSchema`, etc.), implementing serde deserialization with validation, and providing comprehensive error messages for invalid configurations.

---

## Problem Statement

pforge uses declarative YAML configuration to define MCP servers. We need a robust parser that:
1. Deserializes YAML into strongly-typed Rust structs
2. Validates configuration correctness at parse time
3. Supports all tool types (Native, CLI, HTTP, Pipeline)
4. Provides actionable error messages
5. Handles parameter validation rules (min, max, pattern, etc.)

This ticket implements the configuration layer that all other components depend on.

---

## Technical Requirements

### Must Implement

1. **Core Types** (`pforge-config/src/types.rs`):
   - `ForgeConfig` - Root configuration
   - `ForgeMetadata` - Server metadata
   - `ToolDef` - Enum for all tool types
   - `ParamSchema` - Parameter definitions
   - `ParamType` - Simple and complex parameter types
   - `Validation` - Validation rules
   - `TransportType` - Enum for transports

2. **Parser** (`pforge-config/src/parser.rs`):
   - `parse_config(path: &Path) -> Result<ForgeConfig>`
   - `parse_config_from_str(yaml: &str) -> Result<ForgeConfig>`
   - Comprehensive error handling

3. **Validator** (`pforge-config/src/validator.rs`):
   - `validate_config(config: &ForgeConfig) -> Result<()>`
   - Check for duplicate tool names
   - Validate parameter schemas
   - Validate handler paths

4. **Error Types** (`pforge-config/src/error.rs`):
   - `ConfigError` with thiserror
   - Detailed error messages with line numbers

### Dependencies

```toml
[dependencies]
serde = { workspace = true }
serde_json = { workspace = true }
serde_yml = "0.0.10"
thiserror = { workspace = true }
url = "2.5"
```

---

## API Design

### Configuration Types

```rust
// pforge-config/src/types.rs

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use url::Url;

/// Root configuration structure
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
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
pub struct ForgeMetadata {
    pub name: String,
    pub version: String,
    #[serde(default = "default_transport")]
    pub transport: TransportType,
    #[serde(default)]
    pub optimization: OptimizationLevel,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum TransportType {
    Stdio,
    Sse,
    WebSocket,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum OptimizationLevel {
    Debug,
    Release,
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
        #[serde(default)]
        cwd: Option<String>,
        #[serde(default)]
        env: HashMap<String, String>,
        #[serde(default)]
        stream: bool,
    },
    Http {
        name: String,
        description: String,
        endpoint: Url,
        method: HttpMethod,
        #[serde(default)]
        auth: Option<AuthConfig>,
        #[serde(default)]
        headers: HashMap<String, String>,
    },
    Pipeline {
        name: String,
        description: String,
        steps: Vec<PipelineStep>,
    },
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ParamSchema {
    #[serde(flatten)]
    pub fields: HashMap<String, ParamType>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(untagged)]
pub enum ParamType {
    Simple(SimpleType),
    Complex {
        #[serde(rename = "type")]
        ty: SimpleType,
        #[serde(default)]
        required: bool,
        #[serde(default)]
        default: Option<serde_json::Value>,
        description: Option<String>,
        #[serde(default)]
        validation: Option<Validation>,
    },
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum SimpleType {
    String,
    Integer,
    Float,
    Boolean,
    Array,
    Object,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Validation {
    #[serde(default)]
    pub min: Option<f64>,
    #[serde(default)]
    pub max: Option<f64>,
    #[serde(default)]
    pub pattern: Option<String>,
    #[serde(default)]
    pub min_length: Option<usize>,
    #[serde(default)]
    pub max_length: Option<usize>,
}

fn default_transport() -> TransportType {
    TransportType::Stdio
}

impl Default for OptimizationLevel {
    fn default() -> Self {
        Self::Debug
    }
}
```

---

## EXTREME TDD: RED Phase Tests

### Test File: `tests/config_parser_tests.rs`

```rust
use pforge_config::*;

#[test]
fn red_test_parse_minimal_config() {
    // Expected: ❌ FAIL - Parser doesn't exist yet
    let yaml = r#"
forge:
  name: test-server
  version: 0.1.0
  transport: stdio

tools:
  - type: native
    name: hello
    description: "Say hello"
    handler:
      path: handlers::hello::say_hello
    params:
      name:
        type: string
        required: true
"#;

    let config: ForgeConfig = parse_config_from_str(yaml).unwrap();
    assert_eq!(config.forge.name, "test-server");
    assert_eq!(config.forge.version, "0.1.0");
    assert_eq!(config.tools.len(), 1);

    match &config.tools[0] {
        ToolDef::Native { name, params, .. } => {
            assert_eq!(name, "hello");
            assert!(params.fields.contains_key("name"));
        }
        _ => panic!("Expected Native tool"),
    }
}

#[test]
fn red_test_parse_all_tool_types() {
    // Expected: ❌ FAIL - Not all tool types implemented
    let yaml = r#"
forge:
  name: multi-tool
  version: 0.1.0

tools:
  - type: native
    name: native_tool
    description: "Native tool"
    handler:
      path: handlers::native
    params: {}

  - type: cli
    name: cli_tool
    description: "CLI tool"
    command: echo
    args: ["hello"]

  - type: http
    name: http_tool
    description: "HTTP tool"
    endpoint: "https://example.com/api"
    method: GET

  - type: pipeline
    name: pipeline_tool
    description: "Pipeline tool"
    steps:
      - tool: native_tool
        output_var: result
"#;

    let config: ForgeConfig = parse_config_from_str(yaml).unwrap();
    assert_eq!(config.tools.len(), 4);

    assert!(matches!(config.tools[0], ToolDef::Native { .. }));
    assert!(matches!(config.tools[1], ToolDef::Cli { .. }));
    assert!(matches!(config.tools[2], ToolDef::Http { .. }));
    assert!(matches!(config.tools[3], ToolDef::Pipeline { .. }));
}

#[test]
fn red_test_invalid_config_fails() {
    // Expected: ❌ FAIL - Error handling not implemented
    let yaml = r#"
forge:
  name: test-server
  # Missing required version field
  transport: stdio
"#;

    let result = parse_config_from_str(yaml);
    assert!(result.is_err());

    let err = result.unwrap_err();
    assert!(err.to_string().contains("version"));
}

#[test]
fn red_test_parameter_validation_parsing() {
    // Expected: ❌ FAIL - Validation not implemented
    let yaml = r#"
forge:
  name: test
  version: 0.1.0

tools:
  - type: native
    name: validate_test
    description: "Test validation"
    handler:
      path: handlers::test
    params:
      age:
        type: integer
        required: true
        validation:
          min: 0
          max: 150
      email:
        type: string
        validation:
          pattern: "^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\\.[a-zA-Z]{2,}$"
"#;

    let config: ForgeConfig = parse_config_from_str(yaml).unwrap();
    match &config.tools[0] {
        ToolDef::Native { params, .. } => {
            let age = params.fields.get("age").unwrap();
            match age {
                ParamType::Complex { validation: Some(v), .. } => {
                    assert_eq!(v.min, Some(0.0));
                    assert_eq!(v.max, Some(150.0));
                }
                _ => panic!("Expected validated parameter"),
            }
        }
        _ => panic!("Expected Native tool"),
    }
}

#[test]
fn red_test_config_roundtrip_serialization() {
    // Expected: ❌ FAIL - Serialization not working
    let yaml = r#"
forge:
  name: roundtrip
  version: 1.0.0
  transport: sse

tools: []
"#;

    let config: ForgeConfig = parse_config_from_str(yaml).unwrap();
    let serialized = serde_yml::to_string(&config).unwrap();
    let deserialized: ForgeConfig = parse_config_from_str(&serialized).unwrap();

    assert_eq!(config.forge.name, deserialized.forge.name);
    assert_eq!(config.forge.version, deserialized.forge.version);
}

#[test]
fn red_test_duplicate_tool_names_detected() {
    // Expected: ❌ FAIL - Validation not implemented
    let yaml = r#"
forge:
  name: test
  version: 0.1.0

tools:
  - type: native
    name: duplicate
    description: "First"
    handler:
      path: handlers::first
    params: {}

  - type: native
    name: duplicate
    description: "Second"
    handler:
      path: handlers::second
    params: {}
"#;

    let config: ForgeConfig = parse_config_from_str(yaml).unwrap();
    let result = validate_config(&config);
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("duplicate"));
}

// Property-based test
#[cfg(test)]
mod proptests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn proptest_config_parsing_never_panics(
            name in "[a-z]{3,20}",
            version in "[0-9]{1,2}\\.[0-9]{1,2}\\.[0-9]{1,2}",
        ) {
            let yaml = format!(r#"
forge:
  name: {}
  version: {}
  transport: stdio

tools: []
"#, name, version);

            // Should either parse correctly or return error, never panic
            let _ = parse_config_from_str(&yaml);
        }

        #[test]
        fn proptest_config_roundtrip(
            name in "[a-z]{3,20}",
            version in "[0-9]{1,2}\\.[0-9]{1,2}\\.[0-9]{1,2}",
        ) {
            let yaml = format!(r#"
forge:
  name: {}
  version: {}

tools: []
"#, name, version);

            if let Ok(config) = parse_config_from_str(&yaml) {
                let serialized = serde_yml::to_string(&config).unwrap();
                let parsed: ForgeConfig = parse_config_from_str(&serialized).unwrap();

                assert_eq!(parsed.forge.name, name);
                assert_eq!(parsed.forge.version, version);
            }
        }
    }
}
```

---

## GREEN Phase: Minimal Implementation

### Implementation Files

1. **pforge-config/src/lib.rs**
2. **pforge-config/src/types.rs** (see API Design above)
3. **pforge-config/src/parser.rs**
4. **pforge-config/src/validator.rs**
5. **pforge-config/src/error.rs**

### pforge-config/src/parser.rs

```rust
use crate::{ConfigError, ForgeConfig, Result};
use std::path::Path;

pub fn parse_config(path: &Path) -> Result<ForgeConfig> {
    let content = std::fs::read_to_string(path)
        .map_err(|e| ConfigError::IoError(path.to_path_buf(), e))?;

    parse_config_from_str(&content)
}

pub fn parse_config_from_str(yaml: &str) -> Result<ForgeConfig> {
    serde_yml::from_str(yaml)
        .map_err(|e| ConfigError::ParseError(e.to_string()))
}
```

### pforge-config/src/validator.rs

```rust
use crate::{ConfigError, ForgeConfig, Result};
use std::collections::HashSet;

pub fn validate_config(config: &ForgeConfig) -> Result<()> {
    // Check for duplicate tool names
    let mut tool_names = HashSet::new();
    for tool in &config.tools {
        let name = tool.name();
        if !tool_names.insert(name) {
            return Err(ConfigError::DuplicateToolName(name.to_string()));
        }
    }

    // Validate handler paths for native tools
    for tool in &config.tools {
        if let ToolDef::Native { handler, .. } = tool {
            validate_handler_path(&handler.path)?;
        }
    }

    Ok(())
}

fn validate_handler_path(path: &str) -> Result<()> {
    if path.is_empty() {
        return Err(ConfigError::InvalidHandlerPath("empty".to_string()));
    }
    Ok(())
}
```

---

## REFACTOR Phase

After tests pass:
1. Extract common patterns
2. Add documentation comments
3. Run clippy and fix warnings
4. Run `cargo fmt`
5. Verify quality gates

---

## Acceptance Criteria

- [x] All 7 RED tests now PASS (GREEN)
- [x] Property tests pass 10K iterations
- [x] All example configs from spec parse correctly
- [x] Validation catches duplicate tool names
- [x] Error messages are actionable
- [x] Code coverage >80%
- [x] Complexity <20, TDG >0.75
- [x] Zero clippy warnings

---

## Time Tracking

- **RED Phase**: 45 minutes (write comprehensive tests)
- **GREEN Phase**: 90 minutes (implement types, parser, validator)
- **REFACTOR Phase**: 30 minutes (cleanup, docs)
- **Property Tests**: 15 minutes
- **Total**: 3 hours

---

## Next Ticket

**TICKET-1003**: Handler Trait and Registry Foundation

---

**Status**: 📋 Ready for Development
**Created**: 2025-10-02
