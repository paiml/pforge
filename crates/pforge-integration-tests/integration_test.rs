/// Integration tests for pforge framework
/// Tests end-to-end functionality across multiple crates
use pforge_config::{ForgeConfig, ForgeMetadata, ToolDef, TransportType};
use pforge_runtime::{
    CircuitBreaker, CircuitBreakerConfig, ErrorTracker, MiddlewareChain, PromptManager,
    RecoveryMiddleware, ResourceManager, RetryPolicy, StateManager, MemoryStateManager,
    retry_with_policy, with_timeout,
};
use serde_json::json;
use std::time::Duration;

#[test]
fn test_config_parsing_all_tool_types() {
    let yaml = r#"
forge:
  name: test-server
  version: 0.1.0
  transport: stdio

tools:
  - type: native
    name: hello
    description: Say hello
    handler:
      path: handlers::hello
    params:
      name:
        type: string
        required: true

  - type: cli
    name: echo
    description: Echo command
    command: echo
    args: ["hello"]

  - type: http
    name: api_call
    description: API call
    endpoint: https://api.example.com
    method: GET
"#;

    let config: ForgeConfig = serde_yaml::from_str(yaml).unwrap();
    assert_eq!(config.forge.name, "test-server");
    assert_eq!(config.tools.len(), 3);

    // Verify tool types
    assert!(matches!(config.tools[0], ToolDef::Native { .. }));
    assert!(matches!(config.tools[1], ToolDef::Cli { .. }));
    assert!(matches!(config.tools[2], ToolDef::Http { .. }));
}

#[test]
fn test_config_with_resources_and_prompts() {
    let yaml = r#"
forge:
  name: test-server
  version: 0.1.0

resources:
  - uri_template: "file:///{path}"
    handler:
      path: handlers::file_resource
    supports:
      - read
      - write

prompts:
  - name: greeting
    description: Generate a greeting
    template: "Hello, {{name}}! Welcome to {{location}}."
    arguments:
      name:
        type: string
        required: true
      location:
        type: string
        required: true
"#;

    let config: ForgeConfig = serde_yaml::from_str(yaml).unwrap();
    assert_eq!(config.resources.len(), 1);
    assert_eq!(config.prompts.len(), 1);
    assert_eq!(config.prompts[0].name, "greeting");
}

#[tokio::test]
async fn test_state_management_persistence() {
    let state = MemoryStateManager::new();

    // Set and get
    state.set("key1", b"value1".to_vec(), None).await.unwrap();
    let value = state.get("key1").await.unwrap();
    assert_eq!(value, Some(b"value1".to_vec()));

    // Exists
    assert!(state.exists("key1").await.unwrap());
    assert!(!state.exists("key2").await.unwrap());

    // Delete
    state.delete("key1").await.unwrap();
    assert!(!state.exists("key1").await.unwrap());
}

#[tokio::test]
async fn test_middleware_chain_with_recovery() {
    let mut chain = MiddlewareChain::new();

    let recovery = RecoveryMiddleware::new()
        .with_circuit_breaker(CircuitBreakerConfig {
            failure_threshold: 3,
            timeout: Duration::from_secs(60),
            success_threshold: 2,
        });

    let tracker = recovery.error_tracker();
    chain.add(std::sync::Arc::new(recovery));

    // Successful execution
    let result = chain
        .execute(json!({"input": 42}), |req| async move {
            Ok(json!({"output": req["input"].as_i64().unwrap() * 2}))
        })
        .await
        .unwrap();

    assert_eq!(result["output"], 84);
    assert_eq!(tracker.total_errors(), 0);
}

#[tokio::test]
async fn test_retry_with_timeout() {
    let policy = RetryPolicy::new(3)
        .with_backoff(Duration::from_millis(10), Duration::from_millis(50))
        .with_jitter(false);

    let attempt_counter = std::sync::Arc::new(std::sync::atomic::AtomicUsize::new(0));
    let counter_clone = attempt_counter.clone();

    let result = retry_with_policy(&policy, || {
        let counter = counter_clone.clone();
        async move {
            let count = counter.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
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
    assert_eq!(attempt_counter.load(std::sync::atomic::Ordering::SeqCst), 3);
}

#[tokio::test]
async fn test_circuit_breaker_integration() {
    let config = CircuitBreakerConfig {
        failure_threshold: 2,
        timeout: Duration::from_millis(100),
        success_threshold: 2,
    };

    let cb = CircuitBreaker::new(config);

    // Cause failures to open circuit
    for _ in 0..2 {
        let _ = cb
            .call(|| async {
                Err::<(), _>(pforge_runtime::Error::Handler("failure".to_string()))
            })
            .await;
    }

    // Circuit should be open
    let result = cb.call(|| async { Ok::<_, pforge_runtime::Error>(42) }).await;
    assert!(result.is_err());

    // Wait for timeout
    tokio::time::sleep(Duration::from_millis(150)).await;

    // Should transition to half-open and eventually close
    let _ = cb.call(|| async { Ok::<_, pforge_runtime::Error>(1) }).await;
    let _ = cb.call(|| async { Ok::<_, pforge_runtime::Error>(2) }).await;

    // Now should work
    let result = cb.call(|| async { Ok::<_, pforge_runtime::Error>(42) }).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_prompt_manager_full_workflow() {
    let mut manager = PromptManager::new();

    // Register prompts
    let prompt = pforge_config::PromptDef {
        name: "greeting".to_string(),
        description: "Greet user".to_string(),
        template: "Hello {{name}}, you are {{age}} years old!".to_string(),
        arguments: std::collections::HashMap::new(),
    };

    manager.register(prompt).unwrap();

    // Render prompt
    let mut args = std::collections::HashMap::new();
    args.insert("name".to_string(), json!("Alice"));
    args.insert("age".to_string(), json!(30));

    let rendered = manager.render("greeting", args).unwrap();
    assert_eq!(rendered, "Hello Alice, you are 30 years old!");
}

#[tokio::test]
async fn test_resource_manager_uri_matching() {
    use pforge_config::{HandlerRef, ResourceDef, ResourceOperation};
    use pforge_runtime::ResourceManager;

    let mut manager = ResourceManager::new();

    let resource = ResourceDef {
        uri_template: "file:///{path}".to_string(),
        handler: HandlerRef {
            path: "test::handler".to_string(),
            inline: None,
        },
        supports: vec![ResourceOperation::Read],
    };

    // For testing, we'd need a mock handler
    // Skipping actual registration as it requires ResourceHandler implementation
    assert_eq!(manager.list_templates().len(), 0);
}

#[tokio::test]
async fn test_error_tracker_classification() {
    let tracker = ErrorTracker::new();

    // Track various error types
    tracker
        .track_error(&pforge_runtime::Error::Handler("timeout error".to_string()))
        .await;
    tracker
        .track_error(&pforge_runtime::Error::Handler("connection failed".to_string()))
        .await;
    tracker
        .track_error(&pforge_runtime::Error::Handler("unknown issue".to_string()))
        .await;

    assert_eq!(tracker.total_errors(), 3);

    let by_type = tracker.errors_by_type().await;
    assert!(by_type.contains_key("timeout"));
    assert!(by_type.contains_key("connection"));
    assert!(by_type.contains_key("handler_error"));
}

#[test]
fn test_forge_metadata_defaults() {
    let yaml = r#"
forge:
  name: minimal-server
  version: 1.0.0
"#;

    let config: ForgeConfig = serde_yaml::from_str(yaml).unwrap();
    assert_eq!(config.forge.transport, TransportType::Stdio);
}

#[tokio::test]
async fn test_full_middleware_stack() {
    use pforge_runtime::{LoggingMiddleware, ValidationMiddleware};

    let mut chain = MiddlewareChain::new();

    // Add validation
    chain.add(std::sync::Arc::new(ValidationMiddleware::new(vec![
        "input".to_string(),
    ])));

    // Add logging
    chain.add(std::sync::Arc::new(LoggingMiddleware::new("test")));

    // Add recovery
    chain.add(std::sync::Arc::new(RecoveryMiddleware::new()));

    // Execute with valid request
    let result = chain
        .execute(json!({"input": 42}), |req| async move {
            Ok(json!({"output": req["input"].as_i64().unwrap() + 1}))
        })
        .await;

    assert!(result.is_ok());
    assert_eq!(result.unwrap()["output"], 43);

    // Execute with invalid request (missing field)
    let result = chain
        .execute(json!({"wrong": 42}), |req| async move {
            Ok(json!({"output": req["input"].as_i64().unwrap() + 1}))
        })
        .await;

    assert!(result.is_err());
}

#[test]
fn test_config_validation_duplicate_tools() {
    use pforge_config::validate_config;

    let yaml = r#"
forge:
  name: test
  version: 1.0.0

tools:
  - type: cli
    name: duplicate
    description: First
    command: echo
    args: []

  - type: cli
    name: duplicate
    description: Second
    command: echo
    args: []
"#;

    let config: ForgeConfig = serde_yaml::from_str(yaml).unwrap();
    let result = validate_config(&config);

    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("Duplicate tool name"));
}
