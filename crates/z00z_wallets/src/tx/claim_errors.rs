use thiserror::Error;

#[allow(missing_docs)]
#[derive(Debug, Clone, PartialEq, Error)]
/// Claim transaction validation and decoding errors.
pub enum ClaimTxError {
    #[error("structure malformed: {0}")]
    StructureMalformed(String),
    #[error("digest mismatch: {0}")]
    DigestMismatch(String),
    #[error("kind mismatch: {0}")]
    KindMismatch(String),
    #[error("version unsupported: {0}")]
    VersionUnsupported(String),
    #[error("proof_type invalid: {0}")]
    ProofTypeInvalid(String),
    #[error("proof_blob invalid hex: {0}")]
    ProofBlobInvalidHex(String),
    #[error("proof blob decode failed: {0}")]
    ProofBlobDecode(String),
    #[error("claim proof invalid: {0}")]
    ProofVerify(String),
    #[error("claim source root is zero: {0}")]
    SourceRootZero(String),
    #[error("claim source root version invalid: {0}")]
    SourceRootVersion(String),
    #[error("claim source proof version invalid: {0}")]
    SourceProofVer(String),
    #[error("claim source proof mismatch: {0}")]
    SourceProofMismatch(String),
    #[error("claim source leaf count invalid: {0}")]
    SourceLeafCount(String),
    #[error("fee must be zero: {0}")]
    FeeNonZero(String),
    #[error("outputs are empty")]
    OutputsEmpty,
    #[error("output amount invalid: {0}")]
    OutputAmountZero(String),
    #[error("output asset_class invalid: {0}")]
    OutputAssetClassInvalid(String),
    #[error("output nonce invalid hex: {0}")]
    OutputNonceInvalidHex(String),
    #[error("output nonce is zero: {0}")]
    OutputNonceIsZero(String),
    #[error("output asset_id invalid hex: {0}")]
    OutputAssetIdInvalidHex(String),
    #[error("output owner_binding invalid hex: {0}")]
    OutputOwnerBindingInvalidHex(String),
    #[error("output owner_binding mismatch: {0}")]
    OutputOwnerBindingMismatch(String),
    #[error("output nonce mismatch: {0}")]
    OutputNonceMismatch(String),
    #[error("duplicate nonce: {0}")]
    DuplicateNonce(String),
    #[error("duplicate asset_id: {0}")]
    DuplicateAssetId(String),
    #[error("output leaf required: {0}")]
    LeafRequired(String),
    #[error("output leaf invalid: {0}")]
    LeafInvalid(String),
    #[error("output leaf mismatch: {0}")]
    LeafMismatch(String),
    #[error("recipient card required: {0}")]
    RecipientCardRequired(String),
    #[error("recipient card invalid: {0}")]
    RecipientCardInvalid(String),
    #[error("recipient card mismatch: {0}")]
    RecipientCardMismatch(String),
    #[error("output owner attestation required: {0}")]
    OwnerAttestRequired(String),
    #[error("output owner attestation invalid hex: {0}")]
    OwnerAttestInvalidHex(String),
    #[error("output owner attestation invalid: {0}")]
    OwnerAttestInvalid(String),
    #[error("claim authority decode failed: {0}")]
    AuthoritySigDecode(String),
    #[error("claim authority invalid: {0}")]
    AuthoritySigInvalid(String),
    #[error("nullifier invalid hex: {0}")]
    NullifierInvalidHex(String),
    #[error("nullifier mismatch: {0}")]
    NullifierMismatch(String),
}

#[allow(missing_docs)]
#[derive(Debug, Clone, Default)]
/// Structured per-step report for claim verification.
pub struct ClaimTxVerifyReport {
    pub digest_checked: bool,
    pub card_checked: bool,
    pub owner_attest_checked: bool,
    pub leaf_checked: bool,
    pub proof_checked: bool,
    pub authority_checked: bool,
    pub nullifier_checked: bool,
}

#[allow(missing_docs)]
#[derive(Debug, Clone)]
/// Verification result returned by `ClaimTxVerifier`.
pub struct ClaimVerifyResult {
    pub valid: bool,
    pub reject_class: String,
    pub errors: Vec<String>,
    pub report: Option<ClaimTxVerifyReport>,
}
