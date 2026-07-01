//! Wallet service facade with explicit internal seam ownership.
//!
//! `WalletService` remains the stable shallow caller-visible service boundary for the
//! wallet crate, while provisional action, session, store, and reachability
//! lanes stay behind named internal modules instead of a flat `include!`
//! assembly surface.
//! Request-aware receive remains the preferred privacy lane when approved
//! requests are supplied to `recv_range(...)`; the request-bound inbox is now
//! a live wallet-local and off-consensus metadata helper that still feeds
//! approved requests back into the same canonical lane instead of becoming a
//! second receive authority.
//! Single-asset reachability helpers stay plain or card-bound noncanonical
//! and must not be narrated as equal privacy proofs.

use crate::backup::{BackupExporterImpl, BackupImporter, BackupImporterImpl};
use crate::db::{
    ObjectInventoryStore, ScanStatePayload, StealthMetaPayload, TofuPinRecord, TofuPinsPayload,
    WalletAssetStore, WalletIdentity, WalletProfilePayload, WltStore,
};
use crate::domains::hashing::compute_wallet_file_id;
use crate::domains::{AeadEnvelopeDomain, WalletPasswordVerifierDomain};
use crate::key::{
    Bip44Path, KeyManagerImpl, ReceiverKeys, ReceiverSecret, StealthKeyError, Z00ZKeyBranch,
};
use crate::receiver::{
    PaymentRequest, PinEntry, PinnedReceiverCards, ReceiveNext, ReceiveReject, ReceiveReport,
    ReceiveStatus, ReceiverCard, ReceiverManager, ReceiverManagerImpl, ScanChunk, ScanRangeOut,
    ScanResult, StealthOutputScanner, TrustLevel, VerifyResult,
};
use crate::rpc::types::asset::{
    RuntimeAssetListFilter, RuntimeAssetMetadataResponse, RuntimeImportAssetResponse,
    RuntimeListAssetsResponse, RuntimeMergeAssetsResponse, RuntimeReceiveAssetResponse,
    RuntimeSendAssetResponse, RuntimeSplitAssetResponse, RuntimeStakeAssetsResponse,
    RuntimeSwapAssetsResponse, RuntimeUnstakeAssetsResponse,
};
use crate::rpc::types::backup::{
    PersistBackupInfo, PersistBackupSettings, RuntimeBackupSettingsResponse,
    RuntimeCreateBackupResponse, RuntimeListBackupsResponse, RuntimeRestoreBackupResponse,
};
use crate::rpc::types::common::{
    PersistWalletId, RuntimeEncryptedResponse, RuntimeEncryptionMetadata,
};
use crate::rpc::types::security::PersistAuditLogEntry;
use crate::rpc::types::storage::{
    RuntimeCompactStorageParams, RuntimeExportStorageParams, RuntimeGetStorageStatsParams,
};
use crate::rpc::types::wallet::{
    PersistWalletDiscovery, PersistWalletInfo, PersistWalletSettings, RuntimeCreateWalletResponse,
    RuntimeDeleteWalletResponse, RuntimeExportWalletResponse, RuntimeImportWalletResponse,
    RuntimeLockWalletResponse, RuntimeShowSeedPhraseResponse, SessionToken, WalletLifecycleEvent,
    WalletSource,
};
use crate::services::wallet_runtime_config::resolve_wallet_output_dir;
use crate::wallet::persistence::{PasswordVerifierState, ReceiverDeriverState, WalletExportPack};
use crate::wallet::{
    AutoLockPolicy, ChainId as CoreChainId, WalletId as CoreWalletId, WalletState, Z00ZWallet,
};
use crate::{WalletError, WalletResult};
use base64::Engine as _;
use std::collections::{BTreeMap, BTreeSet};
use std::future::Future;
use std::path::{Path, PathBuf};
use std::pin::Pin;
use std::sync::Arc;
use std::time::Duration;
use subtle::ConstantTimeEq;
use tokio::sync::RwLock;
use tokio::task::JoinHandle;
use z00z_core::assets::registry::AssetId;
use z00z_core::assets::{decode_asset_pkg_json, payload_has_secret_field, AssetError};
use z00z_core::genesis::ChainType;
use z00z_core::Asset;
use z00z_crypto::expert::encoding::SafePassword;
use z00z_crypto::{aead, Hidden};
use z00z_utils::codec::{Codec, JsonCodec};
use z00z_utils::io::{create_dir_all, read_dir, rename_file};
use z00z_utils::rng::{RngCoreExt, SecureRngProvider, SystemRngProvider};
use z00z_utils::time::{SystemTimeProvider, TimeProvider};

#[cfg(not(target_arch = "wasm32"))]
use crate::services::wallet_session_manager::WalletSessionManager;

#[path = "wallet_service_core.rs"]
mod wallet_service_core;
#[path = "wallet_service_reachability.rs"]
mod wallet_service_reachability;
#[path = "wallet_service_state.rs"]
mod wallet_service_state;

pub use self::wallet_service_core::{ReceiverUsageOracle, Sleeper, WalletService};
pub use self::wallet_service_state::RateLimitPrecheck;
pub(crate) use self::wallet_session::{VerifiedSession, VerifiedSessionNoTouch};

use self::wallet_service_core::{TokioSleeper, WalletEntropy, WalletEntropyFromRngProvider};
use self::wallet_service_reachability::WalletServiceReachability;
use self::wallet_service_state::{
    BackupCreateRateLimitState, RateLimitWindowState, UnlockAttemptPrecheck, UnlockAttemptState,
    WalletPasswordVerifierState, WalletReceiverDeriverState,
};

#[cfg(test)]
#[path = "test_wallet_service.rs"]
mod test_wallet_service;
#[path = "wallet_actions.rs"]
mod wallet_actions;
#[path = "wallet_session.rs"]
mod wallet_session;
#[path = "wallet_store.rs"]
mod wallet_store;

use self::wallet_store::{recv_claim_asset, recv_range_start};
