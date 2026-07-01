//! Stub helpers for Phase 1 development.
//!
//! This module provides a single, explicit way to construct placeholder
//! values for stub-first development without relying on `Default`.

use crate::rpc::types::{
    asset::{
        RuntimeAddAssetResponse, RuntimeAssetBalanceResponse, RuntimeAssetDetailsResponse,
        RuntimeAssetMetadataResponse, RuntimeImportAssetResponse, RuntimeMergeAssetsResponse,
        RuntimeReceiveAssetResponse, RuntimeSendAssetResponse, RuntimeSplitAssetResponse,
        RuntimeStakeAssetsResponse, RuntimeSwapAssetsResponse, RuntimeUnstakeAssetsResponse,
    },
    backup::{
        PersistBackupInfo, PersistBackupSettings, RuntimeBackupSettingsResponse,
        RuntimeCreateBackupResponse, RuntimeListBackupsResponse, RuntimeRestoreBackupResponse,
    },
    common::{
        PersistTxId, PersistWalletId, RuntimeAssetAmount, RuntimeAssetRef, RuntimeOperationStatus,
        RuntimeOperationStatusWithTx, RuntimeOperationStatusWithWallet,
    },
    network::{RuntimeChainSettings, RuntimeChainSettingsResponse, RuntimeSwitchChainResponse},
    tx::{
        PersistTxInfo, RuntimeBuildTxResponse, RuntimeEstimateFeeResponse, RuntimeExportTxResponse,
        RuntimeListPendingTxResponse, RuntimeSendTxResponse, RuntimeTxDetailsResponse,
        RuntimeTxHistoryResponse, TxStatus,
    },
    wallet::{
        PersistWalletInfo, PersistWalletSettings, RuntimeCreateWalletResponse,
        RuntimeDeleteWalletResponse, RuntimeExportPublicKeyResponse, RuntimeExportWalletResponse,
        RuntimeImportWalletResponse, RuntimeKeyDeriveResponse, RuntimeShowSeedPhraseResponse,
        SessionToken,
    },
};

use crate::ChainType;
use z00z_core::assets::registry::AssetId;
use z00z_core::assets::{AssetClass, AssetDefinition, AssetWire};

/// Explicit stub defaults for stub-first development.
///
/// Prefer this over `Default` for RPC/service placeholder values so it is
/// always obvious when code is still in the stub phase.
pub trait StubDefault {
    /// Construct a deterministic placeholder value.
    fn stub_default() -> Self;
}

include!("stub_defaults_wallet.rs");
include!("stub_defaults_tx.rs");
include!("stub_defaults_backup.rs");
include!("stub_defaults_asset.rs");
