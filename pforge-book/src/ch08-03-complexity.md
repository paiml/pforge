# Complexity Analysis: Keeping Functions Simple

Complex code kills projects. It hides bugs, slows development, and makes maintenance impossible. Studies show defect density increases exponentially with cyclomatic complexity—functions with complexity > 20 are 10x more likely to contain bugs.

pforge enforces a strict complexity limit: **cyclomatic complexity ≤ 20 per function**. This isn't arbitrary—it's based on decades of software engineering research showing that complexity beyond this threshold makes code unmaintainable.

This chapter explains how complexity is measured, why it matters, how to identify complex code, and most importantly—how to simplify it.

## What is Complexity?

Complexity measures how hard code is to understand, test, and modify. pforge tracks two types:

### Cyclomatic Complexity

**Definition**: The number of linearly independent paths through a function's source code.

**Simplified calculation**: Count the number of decision points (if, while, for, match, &&, ||) and add 1.

**Example**:

```rust
// Complexity: 1 (straight-line code, no decisions)
fn add(a: i32, b: i32) -> i32 {
    a + b
}

// Complexity: 2 (one decision point)
fn abs(x: i32) -> i32 {
    if x < 0 {  // +1
        -x
    } else {
        x
    }
}

// Complexity: 4 (three decision points)
fn classify(age: i32) -> &'static str {
    if age < 0 {          // +1
        "invalid"
    } else if age < 13 {  // +1
        "child"
    } else if age < 20 {  // +1
        "teenager"
    } else {
        "adult"
    }
}
```

Each branch creates a new execution path. More paths = more complexity = more tests needed to cover all scenarios.

### Cognitive Complexity

**Definition**: Measures how difficult code is for a human to understand.

Unlike cyclomatic complexity, cognitive complexity:
- **Penalizes nesting**: Deeply nested code is harder to understand
- **Ignores shorthand**: `x && y && z` doesn't add much cognitive load
- **Rewards linear flow**: Sequential code is easier than branching code

**Example**:

```rust
// Cyclomatic: 4, Cognitive: 1
// Short-circuit evaluation is easy to understand
if x && y && z && w {
    do_something();
}

// Cyclomatic: 4, Cognitive: 10
// Nesting increases cognitive load dramatically
if x {          // +1
    if y {      // +2 (nested once)
        if z {  // +3 (nested twice)
            if w { // +4 (nested three times)
                do_something();
            }
        }
    }
}
```

Cognitive complexity better predicts how long it takes to understand code.

## Why Complexity Matters

### Exponential Bug Density

Research by McCabe (1976) and Basili & Perricone (1984) shows:

| Cyclomatic Complexity | Defect Risk |
|-----------------------|-------------|
| 1-10 | Low risk |
| 11-20 | Moderate risk |
| 21-50 | High risk |
| 50+ | Untestable |

Functions with complexity > 20 have **10x higher defect density** than functions with complexity ≤ 10.

### Testing Burden

Cyclomatic complexity equals the minimum number of test cases needed for branch coverage:

```rust
// Complexity: 5
// Requires 5 test cases for full branch coverage
fn validate(input: &str) -> Result<(), String> {
    if input.is_empty() {           // Test case 1
        return Err("empty".into());
    }
    if input.len() > 100 {          // Test case 2
        return Err("too long".into());
    }
    if !input.chars().all(|c| c.is_alphanumeric()) { // Test case 3
        return Err("invalid chars".into());
    }
    match input.chars().next() {
        Some('0'..='9') => Err("starts with digit".into()), // Test case 4
        _ => Ok(())                 // Test case 5
    }
}
```

Complexity 20 requires 20 test cases. Complexity 50 requires 50. High complexity makes thorough testing impractical.

### Comprehension Time

Studies show developers take exponentially longer to understand complex code:

- Complexity 1-5: **2-5 minutes** to understand
- Complexity 6-10: **10-20 minutes** to understand
- Complexity 11-20: **30-60 minutes** to understand
- Complexity > 20: **Hours or days** to understand fully

When onboarding new developers or debugging in production, comprehension speed matters.

### Modification Risk

Making changes to complex code is dangerous:

- **Hard to predict side effects**: Many execution paths mean many places where changes can break things
- **Refactoring is risky**: You can't test all paths, so refactors might introduce bugs
- **Fear of touching code**: Developers avoid modifying complex functions, leading to workarounds and more complexity

## Measuring Complexity

### Using PMAT

Run complexity analysis on your codebase:

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

Healthy codebase:
- **Median < 5**: Most functions are simple
- **Max < 15**: Even the most complex functions are manageable
- **90th percentile < 10**: Only 10% of functions have complexity > 10

### Detailed Analysis

For violations, get detailed output:

```bash
pmat analyze complexity --max-cyclomatic 20 --format detailed
```

**Output**:

```
❌ Function 'process_request' exceeds cyclomatic complexity
   Location: src/handler.rs:156
   Cyclomatic: 24 (max: 20)
   Cognitive: 19

   Breakdown:
   - 8 if statements (4 nested)
   - 3 match expressions
   - 2 for loops
   - 1 while loop

   Recommendations:
   1. Extract validation logic (lines 165-190) → validate_request()
   2. Extract error handling (lines 205-240) → handle_errors()
   3. Use early returns to reduce nesting (lines 250-280)
   4. Replace if-else chain (lines 300-350) with match expression
```

PMAT identifies exactly where complexity comes from and suggests fixes.

### Per-File Analysis

Analyze a specific file:

```bash
pmat analyze complexity src/handler.rs
```

Track complexity during development to catch issues early.

## Identifying Complex Code

### Red Flags

**1. Deep Nesting**

```rust
// BAD: Nesting level 5
fn process(data: &Data) -> Result<String> {
    if data.is_valid() {
        if let Some(user) = data.user() {
            if user.is_active() {
                if let Some(perms) = user.permissions() {
                    if perms.can_read() {
                        // Actual logic buried 5 levels deep
                        return Ok(data.content());
                    }
                }
            }
        }
    }
    Err("Invalid")
}
```

Each nesting level adds cognitive load.

**2. Long Match Expressions**

```rust
// BAD: 15 arms
match command {
    Command::Create => handle_create(),
    Command::Read => handle_read(),
    Command::Update => handle_update(),
    Command::Delete => handle_delete(),
    Command::List => handle_list(),
    Command::Search => handle_search(),
    Command::Filter => handle_filter(),
    Command::Sort => handle_sort(),
    Command::Export => handle_export(),
    Command::Import => handle_import(),
    Command::Validate => handle_validate(),
    Command::Transform => handle_transform(),
    Command::Aggregate => handle_aggregate(),
    Command::Analyze => handle_analyze(),
    Command::Report => handle_report(),
}
```

Each match arm is a decision point. 15 arms = complexity 15.

**3. Boolean Logic Soup**

```rust
// BAD: Complex boolean expression
if (user.is_admin() || user.is_moderator()) &&
   !user.is_banned() &&
   (resource.is_public() || resource.owner() == user.id()) &&
   (time.is_business_hours() || user.has_permission("after_hours")) &&
   !system.is_maintenance_mode() {
    // Allow access
}
```

Each `&&` and `||` adds complexity. This expression has cyclomatic complexity 6 just for the condition.

**4. Loop-within-Loop**

```rust
// BAD: Nested loops with conditions
for user in users {
    if user.is_active() {
        for item in user.items() {
            if item.needs_processing() {
                for dep in item.dependencies() {
                    if dep.is_ready() {
                        process(dep);
                    }
                }
            }
        }
    }
}
```

Nested loops with conditionals create exponential complexity.

**5. Error Handling Maze**

```rust
// BAD: Error handling everywhere
fn complex_operation() -> Result<String> {
    let a = step1().map_err(|e| Error::Step1(e))?;

    if a.needs_validation() {
        validate(&a).map_err(|e| Error::Validation(e))?;
    }

    let b = if a.has_data() {
        step2(&a).map_err(|e| Error::Step2(e))?
    } else {
        default_value()
    };

    match step3(&b) {
        Ok(c) => {
            if c.is_complete() {
                Ok(c.value())
            } else {
                Err(Error::Incomplete)
            }
        }
        Err(e) => {
            if can_retry(&e) {
                retry_step3(&b)
            } else {
                Err(Error::Step3(e))
            }
        }
    }
}
```

Complexity 12 from error handling alone.

## Reducing Complexity

### Strategy 1: Extract Functions

**Before** (complexity 24):

```rust
fn process_request(req: &Request) -> Result<Response> {
    // Validation (complexity +8)
    if req.user.is_empty() {
        return Err(Error::NoUser);
    }
    if req.user.len() > 100 {
        return Err(Error::UserTooLong);
    }
    if !req.user.chars().all(|c| c.is_alphanumeric()) {
        return Err(Error::InvalidUser);
    }
    if req.action.is_empty() {
        return Err(Error::NoAction);
    }

    // Authorization (complexity +6)
    let user = db.get_user(&req.user)?;
    if !user.is_active() {
        return Err(Error::Inactive);
    }
    if user.is_banned() {
        return Err(Error::Banned);
    }
    if !user.has_permission(&req.action) {
        return Err(Error::Forbidden);
    }

    // Processing (complexity +10)
    let result = match req.action.as_str() {
        "read" => db.read(&req.resource),
        "write" => db.write(&req.resource, &req.data),
        "delete" => db.delete(&req.resource),
        "list" => db.list(&req.filter),
        // ... 6 more cases
        _ => Err(Error::UnknownAction)
    }?;

    Ok(Response::new(result))
}
```

**After** (complexity 4):

```rust
fn process_request(req: &Request) -> Result<Response> {
    validate_request(req)?;          // +1
    let user = authorize_request(req)?;  // +1
    let result = execute_action(req, &user)?; // +1
    Ok(Response::new(result))        // +1
}

fn validate_request(req: &Request) -> Result<()> {
    // Complexity 8 isolated in this function
    if req.user.is_empty() {
        return Err(Error::NoUser);
    }
    if req.user.len() > 100 {
        return Err(Error::UserTooLong);
    }
    if !req.user.chars().all(|c| c.is_alphanumeric()) {
        return Err(Error::InvalidUser);
    }
    if req.action.is_empty() {
        return Err(Error::NoAction);
    }
    Ok(())
}

fn authorize_request(req: &Request) -> Result<User> {
    // Complexity 6 isolated here
    let user = db.get_user(&req.user)?;
    if !user.is_active() {
        return Err(Error::Inactive);
    }
    if user.is_banned() {
        return Err(Error::Banned);
    }
    if !user.has_permission(&req.action) {
        return Err(Error::Forbidden);
    }
    Ok(user)
}

fn execute_action(req: &Request, user: &User) -> Result<String> {
    // Complexity 10 isolated here
    match req.action.as_str() {
        "read" => db.read(&req.resource),
        "write" => db.write(&req.resource, &req.data),
        "delete" => db.delete(&req.resource),
        // ...
        _ => Err(Error::UnknownAction)
    }
}
```

**Result**: Main function complexity drops from 24 to 4. Helper functions each have manageable complexity.

### Strategy 2: Early Returns (Guard Clauses)

**Before** (complexity 7, cognitive 10):

```rust
fn process(user: &User, data: &Data) -> Result<String> {
    if user.is_active() {
        if !user.is_banned() {
            if user.has_permission("read") {
                if data.is_valid() {
                    if !data.is_expired() {
                        return Ok(data.content());
                    }
                }
            }
        }
    }
    Err(Error::Forbidden)
}
```

**After** (complexity 7, cognitive 5):

```rust
fn process(user: &User, data: &Data) -> Result<String> {
    if !user.is_active() {
        return Err(Error::Inactive);
    }
    if user.is_banned() {
        return Err(Error::Banned);
    }
    if !user.has_permission("read") {
        return Err(Error::Forbidden);
    }
    if !data.is_valid() {
        return Err(Error::InvalidData);
    }
    if data.is_expired() {
        return Err(Error::Expired);
    }

    Ok(data.content())
}
```

**Result**: Same cyclomatic complexity, but cognitive complexity reduced from 10 to 5. Code is linear and easy to follow.

### Strategy 3: Replace Nested If with Match

**Before** (complexity 8):

```rust
fn classify_status(code: i32) -> &'static str {
    if code >= 200 {
        if code < 300 {
            "success"
        } else if code >= 300 {
            if code < 400 {
                "redirect"
            } else if code >= 400 {
                if code < 500 {
                    "client_error"
                } else {
                    "server_error"
                }
            } else {
                "unknown"
            }
        } else {
            "unknown"
        }
    } else {
        "informational"
    }
}
```

**After** (complexity 5):

```rust
fn classify_status(code: i32) -> &'static str {
    match code {
        100..=199 => "informational",
        200..=299 => "success",
        300..=399 => "redirect",
        400..=499 => "client_error",
        500..=599 => "server_error",
        _ => "unknown"
    }
}
```

**Result**: Complexity drops from 8 to 5. Code is clearer and more maintainable.

### Strategy 4: Use Rust's `?` Operator

**Before** (complexity 10):

```rust
fn load_config() -> Result<Config> {
    let file = match File::open("config.yaml") {
        Ok(f) => f,
        Err(e) => return Err(Error::FileOpen(e))
    };

    let mut contents = String::new();
    if let Err(e) = file.read_to_string(&mut contents) {
        return Err(Error::FileRead(e));
    }

    let config: Config = match serde_yaml::from_str(&contents) {
        Ok(c) => c,
        Err(e) => return Err(Error::Parse(e))
    };

    if config.validate().is_err() {
        return Err(Error::Invalid);
    }

    Ok(config)
}
```

**After** (complexity 3):

```rust
fn load_config() -> Result<Config> {
    let mut file = File::open("config.yaml")
        .map_err(Error::FileOpen)?;

    let mut contents = String::new();
    file.read_to_string(&mut contents)
        .map_err(Error::FileRead)?;

    let config: Config = serde_yaml::from_str(&contents)
        .map_err(Error::Parse)?;

    config.validate()
        .map_err(|_| Error::Invalid)?;

    Ok(config)
}
```

**Result**: Complexity drops from 10 to 3 by leveraging `?` operator.

### Strategy 5: Extract Complex Conditions

**Before** (complexity 8):

```rust
fn should_process(user: &User, resource: &Resource, time: &Time) -> bool {
    (user.is_admin() || user.is_moderator()) &&
    !user.is_banned() &&
    (resource.is_public() || resource.owner() == user.id()) &&
    (time.is_business_hours() || user.has_permission("after_hours")) &&
    !system.is_maintenance_mode()
}
```

**After** (complexity 4):

```rust
fn should_process(user: &User, resource: &Resource, time: &Time) -> bool {
    has_required_role(user) &&
    can_access_resource(user, resource) &&
    is_allowed_time(user, time) &&
    !system.is_maintenance_mode()
}

fn has_required_role(user: &User) -> bool {
    (user.is_admin() || user.is_moderator()) && !user.is_banned()
}

fn can_access_resource(user: &User, resource: &Resource) -> bool {
    resource.is_public() || resource.owner() == user.id()
}

fn is_allowed_time(user: &User, time: &Time) -> bool {
    time.is_business_hours() || user.has_permission("after_hours")
}
```

**Result**: Complexity drops from 8 to 4. Named functions document what each condition means.

### Strategy 6: Polymorphism (Strategy Pattern)

**Before** (complexity 15):

```rust
fn handle_command(cmd: &Command) -> Result<Response> {
    match cmd.type {
        "create" => {
            validate_create(&cmd.data)?;
            db.create(&cmd.data)
        }
        "read" => {
            validate_read(&cmd.id)?;
            db.read(&cmd.id)
        }
        "update" => {
            validate_update(&cmd.id, &cmd.data)?;
            db.update(&cmd.id, &cmd.data)
        }
        "delete" => {
            validate_delete(&cmd.id)?;
            db.delete(&cmd.id)
        }
        // 11 more cases...
        _ => Err(Error::Unknown)
    }
}
```

**After** (complexity 2):

```rust
trait CommandHandler {
    fn validate(&self) -> Result<()>;
    fn execute(&self) -> Result<Response>;
}

struct CreateCommand { data: Data }
impl CommandHandler for CreateCommand {
    fn validate(&self) -> Result<()> { validate_create(&self.data) }
    fn execute(&self) -> Result<Response> { db.create(&self.data) }
}

// Similar impls for Read, Update, Delete, etc.

fn handle_command(cmd: Box<dyn CommandHandler>) -> Result<Response> {
    cmd.validate()?;
    cmd.execute()
}
```

**Result**: Complexity drops from 15 to 2. Each command is isolated in its own type.

## Complexity in Practice

### Example: Refactoring a Complex Function

**Initial state** (complexity 28):

```rust
fn authenticate_and_authorize(
    req: &Request,
    db: &Database,
    cache: &Cache
) -> Result<User> {
    // Validation
    if req.token.is_empty() {
        return Err(Error::NoToken);
    }

    // Check cache
    if let Some(cached) = cache.get(&req.token) {
        if !cached.is_expired() {
            if cached.user.is_active() {
                if !cached.user.is_banned() {
                    if cached.user.has_permission(&req.action) {
                        return Ok(cached.user.clone());
                    }
                }
            }
        }
    }

    // Parse token
    let claims = match jwt::decode(&req.token) {
        Ok(c) => c,
        Err(e) => {
            if e.kind() == jwt::ErrorKind::Expired {
                return Err(Error::TokenExpired);
            } else {
                return Err(Error::InvalidToken);
            }
        }
    };

    // Load user
    let user = db.get_user(claims.user_id)?;

    // Validate user
    if !user.is_active() {
        return Err(Error::UserInactive);
    }
    if user.is_banned() {
        return Err(Error::UserBanned);
    }
    if !user.has_permission(&req.action) {
        return Err(Error::Forbidden);
    }

    // Update cache
    cache.set(&req.token, CachedAuth {
        user: user.clone(),
        expires_at: Time::now() + Duration::hours(1)
    });

    Ok(user)
}
```

**Refactored** (main function complexity 4):

```rust
fn authenticate_and_authorize(
    req: &Request,
    db: &Database,
    cache: &Cache
) -> Result<User> {
    validate_request(req)?;

    if let Some(user) = check_cache(req, cache)? {
        return Ok(user);
    }

    let claims = parse_token(&req.token)?;
    let user = load_and_validate_user(claims.user_id, &req.action, db)?;
    update_cache(&req.token, &user, cache);

    Ok(user)
}

fn validate_request(req: &Request) -> Result<()> {
    if req.token.is_empty() {
        return Err(Error::NoToken);
    }
    Ok(())
}

fn check_cache(req: &Request, cache: &Cache) -> Result<Option<User>> {
    if let Some(cached) = cache.get(&req.token) {
        if cached.is_expired() {
            return Ok(None);
        }

        validate_user_access(&cached.user, &req.action)?;
        return Ok(Some(cached.user.clone()));
    }

    Ok(None)
}

fn parse_token(token: &str) -> Result<Claims> {
    jwt::decode(token).map_err(|e| {
        match e.kind() {
            jwt::ErrorKind::Expired => Error::TokenExpired,
            _ => Error::InvalidToken
        }
    })
}

fn load_and_validate_user(
    user_id: UserId,
    action: &str,
    db: &Database
) -> Result<User> {
    let user = db.get_user(user_id)?;
    validate_user_access(&user, action)?;
    Ok(user)
}

fn validate_user_access(user: &User, action: &str) -> Result<()> {
    if !user.is_active() {
        return Err(Error::UserInactive);
    }
    if user.is_banned() {
        return Err(Error::UserBanned);
    }
    if !user.has_permission(action) {
        return Err(Error::Forbidden);
    }
    Ok(())
}

fn update_cache(token: &str, user: &User, cache: &Cache) {
    cache.set(token, CachedAuth {
        user: user.clone(),
        expires_at: Time::now() + Duration::hours(1)
    });
}
```

**Result**:
- Main function: 28 → 4 (85% reduction)
- All helper functions: < 10 complexity
- Code is testable, readable, maintainable

### When Complexity is Unavoidable

Sometimes high complexity is inherent to the problem:

```rust
// Parser for complex grammar - complexity 25
fn parse_expression(tokens: &[Token]) -> Result<Expr> {
    // Inherently complex: operator precedence, associativity,
    // parentheses, function calls, array access, etc.
    // This complexity reflects problem complexity, not poor design
}
```

**Solutions**:
1. **Accept it, but document**: Add extensive comments explaining the logic
2. **Comprehensive tests**: Ensure every path is tested
3. **Isolate it**: Keep complex logic in dedicated modules
4. **Consider alternatives**: Maybe a parser generator library would simplify this

## Tracking Complexity Trends

Monitor complexity over time:

```bash
# Daily complexity snapshot
echo "$(date),$(pmat analyze complexity --format json | jq -r '.max_cyclomatic')" >> complexity.csv
```

Plot trends to catch regressions early:

```bash
# Visualize complexity trends
gnuplot << EOF
set terminal png size 800,600
set output 'complexity-trend.png'
set xlabel 'Date'
set ylabel 'Max Cyclomatic Complexity'
set datafile separator ","
set xdata time
set timefmt "%Y-%m-%d"
plot 'complexity.csv' using 1:2 with lines title 'Max Complexity'
EOF
```

If max complexity trends upward, intervene before it exceeds 20.

## Complexity Budget

Treat complexity like memory or performance—you have a budget:

**Project-level budget**:
- Total cyclomatic complexity for all functions: **< 500**
- Median complexity: **< 5**
- Max complexity: **< 20**

If adding a new function would exceed the budget, refactor existing code first.

## Summary

Complexity kills maintainability. pforge enforces cyclomatic complexity ≤ 20 per function to prevent unmaintainable code.

**Key techniques to reduce complexity**:

1. **Extract functions**: Break large functions into focused helpers
2. **Early returns**: Replace nesting with guard clauses
3. **Use match**: Replace nested if-else with pattern matching
4. **Leverage `?`**: Simplify error handling
5. **Extract conditions**: Give complex boolean expressions names
6. **Polymorphism**: Replace switch/match with trait dispatch

**Complexity thresholds**:
- **1-5**: Simple, ideal
- **6-10**: Moderate, acceptable
- **11-20**: Complex, refactor when possible
- **> 20**: Exceeds pforge limit, must refactor

The next chapter covers **code coverage**, showing how to ensure your tests actually test the code you write.
