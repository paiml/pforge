//! Transport layer implementation
//!
//! This module provides transport creation based on configuration.

use crate::{Error, Result};
use pforge_config::TransportType;
use pmcp::shared::{
    OptimizedSseConfig, OptimizedSseTransport, StdioTransport, Transport, WebSocketConfig,
    WebSocketTransport,
};
use std::time::Duration;

/// Create a transport based on configuration
pub fn create_transport(transport_type: &TransportType) -> Result<Box<dyn Transport>> {
    match transport_type {
        TransportType::Stdio => {
            let transport = StdioTransport::new();
            Ok(Box::new(transport))
        }
        TransportType::Sse => {
            let config = OptimizedSseConfig {
                url: "http://localhost:8080/sse".to_string(),
                connection_timeout: Duration::from_secs(30),
                keepalive_interval: Duration::from_secs(15),
                max_reconnects: 5,
                reconnect_delay: Duration::from_secs(1),
                buffer_size: 100,
                flush_interval: Duration::from_millis(100),
                enable_pooling: true,
                max_connections: 10,
                enable_compression: false,
            };
            let transport = OptimizedSseTransport::new(config);
            Ok(Box::new(transport))
        }
        TransportType::WebSocket => {
            let url = "ws://localhost:8080/ws"
                .parse()
                .map_err(|e| Error::Handler(format!("Invalid WebSocket URL: {}", e)))?;

            let config = WebSocketConfig {
                url,
                auto_reconnect: true,
                reconnect_delay: Duration::from_secs(1),
                max_reconnect_delay: Duration::from_secs(30),
                max_reconnect_attempts: Some(5),
                ping_interval: Some(Duration::from_secs(30)),
                request_timeout: Duration::from_secs(10),
            };
            let transport = WebSocketTransport::new(config);
            Ok(Box::new(transport))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_stdio_transport() {
        let transport = create_transport(&TransportType::Stdio);
        assert!(transport.is_ok());
        let t = transport.unwrap();
        assert_eq!(t.transport_type(), "stdio");
    }

    #[tokio::test]
    async fn test_create_sse_transport() {
        let transport = create_transport(&TransportType::Sse);
        assert!(transport.is_ok());
    }

    #[test]
    fn test_create_websocket_transport() {
        let transport = create_transport(&TransportType::WebSocket);
        assert!(transport.is_ok());
    }

    // Note: SSE and WebSocket tests require server running, so they're integration tests
}
