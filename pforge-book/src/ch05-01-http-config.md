# HTTP Configuration

HTTP handlers require careful configuration for reliability, security, and performance. This chapter covers advanced HTTP configuration patterns.

## Complete Configuration Example

```yaml
tools:
  - type: http
    name: api_call
    description: "Configured API call with all options"
    endpoint: "https://api.example.com/{{resource}}"
    method: POST
    headers:
      User-Agent: "pforge/1.0"
      Authorization: "Bearer {{token}}"
      Content-Type: "application/json"
      X-Request-ID: "{{request_id}}"
    query:
      version: "v2"
      format: "json"
    body:
      data: "{{payload}}"
    timeout_ms: 30000
    retry:
      max_attempts: 3
      backoff_ms: 1000
    params:
      resource: { type: string, required: true }
      token: { type: string, required: true }
      request_id: { type: string, required: false }
      payload: { type: object, required: true }
```

## Header Management

### Static Headers

```yaml
headers:
  User-Agent: "pforge-client/1.0"
  Accept: "application/json"
  Accept-Language: "en-US"
```

### Dynamic Headers (Templated)

```yaml
headers:
  Authorization: "Bearer {{access_token}}"
  X-Tenant-ID: "{{tenant_id}}"
  X-Correlation-ID: "{{correlation_id}}"
```

### Conditional Headers

For conditional headers, use a Native handler:

```rust
async fn handle(&self, input: Input) -> Result<Output> {
    let mut headers = HashMap::new();
    headers.insert("User-Agent", "pforge");

    if let Some(token) = input.auth_token {
        headers.insert("Authorization", format!("Bearer {}", token));
    }

    if input.use_compression {
        headers.insert("Accept-Encoding", "gzip, deflate");
    }

    let client = reqwest::Client::new();
    let response = client
        .get(&input.url)
        .headers(headers)
        .send()
        .await?;

    // ...
}
```

## Query Parameter Patterns

### Simple Query Params

```yaml
query:
  page: "{{page}}"
  limit: "{{limit}}"
  sort: "name"  # Static value
```

### Array Query Params

```yaml
# Input: { "tags": ["rust", "mcp", "api"] }
# URL: ?tags=rust&tags=mcp&tags=api

query:
  tags: "{{tags}}"  # Automatically handles arrays
```

### Complex Filtering

```yaml
query:
  filter: "created_at>{{start_date}},status={{status}}"
  fields: "id,name,created_at"
```

## Request Body Configuration

### JSON Body

```yaml
tools:
  - type: http
    name: create_resource
    method: POST
    body:
      name: "{{name}}"
      description: "{{description}}"
      metadata:
        source: "pforge"
        timestamp: "{{timestamp}}"
```

### Nested Objects

```yaml
body:
  user:
    name: "{{user_name}}"
    email: "{{user_email}}"
    preferences:
      theme: "{{theme}}"
      notifications: "{{notifications}}"
```

### Array Payloads

```yaml
body:
  items: "{{items}}"  # Array of objects

# Input:
# {
#   "items": [
#     { "id": 1, "name": "foo" },
#     { "id": 2, "name": "bar" }
#   ]
# }
```

## Timeout Configuration

### Global Timeout

```yaml
timeout_ms: 30000  # 30 seconds for entire request
```

### Per-Endpoint Timeouts

```yaml
tools:
  - type: http
    name: quick_lookup
    endpoint: "https://api.example.com/lookup"
    timeout_ms: 1000  # 1 second

  - type: http
    name: heavy_computation
    endpoint: "https://api.example.com/compute"
    timeout_ms: 120000  # 2 minutes
```

### Native Handler Timeout Control

```rust
use tokio::time::{timeout, Duration};

let response = timeout(
    Duration::from_millis(input.timeout_ms),
    client.get(&url).send()
).await
.map_err(|_| Error::Timeout)?;
```

## Retry Configuration

### Basic Retry

```yaml
retry:
  max_attempts: 3
  backoff_ms: 1000  # Wait 1s between retries
```

### Exponential Backoff (Native Handler)

```rust
use backoff::{ExponentialBackoff, Error as BackoffError};

let backoff = ExponentialBackoff {
    initial_interval: Duration::from_millis(100),
    max_interval: Duration::from_secs(10),
    max_elapsed_time: Some(Duration::from_secs(60)),
    ..Default::default()
};

let result = backoff::retry(backoff, || async {
    match client.get(&url).send().await {
        Ok(response) if response.status().is_success() => Ok(response),
        Ok(response) => Err(BackoffError::transient(Error::Http(...))),
        Err(e) => Err(BackoffError::permanent(Error::from(e))),
    }
}).await?;
```

## Response Handling

### Status Code Mapping

HTTP handlers return all responses (2xx, 4xx, 5xx):

```yaml
# Handler returns:
{
  "status": 404,
  "body": { "error": "Not found" },
  "headers": {...}
}
```

**Client decides**:
```javascript
const result = await client.callTool("get_user", { id: "123" });

if (result.status === 404) {
  console.log("User not found");
} else if (result.status >= 400) {
  throw new Error(`API error: ${result.status}`);
}
```

### Header Extraction

```javascript
const result = await client.callTool("api_call", params);

// Rate limiting
const rateLimit = parseInt(result.headers["x-ratelimit-remaining"]);
if (rateLimit < 10) {
  console.warn("Approaching rate limit");
}

// Pagination
const nextPage = result.headers["link"]?.match(/page=(\d+)/)?.[1];
```

## SSL/TLS Configuration

### Accept Self-Signed Certificates (Development)

Use Native handler with custom client:

```rust
let client = reqwest::Client::builder()
    .danger_accept_invalid_certs(true)  // DEVELOPMENT ONLY
    .build()?;
```

### Custom CA Certificates

```rust
use reqwest::Certificate;

let cert = std::fs::read("ca-cert.pem")?;
let cert = Certificate::from_pem(&cert)?;

let client = reqwest::Client::builder()
    .add_root_certificate(cert)
    .build()?;
```

## Connection Pooling

HTTP handlers automatically use connection pooling via reqwest.

### Pool Configuration (Native Handler)

```rust
let client = reqwest::Client::builder()
    .pool_max_idle_per_host(10)
    .pool_idle_timeout(Duration::from_secs(30))
    .build()?;
```

## Common Configuration Patterns

### Pattern 1: Paginated API

```yaml
tools:
  - type: http
    name: list_items
    endpoint: "https://api.example.com/items"
    method: GET
    query:
      page: "{{page}}"
      per_page: "{{per_page}}"
    params:
      page: { type: integer, required: false, default: 1 }
      per_page: { type: integer, required: false, default: 100 }
```

### Pattern 2: Webhook Receiver

```yaml
tools:
  - type: http
    name: trigger_webhook
    endpoint: "https://webhook.example.com/events"
    method: POST
    headers:
      X-Webhook-Secret: "{{secret}}"
    body:
      event: "{{event_type}}"
      payload: "{{data}}"
```

### Pattern 3: File Upload (Use Native Handler)

```rust
use reqwest::multipart;

async fn handle(&self, input: UploadInput) -> Result<UploadOutput> {
    let file_content = std::fs::read(&input.file_path)?;

    let form = multipart::Form::new()
        .text("description", input.description)
        .part("file", multipart::Part::bytes(file_content)
            .file_name(input.file_name));

    let response = self.client
        .post(&input.upload_url)
        .multipart(form)
        .send()
        .await?;

    // ...
}
```

## Testing HTTP Configuration

### Mock Server

```rust
use wiremock::{Mock, MockServer, ResponseTemplate};

#[tokio::test]
async fn test_http_handler() {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/users/123"))
        .respond_with(ResponseTemplate::new(200)
            .set_body_json(json!({
                "id": "123",
                "name": "Alice"
            })))
        .mount(&mock_server)
        .await;

    let handler = HttpHandler::new(
        format!("{}/users/{{id}}", mock_server.uri()),
        HttpMethod::Get,
        HashMap::new(),
        None,
    );

    let result = handler.execute(HttpInput {
        body: None,
        query: [("id", "123")].into(),
    }).await.unwrap();

    assert_eq!(result.status, 200);
    assert_eq!(result.body["name"], "Alice");
}
```

## Next Steps

Chapter 5.2 covers authentication patterns including Bearer tokens, API keys, Basic Auth, and OAuth integration.

---

> "Configuration is declarative. Complexity is in the runtime." - pforge HTTP design
