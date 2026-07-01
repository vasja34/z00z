//! Transaction RPC method definitions - all methods from rpc_naming-spec
//!
//! This module defines the complete JSON-RPC 2.0 interface for transaction operations.

#[cfg(not(target_arch = "wasm32"))]
use jsonrpsee::{core::RpcResult, proc_macros::rpc};

#[cfg(not(target_arch = "wasm32"))]
use super::super::types::security::IdempotencyKey;

#[cfg(not(target_arch = "wasm32"))]
use super::super::types::{
    tx::{
        PersistTxId, PersistTxInfo, RuntimeBroadcastTxResponse, RuntimeBuildTxResponse,
        RuntimeCancelTxResponse, RuntimeEstimateFeeResponse, RuntimeExportTxResponse,
        RuntimeImportTxResponse, RuntimePaginatedResponse, RuntimePaginationParams,
        RuntimeReconcileTxResponse, RuntimeSendTxResponse, RuntimeTxDetailsResponse,
        RuntimeTxHistoryFilter, RuntimeTxHistorySort, RuntimeVerifyTxPkgResponse,
    },
    wallet::SessionToken,
};

/// Transaction RPC trait defining transaction management operations.
///
/// # JSON-RPC 2.0 Methods
///
/// Uses the standardized `kernel.service.method` naming from `rpc_naming-spec`.
#[cfg(not(target_arch = "wasm32"))]
#[rpc(server, client)]
pub trait TxRpc {
    /// Send transaction
    ///
    /// # JSON-RPC Method
    /// ```json
    /// {"jsonrpc": "2.0", "method": "wallet.tx.send_transaction", "params": {"session": {"wallet_id": "...", "token": "..."}, "recipient": "...", "amount": 100, "asset_id": null, "memo": null, "idempotency_key": null, "timestamp": 1700000000}, "id": 1}
    /// ```
    #[method(name = "wallet.tx.send_transaction")]
    #[allow(clippy::too_many_arguments)]
    async fn send_transaction(
        &self,
        session: SessionToken,
        recipient: String,
        amount: u64,
        asset_id: Option<String>,
        memo: Option<String>,
        idempotency_key: Option<IdempotencyKey>,
        timestamp: Option<u64>,
    ) -> RpcResult<RuntimeSendTxResponse>;

    /// Broadcast signed transaction
    ///
    /// # JSON-RPC Method
    /// ```json
    /// {"jsonrpc": "2.0", "method": "wallet.tx.broadcast_transaction", "params": {"session": {"wallet_id": "...", "token": "..."}, "tx_data": "..."}, "id": 1}
    /// ```
    #[method(name = "wallet.tx.broadcast_transaction")]
    async fn broadcast_transaction(
        &self,
        session: SessionToken,
        tx_data: String,
    ) -> RpcResult<RuntimeBroadcastTxResponse>;

    /// Build unsigned transaction
    ///
    /// # JSON-RPC Method
    /// ```json
    /// {"jsonrpc": "2.0", "method": "wallet.tx.build_transaction", "params": {"session": {"wallet_id": "...", "token": "..."}, "recipient": "...", "amount": 100, "asset_id": null}, "id": 1}
    /// ```
    #[method(name = "wallet.tx.build_transaction")]
    async fn build_transaction(
        &self,
        session: SessionToken,
        recipient: String,
        amount: u64,
        asset_id: Option<String>,
    ) -> RpcResult<RuntimeBuildTxResponse>;

    /// Verify a prepared transaction package and extract wallet-owned outputs.
    ///
    /// # JSON-RPC Method
    /// ```json
    /// {"jsonrpc": "2.0", "method": "wallet.tx.verify_transaction_package", "params": {"session": {"wallet_id": "...", "token": "..."}, "tx_data": "..."}, "id": 1}
    /// ```
    #[method(name = "wallet.tx.verify_transaction_package")]
    async fn verify_transaction_package(
        &self,
        session: SessionToken,
        tx_data: String,
    ) -> RpcResult<RuntimeVerifyTxPkgResponse>;

    /// Cancel pending transaction
    ///
    /// # JSON-RPC Method
    /// ```json
    /// {"jsonrpc": "2.0", "method": "wallet.tx.cancel_transaction", "params": {"session": {"wallet_id": "...", "token": "..."}, "tx_id": "..."}, "id": 1}
    /// ```
    #[method(name = "wallet.tx.cancel_transaction")]
    async fn cancel_transaction(
        &self,
        session: SessionToken,
        tx_id: PersistTxId,
    ) -> RpcResult<RuntimeCancelTxResponse>;

    /// Get transaction details
    ///
    /// # JSON-RPC Method
    /// ```json
    /// {"jsonrpc": "2.0", "method": "wallet.tx.get_transaction_details", "params": {"session": {"wallet_id": "...", "token": "..."}, "tx_id": "..."}, "id": 1}
    /// ```
    #[method(name = "wallet.tx.get_transaction_details")]
    async fn get_transaction_details(
        &self,
        session: SessionToken,
        tx_id: PersistTxId,
    ) -> RpcResult<RuntimeTxDetailsResponse>;

    /// Estimate transaction fee
    ///
    /// # JSON-RPC Method
    /// ```json
    /// {"jsonrpc": "2.0", "method": "wallet.tx.estimate_transaction_fee", "params": {"session": {"wallet_id": "...", "token": "..."}, "recipient": "...", "amount": 100, "asset_id": null}, "id": 1}
    /// ```
    #[method(name = "wallet.tx.estimate_transaction_fee")]
    async fn estimate_transaction_fee(
        &self,
        session: SessionToken,
        recipient: String,
        amount: u64,
        asset_id: Option<String>,
    ) -> RpcResult<RuntimeEstimateFeeResponse>;

    /// Export transaction data
    ///
    /// # JSON-RPC Method
    /// ```json
    /// {"jsonrpc": "2.0", "method": "wallet.tx.export_transaction", "params": {"session": {"wallet_id": "...", "token": "..."}, "tx_id": "..."}, "id": 1}
    /// ```
    #[method(name = "wallet.tx.export_transaction")]
    async fn export_transaction(
        &self,
        session: SessionToken,
        tx_id: PersistTxId,
    ) -> RpcResult<RuntimeExportTxResponse>;

    /// Import a portable transaction package.
    ///
    /// # JSON-RPC Method
    /// ```json
    /// {"jsonrpc": "2.0", "method": "wallet.tx.import_transaction", "params": {"session": {"wallet_id": "...", "token": "..."}, "tx_data": "..."}, "id": 1}
    /// ```
    #[method(name = "wallet.tx.import_transaction")]
    async fn import_transaction(
        &self,
        session: SessionToken,
        tx_data: String,
    ) -> RpcResult<RuntimeImportTxResponse>;

    /// Reconcile a pending transaction with the admission boundary.
    ///
    /// # JSON-RPC Method
    /// ```json
    /// {"jsonrpc": "2.0", "method": "wallet.tx.reconcile_transaction", "params": {"session": {"wallet_id": "...", "token": "..."}, "tx_id": "..."}, "id": 1}
    /// ```
    #[method(name = "wallet.tx.reconcile_transaction")]
    async fn reconcile_transaction(
        &self,
        session: SessionToken,
        tx_id: PersistTxId,
    ) -> RpcResult<RuntimeReconcileTxResponse>;

    /// Get transaction history
    ///
    /// # JSON-RPC Method
    /// ```json
    /// {"jsonrpc": "2.0", "method": "wallet.tx.get_transaction_history", "params": {"session": {"wallet_id": "...", "token": "..."}, "pagination": {"limit": 50, "cursor": null}, "filter": null, "sort": null}, "id": 1}
    /// ```
    #[method(name = "wallet.tx.get_transaction_history")]
    async fn get_transaction_history(
        &self,
        session: SessionToken,
        pagination: RuntimePaginationParams,
        filter: Option<RuntimeTxHistoryFilter>,
        sort: Option<RuntimeTxHistorySort>,
    ) -> RpcResult<RuntimePaginatedResponse<PersistTxInfo>>;

    /// List all pending transactions
    ///
    /// # JSON-RPC Method
    /// ```json
    /// {"jsonrpc": "2.0", "method": "wallet.tx.list_pending_transactions", "params": {"session": {"wallet_id": "...", "token": "..."}, "pagination": {"limit": 50, "cursor": null}}, "id": 1}
    /// ```
    #[method(name = "wallet.tx.list_pending_transactions")]
    async fn list_pending_transactions(
        &self,
        session: SessionToken,
        pagination: RuntimePaginationParams,
    ) -> RpcResult<RuntimePaginatedResponse<PersistTxInfo>>;
}
