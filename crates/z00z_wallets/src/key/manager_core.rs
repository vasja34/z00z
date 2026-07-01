//! Key manager implementation for Z00Z wallets.
//!
//! This module provides:
//! - Serializable state types for persistence
//! - KeyManager trait and KeyManagerImpl for key derivation
//!
//! # Invariants
//!
//! The following invariants MUST hold for all instances of `KeyManagerImpl`:
//!
//! ## Invariant 1: Master Key Consistency
//! - If `master_key` is `Some`, it MUST be derivable from `encrypted_seed` (if present)
//! - If `encrypted_seed` is `Some`, `master_key` MUST be present after successful decryption
//! - After `init_from_seed()`, `encrypted_seed` is `None` and `master_key` is `Some`
//! - After `init_from_encrypted_seed()`, both `encrypted_seed` and `master_key` are `Some`
//! - After `clear()`, both `encrypted_seed` and `master_key` are `None`
//!
//! ## Invariant 2: Derived Key Cache Validity
//! - All entries in `derived_public_keys` MUST be derivable from current `master_key`
//! - Cache MUST be cleared when `master_key` changes or is cleared
//! - Cache entries MUST match the result of re-deriving the same path
//!
//! ## Invariant 3: Key Lifecycle
//! - Secret keys are NEVER stored in cache (only public keys)
//! - Secret keys are derived on-demand and immediately consumed/zeroized
//! - Master key persists only until `clear()` or next `init_*()`
//! - Encrypted seed persists for persistence round-trips
//!
//! ## Invariant 4: Thread Safety
//! - `derived_public_keys` cache is protected by `RwLock`
//! - Multiple concurrent reads are safe
//! - Write operations (init, clear, derive) acquire exclusive locks
//!
//! ## Invariant 5: Zeroization
//! - Master key is automatically zeroized on drop (via `Hidden<T>`)
//! - Intermediate secret keys are zeroized immediately after use
//! - `clear()` ensures all sensitive material is zeroized
//!
//! # Constant-Time Comparisons
//!
//! All comparisons of secret material MUST use constant-time operations to prevent timing attacks.
//! Use `subtle::ConstantTimeEq` (`ct_eq`), NOT the `==` operator.
//!
//! # Privacy
//!
//! BIP-44 derivation paths are NOT logged in production to prevent metadata leaks.
//! Paths reveal account/change/index usage patterns which can compromise user privacy.
//!
//! Enable `verbose-logging` feature ONLY for debugging in development/testing environments.
//! **NEVER enable verbose-logging in production builds.**
//!
//! These invariants are enforced at boundaries (constructors, init methods, clear)
//! via `debug_assert!` checks in debug builds.

use std::num::NonZeroUsize;
#[cfg(not(target_arch = "wasm32"))]
use std::path::Path;
use std::sync::atomic::AtomicUsize;
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::{Arc, RwLock, RwLockReadGuard, RwLockWriteGuard};

use lru::LruCache;
use serde::{Deserialize, Serialize};
use subtle::{Choice, ConstantTimeEq};
use thiserror::Error;
use zeroize::Zeroizing;

use z00z_crypto::expert::encoding::ByteArray;
use z00z_crypto::expert::keys::{RistrettoPublicKey, RistrettoSecretKey};
use z00z_crypto::expert::traits::{PublicKeyTrait, SecretKeyTrait};
use z00z_crypto::{DomainHasher, KernelSignature};
use z00z_utils::logger::{Logger, NoopLogger};
use z00z_utils::metrics::{MetricsSink, NoopMetrics};
use z00z_utils::rng::{SecureRngProvider, SystemRngProvider};
use z00z_utils::time::{SystemTimeProvider, TimeProvider};

use crate::domains::hashing::{compute_schnorr_challenge, ChallengeSize};
use crate::key::{
    Bip39Seed64, Bip44KeyManager, Bip44Path, CipherSeedContainer, RistrettoBridge, Z00Z_BIP44_ASSET,
};
use z00z_core::genesis::ChainType;

#[cfg(not(test))]
use crate::domains::WalletSignNonceProdDomain;
#[cfg(test)]
use crate::domains::WalletSignNonceTestDomain;

#[cfg(not(target_arch = "wasm32"))]
use crate::{
    db::open_wallet_store, db::WalletIdentity, rpc::types::common::PersistWalletId,
    wallet::errors::WalletError,
};

/// Maximum number of derived public keys to cache.
/// This prevents memory exhaustion attacks from unbounded cache growth.
pub const MAX_DERIVED_PUBKEY_CACHE: usize = 256;

/// Time-to-live for cached public keys in seconds.
/// Keys older than this will be evicted on next access.
pub const DERIVED_KEY_TTL_SECONDS: u64 = 1800;

/// Validate 1 random cache entry every N derives (production safety check).
/// This provides periodic integrity verification without significant performance impact.
pub const CACHE_SPOT_CHECK_FREQUENCY: u32 = 100;

/// Maximum gap limit for BIP-44 address derivation.
/// BIP-44 standard specifies a gap limit of 20 unused addresses.
pub const BIP44_GAP_LIMIT: u32 = 20;

include!("manager_cache.rs");
include!("manager_state.rs");

// ============================================================================
// KEY MANAGER TRAIT & IMPLEMENTATION
// ============================================================================

// ============================================================================
// KEY MANAGER TRAIT
// ============================================================================

#[derive(Debug, Error)]
#[non_exhaustive]
/// Key manager errors.
#[derive(Clone)]
pub enum KeyManagerError {
    /// Master key not initialized.
    #[error("Master key not initialized - wallet must be unlocked")]
    NotInitialized,
    /// Key not derived for a requested path.
    #[error("Key not derived for path")]
    KeyNotDerived,
    /// Failed to generate a signature.
    #[error("Failed to generate signature")]
    SignatureFailed,
    /// Failed to derive a key.
    #[error("Failed to derive key")]
    DerivationFailed,
    /// Failed to derive a key with additional context.
    #[error("Failed to derive key: {reason}")]
    DerivationFailedWithReason {
        /// Underlying derivation failure reason.
        reason: String,
    },
    /// Derived public key is invalid (identity point).
    #[error("Derived invalid public key (identity point)")]
    InvalidPublicKey,
    /// Invalid input parameters.
    #[error("Invalid parameters")]
    InvalidParameters,
    /// Authentication failure.
    #[error("Authentication failed")]
    AuthenticationFailed,
    /// Storage operation failed.
    #[error("Storage operation failed")]
    StorageFailed,
    /// Underlying cryptographic error.
    #[error("Cryptographic operation failed")]
    Crypto(#[from] z00z_crypto::CryptoError),
    /// A lock was poisoned by a panic while held.
    #[error("Lock poisoned: {lock}")]
    LockPoisoned {
        /// Name of the poisoned lock.
        lock: &'static str,
    },
    /// Key manager state is corrupted or inconsistent.
    #[error("Key manager state is corrupted or inconsistent")]
    StateCorrupted,
    /// Derived key cache corruption detected.
    #[error("Key manager key cache corrupted")]
    CacheCorrupted,
    /// BIP-44 gap limit exceeded.
    #[error("BIP-44 gap limit exceeded: {gap} unused addresses")]
    GapLimitExceeded {
        /// Number of unused addresses currently derived ahead of the last used index.
        gap: u32,
    },
    /// Seed has weak entropy (fails validation).
    #[error("Weak entropy: {0}")]
    WeakEntropy(String),
}

#[cfg(not(target_arch = "wasm32"))]
fn map_storage_unlock_error(err: WalletError) -> KeyManagerError {
    match err {
        WalletError::InvalidPassword => KeyManagerError::AuthenticationFailed,
        _ => KeyManagerError::StorageFailed,
    }
}

/// Key manager result type.
pub type Result<T> = std::result::Result<T, KeyManagerError>;

/// Wallet key derivation and signing interface.
pub trait KeyManager {
    /// Clear all secret material and derived keys.
    fn clear(&mut self);
    /// Derive the public key for a specific BIP-44 path.
    fn derive_key(&self, path: &Bip44Path) -> Result<RistrettoPublicKey>;
    /// Get a previously-derived public key (if cached).
    fn get_public_key(&self, path: &Bip44Path) -> Option<RistrettoPublicKey>;

    /// Derive a secret key transiently.
    ///
    /// The returned key is wrapped in `Zeroizing` and is intended for immediate use only.
    fn derive_secret_transient(&self, path: &Bip44Path) -> Result<Zeroizing<RistrettoSecretKey>>;

    /// Create a Schnorr signature for a message using the key at `path`.
    fn sign(&self, path: &Bip44Path, msg: &[u8]) -> Result<KernelSignature>;
}

include!("manager_impl.rs");
