# Chapter 13: Resources and Prompts

MCP servers can expose more than just tools. The Model Context Protocol supports **resources** (server-managed data sources) and **prompts** (reusable templated instructions). pforge provides first-class support for both through declarative YAML configuration and runtime managers.

## Understanding MCP Resources

Resources in MCP represent server-managed data that clients can read, write, or subscribe to. Think of them as RESTful endpoints but with MCP's type-safe protocol.

**Common use cases:**
- File system access (`file:///path/to/file`)
- Database queries (`db://users/{id}`)
- API proxies (`api://github/{owner}/{repo}`)
- Configuration data (`config://app/settings`)

### Resource Architecture

pforge's resource system is built on three core components:

1. **URI Template Matching** - Regex-based pattern matching with parameter extraction
2. **ResourceHandler Trait** - Read/write/subscribe operations
3. **ResourceManager** - O(n) URI matching and dispatch

```rust
// From crates/pforge-runtime/src/resource.rs
#[async_trait::async_trait]
pub trait ResourceHandler: Send + Sync {
    /// Read resource content
    async fn read(&self, uri: &str, params: HashMap<String, String>) -> Result<Vec<u8>>;

    /// Write resource content (if supported)
    async fn write(
        &self,
        uri: &str,
        params: HashMap<String, String>,
        content: Vec<u8>,
    ) -> Result<()> {
        let _ = (uri, params, content);
        Err(Error::Handler("Write operation not supported".to_string()))
    }

    /// Subscribe to resource changes (if supported)
    async fn subscribe(&self, uri: &str, params: HashMap<String, String>) -> Result<()> {
        let _ = (uri, params);
        Err(Error::Handler("Subscribe operation not supported".to_string()))
    }
}
```

## Defining Resources in YAML

Resources are defined in the `forge.yaml` configuration:

```yaml
forge:
  name: file-server
  version: 0.1.0
  transport: stdio

resources:
  - uri_template: "file:///{path}"
    handler:
      path: handlers::file_resource
    supports:
      - read
      - write

  - uri_template: "config://{section}/{key}"
    handler:
      path: handlers::config_resource
    supports:
      - read
      - subscribe
```

### URI Template Syntax

URI templates use `{param}` syntax for parameter extraction:

```yaml
# Simple path parameter
"file:///{path}"
# Matches: file:///home/user/test.txt
# Params: { path: "home/user/test.txt" }

# Multiple parameters
"api://{service}/{resource}"
# Matches: api://users/profile
# Params: { service: "users", resource: "profile" }

# Nested paths
"db://{database}/tables/{table}"
# Matches: db://production/tables/users
# Params: { database: "production", table: "users" }
```

**Pattern Matching Rules:**
- Parameters followed by `/` match non-greedily (single segment)
- Parameters at the end match greedily (entire path)
- Regex special characters are escaped automatically

## Implementing Resource Handlers

### Example 1: File System Resource

```rust
// src/handlers.rs
use pforge_runtime::{Error, ResourceHandler, Result};
use std::collections::HashMap;
use std::path::PathBuf;
use tokio::fs;

pub struct FileResource {
    base_path: PathBuf,
}

impl FileResource {
    pub fn new(base_path: PathBuf) -> Self {
        Self { base_path }
    }
}

#[async_trait::async_trait]
impl ResourceHandler for FileResource {
    async fn read(&self, uri: &str, params: HashMap<String, String>) -> Result<Vec<u8>> {
        let path = params
            .get("path")
            .ok_or_else(|| Error::Handler("Missing path parameter".to_string()))?;

        let full_path = self.base_path.join(path);

        // Security: Ensure path is within base directory
        let canonical = full_path
            .canonicalize()
            .map_err(|e| Error::Handler(format!("Path error: {}", e)))?;

        if !canonical.starts_with(&self.base_path) {
            return Err(Error::Handler("Path traversal detected".to_string()));
        }

        fs::read(&canonical)
            .await
            .map_err(|e| Error::Handler(format!("Failed to read file: {}", e)))
    }

    async fn write(
        &self,
        uri: &str,
        params: HashMap<String, String>,
        content: Vec<u8>,
    ) -> Result<()> {
        let path = params
            .get("path")
            .ok_or_else(|| Error::Handler("Missing path parameter".to_string()))?;

        let full_path = self.base_path.join(path);

        // Create parent directories if needed
        if let Some(parent) = full_path.parent() {
            fs::create_dir_all(parent)
                .await
                .map_err(|e| Error::Handler(format!("Failed to create directory: {}", e)))?;
        }

        fs::write(&full_path, content)
            .await
            .map_err(|e| Error::Handler(format!("Failed to write file: {}", e)))
    }
}

pub fn file_resource() -> Box<dyn ResourceHandler> {
    Box::new(FileResource::new(PathBuf::from("/tmp/file-server")))
}
```

### Example 2: Database Resource with Caching

```rust
use sled::Db;
use std::sync::Arc;

pub struct DatabaseResource {
    db: Arc<Db>,
}

impl DatabaseResource {
    pub fn new(path: &str) -> Result<Self> {
        let db = sled::open(path)
            .map_err(|e| Error::Handler(format!("Failed to open database: {}", e)))?;
        Ok(Self { db: Arc::new(db) })
    }
}

#[async_trait::async_trait]
impl ResourceHandler for DatabaseResource {
    async fn read(&self, uri: &str, params: HashMap<String, String>) -> Result<Vec<u8>> {
        let key = params
            .get("key")
            .ok_or_else(|| Error::Handler("Missing key parameter".to_string()))?;

        let db = self.db.clone();
        let key = key.clone();

        // Run blocking DB operation in thread pool
        tokio::task::spawn_blocking(move || {
            db.get(key.as_bytes())
                .map_err(|e| Error::Handler(format!("Database error: {}", e)))?
                .map(|v| v.to_vec())
                .ok_or_else(|| Error::Handler(format!("Key not found: {}", key)))
        })
        .await
        .map_err(|e| Error::Handler(format!("Task error: {}", e)))?
    }

    async fn write(
        &self,
        uri: &str,
        params: HashMap<String, String>,
        content: Vec<u8>,
    ) -> Result<()> {
        let key = params
            .get("key")
            .ok_or_else(|| Error::Handler("Missing key parameter".to_string()))?;

        let db = self.db.clone();
        let key = key.clone();

        tokio::task::spawn_blocking(move || {
            db.insert(key.as_bytes(), content)
                .map_err(|e| Error::Handler(format!("Database error: {}", e)))?;
            db.flush()
                .map_err(|e| Error::Handler(format!("Flush error: {}", e)))?;
            Ok(())
        })
        .await
        .map_err(|e| Error::Handler(format!("Task error: {}", e)))?
    }
}

pub fn db_resource() -> Box<dyn ResourceHandler> {
    DatabaseResource::new("/tmp/resource-db")
        .expect("Failed to initialize database")
        .into()
}
```

## Understanding MCP Prompts

Prompts are reusable, templated instructions that clients can discover and render. They help standardize common LLM interaction patterns across your MCP ecosystem.

**Common use cases:**
- Code review templates
- Bug report formats
- Documentation generation prompts
- Data analysis workflows

### Prompt Architecture

```rust
// From crates/pforge-runtime/src/prompt.rs
pub struct PromptManager {
    prompts: HashMap<String, PromptEntry>,
}

struct PromptEntry {
    description: String,
    template: String,
    arguments: HashMap<String, ParamType>,
}
```

**Key Features:**
- **Template Interpolation**: `{{variable}}` syntax
- **Argument Validation**: Type checking and required fields
- **Metadata Discovery**: List available prompts with schemas

## Defining Prompts in YAML

```yaml
forge:
  name: code-review-server
  version: 0.1.0

prompts:
  - name: code_review
    description: "Perform a thorough code review"
    template: |
      Review the following {{language}} code for:
      - Correctness and logic errors
      - Performance issues
      - Security vulnerabilities
      - Code style and best practices

      File: {{filename}}

      ```{{language}}
      {{code}}
      ```

      Focus on: {{focus}}
    arguments:
      language:
        type: string
        required: true
        description: "Programming language"
      filename:
        type: string
        required: true
      code:
        type: string
        required: true
        description: "The code to review"
      focus:
        type: string
        required: false
        default: "all aspects"
        description: "Specific focus areas"

  - name: bug_report
    description: "Generate a bug report from symptoms"
    template: |
      # Bug Report: {{title}}

      ## Environment
      - Version: {{version}}
      - Platform: {{platform}}

      ## Description
      {{description}}

      ## Steps to Reproduce
      {{steps}}

      ## Expected Behavior
      {{expected}}

      ## Actual Behavior
      {{actual}}
    arguments:
      title:
        type: string
        required: true
      version:
        type: string
        required: true
      platform:
        type: string
        required: true
      description:
        type: string
        required: true
      steps:
        type: string
        required: true
      expected:
        type: string
        required: true
      actual:
        type: string
        required: true
```

## Prompt Rendering

The `PromptManager` handles template interpolation at runtime:

```rust
// From crates/pforge-runtime/src/prompt.rs
impl PromptManager {
    pub fn render(&self, name: &str, args: HashMap<String, Value>) -> Result<String> {
        let entry = self
            .prompts
            .get(name)
            .ok_or_else(|| Error::Handler(format!("Prompt '{}' not found", name)))?;

        // Validate required arguments
        self.validate_arguments(entry, &args)?;

        // Perform template interpolation
        self.interpolate(&entry.template, &args)
    }

    fn interpolate(&self, template: &str, args: &HashMap<String, Value>) -> Result<String> {
        let mut result = template.to_string();

        for (key, value) in args {
            let placeholder = format!("{{{{{}}}}}", key);
            let replacement = match value {
                Value::String(s) => s.clone(),
                Value::Number(n) => n.to_string(),
                Value::Bool(b) => b.to_string(),
                Value::Null => String::new(),
                _ => serde_json::to_string(value)
                    .map_err(|e| Error::Handler(format!("Serialization error: {}", e)))?,
            };

            result = result.replace(&placeholder, &replacement);
        }

        // Check for unresolved placeholders
        if result.contains("{{") && result.contains("}}") {
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
```

**Error Handling:**
- Missing required arguments → validation error
- Unresolved placeholders → rendering error
- Type mismatches → serialization error

## Complete Example: Documentation Generator

Let's build a complete MCP server that generates documentation from code.

### forge.yaml

```yaml
forge:
  name: doc-generator
  version: 0.1.0
  transport: stdio

tools:
  - type: cli
    name: extract_symbols
    description: "Extract symbols from source code"
    command: "ctags"
    args: ["-x", "-u", "--language={{language}}", "{{file}}"]
    stream: false

resources:
  - uri_template: "file:///{path}"
    handler:
      path: handlers::file_resource
    supports:
      - read

prompts:
  - name: document_function
    description: "Generate function documentation"
    template: |
      Generate comprehensive documentation for this {{language}} function:

      ```{{language}}
      {{code}}
      ```

      Include:
      1. Brief description
      2. Parameters with types and descriptions
      3. Return value
      4. Exceptions/errors
      5. Usage example
      6. Complexity analysis (if applicable)

      Style: {{style}}
    arguments:
      language:
        type: string
        required: true
      code:
        type: string
        required: true
      style:
        type: string
        required: false
        default: "Google"
        description: "Documentation style (Google, NumPy, reStructuredText)"

  - name: document_class
    description: "Generate class documentation"
    template: |
      Generate comprehensive documentation for this {{language}} class:

      ```{{language}}
      {{code}}
      ```

      Include:
      1. Class purpose and responsibility
      2. Constructor parameters
      3. Public methods overview
      4. Usage examples
      5. Related classes
      6. Thread safety (if applicable)

      Style: {{style}}
    arguments:
      language:
        type: string
        required: true
      code:
        type: string
        required: true
      style:
        type: string
        required: false
        default: "Google"
```

### Handlers Implementation

```rust
// src/handlers.rs
use pforge_runtime::{Error, ResourceHandler, Result};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use tokio::fs;

pub struct FileResource {
    allowed_extensions: Vec<String>,
}

impl FileResource {
    pub fn new() -> Self {
        Self {
            allowed_extensions: vec![
                "rs".to_string(),
                "py".to_string(),
                "js".to_string(),
                "ts".to_string(),
                "go".to_string(),
            ],
        }
    }

    fn is_allowed(&self, path: &Path) -> bool {
        path.extension()
            .and_then(|ext| ext.to_str())
            .map(|ext| self.allowed_extensions.contains(&ext.to_lowercase()))
            .unwrap_or(false)
    }
}

#[async_trait::async_trait]
impl ResourceHandler for FileResource {
    async fn read(&self, uri: &str, params: HashMap<String, String>) -> Result<Vec<u8>> {
        let path = params
            .get("path")
            .ok_or_else(|| Error::Handler("Missing path parameter".to_string()))?;

        let file_path = PathBuf::from(path);

        // Security checks
        if !file_path.exists() {
            return Err(Error::Handler(format!("File not found: {}", path)));
        }

        if !self.is_allowed(&file_path) {
            return Err(Error::Handler(format!(
                "File type not allowed: {:?}",
                file_path.extension()
            )));
        }

        // Read file with size limit (1MB)
        let metadata = fs::metadata(&file_path)
            .await
            .map_err(|e| Error::Handler(format!("Metadata error: {}", e)))?;

        if metadata.len() > 1_048_576 {
            return Err(Error::Handler("File too large (max 1MB)".to_string()));
        }

        fs::read(&file_path)
            .await
            .map_err(|e| Error::Handler(format!("Read error: {}", e)))
    }
}

pub fn file_resource() -> Box<dyn ResourceHandler> {
    Box::new(FileResource::new())
}
```

## Testing Resources and Prompts

### Resource Tests

```rust
#[cfg(test)]
mod resource_tests {
    use super::*;
    use pforge_runtime::ResourceManager;
    use pforge_config::{ResourceDef, ResourceOperation, HandlerRef};
    use std::sync::Arc;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_file_resource_read() {
        let temp_dir = TempDir::new().unwrap();
        let test_file = temp_dir.path().join("test.txt");
        fs::write(&test_file, b"Hello, World!").await.unwrap();

        let mut manager = ResourceManager::new();
        let def = ResourceDef {
            uri_template: "file:///{path}".to_string(),
            handler: HandlerRef {
                path: "handlers::file_resource".to_string(),
                inline: None,
            },
            supports: vec![ResourceOperation::Read],
        };

        manager
            .register(def, Arc::new(FileResource::new(temp_dir.path().to_path_buf())))
            .unwrap();

        let uri = format!("file:///{}", test_file.display());
        let content = manager.read(&uri).await.unwrap();
        assert_eq!(content, b"Hello, World!");
    }

    #[tokio::test]
    async fn test_file_resource_write() {
        let temp_dir = TempDir::new().unwrap();
        let test_file = temp_dir.path().join("output.txt");

        let mut manager = ResourceManager::new();
        let def = ResourceDef {
            uri_template: "file:///{path}".to_string(),
            handler: HandlerRef {
                path: "handlers::file_resource".to_string(),
                inline: None,
            },
            supports: vec![ResourceOperation::Read, ResourceOperation::Write],
        };

        manager
            .register(def, Arc::new(FileResource::new(temp_dir.path().to_path_buf())))
            .unwrap();

        let uri = format!("file:///{}", test_file.display());
        manager.write(&uri, b"Test content".to_vec()).await.unwrap();

        let content = fs::read(&test_file).await.unwrap();
        assert_eq!(content, b"Test content");
    }

    #[tokio::test]
    async fn test_resource_unsupported_operation() {
        let mut manager = ResourceManager::new();
        let def = ResourceDef {
            uri_template: "readonly:///{path}".to_string(),
            handler: HandlerRef {
                path: "handlers::readonly_resource".to_string(),
                inline: None,
            },
            supports: vec![ResourceOperation::Read],
        };

        struct ReadOnlyResource;

        #[async_trait::async_trait]
        impl ResourceHandler for ReadOnlyResource {
            async fn read(&self, _uri: &str, _params: HashMap<String, String>) -> Result<Vec<u8>> {
                Ok(b"readonly".to_vec())
            }
        }

        manager.register(def, Arc::new(ReadOnlyResource)).unwrap();

        let result = manager.write("readonly:///test", b"data".to_vec()).await;
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("does not support write"));
    }
}
```

### Prompt Tests

```rust
#[cfg(test)]
mod prompt_tests {
    use super::*;
    use pforge_runtime::PromptManager;
    use pforge_config::{PromptDef, ParamType, SimpleType};
    use serde_json::json;

    #[test]
    fn test_prompt_render_basic() {
        let mut manager = PromptManager::new();

        let def = PromptDef {
            name: "greeting".to_string(),
            description: "Simple greeting".to_string(),
            template: "Hello, {{name}}! You are {{age}} years old.".to_string(),
            arguments: HashMap::new(),
        };

        manager.register(def).unwrap();

        let mut args = HashMap::new();
        args.insert("name".to_string(), json!("Alice"));
        args.insert("age".to_string(), json!(30));

        let result = manager.render("greeting", args).unwrap();
        assert_eq!(result, "Hello, Alice! You are 30 years old.");
    }

    #[test]
    fn test_prompt_required_validation() {
        let mut manager = PromptManager::new();

        let mut arguments = HashMap::new();
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

        let args = HashMap::new();
        let result = manager.render("greeting", args);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Required argument"));
    }

    #[test]
    fn test_prompt_unresolved_placeholder() {
        let mut manager = PromptManager::new();

        let def = PromptDef {
            name: "test".to_string(),
            description: "Test".to_string(),
            template: "Hello, {{name}}! Welcome to {{location}}.".to_string(),
            arguments: HashMap::new(),
        };

        manager.register(def).unwrap();

        let mut args = HashMap::new();
        args.insert("name".to_string(), json!("Alice"));
        // Missing 'location'

        let result = manager.render("test", args);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Unresolved template variables: location"));
    }
}
```

## Performance Considerations

### Resource Performance

**URI Matching**: O(n) linear search through registered resources
- For <10 resources: ~1μs overhead
- For 100 resources: ~10μs overhead
- Optimization: Pre-sort by specificity, try most specific first

```rust
// Potential optimization: Pattern specificity scoring
impl ResourceManager {
    fn specificity_score(pattern: &str) -> usize {
        // Fewer parameters = more specific
        pattern.matches('{').count()
    }

    pub fn register_with_priority(&mut self, def: ResourceDef, handler: Arc<dyn ResourceHandler>) {
        // Sort by specificity on insert
        self.resources.sort_by_key(|entry| entry.specificity);
    }
}
```

**Caching Strategy**: For read-heavy resources, implement caching:

```rust
use std::sync::RwLock;
use lru::LruCache;

pub struct CachedResource<R: ResourceHandler> {
    inner: R,
    cache: RwLock<LruCache<String, Vec<u8>>>,
}

#[async_trait::async_trait]
impl<R: ResourceHandler> ResourceHandler for CachedResource<R> {
    async fn read(&self, uri: &str, params: HashMap<String, String>) -> Result<Vec<u8>> {
        // Check cache
        if let Some(cached) = self.cache.read().unwrap().peek(uri).cloned() {
            return Ok(cached);
        }

        // Fetch and cache
        let content = self.inner.read(uri, params).await?;
        self.cache.write().unwrap().put(uri.to_string(), content.clone());
        Ok(content)
    }
}
```

### Prompt Performance

**Template Compilation**: Consider pre-compiling templates with a templating engine:

```rust
use handlebars::Handlebars;
use std::sync::Arc;

pub struct CompiledPromptManager {
    handlebars: Arc<Handlebars<'static>>,
    prompts: HashMap<String, PromptEntry>,
}

impl CompiledPromptManager {
    pub fn register(&mut self, def: PromptDef) -> Result<()> {
        // Pre-compile template
        self.handlebars
            .register_template_string(&def.name, &def.template)
            .map_err(|e| Error::Handler(format!("Template compilation failed: {}", e)))?;

        self.prompts.insert(def.name.clone(), PromptEntry::from(def));
        Ok(())
    }

    pub fn render(&self, name: &str, args: HashMap<String, Value>) -> Result<String> {
        self.handlebars
            .render(name, &args)
            .map_err(|e| Error::Handler(format!("Rendering failed: {}", e)))
    }
}
```

**Benchmarks** (using Criterion):

```rust
// benches/prompt_benchmark.rs
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn bench_prompt_render(c: &mut Criterion) {
    let mut manager = PromptManager::new();

    // Register complex template
    let def = PromptDef {
        name: "complex".to_string(),
        description: "Complex template".to_string(),
        template: include_str!("../fixtures/complex_template.txt").to_string(),
        arguments: HashMap::new(),
    };

    manager.register(def).unwrap();

    let args = serde_json::json!({
        "var1": "value1",
        "var2": 42,
        "var3": true,
        // ... 20 more variables
    });

    c.bench_function("prompt_render_complex", |b| {
        b.iter(|| {
            manager.render(black_box("complex"), black_box(args.clone()))
        })
    });
}

criterion_group!(benches, bench_prompt_render);
criterion_main!(benches);
```

## Best Practices

### Resource Security

1. **Path Traversal Protection**: Always validate paths
2. **Size Limits**: Enforce maximum resource sizes
3. **Rate Limiting**: Prevent resource exhaustion
4. **Allowlists**: Only expose specific URI patterns

```rust
pub struct SecureFileResource {
    base_path: PathBuf,
    max_size: u64,
    allowed_extensions: HashSet<String>,
}

impl SecureFileResource {
    async fn read(&self, uri: &str, params: HashMap<String, String>) -> Result<Vec<u8>> {
        let path = self.validate_path(&params)?;
        self.validate_extension(&path)?;
        self.validate_size(&path).await?;

        fs::read(&path).await
            .map_err(|e| Error::Handler(format!("Read error: {}", e)))
    }

    fn validate_path(&self, params: &HashMap<String, String>) -> Result<PathBuf> {
        let path = params
            .get("path")
            .ok_or_else(|| Error::Handler("Missing path".to_string()))?;

        let full_path = self.base_path.join(path);
        let canonical = full_path
            .canonicalize()
            .map_err(|_| Error::Handler("Invalid path".to_string()))?;

        if !canonical.starts_with(&self.base_path) {
            return Err(Error::Handler("Path traversal detected".to_string()));
        }

        Ok(canonical)
    }
}
```

### Prompt Design

1. **Clear Instructions**: Be explicit about format and requirements
2. **Default Values**: Provide sensible defaults for optional parameters
3. **Examples**: Include example outputs in descriptions
4. **Versioning**: Version prompts as they evolve

```yaml
prompts:
  - name: code_review_v2
    description: "Code review with enhanced security focus (v2)"
    template: |
      # Code Review Request

      ## Metadata
      - Language: {{language}}
      - File: {{filename}}
      - Reviewer Focus: {{focus}}
      - Security Level: {{security_level}}

      ## Code
      ```{{language}}
      {{code}}
      ```

      ## Review Checklist
      {{#if include_security}}
      ### Security
      - [ ] Input validation
      - [ ] SQL injection vectors
      - [ ] XSS vulnerabilities
      {{/if}}

      {{#if include_performance}}
      ### Performance
      - [ ] Algorithmic complexity
      - [ ] Memory usage
      - [ ] Database query optimization
      {{/if}}
    arguments:
      language:
        type: string
        required: true
      filename:
        type: string
        required: true
      code:
        type: string
        required: true
      focus:
        type: string
        required: false
        default: "general"
      security_level:
        type: string
        required: false
        default: "standard"
      include_security:
        type: boolean
        required: false
        default: true
      include_performance:
        type: boolean
        required: false
        default: false
```

## Integration Example

Complete server combining tools, resources, and prompts:

```yaml
forge:
  name: full-stack-assistant
  version: 1.0.0
  transport: stdio

tools:
  - type: native
    name: analyze_code
    description: "Analyze code quality and complexity"
    handler:
      path: handlers::analyze_handler
    params:
      code:
        type: string
        required: true
      language:
        type: string
        required: true

resources:
  - uri_template: "workspace:///{path}"
    handler:
      path: handlers::workspace_resource
    supports:
      - read
      - write

  - uri_template: "db://analysis/{id}"
    handler:
      path: handlers::analysis_db_resource
    supports:
      - read
      - subscribe

prompts:
  - name: full_analysis
    description: "Comprehensive code analysis workflow"
    template: |
      1. Read source file: workspace:///{{filepath}}
      2. Analyze code quality using analyze_code tool
      3. Generate report combining:
         - Complexity metrics
         - Security findings
         - Performance recommendations
      4. Store results: db://analysis/{{analysis_id}}
    arguments:
      filepath:
        type: string
        required: true
      analysis_id:
        type: string
        required: true
```

This chapter provided comprehensive coverage of pforge's resource and prompt capabilities, from basic concepts to production-ready implementations with security, testing, and performance considerations.
