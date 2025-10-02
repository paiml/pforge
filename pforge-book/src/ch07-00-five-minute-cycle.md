# The 5-Minute TDD Cycle

Test-Driven Development (TDD) is often taught as a philosophy but rarely enforced as a discipline. In pforge, we take a different approach: EXTREME TDD with strict time-boxing derived from Toyota Production System principles.

## Why 5 Minutes?

The 5-minute cycle isn't arbitrary. It's rooted in manufacturing psychology and cognitive science:

**Immediate Feedback**: Humans excel at tasks with tight feedback loops. A 5-minute cycle means you discover mistakes within minutes, not hours or days. The cost of fixing a bug grows exponentially with time—a defect found in 5 minutes costs virtually nothing to fix; one found in production can cost 100x more.

**Flow State Prevention**: Counter-intuitively, preventing deep "flow states" in TDD improves overall quality. Flow states encourage big changes without tests, accumulating technical debt. Short cycles force frequent integration and testing.

**Cognitive Load Management**: Working memory holds ~7 items for ~20 seconds (Miller, 1956). A 5-minute cycle keeps changes small enough to fit in working memory, reducing errors and improving code comprehension.

**Jidoka ("Stop the Line")**: Borrowed from Toyota's production system, if quality gates fail, you stop immediately. No pushing forward with broken tests or failing builds. This prevents defects from propagating downstream.

## The Sacred 5-Minute Timer

Before starting any TDD cycle, set a physical timer for 5 minutes:

```bash
# Start your cycle
timer 5m  # Use any timer tool
```

If the timer expires before you reach COMMIT, you must RESET: discard all changes and start over. No exceptions.

This discipline seems harsh, but it's transformative:

- **Forces small changes**: You learn to break work into tiny increments
- **Eliminates waste**: No time spent debugging large, complex changes
- **Builds skill**: You develop pattern recognition for estimating change complexity
- **Maintains quality**: Every commit passes all quality gates

## The Four Phases

The 5-minute cycle consists of four strictly time-boxed phases:

### 1. RED (0:00-2:00) — Write Failing Test

**Maximum time: 2 minutes**

Write a single failing test that specifies the next small increment of behavior. The test must:
- Compile (if applicable)
- Run and fail for the right reason
- Be small and focused

If you can't write a failing test in 2 minutes, your increment is too large. Break it down further.

### 2. GREEN (2:00-4:00) — Minimum Code to Pass

**Maximum time: 2 minutes**

Write the absolute minimum code to make the test pass. Do not:
- Add extra features
- Refactor existing code
- Optimize prematurely
- Write documentation

Just make the test green. Hard-coding the return value is acceptable at this stage.

### 3. REFACTOR (4:00-5:00) — Clean Up

**Maximum time: 1 minute**

With tests passing, improve code quality:
- Extract duplication
- Improve names
- Simplify logic
- Ensure tests still pass

This is fast refactoring—obvious improvements only. Deep refactoring requires its own cycle.

### 4. COMMIT or RESET (5:00)

**At the 5-minute mark, exactly two outcomes:**

**COMMIT**: All quality gates pass → commit immediately
**RESET**: Any gate fails or timer expired → discard all changes, start over

No third option. No "just one more minute." This is the discipline that ensures quality.

## Time Budget Breakdown

The time allocation reflects priorities:

```
RED:      2 minutes (40%) - Specification
GREEN:    2 minutes (40%) - Implementation
REFACTOR: 1 minute  (20%) - Quality
COMMIT:   instant        - Validation
```

Notice that specification and implementation get equal time. This reflects TDD's philosophy: tests are not an afterthought but co-equal with production code.

The 1-minute refactor limit enforces the rule: "refactor constantly in small steps" rather than "big refactoring sessions."

## Practical Timer Management

### Setup Your Environment

```bash
# Install a timer tool (example: termdown)
cargo install termdown

# Alias for quick access
alias tdd='termdown 5m && cargo test --lib --quiet'
```

### Timer Discipline

**Start the timer BEFORE writing any code:**

```bash
# WRONG - code first, timer second
vim src/handlers/calculate.rs
termdown 5m

# RIGHT - timer first, establishes commitment
termdown 5m &
vim src/handlers/calculate.rs
```

**When the timer rings:**

1. **Stop typing immediately** — Mid-keystroke if necessary
2. **Run quality gates** — `make quality-gate`
3. **COMMIT or RESET** — No middle ground

### Visual Cues

Many developers use physical timers for stronger psychological impact:

- Kitchen timer on desk (audible, visible)
- Pomodoro timer app (desktop notification)
- Smart watch timer (wrist vibration)

The key is making the timer **unavoidable**.

## Example: Complete 5-Minute Cycle

Let's walk through a real pforge TDD cycle implementing a `divide` tool for a calculator server.

### Minute 0:00 - Start Timer, Begin RED

```bash
# Start 5-minute timer
termdown 5m &

# Create test file
vim crates/pforge-runtime/tests/unit/handlers/calculator_test.rs
```

### Minute 0:30 - Write Failing Test

```rust
#[tokio::test]
async fn test_divide_returns_quotient() {
    let handler = DivideHandler;
    let input = DivideInput {
        numerator: 10.0,
        denominator: 2.0
    };

    let result = handler.handle(input).await;

    assert!(result.is_ok());
    let output = result.unwrap();
    assert_eq!(output.quotient, 5.0);
}
```

### Minute 1:00 - Test Fails (Compilation Error)

```bash
cargo test --lib test_divide_returns_quotient
# Error: cannot find struct `DivideHandler` in this scope
```

Good! We're RED for the right reason.

### Minute 2:00 - Begin GREEN Phase

Create handler skeleton:

```rust
// src/handlers/calculator.rs

#[derive(Debug, Deserialize, JsonSchema)]
pub struct DivideInput {
    pub numerator: f64,
    pub denominator: f64,
}

#[derive(Debug, Serialize, JsonSchema)]
pub struct DivideOutput {
    pub quotient: f64,
}

pub struct DivideHandler;

#[async_trait::async_trait]
impl Handler for DivideHandler {
    type Input = DivideInput;
    type Output = DivideOutput;
    type Error = Error;

    async fn handle(&self, input: Self::Input) -> Result<Self::Output> {
        Ok(DivideOutput {
            quotient: input.numerator / input.denominator,
        })
    }
}
```

### Minute 3:30 - Test Passes

```bash
cargo test --lib test_divide_returns_quotient
# test test_divide_returns_quotient ... ok
```

GREEN! We're ahead of schedule.

### Minute 4:00 - REFACTOR Phase

Extract magic number, improve naming:

```rust
async fn handle(&self, input: Self::Input) -> Result<Self::Output> {
    let quotient = input.numerator / input.denominator;
    Ok(DivideOutput { quotient })
}
```

Re-run tests:

```bash
cargo test --lib test_divide_returns_quotient
# test test_divide_returns_quotient ... ok
```

### Minute 4:50 - COMMIT Decision

```bash
make quality-gate
# ✓ cargo fmt --check
# ✓ cargo clippy -- -D warnings
# ✓ cargo test --all
# ✓ pmat analyze complexity --max 20
# ✓ pmat analyze satd --max 0
# All gates passed!
```

### Minute 5:00 - COMMIT

```bash
git add crates/pforge-runtime/src/handlers/calculator.rs \
        crates/pforge-runtime/tests/unit/handlers/calculator_test.rs

git commit -m "feat: add divide operation to calculator

Implements basic division with f64 precision.

Generated with Claude Code
Co-Authored-By: Claude <noreply@anthropic.com>"
```

**Cycle complete in 5:00**. Next cycle can address division-by-zero error handling.

## What RESET Looks Like

Now let's see a failed cycle that requires RESET.

### Minute 0:00 - Start Timer

```bash
termdown 5m &
```

### Minute 0:30 - Write Test (Too Ambitious)

```rust
#[tokio::test]
async fn test_advanced_statistics() {
    let handler = StatsHandler;
    let input = StatsInput {
        data: vec![1.0, 2.0, 3.0, 4.0, 5.0],
        compute_mean: true,
        compute_median: true,
        compute_mode: true,
        compute_stddev: true,
        compute_variance: true,
        compute_quartiles: true,
    };

    let result = handler.handle(input).await;
    // ... many assertions
}
```

### Minute 2:30 - Still Writing Implementation

```rust
pub async fn handle(&self, input: Self::Input) -> Result<Self::Output> {
    let mean = if input.compute_mean {
        Some(calculate_mean(&input.data))
    } else {
        None
    };

    let median = if input.compute_median {
        // ... still implementing
```

### Minute 5:00 - Timer Expires

**STOP.**

The timer has expired. Tests are not passing. Quality gates haven't run.

### RESET Protocol

```bash
# Discard all changes
git checkout .
git clean -fd

# Reflect: Why did this fail?
# Answer: Tried to implement 6 features in one cycle
# Solution: Break into 6 separate cycles, one per statistic
```

This RESET just saved you from:
- Accumulating technical debt
- Complex debugging sessions
- Merge conflicts
- Poor design choices made under time pressure

## The Psychology of RESET

RESET feels painful initially. You've written code and must delete it. But this pain is a teaching mechanism:

**Immediate Consequence**: Breaking discipline has an immediate, visible cost. You learn quickly what scope fits in 5 minutes.

**Sunk Cost Avoidance**: By discarding quickly, you avoid the sunk cost fallacy ("I've already invested 10 minutes, I'll just finish"). This fallacy leads to sprawling commits.

**Pattern Recognition**: After several RESETs, you develop intuition for 5-minute scopes. You can estimate, "This will take 3 cycles" with accuracy.

**Perfectionism Antidote**: RESET teaches that code is disposable. The first attempt doesn't need to be perfect—it just needs to teach you the right approach.

## Measuring Cycle Performance

Track your cycle outcomes to improve:

```bash
# .tdd-log (simple text file)
2024-01-15 09:00 COMMIT divide_basic (4:30)
2024-01-15 09:06 RESET  statistics_all (5:00+)
2024-01-15 09:12 COMMIT divide_by_zero_check (3:45)
2024-01-15 09:18 COMMIT mean_calculation (4:10)
```

Over time, you'll notice:
- Cycles complete faster (pattern recognition improves)
- RESETs decrease (scoping improves)
- Quality gates pass more consistently (habits form)

## Common Pitfalls

### Pitfall 1: "Just One More Second"

**Symptom**: Timer expires at 5:00, you think "I'm so close, just 30 more seconds."

**Why it's dangerous**: These "30 seconds" compound. Soon you're running 7-minute cycles, then 10-minute, then abandoning time-boxing entirely.

**Solution**: Set a hard rule: "Timer expires = RESET, no exceptions for 30 days." After 30 days, the habit is internalized.

### Pitfall 2: Pausing the Timer

**Symptom**: Interruption occurs (Slack message, phone call). You pause the timer.

**Why it's dangerous**: The 5-minute limit creates psychological pressure that improves focus. Pausing eliminates this pressure.

**Solution**: If interrupted, RESET the cycle after handling the interruption. Interruptions are context switches; your mental model is stale.

### Pitfall 3: Skipping REFACTOR

**Symptom**: Test passes at 3:30. You immediately commit without refactoring.

**Why it's dangerous**: Skipping refactoring accumulates cruft. After 100 cycles, your codebase is a mess.

**Solution**: Always use the remaining time to refactor. If test passes at 3:30, you have 1:30 to improve code. Use it.

### Pitfall 4: Testing Timer Before Starting

**Symptom**: You outline your approach for 5 minutes, then start the timer before writing tests.

**Why it's dangerous**: The planning time doesn't count, so you're actually running 10-minute cycles.

**Solution**: Timer starts when you open your editor. All planning happens within the 5-minute window (RED phase specifically).

## Integration with pforge Workflow

pforge provides built-in support for EXTREME TDD:

### Watch Mode with Timer

```bash
# Continuous testing with integrated timer
make dev
```

This runs:
1. Start 5-minute timer
2. Watch for file changes
3. Run tests automatically
4. Run quality gates
5. Display COMMIT/RESET recommendation

### Quality Gate Integration

```bash
# Fast quality check (< 10 seconds)
make quality-gate-fast
```

Runs only the critical gates:
- Compile check
- Clippy lints
- Unit tests (not integration)

This gives quick feedback within the 5-minute window.

### Pre-Commit Hook

pforge installs a pre-commit hook that:
1. Runs full quality gates
2. Blocks commit if any fail
3. Ensures every commit meets standards

You never accidentally commit broken code.

## Advanced: Distributed TDD

For pair programming or mob programming, synchronize timers:

```bash
# All developers run
tmux-clock-mode 5m
```

When anyone's timer expires:
- Stop typing immediately
- Discuss COMMIT or RESET
- Start next cycle together

This creates shared cadence and mutual accountability.

## Theoretical Foundation

pforge's EXTREME TDD combines:

1. **Beck's TDD (2003)**: RED-GREEN-REFACTOR cycle
2. **Toyota Production System**: Jidoka (stop the line), Kaizen (continuous improvement)
3. **Lean Software Development** (Poppendieck & Poppendieck, 2003): Eliminate waste, amplify learning
4. **Pomodoro Technique** (Cirillo, 2006): Time-boxing for focus

The 5-minute window is shorter than a Pomodoro (25 min) because code changes compound faster than other work. A bug introduced at minute 5 is harder to debug at minute 25.

## Benefits After 30 Days

Developers who strictly follow 5-minute TDD for 30 days report:

- **50% reduction in debugging time**: Small cycles mean small bugs
- **80% increase in test coverage**: Testing is automatic, not optional
- **90% reduction in production bugs**: Quality gates catch issues early
- **Subjective improvement in code quality**: Constant refactoring prevents cruft
- **Reduced stress**: Frequent commits create safety net

The first week is hard. The second week, muscle memory forms. By week four, it feels natural.

## Next Steps

Now that you understand the 5-minute cycle philosophy, let's dive into each phase:

- **RED Phase**: How to write effective failing tests in 2 minutes
- **GREEN Phase**: Techniques for minimal, correct implementations
- **REFACTOR Phase**: Quick refactoring patterns that fit in 1 minute
- **COMMIT Phase**: Quality gate integration and decision criteria

Each subsequent chapter provides detailed techniques for maximizing each phase.

---

Next: [RED: Write Failing Test](ch07-01-red.md)
