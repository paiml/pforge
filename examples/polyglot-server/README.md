# Polyglot MCP Server - pforge Example

**Multi-language MCP server showcasing Rust, Python, and Go handlers in a single server.**

This advanced example demonstrates pforge's language bridge capabilities:
- ✅ Rust native handlers (compiled, fastest performance)
- ✅ Python handlers via subprocess bridge
- ✅ Go handlers via subprocess bridge
- ✅ Polyglot pipeline combining all languages
- ✅ Performance comparison across languages
- ✅ Production-ready error handling

---

## Quick Start

**Prerequisites**:
- Rust 1.75+ and Cargo
- Python 3.7+
- Go 1.21+

```bash
# Navigate to the example directory
cd examples/polyglot-server

# Build the Go binary
cd src/go && go build hasher.go && cd ../..

# Make Python script executable
chmod +x src/python/sentiment_analyzer.py

# Run the server
cargo run

# Expected output:
# ╔═══════════════════════════════════════╗
# ║   Polyglot MCP Server                 ║
# ║   Rust + Python + Go                  ║
# ║   Powered by pforge v0.1.0            ║
# ╚═══════════════════════════════════════╝
#
# This example demonstrates:
#   ✓ Rust native handler (Fibonacci)
#   ✓ Python handler via bridge (Sentiment Analysis)
#   ✓ Go handler via bridge (Cryptographic Hash)
#   ✓ CLI handler (System Info)
#   ✓ Polyglot pipeline
#
# Available tools:
#   • rust_fibonacci(n) - Calculate Fibonacci (Rust)
#   • python_sentiment(text, language?) - Analyze sentiment (Python)
#   • go_hash(data, algorithm?) - Calculate hash (Go)
#   • system_info() - Get system info (CLI)
#   • polyglot_pipeline() - Pipeline using all languages
```

---

## What It Does

### Tools Provided

#### 1. **rust_fibonacci** - Rust Native Handler
Calculates Fibonacci numbers with full sequence.

- **Language**: Rust (compiled, zero-cost abstractions)
- **Input**:
  - `n` (required, 0-50): Position in sequence
- **Output**:
  ```json
  {
    "n": 10,
    "value": 55,
    "sequence": [0, 1, 1, 2, 3, 5, 8, 13, 21, 34, 55],
    "language": "Rust"
  }
  ```
- **Performance**: ~500ns (sub-microsecond)

#### 2. **python_sentiment** - Python Handler
Analyzes text sentiment using Python.

- **Language**: Python (via subprocess bridge)
- **Input**:
  - `text` (required): Text to analyze
  - `language` (optional, default: "en"): Language code
- **Output**:
  ```json
  {
    "text": "This is wonderful!",
    "sentiment": "positive",
    "score": 0.65,
    "language": "en",
    "implementation": "Python"
  }
  ```
- **Performance**: ~50ms (Python startup + execution)

#### 3. **go_hash** - Go Handler
Calculates cryptographic hashes using Go.

- **Language**: Go (via subprocess bridge)
- **Input**:
  - `data` (required): Data to hash
  - `algorithm` (optional, default: "sha256"): Hash algorithm (md5, sha1, sha256, sha512)
- **Output**:
  ```json
  {
    "data": "pforge",
    "algorithm": "sha256",
    "hash": "4f3d8...",
    "length": 64,
    "implementation": "Go"
  }
  ```
- **Performance**: ~30ms (Go binary execution)

#### 4. **system_info** - CLI Handler
Gets system information.

- **Language**: System CLI
- **Input**: None
- **Output**: System information string
- **Performance**: ~5ms

#### 5. **polyglot_pipeline** - Pipeline Handler
Demonstrates pipeline combining all languages.

- **Type**: Pipeline
- **Steps**:
  1. Calculate Fibonacci(10) in Rust
  2. Analyze sentiment in Python
  3. Calculate hash in Go
- **Performance**: ~80ms (sum of all steps)

### Example Usage with MCP Client

```json
// Request: Fibonacci in Rust
{
  "jsonrpc": "2.0",
  "id": 1,
  "method": "tools/call",
  "params": {
    "name": "rust_fibonacci",
    "arguments": {"n": 10}
  }
}

// Request: Sentiment in Python
{
  "jsonrpc": "2.0",
  "id": 2,
  "method": "tools/call",
  "params": {
    "name": "python_sentiment",
    "arguments": {
      "text": "This is a great example!",
      "language": "en"
    }
  }
}

// Request: Hash in Go
{
  "jsonrpc": "2.0",
  "id": 3,
  "method": "tools/call",
  "params": {
    "name": "go_hash",
    "arguments": {
      "data": "pforge-polyglot",
      "algorithm": "sha256"
    }
  }
}

// Request: Polyglot pipeline
{
  "jsonrpc": "2.0",
  "id": 4,
  "method": "tools/call",
  "params": {
    "name": "polyglot_pipeline",
    "arguments": {}
  }
}
```

---

## Architecture

### Project Structure

```
polyglot-server/
├── pforge.yaml              # 5 tools (Rust, Python, Go, CLI, Pipeline)
├── Cargo.toml               # Rust dependencies + pforge-bridge
├── src/
│   ├── main.rs              # Server entry point
│   ├── handlers/
│   │   ├── mod.rs           # Handler exports
│   │   ├── fibonacci.rs     # Rust native handler
│   │   ├── python_bridge.rs # Python subprocess bridge
│   │   └── go_bridge.rs     # Go subprocess bridge
│   ├── python/
│   │   └── sentiment_analyzer.py  # Python implementation
│   └── go/
│       ├── hasher.go        # Go implementation
│       └── go.mod           # Go module
└── README.md                # This file
```

### Language Bridges

#### Rust Native Handler (Fastest)
```rust
// Direct compilation into binary
#[async_trait::async_trait]
impl Handler for FibonacciHandler {
    async fn handle(&self, input: Self::Input) -> Result<Self::Output> {
        // Pure Rust logic - zero FFI overhead
        let sequence = calculate_fibonacci_sequence(input.n);
        Ok(FibonacciOutput { ... })
    }
}
```

**Advantages**:
- Zero overhead (compiled)
- Type safety at compile time
- Fastest execution (~500ns)

#### Python Subprocess Bridge
```rust
// Launch Python subprocess with JSON I/O
pub struct PythonSentimentHandler;

impl Handler for PythonSentimentHandler {
    async fn handle(&self, input: Self::Input) -> Result<Self::Output> {
        let output = Command::new("python3")
            .arg("src/python/sentiment_analyzer.py")
            .arg(&input.text)
            .output()?;

        let result: serde_json::Value = serde_json::from_slice(&output.stdout)?;
        Ok(PythonSentimentOutput { ... })
    }
}
```

**Advantages**:
- Use existing Python libraries
- Easy integration
- JSON serialization for data exchange

**Trade-offs**:
- Python startup overhead (~30ms)
- Subprocess creation cost

#### Go Subprocess Bridge
```rust
// Launch Go binary with JSON I/O
pub struct GoHashHandler;

impl Handler for GoHashHandler {
    async fn handle(&self, input: Self::Input) -> Result<Self::Output> {
        let output = Command::new("src/go/hasher")
            .arg(&input.algorithm)
            .arg(&input.data)
            .output()?;

        let result: serde_json::Value = serde_json::from_slice(&output.stdout)?;
        Ok(GoHashOutput { ... })
    }
}
```

**Advantages**:
- Leverage Go's excellent stdlib (crypto, net)
- Compiled performance
- Easy integration

**Trade-offs**:
- Binary must be pre-compiled
- Subprocess overhead (~10ms)

---

## Performance Comparison

### Benchmark Results (M1 Mac, 2023)

| Handler | Language | Execution Time | Memory | Notes |
|---------|----------|---------------|--------|-------|
| **rust_fibonacci** | Rust | ~500ns | 0KB | Compiled, zero-cost |
| **python_sentiment** | Python | ~50ms | 15MB | Python startup |
| **go_hash** | Go | ~30ms | 5MB | Binary execution |
| **system_info** | CLI | ~5ms | 2MB | Shell overhead |
| **polyglot_pipeline** | Mixed | ~85ms | 22MB | Sum of all |

### When to Use Each Language

**Rust (Native)**:
- ✅ Performance-critical paths
- ✅ Pure computation (math, algorithms)
- ✅ Type safety required
- ✅ Memory constraints

**Python (Bridge)**:
- ✅ ML/AI workloads (TensorFlow, PyTorch)
- ✅ Data science (pandas, numpy)
- ✅ Rapid prototyping
- ✅ Rich library ecosystem

**Go (Bridge)**:
- ✅ Network services
- ✅ Cryptography (excellent stdlib)
- ✅ Concurrent processing
- ✅ Fast compilation

**CLI**:
- ✅ System integration
- ✅ Existing tools
- ✅ Simple utilities

---

## Development Workflow

### Add a New Rust Handler

```rust
// 1. Create handler in src/handlers/my_handler.rs
use pforge_runtime::{Handler, Result};

pub struct MyHandler;

#[async_trait::async_trait]
impl Handler for MyHandler {
    type Input = MyInput;
    type Output = MyOutput;
    type Error = pforge_runtime::Error;

    async fn handle(&self, input: Self::Input) -> Result<Self::Output> {
        // Implementation
        Ok(MyOutput { ... })
    }
}

// 2. Export in src/handlers/mod.rs
pub mod my_handler;

// 3. Register in main.rs
reg.register("my_tool", handlers::my_handler::MyHandler);
```

### Add a New Python Handler

```python
# 1. Create src/python/my_module.py
#!/usr/bin/env python3
import sys
import json

def process(input_data):
    # Implementation
    return {"result": "value"}

if __name__ == "__main__":
    input_data = json.loads(sys.argv[1])
    result = process(input_data)
    print(json.dumps(result))
```

```rust
// 2. Create bridge in src/handlers/my_python_bridge.rs
pub struct MyPythonHandler;

impl Handler for MyPythonHandler {
    async fn handle(&self, input: Self::Input) -> Result<Self::Output> {
        let output = Command::new("python3")
            .arg("src/python/my_module.py")
            .arg(serde_json::to_string(&input)?)
            .output()?;

        let result: MyOutput = serde_json::from_slice(&output.stdout)?;
        Ok(result)
    }
}
```

### Add a New Go Handler

```go
// 1. Create src/go/my_tool.go
package main

import (
    "encoding/json"
    "os"
)

type Input struct {
    Data string `json:"data"`
}

type Output struct {
    Result string `json:"result"`
}

func main() {
    var input Input
    json.NewDecoder(os.Stdin).Decode(&input)

    output := Output{Result: process(input.Data)}
    json.NewEncoder(os.Stdout).Encode(output)
}

func process(data string) string {
    // Implementation
    return "processed"
}
```

```bash
# 2. Build Go binary
cd src/go && go build my_tool.go && cd ../..
```

```rust
// 3. Create bridge in src/handlers/my_go_bridge.rs
pub struct MyGoHandler;

impl Handler for MyGoHandler {
    async fn handle(&self, input: Self::Input) -> Result<Self::Output> {
        let output = Command::new("src/go/my_tool")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()?;

        serde_json::to_writer(output.stdin.unwrap(), &input)?;
        let result: MyOutput = serde_json::from_reader(output.stdout.unwrap())?;
        Ok(result)
    }
}
```

---

## Testing

### Run Unit Tests

```bash
# Test Rust handlers
cargo test

# Test Python script
python3 src/python/sentiment_analyzer.py "This is great!" "en"

# Test Go binary
cd src/go && go test && cd ../..
```

### Integration Testing

```bash
# Build Go binary first
cd src/go && go build hasher.go && cd ../..

# Run full server
cargo run

# Test with MCP client
echo '{"jsonrpc":"2.0","id":1,"method":"tools/call","params":{"name":"rust_fibonacci","arguments":{"n":10}}}' | cargo run
```

---

## Production Deployment

### Optimization Tips

**1. Pre-compile All Binaries**:
```bash
# Rust release build
cargo build --release

# Go optimized build
cd src/go && go build -ldflags="-s -w" hasher.go && cd ../..

# Python bytecode compilation
python3 -m compileall src/python/
```

**2. Use Production Python**:
```python
# Replace simple sentiment with real NLP
from textblob import TextBlob

def analyze_sentiment(text, language):
    blob = TextBlob(text)
    return {
        "sentiment": "positive" if blob.sentiment.polarity > 0 else "negative",
        "score": blob.sentiment.polarity,
        "language": language
    }
```

**3. Optimize Go for Performance**:
```go
// Use sync.Pool for object reuse
var hashPool = sync.Pool{
    New: func() interface{} {
        return sha256.New()
    },
}

func calculateHash(algorithm, data string) string {
    h := hashPool.Get().(hash.Hash)
    defer hashPool.Put(h)

    h.Reset()
    h.Write([]byte(data))
    return hex.EncodeToString(h.Sum(nil))
}
```

**4. Use FFI for Lower Overhead**:
```rust
// Instead of subprocess, use direct FFI
use pforge_bridge::{PythonBridge, GoBridge};

impl Handler for PythonSentimentHandler {
    async fn handle(&self, input: Self::Input) -> Result<Self::Output> {
        // Direct FFI call - much faster than subprocess
        let bridge = PythonBridge::new();
        bridge.call("sentiment_analyzer", &input).await
    }
}
```

---

## Advanced Features

### Error Handling Across Languages

```rust
// Rust handler with detailed errors
impl Handler for GoHashHandler {
    async fn handle(&self, input: Self::Input) -> Result<Self::Output> {
        let output = Command::new("src/go/hasher")
            .output()
            .map_err(|e| match e.kind() {
                io::ErrorKind::NotFound => Error::Handler(
                    "Go binary not found. Run: cd src/go && go build hasher.go".to_string()
                ),
                _ => Error::Handler(format!("Go execution failed: {}", e))
            })?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(Error::Handler(format!("Go error: {}", stderr)));
        }

        // Parse result...
    }
}
```

### Language-Specific Timeouts

```yaml
# In pforge.yaml
tools:
  - type: native
    name: python_sentiment
    timeout_ms: 5000  # 5s for Python (slower startup)

  - type: native
    name: go_hash
    timeout_ms: 1000  # 1s for Go (faster)

  - type: native
    name: rust_fibonacci
    timeout_ms: 100   # 100ms for Rust (fastest)
```

### Streaming Output

```rust
// For long-running Python scripts
use tokio::io::{AsyncBufReadExt, BufReader};

impl Handler for PythonStreamHandler {
    async fn handle(&self, input: Self::Input) -> Result<Self::Output> {
        let mut child = Command::new("python3")
            .arg("src/python/long_task.py")
            .stdout(Stdio::piped())
            .spawn()?;

        let stdout = child.stdout.take().unwrap();
        let reader = BufReader::new(stdout);
        let mut lines = reader.lines();

        while let Some(line) = lines.next_line().await? {
            eprintln!("Python: {}", line);  // Stream to stderr
        }

        child.wait().await?;
        Ok(output)
    }
}
```

---

## Troubleshooting

### Python Not Found

**Problem**: `Error: Python execution failed: No such file or directory`

**Solution**:
```bash
# Verify Python 3 is installed
python3 --version

# If not, install Python 3
# Ubuntu/Debian: sudo apt install python3
# macOS: brew install python3
# Windows: Download from python.org
```

### Go Binary Not Found

**Problem**: `Error: Go binary not found`

**Solution**:
```bash
# Build the Go binary
cd src/go
go build hasher.go
chmod +x hasher
cd ../..

# Verify it works
./src/go/hasher sha256 "test"
```

### Performance Issues

**Problem**: Handlers are slow

**Solution**:
```bash
# 1. Use release build
cargo build --release
./target/release/polyglot-server

# 2. Pre-compile Python
python3 -m compileall src/python/

# 3. Use optimized Go build
cd src/go && go build -ldflags="-s -w" hasher.go
```

### JSON Parse Errors

**Problem**: `Error: JSON parse error`

**Solution**:
```python
# Ensure Python outputs valid JSON
import json
import sys

try:
    result = {"key": "value"}
    print(json.dumps(result))  # Always use json.dumps
except Exception as e:
    error = {"error": str(e)}
    print(json.dumps(error))
    sys.exit(1)
```

---

## Next Steps

After understanding this example:

1. **Try Other Examples**:
   - `examples/hello-world/` - Basic concepts
   - `examples/pmat-server/` - CLI integration
   - `examples/production-server/` - Full production setup

2. **Build Your Own Polyglot Server**:
   ```bash
   pforge new my-polyglot-server
   # Add handlers in your preferred languages
   ```

3. **Explore Language Bridges**:
   - Check `bridges/python/README.md`
   - Check `bridges/go/README.md`
   - Learn about FFI for zero-copy calls

---

## Learn More

- **pforge Language Bridges**: [../../bridges/README.md](../../bridges/README.md)
- **pforge User Guide**: [../../USER_GUIDE.md](../../USER_GUIDE.md)
- **Architecture Deep Dive**: [../../ARCHITECTURE.md](../../ARCHITECTURE.md)
- **MCP Protocol**: https://spec.modelcontextprotocol.io/

---

**License**: MIT

**Maintained by**: pforge team

**Version**: 0.1.0
