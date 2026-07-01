//! BIP facade rooted in the flat `src/key/*.rs` layout.

#[path = "bip32.rs"]
mod bip32;

pub use bip32::{
    Bip32KeyDeriver, Bip39Seed64, Bip44Error, Bip44KeyManager, Bip44Path, Bip44PathBuilder,
    Bip44Validator, Bip44ViolationReason, MasterKeyGenerator, RistrettoBridge, MAX_BIP32_INDEX,
    VIEW_KEY_ACCOUNT_OFFSET, Z00Z_BIP44_ASSET, Z00Z_BIP44_PURPOSE,
};

#[cfg(test)]
pub(crate) use bip32::{reset_seed_zeroized, seed_zeroized};
