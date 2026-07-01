//! RPC Layer for Wallet Communication
#![allow(missing_docs)]
//
// Many RPC types and stubs are spec-driven and intentionally minimal during Phase 1.
// We keep `missing_docs` enabled elsewhere, but silence it for this module tree.
//!
//! This module provides JSON-RPC 2.0 communication layer using `jsonrpsee` library.
//!
//! # Architecture
//!
//! ```text
//! Frontend → RPC Client → [Network] → RPC Server → Core Services
//! ```
//!
//! # Why jsonrpsee?
//!
//! - Industry-standard JSON-RPC 2.0 library (used by Substrate, Polkadot)
//! - Type-safe method registration via proc macros
//! - Built-in WebSocket support for WASM
//! - Saves ~500 lines of boilerplate
//!
//! Reference: `json-rpc-decision.md`

// Wallet-specific RPC extensions
pub mod error_mapping;
#[cfg(not(target_arch = "wasm32"))]
pub mod logging;
pub mod methods;
pub mod types;

#[cfg(not(target_arch = "wasm32"))]
mod dispatcher_handlers;

#[cfg(not(target_arch = "wasm32"))]
pub mod app_dispatcher_wiring;

#[cfg(not(target_arch = "wasm32"))]
pub mod wallet_dispatcher_wiring;

pub use error_mapping::map_wallet_error_to_rpc;

#[cfg(not(target_arch = "wasm32"))]
pub fn register_all_wallet_rpc_methods(
    dispatcher: &z00z_networks_rpc::RpcDispatcher,
    app_rpc: std::sync::Arc<methods::AppRpcImpl>,
    wallet_rpc: std::sync::Arc<methods::WalletRpcImpl>,
    asset_rpc: std::sync::Arc<methods::AssetRpcImpl>,
    tx_rpc: std::sync::Arc<methods::TxRpcImpl>,
    backup_rpc: std::sync::Arc<methods::BackupRpcImpl>,
    key_rpc: std::sync::Arc<methods::KeyRpcImpl>,
    chain_rpc: std::sync::Arc<methods::ChainRpcImpl>,
    network_rpc: std::sync::Arc<methods::NetworkRpcImpl>,
    scan_rpc: std::sync::Arc<methods::ChainScanRpcImpl>,
    storage_rpc: std::sync::Arc<methods::StorageRpcImpl>,
) -> crate::WalletResult<()> {
    wallet_dispatcher_wiring::register_all_wallet_rpc_methods(
        dispatcher,
        wallet_dispatcher_wiring::RpcModules {
            app_rpc,
            wallet_rpc,
            asset_rpc,
            tx_rpc,
            backup_rpc,
            key_rpc,
            chain_rpc,
            network_rpc,
            scan_rpc,
            storage_rpc,
        },
    )
}

#[cfg(not(target_arch = "wasm32"))]
pub fn register_all_app_rpc_methods(
    dispatcher: &z00z_networks_rpc::RpcDispatcher,
    app_rpc: std::sync::Arc<methods::AppRpcImpl>,
    chain_rpc: std::sync::Arc<methods::ChainRpcImpl>,
    network_rpc: std::sync::Arc<methods::NetworkRpcImpl>,
    scan_rpc: std::sync::Arc<methods::ChainScanRpcImpl>,
) {
    app_dispatcher_wiring::register_all_app_rpc_methods(
        dispatcher,
        app_rpc,
        chain_rpc,
        network_rpc,
        scan_rpc,
    );
}

#[cfg(not(target_arch = "wasm32"))]
pub use methods::{WalletRpcClient, WalletRpcServer};
