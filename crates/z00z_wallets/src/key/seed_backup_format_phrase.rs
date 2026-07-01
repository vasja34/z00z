/// A 24-word BIP-39 seed phrase.
///
/// This type stores words as `Hidden<String>` to avoid accidental logging.
/// Uses standard BIP-39: 32 bytes entropy + 1 byte checksum = 24 words.
pub struct SeedPhrase24 {
    language: MnemonicLanguage,
    words: SeedWords,
}

// ============================================================================
// BIP-39 Seed Phrase Implementation
// ============================================================================

impl SeedPhrase24 {
    /// Number of words in the phrase.
    pub fn len(&self) -> usize {
        self.words.len()
    }

    /// Returns true when the phrase has no words.
    pub fn is_empty(&self) -> bool {
        self.words.is_empty()
    }

    /// The detected/selected word list language.
    pub fn language(&self) -> MnemonicLanguage {
        self.language
    }

    /// Returns a copy of the phrase as a single space-separated hidden string.
    pub fn to_phrase(&self) -> Hidden<String> {
        self.words.join(" ")
    }

    /// Expose the phrase to a closure, zeroizing the temporary buffer afterwards.
    pub fn with_phrase<F, T>(&self, f: F) -> T
    where
        F: FnOnce(&str) -> T,
    {
        self.words.join_revealed(" ", f)
    }

    /// Create from BIP-39 entropy bytes (32 bytes).
    ///
    /// # Returns
    /// - `Ok(Self)` if entropy is valid and produces 24 words
    /// - `Err` if entropy is invalid or produces wrong word count
    pub fn from_entropy_bytes(
        entropy_bytes: &[u8; 32],
        language: MnemonicLanguage,
    ) -> Result<Self, SeedPhraseError> {
        let words = mnemonic::from_bytes(entropy_bytes, language)?;
        if words.len() != 24 {
            return Err(SeedPhraseError::InvalidWordCount {
                expected: 24,
                got: words.len(),
            });
        }
        Ok(Self { language, words })
    }

    /// Parse a 24-word phrase in an explicitly provided language.
    ///
    /// This is the preferred entry point for user-provided phrases.
    pub fn parse_in(language: MnemonicLanguage, s: &str) -> Result<Self, SeedPhraseError> {
        let phrase = s.trim();
        if phrase.is_empty() {
            return Err(SeedPhraseError::Empty);
        }

        let seed_words = SeedWords::from_str(phrase)?;
        Self::from_words(language, seed_words)
    }

    /// Construct from already-split words in an explicitly provided language.
    pub fn from_words(
        language: MnemonicLanguage,
        words: SeedWords,
    ) -> Result<Self, SeedPhraseError> {
        if words.len() != 24 {
            return Err(SeedPhraseError::InvalidWordCount {
                expected: 24,
                got: words.len(),
            });
        }

        let detected = mnemonic::detect_language(&words).map_err(|err| match err {
            MnemonicError::AmbiguousLanguage => SeedPhraseError::AmbiguousLanguage,
            other => SeedPhraseError::Mnemonic(other),
        })?;

        if detected != language {
            return Err(SeedPhraseError::LanguageMismatch {
                expected: language,
                detected,
            });
        }

        Ok(Self { language, words })
    }

    /// Convert to BIP-39 entropy bytes (32 bytes).
    ///
    /// # Returns
    /// - `Ok([u8; 32])` if phrase is valid BIP-39
    /// - `Err` if phrase is invalid or not BIP-39 compliant
    pub fn to_entropy_bytes(&self) -> Result<[u8; 32], SeedPhraseError> {
        let joined = self.words.join(" ");
        let mnemonic = joined.with_revealed(|phrase| Mnemonic::parse_in(self.language, phrase))?;
        let entropy = mnemonic.to_entropy();
        if entropy.len() != 32 {
            return Err(SeedPhraseError::InvalidEntropySize);
        }
        let mut out = [0u8; 32];
        out.copy_from_slice(&entropy);
        Ok(out)
    }

    /// Derive the 64-byte BIP-39 seed for this phrase and passphrase.
    ///
    /// This follows the BIP-39 standard (PBKDF2-HMAC-SHA512, 2048 iterations).
    ///
    /// # Security
    ///
    /// - Passphrase is limited to `MAX_PASSPHRASE_LENGTH` bytes to prevent memory exhaustion
    /// - Passphrase is normalized using NFKD as required by BIP-39 specification
    /// - Passphrase is treated as sensitive and never logged
    pub fn to_bip39_seed(&self, passphrase: &str) -> Result<Hidden<[u8; 64]>, SeedPhraseError> {
        use zeroize::Zeroizing;

        let normalized_passphrase = Zeroizing::new(validate_and_normalize_passphrase(passphrase)?);

        let joined = self.words.join(" ");
        let mnemonic = joined.with_revealed(|phrase| Mnemonic::parse_in(self.language, phrase))?;

        Ok(Hidden::hide(
            mnemonic.to_seed_normalized(normalized_passphrase.as_str()),
        ))
    }

    /// Build a 24-word BIP-39 phrase from entropy bytes.
    ///
    /// This is a BIP-39-named convenience wrapper over `from_entropy_bytes`.
    pub fn from_bip39_entropy_bytes(
        entropy_bytes: &[u8],
        language: MnemonicLanguage,
    ) -> Result<Self, SeedPhraseError> {
        if entropy_bytes.len() != 32 {
            return Err(SeedPhraseError::InvalidEntropySize);
        }
        let mut entropy = [0u8; 32];
        entropy.copy_from_slice(entropy_bytes);
        Self::from_entropy_bytes(&entropy, language)
    }

    /// Convert to BIP-39 entropy bytes.
    ///
    /// This is a BIP-39-named convenience wrapper over `to_entropy_bytes`.
    pub fn to_bip39_entropy_bytes(&self) -> Result<Vec<u8>, SeedPhraseError> {
        let entropy = self.to_entropy_bytes()?;
        Ok(entropy.to_vec())
    }
}

impl FromStr for SeedPhrase24 {
    type Err = SeedPhraseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let phrase = s.trim();
        if phrase.is_empty() {
            return Err(SeedPhraseError::Empty);
        }

        let seed_words = SeedWords::from_str(phrase)?;
        if seed_words.len() != 24 {
            return Err(SeedPhraseError::InvalidWordCount {
                expected: 24,
                got: seed_words.len(),
            });
        }

        let language = mnemonic::detect_language(&seed_words).map_err(|err| match err {
            MnemonicError::AmbiguousLanguage => SeedPhraseError::AmbiguousLanguage,
            other => SeedPhraseError::Mnemonic(other),
        })?;

        let _ = mnemonic::to_bytes_with_language(&seed_words, &language)?;
        let joined = seed_words.join(" ");
        joined.with_revealed(|phrase_str| Mnemonic::parse_in(language, phrase_str))?;

        Ok(Self {
            language,
            words: seed_words,
        })
    }
}
