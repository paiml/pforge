# REFACTOR: Clean Up

You have working code. Tests pass. Now you have exactly 1 minute to make it clean. REFACTOR is where minimum code becomes maintainable code, all while protected by your test suite.

## The Purpose of REFACTOR

REFACTOR transforms code from "works" to "works well." You're not adding features—you're improving the structure, readability, and maintainability of existing code.

### Why Refactor Matters

**Technical Debt Prevention**: Without regular refactoring, each cycle adds a little cruft. After 100 cycles, the codebase is unmaintainable.

**Code Comprehension**: Future you (next week) needs to understand current you's code. Clear code reduces cognitive load.

**Change Velocity**: Clean code is easier to modify. Refactoring now saves time in future cycles.

**Bug Prevention**: Clearer code has fewer hiding places for bugs.

## The 1-Minute Budget

You have 1 minute for REFACTOR. This forces discipline:

**Only Obvious Improvements**: If it takes more than 1 minute to refactor, defer it to a dedicated refactoring cycle.

**Safe Changes Only**: You don't have time to debug complex refactorings. Stick to automated refactorings and obvious simplifications.

**Keep Tests Green**: After each refactoring step, tests must still pass. If they don't, revert immediately.

### Time Breakdown

- **0:00-0:30**: Identify improvements (duplication, naming, complexity)
- **0:30-0:50**: Apply refactorings
- **0:50-1:00**: Re-run tests, verify still GREEN

## Common Refactorings That Fit in 1 Minute

### Refactoring 1: Extract Variable

Before:

```rust
async fn handle(&self, input: Self::Input) -> Result<Self::Output> {
    if input.age < 0 || input.age > 120 {
        return Err(Error::Validation("Invalid age".to_string()));
    }

    Ok(AgeOutput {
        category: if input.age < 13 { "child" } else if input.age < 20 { "teenager" } else { "adult" }.to_string(),
    })
}
```

After:

```rust
async fn handle(&self, input: Self::Input) -> Result<Self::Output> {
    if input.age < 0 || input.age > 120 {
        return Err(Error::Validation("Invalid age".to_string()));
    }

    let category = if input.age < 13 {
        "child"
    } else if input.age < 20 {
        "teenager"
    } else {
        "adult"
    };

    Ok(AgeOutput {
        category: category.to_string(),
    })
}
```

**Why**: Extracts complex expression into named variable, improving readability.

**Time**: 15 seconds

### Refactoring 2: Improve Naming

Before:

```rust
async fn handle(&self, input: Self::Input) -> Result<Self::Output> {
    let x = input.a + input.b;
    let y = x * 2;
    let z = y - 10;

    Ok(Output { result: z })
}
```

After:

```rust
async fn handle(&self, input: Self::Input) -> Result<Self::Output> {
    let sum = input.a + input.b;
    let doubled = sum * 2;
    let adjusted = doubled - 10;

    Ok(Output { result: adjusted })
}
```

**Why**: Descriptive names make code self-documenting.

**Time**: 20 seconds

### Refactoring 3: Extract Constant

Before:

```rust
async fn handle(&self, input: Self::Input) -> Result<Self::Output> {
    if input.temperature > 100 {
        return Err(Error::Validation("Temperature too high".to_string()));
    }

    if input.temperature < -273 {
        return Err(Error::Validation("Temperature too low".to_string()));
    }

    Ok(TemperatureOutput { celsius: input.temperature })
}
```

After:

```rust
const BOILING_POINT_CELSIUS: f64 = 100.0;
const ABSOLUTE_ZERO_CELSIUS: f64 = -273.15;

async fn handle(&self, input: Self::Input) -> Result<Self::Output> {
    if input.temperature > BOILING_POINT_CELSIUS {
        return Err(Error::Validation("Temperature too high".to_string()));
    }

    if input.temperature < ABSOLUTE_ZERO_CELSIUS {
        return Err(Error::Validation("Temperature too low".to_string()));
    }

    Ok(TemperatureOutput { celsius: input.temperature })
}
```

**Why**: Magic numbers become named constants with semantic meaning.

**Time**: 25 seconds

### Refactoring 4: Simplify Conditional

Before:

```rust
async fn handle(&self, input: Self::Input) -> Result<Self::Output> {
    let is_valid = if input.value >= 0 && input.value <= 100 {
        true
    } else {
        false
    };

    if !is_valid {
        return Err(Error::Validation("Value out of range".to_string()));
    }

    Ok(Output { value: input.value })
}
```

After:

```rust
async fn handle(&self, input: Self::Input) -> Result<Self::Output> {
    if input.value < 0 || input.value > 100 {
        return Err(Error::Validation("Value out of range".to_string()));
    }

    Ok(Output { value: input.value })
}
```

**Why**: Removes unnecessary boolean variable and inverted logic.

**Time**: 15 seconds

### Refactoring 5: Use Rust Idioms

Before:

```rust
async fn handle(&self, input: Self::Input) -> Result<Self::Output> {
    let mut result = Vec::new();

    for item in input.items {
        let processed = item * 2;
        result.push(processed);
    }

    Ok(Output { items: result })
}
```

After:

```rust
async fn handle(&self, input: Self::Input) -> Result<Self::Output> {
    let items = input.items
        .into_iter()
        .map(|item| item * 2)
        .collect();

    Ok(Output { items })
}
```

**Why**: Idiomatic Rust uses iterators, which are more concise and often faster.

**Time**: 20 seconds

### Refactoring 6: Auto-Format

Always run auto-formatter:

```bash
cargo fmt
```

This instantly fixes:
- Indentation
- Spacing
- Line breaks
- Brace alignment

**Time**: 5 seconds (automated)

## Refactorings That DON'T Fit in 1 Minute

Some refactorings are too complex for the 1-minute window. Defer these to dedicated refactoring cycles:

### Extract Function

```rust
// Complex function that needs extraction
async fn handle(&self, input: Self::Input) -> Result<Self::Output> {
    // 50 lines of complex logic
    // Would take 3-5 minutes to extract safely
}
```

**Why defer**: Extracting requires:
- Identifying the right boundary
- Determining parameters
- Updating all call sites
- Writing tests for new function

This takes > 1 minute. Create a dedicated refactoring cycle.

### Restructure Data

```rust
// Changing struct layout
pub struct User {
    pub name: String,
    pub age: i32,
}

// Want to change to:
pub struct User {
    pub profile: Profile,
}

pub struct Profile {
    pub name: String,
    pub age: i32,
}
```

**Why defer**: Ripple effects across codebase. Needs multiple cycles.

### Change Architecture

```rust
// Moving from direct DB access to repository pattern
// This touches many files and requires careful coordination
```

**Why defer**: Architectural changes need planning and multiple refactoring cycles.

## The Refactoring Checklist

Before finishing REFACTOR phase:

- [ ] Code formatted (`cargo fmt`)
- [ ] No clippy warnings (`cargo clippy`)
- [ ] No duplication within function
- [ ] Variable names are descriptive
- [ ] Constants extracted for magic numbers
- [ ] All tests still pass (`cargo test`)
- [ ] Timer shows less than 5:00 elapsed

## Example: Complete REFACTOR Phase

Let's refactor our division handler.

### Minute 4:00 - Begin REFACTOR

Current code (from GREEN phase):

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

### Minute 4:10 - Identify Improvements

Scan for issues:
- ✓ No duplication
- ✓ Names are clear
- ✓ Logic is simple
- ✓ Error message is helpful

This code is already clean! No refactoring needed.

### Minute 4:15 - Run Formatter and Clippy

```bash
cargo fmt
cargo clippy --quiet
```

Output: No warnings.

### Minute 4:20 - Verify Tests Still Pass

```bash
cargo test --lib --quiet
```

All tests pass.

### Minute 4:25 - REFACTOR Complete

Code is clean, tests pass, ready for COMMIT.

## When Code Needs More Refactoring

Sometimes GREEN code is messy enough that 1 minute isn't enough:

```rust
async fn handle(&self, input: Self::Input) -> Result<Self::Output> {
    let x = input.a;
    let y = input.b;
    let z = input.c;
    let q = x + y * z - (x / y) + (z * x);
    let r = q * 2;
    let s = r - 10;
    let t = s / 2;
    let u = t + q;
    let v = u * s;

    Ok(Output { result: v })
}
```

You have two options:

### Option 1: Partial Refactor

Do what you can in 1 minute:

```rust
async fn handle(&self, input: Self::Input) -> Result<Self::Output> {
    // Improved names (30 seconds)
    let a = input.a;
    let b = input.b;
    let c = input.c;

    let complex_calc = a + b * c - (a / b) + (c * a);
    let doubled = complex_calc * 2;
    let adjusted = doubled - 10;
    let halved = adjusted / 2;
    let combined = halved + complex_calc;
    let final_result = combined * adjusted;

    Ok(Output { result: final_result })
}
```

Then create a TODO for deeper refactoring:

```rust
// TODO(REFACTOR): Extract calculation logic into separate functions
// This calculation is complex and would benefit from decomposition
// Estimated effort: 2-3 TDD cycles
```

### Option 2: COMMIT Then Refactor

If code is working but ugly:
1. COMMIT the working code
2. Start a new cycle dedicated to refactoring
3. Use the same tests as safety net

This is better than extending the cycle to 7-8 minutes.

## Refactoring Without Tests

Never refactor code without tests. If code lacks tests:

1. **Stop**: Don't refactor
2. **Add tests first**: Write tests in separate cycles
3. **Then refactor**: Once tests exist, refactor safely

Refactoring without tests is reckless. You can't verify behavior stays unchanged.

## The Safety of Small Refactorings

Why 1-minute refactorings are safe:

**Small Changes**: Each refactoring is tiny. Easy to understand, easy to verify.

**Frequent Testing**: Run tests after every refactoring. Catch breaks immediately.

**Easy Revert**: If refactoring breaks tests, revert is fast (Git history is < 5 minutes old).

**Muscle Memory**: After 50 cycles, these refactorings become automatic.

## Automated Refactoring Tools

Rust-analyzer provides automated refactorings:

- **Rename**: Rename variable/function (safe, updates all references)
- **Extract variable**: Pull expression into variable
- **Inline variable**: Opposite of extract
- **Change signature**: Modify function parameters

These are safe because the tool maintains correctness. Use them liberally in REFACTOR.

```rust
// In VS Code with rust-analyzer:
// 1. Place cursor on variable name
// 2. Press F2 (rename)
// 3. Type new name
// 4. Press Enter
// All references updated automatically
```

**Time**: 5-10 seconds per refactoring

## REFACTOR Anti-Patterns

### Anti-Pattern 1: Refactoring During GREEN

```rust
// BAD - Refactoring while implementing
async fn handle(&self, input: Self::Input) -> Result<Self::Output> {
    // Writing implementation...
    let result = calculate(input);

    // Oh, let me make this name better...
    // And extract this constant...
    // And simplify this expression...
}
```

**Why it's bad**: GREEN and REFACTOR serve different purposes. Mixing them extends cycle time and confuses goals.

**Fix**: Resist the urge to refactor during GREEN. Write minimum code, even if ugly. Clean it in REFACTOR.

### Anti-Pattern 2: Speculative Refactoring

```rust
// BAD - Refactoring for "future needs"
async fn handle(&self, input: Self::Input) -> Result<Self::Output> {
    // Current need: simple addition
    // But "maybe we'll need subtraction later", so...

    let calculator = GenericCalculator::new();
    calculator.register_operation("add", Box::new(AddOperation));
    // ... 20 more lines of infrastructure
}
```

**Why it's bad**: YAGNI (You Aren't Gonna Need It). Speculative refactoring adds complexity for uncertain future needs.

**Fix**: Refactor for current needs only. When subtraction is actually needed, refactor then.

### Anti-Pattern 3: Breaking Tests

```rust
// REFACTOR starts
async fn handle(&self, input: Self::Input) -> Result<Self::Output> {
    // Some refactoring...
}

// Run tests
cargo test
# test test_calculate ... FAILED

// Continue anyway, assuming I'll fix it later
```

**Why it's bad**: If REFACTOR breaks tests, you've changed behavior. That's a bug, not a refactoring.

**Fix**: If tests break, revert immediately:

```bash
git checkout .
```

Investigate why the refactoring broke tests. Either:
- The refactoring was wrong (fix it)
- The test was wrong (fix it in a separate cycle)

## Measuring Refactoring Effectiveness

Track these metrics:

**Cyclomatic Complexity**: Should decrease or stay flat after refactoring

```bash
pmat analyze complexity --max 20
# Before: function_name: 15
# After:  function_name: 12
```

**Line Count**: Should decrease or stay flat (not always, but often)

**Clippy Warnings**: Should decrease to zero

```bash
cargo clippy
# Before: 3 warnings
# After:  0 warnings
```

## The Refactoring Habit

After 30 days of EXTREME TDD, refactoring becomes automatic:

**Minute 4:00**: Timer hits, you transition to REFACTOR without thinking

**Scan**: Eyes automatically scan for duplication, bad names, complexity

**Refactor**: Fingers execute refactorings via muscle memory

**Test**: Tests run automatically (in watch mode)

**Done**: Clean code, passing tests, ready to commit

This takes 30-40 seconds after the habit forms.

## REFACTOR Success Metrics

Track these to improve:

**Time in REFACTOR**: Average time spent refactoring
- Target: < 1:00
- Excellent: < 0:45
- Expert: < 0:30

**Refactorings Per Cycle**: Average number of refactorings applied
- Target: 1-2
- Excellent: 2-3
- Expert: 3-4 (fast, automated refactorings)

**Test Breaks During REFACTOR**: Tests broken by refactoring
- Target: < 5%
- Excellent: < 2%
- Expert: < 1%

## When to Skip REFACTOR

Sometimes code is clean enough after GREEN:

```rust
async fn handle(&self, input: Self::Input) -> Result<Self::Output> {
    Ok(AddOutput {
        sum: input.a + input.b,
    })
}
```

This is already clean. No refactoring needed.

**Still run the checklist**:
- Run formatter
- Run clippy
- Run tests

But don't force refactoring for the sake of it.

## Deep Refactoring Cycles

For complex refactorings (extract function, change architecture), dedicate full cycles:

**RED**: Write test proving current behavior
**GREEN**: No changes (test already passes)
**REFACTOR**: Apply complex refactoring
**COMMIT**: Verify tests still pass, commit

This uses the 5-minute cycle structure but focuses entirely on refactoring.

## The Psychology of REFACTOR

**Pride**: Refactoring is satisfying. Taking messy code and making it clean feels good.

**Safety**: Tests provide confidence. Refactor boldly knowing tests catch mistakes.

**Discipline**: The 1-minute limit prevents perfectionism. "Good enough" beats "perfect but incomplete."

**Momentum**: Clean code is easier to build upon. Refactoring accelerates future cycles.

## Next Phase: COMMIT

You have clean, tested code. Now it's time for the quality gates to decide: COMMIT or RESET?

This final phase determines if your cycle's work enters the codebase or gets discarded.

---

Previous: [GREEN: Minimum Code](ch07-02-green.md)
Next: [COMMIT: Quality Gates](ch07-04-commit.md)
