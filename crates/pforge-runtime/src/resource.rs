use crate::{Error, Result};
#[cfg(test)]
use pforge_config::HandlerRef;
use pforge_config::{ResourceDef, ResourceOperation};
use regex::Regex;
use std::collections::HashMap;
use std::sync::Arc;

/// Resource handler trait for read/write/subscribe operations
#[async_trait::async_trait]
pub trait ResourceHandler: Send + Sync {
    /// Read resource content
    async fn read(&self, uri: &str, params: HashMap<String, String>) -> Result<Vec<u8>>;

    /// Write resource content (if supported)
    async fn write(
        &self,
        uri: &str,
        params: HashMap<String, String>,
        content: Vec<u8>,
    ) -> Result<()> {
        let _ = (uri, params, content);
        Err(Error::Handler("Write operation not supported".to_string()))
    }

    /// Subscribe to resource changes (if supported)
    async fn subscribe(&self, uri: &str, params: HashMap<String, String>) -> Result<()> {
        let _ = (uri, params);
        Err(Error::Handler(
            "Subscribe operation not supported".to_string(),
        ))
    }
}

/// Resource manager handles URI matching and dispatch
pub struct ResourceManager {
    resources: Vec<ResourceEntry>,
}

struct ResourceEntry {
    uri_template: String,
    pattern: Regex,
    param_names: Vec<String>,
    supports: Vec<ResourceOperation>,
    handler: Arc<dyn ResourceHandler>,
}

impl ResourceManager {
    pub fn new() -> Self {
        Self {
            resources: Vec::new(),
        }
    }

    /// Register a resource with URI template matching
    pub fn register(&mut self, def: ResourceDef, handler: Arc<dyn ResourceHandler>) -> Result<()> {
        let (pattern, param_names) = Self::compile_uri_template(&def.uri_template)?;

        self.resources.push(ResourceEntry {
            uri_template: def.uri_template,
            pattern,
            param_names,
            supports: def.supports,
            handler,
        });

        Ok(())
    }

    /// Match URI and extract parameters (internal use)
    fn match_uri(&self, uri: &str) -> Option<(&ResourceEntry, HashMap<String, String>)> {
        for entry in &self.resources {
            if let Some(captures) = entry.pattern.captures(uri) {
                let mut params = HashMap::new();

                for (i, name) in entry.param_names.iter().enumerate() {
                    if let Some(value) = captures.get(i + 1) {
                        params.insert(name.clone(), value.as_str().to_string());
                    }
                }

                return Some((entry, params));
            }
        }

        None
    }

    /// Read resource by URI
    pub async fn read(&self, uri: &str) -> Result<Vec<u8>> {
        let (entry, params) = self
            .match_uri(uri)
            .ok_or_else(|| Error::Handler(format!("No resource matches URI: {}", uri)))?;

        if !entry.supports.contains(&ResourceOperation::Read) {
            return Err(Error::Handler(format!(
                "Resource {} does not support read operation",
                entry.uri_template
            )));
        }

        entry.handler.read(uri, params).await
    }

    /// Write resource by URI
    pub async fn write(&self, uri: &str, content: Vec<u8>) -> Result<()> {
        let (entry, params) = self
            .match_uri(uri)
            .ok_or_else(|| Error::Handler(format!("No resource matches URI: {}", uri)))?;

        if !entry.supports.contains(&ResourceOperation::Write) {
            return Err(Error::Handler(format!(
                "Resource {} does not support write operation",
                entry.uri_template
            )));
        }

        entry.handler.write(uri, params, content).await
    }

    /// Subscribe to resource changes
    pub async fn subscribe(&self, uri: &str) -> Result<()> {
        let (entry, params) = self
            .match_uri(uri)
            .ok_or_else(|| Error::Handler(format!("No resource matches URI: {}", uri)))?;

        if !entry.supports.contains(&ResourceOperation::Subscribe) {
            return Err(Error::Handler(format!(
                "Resource {} does not support subscribe operation",
                entry.uri_template
            )));
        }

        entry.handler.subscribe(uri, params).await
    }

    /// Compile URI template to regex pattern
    /// Example: "file:///{path}" -> r"^file:///(.+)$" with param_names = ["path"]
    /// Uses non-greedy matching to handle multiple parameters correctly
    fn compile_uri_template(template: &str) -> Result<(Regex, Vec<String>)> {
        let mut pattern = String::from("^");
        let mut param_names = Vec::new();
        let mut chars = template.chars().peekable();

        while let Some(ch) = chars.next() {
            if ch == '{' {
                // Extract parameter name
                let mut param_name = String::new();
                while let Some(&next_ch) = chars.peek() {
                    if next_ch == '}' {
                        chars.next(); // consume '}'
                        break;
                    }
                    param_name.push(chars.next().unwrap());
                }

                if param_name.is_empty() {
                    return Err(Error::Handler(
                        "Empty parameter name in URI template".to_string(),
                    ));
                }

                param_names.push(param_name);

                // Check what comes after the parameter
                // If there's a '/' after, match non-greedy up to next '/'
                // Otherwise, match greedy to end
                if chars.peek() == Some(&'/') {
                    pattern.push_str("([^/]+)"); // Segment matching
                } else {
                    pattern.push_str("(.+)"); // Greedy path matching
                }
            } else {
                // Escape regex special characters
                if ".*+?^$[](){}|\\".contains(ch) {
                    pattern.push('\\');
                }
                pattern.push(ch);
            }
        }

        pattern.push('$');

        let regex = Regex::new(&pattern)
            .map_err(|e| Error::Handler(format!("Invalid URI template regex: {}", e)))?;

        Ok((regex, param_names))
    }

    /// List all registered resource templates
    pub fn list_templates(&self) -> Vec<&str> {
        self.resources
            .iter()
            .map(|e| e.uri_template.as_str())
            .collect()
    }
}

impl Default for ResourceManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct TestResourceHandler {
        read_response: Vec<u8>,
    }

    #[async_trait::async_trait]
    impl ResourceHandler for TestResourceHandler {
        async fn read(&self, _uri: &str, _params: HashMap<String, String>) -> Result<Vec<u8>> {
            Ok(self.read_response.clone())
        }

        async fn write(
            &self,
            _uri: &str,
            _params: HashMap<String, String>,
            _content: Vec<u8>,
        ) -> Result<()> {
            Ok(())
        }
    }

    #[test]
    fn test_uri_template_compilation() {
        let (pattern, params) = ResourceManager::compile_uri_template("file:///{path}").unwrap();
        assert_eq!(params, vec!["path"]);

        let captures = pattern.captures("file:///home/user/test.txt").unwrap();
        assert_eq!(captures.get(1).unwrap().as_str(), "home/user/test.txt");
    }

    #[test]
    fn test_uri_template_multiple_params() {
        let (pattern, params) =
            ResourceManager::compile_uri_template("api://{service}/{resource}").unwrap();
        assert_eq!(params, vec!["service", "resource"]);

        let captures = pattern.captures("api://users/profile").unwrap();
        assert_eq!(captures.get(1).unwrap().as_str(), "users");
        assert_eq!(captures.get(2).unwrap().as_str(), "profile");
    }

    #[tokio::test]
    async fn test_resource_registration_and_matching() {
        let mut manager = ResourceManager::new();

        let def = ResourceDef {
            uri_template: "file:///{path}".to_string(),
            handler: HandlerRef {
                path: "test::handler".to_string(),
                inline: None,
            },
            supports: vec![ResourceOperation::Read],
        };

        let handler = Arc::new(TestResourceHandler {
            read_response: b"test content".to_vec(),
        });

        manager.register(def, handler).unwrap();

        let (entry, params) = manager.match_uri("file:///test.txt").unwrap();
        assert_eq!(entry.uri_template, "file:///{path}");
        assert_eq!(params.get("path").unwrap(), "test.txt");
    }

    #[tokio::test]
    async fn test_resource_read() {
        let mut manager = ResourceManager::new();

        let def = ResourceDef {
            uri_template: "file:///{path}".to_string(),
            handler: HandlerRef {
                path: "test::handler".to_string(),
                inline: None,
            },
            supports: vec![ResourceOperation::Read],
        };

        let handler = Arc::new(TestResourceHandler {
            read_response: b"hello world".to_vec(),
        });

        manager.register(def, handler).unwrap();

        let content = manager.read("file:///test.txt").await.unwrap();
        assert_eq!(content, b"hello world");
    }

    #[tokio::test]
    async fn test_resource_unsupported_operation() {
        let mut manager = ResourceManager::new();

        let def = ResourceDef {
            uri_template: "file:///{path}".to_string(),
            handler: HandlerRef {
                path: "test::handler".to_string(),
                inline: None,
            },
            supports: vec![ResourceOperation::Read],
        };

        let handler = Arc::new(TestResourceHandler {
            read_response: b"test".to_vec(),
        });

        manager.register(def, handler).unwrap();

        let result = manager.write("file:///test.txt", b"data".to_vec()).await;
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("does not support write"));
    }
}
