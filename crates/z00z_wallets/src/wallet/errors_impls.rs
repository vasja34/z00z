impl WalletError {
    #[cfg(not(target_arch = "wasm32"))]
    pub(crate) fn to_public_error(&self, stage: WalletErrorStage) -> WalletPublicError {
        match stage {
            WalletErrorStage::UnlockOpen => match self {
                WalletError::UnsupportedVersion(_) => WalletPublicError::UnsupportedFormat,
                WalletError::UnsupportedKdf(_) => WalletPublicError::UnsupportedFormat,
                WalletError::WalletChainMismatch { .. } => WalletPublicError::UnsupportedFormat,
                WalletError::WalletNetworkMismatch { .. } => WalletPublicError::UnsupportedFormat,
                WalletError::Io(_) | WalletError::InvalidPermissions(_) => {
                    WalletPublicError::IoFailure
                }
                WalletError::Locked | WalletError::SessionExpired | WalletError::SessionInvalid => {
                    WalletPublicError::WalletLocked
                }
                _ => WalletPublicError::InvalidPassword,
            },
        }
    }
}

/// Result type alias for wallet operations
pub type WalletResult<T> = Result<T, WalletError>;

impl From<z00z_crypto::CryptoError> for WalletError {
    fn from(err: z00z_crypto::CryptoError) -> Self {
        Self::CryptoError(err.to_string())
    }
}

impl From<Box<bincode::error::EncodeError>> for WalletError {
    fn from(err: Box<bincode::error::EncodeError>) -> Self {
        Self::SerializationError(err.to_string())
    }
}

impl From<Box<bincode::error::DecodeError>> for WalletError {
    fn from(err: Box<bincode::error::DecodeError>) -> Self {
        Self::SerializationError(err.to_string())
    }
}

/// Trait for classifying errors as transient (retryable) or permanent.
pub trait IsTransient {
    /// Returns `true` if this error is transient and the operation may succeed on retry.
    fn is_transient(&self) -> bool;
}

impl IsTransient for WalletError {
    fn is_transient(&self) -> bool {
        match self {
            WalletError::DatabaseError(_) => true,
            WalletError::RateLimited { .. } => true,
            WalletError::NotOwned
            | WalletError::KeyDerivation(_)
            | WalletError::InsufficientBalance { .. }
            | WalletError::TooManyInputs
            | WalletError::TooManyOutputs
            | WalletError::InvalidAmount(_)
            | WalletError::InvalidFee(_)
            | WalletError::InvalidTransaction(_)
            | WalletError::InsufficientInputs
            | WalletError::BalanceProofFailed
            | WalletError::SerializationError(_)
            | WalletError::NotFound(_)
            | WalletError::JobNotFound(_)
            | WalletError::CryptoError(_)
            | WalletError::IdentityPointNotAllowed
            | WalletError::DuplicateEphemeralR
            | WalletError::ContextError(_)
            | WalletError::CommandNotFound(_)
            | WalletError::CommandFailed(_)
            | WalletError::Locked
            | WalletError::SessionExpired
            | WalletError::SessionInvalid
            | WalletError::InvalidPassword
            | WalletError::WalletInUse
            | WalletError::WalletChainMismatch { .. }
            | WalletError::WalletNetworkMismatch { .. }
            | WalletError::ChecksumMismatch { .. }
            | WalletError::UnsupportedVersion(_)
            | WalletError::UnsupportedKdf(_)
            | WalletError::InvalidPermissions(_)
            | WalletError::Io(_)
            | WalletError::InvalidParams(_)
            | WalletError::InvalidAssetPack(_)
            | WalletError::CommitmentMismatch
            | WalletError::WalletAlreadyExists
            | WalletError::InvalidConfig(_) => false,
        }
    }
}

impl From<WalletEncryptionError> for WalletError {
    fn from(err: WalletEncryptionError) -> Self {
        match err {
            WalletEncryptionError::InvalidPassword => WalletError::InvalidPassword,
            WalletEncryptionError::CryptoError(msg) => WalletError::CryptoError(msg),
            WalletEncryptionError::UnsupportedAlgorithm { got, expected } => {
                WalletError::CryptoError(format!(
                    "Unsupported algorithm: {} (expected {})",
                    got, expected
                ))
            }
            WalletEncryptionError::UnsupportedPayloadVersion { version } => {
                WalletError::CryptoError(format!("Unsupported payload version: {}", version))
            }
            WalletEncryptionError::ChecksumMismatch { expected, actual } => {
                WalletError::CryptoError(format!(
                    "Checksum mismatch (expected {:?}, actual {:?})",
                    expected, actual
                ))
            }
        }
    }
}

impl From<Bip44Error> for WalletError {
    fn from(error: Bip44Error) -> Self {
        WalletError::KeyDerivation(error.to_string())
    }
}