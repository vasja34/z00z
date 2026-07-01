//! DeterministicRngProvider - ChaCha20-based reproducibility RNG
//!
//! ⚠️ **SECURITY WARNING:** This RNG is deterministic and MUST NOT be used for:
//! - Nonces (must be unpredictable)
//! - Ephemeral secrets (session keys, IVs)
//! - Salts (must be unique and unpredictable)
//! - Any value where unpredictability is required
//!
//! ✅ **Approved use cases:**
//! - Genesis block generation (reproducible initial state)
//! - Test fixtures (deterministic test vectors)
//! - Deterministic identifiers where predictability is acceptable

#[cfg(not(test))]
use rand::SeedableRng;
use zeroize::ZeroizeOnDrop;

#[cfg(test)]
use rand::{RngCore, SeedableRng};
use rand_chacha::ChaCha20Rng;

/// Deterministic RNG provider using ChaCha20 stream cipher
///
/// This provider produces deterministic random numbers from a 32-byte seed.
/// Uses ChaCha20 as a stable reproducibility backend for approved deterministic domains.
///
/// **Approved use cases:**
/// - Genesis state generation (MUST be identical across all nodes)
/// - Reproducible tests
/// - Deterministic asset creation
/// - Other explicitly approved reproducibility flows
///
/// This provider is not a universal secure-entropy abstraction. Callers must
/// keep it confined to reproducibility-scoped domains instead of treating the
/// deterministic seed as production randomness.
///
/// # Availability
///
/// This provider is compiled only for explicit reproducibility domains:
/// - `deterministic-rng`
/// - `test-utils`
/// - `test-params-fast`
/// - crate-local unit tests
///
/// # Reproducibility
///
/// - Same seed ALWAYS produces same sequence (deterministic)
/// - 256-bit seed provides a large reproducibility space (2^256 possible sequences)
///
/// # Examples
///
/// ```
/// use z00z_utils::rng::{DeterministicRngProvider};
/// use rand::RngCore;
///
/// // Approved genesis: seed from config
/// let seed = [42u8; 32];
/// let provider = DeterministicRngProvider::from_seed(seed);
/// let mut rng = provider.rng();
///
/// let mut bytes = [0u8; 32];
/// rng.fill_bytes(&mut bytes);
/// ```
///
/// # Genesis Integration
///
/// ```rust,no_run
/// use z00z_utils::rng::{DeterministicRngProvider};
///
/// fn generate_genesis_asset(seed: &[u8; 32], serial_id: u32) {
///     let provider = DeterministicRngProvider::from_seed(*seed);
///     let mut rng = provider.rng();
///     // Use rng for deterministic asset generation
/// }
/// ```
#[derive(Clone, ZeroizeOnDrop)]
pub struct DeterministicRngProvider {
    seed: [u8; 32],
}

impl DeterministicRngProvider {
    /// Create a new deterministic RNG provider from 32-byte seed
    ///
    /// # Arguments
    ///
    /// * `seed` - 32-byte seed for ChaCha20 initialization
    ///
    /// # Returns
    ///
    /// Provider that will always generate the same RNG sequence for given seed
    ///
    /// # Example
    ///
    /// ```
    /// use z00z_utils::rng::DeterministicRngProvider;
    ///
    /// let seed = [1u8; 32];
    /// let provider = DeterministicRngProvider::from_seed(seed);
    /// ```
    pub fn from_seed(seed: [u8; 32]) -> Self {
        Self { seed }
    }

    /// Create a new deterministic RNG instance for an approved reproducibility flow.
    pub fn rng(&self) -> ChaCha20Rng {
        <Self as super::traits::DeterministicRngSource>::rng(self)
    }

    /// Get the seed used by this provider (test-only)
    #[cfg(any(test, feature = "test-utils"))]
    pub fn seed(&self) -> &[u8; 32] {
        &self.seed
    }
}

impl std::fmt::Debug for DeterministicRngProvider {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DeterministicRngProvider")
            .field("seed", &"<redacted>")
            .finish()
    }
}

impl super::traits::DeterministicRngSource for DeterministicRngProvider {
    type Rng = ChaCha20Rng;

    fn rng(&self) -> Self::Rng {
        ChaCha20Rng::from_seed(self.seed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deterministic_rng_seed_output() {
        let seed = [42u8; 32];
        let provider1 = DeterministicRngProvider::from_seed(seed);
        let provider2 = DeterministicRngProvider::from_seed(seed);

        let mut rng1 = provider1.rng();
        let mut rng2 = provider2.rng();

        let val1 = rng1.next_u64();
        let val2 = rng2.next_u64();

        assert_eq!(val1, val2, "Same seed must produce same output");
    }

    #[test]
    fn test_deterministic_rng_different_seeds() {
        let seed1 = [1u8; 32];
        let seed2 = [2u8; 32];
        let provider1 = DeterministicRngProvider::from_seed(seed1);
        let provider2 = DeterministicRngProvider::from_seed(seed2);

        let mut rng1 = provider1.rng();
        let mut rng2 = provider2.rng();

        let val1 = rng1.next_u64();
        let val2 = rng2.next_u64();

        assert_ne!(val1, val2, "Different seeds must produce different output");
    }

    #[test]
    fn test_deterministic_rng_sequence() {
        let seed = [123u8; 32];
        let provider = DeterministicRngProvider::from_seed(seed);

        let mut rng1 = provider.rng();
        let seq1: Vec<u64> = (0..10).map(|_| rng1.next_u64()).collect();

        let mut rng2 = provider.rng();
        let seq2: Vec<u64> = (0..10).map(|_| rng2.next_u64()).collect();

        assert_eq!(seq1, seq2, "Same seed must produce identical sequence");
    }

    #[test]
    fn test_seed_accessor() {
        let seed = [99u8; 32];
        let provider = DeterministicRngProvider::from_seed(seed);
        assert_eq!(provider.seed(), &seed);
    }
}
