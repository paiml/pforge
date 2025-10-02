# API Authentication

HTTP handlers support multiple authentication strategies. This chapter covers implementing Bearer tokens, API keys, Basic Auth, and OAuth patterns.

## Bearer Token Authentication

### Static Token (Configuration)

```yaml
tools:
  - type: http
    name: auth_api_call
    endpoint: "https://api.example.com/data"
    method: GET
    headers:
      Authorization: "Bearer {{access_token}}"
    params:
      access_token:
        type: string
        required: true
        description: "API access token"
```

**Usage**:
```json
{
  "tool": "auth_api_call",
  "params": {
    "access_token": "eyJhbGc..."
  }
}
```

### Dynamic Token (Environment Variable)

```yaml
headers:
  Authorization: "Bearer ${API_TOKEN}"  # From environment
```

## API Key Authentication

### Header-Based API Key

```yaml
tools:
  - type: http
    name: api_key_call
    endpoint: "https://api.example.com/resource"
    method: GET
    headers:
      X-API-Key: "{{api_key}}"
    params:
      api_key: { type: string, required: true }
```

### Query Parameter API Key

```yaml
tools:
  - type: http
    name: query_key_call
    endpoint: "https://api.example.com/resource"
    method: GET
    query:
      api_key: "{{api_key}}"
    params:
      api_key: { type: string, required: true }
```

## Basic Authentication

### YAML Configuration

```yaml
tools:
  - type: http
    name: basic_auth_call
    endpoint: "https://api.example.com/secure"
    method: GET
    auth:
      type: basic
      username: "{{username}}"
      password: "{{password}}"
    params:
      username: { type: string, required: true }
      password: { type: string, required: true }
```

### Native Handler Implementation

```rust
use reqwest::Client;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, JsonSchema)]
struct BasicAuthInput {
    username: String,
    password: String,
    resource: String,
}

#[derive(Serialize, JsonSchema)]
struct ApiResponse {
    status: u16,
    body: serde_json::Value,
}

async fn handle(&self, input: BasicAuthInput) -> Result<ApiResponse> {
    let client = Client::new();

    let response = client
        .get(&format!("https://api.example.com/{}", input.resource))
        .basic_auth(&input.username, Some(&input.password))
        .send()
        .await?;

    Ok(ApiResponse {
        status: response.status().as_u16(),
        body: response.json().await?,
    })
}
```

## OAuth 2.0 Patterns

### Client Credentials Flow

```rust
use serde::{Deserialize, Serialize};
use reqwest::Client;

#[derive(Deserialize)]
struct TokenResponse {
    access_token: String,
    token_type: String,
    expires_in: u64,
}

#[derive(Deserialize, JsonSchema)]
struct OAuthInput {
    client_id: String,
    client_secret: String,
    resource: String,
}

async fn handle(&self, input: OAuthInput) -> Result<ApiResponse> {
    // Step 1: Get access token
    let token_response: TokenResponse = Client::new()
        .post("https://oauth.example.com/token")
        .form(&[
            ("grant_type", "client_credentials"),
            ("client_id", &input.client_id),
            ("client_secret", &input.client_secret),
        ])
        .send()
        .await?
        .json()
        .await?;

    // Step 2: Use access token
    let response = Client::new()
        .get(&format!("https://api.example.com/{}", input.resource))
        .bearer_auth(&token_response.access_token)
        .send()
        .await?;

    Ok(ApiResponse {
        status: response.status().as_u16(),
        body: response.json().await?,
    })
}
```

### Token Refresh Flow

```rust
use std::sync::Arc;
use tokio::sync::RwLock;
use std::time::{SystemTime, UNIX_EPOCH};

struct TokenCache {
    access_token: String,
    expires_at: u64,
}

pub struct OAuthHandler {
    client_id: String,
    client_secret: String,
    token_cache: Arc<RwLock<Option<TokenCache>>>,
    client: Client,
}

impl OAuthHandler {
    async fn get_access_token(&self) -> Result<String> {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)?
            .as_secs();

        // Check cache
        {
            let cache = self.token_cache.read().await;
            if let Some(token) = cache.as_ref() {
                if token.expires_at > now + 60 {  // 1 minute buffer
                    return Ok(token.access_token.clone());
                }
            }
        }

        // Refresh token
        let response: TokenResponse = self.client
            .post("https://oauth.example.com/token")
            .form(&[
                ("grant_type", "client_credentials"),
                ("client_id", &self.client_id),
                ("client_secret", &self.client_secret),
            ])
            .send()
            .await?
            .json()
            .await?;

        let expires_at = now + response.expires_in;

        // Update cache
        {
            let mut cache = self.token_cache.write().await;
            *cache = Some(TokenCache {
                access_token: response.access_token.clone(),
                expires_at,
            });
        }

        Ok(response.access_token)
    }

    async fn handle(&self, input: OAuthInput) -> Result<ApiResponse> {
        let access_token = self.get_access_token().await?;

        let response = self.client
            .get(&format!("https://api.example.com/{}", input.resource))
            .bearer_auth(&access_token)
            .send()
            .await?;

        Ok(ApiResponse {
            status: response.status().as_u16(),
            body: response.json().await?,
        })
    }
}
```

## JWT Authentication

### JWT Token Generation

```rust
use jsonwebtoken::{encode, Header, EncodingKey};
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
struct Claims {
    sub: String,
    exp: u64,
    iat: u64,
}

async fn handle(&self, input: JwtInput) -> Result<ApiResponse> {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)?
        .as_secs();

    let claims = Claims {
        sub: input.user_id,
        iat: now,
        exp: now + 3600,  // 1 hour
    };

    let token = encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(input.secret.as_bytes()),
    )?;

    let response = self.client
        .get(&input.url)
        .bearer_auth(&token)
        .send()
        .await?;

    Ok(ApiResponse {
        status: response.status().as_u16(),
        body: response.json().await?,
    })
}
```

## HMAC Signature Authentication

### AWS Signature V4 Example

```rust
use hmac::{Hmac, Mac};
use sha2::Sha256;
use hex::encode;

type HmacSha256 = Hmac<Sha256>;

fn sign_request(
    secret: &str,
    method: &str,
    path: &str,
    timestamp: u64,
) -> String {
    let string_to_sign = format!("{}\n{}\n{}", method, path, timestamp);

    let mut mac = HmacSha256::new_from_slice(secret.as_bytes())
        .expect("HMAC creation failed");
    mac.update(string_to_sign.as_bytes());

    encode(mac.finalize().into_bytes())
}

async fn handle(&self, input: SignedInput) -> Result<ApiResponse> {
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)?
        .as_secs();

    let signature = sign_request(
        &input.secret,
        "GET",
        &input.path,
        timestamp,
    );

    let response = self.client
        .get(&format!("https://api.example.com{}", input.path))
        .header("X-Timestamp", timestamp.to_string())
        .header("X-Signature", signature)
        .send()
        .await?;

    Ok(ApiResponse {
        status: response.status().as_u16(),
        body: response.json().await?,
    })
}
```

## Authentication Best Practices

### 1. Never Hardcode Secrets

```yaml
# BAD
headers:
  Authorization: "Bearer hardcoded_token_123"

# GOOD
headers:
  Authorization: "Bearer {{access_token}}"
params:
  access_token: { type: string, required: true }
```

### 2. Use Environment Variables

```rust
use std::env;

let api_key = env::var("API_KEY")
    .map_err(|_| Error::Config("API_KEY not set".into()))?;
```

### 3. Implement Token Rotation

```rust
// Rotate tokens before expiry
if token.expires_at - now < 300 {  // 5 minutes before expiry
    token = refresh_token().await?;
}
```

### 4. Secure Token Storage

```rust
use keyring::Entry;

// Store token securely
let entry = Entry::new("pforge", "api_token")?;
entry.set_password(&token)?;

// Retrieve token
let token = entry.get_password()?;
```

## Testing Authentication

### Mock OAuth Server

```rust
#[tokio::test]
async fn test_oauth_flow() {
    let mock_server = MockServer::start().await;

    // Mock token endpoint
    Mock::given(method("POST"))
        .and(path("/token"))
        .respond_with(ResponseTemplate::new(200)
            .set_body_json(json!({
                "access_token": "test_token",
                "token_type": "Bearer",
                "expires_in": 3600
            })))
        .mount(&mock_server)
        .await;

    // Mock API endpoint
    Mock::given(method("GET"))
        .and(path("/data"))
        .and(header("Authorization", "Bearer test_token"))
        .respond_with(ResponseTemplate::new(200)
            .set_body_json(json!({"data": "success"})))
        .mount(&mock_server)
        .await;

    // Test handler
    let handler = OAuthHandler::new(
        "client_id".to_string(),
        "client_secret".to_string(),
        mock_server.uri(),
    );

    let result = handler.handle(OAuthInput {
        resource: "data".to_string(),
    }).await.unwrap();

    assert_eq!(result.status, 200);
}
```

## Next Steps

Chapter 5.3 covers HTTP error handling, including retry strategies, circuit breakers, and graceful degradation patterns.

---

> "Authentication is trust. Handle it with care." - pforge security principle
