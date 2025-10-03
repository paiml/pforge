use crate::{Error, Result};
use reqwest::{Client, Method};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct HttpHandler {
    pub endpoint: String,
    pub method: HttpMethod,
    pub headers: HashMap<String, String>,
    pub auth: Option<AuthConfig>,
    client: Client,
}

#[derive(Debug, Clone)]
pub enum HttpMethod {
    Get,
    Post,
    Put,
    Delete,
    Patch,
}

#[derive(Debug, Clone)]
pub enum AuthConfig {
    Bearer { token: String },
    Basic { username: String, password: String },
    ApiKey { key: String, header: String },
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct HttpInput {
    #[serde(default)]
    pub body: Option<serde_json::Value>,
    #[serde(default)]
    pub query: HashMap<String, String>,
}

#[derive(Debug, Serialize, JsonSchema)]
pub struct HttpOutput {
    pub status: u16,
    pub body: serde_json::Value,
    pub headers: HashMap<String, String>,
}

impl HttpHandler {
    pub fn new(
        endpoint: String,
        method: HttpMethod,
        headers: HashMap<String, String>,
        auth: Option<AuthConfig>,
    ) -> Self {
        Self {
            endpoint,
            method,
            headers,
            auth,
            client: Client::new(),
        }
    }

    pub async fn execute(&self, input: HttpInput) -> Result<HttpOutput> {
        let method = match self.method {
            HttpMethod::Get => Method::GET,
            HttpMethod::Post => Method::POST,
            HttpMethod::Put => Method::PUT,
            HttpMethod::Delete => Method::DELETE,
            HttpMethod::Patch => Method::PATCH,
        };

        let mut request = self.client.request(method, &self.endpoint);

        // Add headers
        for (k, v) in &self.headers {
            request = request.header(k, v);
        }

        // Add authentication
        if let Some(auth) = &self.auth {
            request = match auth {
                AuthConfig::Bearer { token } => request.bearer_auth(token),
                AuthConfig::Basic { username, password } => {
                    request.basic_auth(username, Some(password))
                }
                AuthConfig::ApiKey { key, header } => request.header(header, key),
            };
        }

        // Add query parameters
        if !input.query.is_empty() {
            request = request.query(&input.query);
        }

        // Add body for non-GET requests
        if let Some(body) = input.body {
            request = request.json(&body);
        }

        // Execute request
        let response = request
            .send()
            .await
            .map_err(|e| Error::Http(format!("Request failed: {}", e)))?;

        let status = response.status().as_u16();

        // Extract headers
        let mut headers = HashMap::new();
        for (k, v) in response.headers() {
            if let Ok(v_str) = v.to_str() {
                headers.insert(k.to_string(), v_str.to_string());
            }
        }

        // Parse body as JSON (or empty object if fails)
        let body = response
            .json::<serde_json::Value>()
            .await
            .unwrap_or(serde_json::json!({}));

        Ok(HttpOutput {
            status,
            body,
            headers,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_http_handler_new() {
        let handler = HttpHandler::new(
            "https://api.example.com".to_string(),
            HttpMethod::Get,
            HashMap::new(),
            None,
        );

        assert_eq!(handler.endpoint, "https://api.example.com");
        assert!(handler.headers.is_empty());
        assert!(handler.auth.is_none());
    }

    #[test]
    fn test_http_handler_new_with_auth() {
        let mut headers = HashMap::new();
        headers.insert("Content-Type".to_string(), "application/json".to_string());

        let auth = Some(AuthConfig::Bearer {
            token: "test_token".to_string(),
        });

        let handler = HttpHandler::new(
            "https://api.example.com".to_string(),
            HttpMethod::Post,
            headers.clone(),
            auth,
        );

        assert_eq!(handler.endpoint, "https://api.example.com");
        assert_eq!(handler.headers.len(), 1);
        assert!(handler.auth.is_some());
    }

    #[test]
    fn test_http_input_with_body() {
        let json = r#"{"body": {"key": "value"}, "query": {}}"#;
        let input: HttpInput = serde_json::from_str(json).unwrap();

        assert!(input.body.is_some());
        assert_eq!(input.body.unwrap()["key"], "value");
    }

    #[test]
    fn test_http_input_with_query() {
        let json = r#"{"body": null, "query": {"param": "value"}}"#;
        let input: HttpInput = serde_json::from_str(json).unwrap();

        assert!(input.body.is_none());
        assert_eq!(input.query.get("param"), Some(&"value".to_string()));
    }

    #[test]
    fn test_http_output_serialization() {
        let mut headers = HashMap::new();
        headers.insert("content-type".to_string(), "application/json".to_string());

        let output = HttpOutput {
            status: 200,
            body: serde_json::json!({"result": "success"}),
            headers,
        };

        let json = serde_json::to_string(&output).unwrap();
        assert!(json.contains("\"status\":200"));
        assert!(json.contains("\"result\":\"success\""));
    }

    #[tokio::test]
    async fn test_execute_get_request() {
        let mut server = mockito::Server::new_async().await;
        let mock = server
            .mock("GET", "/test")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(r#"{"message": "success"}"#)
            .create_async()
            .await;

        let handler = HttpHandler::new(
            format!("{}/test", server.url()),
            HttpMethod::Get,
            HashMap::new(),
            None,
        );

        let input = HttpInput {
            body: None,
            query: HashMap::new(),
        };

        let output = handler.execute(input).await.unwrap();

        assert_eq!(output.status, 200);
        assert_eq!(output.body["message"], "success");
        mock.assert_async().await;
    }

    #[tokio::test]
    async fn test_execute_post_request_with_body() {
        let mut server = mockito::Server::new_async().await;
        let mock = server
            .mock("POST", "/api/data")
            .match_header("content-type", "application/json")
            .match_body(mockito::Matcher::JsonString(
                r#"{"key":"value"}"#.to_string(),
            ))
            .with_status(201)
            .with_body(r#"{"id": "123"}"#)
            .create_async()
            .await;

        let handler = HttpHandler::new(
            format!("{}/api/data", server.url()),
            HttpMethod::Post,
            HashMap::new(),
            None,
        );

        let input = HttpInput {
            body: Some(serde_json::json!({"key": "value"})),
            query: HashMap::new(),
        };

        let output = handler.execute(input).await.unwrap();

        assert_eq!(output.status, 201);
        assert_eq!(output.body["id"], "123");
        mock.assert_async().await;
    }

    #[tokio::test]
    async fn test_execute_with_query_params() {
        let mut server = mockito::Server::new_async().await;
        let mock = server
            .mock("GET", "/search")
            .match_query(mockito::Matcher::AllOf(vec![
                mockito::Matcher::UrlEncoded("q".to_string(), "rust".to_string()),
                mockito::Matcher::UrlEncoded("limit".to_string(), "10".to_string()),
            ]))
            .with_status(200)
            .with_body(r#"{"results": []}"#)
            .create_async()
            .await;

        let handler = HttpHandler::new(
            format!("{}/search", server.url()),
            HttpMethod::Get,
            HashMap::new(),
            None,
        );

        let mut query = HashMap::new();
        query.insert("q".to_string(), "rust".to_string());
        query.insert("limit".to_string(), "10".to_string());

        let input = HttpInput { body: None, query };

        let output = handler.execute(input).await.unwrap();

        assert_eq!(output.status, 200);
        mock.assert_async().await;
    }

    #[tokio::test]
    async fn test_execute_with_bearer_auth() {
        let mut server = mockito::Server::new_async().await;
        let mock = server
            .mock("GET", "/protected")
            .match_header("authorization", "Bearer secret_token")
            .with_status(200)
            .with_body(r#"{"authorized": true}"#)
            .create_async()
            .await;

        let handler = HttpHandler::new(
            format!("{}/protected", server.url()),
            HttpMethod::Get,
            HashMap::new(),
            Some(AuthConfig::Bearer {
                token: "secret_token".to_string(),
            }),
        );

        let input = HttpInput {
            body: None,
            query: HashMap::new(),
        };

        let output = handler.execute(input).await.unwrap();

        assert_eq!(output.status, 200);
        assert_eq!(output.body["authorized"], true);
        mock.assert_async().await;
    }

    #[tokio::test]
    async fn test_execute_with_basic_auth() {
        let mut server = mockito::Server::new_async().await;
        let mock = server
            .mock("GET", "/admin")
            .match_header("authorization", "Basic dXNlcjpwYXNz")
            .with_status(200)
            .with_body(r#"{"admin": true}"#)
            .create_async()
            .await;

        let handler = HttpHandler::new(
            format!("{}/admin", server.url()),
            HttpMethod::Get,
            HashMap::new(),
            Some(AuthConfig::Basic {
                username: "user".to_string(),
                password: "pass".to_string(),
            }),
        );

        let input = HttpInput {
            body: None,
            query: HashMap::new(),
        };

        let output = handler.execute(input).await.unwrap();

        assert_eq!(output.status, 200);
        assert_eq!(output.body["admin"], true);
        mock.assert_async().await;
    }

    #[tokio::test]
    async fn test_execute_with_api_key() {
        let mut server = mockito::Server::new_async().await;
        let mock = server
            .mock("GET", "/api")
            .match_header("x-api-key", "my_api_key")
            .with_status(200)
            .with_body(r#"{"valid": true}"#)
            .create_async()
            .await;

        let handler = HttpHandler::new(
            format!("{}/api", server.url()),
            HttpMethod::Get,
            HashMap::new(),
            Some(AuthConfig::ApiKey {
                key: "my_api_key".to_string(),
                header: "x-api-key".to_string(),
            }),
        );

        let input = HttpInput {
            body: None,
            query: HashMap::new(),
        };

        let output = handler.execute(input).await.unwrap();

        assert_eq!(output.status, 200);
        assert_eq!(output.body["valid"], true);
        mock.assert_async().await;
    }

    #[tokio::test]
    async fn test_execute_with_custom_headers() {
        let mut server = mockito::Server::new_async().await;
        let mock = server
            .mock("GET", "/headers")
            .match_header("x-custom", "custom_value")
            .match_header("x-request-id", "123")
            .with_status(200)
            .with_body(r#"{"ok": true}"#)
            .create_async()
            .await;

        let mut headers = HashMap::new();
        headers.insert("x-custom".to_string(), "custom_value".to_string());
        headers.insert("x-request-id".to_string(), "123".to_string());

        let handler = HttpHandler::new(
            format!("{}/headers", server.url()),
            HttpMethod::Get,
            headers,
            None,
        );

        let input = HttpInput {
            body: None,
            query: HashMap::new(),
        };

        let output = handler.execute(input).await.unwrap();

        assert_eq!(output.status, 200);
        mock.assert_async().await;
    }

    #[tokio::test]
    async fn test_execute_put_request() {
        let mut server = mockito::Server::new_async().await;
        let mock = server
            .mock("PUT", "/update")
            .with_status(200)
            .with_body(r#"{"updated": true}"#)
            .create_async()
            .await;

        let handler = HttpHandler::new(
            format!("{}/update", server.url()),
            HttpMethod::Put,
            HashMap::new(),
            None,
        );

        let input = HttpInput {
            body: Some(serde_json::json!({"data": "new_value"})),
            query: HashMap::new(),
        };

        let output = handler.execute(input).await.unwrap();

        assert_eq!(output.status, 200);
        assert_eq!(output.body["updated"], true);
        mock.assert_async().await;
    }

    #[tokio::test]
    async fn test_execute_delete_request() {
        let mut server = mockito::Server::new_async().await;
        let mock = server
            .mock("DELETE", "/resource/123")
            .with_status(204)
            .with_body("")
            .create_async()
            .await;

        let handler = HttpHandler::new(
            format!("{}/resource/123", server.url()),
            HttpMethod::Delete,
            HashMap::new(),
            None,
        );

        let input = HttpInput {
            body: None,
            query: HashMap::new(),
        };

        let output = handler.execute(input).await.unwrap();

        assert_eq!(output.status, 204);
        mock.assert_async().await;
    }

    #[tokio::test]
    async fn test_execute_patch_request() {
        let mut server = mockito::Server::new_async().await;
        let mock = server
            .mock("PATCH", "/partial")
            .with_status(200)
            .with_body(r#"{"patched": true}"#)
            .create_async()
            .await;

        let handler = HttpHandler::new(
            format!("{}/partial", server.url()),
            HttpMethod::Patch,
            HashMap::new(),
            None,
        );

        let input = HttpInput {
            body: Some(serde_json::json!({"field": "value"})),
            query: HashMap::new(),
        };

        let output = handler.execute(input).await.unwrap();

        assert_eq!(output.status, 200);
        assert_eq!(output.body["patched"], true);
        mock.assert_async().await;
    }

    #[tokio::test]
    async fn test_execute_error_handling() {
        let handler = HttpHandler::new(
            "http://localhost:1/nonexistent".to_string(),
            HttpMethod::Get,
            HashMap::new(),
            None,
        );

        let input = HttpInput {
            body: None,
            query: HashMap::new(),
        };

        let result = handler.execute(input).await;
        assert!(result.is_err());
    }
}
