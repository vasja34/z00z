//! SystemRngProvider - production RNG implementation

use super::traits::SecureRngProvider;
use rand::rngs::OsRng;

/// Production RNG provider using cryptographically secure OS randomness
///
/// Returns a new `OsRng` instance for each call. Use this in production code.
/// For testing, use `MockRngProvider` instead.
///
/// # Examples
///
/// ```no_run
/// use z00z_utils::rng::{SecureRngProvider, SystemRngProvider};
/// use rand::RngCore;
///
/// let provider = SystemRngProvider;
/// let mut rng = provider.rng();
/// let mut bytes = [0u8; 32];
/// rng.fill_bytes(&mut bytes);
/// ```
#[derive(Debug, Clone, Copy, Default)]
pub struct SystemRngProvider;

impl SystemRngProvider {
    /// Create a new OS-backed cryptographically secure RNG instance.
    pub fn rng(&self) -> OsRng {
        <Self as SecureRngProvider>::rng(self)
    }
}

impl SecureRngProvider for SystemRngProvider {
    type Rng = OsRng;

    fn rng(&self) -> Self::Rng {
        OsRng
    }
}
