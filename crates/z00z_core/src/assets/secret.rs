use rand::{CryptoRng, RngCore};
use thiserror::Error;
use z00z_utils::rng::SystemRngProvider;

/// Errors for asset secret generation.
#[derive(Debug, Error)]
pub enum AssetSecretError {
    #[error("Insufficient entropy from system RNG")]
    InsufficientEntropy,
    #[error("RNG initialization failed")]
    RngInitFailed(#[from] rand_core::Error),
    #[error("Weak RNG detected in production context")]
    WeakRngInProduction,
}

fn generate_asset_secret_with<R>(rng: &mut R) -> Result<[u8; 32], AssetSecretError>
where
    R: RngCore + CryptoRng,
{
    let mut bytes = [0u8; 32];
    rng.try_fill_bytes(&mut bytes)
        .map_err(AssetSecretError::RngInitFailed)?;

    if bytes == [0u8; 32] {
        return Err(AssetSecretError::InsufficientEntropy);
    }

    Ok(bytes)
}

/// Generate a 32-byte asset secret using the system cryptographic RNG.
pub fn generate_asset_secret_checked() -> Result<[u8; 32], AssetSecretError> {
    let mut rng = SystemRngProvider.rng();
    generate_asset_secret_with(&mut rng)
}

#[cfg(test)]
mod tests {
    use super::{generate_asset_secret_with, AssetSecretError};
    use rand::{CryptoRng, RngCore};

    #[derive(Default)]
    struct ZeroRng;

    impl RngCore for ZeroRng {
        fn next_u32(&mut self) -> u32 {
            0
        }

        fn next_u64(&mut self) -> u64 {
            0
        }

        fn fill_bytes(&mut self, dest: &mut [u8]) {
            dest.fill(0);
        }

        fn try_fill_bytes(&mut self, dest: &mut [u8]) -> Result<(), rand::Error> {
            self.fill_bytes(dest);
            Ok(())
        }
    }

    impl CryptoRng for ZeroRng {}

    #[derive(Default)]
    struct FailRng;

    impl RngCore for FailRng {
        fn next_u32(&mut self) -> u32 {
            0
        }

        fn next_u64(&mut self) -> u64 {
            0
        }

        fn fill_bytes(&mut self, dest: &mut [u8]) {
            dest.fill(0);
        }

        fn try_fill_bytes(&mut self, _dest: &mut [u8]) -> Result<(), rand::Error> {
            Err(rand::Error::new("rng_fail"))
        }
    }

    impl CryptoRng for FailRng {}

    #[test]
    fn test_zero_rng_reject() {
        let mut rng = ZeroRng;
        let result = generate_asset_secret_with(&mut rng);
        assert!(matches!(result, Err(AssetSecretError::InsufficientEntropy)));
    }

    #[test]
    fn test_fail_rng_error() {
        let mut rng = FailRng;
        let result = generate_asset_secret_with(&mut rng);
        assert!(matches!(result, Err(AssetSecretError::RngInitFailed(_))));
    }
}
