use crate::{Error, Result};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::process::Stdio;
use tokio::process::Command;
use tokio::time::{timeout, Duration};

#[derive(Debug, Clone)]
pub struct CliHandler {
    pub command: String,
    pub args: Vec<String>,
    pub cwd: Option<String>,
    pub env: HashMap<String, String>,
    pub timeout_ms: Option<u64>,
    pub stream: bool,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct CliInput {
    #[serde(default)]
    pub args: Vec<String>,
    #[serde(default)]
    pub env: HashMap<String, String>,
}

#[derive(Debug, Serialize, JsonSchema)]
pub struct CliOutput {
    pub stdout: String,
    pub stderr: String,
    pub exit_code: i32,
}

impl CliHandler {
    pub fn new(
        command: String,
        args: Vec<String>,
        cwd: Option<String>,
        env: HashMap<String, String>,
        timeout_ms: Option<u64>,
        stream: bool,
    ) -> Self {
        Self {
            command,
            args,
            cwd,
            env,
            timeout_ms,
            stream,
        }
    }

    pub async fn execute(&self, input: CliInput) -> Result<CliOutput> {
        let mut cmd = Command::new(&self.command);

        // Add base args
        cmd.args(&self.args);

        // Add input args
        cmd.args(&input.args);

        // Set working directory
        if let Some(cwd) = &self.cwd {
            cmd.current_dir(cwd);
        }

        // Set environment variables (base + input)
        for (k, v) in &self.env {
            cmd.env(k, v);
        }
        for (k, v) in &input.env {
            cmd.env(k, v);
        }

        // Configure stdio
        cmd.stdout(Stdio::piped());
        cmd.stderr(Stdio::piped());

        // Execute with timeout
        let exec_future = async {
            let output = cmd.output().await.map_err(|e| {
                Error::Handler(format!(
                    "Failed to execute command '{}': {}",
                    self.command, e
                ))
            })?;

            Ok::<_, Error>(CliOutput {
                stdout: String::from_utf8_lossy(&output.stdout).to_string(),
                stderr: String::from_utf8_lossy(&output.stderr).to_string(),
                exit_code: output.status.code().unwrap_or(-1),
            })
        };

        if let Some(timeout_ms) = self.timeout_ms {
            timeout(Duration::from_millis(timeout_ms), exec_future)
                .await
                .map_err(|_| Error::Timeout)?
        } else {
            exec_future.await
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_cli_handler_new() {
        let handler = CliHandler::new(
            "echo".to_string(),
            vec!["hello".to_string()],
            None,
            HashMap::new(),
            None,
            false,
        );

        assert_eq!(handler.command, "echo");
        assert_eq!(handler.args.len(), 1);
        assert_eq!(handler.args[0], "hello");
        assert!(handler.cwd.is_none());
        assert!(handler.env.is_empty());
        assert!(handler.timeout_ms.is_none());
        assert!(!handler.stream);
    }

    #[tokio::test]
    async fn test_cli_handler_execute_simple() {
        let handler = CliHandler::new(
            "echo".to_string(),
            vec!["hello".to_string()],
            None,
            HashMap::new(),
            None,
            false,
        );

        let input = CliInput {
            args: vec![],
            env: HashMap::new(),
        };

        let result = handler.execute(input).await;
        assert!(result.is_ok());

        let output = result.unwrap();
        assert!(output.stdout.contains("hello"));
        assert_eq!(output.exit_code, 0);
    }

    #[tokio::test]
    async fn test_cli_handler_execute_with_input_args() {
        let handler = CliHandler::new(
            "echo".to_string(),
            vec![],
            None,
            HashMap::new(),
            None,
            false,
        );

        let input = CliInput {
            args: vec!["test".to_string(), "message".to_string()],
            env: HashMap::new(),
        };

        let result = handler.execute(input).await;
        assert!(result.is_ok());

        let output = result.unwrap();
        assert!(output.stdout.contains("test"));
        assert!(output.stdout.contains("message"));
    }

    #[tokio::test]
    async fn test_cli_handler_execute_with_timeout() {
        let handler = CliHandler::new(
            "sleep".to_string(),
            vec!["2".to_string()],
            None,
            HashMap::new(),
            Some(100), // 100ms timeout
            false,
        );

        let input = CliInput {
            args: vec![],
            env: HashMap::new(),
        };

        let result = handler.execute(input).await;
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), Error::Timeout));
    }

    #[tokio::test]
    async fn test_cli_handler_execute_invalid_command() {
        let handler = CliHandler::new(
            "nonexistent_command_that_should_fail".to_string(),
            vec![],
            None,
            HashMap::new(),
            None,
            false,
        );

        let input = CliInput {
            args: vec![],
            env: HashMap::new(),
        };

        let result = handler.execute(input).await;
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), Error::Handler(_)));
    }

    #[tokio::test]
    async fn test_cli_handler_with_env() {
        let mut env = HashMap::new();
        env.insert("TEST_VAR".to_string(), "test_value".to_string());

        let handler = CliHandler::new(
            "sh".to_string(),
            vec!["-c".to_string(), "echo $TEST_VAR".to_string()],
            None,
            env,
            None,
            false,
        );

        let input = CliInput {
            args: vec![],
            env: HashMap::new(),
        };

        let result = handler.execute(input).await;
        assert!(result.is_ok());

        let output = result.unwrap();
        assert!(output.stdout.contains("test_value"));
    }
}
