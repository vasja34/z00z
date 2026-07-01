//! Wiring helpers for routing method-string RPC calls into wallet RPC implementations.

#[cfg(not(target_arch = "wasm32"))]
use crate::rpc::dispatcher_handlers::{
    json_typed_handler_jsonrpsee_err, typed_handler_cap, typed_handler_jsonrpsee_err, NoArgs,
    WalletIdPasswordParams,
};
#[cfg(not(target_arch = "wasm32"))]
use crate::rpc::methods::storage_rpc::StorageRpc as _;
#[cfg(not(target_arch = "wasm32"))]
use crate::rpc::methods::{
    AppRpcImpl, AssetRpcImpl, AssetRpcServer, BackupRpcImpl, BackupRpcServer, ChainRpcImpl,
    ChainScanRpcImpl, KeyRpcImpl, KeyRpcServer, NetworkRpcImpl, ObjectRpcServer, StorageRpcImpl,
    TxRpcImpl, TxRpcServer, WalletRpcImpl, WalletRpcServer,
};
#[cfg(not(target_arch = "wasm32"))]
use crate::rpc::types::{
    asset::RuntimeAssetListFilter,
    backup::PersistBackupSettings,
    common::PersistWalletId,
    key::{RuntimePaymentRequestMetaInput, RuntimeReceiverFilter},
    object::{RuntimeObjectListFilter, RuntimeObjectPackageRequest},
    security::IdempotencyKey,
    storage::{
        RuntimeCompactStorageParams, RuntimeExportStorageParams, RuntimeGetStorageStatsParams,
    },
    tx::{PersistTxId, RuntimePaginationParams, RuntimeTxHistoryFilter, RuntimeTxHistorySort},
    wallet::{SessionToken, WalletLifecycleEvent},
};
#[cfg(not(target_arch = "wasm32"))]
use crate::WalletError;
#[cfg(not(target_arch = "wasm32"))]
use serde::Deserialize;
#[cfg(not(target_arch = "wasm32"))]
use std::sync::Arc;
#[cfg(not(target_arch = "wasm32"))]
use z00z_core::assets::registry::AssetId;
#[cfg(not(target_arch = "wasm32"))]
use z00z_networks_rpc::RpcDispatcher;

#[cfg(not(target_arch = "wasm32"))]
include!("wallet_dispatcher_routes.rs");

#[cfg(not(target_arch = "wasm32"))]
#[derive(Debug, Deserialize)]
struct BackupCreateParams {
    password: String,
    destination: Option<String>,
}

#[cfg(not(target_arch = "wasm32"))]
#[derive(Debug, Deserialize)]
struct WalletLifecycleParams {
    event: WalletLifecycleEvent,
}

#[cfg(not(target_arch = "wasm32"))]
#[derive(Debug, Deserialize)]
struct WalletShowSeedPhraseParams {
    password: String,
    confirmation: String,
}

#[cfg(not(target_arch = "wasm32"))]
#[derive(Debug, Deserialize)]
struct BackupListParams {
    cursor: Option<String>,
    limit: Option<u32>,
}

#[cfg(not(target_arch = "wasm32"))]
#[derive(Debug, Deserialize)]
struct BackupRestoreParams {
    backup_path: String,
    password: String,
    wallet_name: Option<String>,
}

#[cfg(not(target_arch = "wasm32"))]
#[derive(Debug, Deserialize)]
struct BackupConfigureParams {
    settings: Option<PersistBackupSettings>,
}

#[cfg(not(target_arch = "wasm32"))]
#[derive(Debug, Deserialize)]
struct KeyDeriveParams {
    path: String,
}

#[cfg(not(target_arch = "wasm32"))]
#[derive(Debug, Deserialize)]
struct KeyCardParams {}

#[cfg(not(target_arch = "wasm32"))]
#[derive(Debug, Deserialize)]
struct KeyCreatePaymentRequestParams {
    amount: Option<u64>,
    expiry_secs: u64,
    metadata: Option<RuntimePaymentRequestMetaInput>,
}

#[cfg(not(target_arch = "wasm32"))]
#[derive(Debug, Deserialize)]
struct KeyValidatePaymentRequestParams {
    request_compact: String,
}

#[cfg(not(target_arch = "wasm32"))]
#[derive(Debug, Deserialize)]
struct KeyExportPublicParams {
    account: u32,
    password: String,
}

#[cfg(not(target_arch = "wasm32"))]
#[derive(Debug, Deserialize)]
struct KeyRotateParams {
    password: String,
    confirmation: String,
}

#[cfg(not(target_arch = "wasm32"))]
#[derive(Debug, Deserialize)]
struct KeyListReceiversParams {
    limit: Option<usize>,
    cursor: Option<String>,
    filter: Option<RuntimeReceiverFilter>,
}

#[cfg(not(target_arch = "wasm32"))]
#[derive(Debug, Deserialize)]
struct KeyValidateReceiverCardParams {
    card_compact: String,
}

#[cfg(not(target_arch = "wasm32"))]
#[derive(Debug, Deserialize)]
struct KeyLabelReceiverParams {
    receiver_id: String,
    label: String,
}

#[cfg(not(target_arch = "wasm32"))]
#[derive(Debug, Deserialize)]
struct AssetListParams {
    wallet_id: PersistWalletId,
    limit: Option<usize>,
    cursor: Option<String>,
    filter: Option<RuntimeAssetListFilter>,
}

#[cfg(not(target_arch = "wasm32"))]
#[derive(Debug, Deserialize)]
struct AssetAddParams {
    session: SessionToken,
    asset_data: String,
}

#[cfg(not(target_arch = "wasm32"))]
#[derive(Debug, Deserialize)]
struct AssetWalletAssetIdParams {
    wallet_id: PersistWalletId,
    asset_id: AssetId,
}

#[cfg(not(target_arch = "wasm32"))]
#[derive(Debug, Deserialize)]
struct AssetImportParams {
    session: SessionToken,
    asset_data: String,
}

#[cfg(not(target_arch = "wasm32"))]
#[derive(Debug, Deserialize)]
struct AssetMergeParams {
    session: SessionToken,
    asset_ids: Vec<AssetId>,
}

#[cfg(not(target_arch = "wasm32"))]
#[derive(Debug, Deserialize)]
struct AssetSessionAssetIdParams {
    session: SessionToken,
    asset_id: AssetId,
}

#[cfg(not(target_arch = "wasm32"))]
#[derive(Debug, Deserialize)]
struct AssetMetadataParams {
    asset_id: AssetId,
}

#[cfg(not(target_arch = "wasm32"))]
#[derive(Debug, Deserialize)]
struct AssetSendParams {
    session: SessionToken,
    asset_id: AssetId,
    recipient: String,
    amount: u64,
}

#[cfg(not(target_arch = "wasm32"))]
#[derive(Debug, Deserialize)]
struct AssetSplitParams {
    session: SessionToken,
    asset_id: AssetId,
    amounts: Vec<u64>,
}

#[cfg(not(target_arch = "wasm32"))]
#[derive(Debug, Deserialize)]
struct AssetStakeParams {
    session: SessionToken,
    asset_id: AssetId,
    amount: u64,
}

#[cfg(not(target_arch = "wasm32"))]
#[derive(Debug, Deserialize)]
struct AssetSwapParams {
    session: SessionToken,
    from_asset_id: AssetId,
    to_asset_id: AssetId,
    amount: u64,
}

#[cfg(not(target_arch = "wasm32"))]
#[derive(Debug, Deserialize)]
struct AssetUnstakeParams {
    session: SessionToken,
    stake_id: String,
}

#[cfg(not(target_arch = "wasm32"))]
#[derive(Debug, Deserialize)]
struct ObjectListParams {
    wallet_id: PersistWalletId,
    limit: Option<usize>,
    cursor: Option<String>,
    filter: Option<RuntimeObjectListFilter>,
}

#[cfg(not(target_arch = "wasm32"))]
#[derive(Debug, Deserialize)]
struct ObjectVoucherListParams {
    wallet_id: PersistWalletId,
    limit: Option<usize>,
    cursor: Option<String>,
    status: Option<crate::db::OwnedVoucherStatus>,
}

#[cfg(not(target_arch = "wasm32"))]
#[derive(Debug, Deserialize)]
struct ObjectRightListParams {
    wallet_id: PersistWalletId,
    limit: Option<usize>,
    cursor: Option<String>,
    status: Option<crate::db::OwnedRightStatus>,
}

#[cfg(not(target_arch = "wasm32"))]
#[derive(Debug, Deserialize)]
struct ObjectPackageParams {
    session: SessionToken,
    request: RuntimeObjectPackageRequest,
}

#[cfg(not(target_arch = "wasm32"))]
#[derive(Debug, Deserialize)]
struct TxSendParams {
    session: SessionToken,
    recipient: String,
    amount: u64,
    asset_id: Option<String>,
    memo: Option<String>,
    idempotency_key: Option<IdempotencyKey>,
    timestamp: Option<u64>,
}

#[cfg(not(target_arch = "wasm32"))]
#[derive(Debug, Deserialize)]
struct TxBroadcastParams {
    session: SessionToken,
    tx_data: String,
}

#[cfg(not(target_arch = "wasm32"))]
#[derive(Debug, Deserialize)]
struct TxBuildParams {
    session: SessionToken,
    recipient: String,
    amount: u64,
    asset_id: Option<String>,
}

#[cfg(not(target_arch = "wasm32"))]
#[derive(Debug, Deserialize)]
struct TxVerifyPkgParams {
    session: SessionToken,
    tx_data: String,
}

#[cfg(not(target_arch = "wasm32"))]
#[derive(Debug, Deserialize)]
struct TxWalletTxIdParams {
    session: SessionToken,
    tx_id: PersistTxId,
}

#[cfg(not(target_arch = "wasm32"))]
#[derive(Debug, Deserialize)]
struct TxEstimateFeeParams {
    session: SessionToken,
    recipient: String,
    amount: u64,
    asset_id: Option<String>,
}

#[cfg(not(target_arch = "wasm32"))]
#[derive(Debug, Deserialize)]
struct TxGetHistoryParams {
    session: SessionToken,
    pagination: RuntimePaginationParams,
    filter: Option<RuntimeTxHistoryFilter>,
    sort: Option<RuntimeTxHistorySort>,
}

#[cfg(not(target_arch = "wasm32"))]
#[derive(Debug, Deserialize)]
struct TxListPendingParams {
    session: SessionToken,
    pagination: RuntimePaginationParams,
}

#[cfg(not(target_arch = "wasm32"))]
pub(super) struct RpcModules {
    pub(super) app_rpc: Arc<AppRpcImpl>,
    pub(super) wallet_rpc: Arc<WalletRpcImpl>,
    pub(super) asset_rpc: Arc<AssetRpcImpl>,
    pub(super) tx_rpc: Arc<TxRpcImpl>,
    pub(super) backup_rpc: Arc<BackupRpcImpl>,
    pub(super) key_rpc: Arc<KeyRpcImpl>,
    pub(super) chain_rpc: Arc<ChainRpcImpl>,
    pub(super) network_rpc: Arc<NetworkRpcImpl>,
    pub(super) scan_rpc: Arc<ChainScanRpcImpl>,
    pub(super) storage_rpc: Arc<StorageRpcImpl>,
}

/// Registers non-`wallet.*` RPC methods implemented by this crate into a generic [`RpcDispatcher`].
///
/// This is the complete wallet RPC surface (64 methods) used by LocalRpc-style clients.
#[cfg(not(target_arch = "wasm32"))]
pub(super) fn register_all_wallet_rpc_methods(
    dispatcher: &RpcDispatcher,
    modules: RpcModules,
) -> Result<(), WalletError> {
    let RpcModules {
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
    } = modules;

    init_rpc_crypto()?;

    crate::rpc::app_dispatcher_wiring::register_app_methods(dispatcher, app_rpc);
    register_wallet_methods(dispatcher, wallet_rpc);
    register_asset_methods(dispatcher, Arc::clone(&asset_rpc));
    register_object_methods(dispatcher, Arc::clone(&asset_rpc));
    register_tx_methods(dispatcher, tx_rpc);
    register_backup_methods(dispatcher, backup_rpc);
    register_key_methods(dispatcher, key_rpc);
    crate::rpc::app_dispatcher_wiring::register_chain_methods(dispatcher, chain_rpc);
    crate::rpc::app_dispatcher_wiring::register_network_methods(dispatcher, network_rpc);
    crate::rpc::app_dispatcher_wiring::register_scan_methods(dispatcher, scan_rpc);
    register_storage_methods(dispatcher, storage_rpc);

    Ok(())
}

#[cfg(not(target_arch = "wasm32"))]
fn init_rpc_crypto() -> Result<(), WalletError> {
    init_rpc_crypto_configured(
        z00z_crypto::RANGE_PROOF_BITS,
        z00z_crypto::AGGREGATION_FACTOR,
    )
}

#[cfg(not(target_arch = "wasm32"))]
fn init_rpc_crypto_configured(bits: usize, aggregation_factor: usize) -> Result<(), WalletError> {
    use z00z_crypto::vendor::tari::ExtendedPedersenCommitmentFactory;
    use z00z_crypto::BulletproofsPlusService;

    z00z_crypto::initialize();

    BulletproofsPlusService::init(
        bits,
        aggregation_factor,
        ExtendedPedersenCommitmentFactory::default(),
    )
    .map_err(|e| WalletError::CryptoError(format!("BulletproofsPlusService init failed: {e}")))?;

    Ok(())
}

#[cfg(all(test, not(target_arch = "wasm32")))]
mod init_tests {
    use super::*;

    #[test]
    fn test_rpc_crypto_misconfig_error() {
        let res = init_rpc_crypto_configured(0, z00z_crypto::AGGREGATION_FACTOR);
        assert!(res.is_err());
    }
}
