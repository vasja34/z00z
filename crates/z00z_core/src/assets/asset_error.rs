use std::borrow::Cow;

use thiserror::Error;
use z00z_crypto::CryptoError;

/// Shared validation/error surface for asset primitives plus root-owned object,
/// policy, rights, voucher, and genesis descriptor validation.
///
/// Canonical public path: `z00z_core::AssetError`.
/// Compatibility facade: `z00z_core::assets::AssetError`.
#[derive(Debug, Error)]
pub enum AssetError {
    #[error("invalid commitment: {0}")]
    InvalidCommitment(Cow<'static, str>),
    #[error("proof verification failed: {0}")]
    ProofVerificationFailed(Cow<'static, str>),
    #[error("invalid metadata: {0}")]
    InvalidMetadata(Cow<'static, str>),
    #[error("burn not allowed: {0}")]
    BurnNotAllowed(Cow<'static, str>),
    #[error("invalid class: {0}")]
    InvalidClass(Cow<'static, str>),
    #[error("invalid decimals: {0}")]
    InvalidDecimals(Cow<'static, str>),
    #[error("invalid asset: {0}")]
    InvalidAsset(Cow<'static, str>),
    #[error("invalid stealth state: {0}")]
    InvalidStealth(Cow<'static, str>),
    #[error("invalid fee: {0}")]
    InvalidFee(Cow<'static, str>),
    #[error("invalid fee asset: {0}")]
    InvalidFeeAsset(Cow<'static, str>),
    #[error("invalid signature: {0}")]
    InvalidSignature(Cow<'static, str>),
    #[error("invalid yaml: {0}")]
    InvalidYaml(Cow<'static, str>),
    #[error("code generation error: {0}")]
    CodeGeneration(Cow<'static, str>),
    #[error("unsupported version: {0}")]
    UnsupportedVersion(Cow<'static, str>),
    #[error("unsupported crypto version {version}, supported: {supported:?}")]
    UnsupportedCryptoVersion { version: u8, supported: Vec<u8> },
    #[error("serialization error: {0}")]
    Serialization(Cow<'static, str>),
    #[error("integrity check failed: {0}")]
    Integrity(Cow<'static, str>),
    #[error("arithmetic overflow: {0}")]
    ArithmeticOverflow(Cow<'static, str>),
    #[error("invalid domain: {0}")]
    InvalidDomain(Cow<'static, str>),
    #[error("lock poisoned: {0}")]
    LockPoisoned(Cow<'static, str>),
    #[error(
        "burn not allowed for definition {definition_id:?}: policy flags {policy_flags:#010b} missing BURNABLE bit (0x01)"
    )]
    BurnNotAllowedStructured {
        definition_id: [u8; 32],
        policy_flags: u8,
    },
    #[error("invalid serial_id {serial_id}: must be < {max_allowed}")]
    InvalidSerialId { serial_id: u32, max_allowed: u32 },
    #[error(
        "invalid serial_id {serial_id} for definition {definition_id:?}: must be < {max_serials}"
    )]
    InvalidSerialIdStructured {
        definition_id: [u8; 32],
        serial_id: u32,
        max_serials: u32,
    },
    #[error("commitment verification failed: {reason}")]
    InvalidCommitmentStructured {
        reason: String,
        commitment: Option<Vec<u8>>,
    },
    #[error("invalid amount {amount} for asset {symbol}: {reason}")]
    InvalidAmountStructured {
        amount: u64,
        symbol: String,
        reason: String,
    },
    #[error("missing range proof")]
    MissingRangeProof,
    #[error("crypto error: {0}")]
    CryptoError(#[from] CryptoError),
    #[error("range proof creation failed: {source}")]
    RangeProofCreation {
        #[source]
        source: CryptoError,
    },
    #[error("range proof verification failed: {source}")]
    RangeProofVerification {
        #[source]
        source: CryptoError,
    },
    #[error("commitment mismatch")]
    CommitmentMismatch { expected: [u8; 32], got: [u8; 32] },
    #[error("amount overflow")]
    AmountOverflow,
    #[error("amount underflow")]
    AmountUnderflow,
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
}

impl From<z00z_utils::io::IoError> for AssetError {
    fn from(err: z00z_utils::io::IoError) -> Self {
        AssetError::Io(std::io::Error::other(err.to_string()))
    }
}

impl From<z00z_utils::codec::CodecError> for AssetError {
    fn from(err: z00z_utils::codec::CodecError) -> Self {
        AssetError::Serialization(Cow::Owned(err.to_string()))
    }
}

impl From<z00z_utils::config::ConfigError> for AssetError {
    fn from(err: z00z_utils::config::ConfigError) -> Self {
        AssetError::InvalidYaml(Cow::Owned(err.to_string()))
    }
}
