/// End-to-end integration tests for pforge
/// Tests complete server lifecycle, CLI commands, and transport integration
use pforge_config::{ForgeConfig, OptimizationLevel, ToolDef, TransportType};
use std::fs;
use tempfile::TempDir;

// ============================================================================
// CLI Integration Tests
// ============================================================================

#[test]
fn test_config_validation_success() {
    let valid_config = r#"
forge:
  name: test-server
  version: 0.1.0
  transport: stdio
  optimization: release

tools:
  - type: native
    name: test_tool
    description: "Test tool"
    handler:
      path: "test::handler"
    params: {}
"#;

    let config: Result<ForgeConfig, _> = serde_yaml::from_str(valid_config);
    assert!(config.is_ok(), "Valid config should parse successfully");

    let config = config.unwrap();
    assert_eq!(config.forge.name, "test-server");
    assert_eq!(config.forge.optimization, OptimizationLevel::Release);
}

#[test]
fn test_config_validation_invalid_handler_path() {
    let invalid_config = r#"
forge:
  name: test-server
  version: 0.1.0
  transport: stdio

tools:
  - type: native
    name: test_tool
    description: "Test tool"
    handler:
      path: ""
    params: {}
"#;

    let config: Result<ForgeConfig, _> = serde_yaml::from_str(invalid_config);
    assert!(config.is_ok(), "Config should parse");

    let config = config.unwrap();
    // Handler path validation happens separately
    if let ToolDef::Native { handler, .. } = &config.tools[0] {
        assert_eq!(handler.path, "");
    }
}

#[test]
fn test_multiple_tools_same_type() {
    let config = r#"
forge:
  name: multi-tool-server
  version: 0.1.0
  transport: stdio

tools:
  - type: native
    name: tool1
    description: "First tool"
    handler:
      path: "handlers::tool1"
    params: {}

  - type: native
    name: tool2
    description: "Second tool"
    handler:
      path: "handlers::tool2"
    params: {}

  - type: native
    name: tool3
    description: "Third tool"
    handler:
      path: "handlers::tool3"
    params: {}
"#;

    let config: ForgeConfig = serde_yaml::from_str(config).unwrap();
    assert_eq!(config.tools.len(), 3);
    assert!(config
        .tools
        .iter()
        .all(|t| matches!(t, ToolDef::Native { .. })));
}

// ============================================================================
// Transport Configuration Tests
// ============================================================================

#[test]
fn test_stdio_transport_config() {
    let config = r#"
forge:
  name: stdio-server
  version: 1.0.0
  transport: stdio

tools: []
"#;

    let config: ForgeConfig = serde_yaml::from_str(config).unwrap();
    assert_eq!(config.forge.transport, TransportType::Stdio);
}

#[test]
fn test_sse_transport_config() {
    let config = r#"
forge:
  name: sse-server
  version: 1.0.0
  transport: sse

tools: []
"#;

    let config: ForgeConfig = serde_yaml::from_str(config).unwrap();
    assert_eq!(config.forge.transport, TransportType::Sse);
}

#[test]
fn test_websocket_transport_config() {
    let config = r#"
forge:
  name: ws-server
  version: 1.0.0
  transport: websocket

tools: []
"#;

    let config: ForgeConfig = serde_yaml::from_str(config).unwrap();
    assert_eq!(config.forge.transport, TransportType::WebSocket);
}

// ============================================================================
// Handler Type Tests
// ============================================================================

#[test]
fn test_cli_handler_configuration() {
    let config = r#"
forge:
  name: cli-server
  version: 1.0.0
  transport: stdio

tools:
  - type: cli
    name: echo_test
    description: "Echo command test"
    command: echo
    args: ["hello", "world"]
"#;

    let config: ForgeConfig = serde_yaml::from_str(config).unwrap();
    assert_eq!(config.tools.len(), 1);

    if let ToolDef::Cli {
        name,
        command,
        args,
        ..
    } = &config.tools[0]
    {
        assert_eq!(name, "echo_test");
        assert_eq!(command, "echo");
        assert_eq!(args.len(), 2);
    } else {
        panic!("Expected CLI tool");
    }
}

#[test]
fn test_http_handler_configuration() {
    let config = r#"
forge:
  name: http-server
  version: 1.0.0
  transport: stdio

tools:
  - type: http
    name: api_call
    description: "HTTP API call"
    endpoint: https://api.example.com/data
    method: POST
"#;

    let config: ForgeConfig = serde_yaml::from_str(config).unwrap();
    assert_eq!(config.tools.len(), 1);

    if let ToolDef::Http {
        name,
        endpoint,
        method,
        ..
    } = &config.tools[0]
    {
        assert_eq!(name, "api_call");
        assert!(endpoint.contains("api.example.com"));
        assert_eq!(method, &pforge_config::HttpMethod::Post);
    } else {
        panic!("Expected HTTP tool");
    }
}

#[test]
fn test_pipeline_handler_configuration() {
    let config = r#"
forge:
  name: pipeline-server
  version: 1.0.0
  transport: stdio

tools:
  - type: native
    name: step1
    description: "First step"
    handler:
      path: "handlers::step1"
    params: {}

  - type: native
    name: step2
    description: "Second step"
    handler:
      path: "handlers::step2"
    params: {}

  - type: pipeline
    name: full_pipeline
    description: "Multi-step pipeline"
    steps:
      - tool: step1
        output_key: step1_result
      - tool: step2
        input_from: step1_result
"#;

    let config: ForgeConfig = serde_yaml::from_str(config).unwrap();
    assert_eq!(config.tools.len(), 3);

    // Verify pipeline tool exists
    let pipeline = config.tools.iter().find(|t| match t {
        ToolDef::Pipeline { name, .. } => name == "full_pipeline",
        _ => false,
    });

    assert!(pipeline.is_some(), "Pipeline tool should exist");

    if let Some(ToolDef::Pipeline { steps, .. }) = pipeline {
        assert_eq!(steps.len(), 2);
    }
}

// ============================================================================
// Parameter Schema Tests
// ============================================================================

#[test]
fn test_required_parameters() {
    let config = r#"
forge:
  name: param-server
  version: 1.0.0
  transport: stdio

tools:
  - type: native
    name: with_params
    description: "Tool with parameters"
    handler:
      path: "handlers::with_params"
    params:
      required_string:
        type: string
        required: true
      optional_number:
        type: integer
        required: false
"#;

    let config: ForgeConfig = serde_yaml::from_str(config).unwrap();

    if let ToolDef::Native { params, .. } = &config.tools[0] {
        assert_eq!(params.fields.len(), 2);
        assert!(params.fields.contains_key("required_string"));
        assert!(params.fields.contains_key("optional_number"));
    } else {
        panic!("Expected native tool");
    }
}

#[test]
fn test_array_and_object_parameters() {
    let config = r#"
forge:
  name: complex-params
  version: 1.0.0
  transport: stdio

tools:
  - type: native
    name: complex_tool
    description: "Tool with complex parameters"
    handler:
      path: "handlers::complex"
    params:
      items:
        type: array
        required: true
      metadata:
        type: object
        required: false
"#;

    let config: ForgeConfig = serde_yaml::from_str(config).unwrap();

    if let ToolDef::Native { params, .. } = &config.tools[0] {
        assert_eq!(params.fields.len(), 2);
    } else {
        panic!("Expected native tool");
    }
}

// ============================================================================
// State and Resources Tests
// ============================================================================

#[test]
fn test_state_configuration() {
    let config = r#"
forge:
  name: stateful-server
  version: 1.0.0
  transport: stdio

tools: []

state:
  backend: memory
  path: "/tmp/pforge-state"
"#;

    let config: ForgeConfig = serde_yaml::from_str(config).unwrap();

    if let Some(state) = config.state {
        assert_eq!(state.backend, pforge_config::StateBackend::Memory);
        assert_eq!(state.path, "/tmp/pforge-state");
    } else {
        panic!("Expected state configuration");
    }
}

#[test]
fn test_resources_configuration() {
    let config = r#"
forge:
  name: resource-server
  version: 1.0.0
  transport: stdio

tools: []

resources:
  - uri_template: "file:///data/{filename}"
    handler:
      path: "handlers::read_file"
    supports: [read]
"#;

    let config: ForgeConfig = serde_yaml::from_str(config).unwrap();

    assert_eq!(config.resources.len(), 1);
    assert_eq!(config.resources[0].uri_template, "file:///data/{filename}");
    assert_eq!(config.resources[0].handler.path, "handlers::read_file");
}

#[test]
fn test_prompts_configuration() {
    let config = r#"
forge:
  name: prompt-server
  version: 1.0.0
  transport: stdio

tools: []

prompts:
  - name: greeting
    description: "Greeting prompt"
    template: "Hello, {{name}}!"
    arguments:
      name:
        type: string
        required: true
"#;

    let config: ForgeConfig = serde_yaml::from_str(config).unwrap();

    assert_eq!(config.prompts.len(), 1);
    assert_eq!(config.prompts[0].name, "greeting");
    assert_eq!(config.prompts[0].arguments.len(), 1);
}

// ============================================================================
// Full Server Configuration Test
// ============================================================================

#[test]
fn test_complete_server_configuration() {
    let config = r#"
forge:
  name: complete-server
  version: 1.0.0
  transport: stdio
  optimization: release

tools:
  - type: native
    name: calculator_add
    description: "Add two numbers"
    handler:
      path: "handlers::add"
    params:
      a:
        type: integer
        required: true
      b:
        type: integer
        required: true

  - type: cli
    name: system_info
    description: "Get system information"
    command: uname
    args: ["-a"]

  - type: http
    name: fetch_data
    description: "Fetch external data"
    endpoint: https://api.example.com/data
    method: GET

state:
  backend: memory
  path: "/tmp/pforge-state"

resources:
  - uri_template: "file:///data/{filename}"
    handler:
      path: "handlers::read_file"

prompts:
  - name: ask_question
    description: "Ask a question"
    template: "Question: {{question}}"
    arguments:
      question:
        type: string
        required: true
"#;

    let config: ForgeConfig = serde_yaml::from_str(config).unwrap();

    // Verify forge config
    assert_eq!(config.forge.name, "complete-server");
    assert_eq!(config.forge.version, "1.0.0");
    assert_eq!(config.forge.transport, TransportType::Stdio);
    assert_eq!(config.forge.optimization, OptimizationLevel::Release);

    // Verify tools
    assert_eq!(config.tools.len(), 3);

    // Verify state
    assert!(config.state.is_some());

    // Verify resources
    assert_eq!(config.resources.len(), 1);

    // Verify prompts
    assert_eq!(config.prompts.len(), 1);
}

// ============================================================================
// Error Handling Tests
// ============================================================================

#[test]
fn test_malformed_yaml() {
    let bad_yaml = r#"
forge:
  name: broken
  version: 1.0.0
  transport: stdio
tools:
  - type: native
    name: test
    missing_required_field: true
"#;

    let result: Result<ForgeConfig, _> = serde_yaml::from_str(bad_yaml);
    assert!(result.is_err(), "Malformed YAML should fail to parse");
}

#[test]
fn test_invalid_transport_type() {
    let config = r#"
forge:
  name: test
  version: 1.0.0
  transport: invalid_transport

tools: []
"#;

    let result: Result<ForgeConfig, _> = serde_yaml::from_str(config);
    assert!(result.is_err(), "Invalid transport should fail to parse");
}

#[test]
fn test_invalid_optimization_level() {
    let config = r#"
forge:
  name: test
  version: 1.0.0
  transport: stdio
  optimization: invalid_level

tools: []
"#;

    let result: Result<ForgeConfig, _> = serde_yaml::from_str(config);
    assert!(
        result.is_err(),
        "Invalid optimization level should fail to parse"
    );
}

// ============================================================================
// File-based Configuration Tests
// ============================================================================

#[test]
fn test_load_config_from_file() {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("pforge.yaml");

    let config_content = r#"
forge:
  name: file-based-server
  version: 1.0.0
  transport: stdio

tools:
  - type: native
    name: test_tool
    description: "Test tool"
    handler:
      path: "handlers::test"
    params: {}
"#;

    fs::write(&config_path, config_content).unwrap();

    let loaded_config = pforge_config::parse_config(&config_path).unwrap();
    assert_eq!(loaded_config.forge.name, "file-based-server");
    assert_eq!(loaded_config.tools.len(), 1);
}

#[test]
fn test_missing_config_file() {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("nonexistent.yaml");

    let result = pforge_config::parse_config(&config_path);
    assert!(result.is_err(), "Missing file should return error");
}

// ============================================================================
// Optimization Level Tests
// ============================================================================

#[test]
fn test_debug_optimization() {
    let config = r#"
forge:
  name: debug-server
  version: 1.0.0
  transport: stdio
  optimization: debug

tools: []
"#;

    let config: ForgeConfig = serde_yaml::from_str(config).unwrap();
    assert_eq!(config.forge.optimization, OptimizationLevel::Debug);
}

#[test]
fn test_release_optimization() {
    let config = r#"
forge:
  name: release-server
  version: 1.0.0
  transport: stdio
  optimization: release

tools: []
"#;

    let config: ForgeConfig = serde_yaml::from_str(config).unwrap();
    assert_eq!(config.forge.optimization, OptimizationLevel::Release);
}
