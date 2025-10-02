# COMMIT: Quality Gates

You've reached minute 5:00. Tests pass. Code is clean. Now comes the moment of truth: do quality gates pass?

**COMMIT**: All gates pass → Accept the work
**RESET**: Any gate fails → Discard everything

No middle ground. No "mostly passing." This binary decision enforces uncompromising quality standards.

## The Quality Gate Philosophy

Quality gates embody Toyota's **Jidoka** principle: "Stop the line when defects occur." If quality standards aren't met, production halts.

### Why Binary?

**No Compromise**: Quality is non-negotiable. A partially working feature is worse than no feature—it gives false confidence.

**Clear Signal**: Binary outcomes are unambiguous. You know instantly whether the cycle succeeded.

**Forcing Function**: Knowing you might RESET motivates you to stay within the 5-minute budget and write clean code from the start.

**Continuous Integration**: Every commit maintains codebase quality. No "I'll fix it later" accumulation.

## pforge Quality Gates

pforge enforces multiple quality gates via `make quality-gate`:

### Gate 1: Formatting

```bash
cargo fmt --check
```

**What it checks**: Code follows Rust style guide (indentation, spacing, line breaks)

**Why it matters**: Consistent formatting reduces cognitive load and diff noise

**Typical failures**:
- Inconsistent indentation
- Missing/extra line breaks
- Non-standard brace placement

**Fix**: Run `cargo fmt` before checking

### Gate 2: Linting (Clippy)

```bash
cargo clippy -- -D warnings
```

**What it checks**: Common Rust pitfalls, performance issues, style violations

**Why it matters**: Clippy catches bugs and code smells automatically

**Typical failures**:
- Unused variables
- Unnecessary clones
- Redundant pattern matching
- Performance anti-patterns

**Fix**: Address each warning individually or suppress with `#[allow(clippy::...)]` if truly necessary

### Gate 3: Tests

```bash
cargo test --all
```

**What it checks**: All tests (unit, integration, doc tests) pass

**Why it matters**: Broken tests mean broken behavior

**Typical failures**:
- New code breaks existing tests (regression)
- New test doesn't pass (incomplete implementation)
- Flaky tests (non-deterministic behavior)

**Fix**: Debug failing tests, fix implementation, or fix test expectations

### Gate 4: Complexity

```bash
pmat analyze complexity --max 20
```

**What it checks**: Cyclomatic complexity of each function

**Why it matters**: Complex functions are bug-prone and hard to maintain

**Typical failures**:
- Too many conditional branches
- Deeply nested loops
- Long match statements

**Fix**: Extract functions, simplify conditionals, reduce nesting

### Gate 5: Technical Debt

```bash
pmat analyze satd --max 0
```

**What it checks**: Self-Admitted Technical Debt (SATD) comments like `TODO`, `FIXME`, `HACK`

**Why it matters**: SATD comments indicate code that needs improvement

**Typical failures**:
- Leftover `TODO` comments
- `FIXME` markers
- `HACK` acknowledgments

**Fix**: Either address the issue or remove the comment (only if it's not actual debt)

**Exception**: Phase markers like `TODO(RED)`, `TODO(GREEN)`, `TODO(REFACTOR)` are allowed during development but must be removed before COMMIT

### Gate 6: Coverage

```bash
cargo tarpaulin --out Json
```

**What it checks**: Test coverage ≥ 80%

**Why it matters**: Untested code is unverified code

**Typical failures**:
- New code without tests
- Error paths not tested
- Edge cases not covered

**Fix**: Add tests for uncovered lines

### Gate 7: Technical Debt Grade

```bash
pmat analyze tdg --min 0.75
```

**What it checks**: Overall technical debt grade (0-1 scale)

**Why it matters**: Aggregate measure of code health

**Typical failures**:
- Combination of complexity, SATD, dead code, and low coverage
- Accumulation of small issues

**Fix**: Address individual issues contributing to low TDG

## Running Quality Gates

### Fast Check (During REFACTOR)

```bash
make quality-gate-fast
```

Runs subset of gates for quick feedback:
- Formatting
- Clippy
- Unit tests only

**Time**: < 10 seconds

Use this during REFACTOR to catch issues early.

### Full Check (Before COMMIT)

```bash
make quality-gate
```

Runs all gates:
- Formatting
- Clippy
- All tests
- Complexity
- SATD
- Coverage
- TDG

**Time**: < 30 seconds (for small projects)

Use this at minute 4:30-5:00 before deciding COMMIT or RESET.

## The COMMIT Decision

At minute 5:00, run `make quality-gate`:

### Scenario 1: All Gates Pass

```bash
make quality-gate
# ✓ Formatting check passed
# ✓ Clippy check passed
# ✓ Tests passed (15 passed; 0 failed)
# ✓ Complexity check passed (max: 9/20)
# ✓ SATD check passed (0 markers found)
# ✓ Coverage check passed (87.5%)
# ✓ TDG check passed (0.92/0.75)
# All quality gates passed!
```

**Decision: COMMIT**

Stage and commit your changes:

```bash
git add -A
git commit -m "feat: add division handler with zero check

Implements division operation with validation for zero denominator.

Generated with Claude Code
Co-Authored-By: Claude <noreply@anthropic.com>"
```

**Cycle successful**. Start next cycle.

### Scenario 2: One or More Gates Fail

```bash
make quality-gate
# ✓ Formatting check passed
# ✗ Clippy check failed (3 warnings)
# ✓ Tests passed
# ✓ Complexity check passed
# ✓ SATD check passed
# ✗ Coverage check failed (72.3% < 80%)
# ✓ TDG check passed
# Quality gates FAILED
```

**Decision: RESET**

Discard all changes:

```bash
git checkout .
git clean -fd
```

**Cycle failed**. Reflect, then start next cycle with adjusted scope.

### Scenario 3: Timer Expired

```bash
# Check time
echo "Minute: 5:30"
```

Timer expired before running quality gates.

**Decision: RESET**

No exceptions. Even if you're "almost done," RESET.

```bash
git checkout .
git clean -fd
```

## The RESET Protocol

When RESET occurs, follow this protocol:

### Step 1: Discard Changes

```bash
git checkout .
git clean -fd
```

This removes all uncommitted changes—both tracked and untracked files.

### Step 2: Reflect

Don't immediately start the next cycle. Take 30-60 seconds to reflect:

**Why did RESET occur?**
- Timer expired → Scope too large
- Tests failed → Implementation incomplete or incorrect
- Complexity too high → Need simpler approach
- Coverage too low → Missing tests

**What will I do differently next cycle?**
- Smaller scope (fewer features per test)
- Simpler implementation (avoid clever approaches)
- Better planning (think before typing)
- More tests (test error cases too)

### Step 3: Log the RESET

Track your RESETs to identify patterns:

```bash
echo "$(date) RESET divide_by_zero - complexity too high (cycle 5:30)" >> .tdd-log
```

Over time, you'll notice:
- Common failure modes
- Scope estimation improvements
- Decreasing RESET frequency

### Step 4: Start Fresh Cycle

Begin a new 5-minute cycle with adjusted scope:

```bash
termdown 5m &
vim tests/calculator_test.rs
```

Apply lessons learned from the RESET.

## Common COMMIT Failures

### Failure 1: Clippy Warnings

```
warning: unused variable: `temp`
  --> src/handlers/calculate.rs:12:9
   |
12 |     let temp = input.a + input.b;
   |         ^^^^ help: if this is intentional, prefix it with an underscore: `_temp`
```

**Why it happens**: Leftover variables from implementation iterations

**Quick fix** (if < 30 seconds to minute 5:00):

```rust
// Remove unused variable
// let temp = input.a + input.b;  // deleted

Ok(Output { result: input.a + input.b })
```

Re-run quality gates.

**If no time to fix**: RESET

### Failure 2: Test Regression

```
test test_add_positive_numbers ... FAILED

failures:

---- test_add_positive_numbers stdout ----
thread 'test_add_positive_numbers' panicked at 'assertion failed: `(left == right)`
  left: `5`,
 right: `6`'
```

**Why it happens**: New code broke existing functionality

**Quick fix**: Unlikely to fix in < 30 seconds

**Correct action**: RESET

Regression means your change had unintended side effects. You need to rethink the approach.

### Failure 3: Low Coverage

```
Coverage: 72.3% (target: 80%)
Uncovered lines:
  src/handlers/divide.rs:15-18 (error handling)
```

**Why it happens**: Forgot to test error paths

**Quick fix** (if close to time limit): Write missing test in next cycle

**Correct action**: RESET if you want this feature in codebase now

Coverage gates ensure every line is tested. Untested error handling is a bug waiting to happen.

### Failure 4: High Complexity

```
Cyclomatic complexity check failed:
  src/handlers/calculate.rs:handle (complexity: 23, max: 20)
```

**Why it happens**: Too many conditional branches

**Quick fix**: Unlikely in remaining time

**Correct action**: RESET

High complexity indicates the implementation needs redesign. Quick patches won't fix fundamental complexity.

## When to Override Quality Gates

**Never.**

The strict answer: you should never override quality gates in EXTREME TDD. If gates fail, the cycle fails.

However, in practice, there are rare circumstances where you might `git commit --no-verify`:

### Acceptable Override Cases

**Pre-commit hook not installed yet**: First commit setting up the project

**External dependency issues**: Gate tool unavailable (e.g., CI server down, PMAT not installed)

**Emergency hotfix**: Production is down, fix needs to deploy immediately

**Experimental branch**: Explicitly marked WIP branch, not merging to main

### Unacceptable Override Cases

**"I'm in a hurry"**: No. RESET and do it right.

**"The gate is wrong"**: If the gate is genuinely wrong, fix the gate in a separate cycle. Don't override.

**"It's just a style issue"**: Style issues compound. Fix them.

**"I'll fix it in the next commit"**: No. Future you won't fix it. Fix it now or RESET.

## The Pre-Commit Hook

pforge installs a pre-commit hook that runs quality gates automatically:

```bash
.git/hooks/pre-commit
```

Contents:

```bash
#!/bin/bash
set -e

echo "Running quality gates..."
make quality-gate

if [ $? -ne 0 ]; then
    echo "Quality gates failed. Commit blocked."
    exit 1
fi

echo "Quality gates passed. Commit allowed."
exit 0
```

This hook:
- Runs automatically on `git commit`
- Blocks commit if gates fail
- Ensures you never accidentally commit bad code

To bypass (rarely needed):

```bash
git commit --no-verify
```

But this should be exceptional, not routine.

## COMMIT Message Conventions

When COMMIT succeeds, write a clear commit message:

### Format

```
<type>: <short summary>

<detailed description>

Generated with Claude Code
Co-Authored-By: Claude <noreply@anthropic.com>
```

### Types

- `feat`: New feature
- `fix`: Bug fix
- `refactor`: Code restructuring (no behavior change)
- `test`: Add or modify tests
- `docs`: Documentation changes
- `chore`: Build, dependencies, tooling

### Examples

```bash
git commit -m "feat: add divide operation to calculator

Implements basic division with f64 precision. Validates denominator is non-zero and returns appropriate error for division by zero.

Generated with Claude Code
Co-Authored-By: Claude <noreply@anthropic.com>"
```

```bash
git commit -m "test: add edge case for negative numbers

Ensures calculator handles negative operands correctly.

Generated with Claude Code
Co-Authored-By: Claude <noreply@anthropic.com>"
```

```bash
git commit -m "refactor: extract validation into helper function

Reduces cyclomatic complexity from 18 to 12 by extracting input validation logic.

Generated with Claude Code
Co-Authored-By: Claude <noreply@anthropic.com>"
```

## Psychology of COMMIT vs RESET

### The Joy of COMMIT

When quality gates pass:

```
✓ All quality gates passed!
```

There's a genuine dopamine hit. You've:
- Written working code
- Maintained quality standards
- Made progress

This positive reinforcement encourages continuing the discipline.

### The Pain of RESET

When quality gates fail:

```
✗ Quality gates FAILED
```

There's genuine disappointment. You've:
- Spent 5 minutes
- Produced nothing commitworthy
- Must start over

This negative reinforcement teaches you to:
- Scope smaller
- Write cleaner code upfront
- Respect the time budget

### The Learning Curve

First week:
- COMMIT rate: ~50%
- RESET rate: ~50%
- Frequent frustration

Second week:
- COMMIT rate: ~70%
- RESET rate: ~30%
- Pattern recognition forms

Fourth week:
- COMMIT rate: ~90%
- RESET rate: ~10%
- Discipline internalized

The pain of RESETs trains you to succeed. After 30 days, you intuitively scope work to fit 5-minute cycles.

## Tracking COMMIT/RESET Ratios

Track your outcomes to measure improvement:

```bash
# Simple tracking script
echo "$(date) COMMIT feat_divide_basic (4:45)" >> .tdd-log
echo "$(date) RESET  feat_divide_zero (5:30)" >> .tdd-log
```

Calculate weekly stats:

```bash
grep COMMIT .tdd-log | wc -l  # 27
grep RESET .tdd-log | wc -l   # 3

# Success rate: 27/(27+3) = 90%
```

### Target Metrics

**Week 1**: 50% COMMIT rate (learning)
**Week 2**: 70% COMMIT rate (improving)
**Week 4**: 85% COMMIT rate (proficient)
**Week 8**: 95% COMMIT rate (expert)

## When RESET Happens Repeatedly

If you RESET 3+ times on the same feature:

### Stop and Reassess

**Problem**: Your approach isn't working

**Solutions**:
1. **Break down further**: Feature is too large for one cycle
2. **Research first**: You don't understand the domain well enough
3. **Spike solution**: Take 15 minutes outside TDD to explore approaches
4. **Pair program**: Another developer might see a simpler approach
5. **Defer feature**: Maybe this feature needs more design before implementation

### Example: Persistent RESET

```bash
# Attempting to implement JWT authentication
09:00 RESET jwt_auth_validate (5:45)
09:06 RESET jwt_auth_validate (5:30)
09:12 RESET jwt_auth_validate (6:00)
```

After 3 RESETs, stop. Take 15 minutes to:
- Read JWT library documentation
- Write a spike (throwaway code) to understand API
- Identify the smallest incremental step

Then return to TDD with better understanding.

## Quality Gates in CI/CD

Quality gates don't just run locally—they run in CI/CD:

### GitHub Actions Example

```yaml
# .github/workflows/quality.yml
name: Quality Gates

on: [push, pull_request]

jobs:
  quality:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v2
      - name: Install Rust
        uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
      - name: Run Quality Gates
        run: make quality-gate
```

This ensures:
- Every push runs quality gates
- Pull requests can't merge if gates fail
- Team maintains quality standards

## Advanced: Graduated Quality Gates

For larger changes, use graduated quality gates:

### Cycle 1: Core Implementation

- Run fast gates (fmt, clippy, unit tests)
- COMMIT if passing

### Cycle 2: Integration Tests

- Run integration tests
- COMMIT if passing

### Cycle 3: Performance Tests

- Run benchmarks
- COMMIT if no regression

This allows you to make progress in 5-minute increments while building up to full validation.

## The Discipline of Binary Outcomes

The hardest part of EXTREME TDD is accepting binary outcomes:

**No "Good Enough"**: Either all gates pass or they don't. No subjective judgment.

**No "I'll Fix Later"**: Future you won't fix it. Fix it now or RESET.

**No "It's Just One Warning"**: One warning becomes ten warnings becomes unmaintainable code.

This discipline seems harsh, but it's what maintains quality over hundreds of cycles.

## Celebrating COMMITs

Each COMMIT is progress. Celebrate small wins:

```bash
# After COMMIT
echo "✓ Feature complete: divide with zero check"
echo "✓ Tests: 12 passing"
echo "✓ Coverage: 87%"
echo "✓ Cycle time: 4:45"
```

Recognizing progress maintains motivation through the discipline of EXTREME TDD.

## Next Steps

You now understand the complete 5-minute EXTREME TDD cycle:

**RED** (2 min): Write failing test
**GREEN** (2 min): Minimum code to pass
**REFACTOR** (1 min): Clean up
**COMMIT** (instant): Quality gates decide

This cycle, repeated hundreds of times, builds production-quality software with:
- 80%+ test coverage
- Zero technical debt
- Consistent code quality
- Frequent commits (safety net)

The next chapters cover quality gates in detail, testing strategies, and advanced TDD patterns.

---

Previous: [REFACTOR: Clean Up](ch07-03-refactor.md)
Next: [Chapter 8: Quality Gates](ch08-00-quality-gates.md)
