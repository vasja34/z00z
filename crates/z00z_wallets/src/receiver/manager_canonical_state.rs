//! Canonical receiver-cache state format for stable HMAC authentication.
//!
//! # Problem
//!
//! Previous implementation used `bincode` serialization for HMAC-signed cache state.
//! This created fragility:
//! - Serde representation changes break verification
//! - Bincode version updates invalidate old cache-state exports
//! - Internal type changes (like `Bip44Path` encoding) break compatibility
//!
//! # Solution
//!
//! Define fixed-length canonical binary format independent of Rust types:
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────────┐
//! │ VERSION (1 byte) │ ENTRY_COUNT (4 bytes LE)                 │
//! ├─────────────────────────────────────────────────────────────┤
//! │ Entry 0: PATH (20 bytes) │ SPEND_PK (32) │ VIEW_PK (32)     │
//! │ Entry 1: PATH (20 bytes) │ SPEND_PK (32) │ VIEW_PK (32)     │
//! │ ...                                                          │
//! └─────────────────────────────────────────────────────────────┘
//! ```
//!
//! **PATH encoding (20 bytes, little-endian):**
//! ```text
//! [purpose(4) | asset_type(4) | account(4) | change(4) | index(4)]
//! ```
//!
//! Each u32 has high bit (0x8000_0000) set for hardened derivation.
//!
//! # Versioning
//!
//! - Current version: 1
//! - Future versions can change entry format
//! - Parsers MUST reject unknown versions
//!
//! # Migration from Bincode
//!
//! **Breaking change:** Old bincode-serialized cache-state exports are NOT compatible.
//!
//! **Migration options:**
//! 1. **Reject old format** (current behavior) - fail verification, force rebuild
//! 2. **Future: Detect and migrate** - attempt bincode decode on failure
//! 3. **CLI flag:** `--force-rebuild-cache` to recreate from wallet
//!
//! # Security
//!
//! - Fixed-length encoding prevents length-extension attacks
//! - Canonical format ensures HMAC stability across upgrades
//! - Version byte enables forward compatibility
//! - HMAC signs: `wallet_id || version || canonical_entries_bytes`
//!
//! # Usage
//!
//! ```ignore
//! use canonical_state::to_canonical;
//!
//! let entries = vec![(path, spend_pk, view_pk)];
//! let canonical_bytes = to_canonical(&entries);
//! let hmac = hmac_sha256(&mac_key, DOMAIN, LABEL, &canonical_bytes);
//! ```

use crate::key::Bip44Path;
use thiserror::Error;

/// Current receiver-cache state format version.
pub const STATE_VERSION: u8 = 1;

/// Canonical BIP-44 path encoding size (5 x u32 = 20 bytes)
pub const BIP44_PATH_BYTES: usize = 20;

/// Fixed entry size: path(20) + spend_pk(32) + view_pk(32)
pub const ENTRY_BYTES: usize = BIP44_PATH_BYTES + 32 + 32;

/// Type alias for receiver-cache entries `(path, spend_pk, view_pk)`.
pub type ReceiverCacheEntry = (Bip44Path, Vec<u8>, Vec<u8>);

/// Canonical receiver-cache state errors.
#[derive(Debug, Error)]
pub enum CanonicalStateError {
    /// State version does not match `STATE_VERSION`.
    #[error("Invalid receiver-cache state version: expected {expected}, got {actual}")]
    InvalidVersion {
        /// Expected state version.
        expected: u8,
        /// Actual state version.
        actual: u8,
    },

    /// State bytes are too short to contain the required header.
    #[error("Receiver-cache state too short: expected at least {expected} bytes, got {actual}")]
    TooShort {
        /// Minimum required byte length.
        expected: usize,
        /// Actual byte length.
        actual: usize,
    },

    /// State length does not match what the entry count declares.
    #[error(
        "Invalid receiver-cache state length: header says {expected} entries ({expected_bytes} bytes), but data is {actual} bytes"
    )]
    InvalidLength {
        /// Entry count from the state header.
        expected: u32,
        /// Expected byte length for the declared entry count.
        expected_bytes: usize,
        /// Actual byte length of the provided snapshot.
        actual: usize,
    },

    /// One of the entries contains an invalid BIP-44 path encoding.
    #[error("Invalid BIP-44 path encoding at entry {entry_index}: {reason}")]
    InvalidPath {
        /// Index of the failing entry.
        entry_index: usize,
        /// Human-readable error reason.
        reason: String,
    },

    /// One of the entries contains an invalid public key length.
    #[error(
        "Invalid public key length at entry {entry_index}: expected 32 bytes for {key_type}, got {actual}"
    )]
    InvalidKeyLength {
        /// Index of the failing entry.
        entry_index: usize,
        /// Public key type label (spend/view).
        key_type: &'static str,
        /// Actual byte length.
        actual: usize,
    },

    /// Entry count cannot be represented as a `u32`.
    #[error("Entry count overflow: cannot serialize {count} entries (max: u32::MAX)")]
    EntryCountOverflow {
        /// Number of entries.
        count: usize,
    },
}

/// Canonical snapshot entry (fixed-length)
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CanonicalEntry {
    /// BIP-44 derivation path.
    pub path: Bip44Path,
    /// Compressed spend public key bytes.
    pub spend_pk: [u8; 32],
    /// Compressed view public key bytes.
    pub view_pk: [u8; 32],
}

/// Serialize entries to canonical format
///
/// # Errors
///
/// Returns error if:
/// - Entry count exceeds u32::MAX (4,294,967,295 entries)
/// - Any spend_pk or view_pk is not exactly 32 bytes
pub fn to_canonical(entries: &[ReceiverCacheEntry]) -> Result<Vec<u8>, CanonicalStateError> {
    if entries.len() > u32::MAX as usize {
        return Err(CanonicalStateError::EntryCountOverflow {
            count: entries.len(),
        });
    }

    let mut buf = Vec::with_capacity(1 + 4 + entries.len() * ENTRY_BYTES);
    buf.push(STATE_VERSION);
    buf.extend_from_slice(&(entries.len() as u32).to_le_bytes());

    for (idx, (path, spend, view)) in entries.iter().enumerate() {
        if spend.len() != 32 {
            return Err(CanonicalStateError::InvalidKeyLength {
                entry_index: idx,
                key_type: "spend_pk",
                actual: spend.len(),
            });
        }
        if view.len() != 32 {
            return Err(CanonicalStateError::InvalidKeyLength {
                entry_index: idx,
                key_type: "view_pk",
                actual: view.len(),
            });
        }

        buf.extend_from_slice(&path_to_bytes(path));
        buf.extend_from_slice(spend);
        buf.extend_from_slice(view);
    }

    Ok(buf)
}

/// Deserialize entries from canonical format
pub fn from_canonical(bytes: &[u8]) -> Result<Vec<ReceiverCacheEntry>, CanonicalStateError> {
    if bytes.len() < 5 {
        return Err(CanonicalStateError::TooShort {
            expected: 5,
            actual: bytes.len(),
        });
    }

    let version = bytes[0];
    if version != STATE_VERSION {
        return Err(CanonicalStateError::InvalidVersion {
            expected: STATE_VERSION,
            actual: version,
        });
    }

    let entry_count = u32::from_le_bytes([bytes[1], bytes[2], bytes[3], bytes[4]]);
    let expected_len = 5 + (entry_count as usize) * ENTRY_BYTES;
    if bytes.len() != expected_len {
        return Err(CanonicalStateError::InvalidLength {
            expected: entry_count,
            expected_bytes: expected_len,
            actual: bytes.len(),
        });
    }

    let mut entries = Vec::with_capacity(entry_count as usize);
    let mut offset = 5;

    for entry_idx in 0..entry_count as usize {
        let path_bytes: [u8; BIP44_PATH_BYTES] = bytes[offset..offset + BIP44_PATH_BYTES]
            .try_into()
            .expect("BUG: slice length mismatch despite length validation");
        let path = path_from_bytes(&path_bytes).map_err(|e| CanonicalStateError::InvalidPath {
            entry_index: entry_idx,
            reason: e,
        })?;
        offset += BIP44_PATH_BYTES;

        let spend_pk = bytes[offset..offset + 32].to_vec();
        offset += 32;

        let view_pk = bytes[offset..offset + 32].to_vec();
        offset += 32;

        entries.push((path, spend_pk, view_pk));
    }

    Ok(entries)
}

/// Convert Bip44Path to canonical 20-byte encoding
///
/// Format: [purpose(4) | asset_type(4) | account(4) | change(4) | index(4)] (little-endian)
fn path_to_bytes(path: &Bip44Path) -> [u8; BIP44_PATH_BYTES] {
    let mut buf = [0u8; BIP44_PATH_BYTES];

    let purpose: u32 = path.purpose().into();
    let asset_type: u32 = path.asset_type().into();
    let account: u32 = path.account().into();
    let change: u32 = path.change().into();
    let index: u32 = path.address_index().into();

    buf[0..4].copy_from_slice(&purpose.to_le_bytes());
    buf[4..8].copy_from_slice(&asset_type.to_le_bytes());
    buf[8..12].copy_from_slice(&account.to_le_bytes());
    buf[12..16].copy_from_slice(&change.to_le_bytes());
    buf[16..20].copy_from_slice(&index.to_le_bytes());

    buf
}

/// Parse Bip44Path from canonical 20-byte encoding
fn path_from_bytes(bytes: &[u8; BIP44_PATH_BYTES]) -> Result<Bip44Path, String> {
    let purpose_raw = u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]);
    let asset_type_raw = u32::from_le_bytes([bytes[4], bytes[5], bytes[6], bytes[7]]);
    let account_raw = u32::from_le_bytes([bytes[8], bytes[9], bytes[10], bytes[11]]);
    let change_raw = u32::from_le_bytes([bytes[12], bytes[13], bytes[14], bytes[15]]);
    let index_raw = u32::from_le_bytes([bytes[16], bytes[17], bytes[18], bytes[19]]);

    let purpose_hardened = (purpose_raw & 0x8000_0000) != 0;
    let purpose_index = purpose_raw & 0x7FFF_FFFF;

    let asset_hardened = (asset_type_raw & 0x8000_0000) != 0;
    let asset_index = asset_type_raw & 0x7FFF_FFFF;

    let account_hardened = (account_raw & 0x8000_0000) != 0;
    let account_index = account_raw & 0x7FFF_FFFF;

    let change_hardened = (change_raw & 0x8000_0000) != 0;
    let change_index = change_raw & 0x7FFF_FFFF;

    let index_hardened = (index_raw & 0x8000_0000) != 0;
    let index_index = index_raw & 0x7FFF_FFFF;

    const MAX_INDEX: u32 = 0x7FFF_FFFF;
    if purpose_index > MAX_INDEX
        || asset_index > MAX_INDEX
        || account_index > MAX_INDEX
        || change_index > MAX_INDEX
        || index_index > MAX_INDEX
    {
        return Err("BIP-44 index out of range: indices must be <= 0x7FFFFFFF".to_string());
    }

    use std::str::FromStr;
    let path_str = format!(
        "m/{}{}/{}{}/{}{}/{}{}/{}{}",
        purpose_index,
        if purpose_hardened { "'" } else { "" },
        asset_index,
        if asset_hardened { "'" } else { "" },
        account_index,
        if account_hardened { "'" } else { "" },
        change_index,
        if change_hardened { "'" } else { "" },
        index_index,
        if index_hardened { "'" } else { "" }
    );

    Bip44Path::from_str(&path_str)
        .map_err(|e| format!("Failed to parse path '{}': {}", path_str, e))
}

#[cfg(test)]
mod tests {
    include!("test_canonical_state_suite.rs");
}
