# Chapter 11: Fault Tolerance

This chapter covers pforge's built-in fault tolerance mechanisms, including circuit breakers, retries, exponential backoff, and error recovery patterns.

## Why Fault Tolerance Matters

MCP servers often interact with unreliable external systems:
- Network requests can fail or timeout
- CLI commands might hang
- External APIs may be temporarily unavailable
- Services can become overloaded

pforge provides production-ready fault tolerance patterns out of the box.

## Circuit Breakers

Circuit breakers prevent cascading failures by "opening" when too many errors occur, giving failing services time to recover.

### Circuit Breaker States

```rust
pub enum CircuitState {
    Closed,   // Normal operation - requests pass through
    Open,     // Too many failures - reject requests immediately
    HalfOpen, // Testing recovery - allow limited requests
}
```

**State transitions:**
1. **Closed → Open**: After `failure_threshold` consecutive failures
2. **Open → HalfOpen**: After `timeout` duration elapses
3. **HalfOpen → Closed**: After `success_threshold` consecutive successes
4. **HalfOpen → Open**: On any failure during testing

### Configuration

```yaml
# forge.yaml
forge:
  name: resilient-server
  version: 1.0.0

# Configure circuit breaker globally
fault_tolerance:
  circuit_breaker:
    enabled: true
    failure_threshold: 5      # Open after 5 failures
    timeout: 60s              # Wait 60s before testing recovery
    success_threshold: 2      # Close after 2 successes

tools:
  - type: http
    name: fetch_api
    endpoint: "https://api.example.com/data"
    method: GET
    # Circuit breaker applies automatically
```

### Programmatic Usage

```rust
use pforge_runtime::recovery::{CircuitBreaker, CircuitBreakerConfig};
use std::time::Duration;

// Create circuit breaker
let config = CircuitBreakerConfig {
    failure_threshold: 5,
    timeout: Duration::from_secs(60),
    success_threshold: 2,
};

let circuit_breaker = CircuitBreaker::new(config);

// Use circuit breaker
async fn call_external_service() -> Result<Response> {
    circuit_breaker.call(|| async {
        // Your fallible operation
        external_api_call().await
    }).await
}
```

### Real-World Example

```rust
use pforge_runtime::{Handler, Result, Error};
use pforge_runtime::recovery::{CircuitBreaker, CircuitBreakerConfig};
use std::sync::Arc;
use std::time::Duration;

pub struct ResilientApiHandler {
    circuit_breaker: Arc<CircuitBreaker>,
    http_client: reqwest::Client,
}

impl ResilientApiHandler {
    pub fn new() -> Self {
        let config = CircuitBreakerConfig {
            failure_threshold: 3,
            timeout: Duration::from_secs(30),
            success_threshold: 2,
        };

        Self {
            circuit_breaker: Arc::new(CircuitBreaker::new(config)),
            http_client: reqwest::Client::new(),
        }
    }
}

#[async_trait::async_trait]
impl Handler for ResilientApiHandler {
    type Input = ApiInput;
    type Output = ApiOutput;
    type Error = Error;

    async fn handle(&self, input: Self::Input) -> Result<Self::Output> {
        // Circuit breaker wraps the HTTP call
        let response = self.circuit_breaker.call(|| async {
            let resp = self.http_client
                .get(&input.url)
                .send()
                .await
                .map_err(|e| Error::Handler(format!("HTTP error: {}", e)))?;

            let data = resp.text().await
                .map_err(|e| Error::Handler(format!("Parse error: {}", e)))?;

            Ok(data)
        }).await?;

        Ok(ApiOutput { data: response })
    }
}
```

### Monitoring Circuit Breaker State

```rust
// Get current state
let state = circuit_breaker.get_state().await;

match state {
    CircuitState::Closed => println!("Operating normally"),
    CircuitState::Open => println!("Circuit OPEN - rejecting requests"),
    CircuitState::HalfOpen => println!("Testing recovery"),
}

// Get statistics
let stats = circuit_breaker.get_stats();
println!("Failures: {}", stats.failure_count);
println!("Successes: {}", stats.success_count);
```

## Retry Strategies

pforge supports automatic retries with exponential backoff for transient failures.

### Configuration

```yaml
tools:
  - type: http
    name: fetch_data
    endpoint: "https://api.example.com/data"
    method: GET
    retry:
      max_attempts: 3
      initial_delay: 100ms
      max_delay: 5s
      multiplier: 2.0
      jitter: true
```

### Retry Behavior

```
Attempt 1: immediate
Attempt 2: 100ms delay
Attempt 3: 200ms delay (with jitter: 150-250ms)
Attempt 4: 400ms delay (with jitter: 300-500ms)
```

### Custom Retry Logic

```rust
use pforge_runtime::recovery::RetryPolicy;
use std::time::Duration;

pub struct CustomRetryPolicy {
    max_attempts: usize,
    base_delay: Duration,
}

impl RetryPolicy for CustomRetryPolicy {
    fn should_retry(&self, attempt: usize, error: &Error) -> bool {
        // Only retry on specific errors
        match error {
            Error::Timeout => attempt < self.max_attempts,
            Error::Handler(msg) if msg.contains("503") => true,
            _ => false,
        }
    }

    fn delay(&self, attempt: usize) -> Duration {
        // Exponential backoff: base * 2^attempt
        let multiplier = 2_u32.pow(attempt as u32);
        self.base_delay * multiplier

        // Add jitter to prevent thundering herd
        + Duration::from_millis(rand::random::<u64>() % 100)
    }
}
```

## Fallback Handlers

When all retries fail, fallback handlers provide graceful degradation.

### Configuration

```yaml
tools:
  - type: http
    name: fetch_user_data
    endpoint: "https://api.example.com/users/{{user_id}}"
    method: GET
    fallback:
      type: native
      handler: handlers::UserDataFallback
      # Returns cached or default data
```

### Implementation

```rust
use pforge_runtime::recovery::FallbackHandler;
use serde_json::Value;

pub struct UserDataFallback {
    cache: Arc<DashMap<String, Value>>,
}

impl FallbackHandler for UserDataFallback {
    async fn handle_error(&self, error: Error) -> Result<Value> {
        eprintln!("Primary handler failed: {}, using fallback", error);

        // Try cache first
        if let Some(user_id) = extract_user_id_from_error(&error) {
            if let Some(cached) = self.cache.get(&user_id) {
                return Ok(cached.clone());
            }
        }

        // Return default user data
        Ok(serde_json::json!({
            "id": "unknown",
            "name": "Guest User",
            "email": "guest@example.com",
            "cached": true
        }))
    }
}
```

### Fallback Chain

Multiple fallbacks can be chained:

```yaml
tools:
  - type: http
    name: fetch_data
    endpoint: "https://primary-api.example.com/data"
    method: GET
    fallback:
      - type: http
        endpoint: "https://backup-api.example.com/data"
        method: GET
      - type: native
        handler: handlers::CacheFallback
      - type: native
        handler: handlers::DefaultDataFallback
```

## Timeouts

Prevent indefinite blocking with configurable timeouts.

### Per-Tool Timeouts

```yaml
tools:
  - type: native
    name: slow_operation
    handler:
      path: handlers::SlowOperation
    timeout_ms: 5000  # 5 second timeout

  - type: cli
    name: run_tests
    command: pytest
    args: ["tests/"]
    timeout_ms: 300000  # 5 minute timeout

  - type: http
    name: fetch_api
    endpoint: "https://api.example.com/data"
    method: GET
    timeout_ms: 10000  # 10 second timeout
```

### Programmatic Timeouts

```rust
use pforge_runtime::timeout::with_timeout;
use std::time::Duration;

async fn handle(&self, input: Input) -> Result<Output> {
    let result = with_timeout(
        Duration::from_secs(5),
        async {
            slow_operation(input).await
        }
    ).await?;

    Ok(result)
}
```

### Cascading Timeouts

For pipelines, timeouts cascade:

```yaml
tools:
  - type: pipeline
    name: data_pipeline
    timeout_ms: 30000  # Total pipeline timeout
    steps:
      - tool: extract_data
        timeout_ms: 10000  # Step-specific timeout
      - tool: transform_data
        timeout_ms: 10000
      - tool: load_data
        timeout_ms: 10000
```

## Error Tracking

pforge tracks errors for monitoring and debugging.

### Configuration

```yaml
fault_tolerance:
  error_tracking:
    enabled: true
    max_errors: 1000      # Ring buffer size
    classify_by: type     # Group by error type
```

### Error Classification

```rust
use pforge_runtime::recovery::ErrorTracker;

let tracker = ErrorTracker::new();

// Track errors automatically
tracker.track_error(&Error::Timeout).await;
tracker.track_error(&Error::Handler("Connection reset".into())).await;

// Get statistics
let total = tracker.total_errors();
let by_type = tracker.errors_by_type().await;

println!("Total errors: {}", total);
println!("Timeout errors: {}", by_type.get("timeout").unwrap_or(&0));
println!("Connection errors: {}", by_type.get("connection").unwrap_or(&0));
```

### Custom Error Classification

```rust
impl ErrorTracker {
    fn classify_error(&self, error: &Error) -> String {
        match error {
            Error::Handler(msg) => {
                if msg.contains("timeout") {
                    "timeout".to_string()
                } else if msg.contains("connection") {
                    "connection".to_string()
                } else if msg.contains("429") {
                    "rate_limit".to_string()
                } else if msg.contains("503") {
                    "service_unavailable".to_string()
                } else {
                    "handler_error".to_string()
                }
            }
            Error::Timeout => "timeout".to_string(),
            Error::Validation(_) => "validation".to_string(),
            _ => "unknown".to_string(),
        }
    }
}
```

## Recovery Middleware

Combine fault tolerance patterns with middleware.

### Configuration

```yaml
middleware:
  - type: recovery
    circuit_breaker:
      enabled: true
      failure_threshold: 5
      timeout: 60s
    retry:
      max_attempts: 3
      initial_delay: 100ms
    error_tracking:
      enabled: true
```

### Implementation

```rust
use pforge_runtime::{Middleware, Result};
use pforge_runtime::recovery::{
    RecoveryMiddleware,
    CircuitBreakerConfig,
};
use std::sync::Arc;

pub fn create_recovery_middleware() -> Arc<RecoveryMiddleware> {
    let config = CircuitBreakerConfig {
        failure_threshold: 5,
        timeout: Duration::from_secs(60),
        success_threshold: 2,
    };

    Arc::new(
        RecoveryMiddleware::new()
            .with_circuit_breaker(config)
    )
}

// Use in middleware chain
let mut chain = MiddlewareChain::new();
chain.add(create_recovery_middleware());
```

### Middleware Lifecycle

```rust
#[async_trait::async_trait]
impl Middleware for RecoveryMiddleware {
    async fn before(&self, request: Value) -> Result<Value> {
        // Check circuit breaker state before processing
        if let Some(cb) = &self.circuit_breaker {
            let state = cb.get_state().await;
            if state == CircuitState::Open {
                return Err(Error::Handler(
                    "Circuit breaker is OPEN - service unavailable".into()
                ));
            }
        }
        Ok(request)
    }

    async fn after(&self, _request: Value, response: Value) -> Result<Value> {
        // Record success in circuit breaker
        if let Some(cb) = &self.circuit_breaker {
            cb.on_success().await;
        }
        Ok(response)
    }

    async fn on_error(&self, _request: Value, error: Error) -> Result<Value> {
        // Track error
        self.error_tracker.track_error(&error).await;

        // Record failure in circuit breaker
        if let Some(cb) = &self.circuit_breaker {
            cb.on_failure().await;
        }

        Err(error)
    }
}
```

## Bulkhead Pattern

Isolate failures by limiting concurrent requests per tool.

```yaml
tools:
  - type: http
    name: external_api
    endpoint: "https://api.example.com/data"
    method: GET
    bulkhead:
      max_concurrent: 10
      max_queued: 100
      timeout: 5s
```

Implementation:

```rust
use tokio::sync::Semaphore;
use std::sync::Arc;

pub struct BulkheadHandler {
    semaphore: Arc<Semaphore>,
    inner_handler: Box<dyn Handler>,
}

impl BulkheadHandler {
    pub fn new(max_concurrent: usize, inner: Box<dyn Handler>) -> Self {
        Self {
            semaphore: Arc::new(Semaphore::new(max_concurrent)),
            inner_handler: inner,
        }
    }
}

#[async_trait::async_trait]
impl Handler for BulkheadHandler {
    type Input = Value;
    type Output = Value;
    type Error = Error;

    async fn handle(&self, input: Self::Input) -> Result<Self::Output> {
        // Acquire permit (blocks if at limit)
        let _permit = self.semaphore.acquire().await
            .map_err(|_| Error::Handler("Bulkhead full".into()))?;

        // Execute with limited concurrency
        self.inner_handler.handle(input).await
    }
}
```

## Complete Example: Resilient HTTP Tool

```yaml
# forge.yaml
forge:
  name: resilient-api-server
  version: 1.0.0

fault_tolerance:
  circuit_breaker:
    enabled: true
    failure_threshold: 5
    timeout: 60s
    success_threshold: 2
  error_tracking:
    enabled: true

tools:
  - type: http
    name: fetch_user_data
    description: "Fetch user data with full fault tolerance"
    endpoint: "https://api.example.com/users/{{user_id}}"
    method: GET
    timeout_ms: 10000
    retry:
      max_attempts: 3
      initial_delay: 100ms
      max_delay: 5s
      multiplier: 2.0
      jitter: true
    fallback:
      type: native
      handler: handlers::UserDataFallback
    bulkhead:
      max_concurrent: 20
```

## Testing Fault Tolerance

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_circuit_breaker_opens_on_failures() {
        let config = CircuitBreakerConfig {
            failure_threshold: 3,
            timeout: Duration::from_secs(60),
            success_threshold: 2,
        };

        let cb = CircuitBreaker::new(config);

        // Trigger 3 failures
        for _ in 0..3 {
            let _ = cb.call(|| async {
                Err::<(), _>(Error::Handler("Test error".into()))
            }).await;
        }

        // Circuit should be open
        assert_eq!(cb.get_state().await, CircuitState::Open);

        // Requests should be rejected
        let result = cb.call(|| async { Ok(42) }).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_circuit_breaker_recovers() {
        let config = CircuitBreakerConfig {
            failure_threshold: 2,
            timeout: Duration::from_millis(100),
            success_threshold: 2,
        };

        let cb = CircuitBreaker::new(config);

        // Open circuit
        for _ in 0..2 {
            let _ = cb.call(|| async {
                Err::<(), _>(Error::Handler("Test".into()))
            }).await;
        }

        assert_eq!(cb.get_state().await, CircuitState::Open);

        // Wait for timeout
        tokio::time::sleep(Duration::from_millis(150)).await;

        // Circuit should transition to half-open and allow requests
        let _ = cb.call(|| async { Ok(1) }).await;
        assert_eq!(cb.get_state().await, CircuitState::HalfOpen);

        // One more success should close circuit
        let _ = cb.call(|| async { Ok(2) }).await;
        assert_eq!(cb.get_state().await, CircuitState::Closed);
    }

    #[tokio::test]
    async fn test_retry_with_exponential_backoff() {
        let mut attempt = 0;

        let result = retry_with_backoff(
            3,
            Duration::from_millis(10),
            || async {
                attempt += 1;
                if attempt < 3 {
                    Err(Error::Timeout)
                } else {
                    Ok("success")
                }
            }
        ).await;

        assert_eq!(result.unwrap(), "success");
        assert_eq!(attempt, 3);
    }
}
```

## Best Practices

1. **Set appropriate thresholds**: Don't open circuits too aggressively
2. **Use jitter**: Prevent thundering herd on recovery
3. **Monitor circuit state**: Alert when circuits open frequently
4. **Test failure scenarios**: Chaos engineering for resilience
5. **Combine patterns**: Circuit breaker + retry + fallback
6. **Log failures**: Track patterns for debugging
7. **Graceful degradation**: Always provide fallbacks

## Summary

pforge's fault tolerance features provide production-ready resilience:

- **Circuit Breakers**: Prevent cascading failures
- **Retries**: Handle transient errors automatically
- **Exponential Backoff**: Reduce load on failing services
- **Fallbacks**: Graceful degradation
- **Timeouts**: Prevent indefinite blocking
- **Error Tracking**: Monitor and debug failures
- **Bulkheads**: Isolate failures

These patterns combine to create resilient, production-ready MCP servers.

---

**Next:** [Middleware](ch12-00-middleware.md)
