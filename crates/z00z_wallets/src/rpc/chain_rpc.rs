//! Chain RPC method definitions.
//!
//! This module defines the JSON-RPC 2.0 interface for selecting the active chain.

use async_trait::async_trait;

#[cfg(not(target_arch = "wasm32"))]
use jsonrpsee::{core::RpcResult, proc_macros::rpc};

#[cfg(not(target_arch = "wasm32"))]
use super::super::types::network::RuntimeSwitchChainResponse;

/// Chain selection RPC trait - 3 methods.
///
/// # JSON-RPC 2.0 Methods
/// - `app.chain.switch_to_mainnet`
/// - `app.chain.switch_to_testnet`
/// - `app.chain.switch_to_devnet`
#[cfg(not(target_arch = "wasm32"))]
#[rpc(server, client)]
pub trait ChainRpc {
    /// Switch to mainnet.
    ///
    /// # JSON-RPC Method
    /// ```json
    /// {"jsonrpc": "2.0", "method": "app.chain.switch_to_mainnet", "params": {}, "id": 1}
    /// ```
    #[method(name = "app.chain.switch_to_mainnet")]
    async fn switch_to_mainnet(&self) -> RpcResult<RuntimeSwitchChainResponse>;

    /// Switch to testnet.
    ///
    /// # JSON-RPC Method
    /// ```json
    /// {"jsonrpc": "2.0", "method": "app.chain.switch_to_testnet", "params": {}, "id": 1}
    /// ```
    #[method(name = "app.chain.switch_to_testnet")]
    async fn switch_to_testnet(&self) -> RpcResult<RuntimeSwitchChainResponse>;

    /// Switch to devnet.
    ///
    /// # JSON-RPC Method
    /// ```json
    /// {"jsonrpc": "2.0", "method": "app.chain.switch_to_devnet", "params": {}, "id": 1}
    /// ```
    #[method(name = "app.chain.switch_to_devnet")]
    async fn switch_to_devnet(&self) -> RpcResult<RuntimeSwitchChainResponse>;
}

// ============================================================================
// Scan control (dispatcher-based RPC surface)
// ============================================================================

use crate::rpc::types::chain::{
    RuntimeBlockInfo, RuntimeScanStatus, RuntimeStartScanParams, RuntimeStartScanResponse,
};
use crate::rpc::types::common::PersistWalletId;

/// RPC trait for wallet-local scan orchestration control.
///
/// Note: this is currently wired via the crate's `RpcDispatcher` (app.chain.* methods),
/// not via jsonrpsee's `#[rpc]` macro.
#[async_trait]
pub trait ChainScanRpc: Send + Sync {
    /// Start wallet-local scan orchestration for a wallet.
    async fn start_local_scan(&self, params: RuntimeStartScanParams) -> RuntimeStartScanResponse;

    /// Stop ongoing wallet-local scan orchestration.
    async fn stop_local_scan(&self, wallet_id: PersistWalletId);

    /// Get current wallet-local scan status.
    async fn get_local_scan_status(&self, wallet_id: PersistWalletId) -> RuntimeScanStatus;

    /// Get the latest wallet-local chain-tip observation.
    async fn get_local_scan_tip(&self) -> RuntimeBlockInfo;
}
