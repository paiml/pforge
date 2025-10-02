# Property-Based Testing

Property-based testing automatically discovers edge cases by generating thousands of random test inputs and verifying that certain properties (invariants) always hold true. pforge uses **12 property-based tests** with **10,000 iterations each**, totaling **120,000 automated test cases** that would be infeasible to write manually.

## Property-Based Testing Philosophy

Traditional example-based testing tests specific cases. Property-based testing tests universal truths:

| Approach | Example-Based | Property-Based |
|----------|--------------|----------------|
| **Test cases** | Hand-written | Auto-generated |
| **Coverage** | Specific scenarios | Wide input space |
| **Edge cases** | Manual discovery | Automatic discovery |
| **Count** | Dozens | Thousands |
| **Failures** | Show bug | Find + minimize example |

### The Power of Properties

A single property test replaces hundreds of example tests:

```rust
// Example-based: Test specific cases
#[test]
fn test_config_roundtrip_example1() {
    let config = /* specific config */;
    let yaml = serde_yml::to_string(&config).unwrap();
    let parsed: ForgeConfig = serde_yml::from_str(&yaml).unwrap();
    assert_eq!(config.name, parsed.name);
}

#[test]
fn test_config_roundtrip_example2() { /* ... */ }
// ... hundreds more examples needed ...

// Property-based: Test universal property
proptest! {
    #[test]
    fn config_serialization_roundtrip(config in arb_forge_config()) {
        // Tests 10,000 random configs automatically!
        let yaml = serde_yml::to_string(&config)?;
        let parsed: ForgeConfig = serde_yml::from_str(&yaml)?;
        prop_assert_eq!(config.forge.name, parsed.forge.name);
    }
}
```

## Setup and Configuration

pforge uses the `proptest` crate for property-based testing:

```toml
# Cargo.toml
[dev-dependencies]
proptest = "1.0"
```

### Proptest Configuration

```rust
proptest! {
    #![proptest_config(ProptestConfig {
        cases: 10000,  // Run 10K iterations per property
        max_shrink_iters: 10000,  // Minimize failing examples
        ..ProptestConfig::default()
    })]

    #[test]
    fn my_property(input in arb_my_type()) {
        // Test logic...
    }
}
```

## Arbitrary Generators

Generators create random test data. pforge has custom generators for all config types:

### Simple Type Generators

```rust
fn arb_simple_type() -> impl Strategy<Value = SimpleType> {
    prop_oneof![
        Just(SimpleType::String),
        Just(SimpleType::Integer),
        Just(SimpleType::Float),
        Just(SimpleType::Boolean),
        Just(SimpleType::Array),
        Just(SimpleType::Object),
    ]
}

fn arb_transport_type() -> impl Strategy<Value = TransportType> {
    prop_oneof![
        Just(TransportType::Stdio),
        Just(TransportType::Sse),
        Just(TransportType::WebSocket),
    ]
}

fn arb_optimization_level() -> impl Strategy<Value = OptimizationLevel> {
    prop_oneof![
        Just(OptimizationLevel::Debug),
        Just(OptimizationLevel::Release),
    ]
}
```

### Structured Generators

```rust
fn arb_forge_metadata() -> impl Strategy<Value = ForgeMetadata> {
    (
        "[a-z][a-z0-9_-]{2,20}",  // Name regex
        "[0-9]\\.[0-9]\\.[0-9]",  // Version regex
        arb_transport_type(),
        arb_optimization_level(),
    )
        .prop_map(|(name, version, transport, optimization)| ForgeMetadata {
            name,
            version,
            transport,
            optimization,
        })
}

fn arb_handler_ref() -> impl Strategy<Value = HandlerRef> {
    "[a-z][a-z0-9_]{2,10}::[a-z][a-z0-9_]{2,10}"
        .prop_map(|path| HandlerRef { path, inline: None })
}

fn arb_param_schema() -> impl Strategy<Value = ParamSchema> {
    prop::collection::hash_map(
        "[a-z][a-z0-9_]{2,15}",  // Field names
        arb_simple_type().prop_map(ParamType::Simple),
        0..5,  // 0-5 fields
    )
    .prop_map(|fields| ParamSchema { fields })
}
```

### Complex Generators with Constraints

```rust
fn arb_forge_config() -> impl Strategy<Value = ForgeConfig> {
    (
        arb_forge_metadata(),
        prop::collection::vec(arb_tool_def(), 1..10),
    )
        .prop_map(|(forge, tools)| {
            // Ensure unique tool names (constraint)
            let mut unique_tools = Vec::new();
            let mut seen_names = std::collections::HashSet::new();

            for tool in tools {
                let name = tool.name();
                if seen_names.insert(name.to_string()) {
                    unique_tools.push(tool);
                }
            }

            ForgeConfig {
                forge,
                tools: unique_tools,
                resources: vec![],
                prompts: vec![],
                state: None,
            }
        })
}
```

## pforge's 12 Properties

### Category 1: Configuration Properties (6 tests)

#### Property 1: Serialization Roundtrip

**Invariant**: Serializing and deserializing a config preserves its structure.

```rust
proptest! {
    #[test]
    fn config_serialization_roundtrip(config in arb_forge_config()) {
        // YAML roundtrip
        let yaml = serde_yml::to_string(&config).unwrap();
        let parsed: ForgeConfig = serde_yml::from_str(&yaml).unwrap();

        // Key properties preserved
        prop_assert_eq!(&config.forge.name, &parsed.forge.name);
        prop_assert_eq!(&config.forge.version, &parsed.forge.version);
        prop_assert_eq!(config.tools.len(), parsed.tools.len());
    }
}
```

**Edge cases found**: Empty strings, special characters, Unicode in names.

#### Property 2: Tool Name Uniqueness

**Invariant**: After validation, all tool names are unique.

```rust
proptest! {
    #[test]
    fn tool_names_unique(config in arb_forge_config()) {
        let mut names = std::collections::HashSet::new();
        for tool in &config.tools {
            prop_assert!(names.insert(tool.name()));
        }
    }
}
```

**Edge cases found**: Case sensitivity, whitespace differences.

#### Property 3: Valid Configs Pass Validation

**Invariant**: Configs generated by our generators always pass validation.

```rust
proptest! {
    #[test]
    fn valid_configs_pass_validation(config in arb_forge_config()) {
        let result = validate_config(&config);
        prop_assert!(result.is_ok(), "Valid config failed validation: {:?}", result);
    }
}
```

**Edge cases found**: Empty tool lists, minimal configs.

#### Property 4: Handler Paths Contain Separator

**Invariant**: Native tool handler paths always contain `::`.

```rust
proptest! {
    #[test]
    fn native_handler_paths_valid(config in arb_forge_config()) {
        for tool in &config.tools {
            if let ToolDef::Native { handler, .. } = tool {
                prop_assert!(handler.path.contains("::"),
                    "Handler path '{}' doesn't contain ::", handler.path);
            }
        }
    }
}
```

**Edge cases found**: Single-segment paths, paths with multiple separators.

#### Property 5: Transport Types Serialize Correctly

**Invariant**: Transport types roundtrip through serialization.

```rust
proptest! {
    #[test]
    fn transport_types_valid(config in arb_forge_config()) {
        let yaml = serde_yml::to_string(&config.forge.transport).unwrap();
        let parsed: TransportType = serde_yml::from_str(&yaml).unwrap();
        prop_assert_eq!(config.forge.transport, parsed);
    }
}
```

#### Property 6: Tool Names Follow Conventions

**Invariant**: Tool names are lowercase alphanumeric with hyphens/underscores, length 3-50.

```rust
proptest! {
    #[test]
    fn tool_names_follow_conventions(config in arb_forge_config()) {
        for tool in &config.tools {
            let name = tool.name();
            prop_assert!(name.chars().all(|c|
                c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-' || c == '_'
            ), "Tool name '{}' doesn't follow conventions", name);

            prop_assert!(name.len() >= 3 && name.len() <= 50,
                "Tool name '{}' length {} not in range 3-50", name, name.len());
        }
    }
}
```

### Category 2: Validation Properties (2 tests)

#### Property 7: Duplicate Names Always Rejected

**Invariant**: Configs with duplicate tool names always fail validation.

```rust
proptest! {
    #[test]
    fn duplicate_tool_names_rejected(name in "[a-z][a-z0-9_-]{2,20}") {
        let config = ForgeConfig {
            forge: create_test_metadata(),
            tools: vec![
                ToolDef::Native {
                    name: name.clone(),
                    description: "Tool 1".to_string(),
                    handler: HandlerRef { path: "mod1::handler".to_string(), inline: None },
                    params: ParamSchema { fields: HashMap::new() },
                    timeout_ms: None,
                },
                ToolDef::Native {
                    name: name.clone(),  // Duplicate!
                    description: "Tool 2".to_string(),
                    handler: HandlerRef { path: "mod2::handler".to_string(), inline: None },
                    params: ParamSchema { fields: HashMap::new() },
                    timeout_ms: None,
                },
            ],
            resources: vec![],
            prompts: vec![],
            state: None,
        };

        let result = validate_config(&config);
        prop_assert!(result.is_err(), "Duplicate names should fail validation");
        prop_assert!(matches!(result.unwrap_err(), ConfigError::DuplicateToolName(_)));
    }
}
```

#### Property 8: Invalid Handler Paths Rejected

**Invariant**: Handler paths without `::` are always rejected.

```rust
proptest! {
    #[test]
    fn invalid_handler_paths_rejected(path in "[a-z]{3,20}") {
        // Path without :: should fail
        let config = create_config_with_handler_path(path);
        let result = validate_config(&config);
        prop_assert!(result.is_err(), "Invalid handler path should fail validation");
    }
}
```

### Category 3: Edge Case Properties (2 tests)

#### Property 9: Empty Configs Valid

**Invariant**: Configs with only metadata (no tools) are valid.

```rust
proptest! {
    #[test]
    fn empty_config_valid(forge in arb_forge_metadata()) {
        let config = ForgeConfig {
            forge,
            tools: vec![],
            resources: vec![],
            prompts: vec![],
            state: None,
        };

        let result = validate_config(&config);
        prop_assert!(result.is_ok(), "Empty config should be valid");
    }
}
```

#### Property 10: Single Tool Configs Valid

**Invariant**: Any config with exactly one tool is valid.

```rust
proptest! {
    #[test]
    fn single_tool_valid(forge in arb_forge_metadata(), tool in arb_tool_def()) {
        let config = ForgeConfig {
            forge,
            tools: vec![tool],
            resources: vec![],
            prompts: vec![],
            state: None,
        };

        let result = validate_config(&config);
        prop_assert!(result.is_ok(), "Single tool config should be valid");
    }
}
```

### Category 4: Type System Properties (2 tests)

#### Property 11: HTTP Methods Serialize Correctly

```rust
proptest! {
    #[test]
    fn http_methods_valid(method in arb_http_method()) {
        let yaml = serde_yml::to_string(&method).unwrap();
        let parsed: HttpMethod = serde_yml::from_str(&yaml).unwrap();
        prop_assert_eq!(method, parsed);
    }
}
```

#### Property 12: Optimization Levels Consistent

```rust
proptest! {
    #[test]
    fn optimization_levels_consistent(level in arb_optimization_level()) {
        let yaml = serde_yml::to_string(&level).unwrap();
        let parsed: OptimizationLevel = serde_yml::from_str(&yaml).unwrap();
        prop_assert_eq!(level, parsed);
    }
}
```

## Shrinking: Minimal Failing Examples

When a property fails, proptest **shrinks** the input to find the minimal example:

```rust
// Property fails with complex config
Config {
    name: "xyz_server_test_123",
    tools: [tool1, tool2, tool3, tool4],
    ...
}

// Proptest shrinks to minimal failing case
Config {
    name: "a",  // Minimal failing name
    tools: [],  // Minimal failing tools
    ...
}
```

Shrunk examples are **persisted** in `proptest-regressions/` to prevent regressions.

## Running Property Tests

### Basic Commands

```bash
# Run all property tests (10K cases each)
cargo test --test property_test

# Run specific property
cargo test --test property_test config_serialization_roundtrip

# Run with more cases
PROPTEST_CASES=100000 cargo test --test property_test

# Run with seed for reproducibility
PROPTEST_SEED=1234567890 cargo test --test property_test
```

### Release Mode

Property tests run faster in release mode:

```bash
# Recommended: Run in release mode
cargo test --test property_test --release -- --test-threads=1
```

This is the default in `Makefile`:

```bash
make test-property
```

## Regression Files

Failed tests are saved in `proptest-regressions/`:

```
crates/pforge-integration-tests/
└── proptest-regressions/
    └── property_test.txt  # Failing cases
```

Example regression file:

```
# Seeds for failing test cases. Edit at your own risk.
# property: config_serialization_roundtrip
xs 3582691854 1234567890
```

**Important**: Commit regression files to git! They ensure failures don't reoccur.

## Writing New Properties

### Step 1: Define Generator

```rust
fn arb_my_type() -> impl Strategy<Value = MyType> {
    (
        arb_field1(),
        arb_field2(),
    ).prop_map(|(field1, field2)| MyType { field1, field2 })
}
```

### Step 2: Write Property

```rust
proptest! {
    #[test]
    fn my_property(input in arb_my_type()) {
        let result = my_function(input);
        prop_assert!(result.is_ok());
    }
}
```

### Step 3: Run and Refine

```bash
cargo test --test property_test my_property
```

If failures occur:
1. Check if property is actually true
2. Adjust generator constraints
3. Fix implementation bugs
4. Commit regression file

## Property Testing Best Practices

### 1. Test Universal Truths

```rust
// Good: Universal property
proptest! {
    #[test]
    fn serialize_deserialize_roundtrip(x in any::<MyType>()) {
        let json = serde_json::to_string(&x)?;
        let y: MyType = serde_json::from_str(&json)?;
        prop_assert_eq!(x, y);  // Always true
    }
}

// Bad: Specific assertion
proptest! {
    #[test]
    fn bad_property(x in any::<i32>()) {
        prop_assert_eq!(x, 42);  // Only true 1/2^32 times!
    }
}
```

### 2. Use Meaningful Generators

```rust
// Good: Generates valid data
fn arb_email() -> impl Strategy<Value = String> {
    "[a-z]{1,10}@[a-z]{1,10}\\.(com|org|net)"
}

// Bad: Most generated strings aren't emails
fn arb_email_bad() -> impl Strategy<Value = String> {
    any::<String>()  // Generates random bytes
}
```

### 3. Add Constraints to Generators

```rust
fn arb_positive_number() -> impl Strategy<Value = i32> {
    1..=i32::MAX  // Constrained range
}

fn arb_non_empty_vec<T: Arbitrary>() -> impl Strategy<Value = Vec<T>> {
    prop::collection::vec(any::<T>(), 1..100)  // At least 1 element
}
```

### 4. Test Error Conditions

```rust
proptest! {
    #[test]
    fn invalid_input_rejected(bad_input in arb_invalid_input()) {
        let result = validate(bad_input);
        prop_assert!(result.is_err());  // Should always fail
    }
}
```

## Benefits and Limitations

### Benefits

1. **Comprehensive**: 10K+ cases per property vs ~10 manual examples
2. **Edge case discovery**: Finds bugs humans miss
3. **Regression prevention**: Failing cases saved automatically
4. **Documentation**: Properties describe system invariants
5. **Confidence**: Mathematical proof of correctness over input space

### Limitations

1. **Slower**: 10K iterations takes seconds vs milliseconds for unit tests
2. **Complexity**: Generators can be complex to write
3. **False positives**: Properties must be precisely stated
4. **Non-determinism**: Random failures can be hard to debug (use seeds!)

## Integration with CI/CD

Property tests run in CI but with fewer iterations for speed:

```yaml
# .github/workflows/quality.yml
- name: Property tests
  run: |
    PROPTEST_CASES=1000 cargo test --test property_test --release
```

Locally, run full 10K iterations:

```bash
make test-property  # Uses 10K cases
```

## Real-World Impact

Property-based testing has found real bugs in pforge:

1. **Unicode handling**: Tool names with emoji crashed parser
2. **Empty configs**: Validation rejected valid empty tool lists
3. **Case sensitivity**: Duplicate detection was case-sensitive
4. **Whitespace**: Leading/trailing whitespace in names caused issues
5. **Nesting depth**: Deeply nested param schemas caused stack overflow

All caught by property tests before reaching production!

## Summary

Property-based testing provides massive test coverage with minimal code:

- **12 properties** generate **120,000 test cases**
- **Automatic edge case discovery** finds bugs humans miss
- **Shrinking** provides minimal failing examples
- **Regression prevention** through persisted failing cases
- **Mathematical rigor** proves invariants hold

Combined with unit tests (Chapter 9.1) and integration tests (Chapter 9.2), property-based testing ensures pforge's configuration system is rock-solid. Next, Chapter 9.4 covers mutation testing to validate that our tests are actually effective.

## Further Reading

- [Proptest Book](https://proptest-rs.github.io/proptest/)
- [QuickCheck Paper](https://www.cs.tufts.edu/~nr/cs257/archive/john-hughes/quick.pdf) - Original property testing paper
- [Hypothesis](https://hypothesis.readthedocs.io/) - Python property testing
- pforge property tests: `crates/pforge-integration-tests/property_test.rs`
