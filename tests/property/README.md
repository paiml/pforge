# Property-Based Testing

Property-based testing using [proptest](https://github.com/proptest-rs/proptest) for comprehensive validation of pforge invariants.

## Overview

Property-based tests generate thousands of random test cases to verify that certain properties (invariants) hold true across a wide range of inputs. This catches edge cases that would be difficult to identify with traditional example-based testing.

## Test Organization

### `config_properties.rs`
Tests for configuration parsing and serialization:
- **Roundtrip property**: `YAML → Config → YAML` produces valid config
- **Serialization property**: All valid configs can be serialized/deserialized
- **Uniqueness property**: Tool name uniqueness is preserved
- **Validation property**: Invalid configs are rejected consistently

### `handler_properties.rs`
Tests for handler dispatch and execution:
- **JSON property**: Handler dispatch always returns valid JSON
- **Error mapping**: Handler errors correctly convert to `Error` types
- **Consistency property**: Same input → same handler lookup
- **Schema property**: Schema generation is deterministic

### `validation_properties.rs`
Tests for parameter validation:
- **Required fields**: Always validated, never skipped
- **Type coercion**: Consistent across all types
- **Panic freedom**: Invalid JSON never panics
- **Recoverability**: Validation errors are recoverable

## Running Property Tests

```bash
# Run property tests (10,000 cases per property by default)
make test-property

# Run with custom case count
PROPTEST_CASES=100000 cargo test --test property --release

# Run specific property test
cargo test --test property config_roundtrip

# Run with reproducible seed for debugging
PROPTEST_SEED=1234567890 cargo test --test property
```

## Configuration

Property tests use `proptest-regressions/` to persist failing cases for reproducibility. These files are checked into git to ensure failures are tracked.

```toml
# Proptest configuration in Cargo.toml
[dev-dependencies]
proptest = "1.0"

# Per-test configuration
proptest! {
    #![proptest_config(ProptestConfig {
        cases: 10000,
        max_shrink_iters: 10000,
        ..ProptestConfig::default()
    })]
}
```

## Writing New Properties

### Example: Config Roundtrip

```rust
use proptest::prelude::*;
use pforge_config::{parse_config_from_str, ForgeConfig};

proptest! {
    #[test]
    fn config_roundtrip(config in arb_forge_config()) {
        // Serialize to YAML
        let yaml = serde_yml::to_string(&config)?;

        // Parse back
        let parsed = parse_config_from_str(&yaml)?;

        // Should be equivalent
        prop_assert_eq!(config.forge.name, parsed.forge.name);
        prop_assert_eq!(config.tools.len(), parsed.tools.len());
    }
}

// Arbitrary config generator
fn arb_forge_config() -> impl Strategy<Value = ForgeConfig> {
    (arb_forge_metadata(), prop::collection::vec(arb_tool_def(), 1..10))
        .prop_map(|(forge, tools)| ForgeConfig {
            forge,
            tools,
            resources: vec![],
            prompts: vec![],
            state: None,
        })
}
```

## Benefits

- **Comprehensive**: 10,000+ test cases vs ~100 manual tests
- **Edge cases**: Automatically finds corner cases (empty strings, large numbers, special chars)
- **Regression prevention**: Failing cases are persisted
- **Documentation**: Properties describe system invariants clearly

## Phase 3 Goals

- ✅ 12+ property-based tests
- ✅ 10,000+ cases per property
- ✅ CI/CD integration
- ✅ Failure persistence
- ✅ 100% property pass rate

## References

- [Proptest Book](https://proptest-rs.github.io/proptest/intro.html)
- [Property-Based Testing in Rust](https://www.lpalmieri.com/posts/property-based-testing-in-rust/)
- [Phase 3 Roadmap](../../ROADMAP.md#phase-3-quality--testing-week-5-6---cycles-21-30)
