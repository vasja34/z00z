//! HD wallet key derivation
//!
//! BIP-44 key derivation without account level.

pub mod bip;
pub mod manager_core;
#[cfg(not(target_arch = "wasm32"))]
pub mod manager_redb;
/// Stealth receiver key material and derivation helpers.
pub mod receiver_keys;
pub mod seed;

pub use self::manager_core::{
    KeyManager, KeyManagerError, KeyManagerImpl, KeyManagerMetadata, KeyManagerState, Result,
    Z00ZKeyBranch, MAX_DERIVED_PUBKEY_CACHE,
};

pub use crate::domains::hashing::{compute_schnorr_challenge, ChallengeBytes, ChallengeSize};

// Re-export RedB managers.
#[cfg(not(target_arch = "wasm32"))]
pub use manager_redb::{
    RedbKeyManager, RedbKeyManagerError, WalletRedbKeyManager, WalletRedbKeyManagerError,
};

pub use crate::receiver::ChainType;

#[cfg(feature = "test-params-fast")]
pub use receiver_keys::benchmark_recv_keys;

pub use receiver_keys::{
    derive_identity_public_key, derive_identity_secret_key, derive_owner_handle,
    derive_view_public_key, derive_view_secret_key, generate_identity_keypair,
    make_view_key_version, sign_identity, sign_identity_with_rng, verify_identity, ReceiverKeys,
    ReceiverSecret, StealthKeyError, ViewKeyVersion,
};

// Re-export BIP-44 policy constants and error types
#[cfg(test)]
pub(crate) use self::bip::{reset_seed_zeroized, seed_zeroized};
pub use self::bip::{
    Bip32KeyDeriver, Bip39Seed64, Bip44Error, Bip44KeyManager, Bip44Path, Bip44PathBuilder,
    Bip44Validator, Bip44ViolationReason, MasterKeyGenerator, RistrettoBridge,
    VIEW_KEY_ACCOUNT_OFFSET, Z00Z_BIP44_ASSET, Z00Z_BIP44_PURPOSE,
};
pub(crate) use self::seed::{mnemonic, validate_entropy};
pub use self::seed::{
    validate_entropy_result, AeadId, Argon2idParams, CipherSeedContainer, CipherSeedError, KdfId,
    MnemonicLanguage, SeedPhrase24, SeedPhraseError, SeedWords,
};
#[cfg(test)]
pub(crate) use self::seed::{validate_entropy_with_warnings, EntropyWarning};
