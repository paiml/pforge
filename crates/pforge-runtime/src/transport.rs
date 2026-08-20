//! Transport layer implementation
//!
//! This module provides transport creation based on configuration.

use crate::{Error, Result};
use pforge_config::TransportType;
// `OptimizedSseTransport` is deprecated in pmcp 2.x in favour of
// `StreamableHttpTransport`, "which bounds every peer-controlled read".
//
// NOT migrated here, deliberately. The two are different wire protocols with
// unrelated configs — `OptimizedSseConfig` carries keepalive, reconnect,
// pooling and compression knobs that `StreamableHttpTransportConfig` (url,
// extra_headers, auth_provider, session) has no equivalent for. Swapping them
// changes what `transport: sse` actually speaks, so it breaks every client
// configured against a pforge SSE endpoint. That is a product decision for a
// pforge release, not a side effect of a dependency bump. pmcp keeps the type
// "for 2.x compatibility", so it remains available meanwhile.
//
// The deprecation does flag a real exposure — an unbounded peer-controlled
// read is a DoS vector — so this should not sit indefinitely. Tracked
// separately.
#[cfg(feature = "sse")]
#[allow(deprecated)]
use pmcp::shared::{OptimizedSseConfig, OptimizedSseTransport};
use pmcp::shared::{StdioTransport, Transport};
#[cfg(feature = "websocket")]
use pmcp::shared::{WebSocketConfig, WebSocketTransport};
#[cfg(any(feature = "sse", feature = "websocket"))]
use std::time::Duration;

/// Create a transport based on configuration
pub fn create_transport(transport_type: &TransportType) -> Result<Box<dyn Transport>> {
    match transport_type {
        TransportType::Stdio => {
            let transport = StdioTransport::new();
            Ok(Box::new(transport))
        }
        TransportType::Sse => create_sse_transport(),
        TransportType::WebSocket => create_websocket_transport(),
    }
}

// Infallible in this configuration, but the signature must match the
// `not(feature = "sse")` variant, which is the whole point of the split.
#[allow(clippy::unnecessary_wraps)]
#[cfg(feature = "sse")]
fn create_sse_transport() -> Result<Box<dyn Transport>> {
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
    #[allow(deprecated)]
    let transport = OptimizedSseTransport::new(config);
    Ok(Box::new(transport))
}

#[cfg(not(feature = "sse"))]
fn create_sse_transport() -> Result<Box<dyn Transport>> {
    Err(Error::feature_disabled("sse", "transport `sse`"))
}

#[cfg(feature = "websocket")]
fn create_websocket_transport() -> Result<Box<dyn Transport>> {
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

#[cfg(not(feature = "websocket"))]
fn create_websocket_transport() -> Result<Box<dyn Transport>> {
    Err(Error::feature_disabled(
        "websocket",
        "transport `websocket`",
    ))
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

    // The SSE/WebSocket cases are asserted in BOTH feature configurations, because
    // the bug being guarded here was a build that did not exist rather than a
    // behaviour that was wrong: `--no-default-features` failed to COMPILE, so a
    // test written only for the default build could never have caught it.

    #[cfg(feature = "sse")]
    #[tokio::test]
    async fn test_create_sse_transport() {
        let transport = create_transport(&TransportType::Sse);
        assert!(transport.is_ok());
    }

    #[cfg(not(feature = "sse"))]
    #[test]
    fn test_sse_without_feature_errors_and_names_the_feature() {
        let msg = create_transport(&TransportType::Sse)
            .expect_err("sse must fail when compiled out, not fall back to stdio")
            .to_string();
        assert!(
            msg.contains("sse") && msg.contains("--features"),
            "the error must tell the operator how to fix it, got: {msg}"
        );
    }

    #[cfg(feature = "websocket")]
    #[test]
    fn test_create_websocket_transport() {
        let transport = create_transport(&TransportType::WebSocket);
        assert!(transport.is_ok());
    }

    #[cfg(not(feature = "websocket"))]
    #[test]
    fn test_websocket_without_feature_errors_and_names_the_feature() {
        let msg = create_transport(&TransportType::WebSocket)
            .expect_err("websocket must fail when compiled out, not fall back to stdio")
            .to_string();
        assert!(
            msg.contains("websocket") && msg.contains("--features"),
            "the error must tell the operator how to fix it, got: {msg}"
        );
    }

    // Stdio is unconditional, so it must work in EVERY configuration. If a future
    // refactor puts stdio behind a feature, this fails in the minimal build.
    #[test]
    fn test_stdio_works_in_every_feature_configuration() {
        assert!(create_transport(&TransportType::Stdio).is_ok());
    }
}
