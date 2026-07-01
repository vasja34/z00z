//! Key RPC request/response types
//!
//! This module defines all data structures for key management RPC operations.
//!
//! # Architecture Compliance
//!
//! - ✅ Serializable types (serde Serialize/Deserialize)
//! - ✅ Clean DTOs (no business logic)
//! - ✅ Documentation with examples
//! - ✅ Pagination support (cursor-based)

use crate::rpc::types::common::{
    RuntimeOperationStatus, RuntimePaginatedResponse, RuntimeValidationResult,
};
use serde::{Deserialize, Serialize};

// ============================================================================
// Runtime/Persist naming (spec)
// ============================================================================

/// Response for wallet.key.derive (wallet-scoped namespace)
///
/// This response intentionally returns only a public key to preserve the historical
/// JSON schema used by wallet-scoped key operations.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimeKeyDeriveResponse {
    /// Derived public key.
    pub public_key: String,
}

/// Response for wallet.key.export_public (wallet-scoped namespace)
///
/// This response intentionally returns only a public key to preserve the historical
/// JSON schema used by wallet-scoped key operations.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimeExportPublicKeyResponse {
    /// Exported public key.
    pub public_key: String,
}

/// Response for `wallet.key.derive_receiver`.
///
/// Contains the derived receiver-facing public material anchor and canonical
/// BIP44 path. The legacy recipient string is not part of the live RPC contract.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimeDeriveReceiverResponse {
    /// Derived public key (hex-encoded)
    pub public_key: String,
    /// BIP44 path used for derivation
    pub path: String,
}

/// Response for `wallet.key.get_receiver_card` method.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimeGetReceiverCardResponse {
    /// Owner handle (hex).
    pub owner_handle: String,
    /// View public key bytes (hex).
    pub view_key: String,
    /// Identity public key bytes (hex).
    pub identity_key: String,
    /// Signature bytes (hex).
    pub signature: String,
    /// Compact URL-safe verified record encoding.
    pub card_compact: String,
    /// Canonical registry entry id (hex).
    pub registry_entry_id: String,
    /// Publication epoch for the emitted card record.
    pub card_epoch: u64,
    /// Human-readable owner handle display string.
    pub owner_handle_display: String,
}

/// Optional metadata payload for `wallet.key.create_payment_request`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimePaymentRequestMetaInput {
    /// Optional payment memo.
    pub memo: Option<String>,
    /// Optional 16-byte payment id encoded as 32-char hex string.
    pub payment_id: Option<String>,
}

/// Response for `wallet.key.create_payment_request` method.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimeCreatePaymentRequestResponse {
    /// Stable owner handle (hex).
    pub owner_handle: String,
    /// Receiver view key bytes (hex).
    pub view_key: String,
    /// Receiver identity key bytes (hex).
    pub identity_key: String,
    /// Request id bytes (hex).
    pub req_id: String,
    /// Chain id used in request.
    pub chain_id: u32,
    /// Optional fixed amount.
    pub amount: Option<u64>,
    /// Expiry unix timestamp in seconds.
    pub expiry: u64,
    /// Schnorr signature bytes (hex).
    pub signature: String,
    /// Compact URL-safe request payload.
    pub request_compact: String,
}

/// Validation response for `wallet.key.validate_payment_request`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimeValidatePaymentRequestResponse {
    /// Validation result.
    #[serde(flatten)]
    pub result: RuntimeValidationResult,
    /// Validation outcome for TOFU workflow.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub outcome: Option<String>,
    /// Request id bytes (hex).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub req_id: Option<String>,
    /// Owner handle bytes (hex).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub owner_handle: Option<String>,
    /// Request expiry timestamp.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expiry: Option<u64>,
}

/// Response for key.export_public_material method
///
/// Account-level pub material encrypted for security and wrapped with XChaCha20-Poly1305.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimePubMaterialExportResponse {
    /// Response schema version.
    pub schema_version: u32,
    /// Encrypted pub material (base64 of `salt || envelope`).
    ///
    /// Where `salt` is 16 bytes and `envelope` is the canonical integral-nonce
    /// encoding `nonce || ciphertext_and_tag`.
    pub encrypted_pub_material: String,
    /// Encryption algorithm used.
    pub algorithm: String,
    /// Account number.
    pub account: u32,
    /// Master key fingerprint (for verification).
    pub fingerprint: String,
}

/// Response for `wallet.key.rotate_master_key`.
///
/// The durable rotation contract reports the fingerprint of the rotated wallet
/// encryption root and the number of persisted records rewrapped by the
/// successful rewrite.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimeRotateKeyResponse {
    /// Fingerprint of the rotated wallet-encryption root after the flow.
    pub new_fingerprint: String,
    /// Timestamp of rotation (Unix milliseconds)
    pub rotated_at: u64,
    /// Number of persisted records rewrapped by the rotation flow.
    pub records_rewrapped: u32,
}

/// Receiver filter for `wallet.key.list_receivers`.
///
/// Optional filters to narrow down the receiver list.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimeReceiverFilter {
    /// Filter by used/unused status
    /// - `Some(true)` - Only used receivers
    /// - `Some(false)` - Only unused receivers
    /// - `None` - All receivers
    pub used: Option<bool>,
    /// Filter by change/external receiver type
    /// - `Some(true)` - Only internal/change receivers
    /// - `Some(false)` - Only external receivers
    /// - `None` - All receivers
    pub change: Option<bool>,
}

#[cfg(test)]
#[path = "test_key_types.rs"]
mod tests;

/// Single receiver information.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersistReceiverInfo {
    /// Stable receiver identifier.
    pub receiver_id: String,
    /// BIP44 derivation path.
    pub path: String,
    /// Public key material backing this receiver identifier (hex-encoded).
    pub public_key: String,
    /// Balance in smallest unit (if known).
    pub balance: Option<u64>,
    /// Whether this receiver has been reused.
    pub used: bool,
    /// Whether this is an internal/change receiver.
    pub internal: bool,
    /// Optional user-defined label.
    pub label: Option<String>,
    /// Index in the derivation path.
    pub index: u32,
}

/// Paginated list of receivers with cursor for next page.
pub type RuntimeListReceiversResponse = RuntimePaginatedResponse<PersistReceiverInfo>;

/// Validation response for the compact receiver-card payload.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimeValidateReceiverCardResponse {
    #[serde(flatten)]
    pub result: RuntimeValidationResult,
    /// Receiver-card format if valid.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub format: Option<String>,
}

/// Confirmation of receiver label assignment.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimeLabelReceiverResponse {
    /// Receiver identifier that was labeled.
    pub receiver_id: String,
    /// New label value.
    pub label: String,
    #[serde(flatten)]
    pub status: RuntimeOperationStatus,
}
