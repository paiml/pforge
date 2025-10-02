# CLI Wrappers: Argument Templating and Output Parsing

CLI wrappers transform shell commands into type-safe MCP tools. This chapter covers advanced argument handling, parameter interpolation, and output parsing strategies.

## Argument Flow Architecture

Understanding how arguments flow through CLI handlers:

```
YAML Config       User Input        Command Execution
-----------       ----------        -----------------
command: git      params: {         git
args: [           repo: "/foo",  -> -C /foo
  "-C",           format: "json"    log
  "{{repo}}",     }                 --format=json
  "log",
  "--format={{format}}"
]
```

## Parameter Interpolation

### Basic String Substitution

```yaml
tools:
  - type: cli
    name: docker_run
    description: "Run a Docker container"
    command: docker
    args:
      - "run"
      - "--name"
      - "{{container_name}}"
      - "{{image}}"
    params:
      container_name:
        type: string
        required: true
      image:
        type: string
        required: true
```

**Execution**:
```json
// Input
{ "container_name": "my-app", "image": "nginx:latest" }

// Command
docker run --name my-app nginx:latest
```

### Multiple Parameter Types

```yaml
tools:
  - type: cli
    name: ffmpeg_convert
    description: "Convert video files"
    command: ffmpeg
    args:
      - "-i"
      - "{{input_file}}"
      - "-b:v"
      - "{{bitrate}}k"
      - "-r"
      - "{{framerate}}"
      - "{{output_file}}"
    params:
      input_file:
        type: string
        required: true
      bitrate:
        type: integer
        required: false
        default: 1000
      framerate:
        type: integer
        required: false
        default: 30
      output_file:
        type: string
        required: true
```

**Type conversion**:
- `string` → passed as-is
- `integer` → converted to string
- `float` → converted to string
- `boolean` → "true" or "false"

### Conditional Arguments

For conditional arguments, use a Native handler wrapper:

```rust
use pforge_runtime::{Handler, Result, Error};
use tokio::process::Command;

#[derive(Deserialize, JsonSchema)]
struct GrepInput {
    pattern: String,
    file: String,
    case_insensitive: bool,
    line_numbers: bool,
}

#[derive(Serialize, JsonSchema)]
struct GrepOutput {
    stdout: String,
    stderr: String,
    exit_code: i32,
}

pub struct GrepHandler;

#[async_trait::async_trait]
impl Handler for GrepHandler {
    type Input = GrepInput;
    type Output = GrepOutput;
    type Error = Error;

    async fn handle(&self, input: Self::Input) -> Result<Self::Output> {
        let mut cmd = Command::new("grep");

        if input.case_insensitive {
            cmd.arg("-i");
        }

        if input.line_numbers {
            cmd.arg("-n");
        }

        cmd.arg(&input.pattern);
        cmd.arg(&input.file);

        let output = cmd.output().await
            .map_err(|e| Error::Handler(format!("grep failed: {}", e)))?;

        Ok(GrepOutput {
            stdout: String::from_utf8_lossy(&output.stdout).to_string(),
            stderr: String::from_utf8_lossy(&output.stderr).to_string(),
            exit_code: output.status.code().unwrap_or(-1),
        })
    }
}
```

**Why Native for conditional args?**
- YAML is declarative, not conditional
- Rust provides full control over arg construction
- Type-safe boolean-to-flag conversion

## Output Parsing Strategies

### Strategy 1: Raw Output (Default)

```yaml
tools:
  - type: cli
    name: list_files
    command: ls
    args: ["-lah"]
```

**Output**:
```json
{
  "stdout": "total 24K\ndrwxr-xr-x 3 user user 4.0K...",
  "stderr": "",
  "exit_code": 0
}
```

**Use when**: Client will parse output (LLMs are good at this!)

### Strategy 2: Structured Output with jq

```yaml
tools:
  - type: cli
    name: docker_inspect
    description: "Get Docker container details as JSON"
    command: sh
    args:
      - "-c"
      - "docker inspect {{container}} | jq -c '.[0]'"
    params:
      container:
        type: string
        required: true
```

**Output**:
```json
{
  "stdout": "{\"Id\":\"abc123\",\"Name\":\"my-app\",\"State\":{\"Status\":\"running\"}}",
  "stderr": "",
  "exit_code": 0
}
```

**Client parsing**:
```javascript
const result = await client.callTool("docker_inspect", { container: "my-app" });
const parsed = JSON.parse(result.stdout);
console.log(parsed.State.Status); // "running"
```

### Strategy 3: Native Handler Post-Processing

```rust
#[derive(Serialize, JsonSchema)]
struct ProcessedOutput {
    files: Vec<FileInfo>,
    total_size: u64,
}

#[derive(Serialize, JsonSchema)]
struct FileInfo {
    name: String,
    size: u64,
    modified: String,
}

pub struct LsHandler;

#[async_trait::async_trait]
impl Handler for LsHandler {
    type Input = LsInput;
    type Output = ProcessedOutput;
    type Error = Error;

    async fn handle(&self, input: Self::Input) -> Result<Self::Output> {
        let output = Command::new("ls")
            .arg("-lh")
            .arg(&input.directory)
            .output()
            .await?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        let files = parse_ls_output(&stdout)?;
        let total_size = files.iter().map(|f| f.size).sum();

        Ok(ProcessedOutput {
            files,
            total_size,
        })
    }
}

fn parse_ls_output(output: &str) -> Result<Vec<FileInfo>> {
    // Parse ls -lh output into structured data
    output.lines()
        .skip(1) // Skip "total" line
        .map(|line| {
            let parts: Vec<&str> = line.split_whitespace().collect();
            Ok(FileInfo {
                name: parts.last().unwrap_or(&"").to_string(),
                size: parse_size(parts.get(4).unwrap_or(&"0"))?,
                modified: format!("{} {} {}",
                    parts.get(5).unwrap_or(&""),
                    parts.get(6).unwrap_or(&""),
                    parts.get(7).unwrap_or(&"")),
            })
        })
        .collect()
}
```

**Use when**:
- Output needs transformation
- Type safety required downstream
- Complex parsing logic

### Strategy 4: Streaming Parser

For large outputs, parse incrementally:

```rust
use tokio::io::{AsyncBufReadExt, BufReader};

pub async fn stream_parse_logs(
    command: &str,
    args: &[String],
) -> Result<Vec<LogEntry>> {
    let mut child = Command::new(command)
        .args(args)
        .stdout(Stdio::piped())
        .spawn()?;

    let stdout = child.stdout.take()
        .ok_or_else(|| Error::Handler("Failed to capture stdout".into()))?;

    let reader = BufReader::new(stdout);
    let mut lines = reader.lines();
    let mut entries = Vec::new();

    while let Some(line) = lines.next_line().await? {
        if let Ok(entry) = parse_log_line(&line) {
            entries.push(entry);
        }
    }

    Ok(entries)
}
```

## Working Directory Management

### Static Working Directory

```yaml
tools:
  - type: cli
    name: npm_install
    command: npm
    args: ["install"]
    cwd: /home/user/project
```

**Security**: Safe - directory is hardcoded.

### Dynamic Working Directory (Requires Native)

```rust
#[derive(Deserialize, JsonSchema)]
struct NpmInput {
    project_path: String,
}

async fn handle(&self, input: NpmInput) -> Result<NpmOutput> {
    // Validate path is safe
    validate_project_path(&input.project_path)?;

    let output = Command::new("npm")
        .arg("install")
        .current_dir(&input.project_path)
        .output()
        .await?;

    // ... return output
}

fn validate_project_path(path: &str) -> Result<()> {
    // Prevent directory traversal
    if path.contains("..") {
        return Err(Error::Validation("Invalid path".into()));
    }

    // Ensure path exists and is a directory
    let path_obj = std::path::Path::new(path);
    if !path_obj.is_dir() {
        return Err(Error::Validation("Not a directory".into()));
    }

    Ok(())
}
```

## Environment Variable Handling

### Static Environment Variables

```yaml
tools:
  - type: cli
    name: run_script
    command: ./script.sh
    env:
      NODE_ENV: production
      LOG_LEVEL: info
      API_URL: https://api.example.com
```

### Dynamic Environment Variables

CLI handlers accept env vars at runtime:

```yaml
tools:
  - type: cli
    name: aws_cli
    command: aws
    args: ["s3", "ls"]
    env:
      AWS_REGION: us-east-1
    params:
      bucket:
        type: string
        required: true
```

**Runtime override**:
```json
{
  "tool": "aws_cli",
  "params": {
    "bucket": "my-bucket",
    "env": {
      "AWS_REGION": "eu-west-1"  // Overrides static value
    }
  }
}
```

**Merge strategy**:
1. Start with system environment
2. Apply static YAML env vars
3. Apply runtime input env vars (highest priority)

## Exit Code Handling

CLI handlers don't fail on non-zero exit codes - they return the code:

```json
{
  "stdout": "",
  "stderr": "grep: pattern not found",
  "exit_code": 1
}
```

**Client-side handling**:

```javascript
const result = await client.callTool("grep_files", { pattern: "TODO" });

if (result.exit_code !== 0) {
  if (result.exit_code === 1) {
    console.log("Pattern not found (expected)");
  } else {
    throw new Error(`grep failed: ${result.stderr}`);
  }
}
```

**Native handler with validation**:

```rust
async fn handle(&self, input: Input) -> Result<Output> {
    let output = Command::new("grep")
        .args(&input.args)
        .output()
        .await?;

    let exit_code = output.status.code().unwrap_or(-1);

    match exit_code {
        0 => Ok(Output {
            stdout: String::from_utf8_lossy(&output.stdout).to_string(),
        }),
        1 => Ok(Output {
            stdout: String::new(), // Pattern not found - not an error
        }),
        _ => Err(Error::Handler(format!(
            "grep failed with code {}: {}",
            exit_code,
            String::from_utf8_lossy(&output.stderr)
        ))),
    }
}
```

## Complex Command Construction

### Multi-command Pipelines

```yaml
tools:
  - type: cli
    name: count_rust_files
    command: sh
    args:
      - "-c"
      - "find {{directory}} -name '*.rs' | wc -l"
    params:
      directory:
        type: string
        required: true
```

**Security note**: Use `sh -c` sparingly - validate input thoroughly!

### Argument Quoting

pforge automatically quotes arguments with spaces:

```yaml
command: git
args:
  - "commit"
  - "-m"
  - "{{message}}"

# Input: { "message": "fix: resolve bug #123" }
# Executes: git commit -m "fix: resolve bug #123"
```

**Manual quoting not needed** - pforge handles it.

## Real-World Example: Docker Wrapper

```yaml
forge:
  name: docker-wrapper
  version: 0.1.0
  transport: stdio

tools:
  - type: cli
    name: docker_ps
    description: "List running containers"
    command: docker
    args: ["ps", "--format", "json"]

  - type: cli
    name: docker_logs
    description: "Get container logs"
    command: docker
    args: ["logs", "--tail", "{{lines}}", "{{container}}"]
    timeout_ms: 10000
    params:
      container:
        type: string
        required: true
      lines:
        type: integer
        required: false
        default: 100

  - type: cli
    name: docker_exec
    description: "Execute command in container"
    command: docker
    args: ["exec", "-i", "{{container}}", "{{command}}"]
    params:
      container:
        type: string
        required: true
      command:
        type: string
        required: true

  - type: cli
    name: docker_stats
    description: "Stream container stats"
    command: docker
    args: ["stats", "--no-stream", "--format", "json"]
    stream: true
```

## Testing CLI Wrappers

### Unit Test: Argument Construction

```rust
#[test]
fn test_cli_handler_builds_args_correctly() {
    let handler = CliHandler::new(
        "git".to_string(),
        vec!["log".to_string(), "--oneline".to_string()],
        None,
        HashMap::new(),
        None,
        false,
    );

    assert_eq!(handler.command, "git");
    assert_eq!(handler.args, vec!["log", "--oneline"]);
}
```

### Integration Test: Full Execution

```rust
#[tokio::test]
async fn test_cli_wrapper_git_log() {
    let handler = CliHandler::new(
        "git".to_string(),
        vec!["log".to_string(), "--oneline".to_string(), "-n".to_string()],
        Some("/path/to/repo".to_string()),
        HashMap::new(),
        Some(5000),
        false,
    );

    let input = CliInput {
        args: vec!["5".to_string()],
        env: HashMap::new(),
    };

    let result = handler.execute(input).await.unwrap();
    assert_eq!(result.exit_code, 0);
    assert!(!result.stdout.is_empty());
}
```

### Property Test: Exit Code Range

```rust
use proptest::prelude::*;

proptest! {
    #[test]
    fn cli_handler_returns_valid_exit_code(
        cmd in "[a-z]{1,10}",
        args in prop::collection::vec("[a-z]{1,5}", 0..5)
    ) {
        tokio_test::block_on(async {
            let handler = CliHandler::new(
                cmd,
                args,
                None,
                HashMap::new(),
                Some(1000),
                false,
            );

            let result = handler.execute(CliInput::default()).await;

            if let Ok(output) = result {
                prop_assert!(output.exit_code >= -1);
                prop_assert!(output.exit_code <= 255);
            }
        });
    }
}
```

## Performance Optimization

### Reuse Command Instances

Don't recreate CLI handlers per request:

```rust
// SLOW - recreates handler each time
pub async fn slow_wrapper(input: Input) -> Result<Output> {
    let handler = CliHandler::new(...);
    handler.execute(input).await
}

// FAST - reuse handler instance
pub struct FastWrapper {
    handler: CliHandler,
}

impl FastWrapper {
    pub fn new() -> Self {
        Self {
            handler: CliHandler::new(...),
        }
    }

    pub async fn execute(&self, input: Input) -> Result<Output> {
        self.handler.execute(input).await
    }
}
```

### Minimize Argument Allocations

pforge optimizes argument building - but you can help:

```yaml
# SLOW - many small allocations
args: ["--opt1", "{{val1}}", "--opt2", "{{val2}}", "--opt3", "{{val3}}"]

# FAST - fewer, larger args
args: ["--config", "{{config_file}}"]  # Config file contains all options
```

## Common Pitfalls

### Pitfall 1: Shell Metacharacter Injection

```yaml
# UNSAFE
command: sh
args: ["-c", "ls {{user_input}}"]

# Input: { "user_input": "; rm -rf /" }
# Executes: ls ; rm -rf /   # DANGER!
```

**Fix**: Validate input or avoid shell:

```yaml
# SAFE
command: ls
args: ["{{directory}}"]

# Validation in Native handler
fn validate_directory(dir: &str) -> Result<()> {
    if dir.contains(';') || dir.contains('|') {
        return Err(Error::Validation("Invalid characters".into()));
    }
    Ok(())
}
```

### Pitfall 2: Timeout Too Short

```yaml
# WRONG - npm install can take minutes
- type: cli
  command: npm
  args: ["install"]
  timeout_ms: 5000  # 5 seconds - too short!
```

**Fix**: Set realistic timeouts:

```yaml
- type: cli
  command: npm
  args: ["install"]
  timeout_ms: 300000  # 5 minutes
  stream: true  # Show progress
```

### Pitfall 3: Ignoring Exit Codes

```javascript
// WRONG - assumes success
const result = await client.callTool("deploy_app", {});
console.log("Deployed:", result.stdout);

// RIGHT - check exit code
const result = await client.callTool("deploy_app", {});
if (result.exit_code !== 0) {
    throw new Error(`Deploy failed: ${result.stderr}`);
}
console.log("Deployed:", result.stdout);
```

## Next Steps

Chapter 4.2 covers streaming output for long-running commands, including real-time log parsing and progress reporting.

---

> "Wrap, don't rewrite. CLI handlers preserve the Unix philosophy." - pforge design principle
