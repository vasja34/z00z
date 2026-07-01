//! Stealth receiver protocol primitives.
//!
//! Public re-exports in this module are intentionally limited to the
//! canonical runtime surface. Phase 4 removed the extra experimental
//! derivation branches so this module now exposes only stealth-output
//! construction and validation surfaces. Public ECDH, KDF/tag, and ZkPack
//! entrypoints live under `crate::stealth::{ecdh,kdf,zkpack}`, while receiver
//! scan entrypoints live under `crate::receiver`.

/// ECDH, encoding, and ephemeral scalar primitives.
pub(crate) mod crypto;
/// Wallet-facing ECDH entrypoint.
pub mod ecdh;
/// Wallet-facing KDF helpers.
pub mod kdf;
/// Owner-tag and stealth output primitives.
pub mod output;
/// Sender-side post-build validation for lightweight outputs.
pub mod output_validator;
/// Owner tag wrapper and owner-tag trait operations.
pub mod owner_tag;
/// Tag16 and leaf associated-data helpers.
#[path = "tag16.rs"]
pub(crate) mod tag;
/// Wallet-facing deterministic ZkPack helpers.
pub mod zkpack;

use thiserror::Error;

/// Errors for stealth cryptographic protocol operations.
#[derive(Debug, Error, Clone, Copy, PartialEq, Eq)]
pub enum StealthError {
    /// Invalid compressed Ristretto encoding.
    #[error("invalid ristretto point")]
    InvalidRistrettoPoint,
    /// Identity point is rejected by policy.
    #[error("identity point rejected")]
    IdentityPointRejected,
    /// Hedged ephemeral scalar derivation failed.
    #[error("invalid ephemeral scalar")]
    InvalidEphemeralScalar,
    /// Zero scalar is forbidden.
    #[error("zero scalar rejected")]
    ZeroScalarRejected,
    /// ECDH output was identity-equivalent.
    #[error("ecdh identity result")]
    EcdhIdentityResult,
    /// ECDH constraint validation failed.
    #[error("ecdh constraint violation")]
    EcdhConstraintViolation,
    /// Input fields across request/card/output are inconsistent.
    #[error("invalid stealth input")]
    InvalidStealthInput,
    /// Attempted inversion of one-way derivation.
    #[error("not invertible")]
    NotInvertible,
    /// Coin payload encryption failed.
    #[error("stealth payload encryption failed")]
    EncryptFailed,
    /// Retry limit for duplicate-R prevention was exceeded.
    #[error("duplicate R retry limit reached")]
    RetryLimitReached,
}

pub use crypto::owf_constraints_ecdh;
pub use crypto::{derive_sender_salt, generate_r_retry, generate_sender_salt, get_rng_bytes};
#[cfg(feature = "test-params-fast")]
pub use output::benchmark_stealth_output;
pub use output::BuildCheck;
pub use output::{
    bind_stealth_output_wire, build_card_output_serial_checked, build_card_stealth_leaf,
    build_card_stealth_output_validated, build_output_bundle, build_output_bundle_with_rng,
    build_request_output_bundle, build_seeded_output_bundle, build_stealth_leaf,
    build_stealth_leaf_with_blind, build_stealth_leaf_with_rng, build_tx_output_serial_unchecked,
    build_tx_output_unchecked, build_tx_stealth_output_validated, verify_owner_tag,
    verify_owner_tag_with_req, verify_owner_two_factor, SenderWallet, TxStealthOutput,
};
pub use output_validator::{validate_output_self, SenderValidationCtx, TagMode};
pub use owner_tag::{OwnerTag, OwnerTagOperations};

// Public callers should discover ECDH/KDF/ZkPack through `stealth::{ecdh,kdf,zkpack}` and
// receiver scan entrypoints through `receiver`.

// Crate-local shared helpers for sibling modules and tests.
pub(crate) use crypto::{
    compute_dh_receiver, compute_dh_sender, derive_k_dh, derive_k_dh_with_req, derive_s_out,
};
pub(crate) use crypto::{compute_r_pub, derive_r_hedged};
pub(crate) use crypto::{decode_public_key, decode_r_pub, encode_r_pub};
pub(crate) use tag::{compute_leaf_ad, compute_tag16, compute_tag16_with_req};
pub(crate) use z00z_crypto::kdf::compute_owner_tag;
