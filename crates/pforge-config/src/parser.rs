use crate::{ConfigError, ForgeConfig, Result};
use std::path::Path;

pub fn parse_config(path: &Path) -> Result<ForgeConfig> {
    let content =
        std::fs::read_to_string(path).map_err(|e| ConfigError::IoError(path.to_path_buf(), e))?;

    parse_config_from_str(&content)
}

pub fn parse_config_from_str(yaml: &str) -> Result<ForgeConfig> {
    serde_yaml::from_str(yaml).map_err(|e| ConfigError::ParseError(e.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::*;

    #[test]
    fn test_parse_config_from_str_minimal() {
        let yaml = r#"
forge:
  name: test-server
  version: 0.1.0
  transport: stdio

tools:
  - type: native
    name: test_tool
    description: "Test tool"
    handler:
      path: "test::handler"
    params: {}
"#;
        let result = parse_config_from_str(yaml);
        assert!(result.is_ok());
        let config = result.unwrap();
        assert_eq!(config.forge.name, "test-server");
        assert_eq!(config.forge.version, "0.1.0");
        assert_eq!(config.tools.len(), 1);
    }

    #[test]
    fn test_parse_config_invalid_yaml() {
        let yaml = "invalid: yaml: structure: [[[";
        let result = parse_config_from_str(yaml);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), ConfigError::ParseError(_)));
    }

    #[test]
    fn test_parse_config_from_file_not_found() {
        let result = parse_config(Path::new("/nonexistent/file.yaml"));
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), ConfigError::IoError(_, _)));
    }

    #[test]
    fn test_parse_config_with_all_tool_types() {
        let yaml = r#"
forge:
  name: multi-tool
  version: 1.0.0
  transport: stdio

tools:
  - type: native
    name: native_tool
    description: "Native handler"
    handler:
      path: "native::handler"
    params: {}

  - type: cli
    name: cli_tool
    description: "CLI handler"
    command: "echo"
    args: ["test"]

  - type: http
    name: http_tool
    description: "HTTP handler"
    endpoint: "https://api.example.com"
    method: "GET"

  - type: pipeline
    name: pipeline_tool
    description: "Pipeline handler"
    steps:
      - tool: native_tool
    params: {}
"#;
        let result = parse_config_from_str(yaml);
        assert!(result.is_ok());
        let config = result.unwrap();
        assert_eq!(config.tools.len(), 4);
    }
}
