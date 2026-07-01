//! RPC implementations for tx.* wallet methods.

use async_trait::async_trait;
use jsonrpsee::core::RpcResult;
use jsonrpsee::types::ErrorObjectOwned;
use std::sync::Arc;
use tokio::sync::RwLock;
#[cfg(test)]
use z00z_core::assets::AssetPkgWire;
use z00z_utils::codec::{Codec, JsonCodec};
use z00z_utils::rng::SystemRngProvider;
use z00z_utils::time::{SystemTimeProvider, TimeProvider};

use super::{
    tx_rpc::TxRpcServer,
    tx_rpc_admission::{self, SimulatedWalletTxAdmitter, WalletTxAdmitter},
    tx_rpc_broadcast::{self, TX_BROADCAST_MAX_RETRIES},
    tx_rpc_idempotency::{self, TxIdempotencyCache},
    tx_rpc_rate_limits::{self, TxBuildRateLimitStore, TxSendRateLimitStore},
    tx_rpc_support,
    tx_runtime_state::{
        self, ConfirmationEvidenceStore, PendingTxBytesStore, PendingTxStore, SharedTxStore,
    },
};
use crate::{
    chain::BroadcastError,
    domains::hashing::compute_wallet_file_id,
    persistence::tx::TxStorageImpl,
    persistence::{TxConfirmationEvidence, TxStorage},
    rpc::error_mapping::map_wallet_error_to_rpc,
    rpc::types::security::{RuntimeRateLimitError, RuntimeRateLimitTier, SecurityErrorCode},
    rpc::types::{
        common::PersistWalletId,
        common::RuntimeOperationStatus,
        security::IdempotencyKey,
        tx::{
            PersistTxId, PersistTxInfo, PortableWalletTxPackage, RuntimeAdmissionReceipt,
            RuntimeBroadcastTxResponse, RuntimeBuildTxResponse, RuntimeCancelTxResponse,
            RuntimeConfirmationReceipt, RuntimeEstimateFeeResponse, RuntimeExportTxResponse,
            RuntimeImportTxResponse, RuntimePaginatedResponse, RuntimePaginationParams,
            RuntimeReconcileTxResponse, RuntimeSendTxResponse, RuntimeTxDetailsResponse,
            RuntimeTxErrorCode, RuntimeTxHistoryFilter, RuntimeTxHistorySort, RuntimeTxLifecycle,
            RuntimeVerifyTxPkgOut, RuntimeVerifyTxPkgResponse, SortDirection, TxHistorySortBy,
            TxStatus, TxSubmitterRole,
        },
        wallet::SessionToken,
    },
    services::WalletService,
    tx::{
        ThinIndexError, ThinIndexStore, ThinSnapshot, ThinSnapshotCache, ThinSnapshotPin,
        ThinTransportPayload, ThinWalletTxPackage, TxPackage,
    },
    WalletError,
};

const TX_SEND_TIMESTAMP_WINDOW_SECONDS: u64 = 5 * 60;

#[path = "tx_rpc_server.rs"]
mod tx_rpc_server;

pub struct TxRpcImpl {
    service: Arc<WalletService>,
    time_provider: Arc<dyn TimeProvider>,
    tx_admitter: Arc<dyn WalletTxAdmitter>,
    tx_send_rate_limits: TxSendRateLimitStore,
    tx_build_rate_limits: TxBuildRateLimitStore,
    idempotency_cache: TxIdempotencyCache,
    pending_txs: PendingTxStore,
    pending_tx_bytes: PendingTxBytesStore,
    confirmation_evidence: ConfirmationEvidenceStore,
    thin_index: Arc<RwLock<ThinIndexStore>>,
    thin_cache: Arc<RwLock<ThinSnapshotCache>>,
    tx_store: Option<SharedTxStore>,
}

impl TxRpcImpl {
    pub fn new(service: Arc<WalletService>) -> Self {
        Self {
            service,
            time_provider: Arc::new(SystemTimeProvider),
            tx_admitter: Arc::new(SimulatedWalletTxAdmitter),
            tx_send_rate_limits: Arc::new(RwLock::new(Vec::new())),
            tx_build_rate_limits: Arc::new(RwLock::new(Vec::new())),
            idempotency_cache: Arc::new(RwLock::new(Vec::new())),
            pending_txs: Arc::new(RwLock::new(Vec::new())),
            pending_tx_bytes: Arc::new(RwLock::new(Vec::new())),
            confirmation_evidence: Arc::new(RwLock::new(Vec::new())),
            thin_index: Arc::new(RwLock::new(ThinIndexStore::new())),
            thin_cache: Arc::new(RwLock::new(ThinSnapshotCache::new())),
            tx_store: None,
        }
    }

    pub fn with_dependencies(
        service: Arc<WalletService>,
        time_provider: Arc<dyn TimeProvider>,
    ) -> Self {
        Self {
            service,
            time_provider,
            tx_admitter: Arc::new(SimulatedWalletTxAdmitter),
            tx_send_rate_limits: Arc::new(RwLock::new(Vec::new())),
            tx_build_rate_limits: Arc::new(RwLock::new(Vec::new())),
            idempotency_cache: Arc::new(RwLock::new(Vec::new())),
            pending_txs: Arc::new(RwLock::new(Vec::new())),
            pending_tx_bytes: Arc::new(RwLock::new(Vec::new())),
            confirmation_evidence: Arc::new(RwLock::new(Vec::new())),
            thin_index: Arc::new(RwLock::new(ThinIndexStore::new())),
            thin_cache: Arc::new(RwLock::new(ThinSnapshotCache::new())),
            tx_store: None,
        }
    }

    pub fn with_dependencies_and_tx_store(
        service: Arc<WalletService>,
        time_provider: Arc<dyn TimeProvider>,
        tx_store: SharedTxStore,
    ) -> Self {
        Self {
            service,
            time_provider,
            tx_admitter: Arc::new(SimulatedWalletTxAdmitter),
            tx_send_rate_limits: Arc::new(RwLock::new(Vec::new())),
            tx_build_rate_limits: Arc::new(RwLock::new(Vec::new())),
            idempotency_cache: Arc::new(RwLock::new(Vec::new())),
            pending_txs: Arc::new(RwLock::new(Vec::new())),
            pending_tx_bytes: Arc::new(RwLock::new(Vec::new())),
            confirmation_evidence: Arc::new(RwLock::new(Vec::new())),
            thin_index: Arc::new(RwLock::new(ThinIndexStore::new())),
            thin_cache: Arc::new(RwLock::new(ThinSnapshotCache::new())),
            tx_store: Some(tx_store),
        }
    }

    /// Publish one signed thin snapshot into the local helper index store.
    pub async fn publish_thin_snapshot(
        &self,
        snapshot: ThinSnapshot,
    ) -> Result<(), ThinIndexError> {
        self.thin_index.write().await.publish_snapshot(snapshot)
    }

    /// Fetch one thin snapshot by pinned digest.
    pub async fn thin_snapshot(
        &self,
        snapshot_digest_hex: &str,
    ) -> Result<ThinSnapshot, ThinIndexError> {
        self.thin_index.read().await.snapshot(snapshot_digest_hex)
    }

    /// Pin one signed thin snapshot for wrapper construction.
    pub async fn pin_thin_snapshot(
        &self,
        snapshot_digest_hex: &str,
    ) -> Result<ThinSnapshotPin, ThinIndexError> {
        let now_ms = self.now_ms();
        let store = self.thin_index.read().await.clone();
        self.thin_cache
            .write()
            .await
            .pin_snapshot(&store, snapshot_digest_hex, now_ms)
    }

    /// Publish and immediately pin one refreshed thin snapshot.
    pub async fn refresh_thin_snapshot(
        &self,
        snapshot: ThinSnapshot,
    ) -> Result<ThinSnapshotPin, ThinIndexError> {
        let now_ms = self.now_ms();
        let pin = self
            .thin_index
            .write()
            .await
            .refresh_snapshot(snapshot, now_ms)?;
        self.thin_cache.write().await.remember_pin(pin.clone());
        Ok(pin)
    }

    /// Resolve one thin wrapper back into canonical package bytes.
    pub async fn resolve_thin_tx_package(
        &self,
        thin: &ThinWalletTxPackage,
    ) -> Result<(Vec<u8>, TxPackage), ThinIndexError> {
        let now_ms = self.now_ms();
        self.thin_index.read().await.resolve_package(thin, now_ms)
    }

    /// Clear cached thin snapshot pins so the wallet defaults to thick mode.
    pub async fn clear_thin_snapshot_cache(&self) {
        self.thin_cache.write().await.clear();
    }

    /// Build one cached thin-or-thick transport payload for a canonical tx package.
    ///
    /// Thin references are expanded before runtime admission; cache uncertainty
    /// defaults to the canonical thick package payload.
    pub async fn build_cached_tx_transport(
        &self,
        tx_bytes: &[u8],
    ) -> Result<ThinTransportPayload, ThinIndexError> {
        let now_ms = self.now_ms();
        let store = self.thin_index.read().await.clone();
        self.thin_cache
            .write()
            .await
            .build_transport(&store, tx_bytes, now_ms)
    }

    fn wallet_tx_store(
        &self,
        wallet_id: &PersistWalletId,
    ) -> TxStorageImpl<tx_rpc_support::TimeProviderRef> {
        let history_path = self.service.wallet_history_jsonl_path(wallet_id);
        TxStorageImpl::new(
            &history_path,
            tx_rpc_support::TimeProviderRef(Arc::clone(&self.time_provider)),
        )
    }

    async fn load_wallet_tx_items(
        &self,
        wallet_id: &PersistWalletId,
    ) -> RpcResult<Vec<PersistTxInfo>> {
        if self.tx_store.is_none() {
            let store = self.wallet_tx_store(wallet_id);
            let records = store.list().map_err(|error| {
                ErrorObjectOwned::owned(-32603, format!("TxStorage error: {error}"), None::<()>)
            })?;
            let history_rows = store.list_history_rows().map_err(|error| {
                ErrorObjectOwned::owned(-32603, format!("TxStorage error: {error}"), None::<()>)
            })?;
            let pending = self.pending_txs.read().await;

            return Ok(records
                .into_iter()
                .map(|record| {
                    let latest_kind =
                        tx_runtime_state::latest_tx_history_kind(&history_rows, &record.tx_hash);
                    let mut info = tx_runtime_state::tx_record_to_tx_info(
                        wallet_id.clone(),
                        record,
                        latest_kind,
                    );
                    tx_runtime_state::overlay_pending_timestamp(&pending, wallet_id, &mut info);
                    info
                })
                .collect());
        }

        tx_runtime_state::load_wallet_tx_items(self.tx_store.as_ref(), &self.pending_txs, wallet_id)
            .await
    }

    async fn load_wallet_pending_tx_items(
        &self,
        wallet_id: &PersistWalletId,
    ) -> RpcResult<Vec<PersistTxInfo>> {
        if self.tx_store.is_none() {
            let store = self.wallet_tx_store(wallet_id);
            let records = store
                .list_by_status(crate::persistence::TxStatus::Pending)
                .map_err(|error| {
                    ErrorObjectOwned::owned(-32603, format!("TxStorage error: {error}"), None::<()>)
                })?;
            let history_rows = store.list_history_rows().map_err(|error| {
                ErrorObjectOwned::owned(-32603, format!("TxStorage error: {error}"), None::<()>)
            })?;
            let pending = self.pending_txs.read().await;

            return Ok(records
                .into_iter()
                .map(|record| {
                    let latest_kind =
                        tx_runtime_state::latest_tx_history_kind(&history_rows, &record.tx_hash);
                    let mut info = tx_runtime_state::tx_record_to_tx_info(
                        wallet_id.clone(),
                        record,
                        latest_kind,
                    );
                    tx_runtime_state::overlay_pending_timestamp(&pending, wallet_id, &mut info);
                    info
                })
                .collect());
        }

        tx_runtime_state::load_wallet_pending_tx_items(
            self.tx_store.as_ref(),
            &self.pending_txs,
            wallet_id,
        )
        .await
    }

    fn now_ms(&self) -> u64 {
        self.time_provider.compat_unix_timestamp_millis()
    }

    async fn tx_send_precheck(&self, wallet_id: &PersistWalletId) -> Result<(), (u32, u32, u32)> {
        tx_rpc_rate_limits::tx_send_precheck(&self.tx_send_rate_limits, self.now_ms(), wallet_id)
            .await
    }

    async fn tx_build_precheck(&self, wallet_id: &PersistWalletId) -> Result<(), (u32, u32, u32)> {
        tx_rpc_rate_limits::tx_build_precheck(&self.tx_build_rate_limits, self.now_ms(), wallet_id)
            .await
    }

    fn parse_asset_id_hex(asset_id: Option<String>) -> Result<[u8; 32], ErrorObjectOwned> {
        tx_rpc_support::parse_asset_id_hex(asset_id)
    }

    async fn reject_non_asset_cash_id(
        &self,
        wallet_id: &PersistWalletId,
        stable_key: [u8; 32],
    ) -> RpcResult<()> {
        let object = match self
            .service
            .lookup_non_asset_owned_object(wallet_id, stable_key)
            .await
        {
            Ok(object) => object,
            Err(
                WalletError::SessionExpired
                | WalletError::SessionInvalid
                | WalletError::Locked
                | WalletError::NotFound(_),
            ) => None,
            Err(error) => return Err(map_wallet_error_to_rpc(error)),
        };

        let Some(object) = object else {
            return Ok(());
        };

        let family = match object.payload {
            crate::db::OwnedObjectPayload::Asset(_) => "asset",
            crate::db::OwnedObjectPayload::Voucher(_) => "voucher",
            crate::db::OwnedObjectPayload::Right(_) => "right",
        };

        Err(ErrorObjectOwned::owned(
            -32602,
            format!(
                "wallet.tx cash build/send accepts assets only; id belongs to {family} inventory"
            ),
            None::<()>,
        ))
    }

    async fn validate_policy(
        &self,
        wallet_id: &PersistWalletId,
        asset_id: [u8; 32],
        recipient: &str,
        amount: u64,
    ) -> RpcResult<()> {
        tx_rpc_support::validate_policy(
            &self.service,
            &self.time_provider,
            wallet_id,
            asset_id,
            recipient,
            amount,
        )
        .await
    }

    async fn idempotency_get(
        &self,
        wallet_id: &PersistWalletId,
        key: &IdempotencyKey,
    ) -> Option<RuntimeSendTxResponse> {
        tx_rpc_idempotency::idempotency_get(&self.idempotency_cache, self.now_ms(), wallet_id, key)
            .await
    }

    async fn idempotency_put(
        &self,
        wallet_id: &PersistWalletId,
        key: IdempotencyKey,
        resp: &RuntimeSendTxResponse,
    ) {
        tx_rpc_idempotency::idempotency_put(
            &self.idempotency_cache,
            self.now_ms(),
            wallet_id,
            key,
            resp,
        )
        .await;
    }

    async fn verify_session(&self, session: &SessionToken) -> RpcResult<()> {
        tx_rpc_support::verify_session(&self.service, session).await
    }

    fn thin_error_code(error: &ThinIndexError) -> RuntimeTxErrorCode {
        match error {
            ThinIndexError::UnsupportedThinVersion(_)
            | ThinIndexError::UnsupportedSnapshotVersion(_) => {
                RuntimeTxErrorCode::UnsupportedPackageVersion
            }
            ThinIndexError::InvalidHex { .. }
            | ThinIndexError::InvalidSnapshotDigest
            | ThinIndexError::InvalidMetadataHash
            | ThinIndexError::PackageDigestMismatch { .. } => RuntimeTxErrorCode::InvalidDigest,
            ThinIndexError::InvalidSnapshotSignature | ThinIndexError::InvalidSnapshotShape(_) => {
                RuntimeTxErrorCode::ThinSnapshotInvalid
            }
            ThinIndexError::SnapshotExpired { .. }
            | ThinIndexError::SnapshotGenerationMismatch { .. } => {
                RuntimeTxErrorCode::ThinSnapshotStale
            }
            ThinIndexError::SnapshotMissing(_) | ThinIndexError::EntryMissing(_) => {
                RuntimeTxErrorCode::ThinSnapshotMissing
            }
            ThinIndexError::SnapshotConflict { .. }
            | ThinIndexError::EntryConflict(_)
            | ThinIndexError::SnapshotContextMismatch { .. }
            | ThinIndexError::InputRefMismatch => RuntimeTxErrorCode::ThinSnapshotConflict,
            ThinIndexError::PackageKindMismatch { .. }
            | ThinIndexError::PackageTypeMismatch { .. }
            | ThinIndexError::PackageVerificationFailed(_) => RuntimeTxErrorCode::InvalidPackage,
            ThinIndexError::PackageChainMismatch { .. } => RuntimeTxErrorCode::WrongChain,
            ThinIndexError::PackageRootMismatch { .. } => {
                RuntimeTxErrorCode::InvalidPublicSpendProof
            }
        }
    }

    fn thin_error_response(error: ThinIndexError) -> ErrorObjectOwned {
        crate::rpc::error_mapping::runtime_tx_error_response(
            -32602,
            format!("Invalid thin tx package: {error}"),
            vec![Self::thin_error_code(&error)],
            Some(RuntimeTxLifecycle::Failed),
        )
    }

    /// Expand one thick or thin transport payload before runtime admission.
    async fn parse_tx_pkg(&self, tx_data: &str) -> RpcResult<(Vec<u8>, TxPackage)> {
        let thick_error = match tx_rpc_support::parse_tx_pkg(tx_data) {
            Ok(parsed) => return Ok(parsed),
            Err(error) => error,
        };

        let thin = match JsonCodec.deserialize::<ThinWalletTxPackage>(tx_data.as_bytes()) {
            Ok(thin) => thin,
            Err(_) => return Err(thick_error),
        };

        self.resolve_thin_tx_package(&thin)
            .await
            .map_err(Self::thin_error_response)
    }

    async fn scan_pkg_outputs(
        &self,
        wallet_id: &PersistWalletId,
        pkg: &TxPackage,
    ) -> RpcResult<Vec<RuntimeVerifyTxPkgOut>> {
        tx_rpc_support::scan_pkg_outputs(&self.service, wallet_id, pkg).await
    }

    #[cfg(test)]
    fn build_owned_out(
        output: &crate::tx::TxOutputWire,
        scanner: &crate::receiver::StealthOutputScanner,
    ) -> RpcResult<Option<RuntimeVerifyTxPkgOut>> {
        tx_rpc_support::build_owned_out(output, scanner)
    }
}

fn is_import_ready(status: &str) -> bool {
    tx_rpc_support::is_import_ready(status)
}

#[cfg(all(test, not(target_arch = "wasm32")))]
#[path = "test_tx_impl.rs"]
mod tests;
