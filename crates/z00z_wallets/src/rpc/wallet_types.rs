//! Wallet RPC request/response types
//!
//! These types define the data structures for wallet-related RPC methods.
//! All types must be serializable for JSON-RPC 2.0 communication.

use serde::{Deserialize, Serialize};

use super::common::PersistWalletId;
use super::common::{RuntimeOperationStatus, RuntimeOperationStatusWithWallet};
pub use super::key::{RuntimeExportPublicKeyResponse, RuntimeKeyDeriveResponse};
pub use super::security::SessionToken;

/// Policy rules for transaction enforcement
pub use crate::wallet::policy::PolicyRules;

/// Wallet information summary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersistWalletInfo {
    /// Unique wallet identifier
    pub id: PersistWalletId,
    /// Display name
    pub name: String,
    /// Creation timestamp (Unix epoch milliseconds)
    pub created_at: u64,
    /// Lock status
    pub is_locked: bool,
}

/// Target-specific wallet source used for opening existing wallet files.
///
/// 📌 Phase 1: native path is supported.
/// 📌 Future: browser/WASM will provide `bytes` from a file picker.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum WalletSource {
    /// Native filesystem path.
    Path { path: String },
    /// Raw `.wlt` bytes (typically from a browser file picker).
    Bytes { bytes: Vec<u8> },
}

/// Discovery metadata extracted from a `.wlt` file without unlocking it.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersistWalletDiscovery {
    pub wallet_id: PersistWalletId,
    pub network: String,
    pub chain: String,
}

/// Wallet lifecycle events forwarded by the UI/app layer.
///
/// These events allow the wallet service to revoke in-memory sessions holding key material
/// when the application is backgrounded/suspended or the screen locks.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WalletLifecycleEvent {
    Backgrounded,
    Foregrounded,
    Suspended,
    ScreenLocked,
}

/// Response for wallet.list
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimeListWalletsResponse {
    /// Wallet list
    pub wallets: Vec<PersistWalletInfo>,
}

/// Response for wallet.create method
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimeCreateWalletResponse {
    /// Unique identifier for the created wallet
    pub wallet_id: PersistWalletId,
    /// Display name of the wallet
    pub name: String,
    /// Seed phrase returned ONLY on creation.
    pub seed_phrase: String,
    /// Deterministic password strength score (0..=100).
    pub password_strength_score: u8,
    /// Unix timestamp when the wallet was created (milliseconds)
    pub created_at: u64,
}

/// Response for `app.wallet.recover_from_seed`.
///
/// Recovery does not return the seed phrase.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimeRecoverFromSeedResponse {
    /// Unique identifier for the recovered wallet.
    pub wallet_id: PersistWalletId,
    /// Display name of the wallet.
    pub name: String,
    /// Persisted wallet network identifier.
    pub network: String,
    /// Persisted wallet chain identifier.
    pub chain: String,
    /// Deterministic password strength score (0..=100).
    pub password_strength_score: u8,
    /// Unix timestamp when recovery completed (milliseconds).
    pub recovered_at: u64,
}

/// Request parameters for `app.wallet.recover_from_seed`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimeRecoverFromSeedParams {
    /// Display name for the recovered wallet.
    pub name: String,
    /// Password to encrypt the wallet.
    pub password: String,
    /// First entry of the seed phrase used for recovery (24 English words).
    pub mnemonic_a: String,
    /// Second entry of the seed phrase used for recovery (must match `mnemonic_a`).
    pub mnemonic_b: String,
    /// Target wallet network identifier to persist into `.wlt` meta.
    pub network: String,
    /// Target wallet chain identifier to persist into `.wlt` meta.
    pub chain: String,
}

// SessionToken moved to security.rs for unified security types.
// Re-export here so wallet RPC modules keep one import path.

/// Response for wallet.unlock
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimeUnlockWalletResponse {
    #[serde(flatten)]
    pub status: RuntimeOperationStatus,
    /// Session token for authenticated operations.
    pub session_token: SessionToken,
}

/// Request parameters for wallet.create
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimeCreateWalletParams {
    /// Display name for the wallet
    pub name: String,
    /// Password to encrypt the wallet
    pub password: String,
    /// Optional seed phrase for wallet recovery
    pub seed_phrase: Option<String>,
}

/// Request parameters for wallet.unlock
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimeUnlockWalletParams {
    /// Wallet identifier to unlock
    pub wallet_id: PersistWalletId,
    /// Password to decrypt the wallet
    pub password: String,
}

/// Request parameters for wallet.lock
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimeLockWalletParams {
    /// Wallet identifier to lock
    pub wallet_id: PersistWalletId,
}

/// Response for wallet.lock
pub type RuntimeLockWalletResponse = RuntimeOperationStatusWithWallet;

/// Request parameters for wallet.delete
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimeDeleteWalletParams {
    /// Wallet identifier to delete.
    pub wallet_id: PersistWalletId,
    /// Wallet password.
    pub password: String,
    /// Confirmation phrase.
    pub confirm_phrase: String,
}

/// Response for wallet.delete
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimeDeleteWalletResponse {
    #[serde(flatten)]
    pub status: RuntimeOperationStatus,
    /// Wallet identifier.
    pub wallet_id: PersistWalletId,
    /// Whether the wallet was deleted.
    pub deleted: bool,
}

/// Wallet settings fields.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersistWalletSettings {
    /// Auto-lock timeout seconds.
    pub auto_lock_timeout: u64,
    /// Default fee in display units.
    pub default_fee: String,
    /// Currency display code.
    pub currency_display: String,
    /// Policy rules for transaction enforcement (P0 CRITICAL)
    pub policy_rules: Option<PolicyRules>,
    /// Unix timestamp when settings were created (milliseconds).
    pub created_at: u64,
    /// Unix timestamp when settings were updated (milliseconds).
    pub updated_at: u64,
}

/// Response for wallet.show_seed_phrase
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimeShowSeedPhraseResponse {
    /// Encrypted seed phrase payload.
    ///
    /// # Security
    /// Never return secrets in multiple representations to minimize leak surface.
    pub encrypted_payload: super::common::RuntimeEncryptedResponse,
}

/// Request parameters for wallet.export
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimeExportWalletParams {
    /// Wallet identifier to export.
    pub wallet_id: PersistWalletId,
    /// Optional output path.
    pub output_path: Option<String>,
}

/// Response for wallet.export
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimeExportWalletResponse {
    /// Whether the export succeeded.
    pub success: bool,
    /// Export path (if relevant).
    pub export_path: Option<String>,
    /// Encrypted wallet export payload (Phase 1: stub encryption wrapper).
    pub encrypted_payload: Option<super::common::RuntimeEncryptedResponse>,
}

/// Request parameters for wallet.import
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimeImportWalletParams {
    /// Wallet identifier to import.
    pub wallet_id: PersistWalletId,
    /// Display name.
    pub name: String,
}

/// Response for wallet.import
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimeImportWalletResponse {
    #[serde(flatten)]
    pub status: RuntimeOperationStatus,
    /// Wallet identifier.
    pub wallet_id: PersistWalletId,
    /// Display name.
    pub name: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use z00z_utils::codec::{Codec, JsonCodec};

    #[test]
    fn test_wallet_id_serialization() {
        let codec = JsonCodec;
        let wallet_id = PersistWalletId("test-wallet-123".to_string());
        let json_bytes = codec.serialize(&wallet_id).unwrap();
        let deserialized: PersistWalletId = codec.deserialize(&json_bytes).unwrap();
        assert_eq!(wallet_id, deserialized);
    }

    #[test]
    fn test_wallet_info_json_roundtrip() {
        let codec = JsonCodec;
        let info = PersistWalletInfo {
            id: PersistWalletId("wallet-id-456".to_string()),
            name: "Test Wallet".to_string(),
            created_at: 1_703_001_234_000,
            is_locked: false,
        };

        let json_bytes = codec.serialize(&info).unwrap();
        let deserialized: PersistWalletInfo = codec.deserialize(&json_bytes).unwrap();

        assert_eq!(info.id, deserialized.id);
        assert_eq!(info.name, deserialized.name);
        assert_eq!(info.created_at, deserialized.created_at);
        assert_eq!(info.is_locked, deserialized.is_locked);
    }

    #[test]
    fn test_create_wallet_response_json() {
        let codec = JsonCodec;
        let response = RuntimeCreateWalletResponse {
            wallet_id: PersistWalletId("new-wallet-789".to_string()),
            name: "My New Wallet".to_string(),
            seed_phrase: "stub seed phrase".to_string(),
            password_strength_score: 42,
            created_at: 1_703_001_234_000,
        };

        let json_bytes = codec.serialize(&response).unwrap();
        let deserialized: RuntimeCreateWalletResponse = codec.deserialize(&json_bytes).unwrap();

        assert_eq!(response.wallet_id, deserialized.wallet_id);
        assert_eq!(response.name, deserialized.name);
        assert_eq!(response.seed_phrase, deserialized.seed_phrase);
        assert_eq!(
            response.password_strength_score,
            deserialized.password_strength_score
        );
        assert_eq!(response.created_at, deserialized.created_at);
    }

    #[test]
    fn test_session_token_json_roundtrip() {
        let codec = JsonCodec;
        let token = SessionToken {
            token: "session-token-abc123".to_string(),
            wallet_id: PersistWalletId::default(),
            created_at: 1_703_000_000_000,
            expires_at: 1_703_005_000_000,
            last_activity_at: 1_703_000_000_000,
            permissions: vec![
                "wallet.tx.send_transaction".to_string(),
                "app.wallet.export_wallet".to_string(),
            ],
        };

        let json_bytes = codec.serialize(&token).unwrap();
        let deserialized: SessionToken = codec.deserialize(&json_bytes).unwrap();

        assert_eq!(token.token, deserialized.token);
        assert_eq!(token.wallet_id, deserialized.wallet_id);
        assert_eq!(token.expires_at, deserialized.expires_at);
        assert_eq!(token.permissions, deserialized.permissions);
    }
}
