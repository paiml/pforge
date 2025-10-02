# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

pforge is a zero-boilerplate framework for building Model Context Protocol (MCP) servers through declarative YAML configuration. It's built on pmcp (Pragmatic AI Labs MCP SDK) with strict PMAT quality enforcement.

**Design Philosophy**: Cargo Lambda simplicity × Flask ergonomics × Rust guarantees

## Development Commands

### TDD Workflow (Extreme TDD - 5-minute cycles)
```bash
# Continuous testing during development
cargo watch -x 'test --lib --quiet' -x 'clippy --quiet'

# Run all tests
cargo test --all
cargo test --all --release

# Fast development cycle
make dev  # Runs continuous test + clippy watch
```

### Quality Gates (Required before commit)
```bash
# Full quality gate check (run before committing)
make quality-gate

# Individual quality checks
cargo fmt --check              # Code formatting
cargo clippy -- -D warnings    # Linting
cargo test --all              # All tests
cargo tarpaulin --out Json    # Code coverage (min 80%)
pmat analyze complexity --max 20   # Cyclomatic complexity
pmat analyze satd --max 0          # Technical debt comments
pmat analyze tdg --min 0.75        # Technical Debt Grade
```

### Testing
```bash
# Unit tests (< 1ms each)
cargo test --test unit

# Integration tests (< 100ms each)
cargo test --test integration

# Property-based tests
cargo test --test property --release -- --test-threads=1

# Mutation tests (target: 90%+ mutation kill rate)
cargo mutants --check
```

### Benchmarking
```bash
# Run all benchmarks
cargo bench

# Specific benchmark suites
cargo bench --bench dispatch_benchmark
cargo bench --bench throughput_benchmark

# Performance regression check
make bench-check
```

### Building and Running
```bash
# Development build
pforge build --debug

# Release build
pforge build --release

# Run server
pforge serve

# Development mode with hot reload
pforge dev --watch
```

### Profiling and Debugging
```bash
# Verbose logging
RUST_LOG=pforge=trace pforge serve

# Flamegraph profiling
cargo flamegraph --bin pforge -- serve

# Memory profiling (valgrind)
valgrind --leak-check=full --show-leak-kinds=all target/release/pforge serve

# Heap profiling
cargo run --release --features dhat-heap
```

## Architecture

### High-Level Component Stack
```
pforge CLI (Scaffold, Build, Dev, Test, Quality)
    ↓
pforge-codegen (YAML → Rust AST → Optimized Runtime)
    ↓
pforge-runtime (Handler Registry, Type-safe validation, Middleware, State)
    ↓
pmcp v1.6+ (TypedTool, Multi-transport, SIMD JSON parsing - 16x faster than TS SDK)
    ↓
MCP Protocol v2024-10-07 (JSON-RPC 2.0)
```

### Workspace Structure
```
crates/
├── pforge-cli/        # CLI binary and commands (new, build, serve, dev, quality)
├── pforge-runtime/    # Core runtime (handler registry, transport, state, middleware)
├── pforge-codegen/    # Code generation (YAML → Rust AST → optimized runtime)
├── pforge-config/     # Configuration parsing and validation
├── pforge-macro/      # Procedural macros
└── pforge-quality/    # Quality enforcement and PMAT integration

bridges/               # Language bridges (Python, Go, Node.js)
examples/             # Example MCP servers
benches/              # Performance benchmarks
docs/                 # Documentation
```

### Core Abstractions

**Handler Trait** (pforge-runtime/src/handler.rs):
- Zero-cost abstraction compatible with pmcp TypedTool
- Type-safe Input/Output with JsonSchema generation
- Async by default via async_trait

**HandlerRegistry** (pforge-runtime/src/registry.rs):
- O(1) average-case lookup using FxHash (2x faster than SipHash for small keys)
- Compile-time optimized dispatch
- Future optimization: Perfect hashing (FKS algorithm) for O(1) worst-case

**Configuration AST** (pforge-config/src/types.rs):
- ForgeConfig: Root configuration structure
- ToolDef: Enum supporting Native, CLI, HTTP, and Pipeline handlers
- ParamSchema: Type-safe parameter definitions with validation

### Handler Types
1. **Native**: Rust handlers compiled into binary (fastest, < 1μs dispatch)
2. **CLI**: CLI wrapper with streaming support
3. **HTTP**: HTTP endpoint wrapper with auth support
4. **Pipeline**: Tool composition pipeline with conditional execution

## Quality Standards (PMAT Enforcement)

### Zero Tolerance Rules
- **NO** `unwrap()` calls in production code (only in tests)
- **NO** `panic!()` in production code
- **NO** SATD (Self-Admitted Technical Debt) comments
- **NO** functions with cyclomatic complexity > 20

### Required Metrics
- Test coverage: ≥ 80% line coverage
- Technical Debt Grade (TDG): ≥ 0.75
- Mutation kill rate: ≥ 90%
- All error paths must have tests

### TDD Methodology (Toyota Way + PMAT)
**Strict 5-minute cycle enforcement**:
1. RED (2 min max): Write failing test
2. GREEN (2 min max): Minimum code to pass
3. REFACTOR (1 min): Clean up, run quality gates
4. COMMIT: If quality gates pass
5. RESET: If cycle exceeds 5 minutes

Quality gate failures halt development (Jidoka/"stop the line" principle).

### PMAT Quality Gate Integration (PFORGE-3001)

Automated quality enforcement via pre-commit hooks and Makefile targets:

```bash
# Run all quality gates
make quality-gate

# Individual PMAT checks
pmat analyze complexity --max-cyclomatic 20
pmat analyze satd
pmat tdg .
pmat analyze dead-code
```

**Pre-Commit Hook** (`.git/hooks/pre-commit`):
- Automatically runs on every commit
- Enforces: formatting, linting, tests, complexity, SATD, coverage, TDG
- Blocks commits that fail quality standards
- Bypass (NOT recommended): `git commit --no-verify`

**Quality Gate Test Suite** (`integration_test.rs`):
- 8 tests verify PMAT integration
- Tests complexity enforcement, SATD detection, TDG calculation
- Validates pre-commit hook existence and executability
- Ensures quality-gates.yaml configuration is valid

**Metrics Tracked**:
- Cyclomatic Complexity: max 20 (current: 9)
- SATD Comments: Phase markers allowed
- TDG Score: min 75/100 (current: 96/100)
- Dead Code: monitored (current: 0%)
- Test Coverage: min 80% (current: 80.54%)

## Performance Targets

| Metric | Target |
|--------|--------|
| Cold start | < 100ms |
| Tool dispatch (hot) | < 1μs |
| Config parse | < 10ms |
| Schema generation | < 1ms |
| Memory baseline | < 512KB |
| Memory per tool | < 256B |
| Throughput (sequential) | > 100K req/s |
| Throughput (concurrent 8-core) | > 500K req/s |

## Error Handling

All errors use thiserror with specific variants:
- `ToolNotFound`: Requested tool doesn't exist
- `InvalidConfig`: Configuration validation failed
- `Validation`: Parameter validation failed
- `Handler`: Handler execution error
- `Timeout`: Operation exceeded timeout
- `BridgeError`: Language bridge error

Never use `unwrap()` or `expect()` in production - always propagate errors or handle explicitly.

## Testing Strategy

### Test Organization
```
tests/
├── unit/          # Fast tests (< 1ms each) - config, registry, codegen
├── integration/   # Integration tests (< 100ms) - CLI, server, PMAT
├── property/      # Property-based tests using proptest
└── e2e/          # End-to-end tests - stdio, SSE, WebSocket transports
```

### Property-Based Testing
Use `proptest` for:
- Config roundtrip serialization
- Handler dispatch always returns valid JSON
- Parameter validation edge cases

### Benchmarking with Criterion
All benchmarks in `benches/` directory:
- `dispatch_benchmark.rs`: Handler dispatch latency
- `throughput_benchmark.rs`: Sustained throughput testing
- `memory_benchmark.rs`: Memory usage profiling

## Pre-Commit Workflow

Pre-commit hooks automatically run:
0. `scripts/validate_markdown_links.sh` - Validate all markdown links
1. `cargo fmt --check` - Code formatting
2. `cargo clippy -- -D warnings` - Linting
3. `cargo test --all` - All tests
4. `pmat analyze complexity` - Complexity check (≤20)
5. `pmat analyze satd` - No technical debt comments
6. `cargo tarpaulin` - Coverage check (≥80%)
7. `pmat analyze tdg` - Technical Debt Grade (≥0.75)

Commits are **blocked** if any gate fails. This includes broken documentation links.

## CI/CD Pipeline

GitHub Actions workflows (`.github/workflows/`):
- `ci.yml`: Continuous integration
- `quality.yml`: Quality gates + mutation tests + coverage upload
- `release.yml`: Release automation

Performance regression fails if dispatch latency > 2μs.

## Language Bridges

Bridge architecture for polyglot handlers (Python, Go, Node.js):
- Thin FFI layer with stable C ABI
- Zero-copy parameter passing (pointers, not serialization)
- Error semantics preserved across language boundaries
- Type safety leveraged in target language

Bridge implementations in `bridges/` directory.

## Configuration Schema

YAML configuration defines MCP servers declaratively:
```yaml
forge:
  name: server-name
  version: 0.1.0
  transport: stdio|sse|websocket
  optimization: debug|release

tools:
  - type: native|cli|http|pipeline
    name: tool_name
    description: "Tool description"
    handler: { ... }
    params: { ... }
    timeout_ms: 30000  # optional

resources: [ ... ]  # optional
prompts: [ ... ]    # optional
state: { ... }      # optional
```

## Development Philosophy

**Extreme TDD + Lean Manufacturing Principles**:
- Rapid feedback cycles (5-minute max)
- Quality built-in (Jidoka)
- Stop-the-line on quality failures
- Continuous improvement (Kaizen)
- Eliminate waste
- Amplify learning

**Theoretical Foundation**: Combines Beck's TDD with Toyota Production System (Poppendieck & Poppendieck, 2003: *Lean Software Development*).

## Release Checklist

1. `make test` - All tests pass
2. `make quality-gate` - Full quality gate
3. `make mutants` - Mutation tests
4. `make bench-check` - Benchmark regression check
5. Update `Cargo.toml` version
6. Update `CHANGELOG.md`
7. `git tag -a v0.1.0 -m "Release v0.1.0"`
8. `git push --tags`
9. Publish crates in order: runtime → codegen → cli

## Documentation

Primary specification: `docs/specifications/pforge-specification.md` (comprehensive 2400+ line spec)

Examples in `examples/`:
- `hello-world/`: Minimal viable server
- `pmat-server/`: PMAT code analysis integration
- `polyglot-server/`: Multi-language bridge example
