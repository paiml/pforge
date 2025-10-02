# Testing Strategies

Testing is a core pillar of pforge's quality philosophy. With **115 comprehensive tests** across multiple layers and strategies, pforge ensures production-ready reliability through a rigorous, multi-faceted testing approach that combines traditional and advanced testing methodologies.

## The pforge Testing Philosophy

pforge's testing strategy is built on three foundational principles:

1. **Extreme TDD**: 5-minute cycles (RED → GREEN → REFACTOR) with quality gates at every step
2. **Defense in Depth**: Multiple layers of testing catch different classes of bugs
3. **Quality as Code**: Tests are first-class citizens, with coverage targets and mutation testing enforcement

This chapter provides a comprehensive guide to pforge's testing pyramid and how each layer contributes to overall system quality.

## The Testing Pyramid

pforge implements a balanced testing pyramid that ensures comprehensive coverage without sacrificing speed or maintainability:

```
           /\
          /  \          Property-Based Tests (12 tests, 10K cases each)
         /____\         ├─ Config serialization properties
        /      \        ├─ Handler dispatch invariants
       /        \       └─ Validation consistency
      /__________\
     /            \     Integration Tests (26 tests)
    /              \    ├─ Multi-crate workflows
   /                \   ├─ Middleware chains
  /____Unit Tests____\  └─ End-to-end scenarios
 /                    \
/______________________\ Unit Tests (74 tests, <1ms each)
                        ├─ Config parsing
                        ├─ Handler registry
                        ├─ Code generation
                        └─ Type validation
```

### Test Distribution

- **74 Unit Tests**: Fast, focused tests of individual components (<1ms each)
- **26 Integration Tests**: Cross-crate and system-level tests (<100ms each)
- **12 Property-Based Tests**: Automated edge-case discovery (10,000 iterations each)
- **5 Doctests**: Executable documentation examples
- **8 Quality Gate Tests**: PMAT integration and enforcement

**Total: 115 tests** ensuring comprehensive coverage at every level.

## Performance Targets

pforge enforces strict performance requirements for tests to maintain rapid feedback cycles:

| Test Type | Target | Actual | Enforcement |
|-----------|--------|--------|-------------|
| Unit tests | <1ms | <1ms | CI enforced |
| Integration tests | <100ms | 15-50ms | CI enforced |
| Property tests | <5s per property | 2-4s | Local only |
| Full test suite | <30s | ~15s | CI enforced |
| Coverage generation | <2min | ~90s | Makefile target |

Fast tests enable the 5-minute TDD cycle that drives pforge development.

## Quality Metrics

pforge enforces industry-leading quality standards through automated gates:

### Coverage Requirements

- **Line Coverage**: ≥80% (currently ~85%)
- **Branch Coverage**: ≥75% (currently ~78%)
- **Mutation Kill Rate**: ≥90% target with cargo-mutants

### Complexity Limits

- **Cyclomatic Complexity**: ≤20 per function
- **Cognitive Complexity**: ≤15 per function
- **Technical Debt Grade (TDG)**: ≥0.75

### Zero Tolerance

- **No unwrap()**: Production code must handle all errors explicitly
- **No panic!()**: All panics confined to test code only
- **No SATD**: Self-Admitted Technical Debt comments blocked in PRs

## Test Organization

pforge tests are organized by scope and purpose:

```
pforge/
├── crates/*/src/**/*.rs          # Unit tests (inline #[cfg(test)] modules)
├── crates/*/tests/*.rs            # Crate-level integration tests
├── crates/pforge-integration-tests/
│   ├── integration_test.rs        # Cross-crate integration
│   └── property_test.rs           # Property-based tests
└── crates/pforge-cli/tests/
    └── scaffold_tests.rs          # CLI integration tests
```

### Test Module Structure

Each source file includes inline unit tests:

```rust
// In crates/pforge-runtime/src/registry.rs

pub struct HandlerRegistry {
    // Implementation...
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_registry_lookup() {
        // Fast, focused test (<1ms)
    }

    #[tokio::test]
    async fn test_async_dispatch() {
        // Async test with tokio runtime
    }
}
```

## Running Tests

### Quick Test Commands

```bash
# Run all tests (unit + integration + doctests)
make test

# Run only unit tests (fastest feedback)
cargo test --lib

# Run specific test
cargo test test_name

# Run tests in specific crate
cargo test -p pforge-runtime

# Run with verbose output
cargo test -- --nocapture
```

### Continuous Testing

pforge provides a watch mode for extreme TDD:

```bash
# Watch mode: auto-run tests on file changes
make watch

# Manual watch with cargo-watch
cargo watch -x 'test --lib --quiet' -x 'clippy --quiet'
```

Tests re-run automatically on file save, providing <1s feedback for unit tests.

### Coverage Analysis

```bash
# Generate comprehensive coverage report
make coverage

# View summary
make coverage-summary

# Open HTML report in browser
make coverage-open
```

Coverage generation uses `cargo-llvm-cov` with `cargo-nextest` for accurate, fast results.

## Quality Gates

Every commit must pass the quality gate:

```bash
# Run full quality gate (CI equivalent)
make quality-gate
```

This runs:
1. `cargo fmt --check` - Code formatting
2. `cargo clippy -- -D warnings` - Linting with zero warnings
3. `cargo test --all` - All tests
4. `cargo llvm-cov` - Coverage check (≥80%)
5. `pmat analyze complexity --max 20` - Complexity enforcement
6. `pmat analyze satd` - Technical debt detection
7. `pmat tdg` - Technical Debt Grade (≥0.75)

**Development is blocked** if any gate fails (Jidoka/"stop the line" principle).

## Pre-Commit Hooks

pforge uses git hooks to enforce quality before commits:

```bash
# Located at: .git/hooks/pre-commit
#!/bin/bash
set -e

echo "Running pre-commit quality gates..."

# Format check
cargo fmt --check || (echo "Run 'cargo fmt' first" && exit 1)

# Clippy
cargo clippy --all-targets -- -D warnings

# Tests
cargo test --all

# PMAT checks
pmat quality-gate run

echo "✅ All quality gates passed!"
```

Commits are **rejected** if any check fails, ensuring the main branch always passes CI.

## Continuous Integration

GitHub Actions runs comprehensive tests on every PR:

```yaml
# .github/workflows/quality.yml
jobs:
  quality:
    runs-on: ubuntu-latest
    steps:
      - name: Run quality gate
        run: make quality-gate

      - name: Mutation testing
        run: cargo mutants --check

      - name: Upload coverage
        uses: codecov/codecov-action@v3
```

CI enforces:
- All tests pass on multiple platforms (Linux, macOS, Windows)
- Coverage ≥80%
- Zero clippy warnings
- PMAT quality gates pass
- Mutation testing achieves ≥90% kill rate

## Test-Driven Development

pforge uses **Extreme TDD** with strict 5-minute cycles:

### The 5-Minute Cycle

1. **RED (2 min)**: Write a failing test
2. **GREEN (2 min)**: Write minimum code to pass
3. **REFACTOR (1 min)**: Clean up, run quality gates
4. **COMMIT**: If gates pass
5. **RESET**: If cycle exceeds 5 minutes, start over

### Example TDD Session

```rust
// RED: Write failing test (2 min)
#[test]
fn test_config_validation_rejects_duplicates() {
    let config = create_config_with_duplicate_tools();
    let result = validate_config(&config);
    assert!(result.is_err());  // FAILS: validation not implemented
}

// GREEN: Implement minimal solution (2 min)
pub fn validate_config(config: &ForgeConfig) -> Result<()> {
    let mut seen = HashSet::new();
    for tool in &config.tools {
        if !seen.insert(tool.name()) {
            return Err(ConfigError::DuplicateToolName(tool.name()));
        }
    }
    Ok(())
}

// REFACTOR: Clean up (1 min)
// - Add documentation
// - Run clippy
// - Check complexity
// - Commit if all gates pass
```

### Benefits of Extreme TDD

- **Rapid Feedback**: <1s for unit tests
- **Quality Built In**: Tests written first ensure comprehensive coverage
- **Prevention Over Detection**: Bugs caught at creation time
- **Living Documentation**: Tests document expected behavior

## Testing Best Practices

### Unit Test Guidelines

1. **Fast**: Each test must complete in <1ms
2. **Focused**: Test one behavior per test
3. **Isolated**: No shared state between tests
4. **Deterministic**: Same input always produces same result
5. **Clear**: Test name describes what's being tested

```rust
#[test]
fn test_handler_registry_returns_error_for_unknown_tool() {
    let registry = HandlerRegistry::new();
    let result = registry.get("nonexistent");

    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), Error::ToolNotFound(_)));
}
```

### Integration Test Guidelines

1. **Realistic**: Test real workflows
2. **Efficient**: Target <100ms per test
3. **Comprehensive**: Cover all integration points
4. **Independent**: Each test can run in isolation

```rust
#[tokio::test]
async fn test_middleware_chain_with_recovery() {
    let mut chain = MiddlewareChain::new();
    chain.add(Arc::new(ValidationMiddleware::new(vec!["input".to_string()])));
    chain.add(Arc::new(RecoveryMiddleware::new()));

    let result = chain.execute(json!({"input": 42}), handler).await;
    assert!(result.is_ok());
}
```

### Property Test Guidelines

1. **Universal**: Test properties that hold for all inputs
2. **Diverse**: Generate wide range of test cases
3. **Persistent**: Save failing cases for regression prevention
4. **Exhaustive**: Run thousands of iterations (10K default)

```rust
proptest! {
    #[test]
    fn config_serialization_roundtrip(config in arb_forge_config()) {
        let yaml = serde_yml::to_string(&config)?;
        let parsed: ForgeConfig = serde_yml::from_str(&yaml)?;
        prop_assert_eq!(config.forge.name, parsed.forge.name);
    }
}
```

## Common Testing Patterns

### Testing Error Paths

All error paths must be tested:

```rust
#[test]
fn test_handler_timeout_returns_timeout_error() {
    let handler = create_slow_handler();
    let result = execute_with_timeout(handler, Duration::from_millis(10));

    assert!(matches!(result.unwrap_err(), Error::Timeout(_)));
}
```

### Testing Async Code

Use `#[tokio::test]` for async tests:

```rust
#[tokio::test]
async fn test_concurrent_handler_dispatch() {
    let registry = create_registry();

    let handles: Vec<_> = (0..100)
        .map(|i| tokio::spawn(registry.dispatch("tool", &params(i))))
        .collect();

    for handle in handles {
        assert!(handle.await.unwrap().is_ok());
    }
}
```

### Testing State Management

Isolate state between tests:

```rust
#[tokio::test]
async fn test_state_persistence() {
    let state = MemoryStateManager::new();

    state.set("key", b"value".to_vec(), None).await?;
    assert_eq!(state.get("key").await?, Some(b"value".to_vec()));

    state.delete("key").await?;
    assert_eq!(state.get("key").await?, None);
}
```

## Debugging Failed Tests

### Verbose Output

```bash
# Show println! output
cargo test -- --nocapture

# Show test names as they run
cargo test -- --nocapture --test-threads=1
```

### Running Single Tests

```bash
# Run specific test
cargo test test_config_validation

# Run with backtrace
RUST_BACKTRACE=1 cargo test test_config_validation

# Run with full backtrace
RUST_BACKTRACE=full cargo test test_config_validation
```

### Test Filtering

```bash
# Run all tests matching pattern
cargo test config

# Run tests in specific module
cargo test registry::tests

# Run ignored tests
cargo test -- --ignored
```

## Summary

pforge's testing strategy ensures production-ready quality through:

1. **115 comprehensive tests** across all layers
2. **Multiple testing strategies**: unit, integration, property-based, mutation
3. **Strict quality gates**: coverage, complexity, TDD enforcement
4. **Fast feedback loops**: <1ms unit tests, <15s full suite
5. **Continuous quality**: pre-commit hooks, CI/CD pipeline

The following chapters provide detailed guides for each testing layer:

- **Chapter 9.1**: Unit Testing - Fast, focused component tests
- **Chapter 9.2**: Integration Testing - Cross-crate and system tests
- **Chapter 9.3**: Property-Based Testing - Automated edge case discovery
- **Chapter 9.4**: Mutation Testing - Validating test effectiveness

Together, these strategies ensure pforge maintains the highest quality standards while enabling rapid, confident development.
