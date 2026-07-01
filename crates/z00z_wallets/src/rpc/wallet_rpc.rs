//! Wallet RPC method definitions.
//!
//! The `wallet.*` namespace is reserved for wallet session and security operations.
//! Wallet lifecycle (create/import/export/list/remove) lives in `app.*`.

#[cfg(not(target_arch = "wasm32"))]
use jsonrpsee::{core::RpcResult, proc_macros::rpc};

#[cfg(not(target_arch = "wasm32"))]
use super::super::types::common::PersistWalletId;
#[cfg(not(target_arch = "wasm32"))]
use super::super::types::wallet::{
    RuntimeLockWalletResponse, RuntimeShowSeedPhraseResponse, SessionToken, WalletLifecycleEvent,
};

/// Wallet RPC trait defining ALL wallet management operations
///
/// # JSON-RPC 2.0 Methods
///
/// Complete list of wallet.* methods from Z00Z_WALLETS_FOUNDATION.md spec
#[cfg(not(target_arch = "wasm32"))]
#[rpc(server, client)]
pub trait WalletRpc {
    /// Lock wallet
    ///
    /// # JSON-RPC Method
    /// ```json
    /// {"jsonrpc": "2.0", "method": "wallet.session.lock_wallet", "params": {"session": {"token": "...", "wallet_id": "...", "created_at": 0, "expires_at": 0, "last_activity_at": 0, "permissions": []}}, "id": 1}
    /// ```
    #[method(name = "wallet.session.lock_wallet")]
    async fn lock_wallet(&self, session: SessionToken) -> RpcResult<RuntimeLockWalletResponse>;

    /// Show wallet seed phrase
    ///
    /// # JSON-RPC Method
    /// ```json
    /// {"jsonrpc": "2.0", "method": "wallet.session.show_seed_phrase", "params": {"session": {"token": "...", "wallet_id": "...", "created_at": 0, "expires_at": 0, "last_activity_at": 0, "permissions": []}, "password": "***", "confirmation": "I understand"}, "id": 1}
    /// ```
    #[method(name = "wallet.session.show_seed_phrase")]
    async fn show_seed_phrase(
        &self,
        session: SessionToken,
        password: String,
        confirmation: String,
    ) -> RpcResult<RuntimeShowSeedPhraseResponse>;

    /// Unlock wallet with password
    ///
    /// # JSON-RPC Method
    /// ```json
    /// {"jsonrpc": "2.0", "method": "wallet.session.unlock_wallet", "params": {"id": "...", "password": "***"}, "id": 1}
    /// ```
    #[method(name = "wallet.session.unlock_wallet")]
    async fn unlock_wallet(&self, id: PersistWalletId, password: String)
        -> RpcResult<SessionToken>;

    /// Forward a lifecycle event from the UI/app layer into the wallet service.
    ///
    /// # JSON-RPC Method
    /// ```json
    /// {"jsonrpc": "2.0", "method": "wallet.lifecycle.on_event", "params": {"event": "screen_locked"}, "id": 1}
    /// ```
    #[method(name = "wallet.lifecycle.on_event")]
    async fn on_lifecycle_event(&self, event: WalletLifecycleEvent) -> RpcResult<()>;
}
