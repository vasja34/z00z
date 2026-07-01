//! Deterministic compact transaction/output identifier for Z00Z.
//!
//! This helper produces a compact `u64` identifier derived from:
//! - a secret MAC key (not stored in DB)
//! - an output hash (or any 32-byte stable digest)
//!
//! This is useful for RedB-style indexing and correlation without storing raw hashes
//! everywhere.
//!
//! ## Collision Probability
//!
//! With 64-bit output: ~2^-64
//!
//! **Acceptable for:**
//! - Database indexing
//! - Logging
//! - Correlation
//! - Non-cryptographic identification
//!
//! **NOT acceptable for:**
//! - Cryptographic security
//! - Unique identification (use full hash)
//! - Collision-sensitive operations
//!
//! ## Key Management
//!
//! **CRITICAL:** The `mac_key` must be:
//! - **Secret**: Never stored in database or logs
//! - **Stable**: Same key for all derivations in indexing scope
//! - **Generated securely**: Use `generate_mac_key()` or cryptographically secure RNG
//! - **Stored securely**: In wallet encryption, secure memory, or HSM
//!
//! ## Usage Example
//!
//! ```
//! use z00z_wallets::tx::tx_id::{Z00ZTxId, generate_mac_key};
//!
//! // Generate or retrieve MAC key
//! let mac_key = generate_mac_key().unwrap();
//!
//! // Derive TxId
//! let output_hash = [0x42u8; 32];
//! let tx_id = Z00ZTxId::derive(&mac_key, &output_hash).unwrap();
//!
//! // Verify later
//! assert!(tx_id.verify(&mac_key, &output_hash));
//! ```

use z00z_crypto::CryptoError;
use z00z_utils::rng::{RngCoreExt, SystemRngProvider};

/// Domain separation tag for `Z00ZTxId` derivation.
///
/// This constant is part of the on-disk contract.
pub const TX_ID_DOMAIN: &str = "z00z.wallets.tx_id.v1";

/// A compact deterministic identifier.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Z00ZTxId(u64);

impl Z00ZTxId {
    /// Create a new identifier from a raw `u64`.
    pub fn new(value: u64) -> Self {
        Self(value)
    }

    /// Returns the underlying `u64` value.
    pub fn as_u64(self) -> u64 {
        self.0
    }

    /// Derive a deterministic `Z00ZTxId` from `(mac_key, output_hash)`.
    ///
    /// # Security Requirements
    ///
    /// - `mac_key` MUST be secret (never stored in DB/logs)
    /// - `mac_key` MUST be stable (same for all derivations)
    /// - `mac_key` MUST be generated securely
    ///
    /// # Collision Probability
    ///
    /// ~2^-64 (acceptable for indexing, not for security)
    pub fn derive(mac_key: &[u8], output_hash: &[u8; 32]) -> Result<Self, CryptoError> {
        if mac_key.is_empty() {
            return Err(CryptoError::InvalidParameters { param: "mac_key" });
        }

        let hash = crate::domains::hashing::compute_tx_id(mac_key, output_hash);
        let mut first8 = [0u8; 8];
        first8.copy_from_slice(&hash[..8]);
        Ok(Self(u64::from_le_bytes(first8)))
    }

    /// Verify this TxId against the original inputs.
    ///
    /// Returns true if the TxId matches the derived value from mac_key and output_hash.
    ///
    /// # Example
    ///
    /// ```
    /// use z00z_wallets::tx::tx_id::{Z00ZTxId, generate_mac_key};
    ///
    /// let mac_key = generate_mac_key().unwrap();
    /// let output_hash = [0x42u8; 32];
    /// let tx_id = Z00ZTxId::derive(&mac_key, &output_hash).unwrap();
    ///
    /// assert!(tx_id.verify(&mac_key, &output_hash));
    /// assert!(!tx_id.verify(b"wrong", &output_hash));
    /// ```
    pub fn verify(&self, mac_key: &[u8], output_hash: &[u8; 32]) -> bool {
        match Self::derive(mac_key, output_hash) {
            Ok(id) => id == *self,
            Err(_) => false,
        }
    }
}

/// Generate a cryptographically secure MAC key for TxId derivation.
///
/// This is a convenience function that generates a 32-byte random key using
/// the system's cryptographically secure random number generator.
///
/// # Returns
///
/// A 32-byte array suitable for use as a MAC key.
///
/// # Example
///
/// ```
/// use z00z_wallets::tx::tx_id::generate_mac_key;
///
/// let key = generate_mac_key().unwrap();
/// assert_eq!(key.len(), 32);
/// ```
pub fn generate_mac_key() -> Result<[u8; 32], CryptoError> {
    let mut key = [0u8; 32];
    SystemRngProvider.rng().fill_bytes_ext(&mut key);
    Ok(key)
}

impl core::fmt::Display for Z00ZTxId {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_derive_is_deterministic() {
        let key = b"secret";
        let out = [7u8; 32];
        let a = Z00ZTxId::derive(key, &out).unwrap();
        let b = Z00ZTxId::derive(key, &out).unwrap();
        assert_eq!(a, b);
    }

    #[test]
    fn test_derive_changes_key_hash() {
        let out = [7u8; 32];
        let a = Z00ZTxId::derive(b"k1", &out).unwrap();
        let b = Z00ZTxId::derive(b"k2", &out).unwrap();
        assert_ne!(a, b);

        let out2 = [8u8; 32];
        let c = Z00ZTxId::derive(b"k1", &out2).unwrap();
        assert_ne!(a, c);
    }

    #[test]
    fn test_derive_rejects_empty_key() {
        let out = [7u8; 32];
        let err = Z00ZTxId::derive(b"", &out).unwrap_err();
        assert!(matches!(err, CryptoError::InvalidParameters { .. }));
    }

    #[test]
    fn test_verify_works_correctly() {
        let key = b"secret";
        let out = [7u8; 32];
        let tx_id = Z00ZTxId::derive(key, &out).unwrap();

        // Correct inputs
        assert!(tx_id.verify(key, &out));

        // Wrong key
        assert!(!tx_id.verify(b"wrong", &out));

        // Wrong output hash
        assert!(!tx_id.verify(key, &[8u8; 32]));

        // Both wrong
        assert!(!tx_id.verify(b"wrong", &[8u8; 32]));
    }

    #[test]
    fn test_mac_key_32_bytes() {
        let key = generate_mac_key().unwrap();
        assert_eq!(key.len(), 32);
    }

    #[test]
    fn test_generate_mac_is_random() {
        let key1 = generate_mac_key().unwrap();
        let key2 = generate_mac_key().unwrap();
        assert_ne!(key1, key2);
    }

    #[test]
    fn test_mac_key_tx_id() {
        let key = generate_mac_key().unwrap();
        let out = [0x42u8; 32];
        let tx_id = Z00ZTxId::derive(&key, &out).unwrap();
        assert!(tx_id.verify(&key, &out));
    }

    #[test]
    fn test_different_keys_different_ids() {
        let out = [7u8; 32];
        let id1 = Z00ZTxId::derive(b"key1", &out).unwrap();
        let id2 = Z00ZTxId::derive(b"key2", &out).unwrap();
        assert_ne!(id1, id2);
    }

    #[test]
    fn test_different_outputs_different_ids() {
        let key = b"secret";
        let id1 = Z00ZTxId::derive(key, &[1u8; 32]).unwrap();
        let id2 = Z00ZTxId::derive(key, &[2u8; 32]).unwrap();
        assert_ne!(id1, id2);
    }

    #[test]
    fn test_new_and_u64_work() {
        let value = 12345u64;
        let tx_id = Z00ZTxId::new(value);
        assert_eq!(tx_id.as_u64(), value);
    }

    #[test]
    fn test_display_formats_correctly() {
        let tx_id = Z00ZTxId::new(12345);
        let formatted = format!("{}", tx_id);
        assert_eq!(formatted, "12345");
    }

    #[test]
    fn test_ordering_works() {
        let id1 = Z00ZTxId::new(100);
        let id2 = Z00ZTxId::new(200);
        assert!(id1 < id2);
        assert!(id2 > id1);
    }

    #[test]
    fn test_clone_and_copy_work() {
        let id1 = Z00ZTxId::new(123);
        let id2 = id1;
        let id3 = id1;

        assert_eq!(id1, id2);
        assert_eq!(id1, id3);
    }

    #[test]
    fn test_hash_trait_works() {
        use std::collections::HashSet;

        let mut set = HashSet::new();
        let id1 = Z00ZTxId::new(100);
        let id2 = Z00ZTxId::new(100);
        let id3 = Z00ZTxId::new(200);

        set.insert(id1);
        set.insert(id2);
        set.insert(id3);

        // Should have 2 unique values
        assert_eq!(set.len(), 2);
    }

    #[test]
    fn test_constants() {
        assert_eq!(TX_ID_DOMAIN, "z00z.wallets.tx_id.v1");
    }

    #[test]
    fn test_edge_case_max_u64() {
        let min = Z00ZTxId::new(0);
        let max = Z00ZTxId::new(u64::MAX);

        assert_eq!(min.as_u64(), 0);
        assert_eq!(max.as_u64(), u64::MAX);
    }

    #[test]
    fn test_roundtrip_derive_verify() {
        let key = generate_mac_key().unwrap();
        let out = [0x55u8; 32];

        let tx_id = Z00ZTxId::derive(&key, &out).unwrap();
        let hex = format!("{:x}", tx_id.as_u64());

        // Parse back
        let parsed = u64::from_str_radix(&hex, 16).unwrap();
        let reconstructed = Z00ZTxId::new(parsed);

        assert_eq!(tx_id, reconstructed);
        assert!(reconstructed.verify(&key, &out));
    }
}
