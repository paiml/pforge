# Conditional Execution

Pipeline steps can execute conditionally based on variable state. This chapter covers condition syntax, patterns, and advanced branching logic.

## Condition Syntax

### Variable Existence

```yaml
steps:
  - tool: fetch_data
    output_var: data

  - tool: process
    condition: "data"  # Execute if 'data' variable exists
```

### Variable Absence

```yaml
steps:
  - tool: primary
    output_var: result
    error_policy: continue

  - tool: fallback
    condition: "!result"  # Execute if 'result' doesn't exist
```

### Nested Variable Access

```yaml
steps:
  - tool: get_user
    output_var: user

  - tool: send_email
    condition: "user.email_verified"  # Access nested field
```

## Conditional Patterns

### Pattern 1: Primary/Fallback

```yaml
steps:
  - tool: fast_cache
    input: { key: "{{key}}" }
    output_var: data
    error_policy: continue

  - tool: slow_database
    input: { key: "{{key}}" }
    output_var: data
    condition: "!data"  # Only if cache miss
```

### Pattern 2: Feature Flags

```yaml
steps:
  - tool: check_feature
    input: { feature: "new_algorithm", user: "{{user_id}}" }
    output_var: feature_enabled

  - tool: new_algorithm
    input: { data: "{{data}}" }
    condition: "feature_enabled"
    output_var: result

  - tool: old_algorithm
    input: { data: "{{data}}" }
    condition: "!feature_enabled"
    output_var: result
```

### Pattern 3: Validation Gates

```yaml
steps:
  - tool: validate_input
    input: { data: "{{raw}}" }
    output_var: validation

  - tool: process_valid
    input: { data: "{{raw}}" }
    condition: "validation.is_valid"

  - tool: handle_invalid
    input: { errors: "{{validation.errors}}" }
    condition: "!validation.is_valid"
```

## Complex Conditions

### Multiple Variables

Current implementation supports simple conditions. For complex logic, use Native handler:

```rust
async fn handle(&self, input: Input) -> Result<Output> {
    let user = fetch_user(&input.user_id).await?;
    let permissions = fetch_permissions(&input.user_id).await?;

    // Complex condition
    if user.is_admin && permissions.can_write && !user.is_suspended {
        return process_admin_request(input).await;
    }

    if permissions.can_read {
        return process_read_request(input).await;
    }

    Err(Error::Unauthorized)
}
```

### Threshold Checks

```yaml
steps:
  - tool: check_balance
    input: { account: "{{account_id}}" }
    output_var: balance

  - tool: high_value_process
    input: { amount: "{{amount}}" }
    condition: "balance.value >= 1000"  # Future feature

  - tool: standard_process
    input: { amount: "{{amount}}" }
    condition: "balance.value < 1000"   # Future feature
```

**Current workaround**: Use validation tool:

```yaml
steps:
  - tool: check_balance
    output_var: balance

  - tool: classify_tier
    input: { balance: "{{balance}}" }
    output_var: tier  # Returns { "is_high_value": true/false }

  - tool: high_value_process
    condition: "tier.is_high_value"

  - tool: standard_process
    condition: "!tier.is_high_value"
```

## Condition Evaluation

### Implementation

```rust
fn evaluate_condition(
    &self,
    condition: &str,
    variables: &HashMap<String, serde_json::Value>,
) -> bool {
    // Simple variable existence check
    if let Some(var_name) = condition.strip_prefix('!') {
        !variables.contains_key(var_name)
    } else {
        variables.contains_key(condition)
    }
}
```

### Nested Field Access (Future)

```rust
fn evaluate_nested_condition(
    condition: &str,
    variables: &HashMap<String, Value>,
) -> bool {
    let parts: Vec<&str> = condition.split('.').collect();

    if let Some(value) = variables.get(parts[0]) {
        // Navigate nested structure
        let mut current = value;
        for part in &parts[1..] {
            match current {
                Value::Object(map) => {
                    if let Some(next) = map.get(*part) {
                        current = next;
                    } else {
                        return false;
                    }
                }
                _ => return false,
            }
        }

        // Check truthiness
        match current {
            Value::Bool(b) => *b,
            Value::Null => false,
            Value::Number(n) => n.as_f64().unwrap_or(0.0) != 0.0,
            Value::String(s) => !s.is_empty(),
            _ => true,
        }
    } else {
        false
    }
}
```

## Error Handling with Conditions

### Graceful Degradation

```yaml
steps:
  - tool: primary_service
    output_var: result
    error_policy: continue

  - tool: secondary_service
    condition: "!result"
    output_var: result
    error_policy: continue

  - tool: cached_fallback
    condition: "!result"
    output_var: result

  - tool: process_result
    input: { data: "{{result}}" }
    condition: "result"
```

### Cleanup Steps

```yaml
steps:
  - tool: allocate_resources
    output_var: resources

  - tool: process_data
    input: { res: "{{resources}}" }
    output_var: result

  # Always cleanup, even on error
  - tool: cleanup_resources
    input: { res: "{{resources}}" }
    condition: "resources"
    error_policy: continue  # Don't fail if cleanup fails
```

## Testing Conditionals

### Test Condition Evaluation

```rust
#[test]
fn test_condition_evaluation() {
    let pipeline = PipelineHandler::new(vec![]);

    let mut vars = HashMap::new();
    vars.insert("exists".to_string(), json!(true));

    assert!(pipeline.evaluate_condition("exists", &vars));
    assert!(!pipeline.evaluate_condition("!exists", &vars));
    assert!(!pipeline.evaluate_condition("missing", &vars));
    assert!(pipeline.evaluate_condition("!missing", &vars));
}
```

### Test Conditional Execution

```rust
#[tokio::test]
async fn test_conditional_step() {
    let registry = HandlerRegistry::new();
    registry.register("tool1", Box::new(MockTool1));
    registry.register("tool2", Box::new(MockTool2));

    let pipeline = PipelineHandler::new(vec![
        PipelineStep {
            tool: "tool1".to_string(),
            output_var: Some("data".to_string()),
            ..Default::default()
        },
        PipelineStep {
            tool: "tool2".to_string(),
            condition: Some("data".to_string()),
            ..Default::default()
        },
    ]);

    let result = pipeline.execute(
        PipelineInput { variables: HashMap::new() },
        &registry
    ).await.unwrap();

    // Both steps should execute
    assert_eq!(result.results.len(), 2);
    assert!(result.results[1].success);
}
```

### Test Skipped Steps

```rust
#[tokio::test]
async fn test_skipped_step() {
    let pipeline = PipelineHandler::new(vec![
        PipelineStep {
            tool: "tool1".to_string(),
            condition: Some("missing_var".to_string()),
            ..Default::default()
        },
    ]);

    let result = pipeline.execute(
        PipelineInput { variables: HashMap::new() },
        &registry
    ).await.unwrap();

    // Step should be skipped
    assert_eq!(result.results.len(), 0);
}
```

## Advanced Patterns

### Retries with Condition

```yaml
steps:
  - tool: attempt_1
    output_var: result
    error_policy: continue

  - tool: wait_retry
    condition: "!result"
    input: { delay_ms: 1000 }

  - tool: attempt_2
    condition: "!result"
    output_var: result
    error_policy: continue

  - tool: final_attempt
    condition: "!result"
    output_var: result
```

### Multi-Path Workflows

```yaml
steps:
  - tool: classify_request
    input: { type: "{{request_type}}" }
    output_var: classification

  # Path A: Urgent requests
  - tool: urgent_handler
    condition: "classification.is_urgent"

  # Path B: Normal requests
  - tool: normal_handler
    condition: "!classification.is_urgent"

  # Path C: Batch requests
  - tool: batch_handler
    condition: "classification.is_batch"
```

## Best Practices

### 1. Explicit Conditions

```yaml
# BAD - implicit
- tool: fallback

# GOOD - explicit
- tool: fallback
  condition: "!primary_result"
```

### 2. Document Branching

```yaml
steps:
  # Try primary source
  - tool: primary_api
    output_var: data
    error_policy: continue

  # Fallback if primary fails
  - tool: fallback_api
    output_var: data
    condition: "!data"
```

### 3. Test All Paths

```rust
#[tokio::test]
async fn test_all_conditional_paths() {
    // Test primary path
    test_with_variables([("feature_enabled", true)]).await;

    // Test fallback path
    test_with_variables([("feature_enabled", false)]).await;

    // Test error path
    test_with_variables([]).await;
}
```

## Next Steps

Chapter 6.3 covers pipeline state management including variable scoping and memory optimization.

---

> "Conditions control flow. Make the flow visible." - pforge conditional principle
