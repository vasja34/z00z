//! RNG provider trait definitions

use rand::{CryptoRng, RngCore};

/// Unpredictable cryptographic randomness (production use)
///
/// This trait represents truly random, unpredictable RNG sources suitable
/// for cryptographic operations requiring unpredictability (nonces, keys, salts).
///
/// Only `SystemRngProvider` implements this trait, ensuring clear distinction
/// from deterministic RNGs.
///
/// # Examples
///
/// ```no_run
/// use z00z_utils::rng::{SecureRngProvider, SystemRngProvider};
/// use rand::RngCore;
///
/// let provider = SystemRngProvider;
/// let mut rng = provider.rng();
/// let mut nonce = [0u8; 32];
/// rng.fill_bytes(&mut nonce);
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
pub trait SecureRngProvider: Send + Sync {
    /// Type of RNG this provider produces (must be cryptographically secure)
    type Rng: RngCore + CryptoRng + Send;

    /// Get a new RNG instance
    fn rng(&self) -> Self::Rng;
}

/// Deterministic reproducibility (approved genesis/testing only)
///
/// ⚠️ **SECURITY WARNING:** This RNG is deterministic and MUST NOT be used for:
/// - Nonces (must be unpredictable)
/// - Ephemeral secrets (session keys, IVs)
/// - Salts (must be unique and unpredictable)
///
/// ✅ **Approved use cases:**
/// - Genesis block generation (reproducible initial state)
/// - Test fixtures (deterministic test vectors)
/// - Deterministic identifiers where predictability is acceptable
/// - Explicit simulator reproducibility flows that are already scoped away from
///   production entropy claims
///
/// # Examples
///
/// ```
/// # #[cfg(any(feature = "deterministic-rng", feature = "test-utils", feature = "test-params-fast"))]
/// # {
/// use z00z_utils::rng::DeterministicRngProvider;
/// use rand::RngCore;
///
/// let provider = DeterministicRngProvider::from_seed([42u8; 32]);
/// let mut rng = provider.rng();
/// let mut bytes = [0u8; 32];
/// rng.fill_bytes(&mut bytes);
/// # }
/// ```
pub trait DeterministicRngSource: Send + Sync {
    /// Type of RNG this provider produces for reproducibility-only flows.
    type Rng: RngCore + Send;

    /// Get a new RNG instance for a reproducibility-only caller.
    fn rng(&self) -> Self::Rng;
}
