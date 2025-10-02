use crate::{Error, Result};
use serde_json::Value;
use std::sync::Arc;

/// Middleware trait for request/response processing
#[async_trait::async_trait]
pub trait Middleware: Send + Sync {
    /// Process request before handler execution
    /// Returns modified request or error
    async fn before(&self, request: Value) -> Result<Value> {
        Ok(request)
    }

    /// Process response after handler execution
    /// Returns modified response or error
    async fn after(&self, request: Value, response: Value) -> Result<Value> {
        let _ = request;
        Ok(response)
    }

    /// Handle errors from handler or downstream middleware
    async fn on_error(&self, request: Value, error: Error) -> Result<Value> {
        let _ = request;
        Err(error)
    }
}

/// Middleware chain manages ordered middleware execution
pub struct MiddlewareChain {
    middlewares: Vec<Arc<dyn Middleware>>,
}

impl MiddlewareChain {
    pub fn new() -> Self {
        Self {
            middlewares: Vec::new(),
        }
    }

    /// Add middleware to the chain
    pub fn add(&mut self, middleware: Arc<dyn Middleware>) {
        self.middlewares.push(middleware);
    }

    /// Execute middleware chain around a handler
    pub async fn execute<F, Fut>(&self, mut request: Value, handler: F) -> Result<Value>
    where
        F: FnOnce(Value) -> Fut,
        Fut: std::future::Future<Output = Result<Value>>,
    {
        // Execute "before" phase in order
        for middleware in &self.middlewares {
            request = middleware.before(request).await?;
        }

        // Execute handler
        let result = handler(request.clone()).await;

        // Execute "after" phase in reverse order or "on_error" if failed
        match result {
            Ok(mut response) => {
                for middleware in self.middlewares.iter().rev() {
                    response = middleware.after(request.clone(), response).await?;
                }
                Ok(response)
            }
            Err(error) => {
                let mut current_error = error;
                for middleware in self.middlewares.iter().rev() {
                    match middleware.on_error(request.clone(), current_error).await {
                        Ok(recovery_response) => return Ok(recovery_response),
                        Err(new_error) => current_error = new_error,
                    }
                }
                Err(current_error)
            }
        }
    }
}

impl Default for MiddlewareChain {
    fn default() -> Self {
        Self::new()
    }
}

/// Logging middleware - logs requests and responses
pub struct LoggingMiddleware {
    tag: String,
}

impl LoggingMiddleware {
    pub fn new(tag: impl Into<String>) -> Self {
        Self { tag: tag.into() }
    }
}

#[async_trait::async_trait]
impl Middleware for LoggingMiddleware {
    async fn before(&self, request: Value) -> Result<Value> {
        eprintln!(
            "[{}] Request: {}",
            self.tag,
            serde_json::to_string(&request).unwrap_or_default()
        );
        Ok(request)
    }

    async fn after(&self, _request: Value, response: Value) -> Result<Value> {
        eprintln!(
            "[{}] Response: {}",
            self.tag,
            serde_json::to_string(&response).unwrap_or_default()
        );
        Ok(response)
    }

    async fn on_error(&self, _request: Value, error: Error) -> Result<Value> {
        eprintln!("[{}] Error: {}", self.tag, error);
        Err(error)
    }
}

/// Validation middleware - validates request structure
pub struct ValidationMiddleware {
    required_fields: Vec<String>,
}

impl ValidationMiddleware {
    pub fn new(required_fields: Vec<String>) -> Self {
        Self { required_fields }
    }
}

#[async_trait::async_trait]
impl Middleware for ValidationMiddleware {
    async fn before(&self, request: Value) -> Result<Value> {
        if let Value::Object(obj) = &request {
            for field in &self.required_fields {
                if !obj.contains_key(field) {
                    return Err(Error::Handler(format!("Missing required field: {}", field)));
                }
            }
        }
        Ok(request)
    }
}

/// Transform middleware - applies transformations to request/response
pub struct TransformMiddleware<BeforeFn, AfterFn>
where
    BeforeFn: Fn(Value) -> Result<Value> + Send + Sync,
    AfterFn: Fn(Value) -> Result<Value> + Send + Sync,
{
    before_fn: BeforeFn,
    after_fn: AfterFn,
}

impl<BeforeFn, AfterFn> TransformMiddleware<BeforeFn, AfterFn>
where
    BeforeFn: Fn(Value) -> Result<Value> + Send + Sync,
    AfterFn: Fn(Value) -> Result<Value> + Send + Sync,
{
    pub fn new(before_fn: BeforeFn, after_fn: AfterFn) -> Self {
        Self {
            before_fn,
            after_fn,
        }
    }
}

#[async_trait::async_trait]
impl<BeforeFn, AfterFn> Middleware for TransformMiddleware<BeforeFn, AfterFn>
where
    BeforeFn: Fn(Value) -> Result<Value> + Send + Sync,
    AfterFn: Fn(Value) -> Result<Value> + Send + Sync,
{
    async fn before(&self, request: Value) -> Result<Value> {
        (self.before_fn)(request)
    }

    async fn after(&self, _request: Value, response: Value) -> Result<Value> {
        (self.after_fn)(response)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    struct TestMiddleware {
        tag: String,
    }

    #[async_trait::async_trait]
    impl Middleware for TestMiddleware {
        async fn before(&self, mut request: Value) -> Result<Value> {
            if let Value::Object(ref mut obj) = request {
                obj.insert(format!("{}_before", self.tag), Value::Bool(true));
            }
            Ok(request)
        }

        async fn after(&self, _request: Value, mut response: Value) -> Result<Value> {
            if let Value::Object(ref mut obj) = response {
                obj.insert(format!("{}_after", self.tag), Value::Bool(true));
            }
            Ok(response)
        }
    }

    #[tokio::test]
    async fn test_middleware_chain_execution_order() {
        let mut chain = MiddlewareChain::new();

        chain.add(Arc::new(TestMiddleware {
            tag: "first".to_string(),
        }));
        chain.add(Arc::new(TestMiddleware {
            tag: "second".to_string(),
        }));

        let request = json!({});
        let result = chain
            .execute(request, |req| async move {
                // Handler should see both "before" modifications
                assert!(req["first_before"].as_bool().unwrap_or(false));
                assert!(req["second_before"].as_bool().unwrap_or(false));
                Ok(json!({}))
            })
            .await
            .unwrap();

        // Response should have "after" modifications in reverse order
        assert!(result["second_after"].as_bool().unwrap_or(false));
        assert!(result["first_after"].as_bool().unwrap_or(false));
    }

    #[tokio::test]
    async fn test_validation_middleware() {
        let middleware = ValidationMiddleware::new(vec!["name".to_string(), "age".to_string()]);

        // Valid request
        let valid_request = json!({"name": "Alice", "age": 30});
        let result = middleware.before(valid_request).await;
        assert!(result.is_ok());

        // Invalid request - missing field
        let invalid_request = json!({"name": "Alice"});
        let result = middleware.before(invalid_request).await;
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Missing required field"));
    }

    #[tokio::test]
    async fn test_transform_middleware() {
        let middleware = TransformMiddleware::new(
            |mut req| {
                if let Value::Object(ref mut obj) = req {
                    if let Some(Value::String(s)) = obj.get("name") {
                        obj.insert("name".to_string(), Value::String(s.to_uppercase()));
                    }
                }
                Ok(req)
            },
            |mut resp| {
                if let Value::Object(ref mut obj) = resp {
                    obj.insert("transformed".to_string(), Value::Bool(true));
                }
                Ok(resp)
            },
        );

        let request = json!({"name": "alice"});
        let transformed = middleware.before(request).await.unwrap();
        assert_eq!(transformed["name"], "ALICE");

        let response = json!({});
        let transformed = middleware.after(json!({}), response).await.unwrap();
        assert_eq!(transformed["transformed"], true);
    }

    #[tokio::test]
    async fn test_error_handling_middleware() {
        struct RecoveryMiddleware;

        #[async_trait::async_trait]
        impl Middleware for RecoveryMiddleware {
            async fn on_error(&self, _request: Value, error: Error) -> Result<Value> {
                // Attempt to recover from specific errors
                if error.to_string().contains("recoverable") {
                    Ok(json!({"recovered": true}))
                } else {
                    Err(error)
                }
            }
        }

        let mut chain = MiddlewareChain::new();
        chain.add(Arc::new(RecoveryMiddleware));

        // Recoverable error
        let result = chain
            .execute(json!({}), |_| async {
                Err(Error::Handler("recoverable error".to_string()))
            })
            .await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap()["recovered"], true);

        // Non-recoverable error
        let result = chain
            .execute(json!({}), |_| async {
                Err(Error::Handler("fatal error".to_string()))
            })
            .await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_multiple_middleware_composition() {
        let mut chain = MiddlewareChain::new();

        chain.add(Arc::new(ValidationMiddleware::new(vec![
            "input".to_string()
        ])));
        chain.add(Arc::new(TransformMiddleware::new(
            |mut req| {
                if let Value::Object(ref mut obj) = req {
                    if let Some(Value::Number(n)) = obj.get("input") {
                        obj.insert(
                            "doubled".to_string(),
                            Value::Number(serde_json::Number::from(n.as_i64().unwrap() * 2)),
                        );
                    }
                }
                Ok(req)
            },
            Ok,
        )));

        let request = json!({"input": 5});
        let result = chain
            .execute(request, |req| async move {
                assert_eq!(req["doubled"], 10);
                Ok(json!({"result": req["doubled"].as_i64().unwrap() + 1}))
            })
            .await
            .unwrap();

        assert_eq!(result["result"], 11);
    }
}
