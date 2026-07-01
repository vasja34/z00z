//! Core generic RPC type templates (ONE SOURCE OF TRUTH)
//!
//! This module contains reusable generic types for RPC responses.
//! All other type modules should import from here instead of duplicating.

use serde::{Deserialize, Serialize};
use z00z_core::assets::registry::AssetId;
use z00z_core::AssetClass;

// ============================================================================
// PAGINATION
// ============================================================================

/// Query parameters for paginated list endpoints
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimePaginationParams {
    /// Maximum number of items to return
    pub limit: Option<usize>,
    /// Cursor for next page (opaque string)
    pub cursor: Option<String>,
    /// Include total count in response
    pub include_total: Option<bool>,
}

/// Generic cursor pagination metadata for list endpoints.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimeCursorPage {
    /// Cursor for next page (None if last page)
    pub next_cursor: Option<String>,
    /// Whether more items are available
    pub has_more: bool,
    /// Total count (if requested)
    pub total_count: Option<usize>,
}

/// Generic paginated response wrapper.
///
/// This is the canonical "items"-based response shape.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimePaginatedResponse<T> {
    /// List of items in this page
    #[serde(rename = "items")]
    pub items: Vec<T>,
    /// Cursor for next page (None if last page)
    pub next_cursor: Option<String>,
    /// Whether more items are available
    pub has_more: bool,
    /// Total count (if requested)
    pub total_count: Option<usize>,
}

/// Paginated response wrapper serialized with an `assets` field.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimePaginatedAssetsResponse<T> {
    /// List of assets in this page
    #[serde(rename = "assets")]
    pub items: Vec<T>,
    /// Cursor for next page (None if last page)
    pub next_cursor: Option<String>,
    /// Whether more items are available
    pub has_more: bool,
    /// Total count (if requested)
    pub total_count: Option<usize>,
}

// ============================================================================
// OPERATION STATUS
// ============================================================================

/// Generic operation status (success/failure with message)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimeOperationStatus {
    /// Whether operation succeeded
    pub success: bool,
    /// Human-readable message
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub message: String,
}

/// Operation status with transaction reference
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimeOperationStatusWithTx {
    #[serde(flatten)]
    pub status: RuntimeOperationStatus,
    /// Transaction ID
    pub tx_id: PersistTxId,
}

/// Operation status with wallet reference
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimeOperationStatusWithWallet {
    #[serde(flatten)]
    pub status: RuntimeOperationStatus,
    /// Wallet ID
    pub wallet_id: PersistWalletId,
}

// ============================================================================
// VALIDATION
// ============================================================================

/// Generic validation result (valid/invalid with optional error).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimeValidationResult {
    pub valid: bool,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub warnings: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

impl RuntimeValidationResult {
    pub fn valid() -> Self {
        Self {
            valid: true,
            warnings: Vec::new(),
            error: None,
        }
    }

    pub fn valid_with_warnings(warnings: Vec<String>) -> Self {
        Self {
            valid: true,
            warnings,
            error: None,
        }
    }

    pub fn invalid(error: impl Into<String>) -> Self {
        Self {
            valid: false,
            warnings: Vec::new(),
            error: Some(error.into()),
        }
    }
}

// ============================================================================
// PERSISTENT IDS
// ============================================================================

/// Transaction ID (persistent storage format)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PersistTxId(pub String);

impl PersistTxId {
    pub fn new(value: String) -> Self {
        Self(value)
    }
}

/// Wallet ID (persistent storage format)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PersistWalletId(pub String);

impl Default for PersistWalletId {
    fn default() -> Self {
        Self("stub-wallet-id".to_string())
    }
}

// ============================================================================
// EVENTS
// ============================================================================

/// Shared event metadata.
///
/// This is provided as a template for deduplication.
/// Note: flattening this into enums may change JSON shape.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimeEventMeta {
    pub wallet_id: PersistWalletId,
    pub timestamp_ms: u64,
}

// ============================================================================
// ASSET REFERENCE
// ============================================================================

/// Unified asset reference for all asset operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimeAssetRef {
    /// Asset identifier
    pub asset_id: AssetId,
    /// Serial number for uniqueness
    pub serial_id: u32,
    /// Asset symbol (ticker)
    pub symbol: String,
    /// Asset class/category
    pub class: AssetClass,
}

/// Asset reference with amount (prevents positional coupling)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimeAssetAmount {
    #[serde(flatten)]
    pub asset: RuntimeAssetRef,
    /// Amount of asset
    pub amount: u64,
}

// ============================================================================
// BACKGROUND JOBS
// ============================================================================

/// Status of background job/task
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimeJobStatus {
    /// Unique identifier for tracking job progress.
    ///
    /// Present for endpoints that support async job tracking.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub job_id: Option<String>,
    /// Job status (e.g., "started", "completed", "failed").
    ///
    /// Kept as a string for backwards-compat across RPC schemas.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<String>,
    /// Progress ratio (0.0 to 1.0).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub progress: Option<f32>,
    /// Estimated time remaining (seconds).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub eta_seconds: Option<u64>,
}

impl RuntimeJobStatus {
    pub fn progress_or_zero(&self) -> f32 {
        self.progress.unwrap_or(0.0)
    }
}

/// Job execution states
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum JobState {
    /// Job queued, not started
    Pending,
    /// Currently executing
    Running,
    /// Successfully completed
    Completed,
    /// Failed with error
    Failed,
    /// Cancelled by user
    Cancelled,
}

// ============================================================================
// ENCRYPTION
// ============================================================================

/// Encryption metadata for client-side decryption.
///
/// Provides information needed to decrypt a `RuntimeEncryptedResponse`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RuntimeEncryptionMetadata {
    /// Encryption algorithm used (e.g., "xchacha20poly1305")
    pub algorithm: String,
    /// Nonce/IV for decryption (hex-encoded, algorithm-specific length: 24 bytes for XChaCha20-Poly1305)
    pub nonce: String,
    /// Key derivation method (e.g., "HKDF-SHA256")
    pub key_derivation: String,
}

/// Encrypted response wrapper for sensitive data.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RuntimeEncryptedResponse {
    /// Encrypted payload (hex-encoded ciphertext + auth tag)
    pub ciphertext: String,
    /// Encryption metadata for client-side decryption
    pub metadata: RuntimeEncryptionMetadata,
}

impl RuntimeEncryptedResponse {
    pub fn stub(plaintext: &str) -> Self {
        Self {
            ciphertext: format!("encrypted:{}", plaintext),
            metadata: RuntimeEncryptionMetadata {
                algorithm: "xchacha20poly1305".to_string(),
                nonce: "0x000000000000000000000000000000000000000000000000".to_string(),
                key_derivation: "HKDF-SHA256".to_string(),
            },
        }
    }

    pub fn is_encrypted(&self) -> bool {
        !self.ciphertext.is_empty()
    }
}

#[cfg(test)]
#[path = "test_common_types.rs"]
mod tests;
