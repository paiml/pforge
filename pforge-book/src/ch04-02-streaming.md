# Streaming Command Output

Long-running commands like builds, deploys, and log tails need real-time output streaming. This chapter covers CLI handler streaming capabilities and patterns for progressive output delivery.

## Why Streaming Matters

**Without streaming**:
```yaml
- type: cli
  command: npm
  args: ["install"]
  timeout_ms: 300000  # Wait 5 minutes for all output
```

Result: Client sees nothing for 5 minutes, then gets 50KB of logs at once.

**With streaming**:
```yaml
- type: cli
  command: npm
  args: ["install"]
  timeout_ms: 300000
  stream: true  # Enable real-time output
```

Result: Client sees progress updates as they happen.

## Enabling Streaming

### YAML Configuration

```yaml
tools:
  - type: cli
    name: build_project
    description: "Build project with real-time logs"
    command: cargo
    args: ["build", "--release"]
    stream: true  # Key setting
    timeout_ms: 600000  # 10 minutes
```

### How Streaming Works

1. **Command spawns** with `stdout` and `stderr` piped
2. **Output buffers** as it's produced
3. **Server sends** chunks via MCP protocol
4. **Client receives** progressive updates
5. **Complete output** returned at end

**Protocol flow**:
```
Server                          Client
------                          ------
spawn("cargo build")
  ↓
[stdout] "Compiling..."    →    Display "Compiling..."
[stdout] "Building..."     →    Display "Building..."
[stderr] "warning: ..."    →    Display "warning: ..."
[exit] code: 0             →    Display "Complete"
```

## Streaming Patterns

### Pattern 1: Build Progress

```yaml
tools:
  - type: cli
    name: docker_build
    description: "Build Docker image with progress"
    command: docker
    args:
      - "build"
      - "-t"
      - "{{image_name}}"
      - "{{context_dir}}"
    stream: true
    timeout_ms: 1800000  # 30 minutes
    params:
      image_name:
        type: string
        required: true
      context_dir:
        type: string
        required: false
        default: "."
```

**Output stream**:
```
Step 1/8 : FROM node:18
 ---> a1b2c3d4e5f6
Step 2/8 : WORKDIR /app
 ---> Running in abc123...
 ---> def456
...
Successfully built xyz789
Successfully tagged my-app:latest
```

### Pattern 2: Log Tailing

```yaml
tools:
  - type: cli
    name: tail_logs
    description: "Tail application logs"
    command: tail
    args: ["-f", "{{log_file}}"]
    stream: true
    timeout_ms: 3600000  # 1 hour
    params:
      log_file:
        type: string
        required: true
```

**Continuous stream** until timeout or client disconnects.

### Pattern 3: Test Runner

```yaml
tools:
  - type: cli
    name: run_tests
    description: "Run tests with real-time results"
    command: cargo
    args: ["test", "--", "--nocapture"]
    stream: true
    timeout_ms: 300000
```

**Output stream**:
```
running 45 tests
test auth::test_login ... ok
test auth::test_logout ... ok
test db::test_connection ... ok
...
test result: ok. 45 passed; 0 failed
```

### Pattern 4: Interactive Command

```yaml
tools:
  - type: cli
    name: shell_session
    description: "Execute shell command interactively"
    command: sh
    args: ["-c", "{{script}}"]
    stream: true
    params:
      script:
        type: string
        required: true
```

## Native Handler Streaming

For more control, implement streaming in a Native handler:

```rust
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::{Command, Stdio};

#[derive(Deserialize, JsonSchema)]
struct BuildInput {
    project_path: String,
}

#[derive(Serialize, JsonSchema)]
struct BuildOutput {
    success: bool,
    lines: Vec<String>,
    duration_ms: u64,
}

pub struct BuildHandler;

#[async_trait::async_trait]
impl Handler for BuildHandler {
    type Input = BuildInput;
    type Output = BuildOutput;
    type Error = Error;

    async fn handle(&self, input: Self::Input) -> Result<Self::Output> {
        let start = std::time::Instant::now();

        let mut child = Command::new("cargo")
            .arg("build")
            .arg("--release")
            .current_dir(&input.project_path)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| Error::Handler(format!("Spawn failed: {}", e)))?;

        let stdout = child.stdout.take()
            .ok_or_else(|| Error::Handler("No stdout".into()))?;

        let mut reader = BufReader::new(stdout).lines();
        let mut lines = Vec::new();

        while let Some(line) = reader.next_line().await
            .map_err(|e| Error::Handler(format!("Read failed: {}", e)))? {

            // Stream line to client (via logging/events)
            tracing::info!("BUILD: {}", line);
            lines.push(line);
        }

        let status = child.wait().await
            .map_err(|e| Error::Handler(format!("Wait failed: {}", e)))?;

        Ok(BuildOutput {
            success: status.success(),
            lines,
            duration_ms: start.elapsed().as_millis() as u64,
        })
    }
}
```

## Buffering and Backpressure

### Line Buffering (Default)

CLI handlers buffer by line:

```rust
// Internal implementation
let reader = BufReader::new(stdout);
let mut lines = reader.lines();

while let Some(line) = lines.next_line().await? {
    send_to_client(line).await?;
}
```

**Characteristics**:
- Low latency for line-oriented output
- Natural chunking at newlines
- Works well for logs, test output

### Chunk Buffering

For binary or non-line output:

```rust
use tokio::io::AsyncReadExt;

let mut stdout = child.stdout.take().unwrap();
let mut buffer = [0u8; 8192];

loop {
    let n = stdout.read(&mut buffer).await?;
    if n == 0 { break; }

    send_chunk_to_client(&buffer[..n]).await?;
}
```

**Characteristics**:
- Fixed-size chunks (8KB)
- Better for binary data
- Higher throughput

### Backpressure Handling

If client can't keep up:

```rust
use tokio::sync::mpsc;

let (tx, mut rx) = mpsc::channel(100);  // Bounded channel

// Producer (command output)
tokio::spawn(async move {
    while let Some(line) = reader.next_line().await? {
        // Blocks if channel full (backpressure)
        tx.send(line).await?;
    }
});

// Consumer (client)
while let Some(line) = rx.recv().await {
    send_to_client(line).await?;
}
```

**Benefits**:
- Prevents memory bloat
- Smooth delivery rate
- Graceful degradation

## Timeout Management

### Global Timeout

```yaml
- type: cli
  command: npm
  args: ["install"]
  timeout_ms: 300000  # Entire command must complete in 5 minutes
  stream: true
```

**Behavior**: Command killed if it runs longer than 5 minutes, even if streaming.

### Per-Line Timeout

For commands that might stall:

```rust
use tokio::time::{timeout, Duration};

while let Ok(Some(line)) = timeout(
    Duration::from_secs(30),  // 30s per line
    reader.next_line()
).await {
    match line {
        Ok(line) => send_to_client(line).await?,
        Err(e) => return Err(Error::Handler(format!("Read error: {}", e))),
    }
}
```

**Use case**: Detect hung processes that produce no output.

## Progress Parsing

### JSON Progress (Docker, npm, etc.)

```rust
#[derive(Deserialize)]
struct ProgressLine {
    status: String,
    id: Option<String>,
    progress: Option<String>,
}

while let Some(line) = reader.next_line().await? {
    if let Ok(progress) = serde_json::from_str::<ProgressLine>(&line) {
        // Structured progress update
        send_progress(Progress {
            status: progress.status,
            current: parse_progress(&progress.progress),
        }).await?;
    } else {
        // Plain text fallback
        send_text(line).await?;
    }
}
```

### Percentage Progress (builds, downloads)

```rust
fn parse_progress(line: &str) -> Option<f64> {
    // "[===>      ] 45%"
    if let Some(start) = line.find('[') {
        if let Some(end) = line.find('%') {
            let percent_str = &line[start+1..end]
                .trim()
                .split_whitespace()
                .last()?;
            return percent_str.parse().ok();
        }
    }
    None
}
```

### Custom Progress Format

```rust
// Parse: "Compiling foo v1.0.0 (3/45)"
fn parse_cargo_progress(line: &str) -> Option<(u32, u32)> {
    if line.contains("Compiling") {
        if let Some(paren) = line.find('(') {
            let rest = &line[paren+1..];
            let parts: Vec<&str> = rest
                .trim_end_matches(')')
                .split('/')
                .collect();

            if parts.len() == 2 {
                let current = parts[0].parse().ok()?;
                let total = parts[1].parse().ok()?;
                return Some((current, total));
            }
        }
    }
    None
}
```

## Error Stream Handling

### Separate stdout/stderr

```rust
let mut stdout_reader = BufReader::new(
    child.stdout.take().unwrap()
).lines();

let mut stderr_reader = BufReader::new(
    child.stderr.take().unwrap()
).lines();

let stdout_task = tokio::spawn(async move {
    while let Some(line) = stdout_reader.next_line().await? {
        send_stdout(line).await?;
    }
    Ok::<_, Error>(())
});

let stderr_task = tokio::spawn(async move {
    while let Some(line) = stderr_reader.next_line().await? {
        send_stderr(line).await?;
    }
    Ok::<_, Error>(())
});

// Wait for both
tokio::try_join!(stdout_task, stderr_task)?;
```

### Merged Stream

```rust
// Redirect stderr to stdout
let child = Command::new("cargo")
    .arg("build")
    .stdout(Stdio::piped())
    .stderr(Stdio::piped())  // Can also use Stdio::inherit()
    .spawn()?;

// Or merge in shell
command: sh
args: ["-c", "npm install 2>&1"]  # stderr → stdout
```

## Real-World Example: CI/CD Pipeline

```yaml
forge:
  name: ci-pipeline
  version: 0.1.0
  transport: stdio

tools:
  - type: cli
    name: run_tests
    description: "Run test suite with coverage"
    command: cargo
    args: ["tarpaulin", "--out", "Stdout"]
    stream: true
    timeout_ms: 600000

  - type: cli
    name: build_release
    description: "Build optimized release binary"
    command: cargo
    args: ["build", "--release"]
    stream: true
    timeout_ms: 1800000

  - type: cli
    name: deploy
    description: "Deploy to production"
    command: ./scripts/deploy.sh
    args: ["{{environment}}"]
    stream: true
    timeout_ms: 900000
    env:
      CI: "true"
    params:
      environment:
        type: string
        required: true
        enum: ["staging", "production"]
```

**Client usage**:

```javascript
const client = new MCPClient("ci-pipeline");

// Real-time test output
await client.callTool("run_tests", {}, {
  onProgress: (line) => {
    console.log(`TEST: ${line}`);
  }
});

// Real-time build output
await client.callTool("build_release", {}, {
  onProgress: (line) => {
    if (line.includes("Compiling")) {
      updateProgressBar(line);
    }
  }
});

// Real-time deploy output
await client.callTool("deploy", {
  environment: "production"
}, {
  onProgress: (line) => {
    if (line.includes("ERROR")) {
      alert(`Deploy issue: ${line}`);
    }
  }
});
```

## Performance Considerations

### Memory Usage

**Problem**: Storing all output in memory:

```rust
// BAD - unbounded growth
let mut all_output = String::new();
while let Some(line) = reader.next_line().await? {
    all_output.push_str(&line);
    all_output.push('\n');
}
```

**Solution**: Stream without buffering:

```rust
// GOOD - constant memory
while let Some(line) = reader.next_line().await? {
    send_to_client(line).await?;
    // `line` dropped after send
}
```

### Throughput

**Line-by-line** (high latency, low throughput):
```rust
// ~1000 lines/sec
while let Some(line) = reader.next_line().await? {
    send(line).await?;
}
```

**Batch sending** (low latency, high throughput):
```rust
// ~10000 lines/sec
let mut batch = Vec::new();
while let Some(line) = reader.next_line().await? {
    batch.push(line);
    if batch.len() >= 100 {
        send_batch(&batch).await?;
        batch.clear();
    }
}
if !batch.is_empty() {
    send_batch(&batch).await?;
}
```

## Testing Streaming Handlers

### Mock Command Output

```rust
#[tokio::test]
async fn test_streaming_handler() {
    let handler = CliHandler::new(
        "sh".to_string(),
        vec![
            "-c".to_string(),
            "for i in 1 2 3; do echo line$i; sleep 0.1; done".to_string(),
        ],
        None,
        HashMap::new(),
        Some(5000),
        true,  // stream: true
    );

    let input = CliInput::default();
    let result = handler.execute(input).await.unwrap();

    assert_eq!(result.exit_code, 0);
    assert!(result.stdout.contains("line1"));
    assert!(result.stdout.contains("line2"));
    assert!(result.stdout.contains("line3"));
}
```

### Verify Streaming Behavior

```rust
#[tokio::test]
async fn test_stream_delivers_progressively() {
    use tokio::time::{sleep, Duration};

    let (tx, mut rx) = mpsc::channel(10);

    tokio::spawn(async move {
        let handler = CliHandler::new(...);
        // Handler sends to tx as it streams
    });

    // Verify we get updates before completion
    let first = rx.recv().await.unwrap();
    sleep(Duration::from_millis(100)).await;
    let second = rx.recv().await.unwrap();

    assert_ne!(first, second);  // Different lines
}
```

## Next Steps

Chapter 4.3 covers comprehensive integration testing strategies for CLI handlers, including mocking commands and testing error conditions.

---

> "Stream, don't batch. Users want feedback, not wait times." - pforge streaming philosophy
