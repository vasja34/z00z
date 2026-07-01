//! Blinding factor generation helpers for confidential assets.

use z00z_crypto::{Hidden, Z00ZScalar};
use z00z_utils::rng::{RngCoreExt, SystemRngProvider};

pub fn generate_blinding(rng: &mut impl rand::RngCore) -> Hidden<Z00ZScalar> {
    let mut bytes = [0u8; 32];

    for _ in 0..64 {
        rng.fill_bytes_ext(&mut bytes);
        if let Ok(scalar) = Z00ZScalar::try_from_bytes(bytes) {
            return Hidden::hide(scalar);
        }
    }

    let mut system_rng = SystemRngProvider.rng();
    Hidden::hide(Z00ZScalar::random(&mut system_rng))
}

/// Stateless generator for secure transaction blinding factors.
pub struct BlindingFactorGenerator;

impl BlindingFactorGenerator {
    /// Generate one blinding factor wrapped in [`Hidden`].
    pub fn generate(&self) -> Hidden<Z00ZScalar> {
        let mut rng = SystemRngProvider.rng();
        Hidden::hide(Z00ZScalar::random(&mut rng))
    }

    /// Generate a batch of independent blinding factors.
    pub fn generate_batch(&self, count: usize) -> Vec<Hidden<Z00ZScalar>> {
        let mut rng = SystemRngProvider.rng();
        (0..count)
            .map(|_| Hidden::hide(Z00ZScalar::random(&mut rng)))
            .collect()
    }
}

impl Default for BlindingFactorGenerator {
    fn default() -> Self {
        Self
    }
}

#[cfg(test)]
mod tests {
    use super::{generate_blinding, BlindingFactorGenerator};
    use rand::SeedableRng;
    use rand_chacha::ChaCha20Rng;
    use std::collections::BTreeSet;
    use z00z_crypto::Hidden;
    use zeroize::Zeroize;

    #[test]
    fn test_blinding_type_is_zscalar() {
        let generator = BlindingFactorGenerator;
        let blinding = generator.generate();
        let _: [u8; 32] = blinding.reveal().to_bytes();
    }

    #[test]
    fn test_blinding_uniqueness() {
        let generator = BlindingFactorGenerator;
        let first = generator.generate();
        let second = generator.generate();

        assert!(!first.reveal().ct_eq(second.reveal()));
    }

    #[test]
    fn test_hidden_wrapping() {
        let generator = BlindingFactorGenerator;
        let blinding = generator.generate();
        let mut wrapped = Hidden::hide(blinding.reveal().dangerous_clone());
        assert!(!wrapped.reveal().is_zero());
        wrapped.zeroize();
        assert!(wrapped.reveal().is_zero());
    }

    #[test]
    fn test_batch_all_unique() {
        let generator = BlindingFactorGenerator;
        let batch = generator.generate_batch(100);

        let set: BTreeSet<[u8; 32]> = batch.iter().map(|item| item.reveal().to_bytes()).collect();
        assert_eq!(set.len(), batch.len());
    }

    #[test]
    fn test_generate_blind_fn() {
        let mut rng = ChaCha20Rng::from_seed([7u8; 32]);
        let first = generate_blinding(&mut rng);
        let second = generate_blinding(&mut rng);

        assert_ne!(first.reveal().to_bytes(), second.reveal().to_bytes());
    }
}
