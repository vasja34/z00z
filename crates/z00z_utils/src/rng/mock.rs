//! MockRngProvider - deterministic RNG for testing

#[cfg(all(
    not(test),
    not(debug_assertions),
    not(feature = "deterministic-rng"),
    not(feature = "test-utils"),
    not(feature = "test-params-fast")
))]
compile_error!("MockRngProvider MUST NOT be compiled in production builds");

use rand::rngs::StdRng;
#[cfg(not(test))]
use rand::SeedableRng;
use sha2::{Digest, Sha256};
use zeroize::ZeroizeOnDrop;

#[cfg(test)]
use rand::{RngCore, SeedableRng};

/// Mock RNG provider for deterministic testing
///
/// This provider produces deterministic random numbers seeded from a fixed value.
/// Useful for testing code that depends on randomness in a reproducible way.
///
/// # ⚠️ CRITICAL USAGE WARNING
///
/// Each call to `rng()` creates a NEW `StdRng` instance with the same seed.
/// This means:
///
/// ```rust
/// use z00z_utils::rng::{DeterministicRngProvider, MockRngProvider};
///
/// let provider = MockRngProvider::with_u64_seed(42);
/// let rng1 = provider.rng();  // StdRng(seed=...)
/// let rng2 = provider.rng();  // StdRng(seed=...) - SAME!
/// ```
///
/// **For cryptographic operations:**
/// - ✅ Call `rng()` ONCE per operation
/// - ✅ Use the returned RNG for all random values in that operation
/// - ❌ NEVER call `rng()` multiple times expecting different sequences
///
/// **Example (CORRECT):**
/// ```
/// use z00z_utils::rng::{DeterministicRngProvider, MockRngProvider};
/// use rand::RngCore;
///
/// let provider = MockRngProvider::with_u64_seed(42);
/// let mut rng = provider.rng();  // ONE call
/// let nonce1 = rng.next_u64();   // First random value
/// let nonce2 = rng.next_u64();   // Second random value (different!)
/// ```
///
/// **Example (INCORRECT):**
/// ```rust,ignore
/// let provider = MockRngProvider::with_u64_seed(42);
/// let nonce1 = provider.rng().next_u64();  // Random
/// let nonce2 = provider.rng().next_u64();  // SAME as nonce1!
/// ```
///
/// # Examples
///
/// ```
/// use z00z_utils::rng::{DeterministicRngProvider, MockRngProvider};
/// use rand::RngCore;
///
/// let provider = MockRngProvider::with_u64_seed(42);
/// let mut rng = provider.rng();
/// let value = rng.next_u32();
/// ```
#[derive(Clone, ZeroizeOnDrop)]
pub struct MockRngProvider {
    seed: [u8; 32],
}

impl std::fmt::Debug for MockRngProvider {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MockRngProvider")
            .field("seed", &"<redacted>")
            .finish()
    }
}

impl MockRngProvider {
    /// Create a new mock RNG provider with a 256-bit seed
    pub fn with_seed(seed: [u8; 32]) -> Self {
        Self { seed }
    }

    /// Convenience method for tests: expand u64 to 256-bit seed
    pub fn with_u64_seed(seed: u64) -> Self {
        let mut seed_bytes = [0u8; 32];
        seed_bytes[..8].copy_from_slice(&seed.to_le_bytes());
        let hash = Sha256::digest(seed_bytes);
        Self { seed: hash.into() }
    }

    /// Create a new deterministic RNG instance.
    pub fn rng(&self) -> StdRng {
        <Self as super::traits::DeterministicRngSource>::rng(self)
    }
}

impl Default for MockRngProvider {
    fn default() -> Self {
        Self::with_u64_seed(0)
    }
}

impl super::traits::DeterministicRngSource for MockRngProvider {
    type Rng = StdRng;

    fn rng(&self) -> Self::Rng {
        StdRng::from_seed(self.seed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mock_rng_provider_deterministic() {
        let provider = MockRngProvider::with_u64_seed(42);
        let mut rng1 = provider.rng();
        let mut rng2 = provider.rng();

        let val1 = rng1.next_u32();
        let val2 = rng2.next_u32();

        assert_eq!(val1, val2);
    }

    #[test]
    fn test_mock_rng_provider_different() {
        let provider1 = MockRngProvider::with_u64_seed(42);
        let provider2 = MockRngProvider::with_u64_seed(43);

        let mut rng1 = provider1.rng();
        let mut rng2 = provider2.rng();

        let val1 = rng1.next_u32();
        let val2 = rng2.next_u32();

        assert_ne!(val1, val2);
    }

    #[test]
    fn test_mock_rng_provider_fill() {
        let provider = MockRngProvider::with_u64_seed(42);
        let mut rng1 = provider.rng();
        let mut rng2 = provider.rng();

        let mut bytes1 = [0u8; 32];
        let mut bytes2 = [0u8; 32];

        rng1.fill_bytes(&mut bytes1);
        rng2.fill_bytes(&mut bytes2);

        assert_eq!(&bytes1[..], &bytes2[..]);
    }

    #[test]
    fn test_mock_rng_provider_default() {
        let provider = MockRngProvider::default();
        let mut rng = provider.rng();
        let _ = rng.next_u32(); // Should not panic
    }
}
