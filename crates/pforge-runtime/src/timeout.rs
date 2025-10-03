use crate::{Error, Middleware, Result};
use serde_json::Value;
use std::time::Duration;
use tokio::time::timeout;

/// Timeout middleware - enforces time limits on handler execution
/// Note: This is a placeholder - actual timeout enforcement happens in handler execution
pub struct TimeoutMiddleware {
    duration: Duration,
}

impl TimeoutMiddleware {
    pub fn new(duration: Duration) -> Self {
        Self { duration }
    }

    pub fn from_millis(millis: u64) -> Self {
        Self::new(Duration::from_millis(millis))
    }

    pub fn from_secs(secs: u64) -> Self {
        Self::new(Duration::from_secs(secs))
    }

    pub fn duration(&self) -> Duration {
        self.duration
    }
}

#[async_trait::async_trait]
impl Middleware for TimeoutMiddleware {
    async fn before(&self, request: Value) -> Result<Value> {
        Ok(request)
    }

    async fn after(&self, _request: Value, response: Value) -> Result<Value> {
        Ok(response)
    }
}

/// Retry policy configuration
#[derive(Debug, Clone)]
pub struct RetryPolicy {
    /// Maximum number of retry attempts
    pub max_attempts: u32,
    /// Initial backoff duration
    pub initial_backoff: Duration,
    /// Maximum backoff duration
    pub max_backoff: Duration,
    /// Backoff multiplier (exponential backoff)
    pub backoff_multiplier: f64,
    /// Whether to use jitter
    pub use_jitter: bool,
}

impl RetryPolicy {
    pub fn new(max_attempts: u32) -> Self {
        Self {
            max_attempts,
            initial_backoff: Duration::from_millis(100),
            max_backoff: Duration::from_secs(30),
            backoff_multiplier: 2.0,
            use_jitter: true,
        }
    }

    pub fn with_backoff(mut self, initial: Duration, max: Duration) -> Self {
        self.initial_backoff = initial;
        self.max_backoff = max;
        self
    }

    pub fn with_multiplier(mut self, multiplier: f64) -> Self {
        self.backoff_multiplier = multiplier;
        self
    }

    pub fn with_jitter(mut self, use_jitter: bool) -> Self {
        self.use_jitter = use_jitter;
        self
    }

    /// Calculate backoff duration for given attempt
    pub fn backoff_duration(&self, attempt: u32) -> Duration {
        let base_duration =
            self.initial_backoff.as_millis() as f64 * self.backoff_multiplier.powi(attempt as i32);

        let capped = base_duration.min(self.max_backoff.as_millis() as f64);

        if self.use_jitter {
            let jitter = rand::random::<f64>() * capped * 0.1; // 10% jitter
            Duration::from_millis((capped + jitter) as u64)
        } else {
            Duration::from_millis(capped as u64)
        }
    }

    /// Check if error is retryable
    pub fn is_retryable(&self, error: &Error) -> bool {
        // Retry on specific errors (can be customized)
        match error {
            Error::Handler(msg) => {
                // Retry on transient errors
                msg.contains("timeout")
                    || msg.contains("timed out")
                    || msg.contains("connection")
                    || msg.contains("temporary")
            }
            _ => false,
        }
    }
}

impl Default for RetryPolicy {
    fn default() -> Self {
        Self::new(3)
    }
}

/// Retry middleware - retries failed requests with backoff
/// Note: This is a marker - actual retry happens in handler execution layer
pub struct RetryMiddleware {
    policy: RetryPolicy,
}

impl RetryMiddleware {
    pub fn new(policy: RetryPolicy) -> Self {
        Self { policy }
    }

    pub fn with_max_attempts(max_attempts: u32) -> Self {
        Self::new(RetryPolicy::new(max_attempts))
    }

    pub fn policy(&self) -> &RetryPolicy {
        &self.policy
    }
}

#[async_trait::async_trait]
impl Middleware for RetryMiddleware {
    async fn on_error(&self, _request: Value, error: Error) -> Result<Value> {
        // Note: Actual retry logic requires handler re-execution
        // This middleware marks errors as retryable
        // Full retry implementation needs to be in the execution layer
        Err(error)
    }
}

/// Handle retry backoff delay
async fn apply_backoff_delay(policy: &RetryPolicy, attempt: u32, max_attempts: u32) {
    if attempt < max_attempts {
        let backoff = policy.backoff_duration(attempt - 1);
        tokio::time::sleep(backoff).await;
    }
}

/// Process retry attempt result
fn handle_retry_result<T>(
    error: Error,
    policy: &RetryPolicy,
    attempt: &mut u32,
    last_error: &mut Option<Error>,
) -> Option<Result<T>> {
    if !policy.is_retryable(&error) {
        return Some(Err(error));
    }

    *last_error = Some(error);
    *attempt += 1;
    None
}

/// Retry executor - wraps a future with retry logic
pub async fn retry_with_policy<F, Fut, T>(policy: &RetryPolicy, mut operation: F) -> Result<T>
where
    F: FnMut() -> Fut,
    Fut: std::future::Future<Output = Result<T>>,
{
    let mut attempt = 0;
    let mut last_error = None;

    while attempt < policy.max_attempts {
        match operation().await {
            Ok(result) => return Ok(result),
            Err(error) => {
                if let Some(result) =
                    handle_retry_result(error, policy, &mut attempt, &mut last_error)
                {
                    return result;
                }
                apply_backoff_delay(policy, attempt, policy.max_attempts).await;
            }
        }
    }

    Err(last_error.unwrap_or_else(|| Error::Handler("All retry attempts failed".to_string())))
}

/// Timeout executor - wraps a future with timeout
pub async fn with_timeout<F>(duration: Duration, future: F) -> Result<F::Output>
where
    F: std::future::Future,
{
    timeout(duration, future)
        .await
        .map_err(|_| Error::Handler(format!("Operation timed out after {:?}", duration)))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU32, Ordering};
    use std::sync::Arc;

    #[test]
    fn test_retry_policy_backoff() {
        let policy = RetryPolicy::new(3)
            .with_backoff(Duration::from_millis(100), Duration::from_secs(5))
            .with_multiplier(2.0)
            .with_jitter(false);

        let backoff1 = policy.backoff_duration(0);
        let backoff2 = policy.backoff_duration(1);
        let backoff3 = policy.backoff_duration(2);

        assert_eq!(backoff1.as_millis(), 100);
        assert_eq!(backoff2.as_millis(), 200);
        assert_eq!(backoff3.as_millis(), 400);
    }

    #[test]
    fn test_retry_policy_max_backoff() {
        let policy = RetryPolicy::new(10)
            .with_backoff(Duration::from_millis(100), Duration::from_secs(1))
            .with_multiplier(2.0)
            .with_jitter(false);

        let backoff = policy.backoff_duration(10); // Very high attempt
        assert!(backoff <= Duration::from_secs(1));
    }

    #[tokio::test]
    async fn test_retry_with_policy_success() {
        let counter = Arc::new(AtomicU32::new(0));
        let counter_clone = counter.clone();

        let policy = RetryPolicy::new(3)
            .with_backoff(Duration::from_millis(10), Duration::from_millis(50))
            .with_jitter(false);

        let result = retry_with_policy(&policy, || {
            let counter = counter_clone.clone();
            async move {
                let count = counter.fetch_add(1, Ordering::SeqCst);
                if count < 2 {
                    Err(Error::Handler("timeout error".to_string()))
                } else {
                    Ok(42)
                }
            }
        })
        .await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 42);
        assert_eq!(counter.load(Ordering::SeqCst), 3); // Took 3 attempts
    }

    #[tokio::test]
    async fn test_retry_with_policy_max_attempts() {
        let counter = Arc::new(AtomicU32::new(0));
        let counter_clone = counter.clone();

        let policy = RetryPolicy::new(3)
            .with_backoff(Duration::from_millis(10), Duration::from_millis(50))
            .with_jitter(false);

        let result = retry_with_policy(&policy, || {
            let counter = counter_clone.clone();
            async move {
                counter.fetch_add(1, Ordering::SeqCst);
                Err::<(), _>(Error::Handler("timeout error".to_string()))
            }
        })
        .await;

        assert!(result.is_err());
        assert_eq!(counter.load(Ordering::SeqCst), 3); // All 3 attempts used
    }

    #[tokio::test]
    async fn test_retry_non_retryable_error() {
        let counter = Arc::new(AtomicU32::new(0));
        let counter_clone = counter.clone();

        let policy = RetryPolicy::new(3);

        let result = retry_with_policy(&policy, || {
            let counter = counter_clone.clone();
            async move {
                counter.fetch_add(1, Ordering::SeqCst);
                Err::<(), _>(Error::Handler("fatal error".to_string()))
            }
        })
        .await;

        assert!(result.is_err());
        assert_eq!(counter.load(Ordering::SeqCst), 1); // No retries for non-retryable error
    }

    #[tokio::test]
    async fn test_with_timeout_success() {
        let result = with_timeout(Duration::from_secs(1), async {
            tokio::time::sleep(Duration::from_millis(10)).await;
            42
        })
        .await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 42);
    }

    #[tokio::test]
    async fn test_with_timeout_exceeded() {
        let result = with_timeout(Duration::from_millis(50), async {
            tokio::time::sleep(Duration::from_secs(10)).await;
            42
        })
        .await;

        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("timed out"));
    }

    #[tokio::test]
    async fn test_combined_timeout_and_retry() {
        let counter = Arc::new(AtomicU32::new(0));
        let counter_clone = counter.clone();

        let policy = RetryPolicy::new(3)
            .with_backoff(Duration::from_millis(10), Duration::from_millis(50))
            .with_jitter(false);

        let result = retry_with_policy(&policy, || {
            let counter = counter_clone.clone();
            async move {
                let count = counter.fetch_add(1, Ordering::SeqCst);
                if count < 2 {
                    with_timeout(Duration::from_millis(10), async {
                        tokio::time::sleep(Duration::from_secs(10)).await;
                        42
                    })
                    .await
                } else {
                    Ok(100)
                }
            }
        })
        .await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 100);
        assert_eq!(counter.load(Ordering::SeqCst), 3);
    }

    #[test]
    fn test_backoff_multiplier_exact() {
        // Kills mutants that change * to + or / in backoff calculation
        let policy = RetryPolicy::new(5)
            .with_backoff(Duration::from_millis(100), Duration::from_secs(10))
            .with_multiplier(3.0)
            .with_jitter(false);

        // Verify exponential backoff: 100 * 3^0 = 100, 100 * 3^1 = 300, 100 * 3^2 = 900
        assert_eq!(policy.backoff_duration(0).as_millis(), 100);
        assert_eq!(policy.backoff_duration(1).as_millis(), 300);
        assert_eq!(policy.backoff_duration(2).as_millis(), 900);
        assert_eq!(policy.backoff_duration(3).as_millis(), 2700);
    }

    #[test]
    fn test_is_retryable_logic() {
        // Kills mutants that change || to && in is_retryable
        let policy = RetryPolicy::new(3);

        // Should retry on timeout errors
        assert!(policy.is_retryable(&Error::Handler("timeout error".to_string())));
        assert!(policy.is_retryable(&Error::Handler("timed out".to_string())));
        assert!(policy.is_retryable(&Error::Handler("connection failed".to_string())));
        assert!(policy.is_retryable(&Error::Handler("temporary issue".to_string())));

        // Should NOT retry on other errors
        assert!(!policy.is_retryable(&Error::Handler("fatal error".to_string())));
        assert!(!policy.is_retryable(&Error::Timeout));
    }

    #[tokio::test]
    async fn test_retry_attempt_comparison() {
        // Kills mutants that change < to == or > in retry loop
        let counter = Arc::new(AtomicU32::new(0));
        let counter_clone = counter.clone();

        let policy = RetryPolicy::new(5) // Exactly 5 attempts
            .with_backoff(Duration::from_millis(1), Duration::from_millis(10))
            .with_jitter(false);

        let _result = retry_with_policy(&policy, || {
            let counter = counter_clone.clone();
            async move {
                counter.fetch_add(1, Ordering::SeqCst);
                Err::<(), _>(Error::Handler("timeout error".to_string()))
            }
        })
        .await;

        // Must execute exactly max_attempts times (not less, not more)
        assert_eq!(counter.load(Ordering::SeqCst), 5);
    }

    #[tokio::test]
    async fn test_retry_backoff_calculation() {
        // Kills mutants that change - to + or / in backoff calculation (attempt - 1)
        let policy = RetryPolicy::new(3)
            .with_backoff(Duration::from_millis(100), Duration::from_secs(1))
            .with_multiplier(2.0)
            .with_jitter(false);

        let counter = Arc::new(AtomicU32::new(0));
        let counter_clone = counter.clone();
        let start = std::time::Instant::now();

        let _result = retry_with_policy(&policy, || {
            let c = counter_clone.clone();
            async move {
                c.fetch_add(1, Ordering::SeqCst);
                Err::<(), _>(Error::Handler("timeout error".to_string()))
            }
        })
        .await;

        // With 3 attempts: attempt 0 (no sleep), attempt 1 (sleep 100ms), attempt 2 (sleep 200ms)
        // Total time should be ~300ms, not 0ms (if backoff broken) or 600ms+ (if calculation wrong)
        let total_time = start.elapsed();
        assert!(
            total_time >= Duration::from_millis(250),
            "Total time too short: {:?}",
            total_time
        );
        assert!(
            total_time < Duration::from_millis(500),
            "Total time too long: {:?}",
            total_time
        );
    }
}
