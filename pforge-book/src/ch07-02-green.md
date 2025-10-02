# GREEN: Minimum Code

The GREEN phase has one singular goal: make the test pass using the absolute minimum code necessary. You have 2 minutes. Nothing else matters—not elegance, not performance, not extensibility. Just make it GREEN.

## The Minimum Code Principle

"Minimum code" doesn't mean "bad code" or "throw quality out the window." It means **the simplest implementation that satisfies the test specification**.

### What Minimum Means

**Minimum** means:
- No extra features beyond what the test requires
- No "just in case" code
- No premature optimization
- No architectural patterns unless necessary
- Hard-coded values are acceptable if they make the test pass

**Minimum does NOT mean**:
- Skipping error handling required by the test
- Using `unwrap()` instead of proper error propagation
- Introducing compiler warnings
- Violating Rust safety rules

### Why Minimum First?

**Speed**: Get to GREEN fast. Every second you spend on cleverness is a second not spent on the next feature.

**Correctness**: Simple implementations are easier to verify. You can see at a glance if they match the test.

**Deferral**: Complex design emerges from refactoring multiple simple implementations, not from upfront architecture.

**Safety Net**: Once tests pass, you have a safety net for refactoring. You can make it better without fear of breaking it.

## The 2-Minute GREEN Budget

Two minutes to implement and verify:
- **0:00-1:30**: Write implementation
- **1:30-1:50**: Run test
- **1:50-2:00**: Verify GREEN (all tests pass)

If the test doesn't pass by 2:00, you have 3 more minutes (until 5:00) to either fix it or RESET.

## Example: GREEN Phase Walkthrough

Continuing from our RED phase `clamp` function example:

### Minute 2:00 - Begin GREEN Phase

We have a failing test:

```rust
#[test]
fn test_clamp_returns_min_when_below_range() {
    let result = clamp(5, 10, 20);
    assert_eq!(result, 10);
}
```

Error: `cannot find function 'clamp' in this scope`

### Minute 2:10 - Write Minimal Implementation

```rust
// src/lib.rs
pub fn clamp(value: i32, min: i32, max: i32) -> i32 {
    if value < min {
        return min;
    }
    value  // Return value for now
}
```

**Why this is minimum**:
- Only handles the case tested (value < min)
- Doesn't handle value > max (not tested yet)
- Doesn't handle value in range perfectly (but passes test)

### Minute 3:45 - Run Test

```bash
cargo test test_clamp_returns_min_when_below_range
```

Output:
```
test test_clamp_returns_min_when_below_range ... ok
```

GREEN! Test passes.

### Minute 4:00 - Enter REFACTOR Phase

We're GREEN ahead of schedule. Now we can refactor.

## Hard-Coding Is Acceptable

One of TDD's most controversial practices: hard-coding return values is acceptable in GREEN.

### The Hard-Coding Example

```rust
// RED: Test expects specific output
#[tokio::test]
async fn test_greet_returns_hello_world() {
    let handler = GreetHandler;
    let input = GreetInput {
        name: "World".to_string(),
    };

    let result = handler.handle(input).await.unwrap();

    assert_eq!(result.message, "Hello, World!");
}
```

```rust
// GREEN: Hard-coded return value
#[async_trait::async_trait]
impl Handler for GreetHandler {
    type Input = GreetInput;
    type Output = GreetOutput;
    type Error = Error;

    async fn handle(&self, _input: Self::Input) -> Result<Self::Output> {
        Ok(GreetOutput {
            message: "Hello, World!".to_string(),
        })
    }
}
```

**This makes the test pass**. It's valid GREEN code.

### Why Hard-Coding Is Acceptable

**Proves the test works**: If the hard-coded value makes the test pass, you know the test verifies behavior correctly.

**Forces more tests**: The hard-coded implementation is obviously incomplete. You must write more tests to drive out the real logic.

**Defers complexity**: You don't jump to complex string interpolation until tests demand it.

### When to Use Real Implementation

As soon as you write a second test that requires different behavior, hard-coding stops working:

```rust
// Second test
#[tokio::test]
async fn test_greet_returns_personalized_greeting() {
    let handler = GreetHandler;
    let input = GreetInput {
        name: "Alice".to_string(),
    };

    let result = handler.handle(input).await.unwrap();

    assert_eq!(result.message, "Hello, Alice!");
}
```

Now the hard-coded implementation fails. Time for real logic:

```rust
async fn handle(&self, input: Self::Input) -> Result<Self::Output> {
    Ok(GreetOutput {
        message: format!("Hello, {}!", input.name),
    })
}
```

This is the **rule of three**: Hard-code for one test, use real logic after two tests require different behavior.

## Minimum Implementation Patterns

### Pattern 1: Return Literal

Simplest possible—return a literal value:

```rust
// Test expects specific value
async fn handle(&self, _input: Self::Input) -> Result<Self::Output> {
    Ok(GreetOutput {
        message: "Hello, World!".to_string(),
    })
}
```

**When to use**: First test for a handler, specific expected value.

### Pattern 2: Pass Through Input

Return input directly or with minimal transformation:

```rust
// Test expects input echoed back
async fn handle(&self, input: Self::Input) -> Result<Self::Output> {
    Ok(EchoOutput {
        message: input.message,
    })
}
```

**When to use**: Echo, copy, or identity operations.

### Pattern 3: Conditional

Single if-statement for simple branching:

```rust
// Test expects validation
async fn handle(&self, input: Self::Input) -> Result<Self::Output> {
    if input.age < 0 {
        return Err(Error::Validation("Age cannot be negative".to_string()));
    }

    Ok(AgeOutput {
        category: "adult".to_string(),  // Hard-coded for now
    })
}
```

**When to use**: Validation, error cases, simple branching.

### Pattern 4: Simple Calculation

Direct calculation without helper functions:

```rust
// Test expects arithmetic
async fn handle(&self, input: Self::Input) -> Result<Self::Output> {
    Ok(AddOutput {
        sum: input.a + input.b,
    })
}
```

**When to use**: Arithmetic, string formatting, basic transformations.

### Pattern 5: Delegation

Call existing function or library:

```rust
// Test expects file reading
async fn handle(&self, input: Self::Input) -> Result<Self::Output> {
    let contents = tokio::fs::read_to_string(&input.path).await
        .map_err(|e| Error::Handler(e.to_string()))?;

    Ok(ReadOutput { contents })
}
```

**When to use**: File I/O, HTTP requests, database queries (real or mocked).

## Common GREEN Phase Mistakes

### Mistake 1: Over-Engineering

```rust
// BAD - Too complex for first test
async fn handle(&self, input: Self::Input) -> Result<Self::Output> {
    // Generic calculation engine
    let calculator = CalculatorBuilder::new()
        .with_operator(input.operator.parse()?)
        .with_precision(input.precision.unwrap_or(2))
        .with_rounding_mode(RoundingMode::HalfUp)
        .build()?;

    let result = calculator.compute(input.operands)?;

    Ok(CalculatorOutput { result })
}
```

**Why it's bad**: You've written 20 lines of infrastructure for a test that just needs `2 + 2 = 4`.

**Fix**: Start simple, add complexity when tests demand it:

```rust
// GOOD - Minimal for first test
async fn handle(&self, input: Self::Input) -> Result<Self::Output> {
    Ok(CalculatorOutput {
        result: input.a + input.b,
    })
}
```

When you need multiplication, add it:

```rust
async fn handle(&self, input: Self::Input) -> Result<Self::Output> {
    let result = match input.operator.as_str() {
        "+" => input.a + input.b,
        "*" => input.a * input.b,
        _ => return Err(Error::Validation("Unknown operator".to_string())),
    };

    Ok(CalculatorOutput { result })
}
```

### Mistake 2: Premature Optimization

```rust
// BAD - Optimizing before necessary
async fn handle(&self, input: Self::Input) -> Result<Self::Output> {
    // Pre-allocate with capacity
    let mut results = Vec::with_capacity(input.items.len());

    // Parallel processing
    let handles: Vec<_> = input.items
        .into_iter()
        .map(|item| tokio::spawn(async move { process(item) }))
        .collect();

    for handle in handles {
        results.push(handle.await??);
    }

    Ok(Output { results })
}
```

**Why it's bad**: You're optimizing before knowing if there's a performance problem. This adds complexity and time.

**Fix**: Start sequential, optimize when benchmarks show a problem:

```rust
// GOOD - Simple sequential processing
async fn handle(&self, input: Self::Input) -> Result<Self::Output> {
    let mut results = Vec::new();

    for item in input.items {
        results.push(process(item).await?);
    }

    Ok(Output { results })
}
```

### Mistake 3: Adding Untested Features

```rust
// BAD - Features not required by test
async fn handle(&self, input: Self::Input) -> Result<Self::Output> {
    // Test only requires division
    let quotient = input.numerator / input.denominator;

    // But we're also adding:
    let remainder = input.numerator % input.denominator;
    let is_exact = remainder == 0.0;
    let sign = if quotient < 0.0 { -1 } else { 1 };

    Ok(DivideOutput {
        quotient,
        remainder,      // Not tested
        is_exact,       // Not tested
        sign,           // Not tested
    })
}
```

**Why it's bad**: Untested code is unverified code. It might have bugs. It definitely wastes time.

**Fix**: Only implement what tests require:

```rust
// GOOD - Only what the test needs
async fn handle(&self, input: Self::Input) -> Result<Self::Output> {
    Ok(DivideOutput {
        quotient: input.numerator / input.denominator,
    })
}
```

If you need remainder later, a test will drive it out.

### Mistake 4: Skipping Error Handling

```rust
// BAD - Using unwrap() instead of proper error handling
async fn handle(&self, input: Self::Input) -> Result<Self::Output> {
    let file = tokio::fs::read_to_string(&input.path).await.unwrap();
    Ok(ReadOutput { contents: file })
}
```

**Why it's bad**: This violates pforge quality standards. `unwrap()` causes panics in production.

**Fix**: Proper error propagation:

```rust
// GOOD - Proper error handling
async fn handle(&self, input: Self::Input) -> Result<Self::Output> {
    let file = tokio::fs::read_to_string(&input.path).await
        .map_err(|e| Error::Handler(format!("Failed to read file: {}", e)))?;

    Ok(ReadOutput { contents: file })
}
```

The `?` operator and `.map_err()` are just as fast to type as `.unwrap()`.

## Type-Driven GREEN

Rust's type system guides you toward correct implementations:

### Follow the Types

```rust
// You have: input: DivideInput
// You need: Result<DivideOutput>

// Types guide you:
async fn handle(&self, input: Self::Input) -> Result<Self::Output> {
    // input has: numerator (f64), denominator (f64)
    // Output needs: quotient (f64)

    // Types tell you: divide numerator by denominator
    let quotient = input.numerator / input.denominator;

    // Wrap in Output struct
    Ok(DivideOutput { quotient })
}
```

**Follow the types from input to output**. The compiler tells you what's needed.

### Let Compiler Guide You

When the compiler complains, listen:

```
error[E0308]: mismatched types
  --> src/handlers/calculate.rs:15:12
   |
15 |         Ok(quotient)
   |            ^^^^^^^^ expected struct `DivideOutput`, found `f64`
```

Compiler says: "You returned `f64`, but function expects `DivideOutput`."

Fix:

```rust
Ok(DivideOutput { quotient })
```

The compiler is your pair programmer during GREEN.

## Testing Your GREEN Implementation

After writing implementation, verify GREEN:

```bash
# Run the specific test
cargo test test_divide_returns_quotient

# Expected output:
# test test_divide_returns_quotient ... ok
```

If test fails, you have 3 options:

### Option 1: Quick Fix (Under 30 Seconds)

Typo or minor mistake:

```rust
// Wrong
Ok(DivideOutput { quotient: input.numerator * input.denominator })

// Fixed
Ok(DivideOutput { quotient: input.numerator / input.denominator })
```

If you can spot and fix in < 30 seconds, do it.

### Option 2: Continue to REFACTOR (Test Passes)

Test passes? Move to REFACTOR phase even if implementation feels ugly. You'll clean it up next.

### Option 3: RESET (Can't Fix Before 5:00)

If you're at 4:30 and tests still fail with no clear fix, RESET:

```bash
git checkout .
```

Reflect: What went wrong?
- Implementation more complex than expected → Break into smaller tests
- Wrong algorithm → Research before next cycle
- Missing dependencies → Add to setup before next cycle

## GREEN + Quality Gates

Even in GREEN phase, pforge quality standards apply:

### Must Pass:
- **Compilation**: Code must compile
- **No warnings**: Zero compiler warnings
- **No unwrap()**: Proper error handling
- **No panic!()**: Return errors, don't panic

### Deferred to REFACTOR:
- **Clippy lints**: Fix in REFACTOR
- **Formatting**: Auto-format in REFACTOR
- **Complexity**: Simplify in REFACTOR
- **Duplication**: Extract in REFACTOR

The line: GREEN code must be **correct** but not necessarily **clean**.

## Example: Full GREEN Phase

Let's implement division with error handling.

### Test (From RED Phase)

```rust
#[tokio::test]
async fn test_divide_handles_zero_denominator() {
    let handler = DivideHandler;
    let input = DivideInput {
        numerator: 10.0,
        denominator: 0.0,
    };

    let result = handler.handle(input).await;

    assert!(result.is_err());
    match result.unwrap_err() {
        Error::Validation(msg) => {
            assert!(msg.contains("Division by zero"));
        }
        _ => panic!("Wrong error type"),
    }
}
```

### Minute 2:00 - Begin GREEN

Current implementation:

```rust
async fn handle(&self, input: Self::Input) -> Result<Self::Output> {
    Ok(DivideOutput {
        quotient: input.numerator / input.denominator,
    })
}
```

Test fails: no division-by-zero check.

### Minute 2:10 - Add Zero Check

```rust
async fn handle(&self, input: Self::Input) -> Result<Self::Output> {
    if input.denominator == 0.0 {
        return Err(Error::Validation(
            "Division by zero: denominator must be non-zero".to_string()
        ));
    }

    Ok(DivideOutput {
        quotient: input.numerator / input.denominator,
    })
}
```

### Minute 3:40 - Test Passes

```bash
cargo test test_divide_handles_zero_denominator
# test test_divide_handles_zero_denominator ... ok
```

GREEN!

### Minute 4:00 - Enter REFACTOR

We have a working, tested implementation. Now we can refactor.

## Minimum vs. Simplest

There's a subtle but important distinction:

**Minimum**: Least code to pass the test
**Simplest**: Easiest to understand

Usually they're the same, but sometimes minimum is *less* simple:

```rust
// Minimum (hard-coded)
async fn handle(&self, _input: Self::Input) -> Result<Self::Output> {
    Ok(Output { value: 42 })
}

// Simplest (obvious logic)
async fn handle(&self, input: Self::Input) -> Result<Self::Output> {
    Ok(Output { value: input.a + input.b })
}
```

If the simplest implementation is just as fast to write, prefer it over minimum. But if simplest requires significant design, stick with minimum and let tests drive out the design.

## When GREEN Takes Longer Than 2 Minutes

If you reach minute 4:00 (2 minutes into GREEN) and tests don't pass:

### You Have 1 Minute Left

Use it to either:
1. Fix the implementation
2. Debug the failure
3. Decide to RESET

### Don't Rush

Rushing leads to mistakes. Better to RESET and start clean than to force broken code through quality gates.

### Common Reasons for Slow GREEN

**Algorithm complexity**: Chose complex approach. Next cycle, try simpler algorithm.

**Missing knowledge**: Don't know how to implement. Research before next cycle.

**Wrong abstraction**: Fighting the types. Rethink approach.

**Test too large**: Test requires too much code. Break into smaller tests.

## GREEN Phase Checklist

Before moving to REFACTOR:

- [ ] Test passes (verify by running)
- [ ] All existing tests still pass (no regressions)
- [ ] Code compiles without warnings
- [ ] No `unwrap()` or `panic!()` in production code
- [ ] Proper error handling for error cases
- [ ] Timer shows less than 4:00 elapsed

If any item is unchecked and you can't fix in 1 minute, RESET.

## The Joy of GREEN

There's a dopamine hit when tests turn green:

```
test test_divide_returns_quotient ... ok
```

That "ok" is immediate positive feedback. You've made progress. The feature works.

TDD's tight feedback loop (minutes, not hours) creates frequent positive reinforcement, which:
- Maintains motivation
- Builds momentum
- Reduces stress
- Makes coding addictive (in a good way)

## Next Phase: REFACTOR

You have working code. Tests pass. Now you have 1 minute to make it clean.

REFACTOR is where you transform minimum code into maintainable code, with the safety net of passing tests.

---

Previous: [RED: Write Failing Test](ch07-01-red.md)
Next: [REFACTOR: Clean Up](ch07-03-refactor.md)
