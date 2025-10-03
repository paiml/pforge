# PMAT Analysis Server - pforge Example

**Production-ready MCP server for code quality analysis and technical debt metrics.**

This example demonstrates advanced pforge capabilities:
- ✅ CLI tool integration (wrapping PMAT commands)
- ✅ Native handlers with complex business logic
- ✅ Multiple related tools in one server
- ✅ Structured JSON output
- ✅ Comprehensive error handling
- ✅ Unit testing for business logic

---

## Quick Start

**Prerequisites**:
- Rust 1.75+ and Cargo
- PMAT CLI installed (`cargo install pmat-cli`)

```bash
# Navigate to the example directory
cd examples/pmat-server

# Run the server
cargo run

# Expected output:
# ╔═══════════════════════════════════════╗
# ║   PMAT Analysis MCP Server            ║
# ║   Code Quality & Technical Debt       ║
# ║   Powered by pforge v0.1.0            ║
# ╚═══════════════════════════════════════╝
#
# This example demonstrates:
#   ✓ CLI tool integration (pmat commands)
#   ✓ Native handler with complex logic
#   ✓ Multiple analysis tools
#   ✓ Structured quality metrics
#   ✓ JSON output formatting
#
# Available tools:
#   • analyze_complexity() - Check cyclomatic complexity
#   • analyze_satd() - Detect technical debt comments
#   • analyze_tdg() - Calculate Technical Debt Grade
#   • analyze_cognitive() - Check cognitive complexity
#   • metrics_summary(path, include_history?) - Full metrics report
```

---

## What It Does

### Tools Provided

#### 1. **analyze_complexity** - CLI Handler
Analyzes cyclomatic complexity of code using PMAT.

- **Type**: CLI wrapper
- **Command**: `pmat analyze complexity --max 20 --format json`
- **Input**: None (analyzes current directory)
- **Output**: JSON with complexity metrics
- **Performance**: ~100ms (depends on codebase size)

#### 2. **analyze_satd** - CLI Handler
Detects Self-Admitted Technical Debt (SATD) comments.

- **Type**: CLI wrapper
- **Command**: `pmat analyze satd --max 0 --format json`
- **Input**: None
- **Output**: JSON with SATD comments found
- **Performance**: ~50ms

#### 3. **analyze_tdg** - CLI Handler
Calculates Technical Debt Grade (TDG).

- **Type**: CLI wrapper
- **Command**: `pmat analyze tdg --min 0.75 --format json`
- **Input**: None
- **Output**: JSON with TDG score
- **Performance**: ~150ms

#### 4. **analyze_cognitive** - CLI Handler
Analyzes cognitive complexity.

- **Type**: CLI wrapper
- **Command**: `pmat analyze cognitive --max 15 --format json`
- **Input**: None
- **Output**: JSON with cognitive complexity metrics
- **Performance**: ~100ms

#### 5. **metrics_summary** - Native Handler
Generates comprehensive code quality report by aggregating all analyses.

- **Type**: Native Rust handler
- **Input**:
  - `path` (required): File or directory to analyze
  - `include_history` (optional, default: false): Include historical trends
- **Output**: Structured JSON with:
  - Individual metric results (complexity, SATD, TDG, cognitive)
  - Overall quality grade (A+ to F)
  - Pass/fail status for each check
  - Actionable recommendations
- **Performance**: ~400ms (runs all 4 analyses)

### Example Usage with MCP Client

```json
// Request: Analyze specific directory
{
  "jsonrpc": "2.0",
  "id": 1,
  "method": "tools/call",
  "params": {
    "name": "metrics_summary",
    "arguments": {
      "path": "/path/to/codebase",
      "include_history": false
    }
  }
}

// Response: Comprehensive quality report
{
  "jsonrpc": "2.0",
  "id": 1,
  "result": {
    "complexity": {
      "passed": true,
      "value": "15",
      "threshold": "≤20"
    },
    "satd": {
      "passed": false,
      "value": "3",
      "threshold": "0"
    },
    "tdg": {
      "passed": true,
      "value": "0.85",
      "threshold": "≥0.75"
    },
    "cognitive": {
      "passed": true,
      "value": "12",
      "threshold": "≤15"
    },
    "summary": {
      "overall_grade": "A",
      "passed_checks": 3,
      "total_checks": 4,
      "recommendations": [
        "Remove or address Self-Admitted Technical Debt (SATD) comments"
      ]
    }
  }
}
```

---

## Architecture

### Project Structure

```
pmat-server/
├── pforge.yaml              # Declarative configuration (5 tools)
├── Cargo.toml               # Rust dependencies
├── src/
│   ├── main.rs              # Server entry point
│   └── handlers/
│       ├── mod.rs           # Handler module exports
│       └── metrics.rs       # Native metrics aggregation handler
└── README.md                # This file
```

### Configuration (`pforge.yaml`)

The YAML configuration defines 4 CLI wrappers and 1 native handler:

```yaml
forge:
  name: pmat-server
  version: 0.1.0
  transport: stdio
  description: "PMAT Code Analysis MCP Server"

tools:
  # CLI wrappers for individual PMAT commands
  - type: cli
    name: analyze_complexity
    description: "Analyze cyclomatic complexity"
    command: pmat
    args:
      - analyze
      - complexity
      - "--max"
      - "20"
      - "--format"
      - "json"
    stream: false

  # ... (3 more CLI handlers)

  # Native handler for aggregated analysis
  - type: native
    name: metrics_summary
    description: "Generate comprehensive code metrics summary"
    handler:
      path: handlers::metrics::MetricsSummary
    params:
      path:
        type: string
        required: true
        description: "Path to analyze"
      include_history:
        type: boolean
        required: false
        default: false
```

### Native Handler (`src/handlers/metrics.rs`)

The `MetricsSummary` handler demonstrates:

**1. Complex Business Logic**:
```rust
async fn handle(&self, input: Self::Input) -> Result<Self::Output> {
    // Run all 4 PMAT analyses in sequence
    let complexity_result = run_pmat_command(&["analyze", "complexity", ...])?;
    let satd_result = run_pmat_command(&["analyze", "satd", ...])?;
    let tdg_result = run_pmat_command(&["analyze", "tdg", ...])?;
    let cognitive_result = run_pmat_command(&["analyze", "cognitive", ...])?;

    // Parse and aggregate results
    let passed_checks = calculate_passed_checks(&results);
    let overall_grade = calculate_grade(passed_checks, 4);
    let recommendations = generate_recommendations(&results);

    // Return structured output
    Ok(MetricsSummaryOutput { ... })
}
```

**2. Subprocess Management**:
```rust
fn run_pmat_command(args: &[&str]) -> Result<String> {
    let output = Command::new("pmat")
        .args(args)
        .output()
        .map_err(|e| pforge_runtime::Error::Handler(...))?;

    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}
```

**3. Result Processing**:
```rust
fn calculate_grade(passed: u32, total: u32) -> String {
    let percentage = (passed as f64 / total as f64) * 100.0;
    match percentage as u32 {
        100 => "A+",
        90..=99 => "A",
        80..=89 => "B",
        // ...
    }
}
```

---

## Testing

### Run Unit Tests

```bash
cargo test
```

### Test Output

```
running 3 tests
test handlers::metrics::tests::test_calculate_grade ... ok
test handlers::metrics::tests::test_generate_recommendations_all_pass ... ok
test handlers::metrics::tests::test_generate_recommendations_some_fail ... ok

test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

### Integration Testing

Test with a real MCP client:

```bash
# Start the server
cargo run

# In another terminal, use an MCP client to test
echo '{"jsonrpc":"2.0","id":1,"method":"tools/call","params":{"name":"analyze_complexity"}}' | \
  cargo run
```

---

## Development Workflow

### Add a New Analysis Tool

**Step 1**: Add to `pforge.yaml`

```yaml
tools:
  - type: cli
    name: analyze_security
    description: "Run security audit"
    command: cargo
    args:
      - audit
      - --json
    stream: false
```

**Step 2**: No code needed! (CLI handler auto-registered)

**Step 3**: Update `metrics_summary` handler (optional)

```rust
// Add to MetricsSummaryOutput
pub struct MetricsSummaryOutput {
    // ...existing fields...
    pub security: MetricsResult,  // Add this
}

// Add to handle() function
let security_result = run_pmat_command(&["cargo", "audit", "--json"])?;
```

### Customize Grading Logic

Edit `src/handlers/metrics.rs`:

```rust
fn calculate_grade(passed: u32, total: u32) -> String {
    let percentage = (passed as f64 / total as f64) * 100.0;

    // Custom grading scale
    match percentage as u32 {
        95..=100 => "Excellent",
        85..=94 => "Good",
        70..=84 => "Fair",
        _ => "Needs Improvement",
    }
}
```

---

## Use Cases

### 1. CI/CD Quality Gates

Use as a pre-commit hook or CI check:

```bash
# In CI pipeline
pforge-mcp-client metrics_summary --path . | jq '.summary.overall_grade'
# Output: "A+"

# Fail build if grade < B
if [ "$GRADE" < "B" ]; then exit 1; fi
```

### 2. Code Review Assistant

Integrate with code review tools:

```python
# GitHub Action example
import json
import subprocess

result = subprocess.run(
    ["pforge-mcp-client", "metrics_summary", "--path", "./pr_diff"],
    capture_output=True
)

metrics = json.loads(result.stdout)
recommendations = metrics["summary"]["recommendations"]

# Post as PR comment
github.create_comment(pr_number, "\n".join(recommendations))
```

### 3. Technical Debt Dashboard

Build a dashboard showing quality trends:

```javascript
// Fetch metrics periodically
const metrics = await mcpClient.call("metrics_summary", {
  path: "./src",
  include_history: true
});

// Display in web dashboard
dashboard.update({
  grade: metrics.summary.overall_grade,
  trend: metrics.complexity.historical_trend,
  alerts: metrics.summary.recommendations
});
```

---

## Troubleshooting

### PMAT Command Not Found

**Problem**: `Error: failed to run pmat: No such file or directory`

**Solution**: Install PMAT CLI
```bash
cargo install pmat-cli
```

### Permission Denied

**Problem**: `Error: Permission denied (os error 13)`

**Solution**: Ensure PMAT has execute permissions
```bash
chmod +x $(which pmat)
```

### JSON Parse Errors

**Problem**: `Error: failed to parse PMAT output`

**Solution**: Ensure PMAT outputs JSON format
```bash
# Verify PMAT JSON output
pmat analyze complexity --format json
```

### Timeout Issues

**Problem**: Analysis takes too long on large codebases

**Solution**: Add timeout configuration
```yaml
tools:
  - type: cli
    name: analyze_complexity
    command: pmat
    args: [...]
    timeout_ms: 60000  # 60 seconds
```

---

## Performance

### Benchmarks (on M1 Mac, 10,000 LOC codebase)

| Tool | Execution Time | Notes |
|------|---------------|-------|
| **analyze_complexity** | ~80ms | Fast heuristic analysis |
| **analyze_satd** | ~40ms | Simple regex matching |
| **analyze_tdg** | ~120ms | Multi-metric calculation |
| **analyze_cognitive** | ~90ms | AST traversal |
| **metrics_summary** | ~350ms | Runs all 4 + aggregation |

### Optimization Tips

1. **Parallel Execution**: Run analyses concurrently
   ```rust
   let (complexity, satd, tdg, cognitive) = tokio::join!(
       tokio::spawn(run_pmat_command(...)),
       tokio::spawn(run_pmat_command(...)),
       tokio::spawn(run_pmat_command(...)),
       tokio::spawn(run_pmat_command(...)),
   );
   ```

2. **Caching**: Cache results for unchanged files
   ```rust
   let cache_key = format!("{}-{}", path, file_hash);
   if let Some(cached) = cache.get(&cache_key) {
       return cached;
   }
   ```

3. **Incremental Analysis**: Only analyze changed files
   ```rust
   let changed_files = git_diff();
   for file in changed_files {
       analyze(file);
   }
   ```

---

## Advanced Features

### State Management

Track quality trends over time:

```yaml
# Add to pforge.yaml
state:
  backend: sled
  path: ./quality_history.db
  ttl_seconds: 2592000  # 30 days
```

```rust
// In handler
use pforge_runtime::StateManager;

let previous_grade = state.get("last_grade").await?;
let trend = if current_grade > previous_grade { "↑" } else { "↓" };
state.set("last_grade", current_grade).await?;
```

### Middleware Integration

Add logging and metrics:

```rust
use pforge_runtime::{LoggingMiddleware, MetricsMiddleware};

let server = McpServer::new(config)
    .with_middleware(LoggingMiddleware::new())
    .with_middleware(MetricsMiddleware::new());
```

### Custom Output Formats

Support multiple output formats:

```rust
#[derive(Serialize)]
pub enum OutputFormat {
    Json,
    Html,
    Markdown,
}

impl MetricsSummary {
    fn format_output(&self, results: &MetricsSummaryOutput, format: OutputFormat) -> String {
        match format {
            OutputFormat::Json => serde_json::to_string_pretty(results).unwrap(),
            OutputFormat::Html => generate_html_report(results),
            OutputFormat::Markdown => generate_markdown_report(results),
        }
    }
}
```

---

## Next Steps

After understanding this example:

1. **Explore Other Examples**:
   - `examples/hello-world/` - Basic concepts
   - `examples/polyglot-server/` - Multi-language handlers
   - `examples/production-server/` - Full production setup

2. **Build Your Own Analysis Server**:
   ```bash
   pforge new my-analysis-server
   cd my-analysis-server
   # Add your custom analysis tools
   pforge serve
   ```

3. **Extend This Example**:
   - Add security scanning (cargo-audit, cargo-deny)
   - Integrate with external APIs (SonarQube, CodeClimate)
   - Add historical trend analysis
   - Build a web dashboard

---

## Learn More

- **PMAT Documentation**: https://github.com/paiml/pmat
- **pforge User Guide**: [../../USER_GUIDE.md](../../USER_GUIDE.md)
- **MCP Protocol**: https://spec.modelcontextprotocol.io/
- **pforge Repository**: https://github.com/paiml/pforge

---

**License**: MIT

**Maintained by**: pforge team

**Version**: 0.1.0
