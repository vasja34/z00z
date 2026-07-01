/// Internal diagnostic codes for wallet corruption and authentication-adjacent failures.
#[allow(missing_docs)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg(not(target_arch = "wasm32"))]
pub(crate) enum WalletDiagCode {
    MetaInvalid,
    SecretsMissing,
    ContainerInvalid,
    DecompressFail,
    DbOpenFail,
    IntegrityMissing,
}

/// Public wallet error taxonomy for safe, bounded error reporting.
#[allow(missing_docs)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
pub enum WalletPublicError {
    #[error("invalid password")]
    InvalidPassword,
    #[error("corrupted wallet")]
    CorruptedWallet,
    #[error("wallet is locked")]
    WalletLocked,
    #[error("unsupported wallet format")]
    UnsupportedFormat,
    #[error("wallet I/O failure")]
    IoFailure,
}

#[allow(missing_docs)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg(not(target_arch = "wasm32"))]
pub(crate) enum WalletErrorStage {
    UnlockOpen,
}

#[cfg(not(target_arch = "wasm32"))]
impl std::fmt::Display for WalletDiagCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            WalletDiagCode::MetaInvalid => "META_INVALID",
            WalletDiagCode::SecretsMissing => "SECRETS_MISSING",
            WalletDiagCode::ContainerInvalid => "CONTAINER_INVALID",
            WalletDiagCode::DecompressFail => "DECOMPRESS_FAIL",
            WalletDiagCode::DbOpenFail => "DB_OPEN_FAIL",
            WalletDiagCode::IntegrityMissing => "INTEGRITY_MISSING",
        };
        f.write_str(s)
    }
}

/// State transition errors for wallet state machine.
#[allow(missing_docs)]
#[derive(Debug, Error)]
pub enum StateTransitionError {
    #[error("Invalid state transition from {from:?} to {to:?}")]
    InvalidTransition {
        from: super::WalletState,
        to: super::WalletState,
    },
}

/// Wallet errors following Z00Z Design Foundation error handling
#[allow(missing_docs)]
#[derive(Debug, Error)]
pub enum WalletError {
    #[error("Asset not owned by this wallet")]
    NotOwned,
    #[error("Failed to derive key: {0}")]
    KeyDerivation(String),
    #[error("Insufficient balance: need {needed}, have {available}")]
    InsufficientBalance { needed: u64, available: u64 },
    #[error("Too many inputs: max 4 allowed (privacy constraint)")]
    TooManyInputs,
    #[error("Too many outputs: max 2 allowed (privacy constraint)")]
    TooManyOutputs,
    #[error("Invalid amount: {0}")]
    InvalidAmount(String),
    #[error("Invalid fee: {0}")]
    InvalidFee(String),
    #[error("Invalid transaction: {0}")]
    InvalidTransaction(String),
    #[error("No inputs provided")]
    InsufficientInputs,
    #[error("Pedersen balance proof failed")]
    BalanceProofFailed,
    #[error("Database error: {0}")]
    DatabaseError(String),
    #[error("Serialization error: {0}")]
    SerializationError(String),
    #[error("Asset not found: serial_id={0}")]
    NotFound(u32),
    #[error("Cryptographic operation failed: {0}")]
    CryptoError(String),
    #[error("Identity point is not allowed")]
    IdentityPointNotAllowed,
    #[error("Duplicate ephemeral R detected")]
    DuplicateEphemeralR,
    #[error("Context error: {0}")]
    ContextError(String),
    #[error("Command not found: {0}")]
    CommandNotFound(String),
    #[error("Command failed: {0}")]
    CommandFailed(String),
    #[error("Wallet is locked")]
    Locked,
    #[error("Session expired")]
    SessionExpired,
    #[error("Session invalid")]
    SessionInvalid,
    #[error("Invalid password")]
    InvalidPassword,
    #[error("Wallet in use")]
    WalletInUse,
    #[error("Wallet chain mismatch")]
    WalletChainMismatch { expected: String, actual: String },
    #[error("Wallet network mismatch")]
    WalletNetworkMismatch { expected: String, actual: String },
    #[error("Checksum mismatch: expected {expected}, got {actual}")]
    ChecksumMismatch { expected: String, actual: String },
    #[error("Unsupported version: {0}")]
    UnsupportedVersion(u32),
    #[error("Unsupported KDF: {0}")]
    UnsupportedKdf(String),
    #[error("Rate limited: retry after {retry_after_seconds} seconds")]
    RateLimited { retry_after_seconds: u32 },
    #[error("Job not found: {0}")]
    JobNotFound(String),
    #[error("Invalid file permissions: {0}")]
    InvalidPermissions(String),
    #[error("I/O error: {0}")]
    Io(String),
    #[error("Invalid params: {0}")]
    InvalidParams(String),
    #[error("InvalidAssetPack: {0}")]
    InvalidAssetPack(&'static str),
    #[error("CommitmentMismatch")]
    CommitmentMismatch,
    #[error("Wallet already exists")]
    WalletAlreadyExists,
    #[error("Invalid configuration: {0}")]
    InvalidConfig(String),
}
