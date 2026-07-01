/// Mnemonic-related errors.
#[derive(Debug, Error)]
pub enum MnemonicError {
    /// Empty input.
    #[error("seed words are empty")]
    Empty,

    /// A word is not found in the language word list.
    ///
    /// NOTE: The offending word is treated as sensitive input and must never be included
    /// in public error messages.
    #[error("word not found in word list")]
    WordNotFound,

    /// The language cannot be uniquely determined.
    #[error("unknown or ambiguous language")]
    UnknownLanguage,

    /// Multiple languages match the seed words.
    #[error("ambiguous language: multiple wordlists match")]
    AmbiguousLanguage,

    /// Bitstream could not be converted into bytes.
    #[error("invalid encoding: bit-length is not a multiple of 8")]
    InvalidBitLength,

    /// Insufficient entropy (minimum 128 bits required).
    #[error("insufficient entropy: minimum 128 bits required")]
    InsufficientEntropy,

    /// Invalid entropy size (must be 16, 20, 24, 28, or 32 bytes for BIP39).
    #[error("invalid entropy size: must be 16, 20, 24, 28, or 32 bytes")]
    InvalidEntropySize,

    /// Invalid BIP-39 phrase or checksum.
    #[error("invalid mnemonic phrase")]
    InvalidPhrase,
}

/// A sequence of mnemonic words.
///
/// Words are stored as `Hidden<String>` to reduce accidental logging.
/// Implements `Zeroize` to automatically clear memory on drop.
#[derive(Zeroize)]
#[zeroize(drop)]
pub struct SeedWords {
    words: Vec<Hidden<String>>,
}

impl SeedWords {
    /// Create a new `SeedWords`.
    pub fn new(words: Vec<Hidden<String>>) -> Self {
        Self { words }
    }

    /// Number of words.
    pub fn len(&self) -> usize {
        self.words.len()
    }

    /// Returns true if there are no words.
    pub fn is_empty(&self) -> bool {
        self.words.is_empty()
    }

    /// Get a word by index.
    pub fn get_word(&self, index: usize) -> Option<&str> {
        self.words.get(index).map(|w| w.reveal().as_str())
    }

    /// Join words with a separator.
    pub fn join(&self, sep: &str) -> Hidden<String> {
        self.join_revealed(sep, |phrase| Hidden::hide(phrase.to_owned()))
    }

    /// Join words with a separator and expose the joined phrase to a closure.
    ///
    /// Temporarily materializes the phrase in a `Zeroizing<String>` buffer.
    /// The phrase is wiped after the closure returns.
    pub fn join_revealed<F, T>(&self, sep: &str, f: F) -> T
    where
        F: FnOnce(&str) -> T,
    {
        let mut phrase = Zeroizing::new(String::new());
        for (idx, word) in self.words.iter().enumerate() {
            if idx > 0 {
                phrase.push_str(sep);
            }
            phrase.push_str(word.reveal());
        }
        f(phrase.as_str())
    }
}

impl FromStr for SeedWords {
    type Err = MnemonicError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let phrase = s.trim();
        if phrase.is_empty() {
            return Err(MnemonicError::Empty);
        }

        let words = phrase
            .split_whitespace()
            .map(|w| Hidden::hide(w.to_string()))
            .collect::<Vec<_>>();

        Ok(Self::new(words))
    }
}

/// Mnemonic encoding/decoding helpers.
///
/// # Domain Separation
///
/// All operations use domain-separated contexts:
/// - Encoding: `z00z/crypto/seed_encoding`
/// - Validation: `z00z/crypto/seed_validation`
///
/// # Language Policy
///
/// Callers MUST specify the mnemonic language explicitly.
/// Validation and decoding are performed only in the provided language.
///
/// If language is unknown, use `suggest_language()` and prompt the user.
///
/// Note: This is a wallet-level contract. BIP-39 supports multiple languages,
/// but does not standardize language auto-detection.
pub mod mnemonic {
    use super::{SeedWords, MnemonicLanguage, Mnemonic, MnemonicError, Hidden};

    /// Suggest languages that can parse the provided words as a valid BIP-39 phrase.
    ///
    /// Returns an empty vector when no supported language matches.
    pub fn suggest_language(words: &SeedWords) -> Vec<MnemonicLanguage> {
        if words.is_empty() {
            return Vec::new();
        }

        words.join_revealed(" ", |phrase| {
            let mut found = Vec::new();
            for &lang in MnemonicLanguage::ALL.iter() {
                if Mnemonic::parse_in(lang, phrase).is_ok() {
                    found.push(lang);
                }
            }
            found
        })
    }

    /// Convert entropy bytes into BIP-39 seed words.
    ///
    /// # Domain Separation
    ///
    /// This operation uses the domain: `z00z/crypto/seed_encoding`
    ///
    /// # Security Requirements
    ///
    /// - Entropy size MUST be one of BIP-39 sizes: 16, 20, 24, 28, 32 bytes
    /// - Uses the `bip39` crate for checksum computation and validation
    pub fn from_bytes(
        bytes: &[u8],
        language: MnemonicLanguage,
    ) -> Result<SeedWords, MnemonicError> {
        match bytes.len() {
            16 | 20 | 24 | 28 | 32 => {}
            _ => return Err(MnemonicError::InvalidEntropySize),
        }

        let mnemonic =
            Mnemonic::from_entropy_in(language, bytes).map_err(|_| MnemonicError::InvalidPhrase)?;

        let mut words = Vec::with_capacity(mnemonic.word_count());
        for word in mnemonic.words() {
            words.push(Hidden::hide(word.to_string()));
        }

        Ok(SeedWords::new(words))
    }

    /// Convert seed words into BIP-39 entropy bytes using the specified language.
    pub fn to_bytes_with_language(
        words: &SeedWords,
        language: &MnemonicLanguage,
    ) -> Result<Hidden<Vec<u8>>, MnemonicError> {
        if words.is_empty() {
            return Err(MnemonicError::Empty);
        }

        // Validate checksum and word count in the caller-supplied language.
        // NOTE: We intentionally do NOT call `Mnemonic::to_entropy()` here.
        // Upstream `bip39` derives language from the word list when converting
        // to entropy and will panic on ambiguous word sets.
        let _ = words
            .join_revealed(" ", |phrase| Mnemonic::parse_in(*language, phrase))
            .map_err(|_| MnemonicError::InvalidPhrase)?;

        let entropy = entropy_from_words(words, language)?;

        match entropy.len() {
            16 | 20 | 24 | 28 | 32 => {}
            _ => return Err(MnemonicError::InvalidEntropySize),
        }

        Ok(Hidden::hide(entropy))
    }

    fn entropy_from_words(
        words: &SeedWords,
        language: &MnemonicLanguage,
    ) -> Result<Vec<u8>, MnemonicError> {
        let word_count = words.len();
        let total_bits = word_count * 11;
        let checksum_bits = word_count / 3;
        let entropy_bits = total_bits - checksum_bits;
        let entropy_bytes = entropy_bits / 8;

        let mut bit_buf = vec![0u8; total_bits.div_ceil(8)];
        let mut bit_pos = 0usize;
        for idx in 0..word_count {
            let word = words.get_word(idx).ok_or(MnemonicError::InvalidPhrase)?;
            let word_index = language
                .find_word(word)
                .ok_or(MnemonicError::WordNotFound)?;

            for shift in (0..11).rev() {
                let bit = ((word_index >> shift) & 1) as u8;
                let byte_idx = bit_pos / 8;
                let bit_idx = 7 - (bit_pos % 8);
                bit_buf[byte_idx] |= bit << bit_idx;
                bit_pos += 1;
            }
        }

        bit_buf.truncate(entropy_bytes);
        Ok(bit_buf)
    }

    /// Detects the language of a list of seed words.
    ///
    /// # Ambiguous Words
    ///
    /// Some words appear in multiple BIP-39 wordlists:
    /// - "abandon", "about", "above" (English + French)
    /// - "arena", "armada" (Spanish + Italian)
    ///
    /// In such cases, this function returns `Err(MnemonicError::AmbiguousLanguage)`.
    /// The user must specify the language explicitly.
    ///
    /// # Normalization
    ///
    /// Language detection relies on strict `Mnemonic::parse_in(...)` for each
    /// supported language. No approximate word matching is performed.
    ///
    /// # Domain Separation
    ///
    /// This operation uses the domain: `z00z/crypto/seed_validation`
    pub fn detect_language(words: &SeedWords) -> Result<MnemonicLanguage, MnemonicError> {
        if words.is_empty() {
            return Err(MnemonicError::Empty);
        }

        let matches = suggest_language(words);
        match matches.as_slice() {
            [only] => Ok(*only),
            [] => Err(MnemonicError::UnknownLanguage),
            _ => Err(MnemonicError::AmbiguousLanguage),
        }
    }

    /// Validates a seed phrase for minimum entropy and language consistency.
    ///
    /// # Domain Separation
    ///
    /// This operation uses the domain: `z00z/crypto/seed_validation`
    ///
    /// # Returns
    ///
    /// - `Ok((entropy_bytes, language))` if validation passes
    /// - `Err` if validation fails
    pub fn validate_seed_phrase(
        words: &SeedWords,
        language: MnemonicLanguage,
    ) -> Result<Hidden<Vec<u8>>, MnemonicError> {
        // Caller-specified language contract: validate ONLY in the provided language.
        to_bytes_with_language(words, &language)
    }
}

/// Convert entropy bytes into BIP-39 seed words.
///
/// This is a public convenience wrapper around [`mnemonic::from_bytes`], matching the
/// COMBO spec function name.
pub fn from_bytes(entropy: &[u8], language: MnemonicLanguage) -> Result<SeedWords, MnemonicError> {
    mnemonic::from_bytes(entropy, language)
}

/// Validate a seed phrase given as plain words, in an explicitly specified language.
///
/// This is a public convenience wrapper matching the COMBO spec signature.
///
/// This wrapper enforces language consistency:
/// - Detects the seed phrase language first.
/// - Rejects mismatches between detected language and the caller-provided `language`.
/// - Rejects phrases that are valid in multiple languages (ambiguous language).
///
/// # Sensitive Input Boundary
///
/// The `words` slice is caller-owned and may remain in memory after this function returns.
/// If the caller requires zeroization guarantees, convert user input into `SeedWords`
/// and avoid keeping plaintext copies longer than needed.
///
/// # Errors
///
/// - `SeedPhraseError::LanguageMismatch` if detected language does not match `language`.
/// - `SeedPhraseError::AmbiguousLanguage` if words match multiple wordlists.
/// - `SeedPhraseError::Mnemonic(MnemonicError::InvalidPhrase)` if checksum or word count is invalid.
pub fn validate_seed_phrase(
    words: &[String],
    language: MnemonicLanguage,
) -> Result<(), SeedPhraseError> {
    let hidden_words = words
        .iter()
        .map(|w| Hidden::hide(w.clone()))
        .collect::<Vec<_>>();

    let seed_words = SeedWords::new(hidden_words);

    let detected = mnemonic::detect_language(&seed_words).map_err(|err| match err {
        MnemonicError::AmbiguousLanguage => SeedPhraseError::AmbiguousLanguage,
        other => SeedPhraseError::Mnemonic(other),
    })?;
    if detected != language {
        return Err(SeedPhraseError::LanguageMismatch {
            expected: language,
            detected,
        });
    }

    let _ = mnemonic::validate_seed_phrase(&seed_words, language)?;
    Ok(())
}

/// Derive the 64-byte BIP-39 seed from a mnemonic phrase.
///
/// ⚙️ Language Policy:
/// - Caller MUST supply the language explicitly.
/// - Validation and derivation are performed only in the provided language.
pub fn mnemonic_to_seed(
    words: &SeedWords,
    passphrase: &str,
    language: MnemonicLanguage,
) -> Result<Hidden<[u8; 64]>, SeedPhraseError> {
    // Validate and normalize passphrase according to BIP-39 requirements.
    let normalized_passphrase = Zeroizing::new(validate_and_normalize_passphrase(passphrase)?);

    // Validate phrase only in the provided language (checksum included).
    let _ = mnemonic::validate_seed_phrase(words, language)?;

    let seed = words
        .join_revealed(" ", |phrase| {
            Mnemonic::parse_in(language, phrase)
                .map(|m| m.to_seed_normalized(normalized_passphrase.as_str()))
        })
        .map_err(SeedPhraseError::from)?;

    Ok(Hidden::hide(seed))
}

#[cfg(test)]
mod mnemonic_roundtrip_tests {
    use super::{mnemonic, MnemonicLanguage};

    #[test]
    fn test_entropy_roundtrip_bip39_sizes() {
        const SIZES: [usize; 5] = [16, 20, 24, 28, 32];
        for size in SIZES {
            let entropy = (0..size).map(|i| i as u8).collect::<Vec<_>>();
            let words = mnemonic::from_bytes(&entropy, MnemonicLanguage::English)
                .expect("entropy size should be accepted");
            let recovered = mnemonic::to_bytes_with_language(&words, &MnemonicLanguage::English)
                .expect("roundtrip should succeed");
            assert_eq!(recovered.reveal().as_slice(), entropy.as_slice());
        }
    }
}
