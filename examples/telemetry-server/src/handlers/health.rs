use pforge_runtime::{Handler, HealthCheck, HealthStatus, Result};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Debug, Deserialize, JsonSchema)]
pub struct GetHealthInput {}

#[derive(Debug, Serialize, JsonSchema)]
pub struct GetHealthOutput {
    pub status: String,
    pub details: serde_json::Value,
}

pub struct GetHealthHandler {
    pub health: Arc<HealthCheck>,
}

#[async_trait::async_trait]
impl Handler for GetHealthHandler {
    type Input = GetHealthInput;
    type Output = GetHealthOutput;
    type Error = pforge_runtime::Error;

    async fn handle(&self, _input: Self::Input) -> Result<Self::Output> {
        let status = self.health.get_status();
        let details = self.health.export_json();

        Ok(GetHealthOutput {
            status: format!("{:?}", status),
            details,
        })
    }
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct SetComponentHealthInput {
    pub component: String,
    pub status: String,
    #[serde(default)]
    pub message: Option<String>,
}

#[derive(Debug, Serialize, JsonSchema)]
pub struct SetComponentHealthOutput {
    pub component: String,
    pub status: String,
    pub message: String,
}

pub struct SetComponentHealthHandler {
    pub health: Arc<HealthCheck>,
}

#[async_trait::async_trait]
impl Handler for SetComponentHealthHandler {
    type Input = SetComponentHealthInput;
    type Output = SetComponentHealthOutput;
    type Error = pforge_runtime::Error;

    async fn handle(&self, input: Self::Input) -> Result<Self::Output> {
        let status = match input.status.to_lowercase().as_str() {
            "healthy" => HealthStatus::Healthy,
            "degraded" => HealthStatus::Degraded,
            "unhealthy" => HealthStatus::Unhealthy,
            _ => {
                return Err(pforge_runtime::Error::Handler(format!(
                    "Invalid status: {}. Use Healthy, Degraded, or Unhealthy",
                    input.status
                )))
            }
        };

        self.health.register_component_with_message(
            &input.component,
            status,
            input.message.clone(),
        );

        Ok(SetComponentHealthOutput {
            component: input.component,
            status: format!("{:?}", status),
            message: input
                .message
                .unwrap_or_else(|| "Health status updated".to_string()),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_get_health_handler() {
        let health = Arc::new(HealthCheck::new());
        health.register_component("test", HealthStatus::Healthy);

        let handler = GetHealthHandler {
            health: health.clone(),
        };

        let output = handler.handle(GetHealthInput {}).await.unwrap();
        assert_eq!(output.status, "Healthy");
    }

    #[tokio::test]
    async fn test_set_component_health_handler() {
        let health = Arc::new(HealthCheck::new());
        let handler = SetComponentHealthHandler {
            health: health.clone(),
        };

        let input = SetComponentHealthInput {
            component: "database".to_string(),
            status: "Degraded".to_string(),
            message: Some("High latency detected".to_string()),
        };

        let output = handler.handle(input).await.unwrap();
        assert_eq!(output.component, "database");
        assert_eq!(output.status, "Degraded");

        // Verify it was registered
        let component = health.get_component("database").unwrap();
        assert_eq!(component.status, HealthStatus::Degraded);
    }

    #[tokio::test]
    async fn test_invalid_status() {
        let health = Arc::new(HealthCheck::new());
        let handler = SetComponentHealthHandler {
            health: health.clone(),
        };

        let input = SetComponentHealthInput {
            component: "test".to_string(),
            status: "Invalid".to_string(),
            message: None,
        };

        let result = handler.handle(input).await;
        assert!(result.is_err());
    }
}
