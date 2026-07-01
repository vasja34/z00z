use super::{
    create_dir_all, read_dir, recv_claim_asset, recv_range_start, rename_file,
    resolve_wallet_output_dir, Arc, Asset, AssetId, BackupCreateRateLimitState, BackupExporterImpl,
    BackupImporter, BackupImporterImpl, ObjectInventoryStore, Path, PathBuf, PaymentRequest,
    PersistBackupInfo, PersistBackupSettings, PersistWalletId, PinEntry, PinnedReceiverCards,
    RateLimitPrecheck, ReceiveNext, ReceiveReject, ReceiveReport, ReceiveStatus, ReceiverCard,
    ReceiverKeys, ReceiverSecret, RuntimeAssetListFilter, RuntimeAssetMetadataResponse,
    RuntimeBackupSettingsResponse, RuntimeCompactStorageParams, RuntimeCreateBackupResponse,
    RuntimeExportStorageParams, RuntimeGetStorageStatsParams, RuntimeListAssetsResponse,
    RuntimeListBackupsResponse, RuntimeMergeAssetsResponse, RuntimeReceiveAssetResponse,
    RuntimeRestoreBackupResponse, RuntimeSendAssetResponse, RuntimeShowSeedPhraseResponse,
    RuntimeSplitAssetResponse, RuntimeStakeAssetsResponse, RuntimeSwapAssetsResponse,
    RuntimeUnstakeAssetsResponse, SafePassword, ScanChunk, ScanRangeOut, ScanResult,
    ScanStatePayload, SessionToken, StealthKeyError, StealthMetaPayload, StealthOutputScanner,
    SystemRngProvider, SystemTimeProvider, TofuPinRecord, TofuPinsPayload, TrustLevel,
    VerifyResult, WalletEntropy, WalletEntropyFromRngProvider, WalletError, WalletIdentity,
    WalletResult, WalletService, WalletServiceReachability,
};
use crate::db::redb_store::OwnedAssetSource;
use crate::db::WalletAssetStore;
use crate::rpc::types::common::PersistTxId;

include!("wallet_actions_hardening.rs");

include!("wallet_actions_receiver.rs");

include!("wallet_actions_tofu.rs");

include!("wallet_actions_reachability.rs");

// Canonical receive wiring stays in wallet_actions_receive.rs.
// wallet_actions_runtime_inactive.rs remains intentionally excluded unless a
// future phase explicitly wires it in.
include!("wallet_actions_receive.rs");

include!("wallet_actions_backup.rs");

include!("wallet_actions_rpc.rs");
