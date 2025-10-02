use crate::{Error, Middleware, Result};
use serde_json::Value;
use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;

/// Circuit breaker states
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CircuitState {
    Closed,   // Normal operation
    Open,     // Failing, reject requests
    HalfOpen, // Testing if service recovered
}

/// Circuit breaker configuration
#[derive(Debug, Clone)]
pub struct CircuitBreakerConfig {
    /// Number of failures before opening circuit
    pub failure_threshold: usize,
    /// Time to wait before attempting recovery
    pub timeout: Duration,
    /// Number of successes needed to close circuit
    pub success_threshold: usize,
}

impl Default for CircuitBreakerConfig {
    fn default() -> Self {
        Self {
            failure_threshold: 5,
            timeout: Duration::from_secs(60),
            success_threshold: 2,
        }
    }
}

/// Circuit breaker for fault tolerance
pub struct CircuitBreaker {
    config: CircuitBreakerConfig,
    state: Arc<RwLock<CircuitState>>,
    failure_count: Arc<AtomicUsize>,
    success_count: Arc<AtomicUsize>,
    last_failure_time: Arc<RwLock<Option<Instant>>>,
}

impl CircuitBreaker {
    pub fn new(config: CircuitBreakerConfig) -> Self {
        Self {
            config,
            state: Arc::new(RwLock::new(CircuitState::Closed)),
            failure_count: Arc::new(AtomicUsize::new(0)),
            success_count: Arc::new(AtomicUsize::new(0)),
            last_failure_time: Arc::new(RwLock::new(None)),
        }
    }

    pub async fn get_state(&self) -> CircuitState {
        *self.state.read().await
    }

    pub async fn call<F, Fut, T>(&self, operation: F) -> Result<T>
    where
        F: FnOnce() -> Fut,
        Fut: std::future::Future<Output = Result<T>>,
    {
        // Check if we should attempt the operation
        let current_state = self.get_state().await;

        if current_state == CircuitState::Open {
            // Check if timeout has elapsed
            if let Some(last_failure) = *self.last_failure_time.read().await {
                if last_failure.elapsed() >= self.config.timeout {
                    // Transition to half-open
                    *self.state.write().await = CircuitState::HalfOpen;
                    self.success_count.store(0, Ordering::SeqCst);
                } else {
                    return Err(Error::Handler("Circuit breaker is OPEN".to_string()));
                }
            }
        }

        // Attempt the operation
        match operation().await {
            Ok(result) => {
                self.on_success().await;
                Ok(result)
            }
            Err(error) => {
                self.on_failure().await;
                Err(error)
            }
        }
    }

    async fn on_success(&self) {
        let state = self.get_state().await;

        match state {
            CircuitState::HalfOpen => {
                let successes = self.success_count.fetch_add(1, Ordering::SeqCst) + 1;
                if successes >= self.config.success_threshold {
                    *self.state.write().await = CircuitState::Closed;
                    self.failure_count.store(0, Ordering::SeqCst);
                    self.success_count.store(0, Ordering::SeqCst);
                }
            }
            CircuitState::Closed => {
                // Reset failure count on success
                self.failure_count.store(0, Ordering::SeqCst);
            }
            _ => {}
        }
    }

    async fn on_failure(&self) {
        let state = self.get_state().await;

        match state {
            CircuitState::Closed => {
                let failures = self.failure_count.fetch_add(1, Ordering::SeqCst) + 1;
                if failures >= self.config.failure_threshold {
                    *self.state.write().await = CircuitState::Open;
                    *self.last_failure_time.write().await = Some(Instant::now());
                }
            }
            CircuitState::HalfOpen => {
                // Any failure in half-open state immediately opens circuit
                *self.state.write().await = CircuitState::Open;
                *self.last_failure_time.write().await = Some(Instant::now());
                self.failure_count
                    .store(self.config.failure_threshold, Ordering::SeqCst);
            }
            _ => {}
        }
    }

    pub fn get_stats(&self) -> CircuitBreakerStats {
        CircuitBreakerStats {
            failure_count: self.failure_count.load(Ordering::SeqCst),
            success_count: self.success_count.load(Ordering::SeqCst),
        }
    }
}

#[derive(Debug, Clone)]
pub struct CircuitBreakerStats {
    pub failure_count: usize,
    pub success_count: usize,
}

/// Fallback handler for error recovery
pub struct FallbackHandler<F, Fut>
where
    F: Fn(Error) -> Fut + Send + Sync,
    Fut: std::future::Future<Output = Result<Value>> + Send,
{
    fallback_fn: F,
    _phantom: std::marker::PhantomData<Fut>,
}

impl<F, Fut> FallbackHandler<F, Fut>
where
    F: Fn(Error) -> Fut + Send + Sync,
    Fut: std::future::Future<Output = Result<Value>> + Send,
{
    pub fn new(fallback_fn: F) -> Self {
        Self {
            fallback_fn,
            _phantom: std::marker::PhantomData,
        }
    }

    pub async fn handle_error(&self, error: Error) -> Result<Value> {
        (self.fallback_fn)(error).await
    }
}

/// Error tracking for monitoring and debugging
pub struct ErrorTracker {
    total_errors: Arc<AtomicU64>,
    errors_by_type: Arc<RwLock<std::collections::HashMap<String, u64>>>,
}

impl ErrorTracker {
    pub fn new() -> Self {
        Self {
            total_errors: Arc::new(AtomicU64::new(0)),
            errors_by_type: Arc::new(RwLock::new(std::collections::HashMap::new())),
        }
    }

    pub async fn track_error(&self, error: &Error) {
        self.total_errors.fetch_add(1, Ordering::SeqCst);

        let error_type = self.classify_error(error);
        let mut errors = self.errors_by_type.write().await;
        *errors.entry(error_type).or_insert(0) += 1;
    }

    fn classify_error(&self, error: &Error) -> String {
        match error {
            Error::Handler(msg) => {
                if msg.contains("timeout") || msg.contains("timed out") {
                    "timeout".to_string()
                } else if msg.contains("connection") {
                    "connection".to_string()
                } else {
                    "handler_error".to_string()
                }
            }
            _ => "unknown".to_string(),
        }
    }

    pub fn total_errors(&self) -> u64 {
        self.total_errors.load(Ordering::SeqCst)
    }

    pub async fn errors_by_type(&self) -> std::collections::HashMap<String, u64> {
        self.errors_by_type.read().await.clone()
    }

    pub async fn reset(&self) {
        self.total_errors.store(0, Ordering::SeqCst);
        self.errors_by_type.write().await.clear();
    }
}

impl Default for ErrorTracker {
    fn default() -> Self {
        Self::new()
    }
}

/// Recovery middleware - integrates circuit breaker and fallback
pub struct RecoveryMiddleware {
    circuit_breaker: Option<Arc<CircuitBreaker>>,
    error_tracker: Arc<ErrorTracker>,
}

impl RecoveryMiddleware {
    pub fn new() -> Self {
        Self {
            circuit_breaker: None,
            error_tracker: Arc::new(ErrorTracker::new()),
        }
    }

    pub fn with_circuit_breaker(mut self, config: CircuitBreakerConfig) -> Self {
        self.circuit_breaker = Some(Arc::new(CircuitBreaker::new(config)));
        self
    }

    pub fn error_tracker(&self) -> Arc<ErrorTracker> {
        self.error_tracker.clone()
    }
}

impl Default for RecoveryMiddleware {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait::async_trait]
impl Middleware for RecoveryMiddleware {
    async fn before(&self, request: Value) -> Result<Value> {
        // Check circuit breaker before processing
        if let Some(cb) = &self.circuit_breaker {
            let state = cb.get_state().await;
            if state == CircuitState::Open {
                return Err(Error::Handler(
                    "Circuit breaker is OPEN - service unavailable".to_string(),
                ));
            }
        }
        Ok(request)
    }

    async fn on_error(&self, _request: Value, error: Error) -> Result<Value> {
        // Track the error
        self.error_tracker.track_error(&error).await;

        // Record failure in circuit breaker
        if let Some(cb) = &self.circuit_breaker {
            cb.on_failure().await;
        }

        Err(error)
    }

    async fn after(&self, _request: Value, response: Value) -> Result<Value> {
        // Record success in circuit breaker
        if let Some(cb) = &self.circuit_breaker {
            cb.on_success().await;
        }

        Ok(response)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_circuit_breaker_closed_to_open() {
        let config = CircuitBreakerConfig {
            failure_threshold: 3,
            timeout: Duration::from_secs(1),
            success_threshold: 2,
        };

        let cb = CircuitBreaker::new(config);

        // Initially closed
        assert_eq!(cb.get_state().await, CircuitState::Closed);

        // 3 failures should open circuit
        for _ in 0..3 {
            let _ = cb
                .call(|| async { Err::<(), _>(Error::Handler("test error".to_string())) })
                .await;
        }

        assert_eq!(cb.get_state().await, CircuitState::Open);
    }

    #[tokio::test]
    async fn test_circuit_breaker_half_open_recovery() {
        let config = CircuitBreakerConfig {
            failure_threshold: 2,
            timeout: Duration::from_millis(100),
            success_threshold: 2,
        };

        let cb = CircuitBreaker::new(config);

        // Open the circuit
        for _ in 0..2 {
            let _ = cb
                .call(|| async { Err::<(), _>(Error::Handler("test error".to_string())) })
                .await;
        }

        assert_eq!(cb.get_state().await, CircuitState::Open);

        // Wait for timeout
        tokio::time::sleep(Duration::from_millis(150)).await;

        // Next call should transition to half-open
        let _ = cb.call(|| async { Ok::<_, Error>(42) }).await;
        assert_eq!(cb.get_state().await, CircuitState::HalfOpen);

        // One more success should close circuit
        let _ = cb.call(|| async { Ok::<_, Error>(42) }).await;
        assert_eq!(cb.get_state().await, CircuitState::Closed);
    }

    #[tokio::test]
    async fn test_circuit_breaker_rejects_when_open() {
        let config = CircuitBreakerConfig {
            failure_threshold: 1,
            timeout: Duration::from_secs(60),
            success_threshold: 2,
        };

        let cb = CircuitBreaker::new(config);

        // Open the circuit
        let _ = cb
            .call(|| async { Err::<(), _>(Error::Handler("test error".to_string())) })
            .await;

        // Should reject immediately
        let result = cb.call(|| async { Ok::<_, Error>(42) }).await;
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Circuit breaker is OPEN"));
    }

    #[tokio::test]
    async fn test_error_tracker() {
        let tracker = ErrorTracker::new();

        // Track different errors
        tracker
            .track_error(&Error::Handler("timeout error".to_string()))
            .await;
        tracker
            .track_error(&Error::Handler("timeout error".to_string()))
            .await;
        tracker
            .track_error(&Error::Handler("connection error".to_string()))
            .await;
        tracker
            .track_error(&Error::Handler("other error".to_string()))
            .await;

        assert_eq!(tracker.total_errors(), 4);

        let by_type = tracker.errors_by_type().await;
        assert_eq!(by_type.get("timeout"), Some(&2));
        assert_eq!(by_type.get("connection"), Some(&1));
        assert_eq!(by_type.get("handler_error"), Some(&1));
    }

    #[tokio::test]
    async fn test_fallback_handler() {
        let fallback = FallbackHandler::new(|error: Error| async move {
            // Return default value on error
            let _ = error;
            Ok(serde_json::json!({"fallback": true}))
        });

        let result = fallback
            .handle_error(Error::Handler("test".to_string()))
            .await
            .unwrap();

        assert_eq!(result["fallback"], true);
    }

    #[tokio::test]
    async fn test_recovery_middleware_integration() {
        let config = CircuitBreakerConfig {
            failure_threshold: 2,
            timeout: Duration::from_secs(60),
            success_threshold: 2,
        };

        let middleware = RecoveryMiddleware::new().with_circuit_breaker(config);
        let tracker = middleware.error_tracker();

        // Simulate failures
        let _ = middleware
            .on_error(
                serde_json::json!({}),
                Error::Handler("test error".to_string()),
            )
            .await;

        let _ = middleware
            .on_error(
                serde_json::json!({}),
                Error::Handler("test error".to_string()),
            )
            .await;

        // Check error tracking
        assert_eq!(tracker.total_errors(), 2);

        // Circuit should be open, before hook should fail
        let result = middleware.before(serde_json::json!({})).await;
        assert!(result.is_err());
    }
}
