#[test]
fn test_seed_words_join_passes() {
    let words = vec![
        Hidden::hide("one".to_string()),
        Hidden::hide("two".to_string()),
    ];
    let seed_words = SeedWords::new(words);

    let out = seed_words.join_revealed(" ", |phrase| phrase.to_string());
    assert_eq!(out, "one two");
}

#[test]
fn test_seed_phrase_with_passes() {
    let phrase_str = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon art";
    let seed_phrase = SeedPhrase24::parse_in(MnemonicLanguage::English, phrase_str).unwrap();

    seed_phrase.with_phrase(|phrase| {
        assert_eq!(phrase, phrase_str);
    });
}

#[test]
fn test_roundtrip_32_bytes() {
    let bytes = [7u8; 32];
    let phrase = SeedPhrase24::from_entropy_bytes(&bytes, MnemonicLanguage::English).unwrap();
    assert_eq!(phrase.len(), 24);

    let decoded = phrase.to_entropy_bytes().unwrap();
    assert_eq!(decoded, bytes);
}

#[test]
fn test_to_entropy_bytes_roundtrip() {
    let bytes = [42u8; 32];
    let phrase = SeedPhrase24::from_entropy_bytes(&bytes, MnemonicLanguage::English).unwrap();

    let entropy = phrase.to_entropy_bytes().unwrap();
    assert_eq!(entropy, bytes);

    let decoded = phrase.to_bip39_entropy_bytes().unwrap();
    assert_eq!(decoded, bytes.to_vec());
}

#[test]
fn test_bip39_entropy_roundtrips() {
    for len in [16usize, 20, 24, 28, 32] {
        let entropy: Vec<u8> = (0..len).map(|i| (i & 0xFF) as u8).collect();
        let words = mnemonic::from_bytes(&entropy, MnemonicLanguage::English).unwrap();
        let decoded = mnemonic::to_bytes_with_language(&words, &MnemonicLanguage::English).unwrap();

        decoded.with_revealed(|bytes| {
            assert_eq!(bytes.as_slice(), entropy.as_slice());
        });
    }
}

#[test]
fn test_bip39_vector_zero_entropy() {
    use bip39::Language;

    let entropy = [0u8; 16];
    let words = mnemonic::from_bytes(&entropy, Language::English).unwrap();
    let phrase = words.join(" ");
    phrase.with_revealed(|s| {
        assert_eq!(
            s,
            "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about"
        );
    });

    let parsed_words = SeedWords::from_str(
        "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about",
    )
    .unwrap();
    let decoded = mnemonic::to_bytes_with_language(&parsed_words, &Language::English).unwrap();
    decoded.with_revealed(|bytes| {
        assert_eq!(bytes.as_slice(), entropy.as_slice());
    });
}

#[test]
fn test_parse_rejects_word_count() {
    let err = match SeedPhrase24::parse_in(MnemonicLanguage::English, "abandon abandon") {
        Ok(_) => panic!("expected parse to fail"),
        Err(e) => e,
    };
    assert!(matches!(err, SeedPhraseError::InvalidWordCount { .. }));
}

#[test]
fn test_bip39_seed_passphrase_sensitive() {
    let phrase = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon art";

    let seed_phrase = SeedPhrase24::parse_in(MnemonicLanguage::English, phrase).unwrap();
    let seed_a = seed_phrase.to_bip39_seed("").unwrap().with_revealed(|s| *s);
    let seed_b = seed_phrase.to_bip39_seed("").unwrap().with_revealed(|s| *s);
    assert_eq!(seed_a, seed_b);

    let seed_c = seed_phrase
        .to_bip39_seed("TREZOR")
        .unwrap()
        .with_revealed(|s| *s);
    assert_ne!(seed_a, seed_c);
}

#[test]
fn test_passphrase_length_validation() {
    let phrase = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon art";
    let seed_phrase = SeedPhrase24::parse_in(MnemonicLanguage::English, phrase).unwrap();

    let valid_passphrase = "a".repeat(128);
    let result = seed_phrase.to_bip39_seed(&valid_passphrase);
    assert!(result.is_ok());

    let too_long_passphrase = "a".repeat(129);
    let result = seed_phrase.to_bip39_seed(&too_long_passphrase);
    assert!(matches!(
        result,
        Err(SeedPhraseError::PassphraseTooLong { .. })
    ));

    let result = seed_phrase.to_bip39_seed("");
    assert!(result.is_ok());
}

#[test]
fn test_passphrase_too_before_nfkd() {
    let long = "a".repeat(MAX_PASSPHRASE_LENGTH + 1);
    assert!(matches!(
        validate_and_normalize_passphrase(&long),
        Err(SeedPhraseError::PassphraseTooLong { .. })
    ));
}

#[test]
fn test_passphrase_expands_during_nfkd() {
    let composed = "\u{00E9}".repeat(MAX_PASSPHRASE_LENGTH / 2);
    assert_eq!(composed.len(), MAX_PASSPHRASE_LENGTH);

    let result = validate_and_normalize_passphrase(&composed);
    assert!(matches!(
        result,
        Err(SeedPhraseError::PassphraseTooLong {
            length: _,
            max: MAX_PASSPHRASE_LENGTH,
        })
    ));
}

#[test]
fn test_valid_passphrase_with_accents() {
    let passphrase = "caf\u{00E9} r\u{00E9}sum\u{00E9}";
    let normalized = validate_and_normalize_passphrase(passphrase).unwrap();
    assert_ne!(normalized, passphrase);
    assert!(normalized.contains('\u{0301}'));
}

#[test]
fn test_max_passphrase_constant() {
    assert_eq!(MAX_PASSPHRASE_LENGTH, 128);
}

#[test]
fn test_unicode_passphrase_accepted() {
    let phrase = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon art";
    let seed_phrase = phrase.parse::<SeedPhrase24>().unwrap();

    let unicode_passphrase = "café";
    let result = seed_phrase.to_bip39_seed(unicode_passphrase);
    assert!(result.is_ok(), "Unicode passphrase should be accepted");

    let emoji_passphrase = "🔐🔑🛡️";
    let result = seed_phrase.to_bip39_seed(emoji_passphrase);
    assert!(result.is_ok(), "Emoji passphrase should be accepted");
}

#[test]
fn test_empty_passphrase_accepted() {
    let phrase = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon art";
    let seed_phrase = phrase.parse::<SeedPhrase24>().unwrap();

    let result = seed_phrase.to_bip39_seed("");
    assert!(
        result.is_ok(),
        "Empty passphrase should be accepted (BIP-39 allows empty passphrases)"
    );
}

#[test]
fn test_unicode_byte_limit() {
    let phrase = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon art";
    let seed_phrase = phrase.parse::<SeedPhrase24>().unwrap();

    let unit = "é";
    let unit_normalized = unit.nfkd().collect::<String>();
    let unit_norm_bytes = unit_normalized.len();

    let exact_count = MAX_PASSPHRASE_LENGTH / unit_norm_bytes;
    let exact_passphrase = unit.repeat(exact_count);
    let result = seed_phrase.to_bip39_seed(&exact_passphrase);
    assert!(
        result.is_ok(),
        "Exact byte-length passphrase should be accepted"
    );

    let too_long_passphrase = unit.repeat(exact_count + 1);
    let result = seed_phrase.to_bip39_seed(&too_long_passphrase);
    assert!(
        matches!(result, Err(SeedPhraseError::PassphraseTooLong { .. })),
        "Byte-length overflow should be rejected"
    );
}

#[test]
#[cfg(not(target_arch = "wasm32"))]
fn test_auth_err_has_jitter() {
    use z00z_utils::time::Instant;

    let mut timings = vec![];

    for _ in 0..30 {
        let start = Instant::now();
        let _ = CipherSeedContainer::auth_err();
        timings.push(start.elapsed().as_millis());
    }

    let min = *timings.iter().min().unwrap();
    let max = *timings.iter().max().unwrap();
    assert!(
        min >= 80,
        "Minimum timing must be >= 80ms, got: {:?}",
        timings
    );
    assert!(
        max <= 500,
        "Maximum timing must be <= 500ms (sanity bound), got: {:?}",
        timings
    );

    assert!(
        max > min + 10,
        "Insufficient jitter variance: min={}, max={}, expected >10ms spread",
        min,
        max
    );
}
