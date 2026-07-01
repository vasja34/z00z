//! HKDF Info Field Constants
//!
//! All HKDF derivations MUST use non-empty `info` for domain separation.
//!
//! # Format
//!
//! `"z00z.<component>.<purpose>.<version>"`
//!
//! Version suffix (`.v1`, `.v2`, ...) allows future changes (algorithm, output length,
//! or semantic boundaries) without collisions.
//!
//! # Usage Example
//!
//! ```rust
//! use z00z_crypto::kdf::hkdf_expand_32;
//! use z00z_crypto::kdf::kdf_domains;
//!
//! # let master_key = [0xAA; 32];
//! // ✅ CORRECT: Use predefined constants
//! let wallet_key = hkdf_expand_32(&master_key, &[], kdf_domains::HKDF_INFO_WALLET_KEY)?;
//! let backup_key = hkdf_expand_32(&master_key, &[], kdf_domains::HKDF_INFO_BACKUP)?;
//!
//! // ❌ WRONG: Hardcoded strings (error-prone, no compile-time validation)
//! // let key = hkdf_expand_32(&master_key, &[], b"z00z.wallet.key.v1")?;
//! # Ok::<_, z00z_crypto::kdf::KdfError>(())
//! ```

/// Wallet master key derivation.
pub const HKDF_INFO_WALLET_KEY: &[u8] = b"z00z.wallet.key.v1";

/// Wallet encryption key derivation.
pub const HKDF_INFO_WALLET_ENCRYPTION: &[u8] = b"z00z.wallet.encryption.v1";

/// Transaction signing key derivation.
pub const HKDF_INFO_TX_SIGNING: &[u8] = b"z00z.tx.signing.v1";

/// Backup key derivation.
pub const HKDF_INFO_BACKUP: &[u8] = b"z00z.backup.key.v1";

/// OnionNet session key derivation.
pub const HKDF_INFO_ONIONNET_SESSION: &[u8] = b"z00z.onionnet.session.v1";

/// Asset blinding factor derivation.
pub const HKDF_INFO_ASSET_BLINDING: &[u8] = b"z00z.asset.blinding.v1";

/// RedB wallet DATA key derivation (v2 info scheme).
///
/// This is NOT used directly as HKDF `info` in wallet storage; it is provided for
/// cross-component consistency when needed.
pub const HKDF_INFO_REDB_DATA: &[u8] = b"z00z.wallet.redb.data.v2";

/// RedB wallet INDEX key derivation (v2 info scheme).
pub const HKDF_INFO_REDB_INDEX: &[u8] = b"z00z.wallet.redb.index.v2";

/// RedB wallet INTEGRITY key derivation (v2 info scheme).
pub const HKDF_INFO_REDB_INTEGRITY: &[u8] = b"z00z.wallet.redb.integrity.v2";
