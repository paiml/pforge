use pforge_runtime::{Handler, MetricsCollector, Result};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Debug, Deserialize, JsonSchema)]
pub struct GetMetricsInput {}

#[derive(Debug, Serialize, JsonSchema)]
pub struct GetMetricsOutput {
    pub format: String,
    pub metrics: String,
}

pub struct GetMetricsHandler {
    pub collector: Arc<MetricsCollector>,
}

#[async_trait::async_trait]
impl Handler for GetMetricsHandler {
    type Input = GetMetricsInput;
    type Output = GetMetricsOutput;
    type Error = pforge_runtime::Error;

    async fn handle(&self, _input: Self::Input) -> Result<Self::Output> {
        let prometheus = self.collector.export_prometheus();

        Ok(GetMetricsOutput {
            format: "prometheus".to_string(),
            metrics: prometheus,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[tokio::test]
    async fn test_get_metrics_handler() {
        let collector = Arc::new(MetricsCollector::new());
        collector.record_request("test", Duration::from_micros(100), true);

        let handler = GetMetricsHandler {
            collector: collector.clone(),
        };

        let output = handler.handle(GetMetricsInput {}).await.unwrap();
        assert_eq!(output.format, "prometheus");
        assert!(output.metrics.contains("pforge_requests_total"));
    }
}
