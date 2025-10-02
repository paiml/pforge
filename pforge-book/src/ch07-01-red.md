# RED: Write Failing Test

The RED phase is where you define what success looks like before writing any production code. You have exactly 2 minutes to write a failing test that clearly specifies the next increment of behavior.

## The Purpose of RED

RED is about **specification, not testing**. The test you write answers the question: "What should the next tiny piece of functionality do?"

### Why Tests Come First

**Design Pressure**: Writing tests first forces you to think from the caller's perspective. You design interfaces that are pleasant to use, not convenient to implement.

**Clear Goal**: Before writing implementation, you have a concrete, executable definition of "done." The test passes = you're finished.

**Prevents Scope Creep**: Writing tests first forces you to commit to a small scope before getting distracted by implementation details.

**Living Documentation**: Tests document intent better than comments. Comments lie; tests are executable and must stay accurate.

## The 2-Minute Budget

Two minutes to write a test feels tight. It is. This constraint forces several good practices:

**Small Increments**: If you can't write a test in 2 minutes, your increment is too large. Break it down.

**Test Template Reuse**: You'll develop a library of test patterns that you can copy and adapt quickly.

**No Overthinking**: Two minutes prevents analysis paralysis. Write the simplest test that fails for the right reason.

## Anatomy of a Good RED Test

A good RED test has three characteristics:

### 1. Compiles (If Possible)

In typed languages like Rust, the test should compile even if types don't exist yet. Use comments or temporary stubs:

```rust
// COMPILES - Types exist
#[tokio::test]
async fn test_greet_returns_greeting() {
    let handler = GreetHandler;
    let input = GreetInput {
        name: "Alice".to_string(),
    };

    let result = handler.handle(input).await;

    assert!(result.is_ok());
}
```

If types don't exist:

```rust
// DOESN'T COMPILE YET - Types will be created in GREEN
#[tokio::test]
async fn test_divide_handles_zero() {
    let handler = DivideHandler;
    let input = DivideInput {
        numerator: 10.0,
        denominator: 0.0,
    };

    let result = handler.handle(input).await;

    assert!(result.is_err());
    // Will be: Error::Validation("Division by zero")
}
```

Both are valid RED tests. The first runs and fails (returns wrong value). The second doesn't compile (types missing). Either way, you're RED.

### 2. Fails for the Right Reason

The test must fail because the feature doesn't exist, not because of typos or wrong imports:

```rust
// GOOD - Fails because feature missing
#[tokio::test]
async fn test_calculate_mean() {
    let handler = StatisticsHandler;
    let input = StatsInput {
        data: vec![1.0, 2.0, 3.0, 4.0, 5.0],
    };

    let result = handler.handle(input).await.unwrap();

    assert_eq!(result.mean, 3.0);
}
// Fails: field `mean` does not exist in `StatsOutput`
```

```rust
// BAD - Fails because of typo
#[tokio::test]
async fn test_calculate_mean() {
    let handler = StatisticsHander;  // typo!
    // ...
}
// Fails: cannot find struct `StatisticsHander`
```

Run your test immediately after writing it to verify it fails correctly.

### 3. Tests One Thing

Each test should verify one specific behavior:

```rust
// GOOD - One behavior
#[tokio::test]
async fn test_divide_returns_quotient() {
    let handler = DivideHandler;
    let input = DivideInput {
        numerator: 10.0,
        denominator: 2.0,
    };

    let result = handler.handle(input).await.unwrap();

    assert_eq!(result.quotient, 5.0);
}

// GOOD - Different behavior, separate test
#[tokio::test]
async fn test_divide_rejects_zero_denominator() {
    let handler = DivideHandler;
    let input = DivideInput {
        numerator: 10.0,
        denominator: 0.0,
    };

    let result = handler.handle(input).await;

    assert!(result.is_err());
}
```

```rust
// BAD - Multiple behaviors in one test
#[tokio::test]
async fn test_divide_everything() {
    // Tests division
    let result1 = handler.handle(DivideInput { ... }).await.unwrap();
    assert_eq!(result1.quotient, 5.0);

    // Tests zero handling
    let result2 = handler.handle(DivideInput { denominator: 0.0, ... }).await;
    assert!(result2.is_err());

    // Tests negative numbers
    let result3 = handler.handle(DivideInput { numerator: -10.0, ... }).await.unwrap();
    assert_eq!(result3.quotient, -5.0);
}
```

Multiple assertions are fine if they verify the same behavior. Multiple behaviors require separate tests.

## Test Naming Conventions

Test names should read as specifications:

```rust
// GOOD - Reads as specification
test_greet_returns_personalized_message()
test_divide_rejects_zero_denominator()
test_statistics_calculates_mean_correctly()
test_file_read_handles_missing_file()
test_http_call_retries_on_timeout()

// BAD - Vague or implementation-focused
test_greet()
test_division()
test_math_works()
test_error_case()
test_function_1()
```

Pattern: `test_<subject>_<behavior>_<condition>`

Examples:
- `test_calculator_adds_positive_numbers`
- `test_file_handler_creates_missing_directory`
- `test_api_client_refreshes_expired_token`

## Quick Test Templates for pforge

### Handler Happy Path Template

```rust
#[tokio::test]
async fn test_HANDLER_NAME_returns_OUTPUT() {
    let handler = HandlerStruct;
    let input = InputStruct {
        field: value,
    };

    let result = handler.handle(input).await;

    assert!(result.is_ok());
    let output = result.unwrap();
    assert_eq!(output.field, expected_value);
}
```

### Handler Error Case Template

```rust
#[tokio::test]
async fn test_HANDLER_NAME_rejects_INVALID_INPUT() {
    let handler = HandlerStruct;
    let input = InputStruct {
        field: invalid_value,
    };

    let result = handler.handle(input).await;

    assert!(result.is_err());
    match result.unwrap_err() {
        Error::Validation(msg) => assert!(msg.contains("expected error substring")),
        _ => panic!("Wrong error type"),
    }
}
```

### Handler Async Operation Template

```rust
#[tokio::test]
async fn test_HANDLER_NAME_completes_within_timeout() {
    let handler = HandlerStruct;
    let input = InputStruct { /* ... */ };

    let timeout_duration = std::time::Duration::from_secs(5);

    let result = tokio::time::timeout(
        timeout_duration,
        handler.handle(input)
    ).await;

    assert!(result.is_ok(), "Handler timed out");
    assert!(result.unwrap().is_ok());
}
```

Copy these templates, replace the placeholders, and you have a test in under 2 minutes.

## The RED Checklist

Before moving to GREEN, verify:

- [ ] Test compiles OR fails to compile for the right reason (missing types)
- [ ] Test runs and fails OR doesn't compile
- [ ] Test name clearly describes the behavior being specified
- [ ] Test is focused on one specific behavior
- [ ] Timer shows less than 2:00 minutes elapsed

If any item is unchecked, refine the test. If the timer exceeds 2:00, RESET.

## Common RED Phase Mistakes

### Mistake 1: Testing Too Much at Once

```rust
// BAD - Too much for one test
#[tokio::test]
async fn test_calculator_all_operations() {
    // Addition
    assert_eq!(calc.add(2, 3).await.unwrap(), 5);

    // Subtraction
    assert_eq!(calc.subtract(5, 3).await.unwrap(), 2);

    // Multiplication
    assert_eq!(calc.multiply(2, 3).await.unwrap(), 6);

    // Division
    assert_eq!(calc.divide(6, 3).await.unwrap(), 2);
}
```

**Why it's bad**: If this test fails, you don't know which operation broke. Also, implementing all four operations takes more than 2 minutes (GREEN phase).

**Fix**: One test per operation.

### Mistake 2: Testing Implementation Details

```rust
// BAD - Tests internal structure
#[tokio::test]
async fn test_handler_uses_hashmap_internally() {
    let handler = CacheHandler::new();
    // Somehow peek into internals
    assert!(handler.storage.is_hashmap());
}
```

**Why it's bad**: Tests should verify behavior, not implementation. If you refactor from HashMap to BTreeMap, this test breaks even though behavior is unchanged.

**Fix**: Test observable behavior only.

```rust
// GOOD - Tests behavior
#[tokio::test]
async fn test_cache_retrieves_stored_value() {
    let handler = CacheHandler::new();

    handler.store("key", "value").await.unwrap();
    let result = handler.retrieve("key").await.unwrap();

    assert_eq!(result, "value");
}
```

### Mistake 3: Complex Test Setup

```rust
// BAD - Setup takes too long
#[tokio::test]
async fn test_user_registration() {
    // Too much setup
    let db = setup_test_database().await;
    let email_service = MockEmailService::new();
    let password_hasher = Argon2::default();
    let config = load_test_config("config.yaml");
    let logger = setup_test_logger();
    let handler = RegistrationHandler::new(db, email_service, password_hasher, config, logger);

    // Test starts here...
}
```

**Why it's bad**: You've exceeded 2 minutes just on setup. The test hasn't even run yet.

**Fix**: Extract setup to a helper function or use test fixtures:

```rust
// GOOD - Fast setup
#[tokio::test]
async fn test_user_registration() {
    let handler = create_test_registration_handler().await;

    let input = RegistrationInput {
        email: "test@example.com".to_string(),
        password: "securepass123".to_string(),
    };

    let result = handler.handle(input).await;

    assert!(result.is_ok());
}

// Helper function defined once, reused many times
async fn create_test_registration_handler() -> RegistrationHandler {
    let db = setup_test_database().await;
    let email_service = MockEmailService::new();
    // ... etc
    RegistrationHandler::new(db, email_service, /* ... */)
}
```

### Mistake 4: Not Running the Test

**Symptom**: You write a test, assume it fails correctly, and move to GREEN.

**Why it's bad**: The test might already pass (making it useless), or fail for the wrong reason (typo, wrong import).

**Fix**: Always run the test immediately and verify the failure message:

```bash
# After writing test
cargo test test_divide_returns_quotient
# Expected: Test failed (function not implemented)
# If: Test passed → test is useless
# If: Test failed (wrong reason) → fix test first
```

## Advanced RED Techniques

### Outside-In TDD

Start with high-level behavior, let tests drive lower-level design:

```rust
// Minute 0:00 - High-level test
#[tokio::test]
async fn test_api_returns_user_profile() {
    let api = UserAPI::new();

    let result = api.get_profile("user123").await;

    assert!(result.is_ok());
    let profile = result.unwrap();
    assert_eq!(profile.username, "alice");
}
```

This test will drive the creation of:
- `UserAPI` struct
- `get_profile` method
- `Profile` struct
- Database layer (in later cycles)

### Property-Based Testing Hint

For complex logic, use RED to specify properties:

```rust
// Standard example-based test
#[tokio::test]
async fn test_sort_orders_numbers() {
    let input = vec![3, 1, 4, 1, 5];
    let result = sort(input).await;
    assert_eq!(result, vec![1, 1, 3, 4, 5]);
}

// Property-based test (RED phase)
#[tokio::test]
async fn test_sort_maintains_length() {
    use proptest::prelude::*;

    proptest!(|(numbers: Vec<i32>)| {
        let sorted = sort(numbers.clone()).await;
        prop_assert_eq!(sorted.len(), numbers.len());
    });
}
```

Property tests specify invariants rather than specific examples.

### Test-Driven Error Messages

Write the test with the error message you want users to see:

```rust
#[tokio::test]
async fn test_divide_provides_helpful_error_message() {
    let handler = DivideHandler;
    let input = DivideInput {
        numerator: 10.0,
        denominator: 0.0,
    };

    let result = handler.handle(input).await;

    assert!(result.is_err());
    let error = result.unwrap_err();
    let message = format!("{}", error);

    // Specify the exact error message you want
    assert!(message.contains("Division by zero"));
    assert!(message.contains("denominator must be non-zero"));
}
```

This drives you to write good error messages, not generic "An error occurred."

## Integration with pforge Watch Mode

Run tests continuously during RED phase:

```bash
# Terminal 1: Start watch mode
cargo watch -x 'test test_divide_returns_quotient --lib'

# Terminal 2: Edit test
vim crates/pforge-runtime/tests/unit/calculator_test.rs
```

Watch mode gives instant feedback. Save the file, see the failure, confirm it's RED for the right reason.

## RED Phase Workflow Summary

1. **Start timer** (5-minute cycle begins)
2. **Open test file** (under 10 seconds)
3. **Copy test template** (under 20 seconds)
4. **Fill in specifics** (under 60 seconds)
5. **Run test** (under 10 seconds)
6. **Verify failure** (under 20 seconds)
7. **Total**: ~2 minutes

With practice, you'll complete RED in 90 seconds consistently, giving extra time for GREEN and REFACTOR.

## Example: RED Phase Executed Correctly

Let's implement a `clamp` function that constrains a value between min and max.

### Minute 0:00 - Start Timer

```bash
termdown 5m &
vim crates/pforge-runtime/src/lib.rs
```

### Minute 0:10 - Decide on Test

Feature: Clamp function for numbers
Test: Value below min returns min

### Minute 0:20 - Open Test File

```bash
vim crates/pforge-runtime/tests/unit/math_test.rs
```

### Minute 0:30 - Write Test

```rust
#[test]
fn test_clamp_returns_min_when_below_range() {
    let result = clamp(5, 10, 20);
    assert_eq!(result, 10);
}
```

### Minute 0:50 - Run Test

```bash
cargo test test_clamp_returns_min_when_below_range
```

Output:
```
error: cannot find function `clamp` in this scope
```

### Minute 1:00 - Verify RED

Perfect! Test fails because function doesn't exist. This is the right failure.

### Minute 1:10 - Document in Test

```rust
#[test]
fn test_clamp_returns_min_when_below_range() {
    // clamp(value, min, max) constrains value to [min, max]
    let result = clamp(5, 10, 20);
    assert_eq!(result, 10);
}
```

### Minute 2:00 - RED Phase Complete

We have:
- ✅ Test written
- ✅ Test fails for right reason
- ✅ Behavior clearly specified
- ✅ Under 2-minute budget

Time to move to GREEN.

## When RED Takes Longer Than 2 Minutes

If you hit 2:00 and the test isn't ready, you have two options:

### Option 1: Finish Quickly (If < 30 Seconds Remaining)

If you're truly close (just need to add assertions), finish quickly:

```rust
// 1:50 elapsed, just need to add:
assert_eq!(result.value, expected);
// Total: 2:05 - acceptable
```

Minor overruns (< 15 seconds) are acceptable if test is complete and verified RED.

### Option 2: RESET (If Significantly Over)

If you're at 2:30 and still writing the test, RESET:

```bash
git checkout .
```

Reflect: Why did RED take so long?
- Test setup too complex → Need helper function
- Testing too much → Break into smaller tests
- Unclear what to test → Spend 1 minute planning before next cycle

## RED Phase Success Metrics

Track these metrics to improve:

**Time to RED**: Average time to write failing test
- Target: < 2:00
- Excellent: < 1:30
- Expert: < 1:00

**RED Failure Rate**: Tests that fail for wrong reason
- Target: < 10%
- Excellent: < 5%
- Expert: < 1%

**RED Rewrites**: Tests rewritten during same cycle
- Target: < 20%
- Excellent: < 10%
- Expert: < 5%

## Psychological Benefits of RED First

**Confidence**: You know what you're building before you start.

**Clarity**: The test clarifies vague requirements into concrete behavior.

**Progress**: Each RED test is a small, achievable goal.

**Safety Net**: Tests catch regressions as you refactor later.

**Documentation**: Future developers understand intent from tests.

## Next Phase: GREEN

You've written a failing test that specifies behavior. Now it's time to make it pass with the minimum code necessary.

The GREEN phase has one goal: get from RED to GREEN as fast as possible, even if the implementation is ugly. We'll clean it up in REFACTOR.

---

Previous: [The 5-Minute TDD Cycle](ch07-00-five-minute-cycle.md)
Next: [GREEN: Minimum Code](ch07-02-green.md)
