# Pipeline State Management

Pipeline handlers maintain state through variables. This chapter covers variable scoping, memory management, and state persistence patterns.

## Variable Lifecycle

### Creation

Variables are created when tools complete:

```yaml
steps:
  - tool: fetch_data
    output_var: data  # Variable created here
```

### Access

Variables are accessed via interpolation:

```yaml
steps:
  - tool: process
    input:
      payload: "{{data}}"  # Variable accessed here
```

### Persistence

Variables persist through entire pipeline:

```yaml
steps:
  - tool: step1
    output_var: var1

  - tool: step2
    output_var: var2

  - tool: final
    input:
      first: "{{var1}}"   # Still accessible
      second: "{{var2}}"  # Both available
```

## Variable Scoping

### Pipeline Scope

Variables are scoped to the pipeline execution:

```rust
pub struct PipelineOutput {
    pub results: Vec<StepResult>,
    pub variables: HashMap<String, Value>,  // Final state
}
```

### Initial Variables

Input variables seed the pipeline:

```yaml
# Pipeline definition
params:
  user_id: { type: string, required: true }
  config: { type: object, required: false }

# Execution
{
  "variables": {
    "user_id": "123",
    "config": { "debug": true }
  }
}
```

### Variable Shadowing

Later steps can overwrite variables:

```yaml
steps:
  - tool: get_draft
    output_var: document

  - tool: validate
    input: { doc: "{{document}}" }

  - tool: get_final
    output_var: document  # Overwrites previous value
```

## Memory Management

### Variable Storage

```rust
use std::collections::HashMap;
use serde_json::Value;

struct PipelineState {
    variables: HashMap<String, Value>,
}

impl PipelineState {
    fn set(&mut self, key: String, value: Value) {
        self.variables.insert(key, value);
    }

    fn get(&self, key: &str) -> Option<&Value> {
        self.variables.get(key)
    }

    fn size_bytes(&self) -> usize {
        self.variables.iter()
            .map(|(k, v)| {
                k.len() + serde_json::to_vec(v).unwrap().len()
            })
            .sum()
    }
}
```

### Memory Optimization

#### Pattern 1: Drop Unused Variables

```rust
fn cleanup_unused_variables(
    &mut self,
    current_step: usize,
) {
    let future_steps = &self.steps[current_step..];

    self.variables.retain(|var_name, _| {
        // Keep if used in future steps
        future_steps.iter().any(|step| {
            step.uses_variable(var_name)
        })
    });
}
```

#### Pattern 2: Stream Large Data

```yaml
# BAD - store large data in variable
steps:
  - tool: fetch_large_file
    output_var: file_data  # Could be MBs

  - tool: process
    input: { data: "{{file_data}}" }

# GOOD - stream through tool
steps:
  - tool: fetch_and_process
    input: { url: "{{file_url}}" }
    # Tool streams data internally
```

#### Pattern 3: Reference Counting (Future)

```rust
use std::sync::Arc;

struct PipelineState {
    variables: HashMap<String, Arc<Value>>,
}

// Variables shared via Arc, clones are cheap
fn get_variable(&self, key: &str) -> Option<Arc<Value>> {
    self.variables.get(key).cloned()
}
```

## State Persistence

### Stateless Pipelines

Each execution starts fresh:

```yaml
tools:
  - type: pipeline
    name: stateless
    steps:
      - tool: fetch
        output_var: data
      - tool: process
        input: { data: "{{data}}" }

# No state carried between invocations
```

### Stateful Pipelines (Native Handler)

```rust
use std::sync::Arc;
use tokio::sync::RwLock;

pub struct StatefulPipeline {
    cache: Arc<RwLock<HashMap<String, Value>>>,
    pipeline: PipelineHandler,
}

impl StatefulPipeline {
    async fn handle(&self, input: Input) -> Result<Output> {
        let mut variables = input.variables;

        // Inject cached state
        {
            let cache = self.cache.read().await;
            for (k, v) in cache.iter() {
                variables.insert(k.clone(), v.clone());
            }
        }

        // Execute pipeline
        let result = self.pipeline.execute(
            PipelineInput { variables },
            &self.registry,
        ).await?;

        // Update cache with results
        {
            let mut cache = self.cache.write().await;
            for (k, v) in result.variables {
                cache.insert(k, v);
            }
        }

        Ok(result)
    }
}
```

### Persistent State

```rust
use sled::Db;

pub struct PersistentPipeline {
    db: Db,
    pipeline: PipelineHandler,
}

impl PersistentPipeline {
    async fn handle(&self, input: Input) -> Result<Output> {
        // Load state from disk
        let mut variables = input.variables;
        for item in self.db.iter() {
            let (key, value) = item?;
            let key = String::from_utf8(key.to_vec())?;
            let value: Value = serde_json::from_slice(&value)?;
            variables.insert(key, value);
        }

        // Execute
        let result = self.pipeline.execute(
            PipelineInput { variables },
            &self.registry,
        ).await?;

        // Save state to disk
        for (key, value) in &result.variables {
            let value_bytes = serde_json::to_vec(value)?;
            self.db.insert(key.as_bytes(), value_bytes)?;
        }

        Ok(result)
    }
}
```

## Variable Interpolation

### Simple Interpolation

```rust
fn interpolate_variables(
    &self,
    template: &Value,
    variables: &HashMap<String, Value>,
) -> Value {
    match template {
        Value::String(s) => {
            let mut result = s.clone();
            for (key, value) in variables {
                let pattern = format!("{{{{{}}}}}", key);
                if let Some(value_str) = value.as_str() {
                    result = result.replace(&pattern, value_str);
                }
            }
            Value::String(result)
        }
        Value::Object(obj) => {
            let mut new_obj = serde_json::Map::new();
            for (k, v) in obj {
                new_obj.insert(k.clone(), self.interpolate_variables(v, variables));
            }
            Value::Object(new_obj)
        }
        Value::Array(arr) => {
            Value::Array(
                arr.iter()
                    .map(|v| self.interpolate_variables(v, variables))
                    .collect()
            )
        }
        other => other.clone(),
    }
}
```

### Nested Interpolation

```yaml
steps:
  - tool: get_user
    output_var: user

  - tool: get_address
    input:
      address_id: "{{user.address_id}}"  # Nested field access
```

## Advanced State Patterns

### Pattern 1: Accumulator

```yaml
steps:
  - tool: fetch_page_1
    output_var: page1

  - tool: fetch_page_2
    output_var: page2

  - tool: merge_pages
    input:
      pages: ["{{page1}}", "{{page2}}"]
    output_var: all_data
```

### Pattern 2: State Machine

```rust
enum PipelineState {
    Init,
    Fetching,
    Processing,
    Complete,
}

async fn stateful_pipeline(&self, input: Input) -> Result<Output> {
    let mut state = PipelineState::Init;
    let mut variables = input.variables;

    loop {
        state = match state {
            PipelineState::Init => {
                // Initialize
                PipelineState::Fetching
            }
            PipelineState::Fetching => {
                let data = fetch_data().await?;
                variables.insert("data".to_string(), data);
                PipelineState::Processing
            }
            PipelineState::Processing => {
                process_data(&variables).await?;
                PipelineState::Complete
            }
            PipelineState::Complete => break,
        }
    }

    Ok(Output { variables })
}
```

### Pattern 3: Checkpoint/Resume

```rust
#[derive(Serialize, Deserialize)]
struct Checkpoint {
    step_index: usize,
    variables: HashMap<String, Value>,
}

async fn resumable_pipeline(
    &self,
    input: Input,
    checkpoint: Option<Checkpoint>,
) -> Result<(Output, Checkpoint)> {
    let start_step = checkpoint.as_ref().map(|c| c.step_index).unwrap_or(0);
    let mut variables = checkpoint
        .map(|c| c.variables)
        .unwrap_or(input.variables);

    for (i, step) in self.steps.iter().enumerate().skip(start_step) {
        let result = self.execute_step(step, &variables).await?;

        if let Some(var) = &step.output_var {
            variables.insert(var.clone(), result);
        }

        // Save checkpoint after each step
        let checkpoint = Checkpoint {
            step_index: i + 1,
            variables: variables.clone(),
        };
        save_checkpoint(&checkpoint)?;
    }

    Ok((Output { variables: variables.clone() }, Checkpoint {
        step_index: self.steps.len(),
        variables,
    }))
}
```

## Testing State Management

### Test Variable Persistence

```rust
#[tokio::test]
async fn test_variable_persistence() {
    let pipeline = PipelineHandler::new(vec![
        PipelineStep {
            tool: "step1".to_string(),
            output_var: Some("var1".to_string()),
            ..Default::default()
        },
        PipelineStep {
            tool: "step2".to_string(),
            output_var: Some("var2".to_string()),
            ..Default::default()
        },
    ]);

    let result = pipeline.execute(
        PipelineInput { variables: HashMap::new() },
        &registry,
    ).await.unwrap();

    assert!(result.variables.contains_key("var1"));
    assert!(result.variables.contains_key("var2"));
}
```

### Test Memory Usage

```rust
#[tokio::test]
async fn test_memory_optimization() {
    let large_data = vec![0u8; 1_000_000];  // 1MB

    let pipeline = PipelineHandler::new(vec![
        PipelineStep {
            tool: "create_large".to_string(),
            output_var: Some("large".to_string()),
            ..Default::default()
        },
        PipelineStep {
            tool: "process".to_string(),
            input: Some(json!({"data": "{{large}}"})),
            ..Default::default()
        },
    ]);

    let initial_memory = get_memory_usage();

    let _result = pipeline.execute(
        PipelineInput { variables: HashMap::new() },
        &registry,
    ).await.unwrap();

    let final_memory = get_memory_usage();
    let leaked = final_memory - initial_memory;

    assert!(leaked < 100_000);  // Less than 100KB leaked
}
```

## Best Practices

### 1. Minimize State

```yaml
# Keep only necessary variables
output_var: result  # Not: output_var: intermediate_step_23_result
```

### 2. Clear Variable Names

```yaml
# BAD
output_var: d

# GOOD
output_var: validated_user_data
```

### 3. Document State Flow

```yaml
steps:
  # Fetch raw data
  - tool: fetch
    output_var: raw

  # Transform (raw -> processed)
  - tool: transform
    input: { data: "{{raw}}" }
    output_var: processed

  # Store (processed only)
  - tool: store
    input: { data: "{{processed}}" }
```

## Conclusion

You've completed the handler type chapters! You now understand:

- **CLI Handlers**: Wrapping shell commands with streaming
- **HTTP Handlers**: Proxying REST APIs with authentication
- **Pipeline Handlers**: Composing tools with state management

These three handler types, combined with Native handlers, provide the full toolkit for building MCP servers with pforge.

---

> "State is memory. Manage it wisely." - pforge state management principle
