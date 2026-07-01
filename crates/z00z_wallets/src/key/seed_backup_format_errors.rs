/// Errors for seed phrase parsing/encoding.
#[derive(Debug, Error)]
pub enum SeedPhraseError {
    /// The provided seed phrase string is empty.
    #[error("seed phrase is empty")]
    Empty,

    /// The phrase has the wrong word count.
    #[error("seed phrase must have {expected} words, got {got}")]
    InvalidWordCount {
        /// Expected word count.
        expected: usize,
        /// Actual word count.
        got: usize,
    },

    /// The phrase contains a word not present in the selected word list.
    ///
    /// NOTE: The invalid word is treated as sensitive input and must never be included
    /// in public error messages.
    #[error("invalid seed word")]
    InvalidWord,

    /// The provided language does not match the detected language.
    #[error("language mismatch: expected {expected:?}, detected {detected:?}")]
    LanguageMismatch {
        /// Language expected by the caller.
        expected: MnemonicLanguage,
        /// Language detected from the seed words.
        detected: MnemonicLanguage,
    },

    /// The seed words match multiple wordlists.
    #[error("ambiguous language: multiple wordlists match")]
    AmbiguousLanguage,

    /// Bitstream produced by decoding cannot be converted to whole bytes.
    #[error("invalid encoding: bit-length is not a multiple of 8")]
    InvalidBitLength,

    /// Invalid entropy size for the expected format.
    #[error("invalid entropy size")]
    InvalidEntropySize,

    /// Passphrase exceeds maximum allowed length.
    #[error("passphrase too long: {length} bytes (max {max})")]
    PassphraseTooLong {
        /// Observed passphrase length in bytes.
        length: usize,
        /// Maximum allowed passphrase length in bytes.
        max: usize,
    },

    /// Underlying mnemonic conversion failed.
    #[error("invalid seed phrase")]
    Mnemonic(
        #[from]
        #[source]
        MnemonicError,
    ),

    /// Underlying BIP-39 parsing or entropy conversion failed.
    ///
    /// NOTE: The phrase is treated as sensitive and must never be included in errors.
    #[error("invalid seed phrase")]
    Bip39(
        #[from]
        #[source]
        bip39::Error,
    ),
}

/// Validates and normalizes a passphrase according to BIP-39 requirements.
///
/// # Security Checks
///
/// - Enforces maximum length in bytes both before and after normalization
///   to prevent DoS attacks from NFKD expansion
/// - Applies NFKD normalization as required by BIP-39 specification
///
/// # Returns
///
/// - `Ok(String)` containing the NFKD-normalized passphrase if valid
/// - `Err(PassphraseTooLong)` if passphrase exceeds maximum length in bytes
pub fn validate_and_normalize_passphrase(passphrase: &str) -> Result<String, SeedPhraseError> {
    if passphrase.len() > MAX_PASSPHRASE_LENGTH {
        return Err(SeedPhraseError::PassphraseTooLong {
            length: passphrase.len(),
            max: MAX_PASSPHRASE_LENGTH,
        });
    }

    let normalized = passphrase.nfkd().collect::<String>();

    if normalized.len() > MAX_PASSPHRASE_LENGTH {
        return Err(SeedPhraseError::PassphraseTooLong {
            length: normalized.len(),
            max: MAX_PASSPHRASE_LENGTH,
        });
    }

    Ok(normalized)
}