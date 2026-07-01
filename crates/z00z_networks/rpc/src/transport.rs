//! RPC transport abstraction for different execution environments.
//!
//! This module deliberately owns only the request/response carriage contract.
//! Peer identity, authentication, retry policy, and connection lifecycle are
//! adjacent concerns that higher-level network crates must define around this
//! trait instead of extending it ad hoc from inside the RPC crate.

use crate::error::RpcError;
use async_trait::async_trait;
use z00z_utils::codec::Value;

/// Transport layer for RPC communication.
///
/// Different implementations:
/// - `LocalRpc`: In-process server (Desktop/TUI)
/// - `WasmRpc`: WebSocket client (Browser)
/// - `HttpRpc`: HTTP client (Future: remote backend)
///
/// This trait is intentionally narrow: it carries one method call and returns
/// one typed response or transport error. It does not own peer identity,
/// authentication, retry policy, or connection lifecycle state.
///
/// # WASM Compatibility
///
/// Uses `#[async_trait(?Send)]` to support WASM targets where
/// futures are `!Send` (single-threaded environment).
#[async_trait(?Send)] // ?Send for WASM compatibility
pub trait RpcTransport {
    /// Send RPC request and receive response
    async fn call(&self, method: &str, params: Value) -> Result<Value, RpcError>;
}

// Allow boxed transports to be used transparently as transports.
#[async_trait(?Send)]
impl<T> RpcTransport for Box<T>
where
    T: RpcTransport + ?Sized,
{
    async fn call(&self, method: &str, params: Value) -> Result<Value, RpcError> {
        (**self).call(method, params).await
    }
}
