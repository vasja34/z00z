//! Asset RPC method definitions - all methods from rpc_naming-spec
//!
//! This module defines the complete JSON-RPC 2.0 interface for asset operations.

#[cfg(not(target_arch = "wasm32"))]
use jsonrpsee::{core::RpcResult, proc_macros::rpc};

#[cfg(not(target_arch = "wasm32"))]
use super::super::types::{
    asset::{
        RuntimeAddAssetResponse, RuntimeAssetBalanceResponse, RuntimeAssetDetailsResponse,
        RuntimeAssetListFilter, RuntimeAssetMetadataResponse, RuntimeImportAssetResponse,
        RuntimeListAssetsResponse, RuntimeMergeAssetsResponse, RuntimeReceiveAssetResponse,
        RuntimeSendAssetResponse, RuntimeSplitAssetResponse, RuntimeStakeAssetsResponse,
        RuntimeSwapAssetsResponse, RuntimeUnstakeAssetsResponse,
    },
    common::PersistWalletId,
    wallet::SessionToken,
};

#[cfg(not(target_arch = "wasm32"))]
use z00z_core::assets::registry::AssetId;

/// Asset RPC trait defining asset management operations.
///
/// # JSON-RPC 2.0 Methods
///
/// Uses the standardized `kernel.service.method` naming from `rpc_naming-spec`.
#[cfg(not(target_arch = "wasm32"))]
#[rpc(server, client)]
pub trait AssetRpc {
    /// List all assets in active wallet
    ///
    /// # JSON-RPC Method
    /// ```json
    /// {
    ///   "jsonrpc": "2.0",
    ///   "method": "wallet.asset.list_assets",
    ///   "params": {
    ///     "wallet_id": "...",
    ///     "limit": 50,
    ///     "cursor": null,
    ///     "filter": {
    ///       "asset_class": null,
    ///       "min_balance": null
    ///     }
    ///   },
    ///   "id": 1
    /// }
    /// ```
    #[method(name = "wallet.asset.list_assets")]
    async fn list_assets(
        &self,
        wallet_id: PersistWalletId,
        limit: Option<usize>,
        cursor: Option<String>,
        filter: Option<RuntimeAssetListFilter>,
    ) -> RpcResult<RuntimeListAssetsResponse>;

    /// Add new asset to wallet
    ///
    /// # JSON-RPC Method
    /// ```json
    /// {"jsonrpc": "2.0", "method": "wallet.asset.add_asset", "params": {"session": {"token": "...", "wallet_id": "...", "created_at": 0, "expires_at": 0, "last_activity_at": 0, "permissions": []}, "asset_data": "..."}, "id": 1}
    /// ```
    #[method(name = "wallet.asset.add_asset")]
    async fn add_asset(
        &self,
        session: SessionToken,
        asset_data: String,
    ) -> RpcResult<RuntimeAddAssetResponse>;

    /// Get asset balance
    ///
    /// # JSON-RPC Method
    /// ```json
    /// {"jsonrpc": "2.0", "method": "wallet.asset.get_asset_balance", "params": {"wallet_id": "...", "asset_id": "..."}, "id": 1}
    /// ```
    #[method(name = "wallet.asset.get_asset_balance")]
    async fn get_asset_balance(
        &self,
        wallet_id: PersistWalletId,
        asset_id: AssetId,
    ) -> RpcResult<RuntimeAssetBalanceResponse>;

    /// Get detailed asset information
    ///
    /// # JSON-RPC Method
    /// ```json
    /// {"jsonrpc": "2.0", "method": "wallet.asset.get_asset_details", "params": {"wallet_id": "...", "asset_id": "..."}, "id": 1}
    /// ```
    #[method(name = "wallet.asset.get_asset_details")]
    async fn get_asset_details(
        &self,
        wallet_id: PersistWalletId,
        asset_id: AssetId,
    ) -> RpcResult<RuntimeAssetDetailsResponse>;

    /// Import asset to wallet
    ///
    /// # JSON-RPC Method
    /// ```json
    /// {"jsonrpc": "2.0", "method": "wallet.asset.import_asset", "params": {"session": {"token": "...", "wallet_id": "...", "created_at": 0, "expires_at": 0, "last_activity_at": 0, "permissions": []}, "asset_data": "..."}, "id": 1}
    /// ```
    #[method(name = "wallet.asset.import_asset")]
    async fn import_asset(
        &self,
        session: SessionToken,
        asset_data: String,
    ) -> RpcResult<RuntimeImportAssetResponse>;

    /// Compatibility merge surface over wallet-owned assets.
    ///
    /// This remains a non-canonical helper until merge is routed through the
    /// same `wallet.tx.*` plus reconcile authority as confirmed ledger updates.
    ///
    /// # JSON-RPC Method
    /// ```json
    /// {"jsonrpc": "2.0", "method": "wallet.asset.merge_assets", "params": {"session": {"token": "...", "wallet_id": "...", "created_at": 0, "expires_at": 0, "last_activity_at": 0, "permissions": []}, "asset_ids": [...]}, "id": 1}
    /// ```
    #[method(name = "wallet.asset.merge_assets")]
    async fn merge_assets(
        &self,
        session: SessionToken,
        asset_ids: Vec<AssetId>,
    ) -> RpcResult<RuntimeMergeAssetsResponse>;

    /// Get asset metadata
    ///
    /// # JSON-RPC Method
    /// ```json
    /// {"jsonrpc": "2.0", "method": "wallet.asset.get_asset_metadata", "params": {"asset_id": "..."}, "id": 1}
    /// ```
    #[method(name = "wallet.asset.get_asset_metadata")]
    async fn get_asset_metadata(
        &self,
        asset_id: AssetId,
    ) -> RpcResult<RuntimeAssetMetadataResponse>;

    /// Resolve one asset by `asset_id`, scan it for wallet ownership, and report receive status.
    /// Phase 037 treats this as a noncanonical single-asset lane over the
    /// existing receiver path; it is not the canonical privacy receive lane and
    /// this method does not claim or import the asset by itself.
    ///
    /// # JSON-RPC Method
    /// ```json
    /// {"jsonrpc": "2.0", "method": "wallet.asset.receive_asset", "params": {"session": {"token": "...", "wallet_id": "...", "created_at": 0, "expires_at": 0, "last_activity_at": 0, "permissions": []}, "asset_id": "..."}, "id": 1}
    /// ```
    #[method(name = "wallet.asset.receive_asset")]
    async fn receive_asset(
        &self,
        session: SessionToken,
        asset_id: AssetId,
    ) -> RpcResult<RuntimeReceiveAssetResponse>;

    /// Guarded send surface for restricted UX flows.
    ///
    /// This method is not the canonical confirmed spend lifecycle; confirmed
    /// spend authority stays on the `wallet.tx.*` plus reconcile path.
    ///
    /// # JSON-RPC Method
    /// ```json
    /// {"jsonrpc": "2.0", "method": "wallet.asset.send_asset", "params": {"session": {"token": "...", "wallet_id": "...", "created_at": 0, "expires_at": 0, "last_activity_at": 0, "permissions": []}, "asset_id": "...", "recipient": "...", "amount": 100}, "id": 1}
    /// ```
    #[method(name = "wallet.asset.send_asset")]
    async fn send_asset(
        &self,
        session: SessionToken,
        asset_id: AssetId,
        recipient: String,
        amount: u64,
    ) -> RpcResult<RuntimeSendAssetResponse>;

    /// Compatibility split surface for deterministic previews.
    ///
    /// This method does not claim canonical ledger mutation authority.
    ///
    /// # JSON-RPC Method
    /// ```json
    /// {"jsonrpc": "2.0", "method": "wallet.asset.split_asset", "params": {"session": {"token": "...", "wallet_id": "...", "created_at": 0, "expires_at": 0, "last_activity_at": 0, "permissions": []}, "asset_id": "...", "amounts": [...]}, "id": 1}
    /// ```
    #[method(name = "wallet.asset.split_asset")]
    async fn split_asset(
        &self,
        session: SessionToken,
        asset_id: AssetId,
        amounts: Vec<u64>,
    ) -> RpcResult<RuntimeSplitAssetResponse>;

    /// Compatibility stake surface for non-canonical UX round-trips.
    ///
    /// This method does not claim canonical ledger mutation authority.
    ///
    /// # JSON-RPC Method
    /// ```json
    /// {"jsonrpc": "2.0", "method": "wallet.asset.stake_assets", "params": {"session": {"token": "...", "wallet_id": "...", "created_at": 0, "expires_at": 0, "last_activity_at": 0, "permissions": []}, "asset_id": "...", "amount": 100}, "id": 1}
    /// ```
    #[method(name = "wallet.asset.stake_assets")]
    async fn stake_assets(
        &self,
        session: SessionToken,
        asset_id: AssetId,
        amount: u64,
    ) -> RpcResult<RuntimeStakeAssetsResponse>;

    /// Compatibility swap surface for guarded previews.
    ///
    /// This method does not claim canonical ledger mutation authority.
    ///
    /// # JSON-RPC Method
    /// ```json
    /// {"jsonrpc": "2.0", "method": "wallet.asset.swap_assets", "params": {"session": {"token": "...", "wallet_id": "...", "created_at": 0, "expires_at": 0, "last_activity_at": 0, "permissions": []}, "from_asset_id": "...", "to_asset_id": "...", "amount": 100}, "id": 1}
    /// ```
    #[method(name = "wallet.asset.swap_assets")]
    async fn swap_assets(
        &self,
        session: SessionToken,
        from_asset_id: AssetId,
        to_asset_id: AssetId,
        amount: u64,
    ) -> RpcResult<RuntimeSwapAssetsResponse>;

    /// Compatibility unstake surface for non-canonical UX round-trips.
    ///
    /// This method does not claim canonical ledger mutation authority.
    ///
    /// # JSON-RPC Method
    /// ```json
    /// {"jsonrpc": "2.0", "method": "wallet.asset.unstake_assets", "params": {"session": {"token": "...", "wallet_id": "...", "created_at": 0, "expires_at": 0, "last_activity_at": 0, "permissions": []}, "stake_id": "..."}, "id": 1}
    /// ```
    #[method(name = "wallet.asset.unstake_assets")]
    async fn unstake_assets(
        &self,
        session: SessionToken,
        stake_id: String,
    ) -> RpcResult<RuntimeUnstakeAssetsResponse>;
}
