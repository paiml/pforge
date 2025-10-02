# Chapter 15: Benchmarking and Profiling

Rigorous benchmarking is essential for maintaining pforge's performance guarantees. This chapter covers the tools, techniques, and methodologies for measuring and tracking performance across the entire development lifecycle.

## Benchmarking Philosophy

**Key Principles**:
1. **Measure, don't guess**: Intuition about performance is often wrong
2. **Isolate variables**: Benchmark one thing at a time
3. **Statistical rigor**: Account for variance and outliers
4. **Continuous tracking**: Prevent performance regressions
5. **Representative workloads**: Test realistic scenarios

## Criterion: Statistical Benchmarking

Criterion is pforge's primary benchmarking framework, providing statistical analysis and regression detection.

### Basic Benchmark Structure

```rust
// benches/dispatch_benchmark.rs
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use pforge_runtime::HandlerRegistry;

fn bench_handler_dispatch(c: &mut Criterion) {
    let mut registry = HandlerRegistry::new();
    registry.register("test_tool", TestHandler);

    let params = serde_json::to_vec(&TestInput {
        value: "test".to_string(),
    }).unwrap();

    c.bench_function("handler_dispatch", |b| {
        b.iter(|| {
            let result = registry.dispatch(
                black_box("test_tool"),
                black_box(&params),
            );
            black_box(result)
        });
    });
}

criterion_group!(benches, bench_handler_dispatch);
criterion_main!(benches);
```

**Key Functions**:
- `black_box()`: Prevents compiler from optimizing away benchmarked code
- `c.bench_function()`: Runs benchmark with automatic iteration count
- `b.iter()`: Inner benchmark loop

### Running Benchmarks

```bash
# Run all benchmarks
cargo bench

# Run specific benchmark
cargo bench --bench dispatch_benchmark

# Run with filtering
cargo bench handler

# Baseline comparison
cargo bench --save-baseline baseline-v1
# ... make changes ...
cargo bench --baseline baseline-v1

# Generate HTML report
open target/criterion/report/index.html
```

### Benchmark Output

```
handler_dispatch        time:   [812.34 ns 815.92 ns 820.18 ns]
                        change: [-2.3421% -1.2103% +0.1234%] (p = 0.08 > 0.05)
                        No change in performance detected.
Found 3 outliers among 100 measurements (3.00%)
  2 (2.00%) high mild
  1 (1.00%) high severe
```

**Reading Results**:
- **time**: [lower bound, estimate, upper bound] at 95% confidence
- **change**: Performance delta vs previous run
- **outliers**: Data points removed from statistical analysis
- **p-value**: Statistical significance (< 0.05 = significant change)

### Parametric Benchmarks

Compare performance across different input sizes:

```rust
use criterion::BenchmarkId;

fn bench_registry_scaling(c: &mut Criterion) {
    let mut group = c.benchmark_group("registry_scaling");

    for size in [10, 50, 100, 500, 1000].iter() {
        group.bench_with_input(
            BenchmarkId::from_parameter(size),
            size,
            |b, &size| {
                let mut registry = HandlerRegistry::new();

                // Register `size` handlers
                for i in 0..size {
                    registry.register(
                        Box::leak(format!("tool_{}", i).into_boxed_str()),
                        TestHandler,
                    );
                }

                b.iter(|| {
                    registry.get(black_box("tool_0"))
                });
            },
        );
    }

    group.finish();
}
```

**Output**:
```
registry_scaling/10     time:   [6.8234 ns 6.9102 ns 7.0132 ns]
registry_scaling/50     time:   [7.1245 ns 7.2103 ns 7.3098 ns]
registry_scaling/100    time:   [7.3456 ns 7.4523 ns 7.5612 ns]
registry_scaling/500    time:   [8.1234 ns 8.2345 ns 8.3456 ns]
registry_scaling/1000   time:   [8.5678 ns 8.6789 ns 8.7890 ns]
```

**Analysis**: O(1) confirmed - minimal scaling with registry size

### Throughput Benchmarks

Measure operations per second:

```rust
use criterion::Throughput;

fn bench_json_parsing(c: &mut Criterion) {
    let mut group = c.benchmark_group("json_parsing");

    for size in [100, 1024, 10240].iter() {
        let json = generate_json(*size);

        group.throughput(Throughput::Bytes(*size as u64));
        group.bench_with_input(
            BenchmarkId::from_parameter(size),
            &json,
            |b, json| {
                b.iter(|| {
                    serde_json::from_slice::<TestStruct>(black_box(json))
                });
            },
        );
    }

    group.finish();
}
```

**Output**:
```
json_parsing/100        time:   [412.34 ns 415.92 ns 420.18 ns]
                        thrpt:  [237.95 MiB/s 240.35 MiB/s 242.51 MiB/s]

json_parsing/1024       time:   [3.1234 μs 3.2103 μs 3.3098 μs]
                        thrpt:  [309.45 MiB/s 318.92 MiB/s 327.81 MiB/s]
```

### Custom Measurement

For async code or complex setups:

```rust
use criterion::measurement::WallTime;
use criterion::BenchmarkGroup;
use tokio::runtime::Runtime;

fn bench_async_handler(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();

    c.bench_function("async_handler", |b| {
        b.to_async(&rt).iter(|| async {
            let handler = TestHandler;
            let input = TestInput { value: "test".to_string() };
            black_box(handler.handle(input).await)
        });
    });
}
```

## Flamegraphs: Visual CPU Profiling

Flamegraphs show where CPU time is spent in your application.

### Generating Flamegraphs

```bash
# Install cargo-flamegraph
cargo install flamegraph

# Generate flamegraph (Linux/macOS)
cargo flamegraph --bin pforge -- serve

# On macOS, may need sudo:
sudo cargo flamegraph --bin pforge -- serve

# Run workload (in another terminal)
# Send test requests to the server
# Press Ctrl+C to stop profiling

# View flamegraph.svg
open flamegraph.svg
```

### Reading Flamegraphs

**Anatomy**:
- **X-axis**: Alphabetical function ordering (NOT time order!)
- **Y-axis**: Call stack depth
- **Width**: Proportion of CPU time
- **Color**: Random (helps distinguish adjacent functions)

**What to look for**:
1. **Wide boxes**: Functions consuming significant CPU time
2. **Tall stacks**: Deep call chains (potential for inlining)
3. **Repeated patterns**: Opportunities for caching or deduplication
4. **Unexpected functions**: Accidental expensive operations

**Example Analysis**:

```
[====== serde_json::de::from_slice (45%) ======]
       [=== CalculateHandler::handle (30%) ===]
              [= registry lookup (10%) =]
                     [other (15%)]
```

**Interpretation**:
- JSON deserialization is the hottest path (45%)
- Handler execution is second (30%)
- Registry lookup is minimal (10%) - good!

### Differential Flamegraphs

Compare before/after optimization:

```bash
# Before optimization
cargo flamegraph --bin pforge -o before.svg -- serve
# ... run workload ...

# After optimization
cargo flamegraph --bin pforge -o after.svg -- serve
# ... run same workload ...

# Generate diff
diffflame before.svg after.svg > diff.svg
```

**Diff Flamegraph Colors**:
- **Red**: Increased CPU time (regression)
- **Blue**: Decreased CPU time (improvement)
- **Gray**: No significant change

## Memory Profiling

### Valgrind/Massif for Heap Profiling

```bash
# Run with massif (heap profiler)
valgrind --tool=massif \
         --massif-out-file=massif.out \
         ./target/release/pforge serve

# Visualize with massif-visualizer
massif-visualizer massif.out

# Or text analysis
ms_print massif.out
```

**Massif Output**:
```
    MB
10 ^                                      #
   |                                    @:#
   |                                  @@@:#
 8 |                                @@@@:#
   |                              @@@@@@:#
   |                            @@@@@@@@:#
 6 |                          @@@@@@@@@@:#
   |                        @@@@@@@@@@@@:#
   |                      @@@@@@@@@@@@@@:#
 4 |                    @@@@@@@@@@@@@@@@:#
   |                  @@@@@@@@@@@@@@@@@@:#
   |                @@@@@@@@@@@@@@@@@@@@:#
 2 |              @@@@@@@@@@@@@@@@@@@@@@:#
   |            @@@@@@@@@@@@@@@@@@@@@@@@:#
   |@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@:#
 0 +--------------------------------------->ki
   0                                   1000

Number      Allocated     Frequency
-------     ---------     ---------
1           2.5 MB        45%         serde_json::de::from_slice
2           1.8 MB        32%         HandlerRegistry::register
3           0.7 MB        12%         String allocations
```

### dhat-rs for Allocation Profiling

```rust
// Add to main.rs or lib.rs
#[cfg(feature = "dhat-heap")]
#[global_allocator]
static ALLOC: dhat::Alloc = dhat::Alloc;

fn main() {
    #[cfg(feature = "dhat-heap")]
    let _profiler = dhat::Profiler::new_heap();

    // ... rest of main ...
}
```

```toml
# Cargo.toml
[features]
dhat-heap = ["dhat"]

[dependencies]
dhat = { version = "0.3", optional = true }
```

```bash
# Run with heap profiling
cargo run --release --features dhat-heap

# Generates dhat-heap.json

# View in Firefox Profiler
# Open: https://profiler.firefox.com/
# Load dhat-heap.json
```

**dhat Report**:
- Total allocations
- Total bytes allocated
- Peak heap usage
- Allocation hot spots
- Allocation lifetimes

## System-Level Profiling with perf

```bash
# Record performance counters (Linux only)
perf record -F 99 -g --call-graph dwarf ./target/release/pforge serve

# Run workload, then Ctrl+C

# Analyze
perf report

# Generate flamegraph
perf script | stackcollapse-perf.pl | flamegraph.pl > perf.svg

# Advanced: CPU cache misses
perf record -e cache-misses,cache-references ./target/release/pforge serve
perf report

# Branch prediction
perf record -e branch-misses,branches ./target/release/pforge serve
perf report
```

**perf stat** for quick metrics:

```bash
perf stat ./target/release/pforge serve
# Run workload, then Ctrl+C

# Output:
# Performance counter stats for './target/release/pforge serve':
#
#       1,234.56 msec task-clock                #    0.987 CPUs utilized
#             42      context-switches          #    0.034 K/sec
#              3      cpu-migrations            #    0.002 K/sec
#          1,234      page-faults               #    1.000 K/sec
#  4,567,890,123      cycles                    #    3.700 GHz
#  8,901,234,567      instructions              #    1.95  insn per cycle
#  1,234,567,890      branches                  # 1000.000 M/sec
#     12,345,678      branch-misses             #    1.00% of all branches
```

## Tokio Console: Async Runtime Profiling

Monitor async tasks and detect blocking operations:

```toml
# Cargo.toml
[dependencies]
console-subscriber = "0.2"
tokio = { version = "1", features = ["full", "tracing"] }
```

```rust
fn main() {
    console_subscriber::init();

    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(async {
            run_server().await
        });
}
```

```bash
# Terminal 1: Run server with console
cargo run --release

# Terminal 2: Start tokio-console
tokio-console

# Interact with TUI:
# - View task list
# - See task durations
# - Identify blocking tasks
# - Monitor resource usage
```

**Tokio Console Views**:

1. **Tasks View**: All async tasks
   ```
   ID    STATE      TOTAL    BUSY    IDLE    POLLS
   1     Running    500ms    300ms   200ms   1234
   2     Idle       2.1s     100ms   2.0s    567
   ```

2. **Resources View**: Sync primitives
   ```
   TYPE           TOTAL    OPENED   CLOSED
   tcp::Stream    45       50       5
   Mutex          12       12       0
   ```

3. **Async Operations**: Await points
   ```
   LOCATION                TOTAL    AVG      MAX
   handler.rs:45           1234     2.3ms    50ms
   registry.rs:89          567      0.8ms    5ms
   ```

## Load Testing

### wrk for HTTP Load Testing

```bash
# Install wrk
# macOS: brew install wrk
# Linux: apt-get install wrk

# Basic load test (SSE transport)
wrk -t4 -c100 -d30s http://localhost:3000/sse

# With custom script
wrk -t4 -c100 -d30s -s loadtest.lua http://localhost:3000/sse
```

```lua
-- loadtest.lua
request = function()
   body = [[{
     "jsonrpc": "2.0",
     "method": "tools/call",
     "params": {
       "name": "calculate",
       "arguments": {"operation": "add", "a": 5, "b": 3}
     },
     "id": 1
   }]]

   return wrk.format("POST", "/sse", nil, body)
end

response = function(status, headers, body)
   if status ~= 200 then
      print("Error: " .. status)
   end
end
```

**wrk Output**:
```
Running 30s test @ http://localhost:3000/sse
  4 threads and 100 connections
  Thread Stats   Avg      Stdev     Max   +/- Stdev
    Latency     1.23ms    2.45ms   50.00ms   89.12%
    Req/Sec    32.5k     3.2k    40.0k    75.00%
  3900000 requests in 30.00s, 1.50GB read
Requests/sec: 130000.00
Transfer/sec:     51.20MB
```

### Custom Load Testing

```rust
// tests/load_test.rs
use tokio::time::{Duration, Instant};
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};

#[tokio::test(flavor = "multi_thread", worker_threads = 8)]
async fn load_test_concurrent() {
    let server = start_test_server().await;
    let success_count = Arc::new(AtomicU64::new(0));
    let error_count = Arc::new(AtomicU64::new(0));

    let start = Instant::now();
    let duration = Duration::from_secs(30);

    let mut tasks = vec![];

    // Spawn 100 concurrent clients
    for _ in 0..100 {
        let success = success_count.clone();
        let errors = error_count.clone();

        tasks.push(tokio::spawn(async move {
            while start.elapsed() < duration {
                match send_request().await {
                    Ok(_) => success.fetch_add(1, Ordering::Relaxed),
                    Err(_) => errors.fetch_add(1, Ordering::Relaxed),
                };
            }
        }));
    }

    // Wait for all tasks
    for task in tasks {
        task.await.unwrap();
    }

    let elapsed = start.elapsed();
    let total_requests = success_count.load(Ordering::Relaxed);
    let total_errors = error_count.load(Ordering::Relaxed);

    println!("Load Test Results:");
    println!("  Duration: {:?}", elapsed);
    println!("  Successful requests: {}", total_requests);
    println!("  Failed requests: {}", total_errors);
    println!("  Requests/sec: {:.2}", total_requests as f64 / elapsed.as_secs_f64());

    assert!(total_errors < total_requests / 100); // < 1% error rate
    assert!(total_requests / elapsed.as_secs() > 50000); // > 50K req/s
}
```

## Continuous Benchmarking

### GitHub Actions Integration

```yaml
# .github/workflows/bench.yml
name: Benchmarks

on:
  push:
    branches: [main]
  pull_request:

jobs:
  benchmark:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - uses: dtolnay/rust-toolchain@stable

      - name: Run benchmarks
        run: cargo bench --bench dispatch_benchmark

      - name: Store benchmark result
        uses: benchmark-action/github-action-benchmark@v1
        with:
          tool: 'criterion'
          output-file-path: target/criterion/dispatch_benchmark/base/estimates.json
          github-token: ${{ secrets.GITHUB_TOKEN }}
          auto-push: true
          alert-threshold: '110%'  # Alert if 10% slower
          comment-on-alert: true
          fail-on-alert: true
```

### Benchmark Dashboard

Track performance over time:

```yaml
# Separate job for dashboard update
dashboard:
  needs: benchmark
  runs-on: ubuntu-latest
  steps:
    - uses: benchmark-action/github-action-benchmark@v1
      with:
        tool: 'criterion'
        output-file-path: target/criterion/dispatch_benchmark/base/estimates.json
        github-token: ${{ secrets.GITHUB_TOKEN}}
        gh-pages-branch: gh-pages
        benchmark-data-dir-path: 'dev/bench'
```

View at: `https://your-org.github.io/pforge/dev/bench/`

## Benchmark Best Practices

### 1. Warm-Up Phase

```rust
fn bench_with_warmup(c: &mut Criterion) {
    let mut group = c.benchmark_group("with_warmup");
    group.warm_up_time(Duration::from_secs(3)); // Warm up JIT, caches
    group.measurement_time(Duration::from_secs(10)); // Longer measurement

    group.bench_function("handler", |b| {
        b.iter(|| expensive_operation());
    });

    group.finish();
}
```

### 2. Isolate External Factors

```rust
// Bad: Includes setup time
fn bench_bad(c: &mut Criterion) {
    c.bench_function("bad", |b| {
        b.iter(|| {
            let registry = HandlerRegistry::new(); // Setup in measurement!
            registry.dispatch("tool", &params)
        });
    });
}

// Good: Setup outside measurement
fn bench_good(c: &mut Criterion) {
    let registry = HandlerRegistry::new(); // Setup once

    c.bench_function("good", |b| {
        b.iter(|| {
            registry.dispatch("tool", &params) // Only measure dispatch
        });
    });
}
```

### 3. Representative Data

```rust
fn bench_realistic(c: &mut Criterion) {
    // Use realistic input sizes
    let small_input = generate_input(100);
    let medium_input = generate_input(1024);
    let large_input = generate_input(10240);

    c.bench_function("small", |b| b.iter(|| process(&small_input)));
    c.bench_function("medium", |b| b.iter(|| process(&medium_input)));
    c.bench_function("large", |b| b.iter(|| process(&large_input)));
}
```

### 4. Prevent Compiler Optimizations

```rust
use criterion::black_box;

// Bad: Compiler might optimize away the call
fn bench_bad(c: &mut Criterion) {
    c.bench_function("bad", |b| {
        b.iter(|| {
            let result = expensive_function();
            // Result never used - might be optimized away!
        });
    });
}

// Good: Use black_box
fn bench_good(c: &mut Criterion) {
    c.bench_function("good", |b| {
        b.iter(|| {
            let result = expensive_function();
            black_box(result) // Prevents optimization
        });
    });
}
```

## Performance Regression Testing

### Automated Performance Tests

```rust
// tests/performance_test.rs
#[test]
fn test_dispatch_latency_sla() {
    let mut registry = HandlerRegistry::new();
    registry.register("test", TestHandler);

    let params = serde_json::to_vec(&TestInput::default()).unwrap();

    let start = std::time::Instant::now();
    let iterations = 10000;

    for _ in 0..iterations {
        let _ = registry.dispatch("test", &params);
    }

    let elapsed = start.elapsed();
    let avg_latency = elapsed / iterations;

    // SLA: Average latency must be < 1μs
    assert!(
        avg_latency < Duration::from_micros(1),
        "Dispatch latency {} exceeds 1μs SLA",
        avg_latency.as_nanos()
    );
}

#[test]
fn test_memory_usage() {
    use sysinfo::{ProcessExt, System, SystemExt};

    let mut sys = System::new_all();
    let pid = sysinfo::get_current_pid().unwrap();

    sys.refresh_process(pid);
    let baseline = sys.process(pid).unwrap().memory();

    // Register 1000 handlers
    let mut registry = HandlerRegistry::new();
    for i in 0..1000 {
        registry.register(Box::leak(format!("tool_{}", i).into_boxed_str()), TestHandler);
    }

    sys.refresh_process(pid);
    let after = sys.process(pid).unwrap().memory();

    let per_handler = (after - baseline) / 1000;

    // SLA: < 256 bytes per handler
    assert!(
        per_handler < 256,
        "Memory per handler {} exceeds 256B SLA",
        per_handler
    );
}
```

## Summary

Effective benchmarking requires:

1. **Statistical rigor**: Use Criterion for reliable measurements
2. **Visual profiling**: Flamegraphs show where time is spent
3. **Memory awareness**: Profile allocations and heap usage
4. **Continuous tracking**: Automate benchmarks in CI/CD
5. **Realistic workloads**: Test production-like scenarios
6. **SLA enforcement**: Fail tests on regression

**Benchmarking workflow**:
1. Measure baseline with Criterion
2. Profile with flamegraphs to find hot paths
3. Optimize hot paths
4. Verify improvement with Criterion
5. Add regression test
6. Commit with confidence

**Next chapter**: Code generation internals - how pforge transforms YAML into optimized Rust.
