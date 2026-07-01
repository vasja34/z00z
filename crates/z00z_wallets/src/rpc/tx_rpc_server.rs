use super::{
    async_trait, compute_wallet_file_id, is_import_ready, tx_rpc_admission, tx_rpc_broadcast,
    tx_rpc_rate_limits, tx_rpc_support, tx_runtime_state, BroadcastError, Codec, ErrorObjectOwned,
    IdempotencyKey, PersistTxId, PersistTxInfo, PersistWalletId, PortableWalletTxPackage,
    RpcResult, RuntimeAdmissionReceipt, RuntimeBroadcastTxResponse, RuntimeBuildTxResponse,
    RuntimeCancelTxResponse, RuntimeConfirmationReceipt, RuntimeEstimateFeeResponse,
    RuntimeExportTxResponse, RuntimeImportTxResponse, RuntimeOperationStatus,
    RuntimePaginatedResponse, RuntimePaginationParams, RuntimeRateLimitError, RuntimeRateLimitTier,
    RuntimeReconcileTxResponse, RuntimeSendTxResponse, RuntimeTxDetailsResponse,
    RuntimeTxErrorCode, RuntimeTxHistoryFilter, RuntimeTxHistorySort, RuntimeTxLifecycle,
    RuntimeVerifyTxPkgResponse, SecurityErrorCode, SessionToken, SortDirection, SystemRngProvider,
    TxConfirmationEvidence, TxHistorySortBy, TxRpcImpl, TxRpcServer, TxStatus, TxStorage,
    TxSubmitterRole, TX_BROADCAST_MAX_RETRIES, TX_SEND_TIMESTAMP_WINDOW_SECONDS,
};

include!("tx_rpc_server_send.rs");
include!("tx_rpc_server_lifecycle.rs");
include!("tx_rpc_server_finalize.rs");
include!("tx_rpc_server_history.rs");
include!("tx_rpc_server_helpers.rs");

#[async_trait]
impl TxRpcServer for TxRpcImpl {
    async fn send_transaction(
        &self,
        session: SessionToken,
        recipient: String,
        amount: u64,
        asset_id: Option<String>,
        memo: Option<String>,
        idempotency_key: Option<IdempotencyKey>,
        timestamp: Option<u64>,
    ) -> RpcResult<RuntimeSendTxResponse> {
        self.send_transaction_impl(
            session,
            recipient,
            amount,
            asset_id,
            memo,
            idempotency_key,
            timestamp,
        )
        .await
    }

    async fn broadcast_transaction(
        &self,
        session: SessionToken,
        tx_data: String,
    ) -> RpcResult<RuntimeBroadcastTxResponse> {
        self.broadcast_transaction_impl(session, tx_data, None, None)
            .await
    }

    async fn build_transaction(
        &self,
        session: SessionToken,
        recipient: String,
        amount: u64,
        asset_id: Option<String>,
    ) -> RpcResult<RuntimeBuildTxResponse> {
        self.build_transaction_impl(session, recipient, amount, asset_id)
            .await
    }

    async fn verify_transaction_package(
        &self,
        session: SessionToken,
        tx_data: String,
    ) -> RpcResult<RuntimeVerifyTxPkgResponse> {
        self.verify_transaction_package_impl(session, tx_data).await
    }

    async fn cancel_transaction(
        &self,
        session: SessionToken,
        tx_id: PersistTxId,
    ) -> RpcResult<RuntimeCancelTxResponse> {
        self.cancel_transaction_impl(session, tx_id).await
    }

    async fn get_transaction_details(
        &self,
        session: SessionToken,
        tx_id: PersistTxId,
    ) -> RpcResult<RuntimeTxDetailsResponse> {
        self.get_transaction_details_impl(session, tx_id).await
    }

    async fn estimate_transaction_fee(
        &self,
        session: SessionToken,
        recipient: String,
        amount: u64,
        asset_id: Option<String>,
    ) -> RpcResult<RuntimeEstimateFeeResponse> {
        self.estimate_transaction_fee_impl(session, recipient, amount, asset_id)
            .await
    }

    async fn export_transaction(
        &self,
        session: SessionToken,
        tx_id: PersistTxId,
    ) -> RpcResult<RuntimeExportTxResponse> {
        self.export_transaction_impl(session, tx_id).await
    }

    async fn import_transaction(
        &self,
        session: SessionToken,
        tx_data: String,
    ) -> RpcResult<RuntimeImportTxResponse> {
        self.import_transaction_impl(session, tx_data).await
    }

    async fn reconcile_transaction(
        &self,
        session: SessionToken,
        tx_id: PersistTxId,
    ) -> RpcResult<RuntimeReconcileTxResponse> {
        self.reconcile_transaction_impl(session, tx_id).await
    }

    async fn get_transaction_history(
        &self,
        session: SessionToken,
        pagination: RuntimePaginationParams,
        filter: Option<RuntimeTxHistoryFilter>,
        sort: Option<RuntimeTxHistorySort>,
    ) -> RpcResult<RuntimePaginatedResponse<PersistTxInfo>> {
        self.get_transaction_history_impl(session, pagination, filter, sort)
            .await
    }

    async fn list_pending_transactions(
        &self,
        session: SessionToken,
        pagination: RuntimePaginationParams,
    ) -> RpcResult<RuntimePaginatedResponse<PersistTxInfo>> {
        self.list_pending_transactions_impl(session, pagination)
            .await
    }
}
