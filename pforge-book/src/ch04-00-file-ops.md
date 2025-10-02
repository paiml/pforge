# File Operations: CLI Handler Overview

The CLI handler is pforge's bridge to the shell - it wraps command-line tools as MCP tools with **zero custom code**. This chapter demonstrates building a file operations server using common Unix utilities.

## Why CLI Handlers?

**Use CLI handlers when**:
- You want to expose existing shell commands
- The logic already exists in a CLI tool
- You need streaming output from long-running commands
- You're prototyping quickly without writing Rust

**Don't use CLI handlers when**:
- You need complex validation (use Native handlers)
- Performance is critical (< 1μs dispatch - use Native)
- The command has security implications (validate in Rust first)

## The File Operations Server

Let's build a server that wraps common file operations:

```yaml
forge:
  name: file-ops-server
  version: 0.1.0
  transport: stdio
  optimization: release

tools:
  - type: cli
    name: list_files
    description: "List files in a directory"
    command: ls
    args: ["-lah"]
    params:
      path:
        type: string
        required: false
        default: "."
        description: "Directory to list"

  - type: cli
    name: file_info
    description: "Get detailed file information"
    command: stat
    args: []
    params:
      file:
        type: string
        required: true
        description: "Path to file"

  - type: cli
    name: search_files
    description: "Search for files by name pattern"
    command: find
    args: []
    params:
      directory:
        type: string
        required: false
        default: "."
      pattern:
        type: string
        required: true
        description: "File name pattern (e.g., '*.rs')"

  - type: cli
    name: count_lines
    description: "Count lines in a file"
    command: wc
    args: ["-l"]
    params:
      file:
        type: string
        required: true
        description: "Path to file"
```

## CLI Handler Anatomy

Every CLI handler has these components:

### 1. Command and Arguments

```yaml
command: ls
args: ["-lah"]
```

**Base configuration**:
- `command`: The executable to run (`ls`, `git`, `docker`, etc.)
- `args`: Static arguments always passed to the command

### 2. Dynamic Parameters

```yaml
params:
  path:
    type: string
    required: false
    default: "."
```

**Parameter flow**:
1. Client sends: `{ "path": "/home/user" }`
2. pforge appends to args: `["ls", "-lah", "/home/user"]`
3. Executes: `ls -lah /home/user`

### 3. Execution Options

```yaml
tools:
  - type: cli
    name: long_running_task
    command: ./process.sh
    timeout_ms: 60000  # 60 seconds
    cwd: /tmp
    env:
      LOG_LEVEL: debug
    stream: true  # Enable output streaming
```

**Options**:
- `timeout_ms`: Max execution time (default: 30s)
- `cwd`: Working directory
- `env`: Environment variables
- `stream`: Stream output in real-time

## Input and Output Structure

CLI handlers use a standard schema:

### Input

```rust
{
  "args": ["additional", "arguments"],  // Optional
  "env": {                              // Optional
    "CUSTOM_VAR": "value"
  }
}
```

### Output

```rust
{
  "stdout": "command output here",
  "stderr": "any errors here",
  "exit_code": 0
}
```

## Practical Example: Git Integration

```yaml
tools:
  - type: cli
    name: git_status
    description: "Get git repository status"
    command: git
    args: ["status", "--short"]
    cwd: "{{repo_path}}"
    params:
      repo_path:
        type: string
        required: true
        description: "Path to git repository"

  - type: cli
    name: git_log
    description: "Show git commit history"
    command: git
    args: ["log", "--oneline"]
    params:
      repo_path:
        type: string
        required: true
      max_count:
        type: integer
        required: false
        default: 10
        description: "Number of commits to show"
```

**Usage**:

```json
// Request
{
  "tool": "git_log",
  "params": {
    "repo_path": "/home/user/project",
    "max_count": 5
  }
}

// Response
{
  "stdout": "abc123 feat: add new feature\ndef456 fix: resolve bug\n...",
  "stderr": "",
  "exit_code": 0
}
```

## Error Handling

CLI handlers return errors when:

1. **Command not found**:
```json
{
  "error": "Handler: Failed to execute command 'nonexistent': No such file or directory"
}
```

2. **Timeout exceeded**:
```json
{
  "error": "Timeout: Command exceeded 30000ms timeout"
}
```

3. **Non-zero exit code**:
```json
{
  "stdout": "",
  "stderr": "fatal: not a git repository",
  "exit_code": 128
}
```

**Important**: CLI handlers don't automatically fail on non-zero exit codes. Check `exit_code` in your client.

## Performance Characteristics

| Metric | Value |
|--------|-------|
| Dispatch overhead | 5-10μs |
| Command spawn time | 1-5ms |
| Output processing | 10μs/KB |
| Memory per command | ~2KB |

**Compared to Native handlers**:
- 5-10x slower dispatch
- Higher memory usage
- But zero implementation code!

## Security Considerations

### 1. Command Injection Prevention

```yaml
# SAFE - static command and args
command: ls
args: ["-lah"]

# UNSAFE - user input in command (pforge blocks this)
command: "{{user_command}}"  # NOT ALLOWED
```

pforge **never** allows dynamic commands - only static binaries with dynamic arguments.

### 2. Argument Validation

```yaml
params:
  path:
    type: string
    required: true
    pattern: "^[a-zA-Z0-9_/.-]+$"  # Restrict characters
```

**Best practice**: Use JSON Schema validation to restrict input patterns.

### 3. Working Directory Restrictions

```yaml
cwd: /safe/directory  # Static, safe path
# NOT: cwd: "{{user_path}}"  # Would be security risk
```

## When to Use Each Handler Type

**CLI Handler** - Wrapping existing tools:
```yaml
type: cli
command: ffmpeg
args: ["-i", "{{input}}", "{{output}}"]
```

**Native Handler** - Complex validation:
```rust
async fn handle(&self, input: Input) -> Result<Output> {
    validate_path(&input.path)?;
    let output = Command::new("ls")
        .arg(&input.path)
        .output()
        .await?;
    // Custom processing...
}
```

**HTTP Handler** - External APIs:
```yaml
type: http
endpoint: "https://api.github.com/repos/{{owner}}/{{repo}}"
method: GET
```

**Pipeline Handler** - Multi-step workflows:
```yaml
type: pipeline
steps:
  - tool: list_files
    output_var: files
  - tool: count_lines
    input: { file: "{{files}}" }
```

## Common CLI Handler Patterns

### Pattern 1: Optional Arguments

```yaml
params:
  verbose:
    type: boolean
    required: false
    default: false

# In YAML, conditionally include args based on params
# (Future feature - current workaround: use Native handler)
```

### Pattern 2: Environment Configuration

```yaml
env:
  PATH: "/usr/local/bin:/usr/bin"
  LANG: "en_US.UTF-8"
  CUSTOM_CONFIG: "{{config_path}}"
```

### Pattern 3: Streaming Large Output

```yaml
stream: true
timeout_ms: 300000  # 5 minutes

# For commands like:
# - docker build (long running)
# - tail -f (continuous output)
# - npm install (progress updates)
```

## Testing CLI Handlers

CLI handlers are tested at the **integration level**:

```rust
#[tokio::test]
async fn test_cli_handler_ls() {
    let handler = CliHandler::new(
        "ls".to_string(),
        vec!["-lah".to_string()],
        None,
        HashMap::new(),
        None,
        false,
    );

    let input = CliInput {
        args: vec![".".to_string()],
        env: HashMap::new(),
    };

    let result = handler.execute(input).await.unwrap();
    assert_eq!(result.exit_code, 0);
    assert!(result.stdout.contains("total"));
}
```

**Test coverage requirements**:
- Happy path: command succeeds
- Error path: command fails
- Timeout: long-running command
- Environment: env vars passed correctly

## Next Steps

In Chapter 4.1, we'll dive deep into wrapping shell commands, including argument templating and output parsing strategies.

---

> "The best code is no code. CLI handlers let Unix tools do the work." - pforge philosophy
