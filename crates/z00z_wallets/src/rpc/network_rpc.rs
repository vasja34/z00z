//! Network RPC method definitions
//!
//! This module defines the JSON-RPC 2.0 interface for network.* methods.

#[cfg(not(target_arch = "wasm32"))]
use jsonrpsee::{core::RpcResult, proc_macros::rpc};

#[cfg(not(target_arch = "wasm32"))]
use super::super::types::network::{RuntimeChainSettingsResponse, RuntimeSwitchChainResponse};

/// Network RPC trait
///
/// # JSON-RPC 2.0 Methods
///
/// - app.network.switch_to_onionet
/// - app.network.switch_to_tor
///
/// NOTE: chain switching (`mainnet`/`testnet`/`devnet`) is exposed via `chain.*`.
#[cfg(not(target_arch = "wasm32"))]
#[rpc(server, client)]
pub trait NetworkRpc {
    /// Switch to onionet
    ///
    /// # JSON-RPC Method
    /// ```json
    /// {"jsonrpc": "2.0", "method": "app.network.switch_to_onionet", "params": {}, "id": 1}
    /// ```
    #[method(name = "app.network.switch_to_onionet")]
    async fn switch_to_onionet(&self) -> RpcResult<RuntimeSwitchChainResponse>;

    /// Enable/disable Tor connection
    ///
    /// # JSON-RPC Method
    /// ```json
    /// {"jsonrpc": "2.0", "method": "app.network.switch_to_tor", "params": {"enable": true}, "id": 1}
    /// ```
    #[method(name = "app.network.switch_to_tor")]
    async fn switch_to_tor(&self, enable: bool) -> RpcResult<RuntimeChainSettingsResponse>;
}
