# Mutation Testing

Mutation testing validates the quality of your tests by deliberately introducing bugs ("mutations") into your code and checking if your tests catch them. pforge targets a **≥90% mutation kill rate** using `cargo-mutants`, ensuring our 115 tests are actually effective.

## The Problem Mutation Testing Solves

You can have 100% test coverage and still have ineffective tests:

```rust
// Production code
pub fn validate_config(config: &ForgeConfig) -> Result<()> {
    if config.tools.is_empty() {
        return Err(ConfigError::EmptyTools);
    }
    Ok(())
}

// Test with 100% line coverage but zero assertions
#[test]
fn test_validate_config() {
    let config = create_valid_config();
    validate_config(&config);  // ❌ No assertion! Test passes even if code is broken
}
```

**Coverage says**: ✅ 100% line coverage
**Reality**: This test catches nothing!

Mutation testing finds these **weak tests** by mutating code and seeing if tests fail.

## How Mutation Testing Works

1. **Baseline**: Run all tests → they should pass
2. **Mutate**: Change code in a specific way (e.g., change `==` to `!=`)
3. **Test**: Run tests again
4. **Result**:
   - Tests **fail** → Mutation **killed** ✅ (good test!)
   - Tests **pass** → Mutation **survived** ❌ (weak test!)

### Example Mutation

```rust
// Original code
pub fn has_handler(&self, name: &str) -> bool {
    self.handlers.contains_key(name)  // Original
}

// Mutation 1: Change return value
pub fn has_handler(&self, name: &str) -> bool {
    !self.handlers.contains_key(name)  // Mutated: inverted logic
}

// Mutation 2: Change to always return true
pub fn has_handler(&self, name: &str) -> bool {
    true  // Mutated: constant return
}

// Mutation 3: Change to always return false
pub fn has_handler(&self, name: &str) -> bool {
    false  // Mutated: constant return
}
```

**Good test** (catches all mutations):

```rust
#[test]
fn test_has_handler() {
    let mut registry = HandlerRegistry::new();

    // Should return false for non-existent handler
    assert!(!registry.has_handler("nonexistent"));  // Kills mutation 2

    registry.register("test", TestHandler);

    // Should return true for registered handler
    assert!(registry.has_handler("test"));  // Kills mutations 1 & 3
}
```

**Weak test** (mutations survive):

```rust
#[test]
fn test_has_handler_weak() {
    let mut registry = HandlerRegistry::new();
    registry.register("test", TestHandler);

    // Only tests positive case - mutations 1 & 2 survive!
    assert!(registry.has_handler("test"));
}
```

## Setting Up cargo-mutants

### Installation

```bash
cargo install cargo-mutants
```

### Basic Usage

```bash
# Run mutation testing
cargo mutants

# Run on specific crate
cargo mutants -p pforge-runtime

# Run on specific file
cargo mutants --file crates/pforge-runtime/src/registry.rs

# Show what would be mutated without running tests
cargo mutants --list
```

### Configuration

Create `.cargo/mutants.toml`:

```toml
# Timeout per mutant (5 minutes default)
timeout = 300

# Exclude certain patterns
exclude_globs = [
    "**/tests/**",
    "**/*_test.rs",
]

# Additional test args
test_args = ["--release"]
```

## Common Mutations

cargo-mutants applies various mutation operators:

### 1. Replace Function Return Values

```rust
// Original
fn get_count(&self) -> usize {
    self.handlers.len()
}

// Mutations
fn get_count(&self) -> usize { 0 }      // Always 0
fn get_count(&self) -> usize { 1 }      // Always 1
fn get_count(&self) -> usize { usize::MAX }  // Max value
```

**Test that kills**:

```rust
#[test]
fn test_get_count() {
    let registry = HandlerRegistry::new();
    assert_eq!(registry.get_count(), 0);  // Kills non-zero mutations

    registry.register("test", TestHandler);
    assert_eq!(registry.get_count(), 1);  // Kills 0 and MAX mutations
}
```

### 2. Negate Boolean Conditions

```rust
// Original
if config.tools.is_empty() {
    return Err(ConfigError::EmptyTools);
}

// Mutation
if !config.tools.is_empty() {  // Inverted!
    return Err(ConfigError::EmptyTools);
}
```

**Test that kills**:

```rust
#[test]
fn test_validation_rejects_empty_tools() {
    let config = create_config_with_no_tools();
    assert!(validate_config(&config).is_err());  // Catches inversion
}

#[test]
fn test_validation_accepts_valid_tools() {
    let config = create_config_with_tools();
    assert!(validate_config(&config).is_ok());  // Also needed!
}
```

### 3. Change Comparison Operators

```rust
// Original
if count > threshold {
    // ...
}

// Mutations
if count >= threshold { }  // Change > to >=
if count < threshold { }   // Change > to <
if count == threshold { }  // Change > to ==
if count != threshold { }  // Change > to !=
```

**Test that kills**:

```rust
#[test]
fn test_threshold_boundary() {
    assert!(!exceeds_threshold(5, 5));   // count == threshold
    assert!(!exceeds_threshold(4, 5));   // count < threshold
    assert!(exceeds_threshold(6, 5));    // count > threshold
}
```

### 4. Delete Statements

```rust
// Original
fn process(&mut self) {
    self.validate();  // Original
    self.execute();
}

// Mutation: Delete validation
fn process(&mut self) {
    // self.validate();  // Deleted!
    self.execute();
}
```

**Test that kills**:

```rust
#[test]
fn test_process_validates_before_executing() {
    let mut processor = create_invalid_processor();

    // Should fail during validation
    assert!(processor.process().is_err());
}
```

### 5. Replace Binary Operators

```rust
// Original
let sum = a + b;

// Mutations
let sum = a - b;  // + → -
let sum = a * b;  // + → *
let sum = a / b;  // + → /
```

## pforge Mutation Testing Strategy

### Target: 90% Kill Rate

```
Mutation Score = (Killed Mutants / Total Mutants) × 100%

pforge target: ≥ 90%
```

### Running Mutation Tests

```bash
# Full mutation test suite
make mutants

# Or manually
cargo mutants --test-threads=8
```

### Example Run Output

```
Testing mutants:
crates/pforge-runtime/src/registry.rs:114:5: replace HandlerRegistry::new -> HandlerRegistry with Default::default()
    CAUGHT in 0.2s

crates/pforge-runtime/src/registry.rs:121:9: replace <impl HandlerRegistry>::register -> () with ()
    CAUGHT in 0.3s

crates/pforge-config/src/validator.rs:9:20: replace <impl>::validate -> Result<()> with Ok(())
    CAUGHT in 0.2s

crates/pforge-config/src/validator.rs:15:16: replace != with ==
    CAUGHT in 0.1s

Summary:
  Tested: 127 mutants
  Caught: 117 mutants (92.1%)
  Missed: 8 mutants (6.3%)
  Timeout: 2 mutants (1.6%)
```

### Interpreting Results

- **Caught**: ✅ Test suite detected the mutation (good!)
- **Missed**: ❌ Test suite didn't detect mutation (add test!)
- **Timeout**: ⚠️ Test took too long (possibly infinite loop)
- **Unviable**: Mutation wouldn't compile (ignored)

## Improving Kill Rate

### Strategy 1: Test Both Branches

```rust
// Code with branch
fn validate(&self) -> Result<()> {
    if self.is_valid() {
        Ok(())
    } else {
        Err(Error::Invalid)
    }
}

// Weak: Only tests one branch
#[test]
fn test_validate_success() {
    let validator = create_valid();
    assert!(validator.validate().is_ok());
}

// Strong: Tests both branches
#[test]
fn test_validate_success() {
    let validator = create_valid();
    assert!(validator.validate().is_ok());
}

#[test]
fn test_validate_failure() {
    let validator = create_invalid();
    assert!(validator.validate().is_err());
}
```

### Strategy 2: Test Boundary Conditions

```rust
// Code with comparison
fn is_large(&self) -> bool {
    self.size > 100
}

// Weak: Only tests middle of range
#[test]
fn test_is_large() {
    assert!(Item { size: 150 }.is_large());
    assert!(!Item { size: 50 }.is_large());
}

// Strong: Tests boundary
#[test]
fn test_is_large_boundary() {
    assert!(!Item { size: 100 }.is_large());  // Exactly at boundary
    assert!(!Item { size: 99 }.is_large());   // Just below
    assert!(Item { size: 101 }.is_large());   // Just above
}
```

### Strategy 3: Test Return Values

```rust
// Code
fn get_status(&self) -> Status {
    if self.is_ready() {
        Status::Ready
    } else {
        Status::NotReady
    }
}

// Weak: No assertion on return value
#[test]
fn test_get_status() {
    let item = Item::new();
    item.get_status();  // ❌ Doesn't assert anything!
}

// Strong: Asserts actual vs expected
#[test]
fn test_get_status_ready() {
    let item = create_ready_item();
    assert_eq!(item.get_status(), Status::Ready);
}

#[test]
fn test_get_status_not_ready() {
    let item = create_not_ready_item();
    assert_eq!(item.get_status(), Status::NotReady);
}
```

### Strategy 4: Test Error Cases

```rust
// Code
fn parse(input: &str) -> Result<Config> {
    if input.is_empty() {
        return Err(Error::EmptyInput);
    }
    // ... parse logic
    Ok(config)
}

// Weak: Only tests success
#[test]
fn test_parse_success() {
    let result = parse("valid config");
    assert!(result.is_ok());
}

// Strong: Tests both success and error
#[test]
fn test_parse_success() {
    let result = parse("valid config");
    assert!(result.is_ok());
}

#[test]
fn test_parse_empty_input() {
    let result = parse("");
    assert!(matches!(result.unwrap_err(), Error::EmptyInput));
}
```

## Real pforge Mutation Test Results

### Before Mutation Testing

Initial run showed 82% kill rate with 23 surviving mutants:

```
Survived mutations:
1. validator.rs:25 - Changed `contains_key` to always return true
2. registry.rs:142 - Removed error handling
3. config.rs:18 - Changed `is_empty()` to `!is_empty()`
...
```

### After Adding Tests

```rust
// Added test for mutation 1
#[test]
fn test_duplicate_detection_both_cases() {
    // Tests that contains_key is actually checked
    let mut seen = HashSet::new();
    assert!(!seen.contains("key"));  // Not present
    seen.insert("key");
    assert!(seen.contains("key"));   // Present
}

// Added test for mutation 2
#[test]
fn test_error_propagation() {
    let result = fallible_function();
    assert!(result.is_err());
    match result.unwrap_err() {
        Error::Expected => {},  // Verify specific error
        _ => panic!("Wrong error type"),
    }
}

// Added test for mutation 3
#[test]
fn test_empty_check() {
    let empty = Vec::<String>::new();
    assert!(is_empty_error(&empty).is_err());  // Empty case

    let nonempty = vec!["item".to_string()];
    assert!(is_empty_error(&nonempty).is_ok()); // Non-empty case
}
```

### Final Result

```
Summary:
  Tested: 127 mutants
  Caught: 117 mutants (92.1%) ✅
  Missed: 8 mutants (6.3%)
  Timeout: 2 mutants (1.6%)

Mutation score: 92.1% (TARGET: ≥90%)
```

## Acceptable Mutations

Some mutations are acceptable to miss:

### 1. Logging Statements

```rust
// Original
fn process(&self) {
    log::debug!("Processing item");
    // ... actual logic
}

// Mutation: Delete log statement
fn process(&self) {
    // log::debug!("Processing item");  // Deleted
    // ... actual logic
}
```

**Acceptable**: Tests shouldn't depend on logging.

### 2. Performance Optimizations

```rust
// Original
fn calculate(&self) -> i32 {
    self.cached_value.unwrap_or_else(|| expensive_calculation())
}

// Mutation: Always calculate
fn calculate(&self) -> i32 {
    expensive_calculation()  // Remove cache
}
```

**Acceptable**: Result is same, just slower.

### 3. Error Messages

```rust
// Original
return Err(Error::Invalid("Field 'name' is required".to_string()));

// Mutation
return Err(Error::Invalid("".to_string()));
```

**Acceptable if**: Test only checks error variant, not message.

## Integration with CI/CD

### GitHub Actions

```yaml
# .github/workflows/mutation.yml
name: Mutation Testing

on:
  pull_request:
  schedule:
    - cron: '0 0 * * 0'  # Weekly

jobs:
  mutants:
    runs-on: ubuntu-latest
    timeout-minutes: 60

    steps:
      - uses: actions/checkout@v3

      - name: Install cargo-mutants
        run: cargo install cargo-mutants

      - name: Run mutation tests
        run: cargo mutants --test-threads=4

      - name: Check mutation score
        run: |
          SCORE=$(cargo mutants --json | jq '.score')
          if (( $(echo "$SCORE < 90" | bc -l) )); then
            echo "Mutation score $SCORE% below target 90%"
            exit 1
          fi
```

### Local Pre-Push Hook

```bash
#!/bin/bash
# .git/hooks/pre-push

echo "Running mutation tests..."

cargo mutants --test-threads=8 || {
    echo "❌ Mutation testing failed"
    echo "Fix tests or accept surviving mutants"
    exit 1
}

echo "✅ Mutation testing passed"
```

## Performance Optimization

Mutation testing is slow. Optimize:

### 1. Parallel Execution

```bash
# Use all cores
cargo mutants --test-threads=$(nproc)
```

### 2. Incremental Testing

```bash
# Only test changed files
cargo mutants --file src/changed_file.rs
```

### 3. Shorter Timeouts

```bash
# Set 60 second timeout per mutant
cargo mutants --timeout=60
```

### 4. Baseline Filtering

```bash
# Skip mutants in tests
cargo mutants --exclude-globs '**/tests/**'
```

## Mutation Testing Best Practices

### 1. Run Regularly, Not Every Commit

```bash
# Weekly in CI, or before releases
make mutants  # Part of quality gate
```

### 2. Focus on Critical Code

```bash
# Prioritize high-value files
cargo mutants --file src/runtime/registry.rs
cargo mutants --file src/config/validator.rs
```

### 3. Track Metrics Over Time

```bash
# Save mutation scores
cargo mutants --json > mutation-report.json
```

### 4. Don't Aim for 100%

90% is excellent. Diminishing returns above that:

- 90%: ✅ Excellent test quality
- 95%: ⚠️ Very good, some effort
- 100%: ❌ Not worth the effort

### 5. Use with Other Metrics

Mutation testing + coverage + complexity:

```bash
make quality-gate  # Runs all quality checks
```

## Limitations

1. **Slow**: Can take 10-60 minutes for large codebases
2. **False positives**: Some mutations are semantically equivalent
3. **Not exhaustive**: Can't test all possible bugs
4. **Requires good tests**: Mutation testing validates tests, not code

## Summary

Mutation testing is the ultimate validation of test quality:

- **Purpose**: Validate that tests actually catch bugs
- **Target**: ≥90% mutation kill rate
- **Tool**: `cargo-mutants`
- **Integration**: Weekly CI runs, pre-release checks
- **Benefit**: Confidence that tests are effective

### Mutation Testing in Context

| Metric | What it measures | pforge target |
|--------|------------------|---------------|
| Line coverage | Lines executed | ≥80% |
| Mutation score | Tests effectiveness | ≥90% |
| Complexity | Code simplicity | ≤20 |
| TDG | Technical debt | ≥0.75 |

All four metrics together ensure comprehensive quality.

## The Complete Testing Picture

pforge's multi-layered testing strategy:

1. **Unit tests** (Chapter 9.1): Fast, focused component tests
2. **Integration tests** (Chapter 9.2): Cross-component workflows
3. **Property tests** (Chapter 9.3): Automated edge case discovery
4. **Mutation tests** (Chapter 9.4): Validate test effectiveness

Result: **115 high-quality tests** that provide genuine confidence in pforge's reliability.

### Quality Metrics

```
115 total tests
├── 74 unit tests (<1ms each)
├── 26 integration tests (<100ms each)
├── 12 property tests (10K cases each = 120K total)
└── Validated by mutation testing (92% kill rate)

Coverage: 85% lines, 78% branches
Complexity: All functions ≤20
Mutation score: 92%
TDG: 0.82
```

This comprehensive approach ensures pforge maintains production-ready quality while enabling rapid, confident development through strict TDD discipline.

## Further Reading

- [cargo-mutants documentation](https://mutants.rs/)
- [PIT Mutation Testing](https://pitest.org/) - Java mutation testing
- pforge mutation config: `.cargo/mutants.toml`
