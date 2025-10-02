# Code Coverage: Measuring Test Quality

You can't improve what you don't measure. Code coverage reveals what your tests actually test—and more importantly, what they don't.

pforge requires **≥80% line coverage** before allowing commits. This isn't about hitting an arbitrary number—it's about ensuring critical code paths are exercised by tests.

This chapter explains what coverage is, how to measure it, how to interpret coverage reports, and how to achieve meaningful coverage (not just high percentages).

## What is Code Coverage?

Code coverage measures the percentage of your code executed during tests. If your tests run 800 of 1000 lines, you have 80% line coverage.

### Types of Coverage

**1. Line Coverage**

**Definition**: Percentage of lines executed by tests

**Example**:

```rust
fn divide(a: i32, b: i32) -> Result<i32, String> {
    if b == 0 {                        // Line 1 ✅ covered
        return Err("division by zero".into());  // Line 2 ❌ not covered
    }
    Ok(a / b)                          // Line 3 ✅ covered
}

#[test]
fn test_divide() {
    assert_eq!(divide(10, 2), Ok(5));  // Covers lines 1 and 3, not 2
}
```

Line coverage: 66% (2 of 3 lines covered)

To hit 100%: add a test for `b == 0` case.

**2. Branch Coverage**

**Definition**: Percentage of decision branches taken by tests

**Example**:

```rust
fn classify(age: i32) -> &'static str {
    if age < 18 {
        "minor"   // Branch A
    } else {
        "adult"   // Branch B
    }
}

#[test]
fn test_classify() {
    assert_eq!(classify(16), "minor");  // Tests branch A only
}
```

Branch coverage: 50% (1 of 2 branches covered)

To hit 100%: add a test for `age >= 18` case.

**3. Function Coverage**

**Definition**: Percentage of functions called by tests

**Example**:

```rust
fn add(a: i32, b: i32) -> i32 { a + b }      // ✅ called by tests
fn multiply(a: i32, b: i32) -> i32 { a * b } // ❌ never called

#[test]
fn test_add() {
    assert_eq!(add(2, 3), 5);  // Only tests add()
}
```

Function coverage: 50% (1 of 2 functions covered)

**4. Statement Coverage**

**Definition**: Percentage of statements executed (similar to line coverage, but counts logical statements, not lines)

**Example**:

```rust
// One line, two statements
let x = if condition { 5 } else { 10 }; y = x * 2;
```

Line coverage might show 100%, but statement coverage reveals if both statements executed.

### pforge's Coverage Requirements

pforge enforces:
- **Line coverage ≥ 80%**: Most code must be tested
- **Branch coverage ≥ 75%**: Most decision paths must be tested

These thresholds catch the majority of bugs while avoiding diminishing returns (95%+ coverage requires exponentially more test effort).

## Measuring Coverage

### Using cargo-llvm-cov

pforge uses `cargo-llvm-cov` for coverage analysis:

```bash
# Install cargo-llvm-cov
cargo install cargo-llvm-cov

# Run coverage
cargo llvm-cov --all-features --workspace
```

**Or use the Makefile**:

```bash
make coverage
```

This runs a two-phase process:
1. **Phase 1**: Run tests with instrumentation (no report)
2. **Phase 2**: Generate HTML and LCOV reports

**Output**:

```
📊 Running comprehensive test coverage analysis...
🔍 Checking for cargo-llvm-cov and cargo-nextest...
🧹 Cleaning old coverage data...
⚙️  Temporarily disabling global cargo config (mold breaks coverage)...
🧪 Phase 1: Running tests with instrumentation (no report)...
📊 Phase 2: Generating coverage reports...
⚙️  Restoring global cargo config...

📊 Coverage Summary:
==================
Filename                      Lines    Covered    Uncovered    %
------------------------------------------------------------
src/handler.rs                234      198        36          84.6%
src/registry.rs               189      167        22          88.4%
src/config.rs                 145      109        36          75.2%
src/server.rs                 178      156        22          87.6%
src/error.rs                  45       45         0           100%
------------------------------------------------------------
TOTAL                         1247     1021       226         81.9%

💡 COVERAGE INSIGHTS:
- HTML report: target/coverage/html/index.html
- LCOV file: target/coverage/lcov.info
- Open HTML: make coverage-open
```

### Coverage Summary

Quick coverage check without full report:

```bash
make coverage-summary

# or
cargo llvm-cov report --summary-only
```

**Output**:

```
Filename                Lines    Covered    Uncovered    %
----------------------------------------------------------
TOTAL                   1247     1021       226         81.9%
```

### HTML Coverage Report

Open the interactive HTML report:

```bash
make coverage-open
```

This opens `target/coverage/html/index.html` in your browser, showing:
- **File-level coverage**: Which files have low coverage
- **Line-by-line highlighting**: Which lines are covered (green) vs. uncovered (red)
- **Branch visualization**: Which branches are tested

**Example report structure**:

```
pforge Coverage Report
├── src/
│   ├── handler.rs       84.6%  ⚠️
│   ├── registry.rs      88.4%  ✅
│   ├── config.rs        75.2%  ❌
│   ├── server.rs        87.6%  ✅
│   └── error.rs         100%   ✅
└── TOTAL                81.9%  ✅
```

Click any file to see line-by-line coverage.

## Interpreting Coverage Reports

### Reading Line-by-Line Coverage

**HTML report shows**:

```rust
// handler.rs
1  ✅  pub fn process(req: &Request) -> Result<Response> {
2  ✅      validate_request(req)?;
3  ✅      let user = authorize_request(req)?;
4  ❌      if req.is_admin_action() {
5  ❌          audit_log(&req);
6  ❌      }
7  ✅      let result = execute_action(req, &user)?;
8  ✅      Ok(Response::new(result))
9  ✅  }
```

**Green (✅)**: Line was executed by at least one test
**Red (❌)**: Line was never executed

Lines 4-6 are uncovered. Need a test for admin actions.

### Understanding Coverage Gaps

**Gap 1: Error Handling**

```rust
fn parse_config(path: &str) -> Result<Config> {
    let file = File::open(path)?;           // ✅ covered
    let mut contents = String::new();       // ✅ covered
    file.read_to_string(&mut contents)?;    // ✅ covered

    serde_yaml::from_str(&contents)         // ❌ error path not covered
        .map_err(|e| Error::InvalidConfig(e))
}

#[test]
fn test_parse_config() {
    // Only tests happy path
    let config = parse_config("valid.yaml").unwrap();
    assert!(config.is_valid());
}
```

Coverage shows `serde_yaml` line is covered, but the error path (`map_err`) isn't. Add a test with invalid YAML.

**Gap 2: Edge Cases**

```rust
fn calculate_discount(price: f64, percent: f64) -> f64 {
    if percent < 0.0 || percent > 100.0 {   // ❌ not covered
        return 0.0;
    }
    price * (percent / 100.0)               // ✅ covered
}

#[test]
fn test_calculate_discount() {
    assert_eq!(calculate_discount(100.0, 10.0), 10.0);
}
```

Edge case (invalid percent) isn't tested. Add tests for `percent < 0` and `percent > 100`.

**Gap 3: Conditional Branches**

```rust
fn should_notify(user: &User, event: &Event) -> bool {
    user.is_subscribed()                    // ✅ covered (both branches)
        && event.is_important()             // ❌ only true branch covered
        && !user.is_snoozed()              // ❌ not reached
}

#[test]
fn test_should_notify() {
    let user = User { subscribed: true, snoozed: false };
    let event = Event { important: true };
    assert!(should_notify(&user, &event));  // Only tests all true
}
```

Short-circuit evaluation means `is_snoozed()` is only called if previous conditions are true. Need tests where `is_important() == false`.

**Gap 4: Dead Code**

```rust
fn legacy_handler(req: &Request) -> Response {  // ❌ never called
    // Old code path, replaced but not deleted
    Response::new("legacy")
}
```

0% coverage on this function. Either test it or delete it.

### Coverage Metrics Interpretation

**80%+ coverage**: Healthy baseline. Most code paths tested.

**Example**:
```
TOTAL    1247     1021       226         81.9%  ✅
```

**70-79% coverage**: Needs improvement. Many untested paths.

**Example**:
```
TOTAL    1247     921        326         73.8%  ⚠️
```

Action: Identify uncovered critical paths and add tests.

**< 70% coverage**: Poor. Significant portions untested.

**Example**:
```
TOTAL    1247     748        499         60.0%  ❌
```

Action: Audit all uncovered code. Either test it or justify why it's untestable.

**100% coverage**: Often a red flag. Either:
- Very simple codebase (rare)
- Tests are testing trivial code (waste of effort)
- Coverage gaming (hitting lines without meaningful assertions)

Aim for 80-90%, not 100%.

## Improving Coverage

### Strategy 1: Test Error Paths

**Before** (50% coverage):

```rust
fn divide(a: i32, b: i32) -> Result<i32, String> {
    if b == 0 {                                // ❌ not covered
        return Err("division by zero".into()); // ❌ not covered
    }
    Ok(a / b)                                  // ✅ covered
}

#[test]
fn test_divide() {
    assert_eq!(divide(10, 2), Ok(5));
}
```

**After** (100% coverage):

```rust
#[test]
fn test_divide() {
    // Happy path
    assert_eq!(divide(10, 2), Ok(5));

    // Error path
    assert_eq!(divide(10, 0), Err("division by zero".into()));
}
```

**Result**: Coverage 50% → 100%

### Strategy 2: Test All Branches

**Before** (60% branch coverage):

```rust
fn classify(age: i32) -> &'static str {
    if age < 13 {                       // ✅ true branch covered
        "child"                         // ✅ covered
    } else if age < 20 {                // ❌ true branch not covered
        "teenager"                      // ❌ not covered
    } else {                            // ✅ false branch covered
        "adult"                         // ✅ covered
    }
}

#[test]
fn test_classify() {
    assert_eq!(classify(10), "child");
    assert_eq!(classify(25), "adult");
}
```

**After** (100% branch coverage):

```rust
#[test]
fn test_classify() {
    // All branches
    assert_eq!(classify(10), "child");    // age < 13
    assert_eq!(classify(16), "teenager"); // 13 <= age < 20
    assert_eq!(classify(25), "adult");    // age >= 20
}
```

**Result**: Branch coverage 60% → 100%

### Strategy 3: Test Match Arms

**Before** (40% match arm coverage):

```rust
fn handle_command(cmd: Command) -> Result<String> {
    match cmd {
        Command::Read(id) => db.read(&id),     // ✅ covered
        Command::Write(id, data) => {          // ❌ not covered
            db.write(&id, &data)
        }
        Command::Delete(id) => db.delete(&id), // ❌ not covered
        Command::List => db.list(),            // ❌ not covered
    }
}

#[test]
fn test_handle_command() {
    assert!(handle_command(Command::Read("123")).is_ok());
}
```

**After** (100% match arm coverage):

```rust
#[test]
fn test_handle_command() {
    assert!(handle_command(Command::Read("123")).is_ok());
    assert!(handle_command(Command::Write("123", "data")).is_ok());
    assert!(handle_command(Command::Delete("123")).is_ok());
    assert!(handle_command(Command::List).is_ok());
}
```

**Result**: Match arm coverage 25% → 100%

### Strategy 4: Parametric Tests

Test many cases efficiently:

**Before** (3 tests, repetitive):

```rust
#[test]
fn test_validate_empty() {
    assert!(validate("").is_err());
}

#[test]
fn test_validate_too_long() {
    assert!(validate(&"x".repeat(101)).is_err());
}

#[test]
fn test_validate_invalid_chars() {
    assert!(validate("hello@world").is_err());
}
```

**After** (1 parametric test):

```rust
#[test]
fn test_validate() {
    let invalid_cases = vec![
        ("", "empty"),
        (&"x".repeat(101), "too long"),
        ("hello@world", "invalid chars"),
        ("123start", "starts with digit"),
    ];

    for (input, reason) in invalid_cases {
        assert!(validate(input).is_err(), "Should reject: {}", reason);
    }

    let valid_cases = vec!["hello", "user123", "validName"];
    for input in valid_cases {
        assert!(validate(input).is_ok(), "Should accept: {}", input);
    }
}
```

**Result**: More coverage with less code duplication.

### Strategy 5: Property-Based Testing

Use `proptest` to generate test cases:

```rust
use proptest::prelude::*;

proptest! {
    #[test]
    fn test_divide_properties(a in -1000i32..1000, b in -1000i32..1000) {
        if b == 0 {
            // Error path always covered
            assert!(divide(a, b).is_err());
        } else {
            // Success path always covered
            let result = divide(a, b).unwrap();
            assert_eq!(result, a / b);
        }
    }
}
```

Proptest generates hundreds of test cases, ensuring high coverage.

## Coverage Anti-Patterns

### Anti-Pattern 1: Coverage Gaming

**Bad**:

```rust
fn complex_logic(input: &str) -> Result<String> {
    if input.is_empty() {
        return Err("empty".into());
    }
    // ... complex processing
    Ok(result)
}

#[test]
fn test_complex_logic() {
    // Hits all lines but doesn't verify correctness
    let _ = complex_logic("test");
    let _ = complex_logic("");
}
```

Lines are covered, but test has no assertions. It's not really testing anything.

**Good**:

```rust
#[test]
fn test_complex_logic() {
    // Meaningful assertions
    assert_eq!(complex_logic("test"), Ok("processed: test".into()));
    assert_eq!(complex_logic(""), Err("empty".into()));
}
```

### Anti-Pattern 2: Testing Trivial Code

**Bad**:

```rust
// Trivial getter - doesn't need a test
fn name(&self) -> &str {
    &self.name
}

#[test]
fn test_name() {
    let obj = Object { name: "test".into() };
    assert_eq!(obj.name(), "test");
}
```

This inflates coverage without adding value. Focus tests on logic, not boilerplate.

**Good**: Skip trivial getters. Test complex logic instead.

### Anti-Pattern 3: Ignoring Untestable Code

**Bad**:

```rust
fn production_logic() {
    #[cfg(test)]
    {
        // Unreachable in production, but shows as covered
        panic!("test-only panic");
    }

    // Actual logic
}
```

Coverage shows test-only code as covered, hiding gaps in production code.

**Good**: Separate test-only code into test modules.

### Anti-Pattern 4: High Coverage, Low Quality

**Bad**:

```rust
fn authenticate(username: &str, password: &str) -> Result<User> {
    let user = db.get_user(username)?;
    if user.password_hash == hash(password) {
        Ok(user)
    } else {
        Err(Error::InvalidCredentials)
    }
}

#[test]
fn test_authenticate() {
    // Only tests happy path, but achieves 75% line coverage
    let user = authenticate("alice", "password123").unwrap();
    assert_eq!(user.username, "alice");
}
```

High coverage (75%) but critical error path (`Err(Error::InvalidCredentials)`) is untested.

**Good**: Test both happy and error paths:

```rust
#[test]
fn test_authenticate() {
    // Happy path
    assert!(authenticate("alice", "password123").is_ok());

    // Error paths
    assert!(authenticate("alice", "wrong").is_err());
    assert!(authenticate("nonexistent", "password").is_err());
}
```

## Coverage in CI/CD

Enforce coverage in CI:

```yaml
# .github/workflows/coverage.yml
name: Coverage

on: [push, pull_request]

jobs:
  coverage:
    runs-on: ubuntu-latest

    steps:
      - uses: actions/checkout@v3

      - name: Install Rust
        uses: actions-rs/toolchain@v1
        with:
          toolchain: stable

      - name: Install cargo-llvm-cov
        run: cargo install cargo-llvm-cov

      - name: Run coverage
        run: cargo llvm-cov --all-features --workspace --lcov --output-path lcov.info

      - name: Check coverage threshold
        run: |
          COVERAGE=$(cargo llvm-cov report --summary-only | grep -oP '\d+\.\d+(?=%)')
          echo "Coverage: $COVERAGE%"
          if (( $(echo "$COVERAGE < 80.0" | bc -l) )); then
            echo "Coverage $COVERAGE% is below minimum 80%"
            exit 1
          fi

      - name: Upload to Codecov
        uses: codecov/codecov-action@v3
        with:
          files: lcov.info
          fail_ci_if_error: true
```

This blocks PRs with coverage < 80%.

## Coverage Best Practices

### 1. Focus on Critical Paths

Not all code needs equal coverage:

- **100% coverage**: Authentication, authorization, payment processing, security-critical code
- **80-90% coverage**: Business logic, data processing
- **50-70% coverage**: UI code, configuration parsing
- **0% coverage acceptable**: Generated code, vendored dependencies, truly trivial code

### 2. Test Behavior, Not Implementation

**Bad**:

```rust
#[test]
fn test_sort_uses_quicksort() {
    // Tests implementation detail
    let mut arr = vec![3, 1, 2];
    sort(&mut arr);
    // ... somehow verify quicksort was used
}
```

**Good**:

```rust
#[test]
fn test_sort_correctness() {
    // Tests behavior
    let mut arr = vec![3, 1, 2];
    sort(&mut arr);
    assert_eq!(arr, vec![1, 2, 3]);
}
```

Coverage should reflect behavioral tests, not implementation tests.

### 3. Measure Trend, Not Just Snapshot

Track coverage over time:

```bash
# Log coverage daily
echo "$(date),$(cargo llvm-cov report --summary-only | grep -oP '\d+\.\d+(?=%)')" >> coverage.csv
```

If coverage trends downward, intervene:

```
Week 1: 85%  ✅
Week 2: 83%  ⚠️
Week 3: 79%  ❌  (below threshold)
```

### 4. Use Coverage to Find Gaps, Not Drive Development

**Bad approach**: "We need 80% coverage, so let's write tests until we hit it."

**Good approach**: "Let's test all critical functionality. Coverage will tell us what we missed."

Coverage is a diagnostic tool, not a goal.

### 5. Combine with Other Metrics

Coverage alone is insufficient. Combine with:

- **Mutation testing**: Do tests detect bugs when code is changed?
- **Complexity**: Are complex functions tested thoroughly?
- **TDG**: Is overall code quality maintained?

## Coverage Exceptions

Some code is legitimately hard to test:

### 1. Platform-Specific Code

```rust
#[cfg(target_os = "linux")]
fn linux_specific() {
    // Can only test on Linux
}
```

Solution: Test on multiple platforms in CI, or use mocks.

### 2. Initialization Code

```rust
fn main() {
    // Hard to test main() directly
    let runtime = tokio::runtime::Runtime::new().unwrap();
    runtime.block_on(async { run_server().await });
}
```

Solution: Extract logic into testable functions. Keep `main()` minimal.

### 3. External Dependencies

```rust
fn fetch_from_api(url: &str) -> Result<Data> {
    // Relies on external API
    let response = reqwest::blocking::get(url)?;
    // ...
}
```

Solution: Use mocks or integration tests with test servers.

### 4. Compile-Time Configuration

```rust
#[cfg(feature = "encryption")]
fn encrypt(data: &[u8]) -> Vec<u8> {
    // Only compiled with "encryption" feature
}
```

Solution: Test with all feature combinations in CI.

## Summary

Code coverage is a powerful diagnostic tool that reveals what your tests actually test. pforge requires ≥80% line coverage to ensure critical code paths are exercised.

**Key takeaways**:

1. **Coverage types**: Line, branch, function, statement
2. **pforge thresholds**: ≥80% line coverage, ≥75% branch coverage
3. **Measure with**: `cargo llvm-cov` or `make coverage`
4. **Interpret reports**: Focus on uncovered critical paths, not just percentages
5. **Improve coverage**: Test error paths, all branches, match arms
6. **Avoid anti-patterns**: Coverage gaming, testing trivial code, high coverage but low quality
7. **Best practices**: Focus on critical paths, test behavior not implementation, track trends

Coverage reveals gaps. Use it to find untested code, then write meaningful tests—not just to hit a number.

Quality is built in, not tested in. But coverage helps verify you've built it right.
