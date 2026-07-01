/// Password verifier state (salt + argon2 hash)
///
/// SECURITY: Does NOT store the actual password!
/// Stores only what's needed to verify a password attempt.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct PasswordVerifierState {
    /// Random salt for password hashing
    pub salt: [u8; 32],
    /// Argon2 hash of password + salt
    pub verifier: [u8; 32],
}

/// Receiver derivation state (BIP44 indices)
///
/// Tracks the next index for payment and change receiver material.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct ReceiverDeriverState {
    /// Next index for payment receivers (m/44'/1337'/0'/0/x)
    pub next_payment_index: u32,
    /// Next index for change receivers (m/44'/1337'/0'/1/x)
    pub next_change_index: u32,
}

/// Full wallet export payload used by the public backup and export flows.
///
/// The wallet record carries the restorable wallet state, while the seed phrase
/// carries the root secret material required for a full restore.
///
/// This type is an in-memory restore bundle and must only cross encrypted
/// transport or storage boundaries. Public metadata/header surfaces must never
/// serialize `seed_phrase` directly.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct WalletExportIdentity {
    /// Network label associated with the persisted wallet context.
    pub network: String,
    /// Chain label required to reopen the persisted `.wlt` file.
    pub chain: String,
}

/// Encrypted export bundle used by wallet backup and transport flows.
///
/// Together with the encrypted `.wlt` database, this remains one of the only
/// wallet-local authority surfaces. `owned_assets` preserves the cash-only
/// asset plane, while `owned_objects` carries typed Voucher and Right rows
/// additively without creating a second persistence database or a second export
/// contract.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WalletExportPack {
    /// Export pack schema version.
    #[serde(default = "wallet_export_pack_legacy_version")]
    pub version: u32,
    /// Backup manifest proving the exported state shape.
    #[serde(default)]
    pub manifest: Option<crate::db::BackupManifestPayload>,
    /// Current live wallet profile payload.
    #[serde(default)]
    pub wallet_profile: Option<crate::db::WalletProfilePayload>,
    /// Current live owned-asset payloads.
    #[serde(default)]
    pub owned_assets: Vec<crate::db::OwnedAssetPayload>,
    /// Current live non-asset object payloads.
    ///
    /// Assets remain on `owned_assets` so legacy asset-only restores keep their
    /// existing canonical path; this field carries typed Voucher and Right
    /// inventory rows additively and must never be reinterpreted as ordinary
    /// cash balance.
    #[serde(default)]
    pub owned_objects: Vec<crate::db::WalletInventoryPayload>,
    /// Current live scan cursor payload.
    #[serde(default)]
    pub scan_state: Option<crate::db::ScanStatePayload>,
    /// Current live stealth metadata payload.
    #[serde(default)]
    pub stealth_meta: Option<crate::db::StealthMetaPayload>,
    /// Current live TOFU pins payload.
    #[serde(default)]
    pub tofu_pins: Option<crate::db::TofuPinsPayload>,
    /// Current live key-reference payload.
    #[serde(default)]
    pub keys: Option<crate::db::KeysPayload>,
    /// Explicit tx-history plane marker while tx history remains JSONL.
    #[serde(default)]
    pub tx_history_plane: Option<String>,
    /// Root secret material for restore.
    pub seed_phrase: String,
    /// Persisted wallet identity for lossless export/import round-trips.
    #[serde(default)]
    pub wallet_identity: Option<WalletExportIdentity>,
}

fn wallet_export_pack_legacy_version() -> u32 {
    1
}

impl WalletExportPack {
    /// Current export-pack schema version for explicit wallet-state bundles.
    pub const VERSION: u32 = 2;

    /// Returns `true` when the pack carries the explicit profile-first state shape.
    pub fn uses_explicit_state(&self) -> bool {
        self.wallet_profile.is_some()
    }
}
