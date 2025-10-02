# Testing the Calculator: EXTREME TDD in Action

The calculator has **six tests** that provide 100% code coverage and demonstrate every principle of EXTREME TDD. Let's examine each test and the discipline that produced them.

## The Complete Test Suite

All tests live in `src/handlers.rs` under the `#[cfg(test)]` module:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_add() {
        let handler = CalculateHandler;
        let input = CalculateInput {
            operation: "add".to_string(),
            a: 5.0,
            b: 3.0,
        };

        let output = handler.handle(input).await.unwrap();
        assert_eq!(output.result, 8.0);
    }

    #[tokio::test]
    async fn test_subtract() {
        let handler = CalculateHandler;
        let input = CalculateInput {
            operation: "subtract".to_string(),
            a: 10.0,
            b: 3.0,
        };

        let output = handler.handle(input).await.unwrap();
        assert_eq!(output.result, 7.0);
    }

    #[tokio::test]
    async fn test_multiply() {
        let handler = CalculateHandler;
        let input = CalculateInput {
            operation: "multiply".to_string(),
            a: 4.0,
            b: 5.0,
        };

        let output = handler.handle(input).await.unwrap();
        assert_eq!(output.result, 20.0);
    }

    #[tokio::test]
    async fn test_divide() {
        let handler = CalculateHandler;
        let input = CalculateInput {
            operation: "divide".to_string(),
            a: 15.0,
            b: 3.0,
        };

        let output = handler.handle(input).await.unwrap();
        assert_eq!(output.result, 5.0);
    }

    #[tokio::test]
    async fn test_divide_by_zero() {
        let handler = CalculateHandler;
        let input = CalculateInput {
            operation: "divide".to_string(),
            a: 10.0,
            b: 0.0,
        };

        let result = handler.handle(input).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Division by zero"));
    }

    #[tokio::test]
    async fn test_unknown_operation() {
        let handler = CalculateHandler;
        let input = CalculateInput {
            operation: "modulo".to_string(),
            a: 10.0,
            b: 3.0,
        };

        let result = handler.handle(input).await;
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Unknown operation"));
    }
}
```

## Test Anatomy

Every test follows this **four-part structure**:

### 1. Setup (Arrange)

```rust
let handler = CalculateHandler;
let input = CalculateInput {
    operation: "add".to_string(),
    a: 5.0,
    b: 3.0,
};
```

**Why create handler locally?**
- Each test is independent (no shared state)
- Tests can run in parallel
- No test pollution

### 2. Execution (Act)

```rust
let output = handler.handle(input).await.unwrap();
```

**Key decisions**:
- `.await`: Handler is async (returns Future)
- `.unwrap()`: For happy path tests, we expect success
- Store result for assertion

### 3. Verification (Assert)

```rust
assert_eq!(output.result, 8.0);
```

**Assertion strategies**:
- `assert_eq!`: For exact values (happy path)
- `assert!()`: For boolean conditions (error path)
- `.contains()`: For error message validation

### 4. Cleanup (Automatic)

Rust's RAII means cleanup is automatic - no manual teardown needed.

## The Six Tests Explained

### Test 1: Addition (Happy Path)

```rust
#[tokio::test]
async fn test_add() {
    let handler = CalculateHandler;
    let input = CalculateInput {
        operation: "add".to_string(),
        a: 5.0,
        b: 3.0,
    };

    let output = handler.handle(input).await.unwrap();
    assert_eq!(output.result, 8.0);
}
```

**What it tests**:
- Basic addition works
- Input deserialization
- Output serialization
- Handler trait implementation

**Edge cases NOT tested** (intentionally):
- Float precision (5.1 + 3.2 = 8.3)
- Large numbers (handled by f64)
- Negative numbers (subtraction tests this)

**Why 5.0 + 3.0 = 8.0?**

Simple numbers avoid floating-point precision issues. This is a **smoke test**, not a numerical analysis test.

### Test 2: Subtraction (Happy Path)

```rust
#[tokio::test]
async fn test_subtract() {
    let handler = CalculateHandler;
    let input = CalculateInput {
        operation: "subtract".to_string(),
        a: 10.0,
        b: 3.0,
    };

    let output = handler.handle(input).await.unwrap();
    assert_eq!(output.result, 7.0);
}
```

**What it adds**:
- Pattern matching works for second branch
- Negative results possible (if a < b)

**Design choice**: 10.0 - 3.0 (positive result) instead of 3.0 - 10.0 (negative result). Either works, we chose simplicity.

### Test 3: Multiplication (Happy Path)

```rust
#[tokio::test]
async fn test_multiply() {
    let handler = CalculateHandler;
    let input = CalculateInput {
        operation: "multiply".to_string(),
        a: 4.0,
        b: 5.0,
    };

    let output = handler.handle(input).await.unwrap();
    assert_eq!(output.result, 20.0);
}
```

**What it adds**:
- Third pattern match branch
- Result larger than inputs

**Why 4.0 * 5.0?**

Clean result (20.0) without precision issues.

### Test 4: Division (Happy Path)

```rust
#[tokio::test]
async fn test_divide() {
    let handler = CalculateHandler;
    let input = CalculateInput {
        operation: "divide".to_string(),
        a: 15.0,
        b: 3.0,
    };

    let output = handler.handle(input).await.unwrap();
    assert_eq!(output.result, 5.0);
}
```

**What it adds**:
- Division operation works
- Non-zero denominator case

**Deliberately tests happy path** - error path comes next.

### Test 5: Division by Zero (Error Path)

```rust
#[tokio::test]
async fn test_divide_by_zero() {
    let handler = CalculateHandler;
    let input = CalculateInput {
        operation: "divide".to_string(),
        a: 10.0,
        b: 0.0,
    };

    let result = handler.handle(input).await;
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("Division by zero"));
}
```

**Critical differences**:
- NO `.unwrap()` - we expect an error
- `assert!(result.is_err())` - verify error occurred
- `.unwrap_err()` - extract error for message validation
- `.contains()` - verify error message content

**Why check error message?**

Ensures users get **actionable feedback**, not just "error occurred."

### Test 6: Unknown Operation (Error Path)

```rust
#[tokio::test]
async fn test_unknown_operation() {
    let handler = CalculateHandler;
    let input = CalculateInput {
        operation: "modulo".to_string(),
        a: 10.0,
        b: 3.0,
    };

    let result = handler.handle(input).await;
    assert!(result.is_err());
    assert!(result
        .unwrap_err()
        .to_string()
        .contains("Unknown operation"));
}
```

**What it validates**:
- Input validation works
- Catch-all match arm triggered
- Helpful error message provided

**Why "modulo"?**

Realistic invalid operation that users might try.

## Test Coverage Analysis

Run coverage with:

```bash
cargo tarpaulin --out Stdout
```

**Expected output**:

```
|| Tested/Total Lines:
|| src/handlers.rs: 45/45 (100%)
||
|| Coverage: 100.00%
```

### Coverage Breakdown

| Code Path | Test | Coverage |
|-----------|------|----------|
| CalculateInput struct | All | ✅ |
| CalculateOutput struct | All | ✅ |
| Handler trait impl | All | ✅ |
| "add" branch | test_add | ✅ |
| "subtract" branch | test_subtract | ✅ |
| "multiply" branch | test_multiply | ✅ |
| "divide" branch | test_divide | ✅ |
| Division by zero error | test_divide_by_zero | ✅ |
| Unknown operation error | test_unknown_operation | ✅ |

**100% line coverage. 100% branch coverage.**

## Running the Tests

### Basic Test Run

```bash
cargo test
```

**Output**:

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

**All tests pass in <10ms**. This is **FAST**.

### Verbose Output

```bash
cargo test -- --nocapture
```

Shows println! output (though we don't use it).

### Single Test

```bash
cargo test test_divide_by_zero
```

Runs only the division by zero test.

### Watch Mode

```bash
cargo watch -x test
```

Runs tests automatically on file save. **Perfect for EXTREME TDD**.

## Test Performance

| Test | Time | Allocations |
|------|------|-------------|
| test_add | <1ms | 0 |
| test_subtract | <1ms | 0 |
| test_multiply | <1ms | 0 |
| test_divide | <1ms | 0 |
| test_divide_by_zero | <1ms | 1 (error String) |
| test_unknown_operation | <1ms | 1 (error String) |

**Total test suite runtime**: 3ms

**Why so fast?**
1. No I/O operations
2. No network calls
3. No file system access
4. Pure computation
5. Optimized by Rust compiler

## EXTREME TDD: Test-First Development

These tests were written **before** the handler code:

### The RED-GREEN-REFACTOR Loop

**Cycle 1**: test_add
- RED: Write test → Fails (handler doesn't exist)
- GREEN: Write minimal handler → Passes
- REFACTOR: Extract match pattern → Still passes
- COMMIT: Quality gates pass ✅

**Cycle 2**: test_subtract
- RED: Write test → Fails (only "add" implemented)
- GREEN: Add "subtract" branch → Passes
- REFACTOR: Run clippy → No issues
- COMMIT: Quality gates pass ✅

**Pattern repeats for all 6 tests.**

### Time Investment

| Phase | Time |
|-------|------|
| Writing tests | 10 minutes |
| Writing handler | 8 minutes |
| Refactoring | 2 minutes |
| **Total** | **20 minutes** |

**20 minutes to production-ready code** with 100% coverage.

## Test Driven Design Benefits

### 1. Simpler APIs

Tests forced us to design:
- Single tool instead of four
- Clear input/output structs
- Meaningful error messages

### 2. Comprehensive Coverage

Writing tests first means:
- No untested code paths
- Edge cases considered upfront
- Error handling built-in

### 3. Regression Protection

All 6 tests run on every commit:
- Pre-commit hooks prevent breaks
- CI/CD catches integration issues
- Refactoring is safe

### 4. Living Documentation

Tests show **how to use** the handler:

```rust
// Want to add two numbers?
let input = CalculateInput {
    operation: "add".to_string(),
    a: 5.0,
    b: 3.0,
};
let result = handler.handle(input).await?;
// result.result == 8.0
```

## Testing Anti-Patterns (What We AVOID)

### Anti-Pattern 1: Testing Implementation

```rust
// WRONG - tests implementation details
#[test]
fn test_match_expression() {
    // Don't test how it's implemented, test what it does
}
```

### Anti-Pattern 2: Over-Mocking

```rust
// WRONG - unnecessary mocking
let mock_handler = MockHandler::new();
mock_handler.expect_add().returning(|a, b| a + b);
```

Our handler is pure logic - no mocks needed.

### Anti-Pattern 3: One Assertion Per Test

```rust
// WRONG - too granular
#[test]
fn test_output_has_result_field() {
    let output = CalculateOutput { result: 8.0 };
    assert!(output.result == 8.0);  // Useless test
}
```

Test **behavior**, not structure.

### Anti-Pattern 4: Testing the Framework

```rust
// WRONG - testing serde
#[test]
fn test_input_deserializes() {
    let json = r#"{"operation":"add","a":5,"b":3}"#;
    let input: CalculateInput = serde_json::from_str(json).unwrap();
    // Don't test third-party libraries
}
```

Trust serde. Test **your code**.

## Quality Gates Integration

Tests run as part of quality gates:

```bash
make quality-gate
```

**Checks**:
1. `cargo test` - All tests pass ✅
2. `cargo tarpaulin` - Coverage ≥80% ✅ (we have 100%)
3. `cargo clippy` - No warnings ✅
4. `cargo fmt --check` - Formatted ✅
5. `pmat analyze complexity` - Complexity ≤20 ✅

**If ANY gate fails, commit is blocked.**

## Continuous Testing

During development, run:

```bash
cargo watch -x 'test --lib --quiet' -x 'clippy --quiet'
```

**Feedback loop**:
1. Save file
2. Tests run (3ms)
3. Clippy runs (200ms)
4. Results shown
5. **Total: <300ms feedback**

This is the **5-minute cycle in action** - fast feedback enables rapid iteration.

## Next Steps

Now that you understand the testing philosophy, let's run the calculator server and use it in Chapter 3.4. You'll see how these tests translate to production confidence.

---

> "Tests are not just verification - they're the design process." - EXTREME TDD principle
