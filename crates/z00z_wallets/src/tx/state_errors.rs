use thiserror::Error;

/// Tx proof verifier error contract.
#[derive(Debug, Error, PartialEq, Eq)]
pub enum TxProofError {
    /// Invalid proof bytes.
    #[error("invalid tx proof")]
    Invalid,
    /// Unsupported proof version.
    #[error("unsupported tx proof version")]
    Version,
}

/// Spent-index error contract.
#[derive(Debug, Error, PartialEq, Eq)]
pub enum SpentIndexError {
    /// Interval lookup failed.
    #[error("spent index lookup failed")]
    Lookup,
}

/// Deterministic state-update failures.
#[derive(Debug, Error, PartialEq, Eq)]
pub enum StateError {
    /// Batch must contain at least one tx package.
    #[error("tx batch must be non-empty")]
    EmptyBatch,
    /// Tx declared root does not match batch root.
    #[error("tx prev_root mismatch")]
    PrevRoot,
    /// Input set must be non-empty.
    #[error("tx inputs must be non-empty")]
    EmptyInputs,
    /// Output set must be non-empty.
    #[error("tx outputs must be non-empty")]
    EmptyOutputs,
    /// Duplicate input in one tx package.
    #[error("duplicate input in tx package")]
    DupInput,
    /// Duplicate output or existing terminal id conflict.
    #[error("duplicate output terminal id")]
    DupOut,
    /// Missing input in state snapshot.
    #[error("missing input in state")]
    MissingInput,
    /// Input reference is malformed.
    #[error("malformed input reference")]
    BadInputRef,
    /// Resolved leaf does not match declared serial.
    #[error("input leaf-match failed")]
    LeafMatch,
    /// Membership witness missing or malformed.
    #[error("invalid membership witness")]
    BadMember,
    /// Resolved input data is missing or inconsistent.
    #[error("invalid resolved input")]
    BadResolve,
    /// Input was spent after declared root.
    #[error("input spent in interval")]
    SpentAfter,
    /// Input was already consumed earlier in the same batch.
    #[error("input spent in batch")]
    SpentBatch,
    /// State backend rejected operation.
    #[error("state backend failure: {0}")]
    State(String),
    /// Tx proof verifier failed.
    #[error(transparent)]
    TxProof(#[from] TxProofError),
    /// Spent index failed.
    #[error(transparent)]
    SpentIndex(#[from] SpentIndexError),
}
