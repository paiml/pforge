# Running and Using the Calculator

You've built a production-ready calculator with YAML config, Rust handlers, and comprehensive tests. Now let's **run it** and see the EXTREME TDD discipline pay off.

## Project Setup

If you haven't created the calculator yet, start here:

```bash
# Create a new pforge project
pforge new calculator-server --type native
cd calculator-server

# Copy the example files
cp ../examples/calculator/forge.yaml .
cp ../examples/calculator/src/handlers.rs src/
```

Or work directly with the example:

```bash
cd examples/calculator
```

## Build the Server

### Development Build

```bash
cargo build
```

**Output**:

```
   Compiling pforge-example-calculator v0.1.0
    Finished dev [unoptimized + debuginfo] target(s) in 2.34s
```

**Development builds**:
- Include debug symbols
- No optimizations
- Fast compile time (~2s)
- Suitable for testing

### Release Build

```bash
cargo build --release
```

**Output**:

```
   Compiling pforge-example-calculator v0.1.0
    Finished release [optimized] target(s) in 8.67s
```

**Release builds**:
- Full optimizations enabled
- Strip debug symbols
- Slower compile (~8s)
- **10x faster runtime** (<1μs dispatch)

**Use release builds for**:
- Production deployment
- Performance benchmarking
- Integration with MCP clients

## Run the Tests First

Before running the server, verify everything works:

```bash
cargo test
```

**Expected output**:

```
running 6 tests
test tests::test_add ... ok
test tests::test_subtract ... ok
test tests::test_multiply ... ok
test tests::test_divide ... ok
test tests::test_divide_by_zero ... ok
test tests::test_unknown_operation ... ok

test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

**All 6 tests pass in <10ms**. This is the EXTREME TDD confidence - you know it works before running it.

## Start the Server

The calculator uses **stdio transport** (standard input/output), which means it communicates via JSON-RPC over stdin/stdout.

### Manual Testing with JSON-RPC

Create a test file `test_request.json`:

```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "method": "tools/call",
  "params": {
    "name": "calculate",
    "arguments": {
      "operation": "add",
      "a": 5.0,
      "b": 3.0
    }
  }
}
```

Run the server with this input:

```bash
cargo run --release < test_request.json
```

**Expected output**:

```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "result": {
    "content": [
      {
        "type": "text",
        "text": "{\"result\":8.0}"
      }
    ]
  }
}
```

**Success!** 5.0 + 3.0 = 8.0

## Using with MCP Clients

MCP clients like Claude Desktop, Continue, or Cline can connect to your calculator.

### Configure Claude Desktop

Add to `claude_desktop_config.json`:

```json
{
  "mcpServers": {
    "calculator": {
      "command": "cargo",
      "args": ["run", "--release", "--manifest-path", "/path/to/calculator/Cargo.toml"]
    }
  }
}
```

**Replace `/path/to/calculator`** with your actual path.

### Restart Claude Desktop

1. Quit Claude Desktop completely
2. Relaunch
3. Your calculator is now available as a tool!

### Test from Claude

Try asking Claude:

> "What is 123.45 multiplied by 67.89?"

Claude will:
1. See the `calculate` tool is available
2. Call it with `{"operation": "multiply", "a": 123.45, "b": 67.89}`
3. Receive the result: `8380.9005`
4. Respond: "123.45 × 67.89 = 8,380.90"

## Interactive Testing

For development, use a REPL-style workflow:

### Option 1: Use `pforge dev` (if available)

```bash
pforge dev
```

This starts a development server with hot reload.

### Option 2: Manual JSON-RPC

Create `test_all_operations.sh`:

```bash
#!/bin/bash

echo "Testing ADD..."
echo '{"jsonrpc":"2.0","id":1,"method":"tools/call","params":{"name":"calculate","arguments":{"operation":"add","a":10,"b":5}}}' | cargo run --release

echo "Testing SUBTRACT..."
echo '{"jsonrpc":"2.0","id":2,"method":"tools/call","params":{"name":"calculate","arguments":{"operation":"subtract","a":10,"b":5}}}' | cargo run --release

echo "Testing MULTIPLY..."
echo '{"jsonrpc":"2.0","id":3,"method":"tools/call","params":{"name":"calculate","arguments":{"operation":"multiply","a":10,"b":5}}}' | cargo run --release

echo "Testing DIVIDE..."
echo '{"jsonrpc":"2.0","id":4,"method":"tools/call","params":{"name":"calculate","arguments":{"operation":"divide","a":10,"b":5}}}' | cargo run --release

echo "Testing DIVIDE BY ZERO..."
echo '{"jsonrpc":"2.0","id":5,"method":"tools/call","params":{"name":"calculate","arguments":{"operation":"divide","a":10,"b":0}}}' | cargo run --release

echo "Testing UNKNOWN OPERATION..."
echo '{"jsonrpc":"2.0","id":6,"method":"tools/call","params":{"name":"calculate","arguments":{"operation":"modulo","a":10,"b":3}}}' | cargo run --release
```

Run it:

```bash
chmod +x test_all_operations.sh
./test_all_operations.sh
```

## Real-World Usage Examples

### Example 1: Simple Calculation

**Request**:
```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "method": "tools/call",
  "params": {
    "name": "calculate",
    "arguments": {
      "operation": "add",
      "a": 42.5,
      "b": 17.3
    }
  }
}
```

**Response**:
```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "result": {
    "content": [
      {
        "type": "text",
        "text": "{\"result\":59.8}"
      }
    ]
  }
}
```

### Example 2: Division

**Request**:
```json
{
  "jsonrpc": "2.0",
  "id": 2,
  "method": "tools/call",
  "params": {
    "name": "calculate",
    "arguments": {
      "operation": "divide",
      "a": 100,
      "b": 3
    }
  }
}
```

**Response**:
```json
{
  "jsonrpc": "2.0",
  "id": 2,
  "result": {
    "content": [
      {
        "type": "text",
        "text": "{\"result\":33.333333333333336}"
      }
    ]
  }
}
```

**Note the floating-point precision** - this is expected behavior for f64.

### Example 3: Error Handling (Division by Zero)

**Request**:
```json
{
  "jsonrpc": "2.0",
  "id": 3,
  "method": "tools/call",
  "params": {
    "name": "calculate",
    "arguments": {
      "operation": "divide",
      "a": 10,
      "b": 0
    }
  }
}
```

**Response**:
```json
{
  "jsonrpc": "2.0",
  "id": 3,
  "error": {
    "code": -32000,
    "message": "Division by zero"
  }
}
```

**Clean error message** - exactly what we tested!

### Example 4: Error Handling (Unknown Operation)

**Request**:
```json
{
  "jsonrpc": "2.0",
  "id": 4,
  "method": "tools/call",
  "params": {
    "name": "calculate",
    "arguments": {
      "operation": "power",
      "a": 2,
      "b": 8
    }
  }
}
```

**Response**:
```json
{
  "jsonrpc": "2.0",
  "id": 4,
  "error": {
    "code": -32000,
    "message": "Unknown operation: power. Supported: add, subtract, multiply, divide"
  }
}
```

**Helpful error message** tells users what went wrong AND what's supported.

## Performance Verification

Let's verify our <1μs dispatch target:

### Benchmark the Handler

Create `benches/calculator_bench.rs`:

```rust
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use pforge_example_calculator::handlers::{CalculateHandler, CalculateInput};
use pforge_runtime::Handler;

fn benchmark_operations(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();

    c.bench_function("add", |b| {
        let handler = CalculateHandler;
        b.to_async(&rt).iter(|| async {
            let input = CalculateInput {
                operation: "add".to_string(),
                a: black_box(5.0),
                b: black_box(3.0),
            };
            handler.handle(input).await.unwrap()
        });
    });

    c.bench_function("divide", |b| {
        let handler = CalculateHandler;
        b.to_async(&rt).iter(|| async {
            let input = CalculateInput {
                operation: "divide".to_string(),
                a: black_box(15.0),
                b: black_box(3.0),
            };
            handler.handle(input).await.unwrap()
        });
    });
}

criterion_group!(benches, benchmark_operations);
criterion_main!(benches);
```

Run benchmarks:

```bash
cargo bench
```

**Expected output**:

```
add                     time:   [450.23 ns 455.67 ns 461.34 ns]
divide                  time:   [782.45 ns 789.12 ns 796.78 ns]
```

**0.45μs for addition, 0.78μs for division** - we hit our <1μs target!

## Production Deployment

### Docker Container

Create `Dockerfile`:

```dockerfile
FROM rust:1.70 as builder
WORKDIR /app
COPY . .
RUN cargo build --release

FROM debian:bullseye-slim
COPY --from=builder /app/target/release/pforge-example-calculator /usr/local/bin/calculator
ENTRYPOINT ["calculator"]
```

Build and run:

```bash
docker build -t calculator-server .
docker run -i calculator-server
```

### Systemd Service

Create `/etc/systemd/system/calculator.service`:

```ini
[Unit]
Description=Calculator MCP Server
After=network.target

[Service]
Type=simple
User=mcp
ExecStart=/usr/local/bin/calculator
Restart=on-failure
StandardInput=socket
StandardOutput=socket

[Install]
WantedBy=multi-user.target
```

Enable and start:

```bash
sudo systemctl enable calculator
sudo systemctl start calculator
```

## Troubleshooting

### Issue: "Handler not found"

**Symptom**:
```
Error: Handler not found: handlers::calculate_handler
```

**Fix**:
Verify `forge.yaml` has correct path:
```yaml
handler:
  path: handlers::calculate_handler  # Not calculate_handler
```

### Issue: "Invalid JSON-RPC"

**Symptom**:
```
Error: Invalid JSON-RPC request
```

**Fix**:
Ensure request has all required fields:
```json
{
  "jsonrpc": "2.0",    # Required
  "id": 1,             # Required
  "method": "tools/call",  # Required
  "params": { ... }    # Required
}
```

### Issue: "Division by zero"

**Symptom**:
```json
{"error": {"message": "Division by zero"}}
```

**Fix**:
This is **expected behavior**! Your error handling works. Pass non-zero `b` value.

### Issue: Slow Performance

**Symptom**:
Operations take >10μs

**Fix**:
Use `--release` build:
```bash
cargo build --release
cargo run --release
```

Debug builds are 10x slower.

## Quality Gate Check

Before deploying, run the full quality gate:

```bash
cargo test                          # All tests pass
cargo tarpaulin --out Stdout        # 100% coverage
cargo clippy -- -D warnings         # No warnings
cargo fmt --check                   # Formatted
cargo bench                         # Performance verified
```

**If ANY check fails, DO NOT deploy.**

This is EXTREME TDD in action - quality gates prevent production issues.

## What You've Accomplished

You've built a **production-ready MCP server** that:

✅ Has zero boilerplate (26-line YAML config)
✅ Implements four arithmetic operations
✅ Handles errors gracefully (division by zero, unknown operations)
✅ Has 100% test coverage (6 comprehensive tests)
✅ Achieves <1μs dispatch performance
✅ Runs in 20 minutes of development time
✅ Passes all quality gates

**This is the power of EXTREME TDD + pforge.**

## Next Steps

Now that you've mastered the basics:

1. **Chapter 4**: Add state management to your servers
2. **Chapter 5**: Implement HTTP and CLI handlers
3. **Chapter 6**: Build production pipelines
4. **Chapter 7**: Add fault tolerance and retries

You have the foundation. Let's build something bigger.

---

> "Ship with confidence. Test-driven code doesn't fear production." - EXTREME TDD principle
