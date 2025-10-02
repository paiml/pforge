# Data Pipeline: Pipeline Handler Overview

Pipeline handlers compose multiple tools into workflows. This chapter demonstrates building data processing pipelines with conditional execution and state management.

## Why Pipeline Handlers?

**Use pipeline handlers when**:
- Chaining multiple tools together
- Output of one tool feeds input of next
- Conditional execution based on results
- Multi-step workflows with shared state

**Don't use pipeline handlers when**:
- Single tool suffices
- Complex branching logic (use Native)
- Real-time streaming required
- Tools are independent (call separately)

## Example: Data Processing Pipeline

```yaml
forge:
  name: data-pipeline
  version: 0.1.0
  transport: stdio

tools:
  - type: pipeline
    name: process_user_data
    description: "Fetch, validate, transform, and store user data"
    steps:
      - tool: fetch_user
        input:
          user_id: "{{user_id}}"
        output_var: user_data

      - tool: validate_user
        input:
          data: "{{user_data}}"
        output_var: validated
        condition: "user_data"

      - tool: transform_data
        input:
          raw: "{{validated}}"
        output_var: transformed
        condition: "validated"

      - tool: store_data
        input:
          data: "{{transformed}}"
        error_policy: fail_fast
    params:
      user_id:
        type: string
        required: true
```

## Pipeline Anatomy

### Steps

```yaml
steps:
  - tool: step_name        # Tool to execute
    input: {...}           # Input template
    output_var: result     # Store output in variable
    condition: "var_name"  # Execute if variable exists
    error_policy: continue # Or fail_fast
```

### Variable Interpolation

```yaml
steps:
  - tool: get_data
    input:
      id: "{{request_id}}"
    output_var: data

  - tool: process
    input:
      payload: "{{data}}"  # Uses output from previous step
```

### Error Policies

**fail_fast** (default): Stop on first error
```yaml
error_policy: fail_fast
```

**continue**: Skip failed steps, continue pipeline
```yaml
error_policy: continue
```

## Complete Pipeline Example

```yaml
tools:
  # Individual tools
  - type: http
    name: fetch_weather
    endpoint: "https://api.weather.com/{{city}}"
    method: GET
    params:
      city: { type: string, required: true }

  - type: native
    name: parse_weather
    handler:
      path: handlers::parse_weather
    params:
      raw_data: { type: object, required: true }

  - type: http
    name: send_notification
    endpoint: "https://notify.example.com/send"
    method: POST
    body:
      message: "{{message}}"
    params:
      message: { type: string, required: true }

  # Pipeline composing them
  - type: pipeline
    name: weather_alert
    description: "Fetch weather and send alerts if needed"
    steps:
      - tool: fetch_weather
        input:
          city: "{{city}}"
        output_var: raw_weather

      - tool: parse_weather
        input:
          raw_data: "{{raw_weather}}"
        output_var: weather
        condition: "raw_weather"

      - tool: send_notification
        input:
          message: "Alert: {{weather.condition}} in {{city}}"
        condition: "weather.is_alert"
        error_policy: continue

    params:
      city: { type: string, required: true }
```

## Pipeline Execution Flow

```
Input: { "city": "Boston" }
  ↓
Step 1: fetch_weather(city="Boston")
  → Output: { "temp": 32, "condition": "snow" }
  → Store in: raw_weather
  ↓
Step 2: parse_weather(raw_data=raw_weather)
  → Condition: raw_weather exists ✓
  → Output: { "is_alert": true, "condition": "Heavy Snow" }
  → Store in: weather
  ↓
Step 3: send_notification(message="Alert: Heavy Snow in Boston")
  → Condition: weather.is_alert=true ✓
  → Output: { "sent": true }
  ↓
Pipeline Result: { "results": [...], "variables": {...} }
```

## Input/Output Structure

### Pipeline Input

```json
{
  "variables": {
    "city": "Boston",
    "user_id": "123"
  }
}
```

### Pipeline Output

```json
{
  "results": [
    {
      "tool": "fetch_weather",
      "success": true,
      "output": { "temp": 32, "condition": "snow" },
      "error": null
    },
    {
      "tool": "parse_weather",
      "success": true,
      "output": { "is_alert": true },
      "error": null
    },
    {
      "tool": "send_notification",
      "success": true,
      "output": { "sent": true },
      "error": null
    }
  ],
  "variables": {
    "city": "Boston",
    "raw_weather": {...},
    "weather": {...}
  }
}
```

## Error Handling

### Fail Fast (Default)

```yaml
steps:
  - tool: critical_step
    input: {...}
    # Implicit: error_policy: fail_fast

  - tool: next_step
    input: {...}
    # Won't execute if critical_step fails
```

### Continue on Error

```yaml
steps:
  - tool: optional_step
    input: {...}
    error_policy: continue  # Pipeline continues even if this fails

  - tool: final_step
    input: {...}
    # Executes regardless of optional_step outcome
```

## Real-World Example: ETL Pipeline

```yaml
tools:
  - type: pipeline
    name: etl_pipeline
    description: "Extract, Transform, Load data pipeline"
    steps:
      # Extract
      - tool: extract_from_api
        input:
          endpoint: "{{source_url}}"
          api_key: "{{api_key}}"
        output_var: raw_data
        error_policy: fail_fast

      # Transform
      - tool: clean_data
        input:
          data: "{{raw_data}}"
        output_var: cleaned
        condition: "raw_data"

      - tool: enrich_data
        input:
          data: "{{cleaned}}"
        output_var: enriched
        condition: "cleaned"

      - tool: aggregate_data
        input:
          data: "{{enriched}}"
        output_var: aggregated
        condition: "enriched"

      # Load
      - tool: validate_schema
        input:
          data: "{{aggregated}}"
        output_var: validated
        error_policy: fail_fast

      - tool: load_to_database
        input:
          data: "{{validated}}"
          table: "{{target_table}}"
        error_policy: fail_fast

      # Notify
      - tool: send_success_notification
        input:
          message: "ETL completed: {{aggregated.count}} records"
        error_policy: continue

    params:
      source_url: { type: string, required: true }
      api_key: { type: string, required: true }
      target_table: { type: string, required: true }
```

## Performance Characteristics

| Metric | Value |
|--------|-------|
| Dispatch overhead | 50-100μs per step |
| Variable lookup | O(1) HashMap |
| Condition evaluation | < 1μs |
| State memory | ~100B per variable |

## When to Use Native vs Pipeline

**Pipeline Handler** - Linear workflows:
```yaml
type: pipeline
steps:
  - tool: fetch
  - tool: process
  - tool: store
```

**Native Handler** - Complex logic:
```rust
async fn handle(&self, input: Input) -> Result<Output> {
    let data = fetch().await?;

    if data.requires_processing() {
        let processed = complex_transform(data)?;
        store(processed).await?;
    } else {
        quick_store(data).await?;
    }

    Ok(Output { ... })
}
```

## Next Steps

Chapter 6.1 covers tool composition patterns, including parallel execution and error propagation.

---

> "Pipelines compose tools. Tools compose behavior." - pforge composition principle
