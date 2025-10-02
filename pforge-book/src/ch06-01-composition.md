# Tool Composition

Pipeline handlers chain tools together, passing outputs as inputs. This chapter covers composition patterns, data flow, and error propagation.

## Basic Chaining

### Sequential Execution

```yaml
steps:
  - tool: step1
    input: { id: "{{request_id}}" }
    output_var: result1

  - tool: step2
    input: { data: "{{result1}}" }
    output_var: result2

  - tool: step3
    input: { processed: "{{result2}}" }
```

Execution order: step1 → step2 → step3

### Output Variable Scoping

Variables persist throughout pipeline:

```yaml
steps:
  - tool: fetch
    output_var: data

  - tool: validate
    output_var: validated

  - tool: final
    input:
      original: "{{data}}"      # From step 1
      validated: "{{validated}}" # From step 2
```

## Data Transformation Patterns

### Pattern 1: Extract-Transform-Load (ETL)

```yaml
steps:
  # Extract
  - tool: http_get
    input: { url: "{{source}}" }
    output_var: raw

  # Transform
  - tool: parse_json
    input: { json: "{{raw.body}}" }
    output_var: parsed

  - tool: filter_records
    input: { records: "{{parsed}}", criteria: "{{filter}}" }
    output_var: filtered

  # Load
  - tool: bulk_insert
    input: { data: "{{filtered}}", table: "{{target}}" }
```

### Pattern 2: Fan-Out Aggregation

Use Native handler for parallel execution:

```rust
async fn handle(&self, input: Input) -> Result<Output> {
    let futures = input.ids.iter().map(|id| {
        self.registry.dispatch("fetch_item", json!({ "id": id }))
    });

    let results = futures::future::join_all(futures).await;
    let aggregated = aggregate_results(results)?;

    Ok(Output { data: aggregated })
}
```

### Pattern 3: Map-Reduce

```yaml
# Map phase (Native handler)
- tool: map_items
  input: { items: "{{data}}" }
  output_var: mapped

# Reduce phase
- tool: reduce_results
  input: { mapped: "{{mapped}}" }
  output_var: final
```

## Error Propagation

### Explicit Error Handling

```yaml
steps:
  - tool: risky_operation
    input: { data: "{{input}}" }
    output_var: result
    error_policy: fail_fast  # Stop immediately on error

  - tool: cleanup
    input: { id: "{{request_id}}" }
    # Never executes if risky_operation fails
```

### Graceful Degradation

```yaml
steps:
  - tool: primary_source
    input: { id: "{{id}}" }
    output_var: data
    error_policy: continue  # Don't fail pipeline

  - tool: fallback_source
    input: { id: "{{id}}" }
    output_var: data
    condition: "!data"  # Only if primary failed
```

### Error Recovery

```rust
// In PipelineHandler
async fn execute(&self, input: Input) -> Result<Output> {
    let mut variables = input.variables;
    let mut results = Vec::new();

    for step in &self.steps {
        match self.execute_step(step, &variables).await {
            Ok(output) => {
                if let Some(var) = &step.output_var {
                    variables.insert(var.clone(), output.clone());
                }
                results.push(StepResult {
                    tool: step.tool.clone(),
                    success: true,
                    output: Some(output),
                    error: None,
                });
            }
            Err(e) if step.error_policy == ErrorPolicy::Continue => {
                results.push(StepResult {
                    tool: step.tool.clone(),
                    success: false,
                    output: None,
                    error: Some(e.to_string()),
                });
                continue;
            }
            Err(e) => return Err(e),
        }
    }

    Ok(Output { results, variables })
}
```

## Complex Composition Patterns

### Pattern 1: Conditional Branching

```yaml
steps:
  - tool: check_eligibility
    input: { user_id: "{{user_id}}" }
    output_var: eligible

  - tool: premium_process
    input: { user: "{{user_id}}" }
    condition: "eligible.is_premium"

  - tool: standard_process
    input: { user: "{{user_id}}" }
    condition: "!eligible.is_premium"
```

### Pattern 2: Retry with Backoff

```yaml
steps:
  - tool: attempt_operation
    input: { data: "{{data}}" }
    output_var: result
    error_policy: continue

  - tool: retry_operation
    input: { data: "{{data}}", attempt: 2 }
    condition: "!result"
    error_policy: continue

  - tool: final_retry
    input: { data: "{{data}}", attempt: 3 }
    condition: "!result"
```

### Pattern 3: Data Enrichment

```yaml
steps:
  - tool: get_user
    input: { id: "{{user_id}}" }
    output_var: user

  - tool: get_preferences
    input: { user_id: "{{user_id}}" }
    output_var: prefs

  - tool: get_activity
    input: { user_id: "{{user_id}}" }
    output_var: activity

  - tool: merge_profile
    input:
      user: "{{user}}"
      preferences: "{{prefs}}"
      activity: "{{activity}}"
```

## Testing Composition

### Unit Test: Step Execution

```rust
#[tokio::test]
async fn test_step_execution() {
    let registry = HandlerRegistry::new();
    registry.register("tool1", Box::new(Tool1Handler));
    registry.register("tool2", Box::new(Tool2Handler));

    let pipeline = PipelineHandler::new(vec![
        PipelineStep {
            tool: "tool1".to_string(),
            input: Some(json!({"id": "123"})),
            output_var: Some("result".to_string()),
            condition: None,
            error_policy: ErrorPolicy::FailFast,
        },
        PipelineStep {
            tool: "tool2".to_string(),
            input: Some(json!({"data": "{{result}}"})),
            output_var: None,
            condition: None,
            error_policy: ErrorPolicy::FailFast,
        },
    ]);

    let result = pipeline.execute(
        PipelineInput { variables: HashMap::new() },
        &registry
    ).await.unwrap();

    assert_eq!(result.results.len(), 2);
    assert!(result.results[0].success);
    assert!(result.results[1].success);
}
```

### Integration Test: Full Pipeline

```rust
#[tokio::test]
async fn test_etl_pipeline() {
    let pipeline = build_etl_pipeline();
    let input = PipelineInput {
        variables: [
            ("source_url", json!("https://api.example.com/data")),
            ("target_table", json!("processed_data")),
        ].into(),
    };

    let result = pipeline.execute(input, &registry).await.unwrap();

    // Verify all steps executed
    assert_eq!(result.results.len(), 6);

    // Verify data flow
    assert!(result.variables.contains_key("raw_data"));
    assert!(result.variables.contains_key("cleaned"));
    assert!(result.variables.contains_key("validated"));

    // Verify final result
    let final_step = &result.results.last().unwrap();
    assert!(final_step.success);
}
```

## Performance Optimization

### Parallel Step Execution (Future Enhancement)

```yaml
# Current: Sequential
steps:
  - tool: fetch_user
  - tool: fetch_prefs
  - tool: fetch_activity

# Future: Parallel
parallel_steps:
  - [fetch_user, fetch_prefs, fetch_activity]  # Execute in parallel
  - [merge_data]                                # Wait for all, then execute
```

### Variable Cleanup

```rust
// Clean up unused variables to save memory
fn cleanup_variables(&mut self, current_step: usize) {
    self.variables.retain(|var_name, _| {
        self.is_variable_used_after(var_name, current_step)
    });
}
```

## Best Practices

### 1. Minimize State

```yaml
# BAD - accumulating state
steps:
  - tool: step1
    output_var: data1
  - tool: step2
    output_var: data2
  - tool: step3
    output_var: data3
  # All variables kept in memory

# GOOD - only keep what's needed
steps:
  - tool: step1
    output_var: temp
  - tool: step2
    input: { data: "{{temp}}" }
    output_var: result
  # temp can be dropped
```

### 2. Clear Error Policies

```yaml
# Explicit error handling
steps:
  - tool: critical
    error_policy: fail_fast  # Must succeed

  - tool: optional
    error_policy: continue   # Can fail

  - tool: cleanup
    error_policy: fail_fast  # Must run if reached
```

### 3. Meaningful Variable Names

```yaml
# BAD
output_var: data1

# GOOD
output_var: validated_user_profile
```

## Next Steps

Chapter 6.2 covers conditional execution patterns and complex branching logic.

---

> "Composition is about data flow. Make it explicit." - pforge design principle
