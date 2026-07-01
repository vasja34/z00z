use super::{
    Deserialize, IndexKeyBytes, IndexTable, IndexValueBytes, MnemonicLanguage, PathBuf, Serialize,
    TableDefinition, WalletError, WalletResult,
};
use crate::persistence::tx::TxStatus;
use crate::rpc::types::{
    common::{PersistTxId, PersistWalletId},
    wallet::PersistWalletSettings,
};
use crate::wallet::persistence::{PasswordVerifierState, ReceiverDeriverState};
use crate::wallet::WalletState;
use z00z_core::{assets::AssetWire, vouchers::VoucherLifecycleV1, Asset};
use z00z_storage::settlement::{RightLeaf, TerminalId, VoucherLeaf};

#[derive(Debug, Clone)]
pub(crate) enum WltBacking {
    /// The on-disk `.wlt` is zstd-compressed. The database is opened from a tmpfs-backed
    /// uncompressed work file, and writes are flushed back to the zstd file atomically.
    ZstdTmpfs {
        original_path: PathBuf,
        work_path: PathBuf,
    },
}

pub(crate) const META_TABLE: TableDefinition<&str, &[u8]> = TableDefinition::new("meta");
pub(crate) const SECRETS_TABLE: TableDefinition<&str, &[u8]> = TableDefinition::new("secrets");
pub(crate) const OBJECTS_TABLE: TableDefinition<&[u8], &[u8]> = TableDefinition::new("objects");

#[cfg(any(test, feature = "wallet_debug_tools"))]
pub(crate) const INDEX_ACCOUNT_BY_LABEL_TABLE: TableDefinition<&[u8], &[u8]> =
    TableDefinition::new("index_account_by_label");
#[cfg(feature = "wallet_debug_tools")]
pub(crate) const INDEX_RECEIVER_BY_KIND_TABLE: TableDefinition<&[u8], &[u8]> =
    TableDefinition::new("index_receiver_by_kind");
#[cfg(feature = "wallet_debug_tools")]
pub(crate) const INDEX_ASSET_DEF_SYMBOL_TABLE: TableDefinition<&[u8], &[u8]> =
    TableDefinition::new("index_asset_def_by_symbol");
#[cfg(feature = "wallet_debug_tools")]
pub(crate) const INDEX_ASSET_DEF_TABLE: TableDefinition<&[u8], &[u8]> =
    TableDefinition::new("index_asset_out_by_def");
#[cfg(feature = "wallet_debug_tools")]
pub(crate) const INDEX_ASSET_SPENTFLAG_TABLE: TableDefinition<&[u8], &[u8]> =
    TableDefinition::new("index_asset_out_by_spentflag");
#[cfg(feature = "wallet_debug_tools")]
pub(crate) const INDEX_TRACKED_ASSET_SPENTFLAG_TABLE: TableDefinition<&[u8], &[u8]> =
    TableDefinition::new("index_tracked_asset_by_spentflag");
#[cfg(feature = "wallet_debug_tools")]
pub(crate) const INDEX_TX_BY_STATUS_TABLE: TableDefinition<&[u8], &[u8]> =
    TableDefinition::new("index_tx_by_status");
#[cfg(feature = "wallet_debug_tools")]
pub(crate) const INDEX_TX_BY_TIME_TABLE: TableDefinition<&[u8], &[u8]> =
    TableDefinition::new("index_tx_by_time");
#[cfg(feature = "wallet_debug_tools")]
pub(crate) const INDEX_PENDING_STATUS_EXPIRY_TABLE: TableDefinition<&[u8], &[u8]> =
    TableDefinition::new("index_pending_by_status_expiry");
#[cfg(feature = "wallet_debug_tools")]
pub(crate) const INDEX_RECEIPT_BY_TXHASH_TABLE: TableDefinition<&[u8], &[u8]> =
    TableDefinition::new("index_receipt_by_txhash");
#[cfg(feature = "wallet_debug_tools")]
pub(crate) const INDEX_WALLET_ID_TABLE: TableDefinition<&[u8], &[u8]> =
    TableDefinition::new("index_wallet_by_wallet_id");
#[cfg(feature = "wallet_debug_tools")]
pub(crate) const OWNED_ASSET_ID_TABLE: TableDefinition<&[u8], &[u8]> =
    TableDefinition::new("index_owned_asset_by_id");
#[cfg(feature = "wallet_debug_tools")]
pub(crate) const OWNED_DEF_STATUS_TABLE: TableDefinition<&[u8], &[u8]> =
    TableDefinition::new("index_owned_asset_by_def_status");
#[cfg(feature = "wallet_debug_tools")]
pub(crate) const OWNED_ASSET_STATUS_TABLE: TableDefinition<&[u8], &[u8]> =
    TableDefinition::new("index_owned_asset_by_status");
#[cfg(feature = "wallet_debug_tools")]
pub(crate) const OWNED_ASSET_TX_TABLE: TableDefinition<&[u8], &[u8]> =
    TableDefinition::new("index_owned_asset_by_tx");
#[cfg(feature = "wallet_debug_tools")]
pub(crate) const OWNED_ASSET_SCAN_TABLE: TableDefinition<&[u8], &[u8]> =
    TableDefinition::new("index_owned_asset_by_scan");
#[cfg(feature = "wallet_debug_tools")]
pub(crate) const OWNED_OBJECT_FAMILY_TABLE: TableDefinition<&[u8], &[u8]> =
    TableDefinition::new("index_owned_object_by_family");
#[cfg(feature = "wallet_debug_tools")]
pub(crate) const OWNED_OBJECT_STATUS_TABLE: TableDefinition<&[u8], &[u8]> =
    TableDefinition::new("index_owned_object_by_status");
#[cfg(feature = "wallet_debug_tools")]
pub(crate) const OWNED_OBJECT_POLICY_TABLE: TableDefinition<&[u8], &[u8]> =
    TableDefinition::new("index_owned_object_by_policy");
#[cfg(feature = "wallet_debug_tools")]
pub(crate) const OWNED_OBJECT_HOLDER_TABLE: TableDefinition<&[u8], &[u8]> =
    TableDefinition::new("index_owned_object_by_holder");
#[cfg(feature = "wallet_debug_tools")]
pub(crate) const OWNED_VOUCHER_ID_TABLE: TableDefinition<&[u8], &[u8]> =
    TableDefinition::new("index_owned_voucher_by_id");
#[cfg(feature = "wallet_debug_tools")]
pub(crate) const OWNED_RIGHT_ID_TABLE: TableDefinition<&[u8], &[u8]> =
    TableDefinition::new("index_owned_right_by_id");

pub(crate) const INDEX_MANIFEST_TABLE: TableDefinition<&[u8], &[u8]> =
    TableDefinition::new("index_manifest");

pub const PAYLOAD_VERSION_WALLET_ROOT: u16 = 1;
pub const PAYLOAD_VERSION_ACCOUNT: u16 = 1;
pub const PAYLOAD_VERSION_DERIVATION_STATE: u16 = 1;
pub const PAYLOAD_VERSION_SCAN_STATE: u16 = 1;
pub const PAYLOAD_VERSION_APP: u16 = 1;
pub const PAYLOAD_VERSION_CHAIN: u16 = 1;
pub const PAYLOAD_VERSION_KEYS: u16 = 1;
pub const PAYLOAD_VERSION_STEALTH_META: u16 = 1;
pub const PAYLOAD_VERSION_TOFU_PINS: u16 = 1;
pub const PAYLOAD_VERSION_WALLET_PROFILE: u16 = 1;
pub const PAYLOAD_VERSION_OWNED_ASSET: u16 = 1;
pub const PAYLOAD_VERSION_OWNED_VOUCHER: u16 = 1;
pub const PAYLOAD_VERSION_OWNED_RIGHT: u16 = 1;
pub const PAYLOAD_VERSION_WALLET_TX: u16 = 1;
pub const PAYLOAD_VERSION_WALLET_TX_EVENT: u16 = 1;
pub const PAYLOAD_VERSION_BACKUP_MANIFEST: u16 = 1;

#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum ObjectKindId {
    WalletRoot = 1,
    Account = 2,
    DerivationState = 7,
    ScanState = 8,
    App = 15,
    Chain = 16,
    Keys = 17,
    StealthMeta = 18,
    TofuPins = 19,
    WalletProfile = 20,
    OwnedAsset = 21,
    WalletTx = 22,
    WalletTxEvent = 23,
    BackupManifest = 24,
    OwnedVoucher = 25,
    OwnedRight = 26,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct IndexManifestEntry {
    pub(crate) table: IndexTable,
    pub(crate) key: Vec<u8>,
}

#[derive(Debug, Clone)]
pub struct IndexUpdate {
    pub(crate) table: IndexTable,
    pub(crate) semantic_key: Vec<u8>,
    pub(crate) value: IndexValueBytes,
}

impl IndexUpdate {
    #[cfg(test)]
    pub(crate) fn new(
        table: IndexTable,
        semantic_key: Vec<u8>,
        value: Vec<u8>,
    ) -> WalletResult<Self> {
        crate::db::index_codecs::validate_index_semantic_key(&semantic_key)?;
        let value = IndexValueBytes::new(value)?;
        Ok(Self {
            table,
            semantic_key,
            value,
        })
    }

    pub(crate) fn with_value_bytes(
        table: IndexTable,
        semantic_key: Vec<u8>,
        value: IndexValueBytes,
    ) -> WalletResult<Self> {
        crate::db::index_codecs::validate_index_semantic_key(&semantic_key)?;
        Ok(Self {
            table,
            semantic_key,
            value,
        })
    }
}

#[derive(Debug, Clone)]
pub(crate) struct ValidatedIndexUpdate {
    pub(crate) table: IndexTable,
    pub(crate) key: IndexKeyBytes,
    pub(crate) value: IndexValueBytes,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WalletRootPayload {
    pub version: u32,
    pub main_account_id: u128,
    pub created_at: u64,
    pub chain: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountPayload {
    pub account_id: u128,
    pub parent_wallet: u128,
    pub name: String,
    pub derivation_path: String,
    pub public_key: Vec<u8>,
    pub created_at: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DerivationStatePayload {
    pub next_account_index: u32,
    pub next_address_index: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
/// Wallet-native scan cursor substrate for receiver progress and resume.
pub struct ScanStatePayload {
    pub last_scanned_height: u64,
    pub last_scanned_hash: Vec<u8>,
}

impl ScanStatePayload {
    pub fn new(last_scanned_height: u64, last_scanned_hash: Vec<u8>) -> Self {
        Self {
            last_scanned_height,
            last_scanned_hash,
        }
    }

    pub fn advance(&mut self, last_scanned_height: u64, last_scanned_hash: Vec<u8>) {
        self.last_scanned_height = last_scanned_height;
        self.last_scanned_hash = last_scanned_hash;
    }

    pub fn is_origin(&self) -> bool {
        self.last_scanned_height == 0 && self.last_scanned_hash.is_empty()
    }

    pub fn matches_chunk(&self, height: u64, hash: &[u8]) -> bool {
        self.last_scanned_height == height && self.last_scanned_hash.as_slice() == hash
    }

    pub fn height(&self) -> u64 {
        self.last_scanned_height
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AppPlatform {
    Linux,
    Windows,
    Macos,
    Android,
    Ios,
    Other,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppPayload {
    pub app_id: String,
    pub app_name: String,
    pub app_version: String,
    pub platform: AppPlatform,
    pub instance_id: [u8; 16],
    pub created_at: u64,
    pub last_opened_at: Option<u64>,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChainPayload {
    pub chain: String,
    pub chain_id: Option<String>,
    pub genesis_hash: Option<Vec<u8>>,
    pub params: Option<Vec<u8>>,
    pub created_at: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeysPayload {
    pub keyset_id: u128,
    pub account_id: Option<u128>,
    pub signing_keys: Vec<KeyRefPayload>,
    pub created_at: u64,
    pub updated_at: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModeAuditEntry {
    pub from_mode: String,
    pub to_mode: String,
    pub changed_at: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StealthMetaPayload {
    pub view_key_version: u32,
    pub receiver_mode: String,
    pub stealth_activated_at: Option<u64>,
    #[serde(default)]
    pub mode_audit: Vec<ModeAuditEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TofuPinRecord {
    pub owner_handle: [u8; 32],
    pub view_pk: [u8; 32],
    pub identity_pk: [u8; 32],
    pub directory_id: Option<String>,
    pub first_seen: u64,
    pub trust_level: u8,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TofuPinsPayload {
    pub pins: Vec<TofuPinRecord>,
    pub updated_at: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WalletProfilePayload {
    pub version: u32,
    pub wallet_id: PersistWalletId,
    pub name: String,
    pub created_at: u64,
    pub updated_at: u64,
    pub password_verifier: PasswordVerifierState,
    pub receiver_deriver: ReceiverDeriverState,
    pub settings: PersistWalletSettings,
    pub seed_salt: Option<[u8; 16]>,
    pub state: WalletState,
    pub checksum: Option<[u8; 32]>,
}

impl WalletProfilePayload {
    pub const VERSION: u32 = 1;

    pub fn migrate_to_current(self) -> WalletResult<Self> {
        match self.version {
            Self::VERSION => {
                if self.seed_salt.is_none() {
                    return Err(WalletError::InvalidConfig(
                        "Wallet profile missing seed salt".to_string(),
                    ));
                }
                Ok(self)
            }
            version => Err(WalletError::UnsupportedVersion(version)),
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub fn new(
        wallet_id: PersistWalletId,
        name: String,
        created_at: u64,
        updated_at: u64,
        password_verifier: PasswordVerifierState,
        receiver_deriver: ReceiverDeriverState,
        settings: PersistWalletSettings,
        seed_salt: [u8; 16],
        state: WalletState,
    ) -> Self {
        Self {
            version: Self::VERSION,
            wallet_id,
            name,
            created_at,
            updated_at,
            password_verifier,
            receiver_deriver,
            settings,
            seed_salt: Some(seed_salt),
            state,
            checksum: None,
        }
    }

    pub fn compute_checksum(&self) -> [u8; 32] {
        use crate::domains::hashing::{canonicalize_bytes, SnapshotChecksumHasher};
        use z00z_utils::codec::{BincodeCodec, Codec};

        let codec = BincodeCodec;
        let mut hash_input = Vec::with_capacity(384);

        hash_input.extend_from_slice(&self.version.to_le_bytes());
        hash_input.extend_from_slice(self.wallet_id.0.as_bytes());

        let name_bytes = self.name.as_bytes();
        hash_input.extend_from_slice(&(name_bytes.len() as u32).to_le_bytes());
        hash_input.extend_from_slice(name_bytes);

        hash_input.extend_from_slice(&self.created_at.to_le_bytes());
        hash_input.extend_from_slice(&self.updated_at.to_le_bytes());
        hash_input.extend_from_slice(&self.password_verifier.salt);
        hash_input.extend_from_slice(&self.password_verifier.verifier);
        hash_input.extend_from_slice(&self.receiver_deriver.next_payment_index.to_le_bytes());
        hash_input.extend_from_slice(&self.receiver_deriver.next_change_index.to_le_bytes());

        if let Ok(settings_bytes) = codec.serialize(&self.settings) {
            hash_input.extend_from_slice(&settings_bytes);
        }

        match self.seed_salt {
            Some(seed_salt) => {
                hash_input.push(1);
                hash_input.extend_from_slice(&seed_salt);
            }
            None => hash_input.push(0),
        }

        if let Ok(state_bytes) = codec.serialize(&self.state) {
            hash_input.extend_from_slice(&state_bytes);
        }

        let hash = SnapshotChecksumHasher::new_with_label("wallet_profile")
            .chain(canonicalize_bytes(&hash_input))
            .finalize();

        let mut result = [0u8; 32];
        result.copy_from_slice(&hash.as_ref()[..32]);
        result
    }

    pub fn verify_checksum(&self) -> WalletResult<()> {
        match self.checksum {
            None => Err(WalletError::ChecksumMismatch {
                expected: hex::encode(self.compute_checksum()),
                actual: "missing checksum".to_string(),
            }),
            Some(stored_checksum) => {
                use subtle::ConstantTimeEq;

                let computed_checksum = self.compute_checksum();
                if stored_checksum
                    .as_ref()
                    .ct_eq(computed_checksum.as_ref())
                    .unwrap_u8()
                    != 0
                {
                    Ok(())
                } else {
                    Err(WalletError::ChecksumMismatch {
                        expected: hex::encode(stored_checksum),
                        actual: hex::encode(computed_checksum),
                    })
                }
            }
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub fn new_with_checksum(
        wallet_id: PersistWalletId,
        name: String,
        created_at: u64,
        updated_at: u64,
        password_verifier: PasswordVerifierState,
        receiver_deriver: ReceiverDeriverState,
        settings: PersistWalletSettings,
        seed_salt: [u8; 16],
        state: WalletState,
    ) -> Self {
        let mut profile = Self::new(
            wallet_id,
            name,
            created_at,
            updated_at,
            password_verifier,
            receiver_deriver,
            settings,
            seed_salt,
            state,
        );
        profile.checksum = Some(profile.compute_checksum());
        profile
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OwnedAssetStatus {
    Spendable,
    PendingSpend,
    Spent,
    PendingReceive,
    Quarantined,
    Archived,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OwnedAssetSource {
    Scan,
    Import,
    Change,
    Genesis,
    Restore,
    ManualClaim,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AssetSeenRef {
    pub height: Option<u64>,
    pub hash_or_root: Option<Vec<u8>>,
    pub local_time_ms: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ScanRef {
    pub start_height: u64,
    pub end_height: u64,
    pub cursor_hash: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReceiveRef {
    pub request_id: Option<String>,
    pub receiver_handle: Option<String>,
    pub import_tx_id: Option<PersistTxId>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ConfirmRef {
    pub checkpoint_id_hex: Option<String>,
    pub state_root_hex: Option<String>,
    pub evidence_id: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OwnedAssetPolicy {
    pub frozen: bool,
    pub manual_review: bool,
    pub quarantine_reason: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OwnedAssetPayload {
    pub version: u32,
    pub wallet_id: PersistWalletId,
    pub account_id: Option<u128>,
    pub asset_id: [u8; 32],
    pub asset_definition_id: [u8; 32],
    pub asset_wire: AssetWire,
    pub status: OwnedAssetStatus,
    pub source: OwnedAssetSource,
    pub first_seen: Option<AssetSeenRef>,
    pub last_updated_ms: u64,
    pub scan_ref: Option<ScanRef>,
    pub receive_ref: Option<ReceiveRef>,
    pub spend_ref: Option<PersistTxId>,
    pub confirmation_ref: Option<ConfirmRef>,
    pub labels: Vec<String>,
    pub policy: OwnedAssetPolicy,
    pub checksum: Option<[u8; 32]>,
}

impl OwnedAssetPayload {
    pub const VERSION: u32 = 1;

    pub fn migrate_to_current(self) -> WalletResult<Self> {
        match self.version {
            Self::VERSION => Ok(self),
            version => Err(WalletError::UnsupportedVersion(version)),
        }
    }

    pub fn compute_checksum(&self) -> [u8; 32] {
        use crate::domains::hashing::{canonicalize_bytes, SnapshotChecksumHasher};
        use z00z_utils::codec::{BincodeCodec, Codec};

        let mut canonical = self.clone();
        canonical.checksum = None;

        let payload_bytes = BincodeCodec.serialize(&canonical).unwrap_or_default();
        let hash = SnapshotChecksumHasher::new_with_label("owned_asset")
            .chain(canonicalize_bytes(&payload_bytes))
            .finalize();

        let mut out = [0u8; 32];
        out.copy_from_slice(&hash.as_ref()[..32]);
        out
    }

    pub fn verify_checksum(&self) -> WalletResult<()> {
        match self.checksum {
            None => Err(WalletError::ChecksumMismatch {
                expected: hex::encode(self.compute_checksum()),
                actual: "missing checksum".to_string(),
            }),
            Some(stored_checksum) => {
                use subtle::ConstantTimeEq;

                let computed_checksum = self.compute_checksum();
                if stored_checksum
                    .as_ref()
                    .ct_eq(computed_checksum.as_ref())
                    .unwrap_u8()
                    != 0
                {
                    Ok(())
                } else {
                    Err(WalletError::ChecksumMismatch {
                        expected: hex::encode(stored_checksum),
                        actual: hex::encode(computed_checksum),
                    })
                }
            }
        }
    }

    pub fn validate_invariants(&self) -> WalletResult<Asset> {
        let asset = self.asset_wire.clone().to_asset().map_err(|error| {
            WalletError::InvalidConfig(format!("owned asset wire decode failed: {error}"))
        })?;
        asset.validate().map_err(|error| {
            WalletError::InvalidConfig(format!("owned asset validation failed: {error}"))
        })?;

        if asset.asset_id() != self.asset_id {
            return Err(WalletError::InvalidConfig(
                "owned asset id drifted from asset wire".to_string(),
            ));
        }

        if asset.definition.id != self.asset_definition_id {
            return Err(WalletError::InvalidConfig(
                "owned asset definition id drifted from asset wire".to_string(),
            ));
        }

        match self.status {
            OwnedAssetStatus::PendingSpend | OwnedAssetStatus::Spent
                if self.spend_ref.is_none() =>
            {
                return Err(WalletError::InvalidConfig(
                    "owned asset spend status missing tx binding".to_string(),
                ));
            }
            OwnedAssetStatus::Spendable if self.spend_ref.is_some() => {
                return Err(WalletError::InvalidConfig(
                    "spendable owned asset cannot keep a live spend reservation".to_string(),
                ));
            }
            _ => {}
        }

        Ok(asset)
    }

    pub fn is_live_claimed_status(&self) -> bool {
        !matches!(
            self.status,
            OwnedAssetStatus::Spent | OwnedAssetStatus::Archived
        )
    }
}

pub type ObjectSeenRef = AssetSeenRef;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OwnedObjectFamily {
    Asset,
    Voucher,
    Right,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OwnedObjectSource {
    Scan,
    Import,
    Change,
    Genesis,
    Restore,
    ManualClaim,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WalletPolicyAvailability {
    Available,
    Unknown,
    Missing,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OwnedObjectPolicy {
    pub policy_id: Option<[u8; 32]>,
    pub availability: WalletPolicyAvailability,
    pub manual_review: bool,
    pub quarantine_reason: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OwnedVoucherStatus {
    Offered,
    PendingAccept,
    Accepted,
    Redeemable,
    PartiallyRedeemed,
    Redeemed,
    Rejected,
    Refunded,
    Expired,
    Quarantined,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OwnedRightStatus {
    Granted,
    Held,
    Delegated,
    Consumed,
    Revoked,
    Expired,
    Challenged,
    Quarantined,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WalletObjectStatus {
    Asset(OwnedAssetStatus),
    Voucher(OwnedVoucherStatus),
    Right(OwnedRightStatus),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OwnedVoucherPayload {
    pub version: u32,
    pub wallet_id: PersistWalletId,
    pub account_id: Option<u128>,
    pub terminal_id: TerminalId,
    pub voucher_leaf: VoucherLeaf,
    pub status: OwnedVoucherStatus,
    pub source: OwnedObjectSource,
    pub first_seen: Option<ObjectSeenRef>,
    pub last_updated_ms: u64,
    pub scan_ref: Option<ScanRef>,
    pub receive_ref: Option<ReceiveRef>,
    pub confirmation_ref: Option<ConfirmRef>,
    pub labels: Vec<String>,
    pub policy: OwnedObjectPolicy,
    pub holder_opening: Option<Vec<u8>>,
    pub beneficiary_opening: Option<Vec<u8>>,
    pub checksum: Option<[u8; 32]>,
}

impl OwnedVoucherPayload {
    pub const VERSION: u32 = 1;

    pub fn migrate_to_current(self) -> WalletResult<Self> {
        match self.version {
            Self::VERSION => Ok(self),
            version => Err(WalletError::UnsupportedVersion(version)),
        }
    }

    pub fn compute_checksum(&self) -> [u8; 32] {
        use crate::domains::hashing::{canonicalize_bytes, SnapshotChecksumHasher};
        use z00z_utils::codec::{BincodeCodec, Codec};

        let mut canonical = self.clone();
        canonical.checksum = None;

        let payload_bytes = BincodeCodec.serialize(&canonical).unwrap_or_default();
        let hash = SnapshotChecksumHasher::new_with_label("owned_voucher")
            .chain(canonicalize_bytes(&payload_bytes))
            .finalize();

        let mut out = [0u8; 32];
        out.copy_from_slice(&hash.as_ref()[..32]);
        out
    }

    pub fn verify_checksum(&self) -> WalletResult<()> {
        match self.checksum {
            None => Err(WalletError::ChecksumMismatch {
                expected: hex::encode(self.compute_checksum()),
                actual: "missing checksum".to_string(),
            }),
            Some(stored_checksum) => {
                use subtle::ConstantTimeEq;

                let computed_checksum = self.compute_checksum();
                if stored_checksum
                    .as_ref()
                    .ct_eq(computed_checksum.as_ref())
                    .unwrap_u8()
                    != 0
                {
                    Ok(())
                } else {
                    Err(WalletError::ChecksumMismatch {
                        expected: hex::encode(stored_checksum),
                        actual: hex::encode(computed_checksum),
                    })
                }
            }
        }
    }

    pub fn validate_invariants(&self) -> WalletResult<()> {
        self.voucher_leaf
            .validity
            .validate()
            .map_err(|error| WalletError::InvalidConfig(error.to_string()))?;

        if self.terminal_id != self.voucher_leaf.terminal_id {
            return Err(WalletError::InvalidConfig(
                "owned voucher terminal id drifted from voucher leaf".to_string(),
            ));
        }

        if self.voucher_leaf.remaining_value > self.voucher_leaf.face_value {
            return Err(WalletError::InvalidConfig(
                "owned voucher remaining value exceeds face value".to_string(),
            ));
        }

        match self.status {
            OwnedVoucherStatus::Offered | OwnedVoucherStatus::PendingAccept => {
                if self.voucher_leaf.lifecycle != VoucherLifecycleV1::PendingAcceptance {
                    return Err(WalletError::InvalidConfig(
                        "owned voucher pending status drifted from lifecycle".to_string(),
                    ));
                }
            }
            OwnedVoucherStatus::Accepted | OwnedVoucherStatus::Redeemable => {
                if self.voucher_leaf.lifecycle != VoucherLifecycleV1::Active {
                    return Err(WalletError::InvalidConfig(
                        "owned voucher active status drifted from lifecycle".to_string(),
                    ));
                }
            }
            OwnedVoucherStatus::PartiallyRedeemed => {
                if self.voucher_leaf.lifecycle != VoucherLifecycleV1::PartiallyRedeemed
                    || self.voucher_leaf.remaining_value == 0
                    || self.voucher_leaf.remaining_value >= self.voucher_leaf.face_value
                {
                    return Err(WalletError::InvalidConfig(
                        "owned voucher partial redemption status drifted from lifecycle"
                            .to_string(),
                    ));
                }
            }
            OwnedVoucherStatus::Redeemed => {
                if self.voucher_leaf.lifecycle != VoucherLifecycleV1::Redeemed
                    || self.voucher_leaf.remaining_value != 0
                {
                    return Err(WalletError::InvalidConfig(
                        "owned voucher redeemed status drifted from lifecycle".to_string(),
                    ));
                }
            }
            OwnedVoucherStatus::Rejected => {
                if self.voucher_leaf.lifecycle != VoucherLifecycleV1::Rejected {
                    return Err(WalletError::InvalidConfig(
                        "owned voucher rejected status drifted from lifecycle".to_string(),
                    ));
                }
            }
            OwnedVoucherStatus::Refunded => {
                if self.voucher_leaf.lifecycle != VoucherLifecycleV1::Refunded {
                    return Err(WalletError::InvalidConfig(
                        "owned voucher refunded status drifted from lifecycle".to_string(),
                    ));
                }
            }
            OwnedVoucherStatus::Expired => {
                if self.voucher_leaf.lifecycle != VoucherLifecycleV1::Expired {
                    return Err(WalletError::InvalidConfig(
                        "owned voucher expired status drifted from lifecycle".to_string(),
                    ));
                }
            }
            OwnedVoucherStatus::Quarantined => {}
        }

        if self.policy.availability != WalletPolicyAvailability::Available
            && self.status != OwnedVoucherStatus::Quarantined
        {
            return Err(WalletError::InvalidConfig(
                "owned voucher with unavailable policy must stay quarantined".to_string(),
            ));
        }

        if self.status == OwnedVoucherStatus::Quarantined
            && self
                .policy
                .quarantine_reason
                .as_deref()
                .unwrap_or("")
                .trim()
                .is_empty()
        {
            return Err(WalletError::InvalidConfig(
                "owned voucher quarantine must keep a durable reason".to_string(),
            ));
        }

        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OwnedRightPayload {
    pub version: u32,
    pub wallet_id: PersistWalletId,
    pub account_id: Option<u128>,
    pub terminal_id: TerminalId,
    pub right_leaf: RightLeaf,
    pub status: OwnedRightStatus,
    pub source: OwnedObjectSource,
    pub first_seen: Option<ObjectSeenRef>,
    pub last_updated_ms: u64,
    pub scan_ref: Option<ScanRef>,
    pub receive_ref: Option<ReceiveRef>,
    pub confirmation_ref: Option<ConfirmRef>,
    pub labels: Vec<String>,
    pub policy: OwnedObjectPolicy,
    pub holder_opening: Option<Vec<u8>>,
    pub control_opening: Option<Vec<u8>>,
    pub beneficiary_opening: Option<Vec<u8>>,
    pub checksum: Option<[u8; 32]>,
}

impl OwnedRightPayload {
    pub const VERSION: u32 = 1;

    pub fn migrate_to_current(self) -> WalletResult<Self> {
        match self.version {
            Self::VERSION => Ok(self),
            version => Err(WalletError::UnsupportedVersion(version)),
        }
    }

    pub fn compute_checksum(&self) -> [u8; 32] {
        use crate::domains::hashing::{canonicalize_bytes, SnapshotChecksumHasher};
        use z00z_utils::codec::{BincodeCodec, Codec};

        let mut canonical = self.clone();
        canonical.checksum = None;

        let payload_bytes = BincodeCodec.serialize(&canonical).unwrap_or_default();
        let hash = SnapshotChecksumHasher::new_with_label("owned_right")
            .chain(canonicalize_bytes(&payload_bytes))
            .finalize();

        let mut out = [0u8; 32];
        out.copy_from_slice(&hash.as_ref()[..32]);
        out
    }

    pub fn verify_checksum(&self) -> WalletResult<()> {
        match self.checksum {
            None => Err(WalletError::ChecksumMismatch {
                expected: hex::encode(self.compute_checksum()),
                actual: "missing checksum".to_string(),
            }),
            Some(stored_checksum) => {
                use subtle::ConstantTimeEq;

                let computed_checksum = self.compute_checksum();
                if stored_checksum
                    .as_ref()
                    .ct_eq(computed_checksum.as_ref())
                    .unwrap_u8()
                    != 0
                {
                    Ok(())
                } else {
                    Err(WalletError::ChecksumMismatch {
                        expected: hex::encode(stored_checksum),
                        actual: hex::encode(computed_checksum),
                    })
                }
            }
        }
    }

    pub fn validate_invariants(&self) -> WalletResult<()> {
        self.right_leaf
            .check()
            .map_err(|error| WalletError::InvalidConfig(error.to_string()))?;

        if self.terminal_id != self.right_leaf.terminal_id {
            return Err(WalletError::InvalidConfig(
                "owned right terminal id drifted from right leaf".to_string(),
            ));
        }

        if matches!(
            self.status,
            OwnedRightStatus::Delegated | OwnedRightStatus::Consumed | OwnedRightStatus::Challenged
        ) && self.right_leaf.transition_policy_id == [0u8; 32]
        {
            return Err(WalletError::InvalidConfig(
                "owned right transition status requires a transition policy".to_string(),
            ));
        }

        if self.status == OwnedRightStatus::Revoked
            && self.right_leaf.revocation_policy_id == [0u8; 32]
        {
            return Err(WalletError::InvalidConfig(
                "owned right revoked status requires a revocation policy".to_string(),
            ));
        }

        if self.status == OwnedRightStatus::Challenged
            && self.right_leaf.challenge_policy_id == [0u8; 32]
        {
            return Err(WalletError::InvalidConfig(
                "owned right challenged status requires a challenge policy".to_string(),
            ));
        }

        if self.policy.availability != WalletPolicyAvailability::Available
            && self.status != OwnedRightStatus::Quarantined
        {
            return Err(WalletError::InvalidConfig(
                "owned right with unavailable policy must stay quarantined".to_string(),
            ));
        }

        if self.status == OwnedRightStatus::Quarantined
            && self
                .policy
                .quarantine_reason
                .as_deref()
                .unwrap_or("")
                .trim()
                .is_empty()
        {
            return Err(WalletError::InvalidConfig(
                "owned right quarantine must keep a durable reason".to_string(),
            ));
        }

        Ok(())
    }
}

#[allow(clippy::large_enum_variant)]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "family", content = "payload", rename_all = "snake_case")]
pub enum WalletInventoryPayload {
    Asset(OwnedAssetPayload),
    Voucher(OwnedVoucherPayload),
    Right(OwnedRightPayload),
}

impl WalletInventoryPayload {
    pub fn family(&self) -> OwnedObjectFamily {
        match self {
            Self::Asset(_) => OwnedObjectFamily::Asset,
            Self::Voucher(_) => OwnedObjectFamily::Voucher,
            Self::Right(_) => OwnedObjectFamily::Right,
        }
    }

    pub fn status(&self) -> WalletObjectStatus {
        match self {
            Self::Asset(payload) => WalletObjectStatus::Asset(payload.status),
            Self::Voucher(payload) => WalletObjectStatus::Voucher(payload.status),
            Self::Right(payload) => WalletObjectStatus::Right(payload.status),
        }
    }

    pub fn wallet_id(&self) -> &PersistWalletId {
        match self {
            Self::Asset(payload) => &payload.wallet_id,
            Self::Voucher(payload) => &payload.wallet_id,
            Self::Right(payload) => &payload.wallet_id,
        }
    }

    pub fn account_id(&self) -> Option<u128> {
        match self {
            Self::Asset(payload) => payload.account_id,
            Self::Voucher(payload) => payload.account_id,
            Self::Right(payload) => payload.account_id,
        }
    }

    pub fn labels(&self) -> &[String] {
        match self {
            Self::Asset(payload) => &payload.labels,
            Self::Voucher(payload) => &payload.labels,
            Self::Right(payload) => &payload.labels,
        }
    }

    pub fn policy_availability(&self) -> WalletPolicyAvailability {
        match self {
            Self::Asset(_) => WalletPolicyAvailability::Available,
            Self::Voucher(payload) => payload.policy.availability,
            Self::Right(payload) => payload.policy.availability,
        }
    }

    pub fn policy(&self) -> Option<&OwnedObjectPolicy> {
        match self {
            Self::Asset(_) => None,
            Self::Voucher(payload) => Some(&payload.policy),
            Self::Right(payload) => Some(&payload.policy),
        }
    }

    pub fn stable_object_key(&self) -> [u8; 32] {
        match self {
            Self::Asset(payload) => payload.asset_id,
            Self::Voucher(payload) => payload.terminal_id.into_bytes(),
            Self::Right(payload) => payload.terminal_id.into_bytes(),
        }
    }

    pub fn holder_commitment(&self) -> Option<[u8; 32]> {
        match self {
            Self::Asset(_) => None,
            Self::Voucher(payload) => Some(payload.voucher_leaf.holder_commitment),
            Self::Right(payload) => Some(payload.right_leaf.holder_commitment),
        }
    }

    pub fn kind_id(&self) -> u8 {
        match self {
            Self::Asset(_) => ObjectKindId::OwnedAsset as u8,
            Self::Voucher(_) => ObjectKindId::OwnedVoucher as u8,
            Self::Right(_) => ObjectKindId::OwnedRight as u8,
        }
    }

    pub fn payload_version(&self) -> u16 {
        match self {
            Self::Asset(_) => PAYLOAD_VERSION_OWNED_ASSET,
            Self::Voucher(_) => PAYLOAD_VERSION_OWNED_VOUCHER,
            Self::Right(_) => PAYLOAD_VERSION_OWNED_RIGHT,
        }
    }

    pub fn verify_checksum(&self) -> WalletResult<()> {
        match self {
            Self::Asset(payload) => payload.verify_checksum(),
            Self::Voucher(payload) => payload.verify_checksum(),
            Self::Right(payload) => payload.verify_checksum(),
        }
    }

    pub fn validate_invariants(&self) -> WalletResult<()> {
        match self {
            Self::Asset(payload) => {
                let _ = payload.validate_invariants()?;
                Ok(())
            }
            Self::Voucher(payload) => payload.validate_invariants(),
            Self::Right(payload) => payload.validate_invariants(),
        }
    }

    pub fn inventory_sort_key(&self) -> (u8, [u8; 32]) {
        let family_tag = match self.family() {
            OwnedObjectFamily::Asset => 1,
            OwnedObjectFamily::Voucher => 2,
            OwnedObjectFamily::Right => 3,
        };
        (family_tag, self.stable_object_key())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "family", content = "payload", rename_all = "snake_case")]
pub enum OwnedNonAssetPayload {
    Voucher(OwnedVoucherPayload),
    Right(OwnedRightPayload),
}

impl OwnedNonAssetPayload {
    pub fn family(&self) -> OwnedObjectFamily {
        match self {
            Self::Voucher(_) => OwnedObjectFamily::Voucher,
            Self::Right(_) => OwnedObjectFamily::Right,
        }
    }

    pub fn stable_object_key(&self) -> [u8; 32] {
        match self {
            Self::Voucher(payload) => payload.terminal_id.into_bytes(),
            Self::Right(payload) => payload.terminal_id.into_bytes(),
        }
    }

    pub fn kind_id(&self) -> u8 {
        match self {
            Self::Voucher(_) => ObjectKindId::OwnedVoucher as u8,
            Self::Right(_) => ObjectKindId::OwnedRight as u8,
        }
    }

    pub fn payload_version(&self) -> u16 {
        match self {
            Self::Voucher(_) => PAYLOAD_VERSION_OWNED_VOUCHER,
            Self::Right(_) => PAYLOAD_VERSION_OWNED_RIGHT,
        }
    }

    pub fn verify_checksum(&self) -> WalletResult<()> {
        match self {
            Self::Voucher(payload) => payload.verify_checksum(),
            Self::Right(payload) => payload.verify_checksum(),
        }
    }

    pub fn validate_invariants(&self) -> WalletResult<()> {
        match self {
            Self::Voucher(payload) => payload.validate_invariants(),
            Self::Right(payload) => payload.validate_invariants(),
        }
    }
}

impl From<OwnedNonAssetPayload> for WalletInventoryPayload {
    fn from(value: OwnedNonAssetPayload) -> Self {
        match value {
            OwnedNonAssetPayload::Voucher(payload) => Self::Voucher(payload),
            OwnedNonAssetPayload::Right(payload) => Self::Right(payload),
        }
    }
}

pub type OwnedObjectPayload = WalletInventoryPayload;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WalletOwnedObject {
    pub object_id: Option<u128>,
    pub payload: WalletInventoryPayload,
}

impl WalletOwnedObject {
    #[must_use]
    pub fn family(&self) -> OwnedObjectFamily {
        self.payload.family()
    }

    #[must_use]
    pub fn status(&self) -> WalletObjectStatus {
        self.payload.status()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WalletTxRole {
    Sender,
    Receiver,
    Observer,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WalletTxEventType {
    Built,
    Submitted,
    Admitted,
    Imported,
    Exported,
    Cancelled,
    Confirmed,
    Rejected,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WalletTxPayload {
    pub version: u32,
    pub wallet_id: PersistWalletId,
    pub tx_id: PersistTxId,
    pub tx_hash: String,
    pub status: TxStatus,
    pub role: WalletTxRole,
    pub package_bytes: Option<Vec<u8>>,
    pub input_asset_ids: Vec<[u8; 32]>,
    pub output_asset_ids: Vec<[u8; 32]>,
    pub imported: bool,
    pub exported: bool,
    pub submitted_at_ms: Option<u64>,
    pub admitted_at_ms: Option<u64>,
    pub confirmed_at_ms: Option<u64>,
    pub cancelled_at_ms: Option<u64>,
    pub confirmation_evidence_ref: Option<String>,
    pub error_or_reject_reason: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WalletTxEventPayload {
    pub version: u32,
    pub wallet_id: PersistWalletId,
    pub tx_id: PersistTxId,
    pub event_seq: u64,
    pub event_type: WalletTxEventType,
    pub event_time_ms: u64,
    pub payload: Vec<u8>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupManifestPayload {
    pub version: u32,
    pub wallet_id: PersistWalletId,
    pub created_at_ms: u64,
    pub network: String,
    pub chain: String,
    pub profile_count: u32,
    pub owned_asset_count: u32,
    #[serde(default)]
    pub owned_object_count: u32,
    pub scan_state_count: u32,
    pub stealth_meta_count: u32,
    pub tofu_pins_count: u32,
    pub key_ref_count: u32,
    pub tx_record_count: u32,
    pub has_tx_history_sidecar: bool,
    pub tx_history_plane: String,
    pub checksum: Option<[u8; 32]>,
}

impl BackupManifestPayload {
    pub const VERSION: u32 = 3;
    pub const TX_HISTORY_JSONL: &'static str = "jsonl_sidecar";

    pub fn compute_checksum(&self) -> [u8; 32] {
        use crate::domains::hashing::{canonicalize_bytes, SnapshotChecksumHasher};
        use z00z_utils::codec::{BincodeCodec, Codec};

        let mut canonical = self.clone();
        canonical.checksum = None;

        let payload_bytes = BincodeCodec.serialize(&canonical).unwrap_or_default();
        let hash = SnapshotChecksumHasher::new_with_label("backup_manifest")
            .chain(canonicalize_bytes(&payload_bytes))
            .finalize();

        let mut out = [0u8; 32];
        out.copy_from_slice(&hash.as_ref()[..32]);
        out
    }

    pub fn verify_checksum(&self) -> WalletResult<()> {
        match self.checksum {
            None => Err(WalletError::ChecksumMismatch {
                expected: hex::encode(self.compute_checksum()),
                actual: "missing checksum".to_string(),
            }),
            Some(stored_checksum) => {
                use subtle::ConstantTimeEq;

                let computed_checksum = self.compute_checksum();
                if stored_checksum
                    .as_ref()
                    .ct_eq(computed_checksum.as_ref())
                    .unwrap_u8()
                    != 0
                {
                    Ok(())
                } else {
                    Err(WalletError::ChecksumMismatch {
                        expected: hex::encode(stored_checksum),
                        actual: hex::encode(computed_checksum),
                    })
                }
            }
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyRefPayload {
    pub purpose: String,
    pub algo: String,
    pub public_key: Vec<u8>,
    pub secret_name: String,
    pub created_at: u64,
    pub label: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum SeedMainMnemonicLanguage {
    English,
}

impl SeedMainMnemonicLanguage {
    pub(crate) fn to_mnemonic_language(self) -> MnemonicLanguage {
        match self {
            SeedMainMnemonicLanguage::English => MnemonicLanguage::English,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct SeedMainEntropyPayload {
    pub entropy_bytes: Vec<u8>,
    pub mnemonic_language: SeedMainMnemonicLanguage,
}
