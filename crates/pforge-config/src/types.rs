use serde::{Deserialize, Serialize};
use std::collections::HashMap;

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

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum TransportType {
    Stdio,
    Sse,
    #[serde(rename = "websocket")]
    WebSocket,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Default)]
#[serde(rename_all = "lowercase")]
pub enum OptimizationLevel {
    #[default]
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
        endpoint: String,
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

impl ToolDef {
    pub fn name(&self) -> &str {
        match self {
            ToolDef::Native { name, .. } => name,
            ToolDef::Cli { name, .. } => name,
            ToolDef::Http { name, .. } => name,
            ToolDef::Pipeline { name, .. } => name,
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct HandlerRef {
    pub path: String,
    #[serde(default)]
    pub inline: Option<String>,
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
        #[serde(default)]
        description: Option<String>,
        #[serde(default)]
        validation: Option<Validation>,
    },
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
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

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "UPPERCASE")]
pub enum HttpMethod {
    Get,
    Post,
    Put,
    Delete,
    Patch,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum AuthConfig {
    Bearer { token: String },
    Basic { username: String, password: String },
    ApiKey { key: String, header: String },
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PipelineStep {
    pub tool: String,
    #[serde(default)]
    pub input: Option<serde_json::Value>,
    #[serde(default)]
    pub output_var: Option<String>,
    #[serde(default)]
    pub condition: Option<String>,
    #[serde(default)]
    pub error_policy: ErrorPolicy,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Default)]
#[serde(rename_all = "snake_case")]
pub enum ErrorPolicy {
    #[default]
    FailFast,
    Continue,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ResourceDef {
    pub uri_template: String,
    pub handler: HandlerRef,
    #[serde(default)]
    pub supports: Vec<ResourceOperation>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum ResourceOperation {
    Read,
    Write,
    Subscribe,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PromptDef {
    pub name: String,
    pub description: String,
    pub template: String,
    #[serde(default)]
    pub arguments: HashMap<String, ParamType>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct StateDef {
    pub backend: StateBackend,
    pub path: String,
    #[serde(default)]
    pub options: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum StateBackend {
    Sled,
    Memory,
}

fn default_transport() -> TransportType {
    TransportType::Stdio
}
