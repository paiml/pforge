# Quality Gates: The Jidoka Principle

In Toyota's manufacturing system, **Jidoka** means "automation with a human touch" or more commonly: **"Stop the line when defects occur."** If a worker spots a quality issue, they pull the andon cord, halting the entire production line until the problem is fixed.

This principle prevents defects from propagating downstream and accumulating into expensive rework.

pforge applies Jidoka to software development through **automated quality gates**: a series of checks that must pass before code enters the codebase. If any gate fails, development stops. Fix the issue, then proceed.

No compromises. No "I'll fix it later." No technical debt accumulation.

## The Quality Gate Philosophy

Traditional development often treats quality as an afterthought:
- Write code quickly, worry about quality later
- Accumulate technical debt, plan a "cleanup sprint" (that never happens)
- Let failing tests slide, promising to fix them "after the deadline"
- Ignore warnings, complexity, and code smells

This creates a **debt spiral**: poor quality begets more poor quality. Complexity increases. Tests become flaky. Refactoring becomes dangerous. Eventually, the codebase becomes unmaintainable.

**Quality gates prevent this spiral by enforcing standards at every commit.**

### Why Quality Gates Matter

**Prevention over Cure**: Catching issues early is exponentially cheaper than fixing them later. A linting error caught pre-commit takes 30 seconds to fix. The same issue in production might take hours or days.

**Compound Quality**: Each commit builds on previous work. If commit N is low quality, commits N+1, N+2, N+3 inherit that debt. Quality gates ensure every commit maintains baseline standards.

**Rapid Feedback**: Developers get immediate feedback. No waiting for CI, code review, or QA to discover issues.

**Forcing Function**: Knowing that commits will be rejected for quality violations changes behavior. You write cleaner code from the start.

**Collective Ownership**: Quality gates are objective and automated. They apply equally to all contributors, maintaining consistent standards.

## pforge's Quality Gate Stack

pforge enforces **eight quality gates** before allowing commits:

### 0. Documentation Link Validation

**Command**: `pmat validate-docs --fail-on-error`

**What it checks**: All markdown links (both local files and HTTP URLs) are valid

**Why it matters**: Broken documentation links frustrate users and erode trust. Dead links suggest unmaintained projects.

**Example failure**:
```
❌ Broken link found: docs/api.md -> nonexistent-file.md
❌ HTTP 404: https://example.com/deleted-page
```

This catches both local file references that don't exist and external URLs that return 404s. Documentation is code—it must be tested.

### 1. Code Formatting

**Command**: `cargo fmt --check`

**What it checks**: Code follows Rust's standard formatting (indentation, spacing, line breaks)

**Why it matters**: Consistent formatting reduces cognitive load and eliminates bike-shedding. Code review focuses on logic, not style.

**Example failure**:
```
Diff in /home/noah/src/pforge/crates/pforge-runtime/src/handler.rs at line 42:
-pub fn new(name:String)->Self{
+pub fn new(name: String) -> Self {
```

Fix: Run `cargo fmt` to auto-format all code.

### 2. Linting (Clippy)

**Command**: `cargo clippy --all-targets --all-features -- -D warnings`

**What it checks**: Common Rust pitfalls, performance issues, API misuse, code smells

**Why it matters**: Clippy's 500+ lints catch bugs and anti-patterns that humans miss. It encodes decades of Rust experience.

**Example failures**:
```
warning: unnecessary clone
  --> src/handler.rs:23:18
   |
23 |     let s = name.clone();
   |                  ^^^^^^^ help: remove this

warning: this returns a `Result<_, ()>`
  --> src/registry.rs:45:5
   |
45 |     Err(())
   |     ^^^^^^^ help: use a custom error type
```

Fix: Address each warning. For rare false positives, use `#[allow(clippy::lint_name)]` with a comment explaining why.

### 3. Tests

**Command**: `cargo test --all`

**What it checks**: All tests (unit, integration, doc tests) pass

**Why it matters**: Failing tests mean broken behavior. A green test suite is your contract with users.

**Example failure**:
```
test handler::tests::test_validation ... FAILED

---- handler::tests::test_validation stdout ----
thread 'handler::tests::test_validation' panicked at 'assertion failed:
  `(left == right)`
  left: `Error("Invalid parameter")`,
  right: `Ok(...)`'
```

Fix: Debug the test. Either the implementation is wrong or the test expectations are incorrect.

### 4. Complexity Analysis

**Command**: `pmat analyze complexity --max-cyclomatic 20`

**What it checks**: Cyclomatic complexity of each function (max: 20)

**Why it matters**: Complex functions are bug-prone, hard to test, and hard to maintain. Studies show defect density increases exponentially with complexity.

**Example failure**:
```
Function 'process_request' has cyclomatic complexity 23 (max: 20)
  Location: src/handler.rs:156
  Recommendation: Extract helper functions or simplify logic
```

Fix: Refactor. Extract functions, eliminate branches, use early returns, leverage Rust's pattern matching.

### 5. SATD Detection (Self-Admitted Technical Debt)

**Command**: `pmat analyze satd`

**What it checks**: TODO, FIXME, HACK, XXX comments (except Phase 2-4 markers)

**Why it matters**: These comments are promises to fix things "later." Later rarely comes. They accumulate into unmaintainable codebases.

**Example failures**:
```
SATD found: TODO: refactor this mess
  Location: src/handler.rs:89
  Severity: Medium

SATD found: HACK: temporary workaround
  Location: src/registry.rs:234
  Severity: High
```

pforge allows Phase markers (`Phase 2: ...`) because they represent planned work, not technical debt.

Fix: Either fix the issue immediately or remove the comment. No deferred promises.

### 6. Code Coverage

**Command**: `cargo llvm-cov --summary-only` (requires ≥80% line coverage)

**What it checks**: Percentage of code exercised by tests

**Why it matters**: Untested code is unverified code. 80% coverage ensures critical paths are tested.

**Example output**:
```
Filename                      Lines    Covered    Uncovered    %
------------------------------------------------------------
src/handler.rs                234      198        36          84.6%
src/registry.rs               189      167        22          88.4%
src/config.rs                 145      109        36          75.2%  ❌
------------------------------------------------------------
TOTAL                         1247     1021       226         81.9%
```

Fix: Add tests for uncovered code paths. Focus on edge cases, error handling, and boundary conditions.

### 7. Technical Debt Grade (TDG)

**Command**: `pmat tdg .` (requires ≥75/100, Grade C or better)

**What it checks**: Holistic code quality score combining complexity, duplication, documentation, test quality, and maintainability

**Why it matters**: TDG provides a single quality metric. It catches issues that slip through individual gates.

**Example output**:
```
╭─────────────────────────────────────────────────╮
│  TDG Score Report                              │
├─────────────────────────────────────────────────┤
│  Overall Score: 94.6/100 (A)                  │
│  Language: Rust (confidence: 98%)               │
│                                                 │
│  Component Scores:                              │
│    Complexity:      92/100                      │
│    Duplication:     96/100                      │
│    Documentation:   91/100                      │
│    Test Quality:    97/100                      │
│    Maintainability: 95/100                      │
╰─────────────────────────────────────────────────╯
```

A score below 75 indicates systemic quality issues. Fix: Address the lowest component scores first.

### 8. Security Audit

**Command**: `cargo audit` (fails on known vulnerabilities)

**What it checks**: Dependencies against the RustSec Advisory Database

**Why it matters**: Vulnerable dependencies create attack vectors. Automated auditing catches CVEs before they reach production.

**Example failure**:
```
Crate:     time
Version:   0.1.43
Warning:   potential segfault in time
ID:        RUSTSEC-2020-0071
Solution:  Upgrade to >= 0.2.23
```

Fix: Update vulnerable dependencies. Use `cargo update` or modify `Cargo.toml`.

## Running Quality Gates

### Manual Execution

Run all gates before committing:

```bash
make quality-gate
```

This executes all eight gates sequentially, stopping at the first failure. Expected output:

```
📝 Formatting code...
✅ Formatting complete!

🔍 Linting code...
✅ Linting complete!

🧪 Running all tests...
✅ All tests passed!

📊 Running comprehensive test coverage analysis...
✅ Coverage: 81.9% (target: ≥80%)

🔬 Running PMAT quality checks...

  1. Complexity Analysis (max: 20)...
     ✅ All functions within complexity limits

  2. SATD Detection (technical debt)...
     ⚠️  6 Phase markers (allowed)
     ✅ No prohibited SATD comments

  3. Technical Debt Grade (TDG)...
     ✅ Score: 94.6/100 (A)

  4. Dead Code Analysis...
     ✅ No dead code detected

✅ All quality gates passed!
```

### Automated Pre-Commit Hooks

pforge installs a pre-commit hook that runs gates automatically:

```bash
git commit -m "Add feature"

🔒 pforge Quality Gate - Pre-Commit Checks
==========================================

🔗 0/8 Validating markdown links...
✓ All markdown links valid

📝 1/8 Checking code formatting...
✓ Formatting passed

🔍 2/8 Running clippy lints...
✓ Clippy passed

🧪 3/8 Running tests...
✓ All tests passed

🔬 4/8 Analyzing code complexity...
✓ Complexity check passed

📋 5/8 Checking for technical debt comments...
✓ Only phase markers present (allowed)

📊 6/8 Checking code coverage...
✓ Coverage ≥80%

📈 7/8 Calculating Technical Debt Grade...
✓ TDG Grade passed

==========================================
✅ Quality Gate PASSED

All quality checks passed. Proceeding with commit.
[main abc1234] Add feature
```

If any gate fails, the commit is blocked:

```bash
git commit -m "Add buggy feature"

...
🔍 2/8 Running clippy lints...
✗ Clippy warnings/errors found

warning: unused variable: `result`
  --> src/handler.rs:23:9

==========================================
❌ Quality Gate FAILED

Fix the issues above and try again.
To bypass (NOT recommended): git commit --no-verify
```

### Bypassing Quality Gates (Emergency Use Only)

In rare emergencies, you can bypass the pre-commit hook:

```bash
git commit --no-verify -m "Hotfix: critical production issue"
```

**Use this sparingly.** Every bypass creates technical debt. Document why the bypass was necessary and create a follow-up task to fix the issues.

## Quality Gate Workflow Integration

Quality gates integrate with pforge's 5-minute TDD cycle:

1. **RED (0:00-2:00)**: Write failing test
2. **GREEN (2:00-4:00)**: Write minimal code to pass test
3. **REFACTOR (4:00-5:00)**: Clean up, run `make quality-gate`
4. **COMMIT (5:00)**: If gates pass, commit. If gates fail, **RESET**.

The binary COMMIT/RESET decision enforces discipline. You must write quality code within the time budget, or discard everything and start over.

This might seem harsh, but it prevents the gradual quality erosion that plagues most projects.

## Customizing Quality Gates

While pforge's default gates work for most projects, you can customize them via `.pmat/quality-gates.yaml`:

```yaml
gates:
  - name: complexity
    max_cyclomatic: 15        # Stricter than default 20
    max_cognitive: 10
    fail_on_violation: true

  - name: satd
    max_count: 0
    fail_on_violation: true

  - name: test_coverage
    min_line_coverage: 85      # Higher than default 80%
    min_branch_coverage: 80
    fail_on_violation: true

  - name: tdg_score
    min_grade: 0.80            # Grade B or better (stricter)
    fail_on_violation: true

  - name: dead_code
    max_count: 0
    fail_on_violation: true    # Make dead code a hard failure

  - name: lints
    fail_on_warnings: true

  - name: formatting
    enforce_rustfmt: true

  - name: security_audit
    fail_on_vulnerabilities: true
```

Stricter gates improve quality but may slow development velocity initially. Find the balance that works for your team.

## Benefits of Quality Gates

After using quality gates consistently, you'll notice:

**Zero Technical Debt Accumulation**: Issues are fixed immediately, not deferred

**Faster Code Reviews**: Reviewers focus on architecture and logic, not style and obvious bugs

**Confident Refactoring**: High test coverage and low complexity make refactoring safe

**Reduced Debugging Time**: Clean code with good tests means fewer production bugs

**New Developer Onboarding**: Enforced standards help newcomers write quality code from day one

**Maintainability**: Low complexity and high test coverage mean the codebase stays maintainable as it grows

## Common Objections

**"Quality gates slow me down!"**

Initially, yes. You'll spend time formatting code, fixing lints, and improving test coverage. But this upfront investment pays exponential dividends. You're moving slower to move faster—preventing the bugs and debt that would slow you down later.

**"My code is good enough without gates!"**

Perhaps. But quality gates are objective and consistent. They catch issues you miss, especially when tired or rushed. They ensure quality remains high even as the team scales.

**"Sometimes I need to bypass gates for urgent work!"**

Use `--no-verify` for true emergencies, but treat each bypass as technical debt that must be repaid. Log why you bypassed, and create a task to fix it.

**"80% coverage is arbitrary!"**

Somewhat. But research shows 70-80% coverage hits diminishing returns—more tests yield less value. 80% is a pragmatic target that catches most issues without excessive test maintenance.

## What's Next?

The next chapters dive deep into specific quality gates:

- **Chapter 8.1**: Pre-commit hooks—automated enforcement
- **Chapter 8.2**: PMAT integration—the tool behind the gates
- **Chapter 8.3**: Complexity analysis—keeping functions simple
- **Chapter 8.4**: Code coverage—measuring test quality

Quality gates transform development from reactive debugging to proactive quality engineering. They embody the Jidoka principle: **build quality in, don't inspect it in later.**

When quality gates become muscle memory, you'll wonder how you ever shipped code without them.
