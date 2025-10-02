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
