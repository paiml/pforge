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
                Error::Handler(format!("Failed to execute command '{}': {}", self.command, e))
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
