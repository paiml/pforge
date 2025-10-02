# Pre-Commit Hooks: Automated Quality Enforcement

Pre-commit hooks are Git's mechanism for running automated checks before allowing a commit. They enforce quality standards at the exact moment code enters version control—the last line of defense before technical debt infiltrates your codebase.

pforge uses pre-commit hooks to run all eight quality gates automatically. Every commit must pass these gates. No exceptions (unless you use `--no-verify`, which you shouldn't).

This chapter explains how pforge's pre-commit hooks work, how to install them, how to debug failures, and how to customize them for your workflow.

## The Pre-Commit Workflow

Here's what happens when you attempt to commit:

1. **You run**: `git commit -m "Your message"`
2. **Git triggers**: `.git/hooks/pre-commit` (if it exists and is executable)
3. **Hook runs**: All quality gate checks sequentially
4. **Hook returns**:
   - **Exit 0** (success): Commit proceeds normally
   - **Exit 1** (failure): Commit is blocked, changes remain staged

The entire process is transparent. You see exactly which checks run and which fail.

## Installing Pre-Commit Hooks

pforge projects come with a pre-commit hook in `.git/hooks/pre-commit`. If you cloned the repository, you already have it. If you're setting up a new project:

### Option 1: Copy from Template

```bash
# From pforge root directory
cp .git/hooks/pre-commit.sample .git/hooks/pre-commit
chmod +x .git/hooks/pre-commit
```

### Option 2: Create Manually

Create `.git/hooks/pre-commit`:

```bash
#!/bin/bash
# pforge pre-commit hook - PMAT Quality Gate Enforcement

set -e

echo "🔒 pforge Quality Gate - Pre-Commit Checks"
echo "=========================================="

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Track overall status
FAIL=0

# 0. Markdown Link Validation
echo ""
echo "🔗 0/8 Validating markdown links..."
if command -v pmat &> /dev/null; then
    if pmat validate-docs --fail-on-error > /dev/null 2>&1; then
        echo -e "${GREEN}✓${NC} All markdown links valid"
    else
        echo -e "${RED}✗${NC} Broken markdown links found"
        pmat validate-docs --fail-on-error
        FAIL=1
    fi
else
    echo -e "${YELLOW}⚠${NC}  pmat not installed, skipping link validation"
    echo "   Install: cargo install pmat"
fi

# 1. Code Formatting
echo ""
echo "📝 1/8 Checking code formatting..."
if cargo fmt --check --quiet; then
    echo -e "${GREEN}✓${NC} Formatting passed"
else
    echo -e "${RED}✗${NC} Formatting failed - run: cargo fmt"
    FAIL=1
fi

# 2. Linting
echo ""
echo "🔍 2/8 Running clippy lints..."
if cargo clippy --all-targets --all-features --quiet -- -D warnings 2>&1 | grep -q "warning\|error"; then
    echo -e "${RED}✗${NC} Clippy warnings/errors found"
    cargo clippy --all-targets --all-features -- -D warnings
    FAIL=1
else
    echo -e "${GREEN}✓${NC} Clippy passed"
fi

# 3. Tests
echo ""
echo "🧪 3/8 Running tests..."
if cargo test --quiet --all 2>&1 | grep -q "test result:.*FAILED"; then
    echo -e "${RED}✗${NC} Tests failed"
    cargo test --all
    FAIL=1
else
    echo -e "${GREEN}✓${NC} All tests passed"
fi

# 4. Complexity Analysis
echo ""
echo "🔬 4/8 Analyzing code complexity..."
if pmat analyze complexity --max-cyclomatic 20 --format summary 2>&1 | grep -q "VIOLATION\|exceeds"; then
    echo -e "${RED}✗${NC} Complexity violations found (max: 20)"
    pmat analyze complexity --max-cyclomatic 20
    FAIL=1
else
    echo -e "${GREEN}✓${NC} Complexity check passed"
fi

# 5. SATD Detection
echo ""
echo "📋 5/8 Checking for technical debt comments..."
if pmat analyze satd --format summary 2>&1 | grep -q "TODO\|FIXME\|HACK\|XXX"; then
    echo -e "${YELLOW}⚠${NC}  SATD comments found (Phase 2-4 markers allowed)"
    # Only fail on non-phase markers
    if pmat analyze satd --format summary 2>&1 | grep -v "Phase [234]" | grep -q "TODO\|FIXME\|HACK"; then
        echo -e "${RED}✗${NC} Non-phase SATD comments found"
        pmat analyze satd
        FAIL=1
    else
        echo -e "${GREEN}✓${NC} Only phase markers present (allowed)"
    fi
else
    echo -e "${GREEN}✓${NC} No SATD comments"
fi

# 6. Coverage Check
echo ""
echo "📊 6/8 Checking code coverage..."
if command -v cargo-llvm-cov &> /dev/null; then
    if cargo llvm-cov --summary-only 2>&1 | grep -E "[0-9]+\.[0-9]+%" | awk '{if ($1 < 80.0) exit 1}'; then
        echo -e "${GREEN}✓${NC} Coverage ≥80%"
    else
        echo -e "${RED}✗${NC} Coverage <80% - run: make coverage"
        FAIL=1
    fi
else
    echo -e "${YELLOW}⚠${NC}  cargo-llvm-cov not installed, skipping coverage check"
    echo "   Install: cargo install cargo-llvm-cov"
fi

# 7. TDG Score
echo ""
echo "📈 7/8 Calculating Technical Debt Grade..."
if pmat tdg . 2>&1 | grep -E "Grade: [A-F]" | grep -q "[D-F]"; then
    echo -e "${RED}✗${NC} TDG Grade below threshold (need: C+ or better)"
    pmat tdg .
    FAIL=1
else
    echo -e "${GREEN}✓${NC} TDG Grade passed"
fi

# Summary
echo ""
echo "=========================================="
if [ $FAIL -eq 1 ]; then
    echo -e "${RED}❌ Quality Gate FAILED${NC}"
    echo ""
    echo "Fix the issues above and try again."
    echo "To bypass (NOT recommended): git commit --no-verify"
    exit 1
else
    echo -e "${GREEN}✅ Quality Gate PASSED${NC}"
    echo ""
    echo "All quality checks passed. Proceeding with commit."
    exit 0
fi
```

Make it executable:

```bash
chmod +x .git/hooks/pre-commit
```

### Verifying Installation

Test the hook without committing:

```bash
./.git/hooks/pre-commit
```

You should see the quality gate checks run. If the hook isn't found or isn't executable:

```bash
# Check if file exists
ls -la .git/hooks/pre-commit

# Make executable
chmod +x .git/hooks/pre-commit

# Verify
./.git/hooks/pre-commit
```

## Understanding Hook Output

When you commit, the hook produces detailed output for each gate:

### Successful Run

```bash
git commit -m "feat: add user authentication"

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
[main f3a8c21] feat: add user authentication
 3 files changed, 127 insertions(+), 5 deletions(-)
```

The commit succeeds. Your changes are committed with confidence.

### Failed Run: Formatting

```bash
git commit -m "feat: add broken feature"

🔒 pforge Quality Gate - Pre-Commit Checks
==========================================

🔗 0/8 Validating markdown links...
✓ All markdown links valid

📝 1/8 Checking code formatting...
✗ Formatting failed - run: cargo fmt

==========================================
❌ Quality Gate FAILED

Fix the issues above and try again.
To bypass (NOT recommended): git commit --no-verify
```

The commit is blocked. Fix formatting:

```bash
cargo fmt
git add .
git commit -m "feat: add broken feature"
```

### Failed Run: Tests

```bash
git commit -m "feat: add untested feature"

...
🧪 3/8 Running tests...
✗ Tests failed

running 15 tests
test auth::tests::test_login ... ok
test auth::tests::test_logout ... FAILED
test auth::tests::test_session ... ok
...

failures:

---- auth::tests::test_logout stdout ----
thread 'auth::tests::test_logout' panicked at 'assertion failed:
  `(left == right)`
  left: `Some("user123")`,
  right: `None`'

failures:
    auth::tests::test_logout

test result: FAILED. 14 passed; 1 failed

==========================================
❌ Quality Gate FAILED
```

The commit is blocked. Debug and fix the failing test:

```bash
# Fix the test or implementation
cargo test auth::tests::test_logout

# Once fixed, commit again
git commit -m "feat: add untested feature"
```

### Failed Run: Complexity

```bash
git commit -m "feat: add complex handler"

...
🔬 4/8 Analyzing code complexity...
✗ Complexity violations found (max: 20)

Function 'handle_request' has cyclomatic complexity 24 (max: 20)
  Location: src/handlers/auth.rs:89
  Recommendation: Extract helper functions or simplify logic

==========================================
❌ Quality Gate FAILED
```

The commit is blocked. Refactor to reduce complexity:

```bash
# Refactor the complex function
# Extract helpers, simplify branches
cargo test  # Ensure tests still pass
git add .
git commit -m "feat: add complex handler"
```

### Failed Run: Coverage

```bash
git commit -m "feat: add uncovered code"

...
📊 6/8 Checking code coverage...
✗ Coverage <80% - run: make coverage

Filename                      Lines    Covered    Uncovered    %
------------------------------------------------------------
src/handlers/auth.rs          156      98         58          62.8%
------------------------------------------------------------

==========================================
❌ Quality Gate FAILED
```

The commit is blocked. Add tests to increase coverage:

```bash
# Add tests for uncovered code paths
make coverage  # See detailed coverage report
# Write missing tests
cargo test
git add .
git commit -m "feat: add uncovered code"
```

## Hook Performance

Pre-commit hooks add latency to commits. Here's typical timing:

| Gate | Time (avg) | Notes |
|------|-----------|-------|
| Link validation | ~500ms | Depends on doc count and network for HTTP checks |
| Formatting check | ~100ms | Very fast, just checks diffs |
| Clippy | ~2-5s | First run slow, incremental fast |
| Tests | ~1-10s | Depends on test count and parallelization |
| Complexity | ~300ms | Analyzes function metrics |
| SATD | ~200ms | Text search across codebase |
| Coverage | ~5-15s | Slowest gate, instruments and re-runs tests |
| TDG | ~1-2s | Holistic quality analysis |

**Total**: ~10-35 seconds for a full run.

Slow commits are frustrating, but the alternative—broken code entering the repository—is worse. Over time, you'll appreciate the peace of mind.

### Optimizing Hook Performance

**1. Skip Coverage for Trivial Commits**

Coverage is the slowest gate. For small changes (doc updates, minor refactors), you might skip it:

```bash
# Modify .git/hooks/pre-commit
# Comment out the coverage section for local development
# Or make it conditional:

if [ -z "$SKIP_COVERAGE" ]; then
    # Coverage check here
fi
```

Then:

```bash
SKIP_COVERAGE=1 git commit -m "docs: fix typo"
```

**Caution**: Skipping coverage can let untested code slip through. Use sparingly.

**2. Use Incremental Compilation**

Ensure incremental compilation is enabled in `Cargo.toml`:

```toml
[profile.dev]
incremental = true
```

This speeds up Clippy and test runs by reusing previous compilation artifacts.

**3. Run Checks Manually First**

Before committing, run quality gates manually during development:

```bash
# During TDD cycle
cargo watch -x 'test --lib --quiet' -x 'clippy --quiet'

# Before commit
make quality-gate
git commit -m "Your message"  # Faster, checks already passed
```

The pre-commit hook then serves as a final safety check, not the first discovery of issues.

## Debugging Hook Failures

When a hook fails, follow this debugging workflow:

### 1. Identify Which Gate Failed

The hook output clearly shows which gate failed:

```
🔍 2/8 Running clippy lints...
✗ Clippy warnings/errors found
```

### 2. Run the Gate Manually

Run the failing check outside the hook for better output:

```bash
cargo clippy --all-targets --all-features -- -D warnings
```

### 3. Fix the Issue

Address the specific problem:
- **Formatting**: Run `cargo fmt`
- **Clippy**: Fix warnings or add `#[allow(clippy::...)]`
- **Tests**: Debug failing tests
- **Complexity**: Refactor complex functions
- **SATD**: Remove or fix technical debt comments
- **Coverage**: Add missing tests
- **TDG**: Improve lowest-scoring components

### 4. Verify the Fix

Run the gate again to confirm:

```bash
cargo clippy --all-targets --all-features -- -D warnings
```

### 5. Re-attempt Commit

Once fixed, commit again:

```bash
git add .
git commit -m "Your message"
```

### Common Pitfalls

**Hook Not Running**

If the hook doesn't run at all:

```bash
# Check if file exists
ls -la .git/hooks/pre-commit

# Check if executable
chmod +x .git/hooks/pre-commit

# Verify shebang
head -n1 .git/hooks/pre-commit  # Should be #!/bin/bash
```

**Missing Dependencies**

If the hook fails because `pmat` or `cargo-llvm-cov` isn't installed:

```bash
# Install pmat
cargo install pmat

# Install cargo-llvm-cov
cargo install cargo-llvm-cov
```

The hook gracefully skips checks for missing tools, but you should install them for full protection.

**Staged vs. Unstaged Changes**

The hook runs on **staged changes**, not all changes in your working directory:

```bash
# Only staged changes are checked
git add src/main.rs
git commit -m "Update main"  # Hook checks src/main.rs only

# To check all changes, stage everything
git add .
git commit -m "Update all"
```

## Bypassing the Hook (Emergency Only)

In rare emergencies, bypass the hook with `--no-verify`:

```bash
git commit --no-verify -m "hotfix: critical production bug"
```

**When to bypass:**
- Critical production hotfix where seconds matter
- Hook infrastructure is broken (e.g., pmat server down)
- You're committing known-failing code to share with teammates for debugging

**When NOT to bypass:**
- "I'm in a hurry"
- "I'll fix it in the next commit"
- "The failing test is flaky anyway"
- "Coverage is annoying"

Every bypass creates technical debt. Document why you bypassed and create a follow-up task.

### Logging Bypasses

Add logging to track bypasses:

```bash
# In .git/hooks/pre-commit, at the top:
if [ "$1" = "--no-verify" ]; then
    echo "⚠️  BYPASS: Quality gates skipped" >> .git/bypass.log
    echo "  Date: $(date)" >> .git/bypass.log
    echo "  User: $(git config user.name)" >> .git/bypass.log
    echo "" >> .git/bypass.log
fi
```

Review `.git/bypass.log` periodically. Frequent bypasses indicate process problems.

## Customizing Pre-Commit Hooks

Every project has unique needs. Customize the hook to match your workflow.

### Adding Custom Checks

Add project-specific checks:

```bash
# In .git/hooks/pre-commit, after gate 7:

# 8. Custom Security Audit
echo ""
echo "🔐 8/9 Running security audit..."
if cargo audit 2>&1 | grep -q "error\|vulnerability"; then
    echo -e "${RED}✗${NC} Security vulnerabilities found"
    cargo audit
    FAIL=1
else
    echo -e "${GREEN}✓${NC} No vulnerabilities detected"
fi
```

### Removing Checks

Comment out checks you don't need:

```bash
# Skip SATD for projects that allow TODO comments
# 5. SATD Detection
# echo ""
# echo "📋 5/8 Checking for technical debt comments..."
# ...
```

### Conditional Checks

Run certain checks only in specific contexts:

```bash
# Only check coverage on CI, not locally
if [ -n "$CI" ]; then
    echo ""
    echo "📊 6/8 Checking code coverage..."
    # Coverage check here
fi
```

### Per-Branch Checks

Different branches might have different requirements:

```bash
BRANCH=$(git branch --show-current)

if [ "$BRANCH" = "main" ]; then
    # Strict checks for main
    MIN_COVERAGE=90
else
    # Relaxed checks for feature branches
    MIN_COVERAGE=80
fi
```

### Speed vs. Safety Trade-offs

For faster local development:

```bash
# Quick mode: Skip slow checks
if [ -z "$STRICT" ]; then
    echo "Running quick checks (set STRICT=1 for full checks)"
    # Skip coverage and TDG
else
    # Full checks
fi
```

Then:

```bash
# Fast commit
git commit -m "wip: quick iteration"

# Strict commit
STRICT=1 git commit -m "feat: ready for review"
```

## Integration with CI/CD

Pre-commit hooks provide local enforcement. CI/CD provides remote enforcement.

### Dual Enforcement Strategy

Run the same checks in both places:

**Locally** (`.git/hooks/pre-commit`):
- Fast feedback
- Prevent bad commits
- Developer-friendly

**CI** (`.github/workflows/quality.yml`):
- Mandatory for PRs
- Can't be bypassed
- Enforces team standards

### Keeping Them in Sync

Define checks once, use everywhere:

```bash
# scripts/quality-checks.sh
#!/bin/bash

cargo fmt --check
cargo clippy -- -D warnings
cargo test --all
pmat analyze complexity --max-cyclomatic 20
pmat analyze satd
cargo llvm-cov --summary-only
pmat tdg .
```

**Pre-commit hook**:

```bash
# .git/hooks/pre-commit
./scripts/quality-checks.sh || exit 1
```

**CI workflow**:

```yaml
# .github/workflows/quality.yml
- name: Quality Gates
  run: ./scripts/quality-checks.sh
```

Now local and CI use identical checks.

## Team Adoption Strategies

Introducing pre-commit hooks to a team requires buy-in:

### 1. Start Optional

Make hooks opt-in initially:

```bash
# Add to README.md
## Optional: Install Pre-Commit Hooks

cp scripts/pre-commit .git/hooks/pre-commit
chmod +x .git/hooks/pre-commit
```

As developers see the value, adoption grows organically.

### 2. Gradual Rollout

Enable checks incrementally:

**Week 1**: Formatting and linting only
**Week 2**: Add tests
**Week 3**: Add complexity and SATD
**Week 4**: Add coverage and TDG

This avoids overwhelming the team.

### 3. Make Bypasses Visible

Require documentation for bypasses:

```bash
git commit --no-verify -m "hotfix: production down"

# Then immediately create a task:
# TODO: Address quality gate failures from hotfix commit abc1234
```

### 4. Celebrate Wins

Highlight how hooks catch bugs:

"Pre-commit hook caught an unused variable that would have caused a production error. Quality gates work!"

Positive reinforcement encourages adoption.

## Advanced Hook Patterns

### Selective Execution

Run expensive checks only for specific files:

```bash
# Get changed files
FILES=$(git diff --cached --name-only --diff-filter=ACM | grep '\.rs$')

if [ -n "$FILES" ]; then
    # Only run coverage if Rust files changed
    echo "Rust files changed, running coverage..."
    cargo llvm-cov --summary-only
fi
```

### Parallel Execution

Run independent checks in parallel:

```bash
# Run formatting and linting in parallel
cargo fmt --check &
FMT_PID=$!

cargo clippy -- -D warnings &
CLIPPY_PID=$!

wait $FMT_PID || FAIL=1
wait $CLIPPY_PID || FAIL=1
```

This can halve hook execution time.

### Progressive Enhancement

Start with warnings, graduate to errors:

```bash
# Phase 1: Warn about complexity
if pmat analyze complexity --max-cyclomatic 20 2>&1 | grep -q "exceeds"; then
    echo "⚠️  Complexity warning (will be enforced next month)"
fi

# Phase 2 (after deadline): Make it an error
# if pmat analyze complexity --max-cyclomatic 20 2>&1 | grep -q "exceeds"; then
#     FAIL=1
# fi
```

## Troubleshooting

### "Hook takes too long!"

**Solution**: Run checks manually during development, not just at commit time:

```bash
# During development
cargo watch -x test -x clippy

# Then commit is fast
git commit -m "..."
```

### "Hook fails but the check passes manually!"

**Solution**: Environment differences. Ensure the hook uses the same environment:

```bash
# In hook, print environment
echo "PATH: $PATH"
echo "Rust version: $(rustc --version)"
```

Match your shell environment.

### "Hook doesn't run at all!"

**Solution**: Ensure Git hooks are enabled:

```bash
git config --get core.hooksPath  # Should be empty or .git/hooks

# If custom hooks path, move hook there
```

### "Hook runs old version of checks!"

**Solution**: The hook is static. Regenerate it after changing quality standards:

```bash
cp scripts/pre-commit .git/hooks/pre-commit
chmod +x .git/hooks/pre-commit
```

Or make the hook call a script that's version-controlled:

```bash
# .git/hooks/pre-commit
#!/bin/bash
exec ./scripts/quality-checks.sh
```

## Summary

Pre-commit hooks are your first line of defense against quality regressions. They:

- **Automate quality enforcement** at the moment of commit
- **Provide immediate feedback** on quality violations
- **Prevent technical debt** from entering the codebase
- **Ensure consistency** across all contributors

pforge's pre-commit hook runs eight quality gates, blocking commits that fail any check. This enforces uncompromising standards and prevents the quality erosion that plagues most projects.

Hooks may slow down commits initially, but the time saved debugging production issues and managing technical debt far outweighs the upfront cost.

The next chapter explores **PMAT**, the tool that powers complexity analysis, SATD detection, and TDG scoring.
