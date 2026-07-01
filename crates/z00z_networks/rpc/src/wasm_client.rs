//! WASM RPC Client Implementation
//!
//! WebSocket-based RPC client for browser environments.
//! Uses jsonrpsee's WasmClientBuilder for WASM compatibility.

#[cfg(target_arch = "wasm32")]
use async_trait::async_trait;
#[cfg(target_arch = "wasm32")]
use jsonrpsee::core::client::ClientT;
#[cfg(target_arch = "wasm32")]
use jsonrpsee::wasm_client::WasmClientBuilder;
#[cfg(target_arch = "wasm32")]
use std::sync::Arc;
#[cfg(any(target_arch = "wasm32", test))]
use z00z_utils::codec::Value;

#[cfg(target_arch = "wasm32")]
use crate::{RpcError, RpcTransport};

#[cfg(any(target_arch = "wasm32", test))]
fn value_shape(value: &Value) -> String {
    match value {
        Value::Null => String::from("null"),
        Value::Bool(_) => String::from("bool"),
        Value::Number(_) => String::from("number"),
        Value::String(text) => format!("string(len={})", text.len()),
        Value::Array(items) => format!("array(len={})", items.len()),
        Value::Object(items) => format!("object(keys={})", items.len()),
    }
}

#[cfg(any(target_arch = "wasm32", test))]
fn req_log(method: &str, params: &Value) -> String {
    format!("RPC call method={method} params={}", value_shape(params))
}

#[cfg(any(target_arch = "wasm32", test))]
fn resp_log(response: &Value) -> String {
    format!("RPC response summary={}", value_shape(response))
}

/// WebSocket RPC client for WASM environments
///
/// Wraps jsonrpsee's WasmClient for browser WebSocket communication.
///
/// # Example
///
/// ```no_run
/// use z00z_networks_rpc::WasmRpcClient;
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let client = WasmRpcClient::new("ws://localhost:9944").await?;
/// let result = client
///     .call("wallet.list", z00z_utils::codec::Value::Object(Default::default()))
///     .await?;
/// # Ok(())
/// # }
/// ```
#[cfg(target_arch = "wasm32")]
pub struct WasmRpcClient {
    /// Inner jsonrpsee WASM client (generic over service layer)
    client: jsonrpsee::core::client::Client,
    /// Logger for diagnostics (optional, requires "logger" feature)
    #[cfg(feature = "logger")]
    logger: Option<Arc<dyn z00z_utils::logger::Logger>>,
}

#[cfg(target_arch = "wasm32")]
impl WasmRpcClient {
    /// Create new WASM RPC client
    ///
    /// Connects to the specified WebSocket URL.
    ///
    /// # Arguments
    ///
    /// * `worker_url` - WebSocket URL (e.g., "ws://localhost:9944")
    ///
    /// # Errors
    ///
    /// Returns `RpcError::TransportError` if connection fails.
    pub async fn new(worker_url: &str) -> Result<Self, RpcError> {
        Self::new_with_logger(worker_url, None).await
    }

    /// Create new WASM RPC client with logger
    ///
    /// # Arguments
    ///
    /// * `worker_url` - WebSocket URL
    /// * `logger` - Optional logger for diagnostics (requires "logger" feature)
    #[cfg(feature = "logger")]
    pub async fn new_with_logger(
        worker_url: &str,
        logger: Option<Arc<dyn z00z_utils::logger::Logger>>,
    ) -> Result<Self, RpcError> {
        if let Some(ref log) = logger {
            let _ = worker_url;
            log.info("Connecting to worker endpoint");
        }

        let client = WasmClientBuilder::default()
            .build(worker_url)
            .await
            .map_err(|e| RpcError::TransportError(e.to_string()))?;

        if let Some(ref log) = logger {
            log.info("Connected to worker endpoint");
        }

        Ok(Self { client, logger })
    }

    /// Create new WASM RPC client with logger (no-op when logger feature disabled)
    ///
    /// # Arguments
    ///
    /// * `worker_url` - WebSocket URL
    /// * `_logger` - Ignored when logger feature is disabled
    #[cfg(not(feature = "logger"))]
    pub async fn new_with_logger(
        worker_url: &str,
        _logger: Option<Arc<()>>, // Dummy type when feature disabled
    ) -> Result<Self, RpcError> {
        let client = WasmClientBuilder::default()
            .build(worker_url)
            .await
            .map_err(|e| RpcError::TransportError(e.to_string()))?;

        Ok(Self { client })
    }
}

#[cfg(target_arch = "wasm32")]
#[async_trait(?Send)] // WASM is !Send (single-threaded)
impl RpcTransport for WasmRpcClient {
    async fn call(&self, method: &str, params: Value) -> Result<Value, RpcError> {
        #[cfg(feature = "logger")]
        if let Some(ref logger) = self.logger {
            logger.debug(&req_log(method, &params));
        }

        let response: Value = self
            .client
            .request(method, jsonrpsee::rpc_params![params])
            .await
            .map_err(|e| RpcError::RequestFailed(e.to_string()))?;

        #[cfg(feature = "logger")]
        if let Some(ref logger) = self.logger {
            logger.debug(&resp_log(&response));
        }

        Ok(response)
    }
}

#[cfg(test)]
mod redaction_tests {
    use super::*;
    use z00z_utils::codec::json;

    #[test]
    fn test_wasm_log_redaction() {
        let msg = req_log(
            "wallet.key.rotate_master_key",
            &json!({
                "password": "StrongPassw0rd!",
                "seed_phrase": "seed words",
                "receiver_secret": "secret"
            }),
        );

        assert!(msg.contains("wallet.key.rotate_master_key"));
        assert!(msg.contains("object(keys=3)"));
        assert!(!msg.contains("StrongPassw0rd!"));
        assert!(!msg.contains("seed words"));
        assert!(!msg.contains("receiver_secret"));
    }

    #[test]
    fn test_wasm_resp_redaction() {
        let msg = resp_log(&json!({
            "wallet_id": "wallet-123",
            "session": "raw-session-token",
            "seed_phrase": "seed words"
        }));

        assert!(msg.contains("object(keys=3)"));
        assert!(!msg.contains("wallet-123"));
        assert!(!msg.contains("raw-session-token"));
        assert!(!msg.contains("seed words"));
    }
}

#[cfg(test)]
#[cfg(target_arch = "wasm32")]
mod tests {
    use super::*;
    use wasm_bindgen_test::*;

    wasm_bindgen_test_configure!(run_in_browser);

    #[wasm_bindgen_test]
    async fn test_wasm_rpc_client_creation() {
        // This test requires a running Web Worker or backend
        // In real tests, mock the WebSocket connection
        let result = WasmRpcClient::new("ws://localhost:9944").await;

        // We expect connection to fail in test environment (no worker running)
        // This just verifies the code compiles and error handling works
        assert!(result.is_err());
    }
}
