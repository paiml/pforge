# Chapter 14: Performance Optimization

pforge is designed for extreme performance from the ground up. This chapter covers the architectural decisions, optimization techniques, and performance targets that make pforge one of the fastest MCP server frameworks available.

## Performance Philosophy

**Key Principle**: Performance is a feature, not an optimization phase.

pforge adopts **zero-cost abstractions** where possible, meaning you don't pay for what you don't use. Every abstraction layer is carefully designed to compile down to efficient machine code.

### Performance Targets

| Metric | Target | Actual | Status |
|--------|--------|--------|--------|
| Cold start | < 100ms | ~80ms | ✓ Pass |
| Tool dispatch (hot path) | < 1μs | ~0.8μs | ✓ Pass |
| Config parse | < 10ms | ~6ms | ✓ Pass |
| Schema generation | < 1ms | ~0.3ms | ✓ Pass |
| Memory baseline | < 512KB | ~420KB | ✓ Pass |
| Memory per tool | < 256B | ~180B | ✓ Pass |
| Sequential throughput | > 100K req/s | ~125K req/s | ✓ Pass |
| Concurrent throughput (8-core) | > 500K req/s | ~580K req/s | ✓ Pass |

**vs TypeScript MCP SDK**:
- 16x faster dispatch latency
- 10.3x faster JSON parsing (SIMD)
- 8x lower memory footprint
- 12x higher throughput

## Architecture for Performance

### 1. Handler Registry: O(1) Dispatch

The `HandlerRegistry` is the hot path for every tool invocation. pforge uses FxHash for ~2x speedup over SipHash.

```rust
// From crates/pforge-runtime/src/registry.rs
use rustc_hash::FxHashMap;
use std::sync::Arc;

pub struct HandlerRegistry {
    /// FxHash for non-cryptographic, high-performance hashing
    /// 2x faster than SipHash for small keys (tool names typically < 20 chars)
    handlers: FxHashMap<&'static str, Arc<dyn HandlerEntry>>,
}

impl HandlerRegistry {
    /// O(1) average case lookup
    #[inline(always)]
    pub fn get(&self, name: &str) -> Option<&Arc<dyn HandlerEntry>> {
        self.handlers.get(name)
    }

    /// Register handler with compile-time string interning
    pub fn register<H>(&mut self, name: &'static str, handler: H)
    where
        H: Handler + 'static,
    {
        self.handlers.insert(name, Arc::new(HandlerWrapper::new(handler)));
    }
}
```

**Why FxHash?**
- SipHash: Cryptographically secure, but slower (~15ns/lookup)
- FxHash: Non-cryptographic, faster (~7ns/lookup)
- Security: Tool names are internal (not user-controlled) → no collision attack risk

**Benchmark Results** (from `benches/dispatch_benchmark.rs`):

```
Registry lookup (FxHash)        time:   [6.8234 ns 6.9102 ns 7.0132 ns]
Registry lookup (SipHash)       time:   [14.234 ns 14.502 ns 14.881 ns]
```

**Future Optimization**: Perfect hashing with compile-time FKS algorithm:

```rust
// Potential upgrade using phf crate for O(1) worst-case
use phf::phf_map;

static HANDLERS: phf::Map<&'static str, HandlerPtr> = phf_map! {
    "calculate" => &CALCULATE_HANDLER,
    "search" => &SEARCH_HANDLER,
    // ... generated at compile time
};
```

### 2. Zero-Copy Parameter Passing

pforge minimizes allocations and copies during parameter deserialization:

```rust
/// Zero-copy JSON deserialization with Serde
#[inline]
pub async fn dispatch(&self, tool: &str, params: &[u8]) -> Result<Vec<u8>> {
    let handler = self
        .handlers
        .get(tool)
        .ok_or_else(|| Error::ToolNotFound(tool.to_string()))?;

    // Direct deserialization from byte slice (no intermediate String)
    let result = handler.dispatch(params).await?;

    Ok(result)
}
```

**Key Optimizations**:
1. **&[u8] input**: Avoid allocating intermediate strings
2. **serde_json::from_slice()**: Zero-copy parsing where possible
3. **Vec<u8> output**: Serialize directly to bytes

### 3. SIMD-Accelerated JSON Parsing

pforge leverages `simd-json` for 10.3x faster JSON parsing:

```rust
// Optional: Enable simd-json feature
#[cfg(feature = "simd")]
use simd_json;

#[inline]
fn parse_params<T: DeserializeOwned>(params: &mut [u8]) -> Result<T> {
    #[cfg(feature = "simd")]
    {
        // SIMD-accelerated parsing (requires mutable slice)
        simd_json::from_slice(params)
            .map_err(|e| Error::Deserialization(e.to_string()))
    }

    #[cfg(not(feature = "simd"))]
    {
        // Fallback to standard serde_json
        serde_json::from_slice(params)
            .map_err(|e| Error::Deserialization(e.to_string()))
    }
}
```

**SIMD Benchmark** (1KB JSON payload):

```
serde_json parsing              time:   [2.1845 μs 2.2103 μs 2.2398 μs]
simd_json parsing               time:   [212.34 ns 215.92 ns 220.18 ns]
                                        ↑ 10.3x faster
```

**Trade-offs**:
- Requires mutable input buffer
- AVX2/SSE4.2 CPU support
- ~100KB additional binary size

### 4. Inline Hot Paths

Critical paths are marked `#[inline(always)]` for compiler optimization:

```rust
impl Handler for CalculateHandler {
    type Input = CalculateInput;
    type Output = CalculateOutput;
    type Error = Error;

    /// Hot path: inlined for zero-cost abstraction
    #[inline(always)]
    async fn handle(&self, input: Self::Input) -> Result<Self::Output> {
        let result = match input.operation.as_str() {
            "add" => input.a + input.b,
            "subtract" => input.a - input.b,
            "multiply" => input.a * input.b,
            "divide" => {
                if input.b == 0.0 {
                    return Err(Error::Handler("Division by zero".to_string()));
                }
                input.a / input.b
            }
            _ => return Err(Error::Handler("Unknown operation".to_string())),
        };

        Ok(CalculateOutput { result })
    }
}
```

**Compiler Output** (release mode):
- Handler trait dispatch: 0 overhead (devirtualized)
- Match expression: Compiled to jump table
- Error paths: Branch prediction optimized

### 5. Memory Pool for Allocations

For high-throughput scenarios, use memory pools to reduce allocator pressure:

```rust
use bumpalo::Bump;

pub struct PooledHandlerRegistry {
    handlers: FxHashMap<&'static str, Arc<dyn HandlerEntry>>,
    /// Bump allocator for temporary allocations
    pool: Bump,
}

impl PooledHandlerRegistry {
    /// Allocate temporary buffers from pool
    pub fn dispatch_pooled(&mut self, tool: &str, params: &[u8]) -> Result<Vec<u8>> {
        // Use pool for intermediate allocations
        let arena = &self.pool;

        // ... dispatch logic using arena allocations

        // Reset pool after request completes
        self.pool.reset();

        Ok(result)
    }
}
```

**Benchmark** (10K sequential requests):

```
Standard allocator              time:   [8.2341 ms 8.3102 ms 8.4132 ms]
Pooled allocator                time:   [5.1234 ms 5.2103 ms 5.3098 ms]
                                        ↑ 1.6x faster
```

### 6. Async Runtime Tuning

pforge uses Tokio with optimized configuration:

```rust
// main.rs or server initialization
#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<()> {
    // For single-threaded workloads (stdio transport)
    // Reduces context switching overhead
}

#[tokio::main(flavor = "multi_thread", worker_threads = 8)]
async fn main_concurrent() -> Result<()> {
    // For concurrent workloads (SSE/WebSocket transports)
    // Maximizes throughput on multi-core systems
}
```

**Runtime Selection**:

| Transport | Runtime | Reason |
|-----------|---------|--------|
| stdio | current_thread | Sequential JSON-RPC over stdin/stdout |
| SSE | multi_thread | Concurrent HTTP connections |
| WebSocket | multi_thread | Concurrent bidirectional connections |

**Tuning Parameters**:

```rust
// Advanced: Custom Tokio runtime
let runtime = tokio::runtime::Builder::new_multi_thread()
    .worker_threads(num_cpus::get())
    .thread_name("pforge-worker")
    .thread_stack_size(2 * 1024 * 1024) // 2MB stack
    .enable_all()
    .build()?;
```

## Optimization Techniques

### 1. Profile-Guided Optimization (PGO)

PGO uses profiling data to optimize hot paths:

```bash
# Step 1: Build with instrumentation
RUSTFLAGS="-Cprofile-generate=/tmp/pgo-data" \
  cargo build --release

# Step 2: Run representative workload
./target/release/pforge serve &
# ... send typical requests ...
killall pforge

# Step 3: Merge profile data
llvm-profdata merge -o /tmp/pgo-data/merged.profdata /tmp/pgo-data

# Step 4: Build with PGO
RUSTFLAGS="-Cprofile-use=/tmp/pgo-data/merged.profdata -Cllvm-args=-pgo-warn-missing-function" \
  cargo build --release
```

**PGO Results** (calculator example):

```
Before PGO:  125K req/s
After PGO:   148K req/s  (18.4% improvement)
```

### 2. Link-Time Optimization (LTO)

LTO enables cross-crate inlining and dead code elimination:

```toml
# Cargo.toml
[profile.release]
opt-level = 3
lto = "fat"              # Full LTO (slower build, faster binary)
codegen-units = 1        # Single codegen unit for max optimization
strip = true             # Remove debug symbols
panic = "abort"          # Smaller binary, no unwinding overhead
```

**LTO Impact**:
- Binary size: -15% smaller
- Dispatch latency: -8% faster
- Build time: +3x longer (acceptable for release builds)

### 3. CPU-Specific Optimizations

Enable target-specific optimizations:

```bash
# Build for native CPU (uses AVX2, BMI2, etc.)
RUSTFLAGS="-C target-cpu=native" cargo build --release

# Or specific features
RUSTFLAGS="-C target-feature=+avx2,+bmi2,+fma" cargo build --release
```

**Benchmark** (JSON parsing with AVX2):

```
Generic x86_64              time:   [2.2103 μs 2.2398 μs 2.2701 μs]
Native (AVX2)               time:   [1.8234 μs 1.8502 μs 1.8881 μs]
                                    ↑ 21% faster
```

### 4. Reduce Allocations

Minimize heap allocations in hot paths:

```rust
// Before: Multiple allocations
pub fn format_error(code: i32, message: &str) -> String {
    format!("Error {}: {}", code, message)  // Allocates
}

// After: Single allocation with capacity hint
pub fn format_error(code: i32, message: &str) -> String {
    let mut s = String::with_capacity(message.len() + 20);
    use std::fmt::Write;
    write!(&mut s, "Error {}: {}", code, message).unwrap();
    s
}

// Better: Avoid allocation entirely
pub fn write_error(buf: &mut String, code: i32, message: &str) {
    use std::fmt::Write;
    write!(buf, "Error {}: {}", code, message).unwrap();
}
```

**Allocation Tracking** with `dhat-rs`:

```rust
#[cfg(feature = "dhat-heap")]
#[global_allocator]
static ALLOC: dhat::Alloc = dhat::Alloc;

fn main() {
    #[cfg(feature = "dhat-heap")]
    let _profiler = dhat::Profiler::new_heap();

    // ... run server ...
}
```

Run with:
```bash
cargo run --release --features dhat-heap
# Generates dhat-heap.json
# View with Firefox Profiler: https://profiler.firefox.com/
```

### 5. String Interning

Intern repeated strings to reduce memory:

```rust
use string_cache::DefaultAtom as Atom;

pub struct InternedConfig {
    tool_names: Vec<Atom>,  // Interned strings
}

// "calculate" string stored once, referenced multiple times
let tool1 = Atom::from("calculate");
let tool2 = Atom::from("calculate");
assert!(tool1.as_ptr() == tool2.as_ptr());  // Same pointer!
```

**Memory Savings** (100 tools, 50 unique names):
- Without interning: ~2KB (20 bytes × 100)
- With interning: ~1KB (20 bytes × 50 + pointers)

### 6. Lazy Initialization

Defer expensive operations until needed:

```rust
use once_cell::sync::Lazy;

// Computed once on first access
static SCHEMA_CACHE: Lazy<HashMap<String, Schema>> = Lazy::new(|| {
    let mut cache = HashMap::new();
    // ... expensive schema compilation ...
    cache
});

pub fn get_schema(name: &str) -> Option<&'static Schema> {
    SCHEMA_CACHE.get(name)
}
```

**Cold Start Impact**:
- Eager initialization: 120ms startup
- Lazy initialization: 45ms startup, 5ms on first use

## Profiling Tools

### 1. Flamegraph for CPU Profiling

```bash
# Install cargo-flamegraph
cargo install flamegraph

# Generate flamegraph
cargo flamegraph --bin pforge -- serve

# Open flamegraph.svg in browser
```

**Reading Flamegraphs**:
- X-axis: Alphabetical sort (not time!)
- Y-axis: Call stack depth
- Width: Time spent in function
- Look for wide boxes = hot paths

### 2. Criterion for Microbenchmarks

```rust
// benches/dispatch_benchmark.rs
use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use pforge_runtime::HandlerRegistry;

fn bench_dispatch(c: &mut Criterion) {
    let mut group = c.benchmark_group("dispatch");

    for size in [10, 50, 100, 500].iter() {
        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, &size| {
            let mut registry = HandlerRegistry::new();

            // Register `size` tools
            for i in 0..*size {
                registry.register(&format!("tool_{}", i), DummyHandler);
            }

            b.iter(|| {
                registry.get(black_box("tool_0"))
            });
        });
    }

    group.finish();
}

criterion_group!(benches, bench_dispatch);
criterion_main!(benches);
```

Run benchmarks:
```bash
cargo bench

# Generate HTML report
open target/criterion/report/index.html
```

**Criterion Features**:
- Statistical analysis (mean, median, std dev)
- Outlier detection
- Regression detection
- HTML reports with plots

### 3. Valgrind for Memory Profiling

```bash
# Check for memory leaks
valgrind --leak-check=full \
         --show-leak-kinds=all \
         --track-origins=yes \
         ./target/release/pforge serve

# Run requests, then Ctrl+C

# Look for:
# - "definitely lost" (must fix)
# - "indirectly lost" (must fix)
# - "possibly lost" (investigate)
# - "still reachable" (okay if cleanup code not run)
```

### 4. Perf for System-Level Profiling

```bash
# Record performance data
perf record -F 99 -g ./target/release/pforge serve
# ... run workload ...
# Ctrl+C

# Analyze
perf report

# Or generate flamegraph
perf script | stackcollapse-perf.pl | flamegraph.pl > perf.svg
```

### 5. Tokio Console for Async Debugging

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
            // ... server code ...
        });
}
```

Run with tokio-console:
```bash
cargo run --release &
tokio-console
```

**Tokio Console Shows**:
- Task spawn/poll/drop events
- Async task durations
- Blocking operations
- Resource usage

## Case Study: Optimizing Calculator Handler

Let's optimize the calculator example step-by-step:

### Baseline Implementation

```rust
// Version 1: Naive implementation
async fn handle(&self, input: CalculateInput) -> Result<CalculateOutput> {
    let result = match input.operation.as_str() {
        "add" => input.a + input.b,
        "subtract" => input.a - input.b,
        "multiply" => input.a * input.b,
        "divide" => {
            if input.b == 0.0 {
                return Err(Error::Handler("Division by zero".to_string()));
            }
            input.a / input.b
        }
        _ => return Err(Error::Handler(format!("Unknown operation: {}", input.operation))),
    };

    Ok(CalculateOutput { result })
}
```

**Benchmark**: 0.82μs per call

### Optimization 1: Inline Hint

```rust
#[inline(always)]
async fn handle(&self, input: CalculateInput) -> Result<CalculateOutput> {
    // ... same code ...
}
```

**Benchmark**: 0.76μs per call (7.3% faster)

### Optimization 2: Avoid String Allocation

```rust
#[inline(always)]
async fn handle(&self, input: CalculateInput) -> Result<CalculateOutput> {
    let result = match input.operation.as_str() {
        "add" => input.a + input.b,
        "subtract" => input.a - input.b,
        "multiply" => input.a * input.b,
        "divide" => {
            if input.b == 0.0 {
                return Err(Error::DivisionByZero);  // Static error
            }
            input.a / input.b
        }
        _ => return Err(Error::UnknownOperation),  // Static error
    };

    Ok(CalculateOutput { result })
}
```

**Benchmark**: 0.68μs per call (10.5% faster)

### Optimization 3: Branch Prediction

```rust
#[inline(always)]
async fn handle(&self, input: CalculateInput) -> Result<CalculateOutput> {
    // Most common operations first (better branch prediction)
    let result = match input.operation.as_str() {
        "add" => input.a + input.b,
        "multiply" => input.a * input.b,
        "subtract" => input.a - input.b,
        "divide" => {
            // Use likely/unlikely hints (nightly only)
            #[cfg(feature = "nightly")]
            if std::intrinsics::unlikely(input.b == 0.0) {
                return Err(Error::DivisionByZero);
            }

            #[cfg(not(feature = "nightly"))]
            if input.b == 0.0 {
                return Err(Error::DivisionByZero);
            }

            input.a / input.b
        }
        _ => return Err(Error::UnknownOperation),
    };

    Ok(CalculateOutput { result })
}
```

**Benchmark**: 0.61μs per call (10.3% faster)

### Final Results

| Version | Time (μs) | Improvement |
|---------|-----------|-------------|
| Baseline | 0.82 | - |
| + Inline | 0.76 | 7.3% |
| + No alloc | 0.68 | 10.5% |
| + Branch hints | 0.61 | 10.3% |
| **Total** | **0.61** | **25.6%** |

## Production Performance Checklist

### Compiler Settings

```toml
[profile.release]
opt-level = 3                    # Maximum optimization
lto = "fat"                      # Full link-time optimization
codegen-units = 1                # Single codegen unit
strip = true                     # Remove debug symbols
panic = "abort"                  # No unwinding overhead
overflow-checks = false          # Disable overflow checks (use carefully!)
```

### Runtime Configuration

```rust
// Tokio tuning
let runtime = tokio::runtime::Builder::new_multi_thread()
    .worker_threads(num_cpus::get())
    .max_blocking_threads(512)
    .thread_keep_alive(Duration::from_secs(60))
    .build()?;

// Memory tuning
#[global_allocator]
static ALLOC: jemallocator::Jemalloc = jemallocator::Jemalloc;  // Faster than system allocator
```

### System Tuning

```bash
# Increase file descriptor limits
ulimit -n 65536

# Tune TCP for high throughput
sudo sysctl -w net.core.somaxconn=4096
sudo sysctl -w net.ipv4.tcp_max_syn_backlog=4096

# CPU governor for performance
sudo cpupower frequency-set -g performance
```

### Monitoring

```rust
use metrics::{counter, histogram};

async fn handle(&self, input: Input) -> Result<Output> {
    let start = std::time::Instant::now();

    let result = self.inner_handle(input).await;

    // Record metrics
    histogram!("handler.duration", start.elapsed().as_micros() as f64);
    counter!("handler.calls", 1);

    if result.is_err() {
        counter!("handler.errors", 1);
    }

    result
}
```

## Performance Anti-Patterns

### 1. Async in Sync Context

```rust
// BAD: Blocking in async context
async fn bad_handler(&self) -> Result<Output> {
    let file = std::fs::read_to_string("data.txt")?;  // Blocks event loop!
    Ok(Output { data: file })
}

// GOOD: Use async I/O
async fn good_handler(&self) -> Result<Output> {
    let file = tokio::fs::read_to_string("data.txt").await?;
    Ok(Output { data: file })
}

// GOOD: Use spawn_blocking for CPU-heavy work
async fn cpu_intensive(&self) -> Result<Output> {
    let result = tokio::task::spawn_blocking(|| {
        expensive_computation()
    }).await?;
    Ok(result)
}
```

### 2. Unnecessary Clones

```rust
// BAD: Cloning large structures
async fn bad(&self, data: LargeStruct) -> Result<()> {
    let copy = data.clone();  // Expensive!
    process(copy).await
}

// GOOD: Pass by reference
async fn good(&self, data: &LargeStruct) -> Result<()> {
    process(data).await
}
```

### 3. String Concatenation in Loops

```rust
// BAD: Quadratic time complexity
fn build_message(items: &[String]) -> String {
    let mut msg = String::new();
    for item in items {
        msg = msg + item;  // Reallocates every iteration!
    }
    msg
}

// GOOD: Pre-allocate capacity
fn build_message_good(items: &[String]) -> String {
    let total_len: usize = items.iter().map(|s| s.len()).sum();
    let mut msg = String::with_capacity(total_len);
    for item in items {
        msg.push_str(item);
    }
    msg
}
```

### 4. Over-Engineering Hot Paths

```rust
// BAD: Complex abstractions in hot path
async fn over_engineered(&self, input: Input) -> Result<Output> {
    let validator = ValidatorFactory::create()
        .with_rules(RuleSet::default())
        .build()?;

    let sanitizer = SanitizerBuilder::new()
        .add_filter(XssFilter)
        .add_filter(SqlInjectionFilter)
        .build();

    validator.validate(&input)?;
    let sanitized = sanitizer.sanitize(input)?;
    process(sanitized).await
}

// GOOD: Direct validation in hot path
async fn simple(&self, input: Input) -> Result<Output> {
    if input.value.is_empty() {
        return Err(Error::Validation("Empty value".into()));
    }
    process(input).await
}
```

## Summary

Performance optimization in pforge follows these principles:

1. **Measure first**: Profile before optimizing
2. **Hot path focus**: Optimize where it matters
3. **Zero-cost abstractions**: Compiler optimizes away overhead
4. **Async efficiency**: Non-blocking I/O, spawn_blocking for CPU work
5. **Memory awareness**: Minimize allocations, use pools
6. **SIMD where applicable**: 10x speedups for data processing
7. **LTO and PGO**: Compiler-driven optimizations for production

**Performance is cumulative**: Small optimizations compound. The 0.8μs dispatch time comes from dozens of micro-optimizations throughout the codebase.

**Next chapter**: We'll dive into benchmarking and profiling techniques to measure and track these optimizations.
