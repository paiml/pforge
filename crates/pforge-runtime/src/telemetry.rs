//! # Telemetry and Observability
//!
//! Provides comprehensive telemetry, metrics collection, and observability features
//! for pforge MCP servers.
//!
//! ## Features
//!
//! - **Prometheus Metrics**: Request counts, latencies, error rates
//! - **Health Checks**: Readiness and liveness probes
//! - **Distributed Tracing**: OpenTelemetry integration ready
//! - **Structured Logging**: Integration with tracing crate
//!
//! ## Example
//!
//! ```rust
//! use pforge_runtime::telemetry::{MetricsCollector, HealthCheck};
//!
//! # #[tokio::main]
//! # async fn main() {
//! let collector = MetricsCollector::new();
//!
//! // Record a request
//! let start = std::time::Instant::now();
//! // ... handle request ...
//! collector.record_request("greet", start.elapsed(), true);
//!
//! // Check health
//! let health = HealthCheck::new();
//! assert!(health.is_healthy());
//! # }
//! ```

use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant, SystemTime};

/// Prometheus-compatible metrics collector
#[derive(Clone)]
pub struct MetricsCollector {
    /// Total requests by tool name
    request_counts: Arc<dashmap::DashMap<String, AtomicU64>>,
    /// Total errors by tool name
    error_counts: Arc<dashmap::DashMap<String, AtomicU64>>,
    /// Request latencies (sum in microseconds)
    latency_sums: Arc<dashmap::DashMap<String, AtomicU64>>,
    /// Server start time
    start_time: Arc<Instant>,
}

impl MetricsCollector {
    /// Create a new metrics collector
    pub fn new() -> Self {
        Self {
            request_counts: Arc::new(dashmap::DashMap::new()),
            error_counts: Arc::new(dashmap::DashMap::new()),
            latency_sums: Arc::new(dashmap::DashMap::new()),
            start_time: Arc::new(Instant::now()),
        }
    }

    /// Record a request with latency and success status
    pub fn record_request(&self, tool: &str, latency: Duration, success: bool) {
        // Increment request count
        self.request_counts
            .entry(tool.to_string())
            .or_insert_with(|| AtomicU64::new(0))
            .fetch_add(1, Ordering::Relaxed);

        // Record latency
        let micros = latency.as_micros() as u64;
        self.latency_sums
            .entry(tool.to_string())
            .or_insert_with(|| AtomicU64::new(0))
            .fetch_add(micros, Ordering::Relaxed);

        // Record error if applicable
        if !success {
            self.error_counts
                .entry(tool.to_string())
                .or_insert_with(|| AtomicU64::new(0))
                .fetch_add(1, Ordering::Relaxed);
        }
    }

    /// Get total request count for a tool
    pub fn get_request_count(&self, tool: &str) -> u64 {
        self.request_counts
            .get(tool)
            .map(|v| v.load(Ordering::Relaxed))
            .unwrap_or(0)
    }

    /// Get total error count for a tool
    pub fn get_error_count(&self, tool: &str) -> u64 {
        self.error_counts
            .get(tool)
            .map(|v| v.load(Ordering::Relaxed))
            .unwrap_or(0)
    }

    /// Get average latency for a tool in microseconds
    pub fn get_avg_latency_micros(&self, tool: &str) -> Option<f64> {
        let count = self.get_request_count(tool);
        if count == 0 {
            return None;
        }

        let sum = self
            .latency_sums
            .get(tool)
            .map(|v| v.load(Ordering::Relaxed))
            .unwrap_or(0);

        Some(sum as f64 / count as f64)
    }

    /// Get error rate (0.0 to 1.0) for a tool
    pub fn get_error_rate(&self, tool: &str) -> f64 {
        let total = self.get_request_count(tool);
        if total == 0 {
            return 0.0;
        }

        let errors = self.get_error_count(tool);
        errors as f64 / total as f64
    }

    /// Get uptime in seconds
    pub fn uptime_seconds(&self) -> u64 {
        self.start_time.elapsed().as_secs()
    }

    /// Export metrics in Prometheus text format
    pub fn export_prometheus(&self) -> String {
        let mut output = String::new();

        // Request count metric
        output.push_str("# HELP pforge_requests_total Total number of requests\n");
        output.push_str("# TYPE pforge_requests_total counter\n");
        for entry in self.request_counts.iter() {
            let count = entry.value().load(Ordering::Relaxed);
            output.push_str(&format!(
                "pforge_requests_total{{tool=\"{}\"}} {}\n",
                entry.key(),
                count
            ));
        }

        // Error count metric
        output.push_str("# HELP pforge_errors_total Total number of errors\n");
        output.push_str("# TYPE pforge_errors_total counter\n");
        for entry in self.error_counts.iter() {
            let count = entry.value().load(Ordering::Relaxed);
            output.push_str(&format!(
                "pforge_errors_total{{tool=\"{}\"}} {}\n",
                entry.key(),
                count
            ));
        }

        // Latency metric
        output.push_str("# HELP pforge_latency_microseconds_sum Sum of request latencies\n");
        output.push_str("# TYPE pforge_latency_microseconds_sum counter\n");
        for entry in self.latency_sums.iter() {
            let sum = entry.value().load(Ordering::Relaxed);
            output.push_str(&format!(
                "pforge_latency_microseconds_sum{{tool=\"{}\"}} {}\n",
                entry.key(),
                sum
            ));
        }

        // Uptime metric
        output.push_str("# HELP pforge_uptime_seconds Server uptime in seconds\n");
        output.push_str("# TYPE pforge_uptime_seconds gauge\n");
        output.push_str(&format!(
            "pforge_uptime_seconds {}\n",
            self.uptime_seconds()
        ));

        output
    }

    /// Get metrics summary as JSON
    pub fn export_json(&self) -> serde_json::Value {
        let mut tools = serde_json::Map::new();

        for entry in self.request_counts.iter() {
            let tool = entry.key();
            let requests = entry.value().load(Ordering::Relaxed);
            let errors = self.get_error_count(tool);
            let avg_latency = self.get_avg_latency_micros(tool);

            let mut tool_data = serde_json::Map::new();
            tool_data.insert("requests".to_string(), serde_json::json!(requests));
            tool_data.insert("errors".to_string(), serde_json::json!(errors));
            tool_data.insert(
                "error_rate".to_string(),
                serde_json::json!(self.get_error_rate(tool)),
            );
            if let Some(latency) = avg_latency {
                tool_data.insert("avg_latency_micros".to_string(), serde_json::json!(latency));
            }

            tools.insert(tool.clone(), serde_json::Value::Object(tool_data));
        }

        serde_json::json!({
            "uptime_seconds": self.uptime_seconds(),
            "tools": tools
        })
    }
}

impl Default for MetricsCollector {
    fn default() -> Self {
        Self::new()
    }
}

/// Health check status
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HealthStatus {
    /// Service is healthy and ready
    Healthy,
    /// Service is degraded but operational
    Degraded,
    /// Service is unhealthy
    Unhealthy,
}

impl HealthStatus {
    /// Check if status is healthy
    pub fn is_healthy(&self) -> bool {
        matches!(self, HealthStatus::Healthy)
    }

    /// Convert to HTTP status code
    pub fn http_status(&self) -> u16 {
        match self {
            HealthStatus::Healthy => 200,
            HealthStatus::Degraded => 200,
            HealthStatus::Unhealthy => 503,
        }
    }
}

/// Component health check result
#[derive(Debug, Clone)]
pub struct ComponentHealth {
    /// Component name
    pub name: String,
    /// Health status
    pub status: HealthStatus,
    /// Optional message
    pub message: Option<String>,
    /// Check timestamp
    pub timestamp: SystemTime,
}

/// Health check aggregator
#[derive(Clone)]
pub struct HealthCheck {
    /// Component health status
    components: Arc<dashmap::DashMap<String, ComponentHealth>>,
    /// Server start time
    start_time: Arc<SystemTime>,
}

impl HealthCheck {
    /// Create a new health check
    pub fn new() -> Self {
        Self {
            components: Arc::new(dashmap::DashMap::new()),
            start_time: Arc::new(SystemTime::now()),
        }
    }

    /// Register a component health status
    pub fn register_component(&self, name: impl Into<String>, status: HealthStatus) {
        self.register_component_with_message(name, status, None);
    }

    /// Register a component with message
    pub fn register_component_with_message(
        &self,
        name: impl Into<String>,
        status: HealthStatus,
        message: Option<String>,
    ) {
        let name = name.into();
        self.components.insert(
            name.clone(),
            ComponentHealth {
                name,
                status,
                message,
                timestamp: SystemTime::now(),
            },
        );
    }

    /// Get overall health status
    pub fn get_status(&self) -> HealthStatus {
        if self.components.is_empty() {
            return HealthStatus::Healthy;
        }

        let mut has_degraded = false;
        for component in self.components.iter() {
            match component.status {
                HealthStatus::Unhealthy => return HealthStatus::Unhealthy,
                HealthStatus::Degraded => has_degraded = true,
                HealthStatus::Healthy => {}
            }
        }

        if has_degraded {
            HealthStatus::Degraded
        } else {
            HealthStatus::Healthy
        }
    }

    /// Check if service is healthy
    pub fn is_healthy(&self) -> bool {
        self.get_status().is_healthy()
    }

    /// Export health status as JSON
    pub fn export_json(&self) -> serde_json::Value {
        let overall_status = self.get_status();
        let mut components = Vec::new();

        for entry in self.components.iter() {
            let health = entry.value();
            components.push(serde_json::json!({
                "name": health.name,
                "status": format!("{:?}", health.status),
                "message": health.message,
                "timestamp": health.timestamp
                    .duration_since(SystemTime::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs()
            }));
        }

        serde_json::json!({
            "status": format!("{:?}", overall_status),
            "uptime_seconds": SystemTime::now()
                .duration_since(*self.start_time)
                .unwrap_or_default()
                .as_secs(),
            "components": components
        })
    }

    /// Get component health
    pub fn get_component(&self, name: &str) -> Option<ComponentHealth> {
        self.components.get(name).map(|c| c.clone())
    }

    /// Remove component
    pub fn remove_component(&self, name: &str) {
        self.components.remove(name);
    }
}

impl Default for HealthCheck {
    fn default() -> Self {
        Self::new()
    }
}

/// Telemetry middleware for automatic metrics collection
pub struct TelemetryMiddleware {
    /// Metrics collector
    collector: MetricsCollector,
}

impl TelemetryMiddleware {
    /// Create new telemetry middleware
    pub fn new(collector: MetricsCollector) -> Self {
        Self { collector }
    }

    /// Get reference to metrics collector
    pub fn collector(&self) -> &MetricsCollector {
        &self.collector
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_metrics_collector() {
        let collector = MetricsCollector::new();

        // Record successful request
        collector.record_request("greet", Duration::from_micros(100), true);
        assert_eq!(collector.get_request_count("greet"), 1);
        assert_eq!(collector.get_error_count("greet"), 0);
        assert_eq!(collector.get_avg_latency_micros("greet"), Some(100.0));

        // Record failed request
        collector.record_request("greet", Duration::from_micros(200), false);
        assert_eq!(collector.get_request_count("greet"), 2);
        assert_eq!(collector.get_error_count("greet"), 1);
        assert_eq!(collector.get_avg_latency_micros("greet"), Some(150.0));

        // Error rate
        assert_eq!(collector.get_error_rate("greet"), 0.5);
    }

    #[test]
    fn test_prometheus_export() {
        let collector = MetricsCollector::new();
        collector.record_request("greet", Duration::from_micros(100), true);

        let output = collector.export_prometheus();
        assert!(output.contains("pforge_requests_total"));
        assert!(output.contains("pforge_errors_total"));
        assert!(output.contains("pforge_latency_microseconds_sum"));
        assert!(output.contains("pforge_uptime_seconds"));
    }

    #[test]
    fn test_json_export() {
        let collector = MetricsCollector::new();
        collector.record_request("greet", Duration::from_micros(100), true);

        let json = collector.export_json();
        assert!(json["uptime_seconds"].is_u64());
        assert!(json["tools"]["greet"]["requests"].is_u64());
        assert_eq!(json["tools"]["greet"]["requests"], 1);
    }

    #[test]
    fn test_health_check() {
        let health = HealthCheck::new();
        assert!(health.is_healthy());

        // Register healthy component
        health.register_component("database", HealthStatus::Healthy);
        assert_eq!(health.get_status(), HealthStatus::Healthy);

        // Register degraded component
        health.register_component("cache", HealthStatus::Degraded);
        assert_eq!(health.get_status(), HealthStatus::Degraded);

        // Register unhealthy component
        health.register_component("storage", HealthStatus::Unhealthy);
        assert_eq!(health.get_status(), HealthStatus::Unhealthy);
    }

    #[test]
    fn test_health_json_export() {
        let health = HealthCheck::new();
        health.register_component_with_message(
            "service",
            HealthStatus::Healthy,
            Some("All systems operational".to_string()),
        );

        let json = health.export_json();
        assert_eq!(json["status"], "Healthy");
        assert!(json["uptime_seconds"].is_u64());
        assert_eq!(json["components"].as_array().unwrap().len(), 1);
    }

    #[test]
    fn test_component_management() {
        let health = HealthCheck::new();
        health.register_component("test", HealthStatus::Healthy);

        let component = health.get_component("test");
        assert!(component.is_some());
        assert_eq!(component.unwrap().name, "test");

        health.remove_component("test");
        assert!(health.get_component("test").is_none());
    }
}
