# HTTP Error Handling

HTTP handlers must gracefully handle network failures, timeouts, and API errors. This chapter covers retry strategies, circuit breakers, and graceful degradation.

## Error Types

### Network Errors

```json
{
  "error": "Http: Connection refused"
}
```

### HTTP Status Errors

HTTP handlers return status codes, not errors:

```json
{
  "status": 404,
  "body": { "message": "Not Found" },
  "headers": {...}
}
```

**Client handles status**:
```javascript
if (result.status >= 400) {
  throw new APIError(result.status, result.body);
}
```

### Timeout Errors

```json
{
  "error": "Timeout: Request exceeded 30000ms"
}
```

## Retry Strategies

### Exponential Backoff (Native Handler)

```rust
use backoff::{ExponentialBackoff, Error as BackoffError};
use std::time::Duration;

async fn handle_with_retry(&self, input: Input) -> Result<Output> {
    let backoff = ExponentialBackoff {
        initial_interval: Duration::from_millis(100),
        multiplier: 2.0,
        max_interval: Duration::from_secs(30),
        max_elapsed_time: Some(Duration::from_mins(5)),
        ..Default::default()
    };

    backoff::retry(backoff, || async {
        match self.client.get(&input.url).send().await {
            Ok(resp) if resp.status().is_success() => Ok(resp),
            Ok(resp) if resp.status().is_server_error() => {
                // Retry 5xx errors
                Err(BackoffError::transient(Error::Http(...)))
            },
            Ok(resp) => {
                // Don't retry 4xx errors
                Err(BackoffError::permanent(Error::Http(...)))
            },
            Err(e) if e.is_timeout() => {
                // Retry timeouts
                Err(BackoffError::transient(Error::from(e)))
            },
            Err(e) => Err(BackoffError::permanent(Error::from(e))),
        }
    }).await
}
```

### Retry with Jitter

```rust
use rand::Rng;

async fn retry_with_jitter<F, Fut, T>(
    max_attempts: u32,
    base_delay_ms: u64,
    operation: F,
) -> Result<T>
where
    F: Fn() -> Fut,
    Fut: std::future::Future<Output = Result<T>>,
{
    let mut attempt = 0;
    let mut rng = rand::thread_rng();

    loop {
        match operation().await {
            Ok(result) => return Ok(result),
            Err(e) if attempt >= max_attempts - 1 => return Err(e),
            Err(_) => {
                let jitter = rng.gen_range(0..base_delay_ms / 2);
                let delay = (base_delay_ms * 2_u64.pow(attempt)) + jitter;
                tokio::time::sleep(Duration::from_millis(delay)).await;
                attempt += 1;
            }
        }
    }
}
```

## Circuit Breaker Pattern

### Implementation

```rust
use std::sync::Arc;
use tokio::sync::RwLock;
use std::time::{Instant, Duration};

#[derive(Clone)]
enum CircuitState {
    Closed,
    Open { opened_at: Instant },
    HalfOpen,
}

struct CircuitBreaker {
    state: Arc<RwLock<CircuitState>>,
    failure_threshold: u32,
    timeout: Duration,
    failures: Arc<RwLock<u32>>,
}

impl CircuitBreaker {
    async fn call<F, Fut, T>(&self, operation: F) -> Result<T>
    where
        F: FnOnce() -> Fut,
        Fut: std::future::Future<Output = Result<T>>,
    {
        // Check state
        let state = self.state.read().await.clone();

        match state {
            CircuitState::Open { opened_at } => {
                if opened_at.elapsed() > self.timeout {
                    // Transition to HalfOpen
                    *self.state.write().await = CircuitState::HalfOpen;
                } else {
                    return Err(Error::CircuitOpen);
                }
            }
            CircuitState::HalfOpen | CircuitState::Closed => {}
        }

        // Execute operation
        match operation().await {
            Ok(result) => {
                // Success - close circuit
                *self.state.write().await = CircuitState::Closed;
                *self.failures.write().await = 0;
                Ok(result)
            }
            Err(e) => {
                // Failure - increment counter
                let mut failures = self.failures.write().await;
                *failures += 1;

                if *failures >= self.failure_threshold {
                    // Open circuit
                    *self.state.write().await = CircuitState::Open {
                        opened_at: Instant::now(),
                    };
                }

                Err(e)
            }
        }
    }
}
```

### Usage

```rust
let breaker = CircuitBreaker::new(
    5,  // failure_threshold
    Duration::from_secs(60),  // timeout
);

let result = breaker.call(|| async {
    self.client.get(&url).send().await
}).await?;
```

## Fallback Patterns

### Primary/Secondary Endpoints

```rust
async fn handle_with_fallback(&self, input: Input) -> Result<Output> {
    // Try primary endpoint
    match self.client.get(&self.primary_url).send().await {
        Ok(resp) if resp.status().is_success() => {
            return Ok(resp.json().await?);
        }
        Err(e) => {
            tracing::warn!("Primary endpoint failed: {}", e);
        }
        _ => {}
    }

    // Fallback to secondary
    tracing::info!("Using fallback endpoint");
    let resp = self.client.get(&self.fallback_url).send().await?;
    Ok(resp.json().await?)
}
```

### Cached Response Fallback

```rust
use lru::LruCache;
use std::sync::Arc;
use tokio::sync::Mutex;

struct CachedHandler {
    client: Client,
    cache: Arc<Mutex<LruCache<String, serde_json::Value>>>,
}

impl CachedHandler {
    async fn handle(&self, input: Input) -> Result<Output> {
        let cache_key = format!("{}-{}", input.resource, input.id);

        // Try API
        match self.client.get(&input.url).send().await {
            Ok(resp) if resp.status().is_success() => {
                let data: serde_json::Value = resp.json().await?;

                // Update cache
                self.cache.lock().await.put(cache_key.clone(), data.clone());

                Ok(Output { data })
            }
            _ => {
                // Fallback to cache
                if let Some(cached) = self.cache.lock().await.get(&cache_key) {
                    tracing::warn!("Using cached response");
                    return Ok(Output { data: cached.clone() });
                }

                Err(Error::Unavailable)
            }
        }
    }
}
```

## Rate Limiting

### Token Bucket Implementation

```rust
use std::time::Instant;

struct TokenBucket {
    tokens: f64,
    capacity: f64,
    rate: f64,  // tokens per second
    last_refill: Instant,
}

impl TokenBucket {
    async fn acquire(&mut self) -> Result<()> {
        let now = Instant::now();
        let elapsed = now.duration_since(self.last_refill).as_secs_f64();

        // Refill tokens
        self.tokens = (self.tokens + elapsed * self.rate).min(self.capacity);
        self.last_refill = now;

        if self.tokens >= 1.0 {
            self.tokens -= 1.0;
            Ok(())
        } else {
            let wait_time = ((1.0 - self.tokens) / self.rate) * 1000.0;
            tokio::time::sleep(Duration::from_millis(wait_time as u64)).await;
            self.tokens = 0.0;
            Ok(())
        }
    }
}

// Usage
async fn handle(&self, input: Input) -> Result<Output> {
    self.rate_limiter.lock().await.acquire().await?;
    let resp = self.client.get(&input.url).send().await?;
    Ok(resp.json().await?)
}
```

## Timeout Management

### Adaptive Timeouts

```rust
use std::collections::VecDeque;

struct AdaptiveTimeout {
    latencies: VecDeque<Duration>,
    window_size: usize,
}

impl AdaptiveTimeout {
    fn get_timeout(&self) -> Duration {
        if self.latencies.is_empty() {
            return Duration::from_secs(30);  // Default
        }

        let avg: Duration = self.latencies.iter().sum::<Duration>() / self.latencies.len() as u32;
        avg * 3  // 3x average latency
    }

    fn record(&mut self, latency: Duration) {
        self.latencies.push_back(latency);
        if self.latencies.len() > self.window_size {
            self.latencies.pop_front();
        }
    }
}

async fn handle(&self, input: Input) -> Result<Output> {
    let timeout_duration = self.adaptive_timeout.lock().await.get_timeout();
    let start = Instant::now();

    let result = tokio::time::timeout(
        timeout_duration,
        self.client.get(&input.url).send()
    ).await??;

    self.adaptive_timeout.lock().await.record(start.elapsed());
    Ok(result.json().await?)
}
```

## Error Recovery Patterns

### Pattern 1: Retry-Then-Circuit

```rust
async fn robust_call(&self, input: Input) -> Result<Output> {
    // Try with retries
    let result = retry_with_backoff(3, || async {
        self.client.get(&input.url).send().await
    }).await;

    // If retries exhausted, open circuit
    match result {
        Ok(resp) => Ok(resp.json().await?),
        Err(_) => {
            self.circuit_breaker.open();
            Err(Error::Unavailable)
        }
    }
}
```

### Pattern 2: Parallel Requests

```rust
async fn parallel_fallback(&self, input: Input) -> Result<Output> {
    let primary = self.client.get(&self.primary_url).send();
    let secondary = self.client.get(&self.secondary_url).send();

    // Use first successful response
    tokio::select! {
        Ok(resp) = primary => Ok(resp.json().await?),
        Ok(resp) = secondary => {
            tracing::info!("Used secondary endpoint");
            Ok(resp.json().await?)
        },
        else => Err(Error::Unavailable),
    }
}
```

## Testing Error Scenarios

### Mock Network Failures

```rust
#[tokio::test]
async fn test_retry_on_failure() {
    let mock_server = MockServer::start().await;

    // Fail twice, succeed third time
    mock_server.register_as_sequence(vec![
        Mock::given(method("GET"))
            .respond_with(ResponseTemplate::new(500)),
        Mock::given(method("GET"))
            .respond_with(ResponseTemplate::new(500)),
        Mock::given(method("GET"))
            .respond_with(ResponseTemplate::new(200)
                .set_body_json(json!({"success": true}))),
    ]).await;

    let handler = RetryHandler::new(mock_server.uri(), 3);
    let result = handler.handle(Input {}).await.unwrap();

    assert_eq!(result.data["success"], true);
}
```

## Next Steps

Chapter 6.0 introduces Pipeline handlers for composing multiple tools into workflows.

---

> "Errors are inevitable. Recovery is engineering." - pforge resilience principle
