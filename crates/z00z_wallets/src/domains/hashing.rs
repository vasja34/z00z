//! Unified cryptographic hashing module with domain separation.
//!
//! This module provides a single source of truth for all hashing operations
//! in Z00Z wallets. All cryptographic hashes MUST use domain-separated hashers
//! with length framing to prevent ambiguous concatenation.
//!
//! # MAC Security
//!
//! Index MACs use HMAC-SHA256 for tamper detection.
//! - Key: 32-byte derived wallet `INDEX_KEY`
//! - Output: 32-byte authentication tag
//!
//! CRITICAL: Always use `verify_index_mac()` for comparison.
//! DO NOT use `==` operator - it can introduce timing side-channels.

use blake2::Digest;
use z00z_crypto::{frame_bytes, frame_str, frame_u32_le, DomainHasher, DomainHasher256};

use crate::domains::{
    EncryptionChecksumDomain, IndexMacDomain, PasswordBloomDomain, PayRefDomain,
    RedbWalletDataKeyDomain, RedbWalletIndexKeyDomain, RedbWalletIntegrityKeyDomain,
    SchnorrChallengeDomain, SnapshotChecksumDomain, TxHashDomain, TxIdDomain, WalletBIP44Domain,
    WalletBlindingDomain, WalletChangeDomain, WalletFileIdDomain, WalletFingerprintDomain,
    WalletIntegrityDomain, WalletMasterKeyDomain, WalletPaymentDomain, WalletSeedSaltDomain,
    WalletSessionDomain,
};

// ============================================================================
// TYPE ALIASES FOR CONVENIENCE
// ============================================================================

/// 512-bit domain-separated hasher for key derivation
pub type WalletMasterKeyHasher = DomainHasher<WalletMasterKeyDomain>;
/// 512-bit domain-separated hasher for payment key derivation
pub type PaymentKeyHasher = DomainHasher<WalletPaymentDomain>;
/// 512-bit domain-separated hasher for change key derivation
pub type ChangeKeyHasher = DomainHasher<WalletChangeDomain>;
/// 512-bit domain-separated hasher for BIP-44 key derivation
pub type Bip44Hasher = DomainHasher<WalletBIP44Domain>;
/// 512-bit domain-separated hasher for session key derivation
pub type SessionKeyHasher = DomainHasher<WalletSessionDomain>;
/// 512-bit domain-separated hasher for blinding key derivation
pub type BlindingKeyHasher = DomainHasher<WalletBlindingDomain>;

/// 512-bit domain-separated hasher for transaction IDs
pub type TxIdHasher = DomainHasher<TxIdDomain>;
/// 512-bit domain-separated hasher for transaction hashes
pub type TxHashHasher = DomainHasher<TxHashDomain>;
/// 512-bit domain-separated hasher for Schnorr signature challenges
pub type SchnorrChallengeHasher = DomainHasher<SchnorrChallengeDomain>;

/// Output size selector for Schnorr challenge hashing.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChallengeSize {
    /// 256-bit (32-byte) challenge output
    B256,
    /// 512-bit (64-byte) challenge output
    B512,
}

/// Tagged challenge bytes produced by the Schnorr challenge hasher.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChallengeBytes {
    /// 256-bit challenge
    B256([u8; 32]),
    /// 512-bit challenge
    B512([u8; 64]),
}

impl ChallengeBytes {
    /// Extract a 256-bit challenge, returning `None` for 512-bit variants.
    pub fn into_b256(self) -> Option<[u8; 32]> {
        match self {
            ChallengeBytes::B256(bytes) => Some(bytes),
            ChallengeBytes::B512(_) => None,
        }
    }

    /// Extract a 512-bit challenge, returning `None` for 256-bit variants.
    pub fn into_b512(self) -> Option<[u8; 64]> {
        match self {
            ChallengeBytes::B256(_) => None,
            ChallengeBytes::B512(bytes) => Some(bytes),
        }
    }

    /// View the challenge bytes as a slice regardless of variant.
    pub fn as_slice(&self) -> &[u8] {
        match self {
            ChallengeBytes::B256(bytes) => bytes.as_slice(),
            ChallengeBytes::B512(bytes) => bytes.as_slice(),
        }
    }
}

/// 256-bit domain-separated hasher for encryption checksums
pub type EncryptionChecksumHasher = DomainHasher256<EncryptionChecksumDomain>;
/// 256-bit domain-separated hasher for password bloom filters
pub type PasswordBloomHasher = DomainHasher256<PasswordBloomDomain>;
/// 256-bit domain-separated HMAC hasher for database index MACs
pub type IndexMacHasher = DomainHasher256<IndexMacDomain>;
/// 256-bit domain-separated hasher for receiver-cache state checksums.
pub type SnapshotChecksumHasher = DomainHasher256<SnapshotChecksumDomain>;
/// 256-bit domain-separated hasher for wallet file integrity tags
pub type WalletIntegrityHasher = DomainHasher256<WalletIntegrityDomain>;

/// 256-bit domain-separated hasher for wallet file IDs
pub type WalletFileIdHasher = DomainHasher256<WalletFileIdDomain>;
/// 256-bit domain-separated hasher for wallet seed encryption salts
pub type WalletSeedSaltHasher = DomainHasher256<WalletSeedSaltDomain>;
/// 256-bit domain-separated hasher for RPC fingerprints
pub type WalletFingerprintHasher = DomainHasher256<WalletFingerprintDomain>;
/// 256-bit domain-separated hasher for payment references
pub type PayRefHasher = DomainHasher256<PayRefDomain>;

/// 256-bit domain-separated hasher for RedB wallet data keys
pub type RedbWalletDataKeyHasher = DomainHasher256<RedbWalletDataKeyDomain>;
/// 256-bit domain-separated hasher for RedB wallet index keys
pub type RedbWalletIndexKeyHasher = DomainHasher256<RedbWalletIndexKeyDomain>;
/// 256-bit domain-separated hasher for RedB wallet integrity keys
pub type RedbWalletIntegrityKeyHasher = DomainHasher256<RedbWalletIntegrityKeyDomain>;

// ============================================================================
// CANONICALIZATION HELPERS
// ============================================================================

/// Canonicalize string input before hashing.
///
/// # Operations
/// - Trim whitespace
/// - Normalize Unicode (NFC)
/// - Replace tabs/newlines with spaces
/// - Convert to lowercase
pub fn canonicalize_string(input: &str) -> Vec<u8> {
    use unicode_normalization::UnicodeNormalization;

    let normalized_whitespace = input
        .chars()
        .map(|c| if c.is_whitespace() { ' ' } else { c })
        .collect::<String>();

    normalized_whitespace
        .trim()
        .nfc()
        .collect::<String>()
        .to_lowercase()
        .into_bytes()
}

/// Canonicalize byte input (ensure consistent representation).
pub fn canonicalize_bytes(input: &[u8]) -> Vec<u8> {
    input.to_vec()
}

/// Canonicalize integer input (little-endian).
pub fn canonicalize_u32(value: u32) -> [u8; 4] {
    value.to_le_bytes()
}

/// Canonicalize u64 input (little-endian).
pub fn canonicalize_u64(value: u64) -> [u8; 8] {
    value.to_le_bytes()
}

// ============================================================================
// STANDARDIZED HASHING FUNCTIONS
// ============================================================================

/// Compute wallet file ID from wallet_id (used for deterministic filenames).
pub fn compute_wallet_file_id(wallet_id: &str) -> [u8; 32] {
    let hash = WalletFileIdHasher::new_with_label("file_id")
        .chain_update(frame_str(wallet_id))
        .finalize();
    let mut result = [0u8; 32];
    result.copy_from_slice(&hash.as_ref()[..32]);
    result
}

/// Compute seed-phrase encryption salt from wallet_id.
pub fn compute_seed_salt(wallet_id: &str) -> [u8; 16] {
    let hash = WalletSeedSaltHasher::new_with_label("seed_salt")
        .chain_update(frame_str(wallet_id))
        .finalize();
    let mut out = [0u8; 16];
    out.copy_from_slice(&hash.as_ref()[..16]);
    out
}

/// Compute wallet seed hash through crypto hash policy module.
pub fn compute_wallet_seed_hash(seed: &[u8]) -> z00z_crypto::hash_policy::WalletHash {
    z00z_crypto::hash_policy::compute_wallet_seed_hash(seed)
}

/// Compute RPC fingerprint from string value.
pub fn compute_fingerprint(value: &str) -> [u8; 32] {
    let hash = WalletFingerprintHasher::new_with_label("fingerprint")
        .chain_update(frame_str(value))
        .finalize();
    let mut result = [0u8; 32];
    result.copy_from_slice(&hash.as_ref()[..32]);
    result
}

/// Compute password bloom filter hash.
pub fn compute_password_bloom(prefix: u8, input: &[u8]) -> [u8; 16] {
    let hash = PasswordBloomHasher::new_with_label("bloom")
        .chain_update([prefix])
        .chain_update(frame_bytes(input))
        .finalize();
    let mut out = [0u8; 16];
    out.copy_from_slice(&hash.as_ref()[..16]);
    out
}

/// Compute index MAC for database keys.
pub fn compute_index_mac(key: &[u8; 32], msg: &[u8]) -> [u8; 32] {
    z00z_crypto::hash::hmac_sha256_raw(key, msg)
}

/// Constant-time MAC verification (prevents timing attacks).
///
/// Returns true if `computed` matches `expected`.
pub fn verify_index_mac(computed: &[u8; 32], expected: &[u8; 32]) -> bool {
    use subtle::ConstantTimeEq;
    computed.ct_eq(expected).into()
}

/// Compute snapshot checksum.
pub fn compute_snapshot_checksum(fields: &[u8]) -> [u8; 32] {
    let hash = SnapshotChecksumHasher::new_with_label("snapshot")
        .chain_update(frame_bytes(fields))
        .finalize();
    let mut result = [0u8; 32];
    result.copy_from_slice(&hash.as_ref()[..32]);
    result
}

/// Compute wallet integrity hash.
pub fn compute_wallet_integrity(bytes: &[u8]) -> [u8; 32] {
    let hash = WalletIntegrityHasher::new_with_label("integrity")
        .chain_update(frame_bytes(bytes))
        .finalize();
    let mut result = [0u8; 32];
    result.copy_from_slice(&hash.as_ref()[..32]);
    result
}

/// Compute PayRef from block hash and output hash.
pub fn compute_pay_ref(block_hash: &[u8; 32], output_hash: &[u8; 32]) -> [u8; 32] {
    let hash = PayRefHasher::new_with_label("pay_ref")
        .chain_update(frame_bytes(block_hash))
        .chain_update(frame_bytes(output_hash))
        .finalize();
    let mut result = [0u8; 32];
    result.copy_from_slice(&hash.as_ref()[..32]);
    result
}

/// Compute transaction ID from mac_key and output_hash.
pub fn compute_tx_id(mac_key: &[u8], output_hash: &[u8; 32]) -> [u8; 32] {
    let hash = TxIdHasher::new_with_label("tx_id")
        .chain_update(frame_bytes(mac_key))
        .chain_update(frame_bytes(output_hash))
        .finalize();
    let mut result = [0u8; 32];
    result.copy_from_slice(&hash.as_ref()[..32]);
    result
}

/// Compute Schnorr challenge for transaction signing.
pub fn compute_schnorr_challenge(
    nonce: &[u8; 32],
    pubkey: &[u8; 32],
    msg: &[u8],
    size: ChallengeSize,
) -> ChallengeBytes {
    let hash = SchnorrChallengeHasher::new_with_label("challenge")
        .chain_update(frame_bytes(nonce))
        .chain_update(frame_bytes(pubkey))
        .chain_update(frame_bytes(msg))
        .finalize();

    match size {
        ChallengeSize::B256 => {
            let mut result = [0u8; 32];
            result.copy_from_slice(&hash.as_ref()[..32]);
            ChallengeBytes::B256(result)
        }
        ChallengeSize::B512 => {
            let mut result = [0u8; 64];
            result.copy_from_slice(&hash.as_ref()[..64]);
            ChallengeBytes::B512(result)
        }
    }
}

/// Derive wallet master key from seed.
pub fn derive_wallet_master_key(seed: &[u8]) -> [u8; 32] {
    let hash = WalletMasterKeyHasher::new_with_label("master_key")
        .chain_update(frame_bytes(seed))
        .finalize();
    let mut result = [0u8; 32];
    result.copy_from_slice(&hash.as_ref()[..32]);
    result
}

/// Derive payment key from master key and index.
pub fn derive_payment_key(master_key: &[u8; 32], index: u32) -> [u8; 32] {
    let hash = PaymentKeyHasher::new_with_label("payment")
        .chain_update(frame_bytes(master_key))
        .chain_update(frame_u32_le(index))
        .finalize();
    let mut result = [0u8; 32];
    result.copy_from_slice(&hash.as_ref()[..32]);
    result
}

/// Derive change key from master key and index.
pub fn derive_change_key(master_key: &[u8; 32], index: u32) -> [u8; 32] {
    let hash = ChangeKeyHasher::new_with_label("change")
        .chain_update(frame_bytes(master_key))
        .chain_update(frame_u32_le(index))
        .finalize();
    let mut result = [0u8; 32];
    result.copy_from_slice(&hash.as_ref()[..32]);
    result
}

/// Compute encryption checksum for plaintext.
pub fn compute_encryption_checksum(plaintext: &[u8]) -> [u8; 32] {
    let hash = EncryptionChecksumHasher::new_with_label("checksum")
        .chain_update(frame_bytes(plaintext))
        .finalize();
    let mut result = [0u8; 32];
    result.copy_from_slice(&hash.as_ref()[..32]);
    result
}

/// Compute HKDF info for RedB wallet data key.
pub fn redb_wallet_hkdf_info_data() -> [u8; 32] {
    let hash = RedbWalletDataKeyHasher::new_with_label("hkdf_info")
        .chain_update(b"data_key.v2")
        .finalize();
    let mut result = [0u8; 32];
    result.copy_from_slice(&hash.as_ref()[..32]);
    result
}

/// Compute HKDF info for RedB wallet index key.
pub fn redb_wallet_hkdf_info_index() -> [u8; 32] {
    let hash = RedbWalletIndexKeyHasher::new_with_label("hkdf_info")
        .chain_update(b"index_key.v2")
        .finalize();
    let mut result = [0u8; 32];
    result.copy_from_slice(&hash.as_ref()[..32]);
    result
}

/// Compute HKDF info for RedB wallet integrity key.
pub fn redb_wallet_hkdf_info_integrity() -> [u8; 32] {
    let hash = RedbWalletIntegrityKeyHasher::new_with_label("hkdf_info")
        .chain_update(b"integrity_key.v2")
        .finalize();
    let mut result = [0u8; 32];
    result.copy_from_slice(&hash.as_ref()[..32]);
    result
}

#[cfg(test)]
#[path = "test_hashing.rs"]
mod tests;
