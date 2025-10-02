# PMAT: Pragmatic Metrics Analysis Tool

PMAT (Pragmatic Metrics Analysis Tool) is the engine powering pforge's quality gates. It analyzes code quality across multiple dimensions: complexity, technical debt, duplication, documentation, and maintainability.

Where traditional metrics tools generate reports that developers ignore, PMAT enforces standards. It's not just measurement—it's enforcement.

This chapter explains what PMAT is, how it integrates with pforge, how to interpret its output, and how to use it to maintain production-grade code quality.

## What is PMAT?

PMAT is a command-line tool for analyzing code quality metrics. It supports multiple languages (Rust, Python, JavaScript, Go, Java) and provides actionable insights rather than just numbers.

**Design philosophy**: Metrics should drive action, not just inform.

Traditional tools tell you "your code has high complexity." PMAT tells you "function `process_request` at line 89 has complexity 24 (max: 20)—extract helper functions or simplify logic."

### Core Features

**Complexity Analysis**: Measures cyclomatic and cognitive complexity per function
**SATD Detection**: Finds self-admitted technical debt (TODO, FIXME, HACK comments)
**Technical Debt Grade (TDG)**: Holistic quality score (0-100)
**Dead Code Detection**: Identifies unused functions, variables, imports
**Documentation Validation**: Checks for broken markdown links (local files and HTTP)
**Duplication Analysis**: Detects code clones
**Custom Thresholds**: Configurable limits for each metric

### Installation

PMAT is written in Rust and distributed via cargo:

```bash
cargo install pmat
```

Verify installation:

```bash
pmat --version
# pmat 0.3.0
```

pforge projects include PMAT by default. If you're adding it to an existing project:

```bash
# Add to project dependencies
cargo add pmat --dev

# Or install globally
cargo install pmat
```

## PMAT Commands

PMAT provides several analysis commands:

### 1. Complexity Analysis

```bash
pmat analyze complexity [OPTIONS] [PATH]
```

Analyzes cyclomatic and cognitive complexity for all functions.

**Options**:
- `--max-cyclomatic <N>`: Maximum allowed cyclomatic complexity (default: 10)
- `--max-cognitive <N>`: Maximum allowed cognitive complexity (default: 15)
- `--format <FORMAT>`: Output format (summary, json, detailed)
- `--fail-on-violation`: Exit with code 1 if violations found

**Example**:

```bash
pmat analyze complexity --max-cyclomatic 20 --format summary
```

**Output**:

```
# Complexity Analysis Summary

📊 **Files analyzed**: 23
🔧 **Total functions**: 187

## Complexity Metrics

- **Median Cyclomatic**: 3.0
- **Median Cognitive**: 2.0
- **Max Cyclomatic**: 12
- **Max Cognitive**: 15
- **90th Percentile Cyclomatic**: 8
- **90th Percentile Cognitive**: 10

## Violations (0)

✅ All functions within complexity limits (max cyclomatic: 20)
```

If violations exist:

```
## Violations (2)

❌ Function 'handle_authentication' exceeds cyclomatic complexity
   Location: src/auth.rs:145
   Cyclomatic: 24 (max: 20)
   Cognitive: 18 (max: 15)
   Recommendation: Extract helper functions for validation logic

❌ Function 'process_pipeline' exceeds cyclomatic complexity
   Location: src/pipeline.rs:89
   Cyclomatic: 22 (max: 20)
   Cognitive: 16 (max: 15)
   Recommendation: Use match statements instead of nested if-else
```

### 2. SATD Detection

```bash
pmat analyze satd [OPTIONS] [PATH]
```

Finds self-admitted technical debt comments: TODO, FIXME, HACK, XXX, BUG.

**Options**:
- `--format <FORMAT>`: Output format (summary, json, detailed)
- `--severity <LEVEL>`: Minimum severity to report (low, medium, high, critical)
- `--fail-on-violation`: Exit with code 1 if violations found

**Example**:

```bash
pmat analyze satd --format summary
```

**Output**:

```
# SATD Analysis Summary

Found 6 SATD violations in 5 files

Total violations: 6

## Severity Distribution
- Critical: 1
- High: 0
- Medium: 0
- Low: 5

## Top Violations
1. ./crates/pforge-cli/src/commands/dev.rs:8 - Requirement (Low)
   TODO: Implement hot reload functionality

2. ./crates/pforge-runtime/src/state.rs:54 - Requirement (Low)
   TODO: Add Redis backend support

3. ./pforge-book/book/searcher.js:148 - Security (Critical)
   FIXME: Sanitize user input to prevent XSS

4. ./crates/pforge-runtime/src/server.rs:123 - Design (Low)
   TODO: Refactor transport selection logic

5. ./crates/pforge-runtime/src/state.rs:101 - Requirement (Low)
   TODO: Add TTL support for cached items
```

### 3. Technical Debt Grade (TDG)

```bash
pmat tdg [PATH]
```

Calculates a holistic quality score combining complexity, duplication, documentation, test quality, and maintainability.

**Example**:

```bash
pmat tdg .
```

**Output**:

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

## Recommendations

1. **Complexity** (92/100):
   - Consider refactoring functions with cyclomatic complexity > 15
   - 3 functions could benefit from extraction

2. **Documentation** (91/100):
   - Add doc comments to 5 public functions
   - Update outdated README sections

3. **Maintainability** (95/100):
   - Reduce nesting depth in pipeline.rs:parse_config
   - Consider using builder pattern in config.rs
```

TDG grades:
- **90-100 (A)**: Excellent, production-ready
- **75-89 (B)**: Good, minor improvements needed
- **60-74 (C)**: Acceptable, significant improvements recommended
- **Below 60 (D-F)**: Poor, major refactoring required

pforge requires TDG ≥ 75 (Grade C or better).

### 4. Dead Code Analysis

```bash
pmat analyze dead-code [OPTIONS] [PATH]
```

Identifies unused functions, variables, and imports.

**Example**:

```bash
pmat analyze dead-code --format summary
```

**Output**:

```
# Dead Code Analysis

## Summary
- Total files analyzed: 23
- Dead functions: 0
- Unused variables: 0
- Unused imports: 0

✅ No dead code detected
```

### 5. Documentation Link Validation

```bash
pmat validate-docs [OPTIONS] [PATH]
```

Validates all markdown links (local files and HTTP URLs).

**Options**:
- `--fail-on-error`: Exit with code 1 if broken links found
- `--timeout <MS>`: HTTP request timeout in milliseconds (default: 5000)
- `--format <FORMAT>`: Output format (summary, json, detailed)

**Example**:

```bash
pmat validate-docs --fail-on-error
```

**Output** (success):

```
# Documentation Link Validation

📚 Scanned 47 markdown files
🔗 Validated 234 links
✅ All links valid

## Statistics
- Local file links: 156 (100% valid)
- HTTP/HTTPS links: 78 (100% valid)
- Anchor links: 0
```

**Output** (failure):

```
# Documentation Link Validation

❌ Found 3 broken links

## Broken Links

1. docs/api.md:23
   Link: ./nonexistent-file.md
   Error: File not found

2. README.md:89
   Link: https://example.com/deleted-page
   Error: HTTP 404 Not Found

3. docs/architecture.md:145
   Link: ../specs/missing-spec.md
   Error: File not found

## Summary
- Total links: 234
- Valid: 231 (98.7%)
- Broken: 3 (1.3%)

Exit code: 1
```

## PMAT Configuration

Configure PMAT thresholds in `.pmat/quality-gates.yaml`:

```yaml
gates:
  - name: complexity
    max_cyclomatic: 20        # pforge default
    max_cognitive: 15
    fail_on_violation: true

  - name: satd
    max_count: 0              # Zero tolerance for non-phase markers
    fail_on_violation: true
    allowed_patterns:
      - "Phase [234]:"        # Allow phase planning markers

  - name: test_coverage
    min_line_coverage: 80     # Minimum 80% line coverage
    min_branch_coverage: 75   # Minimum 75% branch coverage
    fail_on_violation: true

  - name: tdg_score
    min_grade: 0.75           # Minimum 75/100 (Grade C)
    fail_on_violation: true

  - name: dead_code
    max_count: 0
    fail_on_violation: false  # Warning only, don't block commits

  - name: lints
    fail_on_warnings: true

  - name: formatting
    enforce_rustfmt: true

  - name: security_audit
    fail_on_vulnerabilities: true
```

### Adjusting Thresholds

Different projects have different needs:

**Stricter (e.g., financial systems, medical software)**:

```yaml
gates:
  - name: complexity
    max_cyclomatic: 10        # Stricter than pforge default
    max_cognitive: 8

  - name: test_coverage
    min_line_coverage: 95     # Very high coverage
    min_branch_coverage: 90

  - name: tdg_score
    min_grade: 0.85           # Grade B or better
```

**More Lenient (e.g., prototypes, research projects)**:

```yaml
gates:
  - name: complexity
    max_cyclomatic: 30        # More lenient
    max_cognitive: 20

  - name: test_coverage
    min_line_coverage: 60     # Lower coverage acceptable
    min_branch_coverage: 50

  - name: tdg_score
    min_grade: 0.60           # Grade D acceptable
```

## Understanding PMAT Metrics

### Cyclomatic Complexity

**Definition**: Number of linearly independent paths through code.

**Formula**: `E - N + 2P` where:
- E = edges in control flow graph
- N = nodes in control flow graph
- P = number of connected components (usually 1)

**Simplified**: Count decision points (if, while, for, match) + 1

**Example**:

```rust
// Cyclomatic complexity: 1 (no branches)
fn add(a: i32, b: i32) -> i32 {
    a + b
}

// Cyclomatic complexity: 3
fn classify(age: i32) -> &'static str {
    if age < 13 {        // +1
        "child"
    } else if age < 20 { // +1
        "teenager"
    } else {
        "adult"
    }
}

// Cyclomatic complexity: 5
fn validate(input: &str) -> Result<(), String> {
    if input.is_empty() {           // +1
        return Err("empty".into());
    }
    if input.len() > 100 {          // +1
        return Err("too long".into());
    }
    if !input.chars().all(|c| c.is_alphanumeric()) { // +1
        return Err("invalid chars".into());
    }

    match input.chars().next() {    // +1
        Some('0'..='9') => Err("starts with digit".into()),
        _ => Ok(())
    }
}
```

**Why it matters**: Complexity > 20 indicates:
- Too many execution paths to test thoroughly
- High cognitive load for readers
- Likely to contain bugs
- Hard to modify safely

**How to reduce**:
- Extract functions
- Use early returns
- Leverage Rust's `?` operator
- Replace nested if-else with match

### Cognitive Complexity

**Definition**: Measures how hard code is to understand (not just test).

Unlike cyclomatic complexity, cognitive complexity:
- Penalizes nesting (nested if is worse than flat if)
- Ignores shorthand structures (x && y doesn't add complexity)
- Rewards language features that reduce cognitive load

**Example**:

```rust
// Cyclomatic: 4, Cognitive: 1 (shorthand logical operators)
if x && y && z && w {
    do_something();
}

// Cyclomatic: 4, Cognitive: 7 (nesting adds cognitive load)
if x {          // +1
    if y {      // +2 (nested)
        if z {  // +3 (deeply nested)
            if w { // +4 (very deeply nested)
                do_something();
            }
        }
    }
}
```

**Why it matters**: Cognitive complexity predicts how long it takes to understand code. High cognitive complexity means:
- New developers struggle
- Bugs hide in complex logic
- Refactoring is risky

**How to reduce**:
- Flatten nesting (use early returns)
- Extract complex conditions into named functions
- Use guard clauses
- Leverage pattern matching

### Self-Admitted Technical Debt (SATD)

**Definition**: Comments where developers admit issues need fixing.

Common markers:
- `TODO`: Work to be done
- `FIXME`: Broken code that needs fixing
- `HACK`: Inelegant solution
- `XXX`: Warning or important note
- `BUG`: Known defect

**Example**:

```rust
// TODO: Add input validation
fn process(input: &str) -> String {
    // HACK: This is a temporary workaround
    input.replace("bad", "good")
    // FIXME: Handle Unicode properly
}
```

**Why it matters**: SATD comments are promises. They accumulate into:
- Unmaintainable codebases
- Security vulnerabilities (deferred validation)
- Performance issues (deferred optimization)

pforge's zero-tolerance policy: Fix it now or don't commit it.

**Exception**: Phase markers for planned work:

```rust
// Phase 2: Add Redis caching
// Phase 3: Implement distributed locking
// Phase 4: Add metrics collection
```

These represent roadmap items, not technical debt.

### Technical Debt Grade (TDG)

**Definition**: Composite score (0-100) measuring overall code quality.

**Components**:

1. **Complexity (20%)**: Average cyclomatic and cognitive complexity
2. **Duplication (20%)**: Percentage of duplicated code blocks
3. **Documentation (20%)**: Doc comment coverage and quality
4. **Test Quality (20%)**: Coverage, assertion quality, test maintainability
5. **Maintainability (20%)**: Code organization, modularity, coupling

**Calculation** (simplified):

```
TDG = (complexity_score × 0.2) +
      (duplication_score × 0.2) +
      (documentation_score × 0.2) +
      (test_quality_score × 0.2) +
      (maintainability_score × 0.2)
```

Each component scores 0-100 based on thresholds:

**Complexity Score**:
- Median cyclomatic ≤ 5: 100 points
- Median cyclomatic 6-10: 80 points
- Median cyclomatic 11-15: 60 points
- Median cyclomatic > 15: 40 points

**Duplication Score**:
- Duplication < 3%: 100 points
- Duplication 3-5%: 80 points
- Duplication 6-10%: 60 points
- Duplication > 10%: 40 points

Similar thresholds for other components.

**Why it matters**: TDG catches quality issues that individual metrics miss. A codebase might have low complexity but poor documentation, or great tests but high duplication. TDG reveals the weakest link.

## PMAT in Practice

### Daily Development Workflow

**1. Pre-Development Check**

Before starting work, check current quality:

```bash
pmat tdg .
```

Understand your baseline. TDG at 85? Good. TDG at 65? You're adding to a problematic codebase.

**2. During Development**

Run complexity checks frequently:

```bash
# In watch mode
cargo watch -x test -c "pmat analyze complexity --max-cyclomatic 20"

# Or manually after each function
pmat analyze complexity src/myfile.rs --max-cyclomatic 20
```

Catch complexity early, before it becomes entrenched.

**3. Before Committing**

Run full quality gate:

```bash
make quality-gate
# or
pmat analyze complexity --max-cyclomatic 20 --fail-on-violation
pmat analyze satd --fail-on-violation
pmat tdg .
```

Fix any violations before committing.

**4. Post-Commit Verification**

CI runs the same checks. If local gates passed but CI fails, your environment differs. Align them.

### Refactoring Guidance

PMAT guides refactoring:

**Complexity Violations**

```bash
pmat analyze complexity --format detailed
```

Output shows exactly which functions exceed limits:

```
Function 'handle_request' (src/handler.rs:89)
  Cyclomatic: 24
  Cognitive: 19

  High complexity due to:
  - 12 if statements (8 nested)
  - 3 match expressions
  - 2 for loops

  Recommendations:
  1. Extract validation logic (lines 95-120) into validate_request()
  2. Extract error handling (lines 145-180) into handle_errors()
  3. Use early returns to reduce nesting (lines 200-230)
```

Follow the recommendations. After refactoring:

```bash
pmat analyze complexity src/handler.rs
```

Confirm complexity is now within limits.

**Low TDG Score**

```bash
pmat tdg . --verbose
```

Shows which component drags down the score:

```
Component Scores:
  Complexity:      92/100 ✅
  Duplication:     45/100 ❌  (12% code duplication)
  Documentation:   88/100 ✅
  Test Quality:    91/100 ✅
  Maintainability: 89/100 ✅

Primary issue: Duplication

Duplicated blocks:
1. src/auth.rs:45-67 duplicates src/session.rs:89-111 (23 lines)
2. src/parser.rs:120-145 duplicates src/validator.rs:200-225 (26 lines)

Recommendation: Extract shared logic into common utilities
```

Focus refactoring on duplication to improve TDG.

### CI/CD Integration

Run PMAT in CI to enforce quality:

```yaml
# .github/workflows/quality.yml
name: Quality Gates

on: [push, pull_request]

jobs:
  pmat-checks:
    runs-on: ubuntu-latest

    steps:
      - uses: actions/checkout@v3

      - name: Install Rust
        uses: actions-rs/toolchain@v1
        with:
          toolchain: stable

      - name: Install PMAT
        run: cargo install pmat

      - name: Complexity Check
        run: pmat analyze complexity --max-cyclomatic 20 --fail-on-violation

      - name: SATD Check
        run: pmat analyze satd --fail-on-violation

      - name: TDG Check
        run: |
          SCORE=$(pmat tdg . --format json | jq -r '.score')
          if (( $(echo "$SCORE < 75" | bc -l) )); then
            echo "TDG score $SCORE below minimum 75"
            exit 1
          fi

      - name: Dead Code Check
        run: pmat analyze dead-code --fail-on-violation

      - name: Documentation Links
        run: pmat validate-docs --fail-on-error
```

PRs cannot merge if PMAT checks fail.

## Interpreting PMAT Output

### Green Flags (Good Quality)

```
# Complexity Analysis Summary

📊 **Files analyzed**: 23
🔧 **Total functions**: 187

## Complexity Metrics

- **Median Cyclomatic**: 3.0   ✅ (low)
- **Median Cognitive**: 2.0    ✅ (low)
- **Max Cyclomatic**: 12       ✅ (well below 20)
- **90th Percentile**: 8       ✅ (healthy)

## TDG Score: 94.6/100 (A)     ✅ (excellent)

## SATD: 0 violations           ✅ (clean)

## Dead Code: 0 functions       ✅ (no waste)
```

This codebase is production-ready. Maintain these standards.

### Yellow Flags (Needs Attention)

```
# Complexity Analysis Summary

- **Median Cyclomatic**: 8.0   ⚠️  (rising)
- **Max Cyclomatic**: 19       ⚠️  (approaching limit)
- **90th Percentile**: 15      ⚠️  (many complex functions)

## TDG Score: 78/100 (C+)       ⚠️  (acceptable but declining)

## SATD: 12 violations          ⚠️  (accumulating debt)
```

Quality is declining. Act now before it becomes a red flag:
- Refactor the most complex functions
- Address SATD comments
- Improve the weakest TDG components

### Red Flags (Action Required)

```
# Complexity Analysis Summary

- **Median Cyclomatic**: 15.0  ❌ (very high)
- **Max Cyclomatic**: 34       ❌ (exceeds limit)
- **90th Percentile**: 25      ❌ (systemic complexity)

## TDG Score: 58/100 (D-)       ❌ (poor quality)

## SATD: 47 violations          ❌ (heavy technical debt)

## Dead Code: 23 functions      ❌ (maintenance burden)
```

This codebase has serious quality issues:
- **Stop feature development**
- **Dedicate sprint to quality**
- **Refactor highest complexity functions first**
- **Eliminate dead code**
- **Address all SATD comments**

### Pattern Recognition

**Gradual Decline**:

```
Week 1: TDG 95/100
Week 2: TDG 92/100
Week 3: TDG 88/100
Week 4: TDG 83/100
```

Trend is negative. Intervene before it drops below 75.

**Stable Quality**:

```
Week 1: TDG 88/100
Week 2: TDG 87/100
Week 3: TDG 89/100
Week 4: TDG 88/100
```

Healthy stability. Maintain current practices.

**Recovery**:

```
Week 1: TDG 65/100 (dedicated quality sprint)
Week 2: TDG 72/100 (refactoring)
Week 3: TDG 79/100 (debt reduction)
Week 4: TDG 85/100 (back to healthy)
```

Successful quality recovery. Document lessons learned.

## Troubleshooting PMAT

### "PMAT command not found"

**Solution**: Install PMAT globally:

```bash
cargo install pmat
which pmat  # Verify installation
```

Or add to project:

```bash
cargo add pmat --dev
cargo run --bin pmat -- analyze complexity
```

### "Complexity calculation seems wrong"

**Check**: Ensure you're comparing the right metrics:

```bash
# Cyclomatic complexity
pmat analyze complexity --show-cyclomatic

# Cognitive complexity
pmat analyze complexity --show-cognitive
```

They measure different things. A function can have low cyclomatic but high cognitive complexity (deep nesting).

### "TDG score unexpectedly low"

**Debug**: Get detailed component breakdown:

```bash
pmat tdg . --verbose
```

Find which component scores lowest. Focus improvement there.

### "SATD detection misses comments"

**Check**: PMAT looks for exact patterns:

```rust
// TODO: works          ✅ detected
// todo: works          ✅ detected (case-insensitive)
// Todo: works          ✅ detected
// @TODO works          ❌ not detected (non-standard format)
```

Use standard markers: TODO, FIXME, HACK, XXX, BUG.

### "Link validation fails in CI but passes locally"

**Cause**: Network differences. Local machine can reach internal URLs, CI cannot.

**Solution**: Use `--skip-external` flag in CI:

```bash
pmat validate-docs --fail-on-error --skip-external
```

Or mock external URLs in CI.

## Advanced PMAT Usage

### Custom Metrics

Extend PMAT with custom analysis:

```bash
# Combine PMAT with other tools
pmat analyze complexity --format json > complexity.json
pmat tdg . --format json > tdg.json

# Merge reports
jq -s '.[0] + .[1]' complexity.json tdg.json > combined.json
```

### Historical Tracking

Track quality over time:

```bash
# Save metrics daily
echo "$(date),$(pmat tdg . --format json | jq -r '.score')" >> metrics.csv

# Plot trends
gnuplot << EOF
  set datafile separator ","
  set xdata time
  set timefmt "%Y-%m-%d"
  plot 'metrics.csv' using 1:2 with lines title 'TDG Score'
EOF
```

### Automated Refactoring

Use PMAT to prioritize refactoring:

```bash
# Find most complex functions
pmat analyze complexity --format json | \
  jq -r '.functions | sort_by(.cyclomatic) | reverse | .[0:5]'

# Output: Top 5 most complex functions
# Refactor these first for maximum impact
```

## Summary

PMAT transforms quality from aspiration to enforcement. It:

- **Measures** complexity, debt, and maintainability objectively
- **Enforces** thresholds via fail-on-violation flags
- **Guides** refactoring with specific recommendations
- **Tracks** quality trends over time

pforge integrates PMAT into every commit via pre-commit hooks and CI checks. This ensures code quality never regresses.

Key takeaways:

1. **Cyclomatic complexity > 20**: Refactor immediately
2. **TDG < 75**: Quality is below acceptable threshold
3. **SATD comments**: Fix or remove, don't defer
4. **Broken doc links**: Documentation is code, test it

The next chapter explores **complexity analysis** in depth, showing how to identify, measure, and reduce code complexity systematically.
