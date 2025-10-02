# pforge: Declarative MCP Server Framework

**Code Name**: Pragmatic Forge  
**Version**: 0.1.0-alpha  
**License**: MIT  
**Repository**: `github.com/paiml/pforge`

## Executive Summary

pforge is a zero-boilerplate framework for building Model Context Protocol servers through declarative YAML configuration. Built on pmcp (Pragmatic AI Labs MCP SDK) and enforced by PMAT quality gates, pforge enables sub-10-line MCP server definitions with compile-time type safety and production-grade performance.

**Design Philosophy**: Cargo Lambda simplicity × Flask ergonomics × Rust guarantees

**Core Metrics**:
- Tool registration: < 5 lines of YAML per tool
- Cold start: < 100ms (including grammar cache)
- Hot path dispatch: < 1μs (pmcp baseline)
- Memory overhead: < 512KB per server instance
- Quality: TDG ≥ 0.75, zero technical debt tolerance

---

## Table of Contents

1. [Architecture Overview](#architecture-overview)
2. [Core Abstractions](#core-abstractions)
3. [YAML Configuration Schema](#yaml-configuration-schema)
4. [Implementation Roadmap](#implementation-roadmap)
5. [TDD Methodology](#tdd-methodology)
6. [Quality Gates](#quality-gates)
7. [Performance Targets](#performance-targets)
8. [Language Bridge Architecture](#language-bridge-architecture)
9. [Examples](#examples)
10. [Development Workflow](#development-workflow)

---

## Architecture Overview

### High-Level Design

```
┌─────────────────────────────────────────────────────────────┐
│                         pforge CLI                          │
│  (Scaffold, Build, Dev, Test, Quality)                     │
└─────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────┐
│                    pforge-codegen                           │
│  YAML → Rust AST → Optimized Runtime                       │
│  (Compile-time validation, type generation)                 │
└─────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────┐
│                     pforge-runtime                          │
│  • Handler Registry (O(1) dispatch)                         │
│  • Type-safe parameter validation                           │
│  • Middleware chain                                         │
│  • State management (optional)                              │
└─────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────┐
│                        pmcp v1.6+                           │
│  • TypedTool with JsonSchema generation                     │
│  • Multi-transport (stdio, SSE, WebSocket)                  │
│  • SIMD-accelerated parsing (10.3x speedup)                 │
│    └─ Leverages Langdale & Lemire (2019) simdjson algorithm │
│       for parsing gigabytes of JSON per second using SIMD   │
│       instructions (AVX2/SSE4.2)                            │
│  • Production-ready (16x faster than TS SDK)                │
└─────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────┐
│                    MCP Protocol v2024-10-07                 │
│  (JSON-RPC 2.0 over stdio/SSE/WebSocket)                   │
└─────────────────────────────────────────────────────────────┘
```

### Component Dependency Graph

```
pforge-cli
    ├── pforge-codegen
    │       ├── syn (parsing)
    │       ├── quote (codegen)
    │       └── schemars (schema)
    ├── pforge-runtime
    │       ├── pmcp (protocol)
    │       ├── tokio (async)
    │       └── dashmap (concurrent state)
    ├── pforge-macro
    │       └── proc-macro2
    └── pforge-quality
            ├── pmat (quality enforcement)
            └── criterion (benchmarking)
```

---

## Core Abstractions

### 1. Handler Trait (Zero-Cost Abstraction)

```rust
/// Core handler abstraction - compatible with pmcp TypedTool
#[async_trait::async_trait]
pub trait Handler: Send + Sync + 'static {
    type Input: JsonSchema + DeserializeOwned;
    type Output: JsonSchema + Serialize;
    type Error: Into<pforge::Error>;

    async fn handle(&self, input: Self::Input) -> Result<Self::Output, Self::Error>;
    
    /// Optional: Override for custom schema generation
    fn input_schema() -> schemars::schema::RootSchema {
        schemars::schema_for!(Self::Input)
    }
    
    fn output_schema() -> schemars::schema::RootSchema {
        schemars::schema_for!(Self::Output)
    }
}
```

### 2. Handler Registry (Compile-Time Optimized)

```rust
/// Zero-overhead handler registry with O(1) average-case lookup
/// 
/// Uses FxHash for ~2x speedup over SipHash for small keys (tool names typically <20 chars).
/// SipHash provides cryptographic resistance to hash collision attacks (Bernstein, 2012),
/// but this security property is unnecessary for internally-controlled tool name keys.
/// 
/// Future optimization: True compile-time perfect hashing (FKS algorithm; Fredman et al., 1984)
/// would provide O(1) worst-case guarantees since tool names are known at build time.
/// See: https://crates.io/crates/phf for compile-time perfect hash generation.
pub struct HandlerRegistry {
    /// FxHash for non-cryptographic, high-performance hashing
    handlers: FxHashMap<&'static str, Arc<dyn HandlerEntry>>,
    /// Pre-sorted index for binary search fallback (O(log n) worst-case)
    index: Box<[(&'static str, usize)]>,
}

trait HandlerEntry: Send + Sync {
    /// Direct dispatch without dynamic allocation
    fn dispatch(&self, params: &[u8]) -> BoxFuture<'static, Result<Vec<u8>, Error>>;
    
    /// Schema metadata (cached at construction)
    fn schema(&self) -> &JsonSchema;
}

impl HandlerRegistry {
    /// O(1) average case, O(log n) worst case
    #[inline(always)]
    pub fn dispatch(&self, tool: &str, params: &[u8]) -> BoxFuture<'static, Result<Vec<u8>, Error>> {
        match self.handlers.get(tool) {
            Some(handler) => handler.dispatch(params),
            None => Box::pin(async { Err(Error::ToolNotFound(tool.to_string())) }),
        }
    }
}
```

### 3. Configuration AST

```rust
/// Root configuration structure
#[derive(Debug, Clone, Deserialize)]
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

#[derive(Debug, Clone, Deserialize)]
pub struct ForgeMetadata {
    pub name: String,
    pub version: String,
    #[serde(default = "default_transport")]
    pub transport: TransportType,
    #[serde(default)]
    pub optimization: OptimizationLevel,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ToolDef {
    /// Rust handler (compiled into binary)
    Native {
        name: String,
        description: String,
        handler: HandlerRef,
        params: ParamSchema,
        #[serde(default)]
        timeout_ms: Option<u64>,
    },
    /// CLI wrapper with streaming support
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
    /// HTTP endpoint wrapper
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
    /// Tool composition pipeline
    Pipeline {
        name: String,
        description: String,
        steps: Vec<PipelineStep>,
    },
}

#[derive(Debug, Clone, Deserialize)]
pub struct HandlerRef {
    /// Rust path: "handlers::my_module::my_handler"
    pub path: String,
    /// Optional inline implementation for simple cases
    pub inline: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ParamSchema {
    #[serde(flatten)]
    pub fields: HashMap<String, ParamType>,
}

#[derive(Debug, Clone, Deserialize)]
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

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SimpleType {
    String,
    Integer,
    Float,
    Boolean,
    Array,
    Object,
}

#[derive(Debug, Clone, Deserialize)]
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
```

---

## YAML Configuration Schema

### Minimal Example (Hello World)

```yaml
forge:
  name: hello-server
  version: 0.1.0
  transport: stdio

tools:
  - type: native
    name: hello
    description: "Say hello to someone"
    handler:
      path: handlers::hello::say_hello
    params:
      name:
        type: string
        required: true
        description: "Name to greet"
```

### Intermediate Example (CLI + Native)

```yaml
forge:
  name: analysis-server
  version: 0.1.0
  transport: stdio
  optimization: release

tools:
  # Native Rust handler using PMAT
  - type: native
    name: analyze_complexity
    description: "Analyze code complexity using PMAT"
    handler:
      path: handlers::pmat::complexity
    params:
      path:
        type: string
        required: true
        description: "Path to analyze"
      threshold:
        type: integer
        default: 20
        validation:
          min: 1
          max: 100
    timeout_ms: 30000
    
  # CLI wrapper
  - type: cli
    name: git_status
    description: "Get git repository status"
    command: git
    args: ["status", "--porcelain"]
    cwd: "{workspace}"
    stream: true
```

### Advanced Example (Full Featured)

```yaml
forge:
  name: production-server
  version: 1.0.0
  transport: sse
  optimization: release
  
tools:
  # Complex native handler with validation
  - type: native
    name: technical_debt_grading
    description: "Comprehensive code quality analysis"
    handler:
      path: handlers::pmat::tdg_analysis
    params:
      path:
        type: string
        required: true
        pattern: "^[a-zA-Z0-9/_.-]+$"
      include_components:
        type: boolean
        default: true
      storage_backend:
        type: string
        default: "sled"
        validation:
          pattern: "^(sled|rocksdb|memory)$"
    timeout_ms: 60000
    
  # HTTP service integration
  - type: http
    name: openai_review
    description: "AI-powered code review"
    endpoint: "https://api.openai.com/v1/chat/completions"
    method: POST
    auth:
      type: bearer
      token_env: OPENAI_API_KEY
    headers:
      Content-Type: application/json
    body_template: |
      {
        "model": "gpt-4",
        "messages": [{"role": "user", "content": "{{prompt}}"}]
      }
      
  # Tool composition pipeline
  - type: pipeline
    name: analyze_and_fix
    description: "Analyze code and suggest fixes"
    steps:
      - tool: technical_debt_grading
        output_var: analysis
        error_policy: fail_fast
      - tool: openai_review
        input:
          prompt: "Review this analysis: {{analysis}}"
        condition: "{{analysis.grade}} < 'B'"
        
resources:
  - uri_template: "pmat://analysis/{path}/**"
    handler:
      path: handlers::pmat::resource_provider
    supports:
      - read
      - subscribe
    cache:
      strategy: lru
      max_size: 1000
      ttl_seconds: 300
      
prompts:
  - name: comprehensive_review
    description: "Full code review with PMAT analysis"
    template: |
      Review the following code with focus on:
      {{#if pmat_analysis}}
      - Complexity: {{pmat_analysis.cyclomatic}}
      - Technical Debt Grade: {{pmat_analysis.grade}}
      - Issues: {{pmat_analysis.satd_count}}
      {{/if}}
      
      Code:
      ```{{language}}
      {{code}}
      ```
    arguments:
      code:
        type: string
        required: true
      language:
        type: string
        default: "rust"
      pmat_analysis:
        type: object
        required: false
        
state:
  backend: sled
  path: ./forge-state
  options:
    cache_capacity: 1073741824  # 1GB
    compression: true
```

---

## Implementation Roadmap

### Phase 1: Foundation (TDD Cycle 1-10)

**Milestone**: Minimal viable server with stdio transport

#### Cycle 1: Project Scaffolding
```bash
# Test: Verify project structure creation
cargo test --test scaffold_tests::test_new_project_structure

# Implementation
pforge new hello-server
cd hello-server
tree .
# hello-server/
# ├── Cargo.toml
# ├── pforge.yaml
# ├── src/
# │   ├── main.rs
# │   └── handlers/
# │       └── mod.rs
# ├── tests/
# │   └── integration_test.rs
# └── .pmat/
#     └── quality-gates.yaml
```

**Tests**:
```rust
// tests/scaffold_tests.rs
#[test]
fn test_new_project_structure() {
    let temp = TempDir::new().unwrap();
    let project = temp.path().join("test-server");
    
    pforge::scaffold::create_project(&project, "test-server").unwrap();
    
    assert!(project.join("Cargo.toml").exists());
    assert!(project.join("pforge.yaml").exists());
    assert!(project.join("src/main.rs").exists());
    assert!(project.join("src/handlers/mod.rs").exists());
    
    // Verify Cargo.toml has correct dependencies
    let cargo_toml = std::fs::read_to_string(project.join("Cargo.toml")).unwrap();
    assert!(cargo_toml.contains("pmcp = "));
    assert!(cargo_toml.contains("pforge-runtime = "));
}

#[test]
fn test_generated_pforge_yaml_valid() {
    let temp = TempDir::new().unwrap();
    let project = temp.path().join("test-server");
    
    pforge::scaffold::create_project(&project, "test-server").unwrap();
    
    let config_path = project.join("pforge.yaml");
    let config: ForgeConfig = serde_yml::from_reader(
        std::fs::File::open(config_path).unwrap()
    ).unwrap();
    
    assert_eq!(config.forge.name, "test-server");
    assert_eq!(config.forge.transport, TransportType::Stdio);
}

#[test]
fn test_generated_project_compiles() {
    let temp = TempDir::new().unwrap();
    let project = temp.path().join("test-server");
    
    pforge::scaffold::create_project(&project, "test-server").unwrap();
    
    let output = Command::new("cargo")
        .arg("build")
        .current_dir(&project)
        .output()
        .unwrap();
    
    assert!(output.status.success(), "Generated project should compile");
}
```

#### Cycle 2: YAML Parser with Validation
```rust
// tests/config_tests.rs
#[test]
fn test_parse_minimal_config() {
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
    
    let config: ForgeConfig = serde_yml::from_str(yaml).unwrap();
    assert_eq!(config.forge.name, "test-server");
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
fn test_invalid_config_fails() {
    let yaml = r#"
forge:
  name: test-server
  # Missing required version field
  transport: stdio
"#;
    
    let result: Result<ForgeConfig, _> = serde_yml::from_str(yaml);
    assert!(result.is_err());
}

#[test]
fn test_parameter_validation_parsing() {
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
        validation:
          min: 0
          max: 150
      email:
        type: string
        validation:
          pattern: "^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\\.[a-zA-Z]{2,}$"
"#;
    
    let config: ForgeConfig = serde_yml::from_str(yaml).unwrap();
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
```

#### Cycle 3: Handler Registry Implementation
```rust
// tests/registry_tests.rs
#[tokio::test]
async fn test_handler_registration() {
    let mut registry = HandlerRegistry::new();
    
    // Register a simple handler
    registry.register("test_handler", TestHandler);
    
    assert!(registry.has_handler("test_handler"));
    assert!(!registry.has_handler("nonexistent"));
}

#[tokio::test]
async fn test_handler_dispatch() {
    let mut registry = HandlerRegistry::new();
    registry.register("echo", EchoHandler);
    
    let params = serde_json::json!({"message": "hello"});
    let result = registry.dispatch("echo", &serde_json::to_vec(&params).unwrap())
        .await
        .unwrap();
    
    let response: serde_json::Value = serde_json::from_slice(&result).unwrap();
    assert_eq!(response["message"], "hello");
}

#[tokio::test]
async fn test_nonexistent_handler_error() {
    let registry = HandlerRegistry::new();
    
    let result = registry.dispatch("nonexistent", b"{}").await;
    assert!(matches!(result, Err(Error::ToolNotFound(_))));
}

#[test]
fn test_registry_performance() {
    let mut registry = HandlerRegistry::new();
    
    // Register 1000 handlers
    for i in 0..1000 {
        registry.register(&format!("handler_{}", i), DummyHandler);
    }
    
    // Benchmark lookup time
    let start = std::time::Instant::now();
    for i in 0..10_000 {
        let _ = registry.has_handler(&format!("handler_{}", i % 1000));
    }
    let elapsed = start.elapsed();
    
    // Should complete in < 1ms (100ns per lookup)
    assert!(elapsed.as_micros() < 1000);
}
```

#### Cycle 4: Code Generation (build.rs)
```rust
// tests/codegen_tests.rs
#[test]
fn test_generate_param_struct() {
    let tool_def = ToolDef::Native {
        name: "test_tool".to_string(),
        description: "Test".to_string(),
        handler: HandlerRef {
            path: "handlers::test".to_string(),
            inline: None,
        },
        params: ParamSchema {
            fields: HashMap::from([
                ("name".to_string(), ParamType::Simple(SimpleType::String)),
                ("age".to_string(), ParamType::Simple(SimpleType::Integer)),
            ]),
        },
        timeout_ms: None,
    };
    
    let generated = codegen::generate_param_struct(&tool_def);
    
    // Verify generated code compiles and contains expected fields
    assert!(generated.contains("struct TestToolParams"));
    assert!(generated.contains("name: String"));
    assert!(generated.contains("age: i64"));
    assert!(generated.contains("derive(Debug, Serialize, Deserialize, JsonSchema)"));
}

#[test]
fn test_generate_handler_registration() {
    let config = ForgeConfig {
        forge: ForgeMetadata {
            name: "test".to_string(),
            version: "0.1.0".to_string(),
            transport: TransportType::Stdio,
            optimization: OptimizationLevel::Debug,
        },
        tools: vec![
            ToolDef::Native {
                name: "tool1".to_string(),
                description: "Tool 1".to_string(),
                handler: HandlerRef {
                    path: "handlers::tool1".to_string(),
                    inline: None,
                },
                params: ParamSchema { fields: HashMap::new() },
                timeout_ms: None,
            },
        ],
        resources: vec![],
        prompts: vec![],
        state: None,
    };
    
    let generated = codegen::generate_server(&config);
    
    assert!(generated.contains("ServerBuilder::new()"));
    assert!(generated.contains(".name(\"test\")"));
    assert!(generated.contains(".tool_typed(\"tool1\""));
}
```

#### Cycle 5: pmcp Integration
```rust
// tests/pmcp_integration_tests.rs
#[tokio::test]
async fn test_pmcp_server_initialization() {
    let config = ForgeConfig::default();
    let server = pforge::runtime::create_server(&config).unwrap();
    
    // Verify server can initialize
    assert_eq!(server.name(), "test");
}

#[tokio::test]
async fn test_typed_tool_registration() {
    let server = pmcp::ServerBuilder::new()
        .name("test")
        .version("0.1.0")
        .tool_typed("test_tool", |args: TestParams, _extra| {
            Box::pin(async move {
                Ok(serde_json::json!({"result": args.input}))
            })
        })
        .build()
        .unwrap();
    
    // Verify tool is registered
    // (Implementation detail: test through MCP protocol)
}

#[tokio::test]
async fn test_schema_generation() {
    #[derive(Debug, Serialize, Deserialize, JsonSchema)]
    struct TestParams {
        name: String,
        age: u32,
    }
    
    let schema = schemars::schema_for!(TestParams);
    
    // Verify schema has required fields
    let properties = schema.schema.object.as_ref().unwrap().properties.clone();
    assert!(properties.contains_key("name"));
    assert!(properties.contains_key("age"));
}
```

### Phase 2: Advanced Handlers (TDD Cycle 11-20)

#### Cycle 11-12: CLI Handler Implementation
```rust
// tests/cli_handler_tests.rs
#[tokio::test]
async fn test_cli_handler_basic() {
    let handler = CliHandler {
        command: "echo".to_string(),
        args: vec!["hello".to_string()],
        cwd: None,
        env: HashMap::new(),
        stream: false,
    };
    
    let result = handler.execute().await.unwrap();
    assert!(result.stdout.contains("hello"));
}

#[tokio::test]
async fn test_cli_handler_with_env() {
    let mut env = HashMap::new();
    env.insert("TEST_VAR".to_string(), "test_value".to_string());
    
    let handler = CliHandler {
        command: "printenv".to_string(),
        args: vec!["TEST_VAR".to_string()],
        cwd: None,
        env,
        stream: false,
    };
    
    let result = handler.execute().await.unwrap();
    assert!(result.stdout.contains("test_value"));
}

#[tokio::test]
async fn test_cli_handler_streaming() {
    let handler = CliHandler {
        command: "sh".to_string(),
        args: vec!["-c".to_string(), "for i in 1 2 3; do echo $i; sleep 0.1; done".to_string()],
        cwd: None,
        env: HashMap::new(),
        stream: true,
    };
    
    let mut stream = handler.execute_streaming().await.unwrap();
    let mut outputs = vec![];
    
    while let Some(line) = stream.next().await {
        outputs.push(line);
    }
    
    assert_eq!(outputs.len(), 3);
    assert_eq!(outputs[0].trim(), "1");
    assert_eq!(outputs[1].trim(), "2");
    assert_eq!(outputs[2].trim(), "3");
}

#[tokio::test]
async fn test_cli_handler_error() {
    let handler = CliHandler {
        command: "nonexistent_command_xyz".to_string(),
        args: vec![],
        cwd: None,
        env: HashMap::new(),
        stream: false,
    };
    
    let result = handler.execute().await;
    assert!(result.is_err());
}
```

#### Cycle 13-14: HTTP Handler Implementation
```rust
// tests/http_handler_tests.rs
#[tokio::test]
async fn test_http_handler_get() {
    let handler = HttpHandler {
        endpoint: "https://httpbin.org/get".parse().unwrap(),
        method: HttpMethod::Get,
        auth: None,
        headers: HashMap::new(),
        body_template: None,
    };
    
    let result = handler.execute(serde_json::json!({})).await.unwrap();
    assert!(result["url"].as_str().unwrap().contains("httpbin.org"));
}

#[tokio::test]
async fn test_http_handler_post_with_body() {
    let handler = HttpHandler {
        endpoint: "https://httpbin.org/post".parse().unwrap(),
        method: HttpMethod::Post,
        auth: None,
        headers: HashMap::new(),
        body_template: Some(r#"{"test": "{{value}}"}"#.to_string()),
    };
    
    let result = handler.execute(serde_json::json!({"value": "test_data"}))
        .await
        .unwrap();
    
    let data = &result["json"];
    assert_eq!(data["test"], "test_data");
}

#[tokio::test]
async fn test_http_handler_with_auth() {
    let handler = HttpHandler {
        endpoint: "https://httpbin.org/bearer".parse().unwrap(),
        method: HttpMethod::Get,
        auth: Some(AuthConfig::Bearer {
            token: "test_token".to_string(),
        }),
        headers: HashMap::new(),
        body_template: None,
    };
    
    let result = handler.execute(serde_json::json!({})).await.unwrap();
    assert!(result["authenticated"].as_bool().unwrap());
}
```

#### Cycle 15-16: Pipeline Handler Implementation
```rust
// tests/pipeline_tests.rs
#[tokio::test]
async fn test_pipeline_sequential_execution() {
    let pipeline = PipelineHandler {
        steps: vec![
            PipelineStep {
                tool: "step1".to_string(),
                input: None,
                output_var: Some("result1".to_string()),
                condition: None,
                error_policy: ErrorPolicy::FailFast,
            },
            PipelineStep {
                tool: "step2".to_string(),
                input: Some(serde_json::json!({"data": "{{result1}}"})),
                output_var: None,
                condition: None,
                error_policy: ErrorPolicy::FailFast,
            },
        ],
    };
    
    let mut registry = HandlerRegistry::new();
    registry.register("step1", Step1Handler);
    registry.register("step2", Step2Handler);
    
    let result = pipeline.execute(&registry, serde_json::json!({}))
        .await
        .unwrap();
    
    assert!(result["final_output"].is_object());
}

#[tokio::test]
async fn test_pipeline_conditional_execution() {
    let pipeline = PipelineHandler {
        steps: vec![
            PipelineStep {
                tool: "check".to_string(),
                input: None,
                output_var: Some("check_result".to_string()),
                condition: None,
                error_policy: ErrorPolicy::FailFast,
            },
            PipelineStep {
                tool: "action".to_string(),
                input: None,
                output_var: None,
                condition: Some("{{check_result.passed}} == true".to_string()),
                error_policy: ErrorPolicy::FailFast,
            },
        ],
    };
    
    // Test when condition is true
    let mut registry = HandlerRegistry::new();
    registry.register("check", SuccessCheckHandler);
    registry.register("action", ActionHandler);
    
    let result = pipeline.execute(&registry, serde_json::json!({}))
        .await
        .unwrap();
    assert!(result["action_executed"].as_bool().unwrap());
}

#[tokio::test]
async fn test_pipeline_error_handling() {
    let pipeline_fail_fast = PipelineHandler {
        steps: vec![
            PipelineStep {
                tool: "failing_step".to_string(),
                input: None,
                output_var: None,
                condition: None,
                error_policy: ErrorPolicy::FailFast,
            },
            PipelineStep {
                tool: "next_step".to_string(),
                input: None,
                output_var: None,
                condition: None,
                error_policy: ErrorPolicy::FailFast,
            },
        ],
    };
    
    let mut registry = HandlerRegistry::new();
    registry.register("failing_step", FailingHandler);
    registry.register("next_step", NextHandler);
    
    let result = pipeline_fail_fast.execute(&registry, serde_json::json!({})).await;
    assert!(result.is_err());
    
    // Test continue on error
    let pipeline_continue = PipelineHandler {
        steps: vec![
            PipelineStep {
                tool: "failing_step".to_string(),
                input: None,
                output_var: None,
                condition: None,
                error_policy: ErrorPolicy::Continue,
            },
            PipelineStep {
                tool: "next_step".to_string(),
                input: None,
                output_var: Some("final".to_string()),
                condition: None,
                error_policy: ErrorPolicy::FailFast,
            },
        ],
    };
    
    let result = pipeline_continue.execute(&registry, serde_json::json!({}))
        .await
        .unwrap();
    assert!(result["final"].is_object());
}
```

### Phase 3: Quality Gates Integration (TDD Cycle 21-30)

#### Cycle 21-22: PMAT Integration
```rust
// tests/pmat_integration_tests.rs
#[test]
fn test_pmat_quality_gate() {
    let project_path = std::env::current_dir().unwrap();
    
    let result = pmat::quality::run_quality_gate(&project_path, true);
    
    assert!(result.is_ok(), "Quality gates should pass");
    
    let report = result.unwrap();
    assert!(report.complexity.max_cyclomatic <= 20);
    assert_eq!(report.satd.count, 0);
    assert!(report.coverage.line_coverage > 0.80);
}

#[test]
fn test_tdg_score() {
    let source_path = std::path::Path::new("src/lib.rs");
    
    let tdg_report = pmat::analysis::technical_debt_grading(source_path).unwrap();
    
    // Enforce TDG >= 0.75
    assert!(
        tdg_report.overall_grade >= 0.75,
        "TDG score {} below threshold 0.75",
        tdg_report.overall_grade
    );
}

#[test]
fn test_no_unwraps_in_production() {
    let source_files = pmat::analysis::find_rust_files("src/").unwrap();
    
    for file in source_files {
        let content = std::fs::read_to_string(&file).unwrap();
        
        // No unwrap() calls allowed in production code (except tests)
        if !file.to_str().unwrap().contains("test") {
            assert!(
                !content.contains(".unwrap()"),
                "Found unwrap() in production code: {:?}",
                file
            );
        }
    }
}
```

#### Cycle 23-24: Performance Benchmarking
```rust
// benches/dispatch_benchmark.rs
use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};

fn bench_handler_dispatch(c: &mut Criterion) {
    let mut group = c.benchmark_group("handler_dispatch");
    
    let mut registry = HandlerRegistry::new();
    registry.register("test_handler", BenchHandler);
    
    let params = serde_json::json!({"value": 42});
    let params_bytes = serde_json::to_vec(&params).unwrap();
    
    group.bench_function("single_dispatch", |b| {
        b.iter(|| {
            let runtime = tokio::runtime::Runtime::new().unwrap();
            runtime.block_on(async {
                registry.dispatch(
                    black_box("test_handler"),
                    black_box(&params_bytes)
                ).await.unwrap()
            })
        });
    });
    
    // Target: < 1μs per dispatch
    group.bench_function("batch_dispatch", |b| {
        b.iter(|| {
            let runtime = tokio::runtime::Runtime::new().unwrap();
            runtime.block_on(async {
                for _ in 0..1000 {
                    registry.dispatch(
                        black_box("test_handler"),
                        black_box(&params_bytes)
                    ).await.unwrap();
                }
            })
        });
    });
    
    group.finish();
}

fn bench_config_parsing(c: &mut Criterion) {
    let yaml = include_str!("../fixtures/benchmark_config.yaml");
    
    c.bench_function("parse_yaml_config", |b| {
        b.iter(|| {
            let _config: ForgeConfig = serde_yml::from_str(black_box(yaml)).unwrap();
        });
    });
}

fn bench_schema_generation(c: &mut Criterion) {
    c.bench_function("generate_json_schema", |b| {
        b.iter(|| {
            let _schema = schemars::schema_for!(BenchParams);
        });
    });
}

criterion_group!(
    benches,
    bench_handler_dispatch,
    bench_config_parsing,
    bench_schema_generation
);
criterion_main!(benches);
```

### Phase 4: Production Readiness (TDD Cycle 31-40)

#### Cycle 31-32: Error Recovery and Resilience
```rust
// tests/error_recovery_tests.rs
#[tokio::test]
async fn test_timeout_handling() {
    let handler = TimeoutTestHandler {
        delay: Duration::from_secs(5),
    };
    
    let result = tokio::time::timeout(
        Duration::from_secs(1),
        handler.handle(())
    ).await;
    
    assert!(result.is_err(), "Handler should timeout");
}

#[tokio::test]
async fn test_retry_on_transient_failure() {
    let handler = RetryableHandler {
        fail_count: Arc::new(AtomicUsize::new(2)), // Fail first 2 attempts
    };
    
    let retry_config = RetryConfig {
        max_attempts: 3,
        backoff: BackoffStrategy::Exponential {
            initial: Duration::from_millis(10),
            max: Duration::from_secs(1),
        },
    };
    
    let result = retry_with_backoff(&handler, (), retry_config).await;
    assert!(result.is_ok(), "Should succeed after retries");
}

#[tokio::test]
async fn test_circuit_breaker() {
    let handler = UnreliableHandler;
    let breaker = CircuitBreaker::new(
        5,                              // failure_threshold
        Duration::from_secs(10),        // timeout
        Duration::from_secs(5),         // reset_timeout
    );
    
    // Trigger failures to open circuit
    for _ in 0..6 {
        let _ = breaker.call(&handler, ()).await;
    }
    
    assert!(breaker.is_open(), "Circuit should be open after failures");
    
    // Should fail fast without calling handler
    let start = Instant::now();
    let result = breaker.call(&handler, ()).await;
    let elapsed = start.elapsed();
    
    assert!(result.is_err());
    assert!(elapsed < Duration::from_millis(10), "Should fail immediately");
}
```

#### Cycle 33-34: State Management
```rust
// tests/state_tests.rs
#[tokio::test]
async fn test_sled_state_backend() {
    let temp = TempDir::new().unwrap();
    let state = StateManager::new(StateConfig {
        backend: StateBackend::Sled,
        path: temp.path().to_path_buf(),
        cache_capacity: 1024 * 1024,
        compression: false,
    }).await.unwrap();
    
    // Test basic operations
    state.set("key1", b"value1").await.unwrap();
    let value = state.get("key1").await.unwrap();
    assert_eq!(value, Some(b"value1".to_vec()));
    
    // Test persistence
    drop(state);
    let state = StateManager::new(StateConfig {
        backend: StateBackend::Sled,
        path: temp.path().to_path_buf(),
        cache_capacity: 1024 * 1024,
        compression: false,
    }).await.unwrap();
    
    let value = state.get("key1").await.unwrap();
    assert_eq!(value, Some(b"value1".to_vec()));
}

#[tokio::test]
async fn test_state_ttl() {
    let state = StateManager::memory();
    
    state.set_with_ttl("key1", b"value1", Duration::from_millis(100))
        .await
        .unwrap();
    
    let value = state.get("key1").await.unwrap();
    assert!(value.is_some());
    
    tokio::time::sleep(Duration::from_millis(150)).await;
    
    let value = state.get("key1").await.unwrap();
    assert!(value.is_none(), "Value should expire after TTL");
}

#[tokio::test]
async fn test_concurrent_state_access() {
    let state = Arc::new(StateManager::memory());
    let mut handles = vec![];
    
    for i in 0..100 {
        let state = state.clone();
        let handle = tokio::spawn(async move {
            let key = format!("key_{}", i);
            let value = format!("value_{}", i);
            state.set(&key, value.as_bytes()).await.unwrap();
            
            let retrieved = state.get(&key).await.unwrap().unwrap();
            assert_eq!(retrieved, value.as_bytes());
        });
        handles.push(handle);
    }
    
    for handle in handles {
        handle.await.unwrap();
    }
}
```

---

## TDD Methodology

### Extreme TDD Workflow (Toyota Way + PMAT)

```
┌─────────────────────────────────────────────────────────────┐
│                    RED → GREEN → REFACTOR                   │
│             (Strict 5-Minute Cycle Enforcement)             │
└─────────────────────────────────────────────────────────────┘

Cycle Structure:
1. RED (2 min max):   Write failing test
2. GREEN (2 min max): Minimum code to pass
3. REFACTOR (1 min):  Clean up, run quality gates
4. COMMIT:            If quality gates pass
5. RESET:             If cycle exceeds 5 minutes

Quality Gate Integration (Jidoka - "Stop the Line"):
├── Pre-commit: Run PMAT quality gates
├── Complexity: Max 20 cyclomatic per function
├── Coverage: Min 80% line coverage
├── SATD: Zero technical debt comments
└── TDG: Maintain >= 0.75 score
```

**Theoretical Foundation**: This methodology combines Beck's Test-Driven Development with Toyota Production System principles as formalized by Poppendieck & Poppendieck (2003) in *Lean Software Development*. The emphasis on rapid feedback cycles, quality built-in (Jidoka), and continuous improvement (Kaizen) directly parallels Lean manufacturing's "eliminate waste" and "amplify learning" principles. Quality gate failures halt development until resolved, embodying the TPS "stop the line" philosophy (andon cord) translated to software engineering.

### Test Organization

```
tests/
├── unit/                  # Fast tests (< 1ms each)
│   ├── config_tests.rs
│   ├── registry_tests.rs
│   └── codegen_tests.rs
├── integration/           # Integration tests (< 100ms each)
│   ├── cli_tests.rs
│   ├── server_tests.rs
│   └── pmat_integration_tests.rs
├── property/              # Property-based tests (slower)
│   ├── config_property_tests.rs
│   └── handler_property_tests.rs
└── e2e/                   # End-to-end tests
    ├── stdio_e2e_tests.rs
    ├── sse_e2e_tests.rs
    └── websocket_e2e_tests.rs
```

### Property-Based Testing Examples

```rust
// tests/property/config_property_tests.rs
use proptest::prelude::*;

proptest! {
    #[test]
    fn test_config_roundtrip(
        name in "[a-z]{3,20}",
        version in "[0-9]{1,2}\\.[0-9]{1,2}\\.[0-9]{1,2}",
    ) {
        let config = ForgeConfig {
            forge: ForgeMetadata {
                name: name.clone(),
                version: version.clone(),
                transport: TransportType::Stdio,
                optimization: OptimizationLevel::Debug,
            },
            tools: vec![],
            resources: vec![],
            prompts: vec![],
            state: None,
        };
        
        // Serialize to YAML
        let yaml = serde_yml::to_string(&config).unwrap();
        
        // Deserialize back
        let parsed: ForgeConfig = serde_yml::from_str(&yaml).unwrap();
        
        // Verify roundtrip
        assert_eq!(parsed.forge.name, name);
        assert_eq!(parsed.forge.version, version);
    }
}

proptest! {
    #[test]
    fn test_handler_dispatch_always_returns_valid_json(
        tool_name in "[a-z_]{3,20}",
        params in prop::collection::vec(any::<u8>(), 0..1024),
    ) {
        let runtime = tokio::runtime::Runtime::new().unwrap();
        let registry = HandlerRegistry::new();
        
        runtime.block_on(async {
            let result = registry.dispatch(&tool_name, &params).await;
            
            // Either error or valid JSON
            match result {
                Ok(bytes) => {
                    let _ = serde_json::from_slice::<serde_json::Value>(&bytes)
                        .expect("Handler should return valid JSON");
                }
                Err(_) => {
                    // Errors are acceptable
                }
            }
        });
    }
}
```

### Mutation Testing

```toml
# Cargo.toml
[dev-dependencies]
cargo-mutants = "24.0"

# .cargo/mutants.toml
exclude_globs = [
    "tests/**",
    "benches/**",
    "examples/**",
]

minimum_test_timeout = 60
```

```bash
# Run mutation tests
cargo mutants --check

# Target: 90%+ mutation kill rate
```

---

## Quality Gates

### Pre-Commit Hooks

```yaml
# .pmat/quality-gates.yaml
gates:
  - name: complexity
    tool: pmat analyze complexity
    max_cyclomatic: 20
    max_cognitive: 15
    fail_on_violation: true
    
  - name: technical_debt
    tool: pmat analyze satd
    max_count: 0
    fail_on_violation: true
    
  - name: test_coverage
    tool: cargo tarpaulin
    min_line_coverage: 80
    min_branch_coverage: 75
    fail_on_violation: true
    
  - name: tdg_score
    tool: pmat analyze tdg
    min_grade: 0.75
    fail_on_violation: true
    description: |
      Technical Debt Grade quantifies code quality debt using Cunningham's metaphor (1992).
      Like financial debt accruing interest, immature code becomes costlier to maintain over time.
      TDG >= 0.75 ensures technical debt remains serviceable and doesn't compound into
      development paralysis. Measured across 6 orthogonal dimensions: structural complexity,
      semantic complexity, code duplication, coupling, documentation, and consistency.
    
  - name: dead_code
    tool: pmat analyze dead-code
    max_count: 0
    fail_on_violation: true
    
  - name: lints
    tool: cargo clippy
    args: ["--", "-D", "warnings"]
    fail_on_violation: true
    
  - name: formatting
    tool: cargo fmt
    args: ["--check"]
    fail_on_violation: true
    
  - name: security_audit
    tool: cargo audit
    fail_on_violation: true
```

### CI/CD Pipeline

```yaml
# .github/workflows/quality.yml
name: Quality Gates

on: [push, pull_request]

jobs:
  quality:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      
      - name: Install Rust
        uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
          
      - name: Install PMAT
        run: cargo install pmat
        
      - name: Run Quality Gates
        run: make quality-gate
        
      - name: Run Mutation Tests
        run: cargo mutants --check
        
      - name: Upload Coverage
        uses: codecov/codecov-action@v3
        with:
          files: ./coverage.json
          
  benchmarks:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      
      - name: Run Benchmarks
        run: cargo bench --bench dispatch_benchmark
        
      - name: Performance Regression Check
        run: |
          # Fail if dispatch latency > 2μs
          if [ $(cat target/criterion/handler_dispatch/new/estimates.json | jq '.mean.point_estimate') -gt 2000 ]; then
            echo "Performance regression detected"
            exit 1
          fi
```

### Makefile Integration

```makefile
# Makefile
.PHONY: all test quality-gate dev bench

all: quality-gate test bench

# Run all tests
test:
	cargo test --all
	cargo test --all --release

# Fast development cycle (< 1 second)
dev:
	cargo watch -x 'test --lib --quiet' -x 'clippy --quiet'

# Full quality gate check
quality-gate:
	@echo "Running quality gates..."
	cargo fmt --check
	cargo clippy -- -D warnings
	cargo test --all
	cargo tarpaulin --out Json --output-path coverage.json
	pmat analyze complexity --max 20
	pmat analyze satd --max 0
	pmat analyze tdg --min 0.75
	@echo "✓ All quality gates passed"

# Run property tests (slower)
property-tests:
	cargo test --test property --release -- --test-threads=1

# Run mutation tests
mutants:
	cargo mutants --check

# Benchmarks
bench:
	cargo bench

# Performance regression check
bench-check:
	cargo bench --bench dispatch_benchmark
	@./scripts/check_performance_regression.sh

# Pre-commit hook
pre-commit: quality-gate
	@echo "✓ Ready to commit"
```

---

## Performance Targets

### Dispatch Performance

| Metric | Target | Measured | Status |
|--------|--------|----------|--------|
| Cold start | < 100ms | TBD | 🟡 |
| Tool dispatch (hot) | < 1μs | TBD | 🟡 |
| Config parse | < 10ms | TBD | 🟡 |
| Schema generation | < 1ms | TBD | 🟡 |
| Memory baseline | < 512KB | TBD | 🟡 |
| Memory per tool | < 256B | TBD | 🟡 |

### Throughput Targets

```rust
// benches/throughput_benchmark.rs
fn bench_sustained_throughput(c: &mut Criterion) {
    let mut group = c.benchmark_group("throughput");
    group.throughput(criterion::Throughput::Elements(10_000));
    
    group.bench_function("10k_sequential", |b| {
        b.iter(|| {
            let runtime = tokio::runtime::Runtime::new().unwrap();
            runtime.block_on(async {
                for _ in 0..10_000 {
                    registry.dispatch("test", &params).await.unwrap();
                }
            });
        });
    });
    
    group.bench_function("10k_concurrent", |b| {
        b.iter(|| {
            let runtime = tokio::runtime::Runtime::new().unwrap();
            runtime.block_on(async {
                let mut handles = vec![];
                for _ in 0..10_000 {
                    let registry = registry.clone();
                    let params = params.clone();
                    handles.push(tokio::spawn(async move {
                        registry.dispatch("test", &params).await.unwrap()
                    }));
                }
                for handle in handles {
                    handle.await.unwrap();
                }
            });
        });
    });
    
    // Target: > 100K req/s sequential, > 500K req/s concurrent (8 cores)
}
```

### Memory Profiling

```bash
# Memory leak detection
valgrind --leak-check=full --show-leak-kinds=all \
  target/release/pforge serve

# Heap profiling
cargo run --release --features dhat-heap

# Memory usage over time
heaptrack target/release/pforge serve
heaptrack_gui heaptrack.pforge.*.gz
```

---

## Language Bridge Architecture

### Design Principles

1. **Thin Wrapper**: Bridge is minimal FFI layer
2. **Zero-Copy**: Pass pointers, not serialized data
3. **Error Propagation**: Preserve error semantics
4. **Type Safety**: Leverage target language type systems

### Rust Bridge (pforge-bridge-rust)

```rust
// pforge-bridge-rust/src/lib.rs

/// Stable ABI for cross-language handlers
#[repr(C)]
pub struct CHandler {
    pub execute: unsafe extern "C" fn(
        params_ptr: *const u8,
        params_len: usize,
        result_ptr: *mut *mut u8,
        result_len: *mut usize,
    ) -> i32,
    pub schema: unsafe extern "C" fn() -> *const u8,
    pub cleanup: unsafe extern "C" fn(*mut u8),
}

/// Safe Rust wrapper
pub struct BridgeHandler {
    c_handler: CHandler,
}

#[async_trait]
impl Handler for BridgeHandler {
    type Input = serde_json::Value;
    type Output = serde_json::Value;
    type Error = Error;
    
    async fn handle(&self, input: Self::Input) -> Result<Self::Output, Self::Error> {
        let params = serde_json::to_vec(&input)?;
        
        let mut result_ptr: *mut u8 = std::ptr::null_mut();
        let mut result_len: usize = 0;
        
        let ret = unsafe {
            (self.c_handler.execute)(
                params.as_ptr(),
                params.len(),
                &mut result_ptr,
                &mut result_len,
            )
        };
        
        if ret != 0 {
            return Err(Error::BridgeError(ret));
        }
        
        let result_bytes = unsafe {
            std::slice::from_raw_parts(result_ptr, result_len)
        };
        
        let output = serde_json::from_slice(result_bytes)?;
        
        unsafe {
            (self.c_handler.cleanup)(result_ptr);
        }
        
        Ok(output)
    }
}
```

### Python Bridge (pforge-bridge-python)

```python
# pforge_bridge/__init__.py
import ctypes
import json
from typing import Callable, Any, Dict

class PforgeHandler:
    """Python handler that can be registered with pforge"""
    
    def __init__(self, func: Callable[[Dict], Dict]):
        self.func = func
        
    def to_c_handler(self) -> ctypes.Structure:
        """Convert Python handler to C ABI"""
        
        @ctypes.CFUNCTYPE(
            ctypes.c_int,
            ctypes.POINTER(ctypes.c_uint8),
            ctypes.c_size_t,
            ctypes.POINTER(ctypes.POINTER(ctypes.c_uint8)),
            ctypes.POINTER(ctypes.c_size_t)
        )
        def execute(params_ptr, params_len, result_ptr, result_len):
            try:
                # Deserialize input
                params_bytes = ctypes.string_at(params_ptr, params_len)
                params = json.loads(params_bytes)
                
                # Call Python handler
                result = self.func(params)
                
                # Serialize output
                result_bytes = json.dumps(result).encode('utf-8')
                result_buffer = (ctypes.c_uint8 * len(result_bytes))(*result_bytes)
                
                # Return via out parameters
                result_ptr[0] = ctypes.cast(result_buffer, ctypes.POINTER(ctypes.c_uint8))
                result_len[0] = len(result_bytes)
                
                return 0
            except Exception as e:
                print(f"Error in handler: {e}")
                return -1
        
        return execute

# Example usage
def my_handler(params: Dict) -> Dict:
    name = params.get("name", "World")
    return {"message": f"Hello, {name}!"}

handler = PforgeHandler(my_handler)
```

### Go Bridge (pforge-bridge-go)

```go
// pforge_bridge.go
package pforge

/*
#include <stdint.h>
#include <stdlib.h>

typedef int (*execute_fn)(
    const uint8_t* params_ptr,
    size_t params_len,
    uint8_t** result_ptr,
    size_t* result_len
);
*/
import "C"
import (
    "encoding/json"
    "unsafe"
)

type Handler func(params map[string]interface{}) (map[string]interface{}, error)

//export ExecuteHandler
func ExecuteHandler(
    paramsPtr *C.uint8_t,
    paramsLen C.size_t,
    resultPtr **C.uint8_t,
    resultLen *C.size_t,
) C.int {
    // Get handler from registry
    handler := getHandler()
    
    // Deserialize params
    paramsBytes := C.GoBytes(unsafe.Pointer(paramsPtr), C.int(paramsLen))
    var params map[string]interface{}
    if err := json.Unmarshal(paramsBytes, &params); err != nil {
        return -1
    }
    
    // Call handler
    result, err := handler(params)
    if err != nil {
        return -2
    }
    
    // Serialize result
    resultBytes, err := json.Marshal(result)
    if err != nil {
        return -3
    }
    
    // Allocate C memory for result
    *resultPtr = (*C.uint8_t)(C.malloc(C.size_t(len(resultBytes))))
    C.memcpy(
        unsafe.Pointer(*resultPtr),
        unsafe.Pointer(&resultBytes[0]),
        C.size_t(len(resultBytes)),
    )
    *resultLen = C.size_t(len(resultBytes))
    
    return 0
}
```

---

## Examples

### Example 1: Hello World Server

```yaml
# hello-server/pforge.yaml
forge:
  name: hello-server
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
```

```rust
// hello-server/src/handlers/hello.rs
use pforge::prelude::*;

#[derive(Debug, Deserialize, Serialize, JsonSchema)]
pub struct HelloParams {
    name: String,
}

#[derive(Debug, Deserialize, Serialize, JsonSchema)]
pub struct HelloResponse {
    message: String,
}

pub async fn say_hello(params: HelloParams) -> Result<HelloResponse, Error> {
    Ok(HelloResponse {
        message: format!("Hello, {}!", params.name),
    })
}
```

```bash
# Build and run
cd hello-server
pforge build --release
pforge serve

# Test via MCP client
echo '{"jsonrpc":"2.0","id":1,"method":"tools/call","params":{"name":"hello","arguments":{"name":"World"}}}' \
  | ./target/release/hello-server
```

### Example 2: PMAT Analysis Server

```yaml
# pmat-server/pforge.yaml
forge:
  name: pmat-server
  version: 1.0.0
  transport: stdio
  optimization: release

tools:
  - type: native
    name: analyze_complexity
    description: "Analyze code complexity"
    handler:
      path: handlers::pmat::complexity
    params:
      path:
        type: string
        required: true
      threshold:
        type: integer
        default: 20
    timeout_ms: 30000
    
  - type: native
    name: technical_debt_grading
    description: "Grade technical debt"
    handler:
      path: handlers::pmat::tdg
    params:
      path:
        type: string
        required: true
      include_components:
        type: boolean
        default: true
```

```rust
// pmat-server/src/handlers/pmat.rs
use pforge::prelude::*;
use pmat::services::code_analysis::CodeAnalysisService;

#[derive(Debug, Deserialize, Serialize, JsonSchema)]
pub struct ComplexityParams {
    path: String,
    #[serde(default = "default_threshold")]
    threshold: u32,
}

#[derive(Debug, Deserialize, Serialize, JsonSchema)]
pub struct ComplexityReport {
    pub max_cyclomatic: u32,
    pub max_cognitive: u32,
    pub violations: Vec<ComplexityViolation>,
}

pub async fn complexity(params: ComplexityParams) -> Result<ComplexityReport, Error> {
    let service = CodeAnalysisService::new();
    let analysis = service
        .analyze_complexity(&params.path, Some(params.threshold))
        .await
        .map_err(|e| Error::Analysis(e.to_string()))?;
    
    Ok(ComplexityReport {
        max_cyclomatic: analysis.max_cyclomatic,
        max_cognitive: analysis.max_cognitive,
        violations: analysis.violations.into_iter().map(Into::into).collect(),
    })
}

#[derive(Debug, Deserialize, Serialize, JsonSchema)]
pub struct TdgParams {
    path: String,
    #[serde(default)]
    include_components: bool,
}

#[derive(Debug, Deserialize, Serialize, JsonSchema)]
pub struct TdgReport {
    pub overall_grade: f64,
    pub components: Option<TdgComponents>,
}

pub async fn tdg(params: TdgParams) -> Result<TdgReport, Error> {
    let service = CodeAnalysisService::new();
    let report = service
        .analyze_tdg(&params.path, params.include_components)
        .await
        .map_err(|e| Error::Analysis(e.to_string()))?;
    
    Ok(TdgReport {
        overall_grade: report.overall_grade,
        components: params.include_components.then(|| report.components.into()),
    })
}
```

### Example 3: Multi-Language Bridge

```yaml
# polyglot-server/pforge.yaml
forge:
  name: polyglot-server
  version: 0.1.0
  transport: stdio

tools:
  # Rust handler (native)
  - type: native
    name: rust_analyze
    description: "Fast Rust analysis"
    handler:
      path: handlers::rust_tools::analyze
    params:
      data:
        type: string
        required: true
        
  # Python handler (bridge)
  - type: bridge
    name: python_ml
    description: "Python ML processing"
    language: python
    module: handlers.ml_tools
    function: process
    params:
      features:
        type: array
        required: true
        
  # Go handler (bridge)
  - type: bridge
    name: go_concurrent
    description: "Go concurrent processing"
    language: go
    package: handlers/concurrent
    function: Process
    params:
      items:
        type: array
        required: true
```

---

## Development Workflow

### Daily Development Cycle

```bash
# 1. Start development session
pforge dev --watch

# In another terminal, run tests continuously
cargo watch -x 'test --lib'

# 2. Write failing test (RED phase)
vim tests/my_feature_tests.rs

# 3. Write minimal implementation (GREEN phase)
vim src/my_feature.rs

# 4. Refactor and verify quality
pforge quality-gate

# 5. Commit if all gates pass
git add .
git commit -m "feat: implement feature X"

# Pre-commit hook runs automatically:
# - cargo fmt --check
# - cargo clippy
# - cargo test
# - pmat quality gates
```

### Release Checklist

```bash
# 1. Ensure all tests pass
make test

# 2. Run full quality gate
make quality-gate

# 3. Run mutation tests
make mutants

# 4. Run benchmarks and check for regressions
make bench-check

# 5. Update version
vim Cargo.toml  # Bump version
vim CHANGELOG.md  # Document changes

# 6. Tag release
git tag -a v0.1.0 -m "Release v0.1.0"
git push --tags

# 7. Publish to crates.io
cargo publish -p pforge-runtime
cargo publish -p pforge-codegen
cargo publish -p pforge-cli
```

### Debugging Tools

```bash
# Verbose logging
RUST_LOG=pforge=trace pforge serve

# Debug build with symbols
pforge build --debug

# Profile with flamegraph
cargo flamegraph --bin pforge -- serve

# Memory profiling
valgrind --tool=massif --massif-out-file=massif.out \
  target/release/pforge serve

# Analyze memory usage
ms_print massif.out
```

---

## Appendix: Complete Type Definitions

### Error Types

```rust
// pforge-runtime/src/error.rs
use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("Tool not found: {0}")]
    ToolNotFound(String),
    
    #[error("Invalid configuration: {0}")]
    InvalidConfig(String),
    
    #[error("Validation error: {0}")]
    Validation(String),
    
    #[error("Handler error: {0}")]
    Handler(String),
    
    #[error("Analysis error: {0}")]
    Analysis(String),
    
    #[error("Bridge error: {0}")]
    BridgeError(i32),
    
    #[error("Timeout after {0:?}")]
    Timeout(std::time::Duration),
    
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
    
    #[error("YAML error: {0}")]
    Yaml(#[from] serde_yml::Error),
    
    #[error("pmcp error: {0}")]
    Pmcp(#[from] pmcp::Error),
}

pub type Result<T> = std::result::Result<T, Error>;
```

### Transport Abstractions

```rust
// pforge-runtime/src/transport.rs
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TransportType {
    Stdio,
    Sse,
    WebSocket,
}

pub trait Transport: Send + Sync {
    fn create_client(&self) -> BoxFuture<'static, Result<Box<dyn TransportClient>>>;
}

#[async_trait]
pub trait TransportClient: Send + Sync {
    async fn initialize(&mut self) -> Result<ServerInfo>;
    async fn call_tool(&mut self, name: &str, args: Value) -> Result<Value>;
    async fn shutdown(&mut self) -> Result<()>;
}
```

---

## Repository Structure

```
pforge/
├── Cargo.toml                    # Workspace configuration
├── README.md                     # User-facing documentation
├── ARCHITECTURE.md               # This document
├── LICENSE                       # MIT license
├── Makefile                      # Development commands
├── .github/
│   ├── workflows/
│   │   ├── ci.yml               # Continuous integration
│   │   ├── quality.yml          # Quality gates
│   │   └── release.yml          # Release automation
│   └── CODEOWNERS               # Code ownership
├── .pmat/
│   └── quality-gates.yaml       # PMAT configuration
│
├── crates/
│   ├── pforge-cli/              # CLI binary
│   │   ├── Cargo.toml
│   │   ├── src/
│   │   │   ├── main.rs
│   │   │   ├── commands/
│   │   │   │   ├── new.rs
│   │   │   │   ├── build.rs
│   │   │   │   ├── serve.rs
│   │   │   │   ├── dev.rs
│   │   │   │   └── quality.rs
│   │   │   └── templates/
│   │   └── tests/
│   │
│   ├── pforge-runtime/          # Core runtime
│   │   ├── Cargo.toml
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── handler.rs
│   │   │   ├── registry.rs
│   │   │   ├── transport.rs
│   │   │   ├── state.rs
│   │   │   ├── middleware.rs
│   │   │   └── error.rs
│   │   ├── tests/
│   │   │   ├── unit/
│   │   │   ├── integration/
│   │   │   └── property/
│   │   └── benches/
│   │
│   ├── pforge-codegen/          # Code generation
│   │   ├── Cargo.toml
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── generator.rs
│   │   │   ├── schema.rs
│   │   │   └── optimize.rs
│   │   └── tests/
│   │
│   ├── pforge-config/           # Configuration parsing
│   │   ├── Cargo.toml
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── parser.rs
│   │   │   ├── validator.rs
│   │   │   └── types.rs
│   │   └── tests/
│   │
│   ├── pforge-macro/            # Procedural macros
│   │   ├── Cargo.toml
│   │   ├── src/
│   │   │   └── lib.rs
│   │   └── tests/
│   │
│   └── pforge-quality/          # Quality enforcement
│       ├── Cargo.toml
│       ├── src/
│       │   ├── lib.rs
│       │   ├── gates.rs
│       │   └── pmat.rs
│       └── tests/
│
├── bridges/                     # Language bridges
│   ├── pforge-bridge-python/
│   │   ├── setup.py
│   │   ├── pforge_bridge/
│   │   └── tests/
│   ├── pforge-bridge-go/
│   │   ├── go.mod
│   │   ├── pforge_bridge.go
│   │   └── pforge_bridge_test.go
│   └── pforge-bridge-node/
│       ├── package.json
│       ├── src/
│       └── test/
│
├── examples/                    # Example servers
│   ├── hello-world/
│   ├── pmat-server/
│   ├── polyglot-server/
│   └── production-server/
│
├── docs/                        # Documentation
│   ├── guide/
│   │   ├── getting-started.md
│   │   ├── configuration.md
│   │   └── deployment.md
│   ├── api/
│   └── architecture/
│
├── benches/                     # Benchmarks
│   ├── dispatch_benchmark.rs
│   ├── throughput_benchmark.rs
│   └── memory_benchmark.rs
│
└── scripts/                     # Utility scripts
    ├── install.sh
    ├── check_performance_regression.sh
    └── release.sh
```

---

## Success Metrics

### Technical Metrics
- [ ] All tests pass (100% pass rate)
- [ ] Test coverage > 80% (line coverage)
- [ ] TDG score ≥ 0.75
- [ ] Zero SATD comments
- [ ] Complexity ≤ 20 per function
- [ ] Mutation kill rate > 90%
- [ ] Cold start < 100ms
- [ ] Hot path dispatch < 1μs

### Quality Metrics
- [ ] Zero `unwrap()` calls in production
- [ ] Zero `panic!()` in production
- [ ] All error paths tested
- [ ] All public APIs documented
- [ ] Benchmark suite comprehensive
- [ ] Memory leak free (valgrind clean)

### Usability Metrics
- [ ] New server scaffold < 30 seconds
- [ ] Documentation complete
- [ ] Examples cover common use cases
- [ ] Error messages actionable
- [ ] IDE support (rust-analyzer works)

---

## Conclusion

pforge represents a new paradigm in MCP server development: declarative, type-safe, and production-ready by default. By building on pmcp's performance guarantees and enforcing PMAT's quality standards through extreme TDD, we create a framework that scales from prototypes to production without compromise.

**Key Innovations**:
1. YAML-driven tool composition with compile-time validation
2. Zero-cost abstraction over pmcp's TypedTool system
3. Toyota Way quality enforcement via PMAT integration
4. Language bridge architecture for polyglot ecosystems
5. Sub-microsecond dispatch with predictable performance

The framework's success will be measured not in features, but in **reliability**: every server built with pforge should pass production quality gates by default, with performance characteristics that match hand-tuned implementations.

---

**Document Version**: 1.0.0  
**Last Updated**: 2025-01-20  
**Author**: Pragmatic AI Labs  
**Status**: DRAFT - Ready for Implementation
