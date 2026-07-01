//! Transaction Signer - Schnorr Signatures for Z00Z
//!
//! This module provides transaction signing using Tari crypto's RistrettoSchnorr.
//!
//! ## Architecture
//!
//! ```text
//! WalletRedbKeyManager (storage)
//!     ↓ provides master_key
//! wallet_keys.rs (derivation)
//!     ↓ provides secret_key for path
//! WalletSigner (signing)
//!     ↓ creates signature
//! KernelSignature (RistrettoSchnorr)
//! ```
//!
//! ## Security Properties
//!
//! - Uses Blake2b-512 for challenge computation
//! - Domain-separated via wallet hashing compute_schnorr_challenge
//! - Random nonce generation per signature
//! - `Hidden<T>` for secret key protection
//! - Tari crypto's RistrettoSchnorr (audited implementation)

use crate::key::{compute_schnorr_challenge, ChallengeSize};
use subtle::ConstantTimeEq;
use thiserror::Error;
use z00z_crypto::expert::encoding::ByteArray;
use z00z_crypto::expert::keys::{RistrettoPublicKey, RistrettoSecretKey};
use z00z_crypto::expert::traits::{PublicKeyTrait, SecretKeyTrait};
use z00z_crypto::{Hidden, KernelSignature, Z00ZRistrettoPoint, Z00ZScalar};
use z00z_utils::rng::SecureRngProvider;

/// Signer errors.
#[derive(Debug, Error)]
pub enum SignerError {
    /// Signing failed.
    #[error("signing failed: {0}")]
    SigningFailed(String),

    /// Invalid key.
    #[error("invalid key: {0}")]
    InvalidKey(String),

    /// Verification failed.
    #[error("verification failed: {0}")]
    VerificationFailed(String),

    /// Cryptographic error.
    #[error("cryptographic error: {0}")]
    Crypto(String),
}

/// Signer result type.
pub type SignerResult<T> = std::result::Result<T, SignerError>;

/// Transaction signer trait.
///
/// Returns opaque signature bytes for now.
///
/// # Implementation Requirements
///
/// - Must use z00z_crypto::RistrettoSchnorr for signatures
/// - Must use `z00z_crypto::Hidden<T>` for secret key protection
/// - Must use z00z_utils::rng::SecureRngProvider for signature randomness
pub trait Signer {
    /// Sign a serialized transaction.
    fn sign(
        &self,
        tx_bytes: &[u8],
        secret_key: &Hidden<RistrettoSecretKey>,
    ) -> SignerResult<Vec<u8>>;

    /// Sign an arbitrary message.
    fn sign_message(
        &self,
        message: &[u8],
        secret_key: &Hidden<RistrettoSecretKey>,
    ) -> SignerResult<Vec<u8>>;

    /// Verify an opaque signature.
    fn verify(
        &self,
        message: &[u8],
        signature: &[u8],
        public_key_bytes: &[u8],
    ) -> SignerResult<bool>;

    /// Multi-sign a transaction.
    fn multi_sign(
        &self,
        tx_bytes: &[u8],
        secret_keys: &[Hidden<RistrettoSecretKey>],
    ) -> SignerResult<Vec<u8>>;
}

/// Default Signer implementation.
#[derive(Debug)]
pub struct SignerImpl<R: SecureRngProvider> {
    rng_provider: R,
}

impl<R: SecureRngProvider> SignerImpl<R> {
    /// Create a new signer with RNG provider.
    pub fn new(rng_provider: R) -> Self {
        Self { rng_provider }
    }
}

impl<R: SecureRngProvider> Signer for SignerImpl<R> {
    fn sign(
        &self,
        tx_bytes: &[u8],
        secret_key: &Hidden<RistrettoSecretKey>,
    ) -> SignerResult<Vec<u8>> {
        // Validate secret key is not zero
        if secret_key
            .reveal()
            .ct_eq(&RistrettoSecretKey::default())
            .unwrap_u8()
            != 0
        {
            return Err(SignerError::InvalidKey("zero secret key".to_string()));
        }

        // Use the same implementation as manager_impl.rs::sign()
        let mut rng = self.rng_provider.rng();

        let nonce = RistrettoSecretKey::random(&mut rng);
        let public_key = RistrettoPublicKey::from_secret_key(secret_key.reveal());
        let public_nonce = RistrettoPublicKey::from_secret_key(&nonce);

        let public_nonce_bytes: &[u8; 32] = public_nonce
            .as_bytes()
            .try_into()
            .map_err(|_| SignerError::Crypto("invalid public nonce length".to_string()))?;
        let public_key_bytes: &[u8; 32] = public_key
            .as_bytes()
            .try_into()
            .map_err(|_| SignerError::Crypto("invalid public key length".to_string()))?;

        let challenge = compute_schnorr_challenge(
            public_nonce_bytes,
            public_key_bytes,
            tx_bytes,
            ChallengeSize::B512,
        )
        .into_b512()
        .ok_or_else(|| SignerError::Crypto("unexpected challenge size".to_string()))?;

        let sig = KernelSignature::sign_raw_uniform(secret_key.reveal(), nonce, &challenge)
            .map_err(|e| SignerError::SigningFailed(e.to_string()))?;

        // CRITICAL: Verify signature s is not zero
        if sig
            .get_signature()
            .ct_eq(&RistrettoSecretKey::default())
            .unwrap_u8()
            != 0
        {
            return Err(SignerError::SigningFailed(
                "generated signature s is zero".to_string(),
            ));
        }

        // Serialize signature to bytes: [public_nonce][signature]
        let public_nonce_bytes = sig.get_public_nonce().as_bytes();
        let signature_bytes = sig.get_signature().as_bytes();

        let mut result = Vec::with_capacity(public_nonce_bytes.len() + signature_bytes.len());
        result.extend_from_slice(public_nonce_bytes);
        result.extend_from_slice(signature_bytes);

        Ok(result)
    }

    fn sign_message(
        &self,
        message: &[u8],
        secret_key: &Hidden<RistrettoSecretKey>,
    ) -> SignerResult<Vec<u8>> {
        // Validate secret key is not zero
        if secret_key
            .reveal()
            .ct_eq(&RistrettoSecretKey::default())
            .unwrap_u8()
            != 0
        {
            return Err(SignerError::InvalidKey("zero secret key".to_string()));
        }

        // Same as sign() but for arbitrary messages
        let mut rng = self.rng_provider.rng();

        let nonce = RistrettoSecretKey::random(&mut rng);
        let public_key = RistrettoPublicKey::from_secret_key(secret_key.reveal());
        let public_nonce = RistrettoPublicKey::from_secret_key(&nonce);

        let public_nonce_bytes: &[u8; 32] = public_nonce
            .as_bytes()
            .try_into()
            .map_err(|_| SignerError::Crypto("invalid public nonce length".to_string()))?;
        let public_key_bytes: &[u8; 32] = public_key
            .as_bytes()
            .try_into()
            .map_err(|_| SignerError::Crypto("invalid public key length".to_string()))?;

        let challenge = compute_schnorr_challenge(
            public_nonce_bytes,
            public_key_bytes,
            message,
            ChallengeSize::B512,
        )
        .into_b512()
        .ok_or_else(|| SignerError::Crypto("unexpected challenge size".to_string()))?;

        let sig = KernelSignature::sign_raw_uniform(secret_key.reveal(), nonce, &challenge)
            .map_err(|e| SignerError::SigningFailed(e.to_string()))?;

        // CRITICAL: Verify signature s is not zero
        if sig
            .get_signature()
            .ct_eq(&RistrettoSecretKey::default())
            .unwrap_u8()
            != 0
        {
            return Err(SignerError::SigningFailed(
                "generated signature s is zero".to_string(),
            ));
        }

        // Serialize signature to bytes: [public_nonce][signature]
        let public_nonce_bytes = sig.get_public_nonce().as_bytes();
        let signature_bytes = sig.get_signature().as_bytes();

        let mut result = Vec::with_capacity(public_nonce_bytes.len() + signature_bytes.len());
        result.extend_from_slice(public_nonce_bytes);
        result.extend_from_slice(signature_bytes);

        Ok(result)
    }

    fn verify(
        &self,
        message: &[u8],
        signature: &[u8],
        public_key_bytes: &[u8],
    ) -> SignerResult<bool> {
        // Verify signature length (64 bytes: 32 nonce + 32 s)
        if signature.len() != 64 {
            return Err(SignerError::InvalidKey(format!(
                "signature must be 64 bytes, got {}",
                signature.len()
            )));
        }

        // Parse public key
        let public_key = Z00ZRistrettoPoint::try_from_bytes(
            public_key_bytes
                .try_into()
                .map_err(|_| SignerError::InvalidKey("invalid public key length".to_string()))?,
        )
        .map_err(|e| SignerError::InvalidKey(format!("invalid public key: {}", e)))?;

        // Parse public nonce (first 32 bytes)
        let public_nonce = Z00ZRistrettoPoint::try_from_bytes(
            signature[..32]
                .try_into()
                .map_err(|_| SignerError::InvalidKey("invalid nonce length".to_string()))?,
        )
        .map_err(|e| SignerError::InvalidKey(format!("invalid nonce: {}", e)))?;

        // Parse signature s (last 32 bytes)
        let s = Z00ZScalar::try_from_bytes(
            signature[32..]
                .try_into()
                .map_err(|_| SignerError::InvalidKey("invalid signature s length".to_string()))?,
        )
        .map_err(|e| SignerError::InvalidKey(format!("invalid signature s: {}", e)))?;

        // CRITICAL: Verify s is not zero
        if s.is_zero() {
            return Err(SignerError::InvalidKey("signature s is zero".to_string()));
        }

        // Reconstruct signature
        let sig = KernelSignature::new(public_nonce.reveal().clone(), s.reveal().clone());

        // Compute challenge
        let public_nonce_bytes: &[u8; 32] = public_nonce
            .as_bytes()
            .try_into()
            .map_err(|_| SignerError::Crypto("invalid public nonce length".to_string()))?;
        let public_key_bytes: &[u8; 32] = public_key
            .as_bytes()
            .try_into()
            .map_err(|_| SignerError::Crypto("invalid public key length".to_string()))?;

        let challenge = compute_schnorr_challenge(
            public_nonce_bytes,
            public_key_bytes,
            message,
            ChallengeSize::B512,
        )
        .into_b512()
        .ok_or_else(|| SignerError::Crypto("unexpected challenge size".to_string()))?;

        // Verify using raw uniform verification (challenge is already computed)
        Ok(sig.verify_raw_uniform(public_key.reveal(), &challenge))
    }

    fn multi_sign(
        &self,
        tx_bytes: &[u8],
        secret_keys: &[Hidden<RistrettoSecretKey>],
    ) -> SignerResult<Vec<u8>> {
        // Z00Z-native: Simple multi-signature by concatenating individual signatures
        // Each signature is 64 bytes: [public_nonce][s]
        // Total size = 64 * secret_keys.len()

        if secret_keys.is_empty() {
            return Err(SignerError::InvalidKey(
                "empty secret keys list".to_string(),
            ));
        }

        let mut aggregated = Vec::with_capacity(64 * secret_keys.len());

        for secret_key in secret_keys {
            // Validate key is not zero
            if secret_key
                .reveal()
                .ct_eq(&RistrettoSecretKey::default())
                .unwrap_u8()
                != 0
            {
                return Err(SignerError::InvalidKey("zero secret key".to_string()));
            }

            let mut rng = self.rng_provider.rng();
            let nonce = RistrettoSecretKey::random(&mut rng);
            let public_key = RistrettoPublicKey::from_secret_key(secret_key.reveal());
            let public_nonce = RistrettoPublicKey::from_secret_key(&nonce);

            let public_nonce_bytes: &[u8; 32] = public_nonce
                .as_bytes()
                .try_into()
                .map_err(|_| SignerError::Crypto("invalid public nonce length".to_string()))?;
            let public_key_bytes: &[u8; 32] = public_key
                .as_bytes()
                .try_into()
                .map_err(|_| SignerError::Crypto("invalid public key length".to_string()))?;

            let challenge = compute_schnorr_challenge(
                public_nonce_bytes,
                public_key_bytes,
                tx_bytes,
                ChallengeSize::B512,
            )
            .into_b512()
            .ok_or_else(|| SignerError::Crypto("unexpected challenge size".to_string()))?;

            let sig = KernelSignature::sign_raw_uniform(secret_key.reveal(), nonce, &challenge)
                .map_err(|e| SignerError::SigningFailed(e.to_string()))?;

            // Serialize: [public_nonce][signature]
            let public_nonce_bytes = sig.get_public_nonce().as_bytes();
            let signature_bytes = sig.get_signature().as_bytes();

            aggregated.extend_from_slice(public_nonce_bytes);
            aggregated.extend_from_slice(signature_bytes);
        }

        Ok(aggregated)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use z00z_crypto::expert::traits::SecretKeyTrait;
    use z00z_utils::rng::{MockRngProvider, SystemRngProvider};

    #[test]
    fn test_sign_message_works() {
        let signer = SignerImpl::new(SystemRngProvider);

        // Create test secret key
        let mut rng = MockRngProvider::with_u64_seed(42).rng();
        let sk = Hidden::hide(RistrettoSecretKey::random(&mut rng));

        let message = b"test message";
        let result = signer.sign_message(message, &sk);

        // Should succeed (not return error)
        assert!(result.is_ok());
        let signature = result.unwrap();
        assert_eq!(signature.len(), 64); // RistrettoSchnorr bytes
    }
}
