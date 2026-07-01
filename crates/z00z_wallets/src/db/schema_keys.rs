//! RedB wallet persistence schema keys.

use serde::{Deserialize, Serialize};

// meta table keys
/// Wallet identifier stored in the meta table.
pub const META_WALLET_ID: &str = "wallet.id";
/// Wallet schema version stored in the meta table.
pub const META_SCHEMA_VERSION: &str = "wallet.schema_version";
/// Wallet KDF parameters record key.
pub const META_WALLET_KDF: &str = "wallet.kdf";
/// Wallet initialization flag key.
pub const META_WALLET_INITIALIZED: &str = "wallet.initialized";
/// Wallet creation timestamp key.
pub const META_WALLET_CREATED_AT: &str = "wallet.created_at";
/// Wallet last update timestamp key.
pub const META_WALLET_UPDATED_AT: &str = "wallet.updated_at";
/// Wallet chain identifier key.
pub const META_WALLET_CHAIN: &str = "wallet.chain";
/// Wallet network identifier key.
pub const META_WALLET_NETWORK: &str = "wallet.network";
/// Wallet save sequence key.
pub const META_WALLET_SAVE_SEQ: &str = "wallet.save_seq";
/// Durable master-key rotation in-progress marker.
pub const META_ROTATION_IN_PROGRESS: &str = "wallet.rotation_in_progress";

// Migration markers
/// Index key encoding format version marker.
pub const META_INDEX_FORMAT_VERSION: &str = "wallet.index_format_version";

/// Secret AAD format version marker.
pub const META_AAD_SECRET_VERSION: &str = "wallet.aad_secret_version";

/// HKDF info scheme version marker.
///
/// v1: older RedB HKDF info hash with empty input.
/// v2: RedB HKDF info hash with explicit non-empty context bytes.
pub const META_HKDF_INFO_VERSION: &str = "wallet.hkdf_info_version";

// Integrity record (Task 3.5)
/// Wallet integrity record key (v1).
pub const META_WALLET_INTEGRITY: &str = "wallet.integrity.v1";

// Object pointers
/// Wallet profile object id pointer key.
pub const META_WALLET_PROFILE_OBJECT_ID: &str = "wallet.profile_object_id";
/// Derivation state object id pointer key.
pub const META_DERIVATION_STATE_OBJECT_ID: &str = "wallet.derivation_state_object_id";
/// Scan state object id pointer key.
pub const META_SCAN_STATE_OBJECT_ID: &str = "wallet.scan_state_object_id";
/// App object id pointer key.
pub const META_APP_OBJECT_ID: &str = "wallet.app_object_id";
/// Chain object id pointer key.
pub const META_CHAIN_OBJECT_ID: &str = "wallet.chain_object_id";
/// Keys object id pointer key.
pub const META_KEYS_OBJECT_ID: &str = "wallet.keys_object_id";
/// Stealth metadata object id pointer key.
pub const META_STEALTH_META_OBJECT_ID: &str = "wallet.stealth_meta_object_id";
/// TOFU pins object id pointer key.
pub const META_TOFU_PINS_OBJECT_ID: &str = "wallet.tofu_pins_object_id";

// secrets table keys
/// Encrypted master key record key.
pub const SECRETS_MASTER_KEY: &str = "master_key";
/// Main seed secret record key.
pub const SECRETS_SEED_MAIN: &str = "seed_main";

// Policy marker secrets
/// Timestamp (unix millis) when seed was revealed (policy marker).
pub const SECRETS_SEED_MAIN_REVEALED_AT: &str = "seed_main.revealed_at";

/// Index table enumeration for wallet database indexes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[repr(u8)]
pub enum IndexTable {
    /// Accounts indexed by label.
    AccountByLabel = 1,
    /// Receivers indexed by kind.
    ReceiverByKind = 2,
    /// Asset definitions indexed by symbol.
    AssetDefBySymbol = 3,
    /// Asset outputs indexed by definition.
    AssetOutByDef = 4,
    /// Asset outputs indexed by spent flag.
    AssetOutBySpentFlag = 5,
    /// Tracked assets indexed by spent flag.
    TrackedAssetBySpentFlag = 6,
    /// Transactions indexed by status.
    TxByStatus = 7,
    /// Transactions indexed by time.
    TxByTime = 8,
    /// Pending items indexed by status and expiry.
    PendingByStatusExpiry = 9,
    /// Receipts indexed by transaction hash.
    ReceiptByTxHash = 10,
    /// Wallet records indexed by wallet id.
    WalletByWalletId = 11,
    /// Owned assets indexed by canonical asset id.
    OwnedAssetById = 12,
    /// Owned assets indexed by definition id plus status.
    OwnedAssetByDefStatus = 13,
    /// Owned assets indexed by wallet-local status.
    OwnedAssetByStatus = 14,
    /// Owned assets indexed by pending or confirmed tx id.
    OwnedAssetByTx = 15,
    /// Owned assets indexed by scan position reference.
    OwnedAssetByScan = 16,
    /// Owned non-asset objects indexed by family.
    OwnedObjectByFamily = 17,
    /// Owned non-asset objects indexed by family plus lifecycle status.
    OwnedObjectByStatus = 18,
    /// Owned non-asset objects indexed by family plus policy availability.
    OwnedObjectByPolicy = 19,
    /// Owned non-asset objects indexed by family plus holder commitment.
    OwnedObjectByHolder = 20,
    /// Owned vouchers indexed by canonical terminal id.
    OwnedVoucherById = 21,
    /// Owned rights indexed by canonical terminal id.
    OwnedRightById = 22,
}

impl IndexTable {
    /// Canonical RedB table name for this index.
    pub const fn store_name(self) -> &'static str {
        match self {
            Self::AccountByLabel => "index_account_by_label",
            Self::ReceiverByKind => "index_receiver_by_kind",
            Self::AssetDefBySymbol => "index_asset_def_by_symbol",
            Self::AssetOutByDef => "index_asset_out_by_def",
            Self::AssetOutBySpentFlag => "index_asset_out_by_spentflag",
            Self::TrackedAssetBySpentFlag => "index_tracked_asset_by_spentflag",
            Self::TxByStatus => "index_tx_by_status",
            Self::TxByTime => "index_tx_by_time",
            Self::PendingByStatusExpiry => "index_pending_by_status_expiry",
            Self::ReceiptByTxHash => "index_receipt_by_txhash",
            Self::WalletByWalletId => "index_wallet_by_wallet_id",
            Self::OwnedAssetById => "index_owned_asset_by_id",
            Self::OwnedAssetByDefStatus => "index_owned_asset_by_def_status",
            Self::OwnedAssetByStatus => "index_owned_asset_by_status",
            Self::OwnedAssetByTx => "index_owned_asset_by_tx",
            Self::OwnedAssetByScan => "index_owned_asset_by_scan",
            Self::OwnedObjectByFamily => "index_owned_object_by_family",
            Self::OwnedObjectByStatus => "index_owned_object_by_status",
            Self::OwnedObjectByPolicy => "index_owned_object_by_policy",
            Self::OwnedObjectByHolder => "index_owned_object_by_holder",
            Self::OwnedVoucherById => "index_owned_voucher_by_id",
            Self::OwnedRightById => "index_owned_right_by_id",
        }
    }
}
