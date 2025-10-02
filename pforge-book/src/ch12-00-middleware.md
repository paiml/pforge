# Chapter 12: Middleware

This chapter explores pforge's middleware chain architecture, built-in middleware, and custom middleware patterns for cross-cutting concerns.

## What is Middleware?

Middleware intercepts requests and responses, enabling cross-cutting functionality:
- Logging and monitoring
- Authentication and authorization
- Request validation
- Response transformation
- Error handling
- Performance tracking

## Middleware Chain Architecture

pforge executes middleware in a layered approach:

```
Request → Middleware 1 → Middleware 2 → ... → Handler → ... → Middleware 2 → Middleware 1 → Response
          (before)       (before)              (execute)       (after)        (after)
```

### Execution Order

```rust
// From crates/pforge-runtime/src/middleware.rs

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
```

## Built-in Middleware

### 1. Logging Middleware

Logs all requests and responses:

```yaml
middleware:
  - type: logging
    tag: "my-server"
    level: info
    include_request: true
    include_response: true
```

Implementation:

```rust
pub struct LoggingMiddleware {
    tag: String,
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
```

### 2. Validation Middleware

Validates request structure before processing:

```yaml
middleware:
  - type: validation
    required_fields:
      - user_id
      - session_token
    schema: request_schema.json
```

```rust
pub struct ValidationMiddleware {
    required_fields: Vec<String>,
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
```

### 3. Transform Middleware

Applies transformations to requests/responses:

```yaml
middleware:
  - type: transform
    request:
      uppercase_fields: [name, email]
      add_timestamp: true
    response:
      remove_fields: [internal_id]
      format: compact
```

```rust
pub struct TransformMiddleware<BeforeFn, AfterFn>
where
    BeforeFn: Fn(Value) -> Result<Value> + Send + Sync,
    AfterFn: Fn(Value) -> Result<Value> + Send + Sync,
{
    before_fn: BeforeFn,
    after_fn: AfterFn,
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
```

### 4. Recovery Middleware

Fault tolerance (covered in Chapter 11):

```yaml
middleware:
  - type: recovery
    circuit_breaker:
      enabled: true
      failure_threshold: 5
    error_tracking:
      enabled: true
```

## Custom Middleware

### Implementing the Middleware Trait

```rust
use pforge_runtime::{Middleware, Result, Error};
use serde_json::Value;

pub struct CustomMiddleware {
    config: CustomConfig,
}

#[async_trait::async_trait]
impl Middleware for CustomMiddleware {
    /// Process request before handler execution
    async fn before(&self, request: Value) -> Result<Value> {
        // Modify or validate request
        let mut req = request;

        // Add custom fields
        if let Value::Object(ref mut obj) = req {
            obj.insert("timestamp".to_string(), Value::Number(
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)?
                    .as_secs()
                    .into()
            ));
        }

        Ok(req)
    }

    /// Process response after handler execution
    async fn after(&self, request: Value, response: Value) -> Result<Value> {
        // Transform response
        let mut resp = response;

        // Add request ID from request
        if let (Value::Object(ref req_obj), Value::Object(ref mut resp_obj)) = (&request, &mut resp) {
            if let Some(req_id) = req_obj.get("request_id") {
                resp_obj.insert("request_id".to_string(), req_id.clone());
            }
        }

        Ok(resp)
    }

    /// Handle errors from handler or downstream middleware
    async fn on_error(&self, request: Value, error: Error) -> Result<Value> {
        // Log error details
        eprintln!("Error processing request: {:?}, error: {}", request, error);

        // Optionally recover or transform error
        Err(error)
    }
}
```

### Real-World Example: Authentication Middleware

```rust
use pforge_runtime::{Middleware, Result, Error};
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

pub struct AuthMiddleware {
    sessions: Arc<RwLock<HashMap<String, SessionInfo>>>,
}

#[derive(Clone)]
struct SessionInfo {
    user_id: String,
    expires_at: std::time::SystemTime,
}

impl AuthMiddleware {
    pub fn new() -> Self {
        Self {
            sessions: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}

#[async_trait::async_trait]
impl Middleware for AuthMiddleware {
    async fn before(&self, mut request: Value) -> Result<Value> {
        // Extract session token from request
        let token = request.get("session_token")
            .and_then(|v| v.as_str())
            .ok_or_else(|| Error::Handler("Missing session_token".into()))?;

        // Validate session
        let sessions = self.sessions.read().await;
        let session = sessions.get(token)
            .ok_or_else(|| Error::Handler("Invalid session".into()))?;

        // Check expiration
        if session.expires_at < std::time::SystemTime::now() {
            return Err(Error::Handler("Session expired".into()));
        }

        // Add user_id to request
        if let Value::Object(ref mut obj) = request {
            obj.insert("user_id".to_string(), Value::String(session.user_id.clone()));
        }

        Ok(request)
    }
}
```

## Middleware Composition

### Sequential Middleware

```yaml
middleware:
  - type: logging
    tag: "request-log"

  - type: auth
    session_store: redis

  - type: validation
    required_fields: [user_id]

  - type: transform
    request:
      sanitize: true

  - type: recovery
    circuit_breaker:
      enabled: true
```

### Conditional Middleware

Apply middleware only to specific tools:

```yaml
tools:
  - type: native
    name: public_tool
    handler:
      path: handlers::PublicHandler
    # No auth middleware

  - type: native
    name: protected_tool
    handler:
      path: handlers::ProtectedHandler
    middleware:
      - type: auth
        required_role: admin
      - type: audit
        log_level: debug
```

## Performance Middleware

Track execution time and metrics:

```rust
use std::time::Instant;

pub struct PerformanceMiddleware {
    metrics: Arc<DashMap<String, Vec<Duration>>>,
}

#[async_trait::async_trait]
impl Middleware for PerformanceMiddleware {
    async fn before(&self, mut request: Value) -> Result<Value> {
        // Store start time in request
        if let Value::Object(ref mut obj) = request {
            obj.insert("_start_time".to_string(),
                Value::String(Instant::now().elapsed().as_nanos().to_string()));
        }
        Ok(request)
    }

    async fn after(&self, request: Value, response: Value) -> Result<Value> {
        // Calculate elapsed time
        if let Value::Object(ref obj) = request {
            if let Some(Value::String(start)) = obj.get("_start_time") {
                if let Ok(start_nanos) = start.parse::<u128>() {
                    let elapsed = Duration::from_nanos(
                        Instant::now().elapsed().as_nanos().saturating_sub(start_nanos) as u64
                    );

                    // Store metric
                    let tool_name = obj.get("tool")
                        .and_then(|v| v.as_str())
                        .unwrap_or("unknown");

                    self.metrics.entry(tool_name.to_string())
                        .or_insert_with(Vec::new)
                        .push(elapsed);
                }
            }
        }

        Ok(response)
    }
}
```

## Error Recovery Middleware

```rust
pub struct ErrorRecoveryMiddleware {
    fallback_fn: Arc<dyn Fn(Error) -> Value + Send + Sync>,
}

#[async_trait::async_trait]
impl Middleware for ErrorRecoveryMiddleware {
    async fn on_error(&self, _request: Value, error: Error) -> Result<Value> {
        // Attempt recovery
        match error {
            Error::Timeout => {
                // Return cached or default data
                Ok((self.fallback_fn)(error))
            }
            Error::Handler(ref msg) if msg.contains("503") => {
                // Service unavailable - use fallback
                Ok((self.fallback_fn)(error))
            }
            _ => {
                // Cannot recover - propagate error
                Err(error)
            }
        }
    }
}
```

## Testing Middleware

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[tokio::test]
    async fn test_middleware_chain_execution_order() {
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

        let mut chain = MiddlewareChain::new();
        chain.add(Arc::new(TestMiddleware { tag: "first".to_string() }));
        chain.add(Arc::new(TestMiddleware { tag: "second".to_string() }));

        let request = json!({});
        let result = chain.execute(request, |req| async move {
            // Verify before hooks ran
            assert!(req["first_before"].as_bool().unwrap_or(false));
            assert!(req["second_before"].as_bool().unwrap_or(false));
            Ok(json!({}))
        }).await.unwrap();

        // Verify after hooks ran in reverse order
        assert!(result["second_after"].as_bool().unwrap_or(false));
        assert!(result["first_after"].as_bool().unwrap_or(false));
    }

    #[tokio::test]
    async fn test_validation_middleware() {
        let middleware = ValidationMiddleware::new(vec!["name".to_string(), "age".to_string()]);

        // Valid request
        let valid = json!({"name": "Alice", "age": 30});
        assert!(middleware.before(valid).await.is_ok());

        // Invalid request
        let invalid = json!({"name": "Alice"});
        assert!(middleware.before(invalid).await.is_err());
    }

    #[tokio::test]
    async fn test_error_recovery_middleware() {
        struct RecoveryMiddleware;

        #[async_trait::async_trait]
        impl Middleware for RecoveryMiddleware {
            async fn on_error(&self, _request: Value, error: Error) -> Result<Value> {
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
        let result = chain.execute(json!({}), |_| async {
            Err(Error::Handler("recoverable error".into()))
        }).await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap()["recovered"], true);
    }
}
```

## Best Practices

1. **Keep middleware focused**: Each middleware should have a single responsibility
2. **Order matters**: Place authentication before authorization, logging first
3. **Performance**: Minimize work in hot path (before/after)
4. **Error handling**: Decide whether to recover or propagate
5. **State sharing**: Use Arc for shared state
6. **Testing**: Test middleware in isolation and in chains
7. **Documentation**: Document middleware execution order

## Summary

pforge's middleware system provides:

- **Layered architecture**: Request → Middleware → Handler → Middleware → Response
- **Built-in middleware**: Logging, validation, transformation, recovery
- **Custom middleware**: Implement the Middleware trait
- **Flexible composition**: Sequential and conditional middleware
- **Error handling**: Recovery and propagation patterns
- **Performance tracking**: Execution time monitoring

Middleware enables clean separation of concerns and reusable cross-cutting functionality.

---

**Next:** [Resources & Prompts](ch13-00-resources-prompts.md)
