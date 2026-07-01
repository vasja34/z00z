use thiserror::Error;

/// Transaction verifier errors.
#[derive(Debug, Error)]
pub enum TxVerifierError {
    /// Generic verification failure.
    #[error("verification failed: {0}")]
    VerificationFailed(String),

    /// Signature is invalid.
    #[error("invalid signature: {0}")]
    InvalidSignature(String),

    /// Range proof is invalid.
    #[error("invalid range proof: {0}")]
    InvalidRangeProof(String),

    /// Structure mismatch.
    #[error("invalid structure: {0}")]
    InvalidStructure(String),
}

/// Transaction verifier result type.
pub type TxVerifierResult<T> = std::result::Result<T, TxVerifierError>;
