//! Seed phrase generation helpers.
//!
//! This module centralizes seed phrase generation to keep wallet lifecycle policy
//! in one place and avoid duplicate implementations.

use crate::key::{MnemonicLanguage, SeedPhrase24};

use crate::{WalletError, WalletResult};
use z00z_crypto::Hidden;

/// Generate a fresh 24-word English seed phrase using caller-supplied entropy.
///
/// The entropy source is injected via the `fill_bytes` closure to comply with
/// trait-based DI rules (callers decide how to source randomness).
pub(crate) fn generate_seed_phrase_24_english<F>(mut fill_bytes: F) -> WalletResult<Hidden<String>>
where
    F: FnMut(&mut [u8]),
{
    let mut entropy_bytes = [0u8; 32];
    fill_bytes(&mut entropy_bytes);

    let phrase = SeedPhrase24::from_bip39_entropy_bytes(&entropy_bytes, MnemonicLanguage::English)
        .map_err(|_| WalletError::CryptoError("Failed to generate seed phrase".to_string()))?;

    // Minimize lifetime of raw entropy.
    entropy_bytes.fill(0);

    Ok(phrase.to_phrase())
}
