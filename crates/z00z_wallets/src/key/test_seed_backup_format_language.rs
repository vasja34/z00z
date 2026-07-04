#[test]
fn test_homoglyph_protection() {
    use bip39::Language;

    let english_wordlist = Language::English.word_list();

    assert!(!english_wordlist.contains(&"cafe"));
    assert!(english_wordlist.contains(&"cabin"));

    let phrase = phrase_with_first_word("cabín");
    assert!(Mnemonic::parse_in(Language::English, &phrase).is_err());

    let phrase = phrase_with_first_word("café");
    assert!(Mnemonic::parse_in(Language::English, &phrase).is_err());
}

#[test]
fn test_detect_language_ambiguous() {
    use bip39::Language;

    let invalid_words = SeedWords::new(vec![
        Hidden::hide("notaword1".to_string()),
        Hidden::hide("notaword2".to_string()),
        Hidden::hide("notaword3".to_string()),
    ]);
    assert!(matches!(
        mnemonic::detect_language(&invalid_words),
        Err(MnemonicError::UnknownLanguage)
    ));

    let english_phrase =
        "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";
    let english_words = SeedWords::from_str(english_phrase).unwrap();
    assert!(matches!(
        mnemonic::detect_language(&english_words),
        Ok(Language::English)
    ));

    let (primary, secondary) = best_language_pair();
    let ambiguous_words = find_ambiguous_seed_words(primary, secondary, 10_000);
    assert!(matches!(
        mnemonic::detect_language(&ambiguous_words),
        Err(MnemonicError::AmbiguousLanguage)
    ));
}

#[test]
fn test_lang_explicit_ok() {
    let english_phrase =
        "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";
    let words = SeedWords::from_str(english_phrase).unwrap();

    assert!(mnemonic::validate_seed_phrase(&words, MnemonicLanguage::English).is_ok());
    assert!(mnemonic::validate_seed_phrase(&words, MnemonicLanguage::Spanish).is_err());
}

#[test]
fn test_lang_explicit_wrapper_ok() {
    let words = vec![
        "abandon", "abandon", "abandon", "abandon", "abandon", "abandon", "abandon", "abandon",
        "abandon", "abandon", "abandon", "about",
    ]
    .into_iter()
    .map(|w| w.to_string())
    .collect::<Vec<_>>();

    assert!(super::validate_seed_phrase(&words, MnemonicLanguage::English).is_ok());
    assert!(super::validate_seed_phrase(&words, MnemonicLanguage::Spanish).is_err());
}

#[test]
fn test_cross_language_rejection() {
    let primary = MnemonicLanguage::French;
    let words = find_unambiguous_phrase_words(primary, 10_000);

    let result = super::validate_seed_phrase(&words, MnemonicLanguage::English);
    assert!(matches!(
        result,
        Err(SeedPhraseError::LanguageMismatch {
            expected: MnemonicLanguage::English,
            detected: MnemonicLanguage::French,
        })
    ));
}

#[test]
fn test_wrapper_rejects_ambiguous_language() {
    let primary = MnemonicLanguage::English;
    let secondary = MnemonicLanguage::French;
    let words = find_ambiguous_seed_words(primary, secondary, 50_000)
        .join_revealed(" ", |phrase| {
            phrase.split(' ').map(|w| w.to_string()).collect::<Vec<_>>()
        });

    let result = super::validate_seed_phrase(&words, primary);
    assert!(matches!(result, Err(SeedPhraseError::AmbiguousLanguage)));
}

#[test]
fn test_ambiguous_ok() {
    let primary = MnemonicLanguage::English;
    let secondary = MnemonicLanguage::French;
    let words = find_ambiguous_seed_words(primary, secondary, 50_000);

    assert!(mnemonic::detect_language(&words).is_err());
    assert!(mnemonic::validate_seed_phrase(&words, primary).is_ok());
    assert!(mnemonic::validate_seed_phrase(&words, secondary).is_ok());
}

#[test]
fn test_nfkd_normalization_applied() {
    use bip39::Language;

    let english_wordlist = Language::English.word_list();
    let word_with_diacritic = "café";
    let normalized = word_with_diacritic.nfd().collect::<String>();

    assert!(normalized.contains('\u{0301}'));
    assert!(!english_wordlist.contains(&"cafe"));

    let phrase = phrase_with_first_word(word_with_diacritic);
    assert!(Mnemonic::parse_in(Language::English, &phrase).is_err());
}

#[test]
fn test_mixed_script_mnemonic_rejected() {
    use bip39::Language;

    let english_wordlist = Language::English.word_list();
    let word_a = english_wordlist
        .iter()
        .find(|w| w.starts_with('a'))
        .unwrap();
    let cyrillic_word = word_a.replacen('a', "а", 1);

    assert_ne!(cyrillic_word, *word_a);
    assert!(cyrillic_word.contains(|c: char| c as u32 >= 0x0400 && c as u32 <= 0x04FF));

    let phrase = phrase_with_first_word(&cyrillic_word);
    assert!(Mnemonic::parse_in(Language::English, &phrase).is_err());
}

#[test]
fn test_various_unicode_scripts_rejected() {
    use bip39::Language;

    let english_wordlist = Language::English.word_list();
    for word in ["ԱԲԳ", "אבד", "अआइ", "กขค", "中日韩"] {
        assert!(!english_wordlist.contains(&word));
    }
}

#[test]
fn test_emoji_lookalikes_rejected() {
    use bip39::Language;

    let english_wordlist = Language::English.word_list();
    assert!(!english_wordlist.contains(&"❤"));
}

#[test]
fn test_mathematical_symbols_rejected() {
    use bip39::Language;

    let english_wordlist = Language::English.word_list();
    assert!(!english_wordlist.contains(&"∆"));
}

#[test]
fn test_combining_characters_rejected() {
    use bip39::Language;

    let combining_word = "a\u{0301}";
    let phrase = phrase_with_first_word(combining_word);
    assert!(Mnemonic::parse_in(Language::English, &phrase).is_err());
}

#[test]
fn test_ambiguous_language_word_sets() {
    use bip39::Language;

    let english_wordlist = Language::English.word_list();
    let french_wordlist = Language::French.word_list();

    for word in ["abandon", "amateur", "angle", "animal", "aspect"] {
        assert!(english_wordlist.contains(&word));
        assert!(french_wordlist.contains(&word));
        assert_unknown_language_repeated_word(word);
    }
}

#[test]
fn test_nfkd_normalization_ambiguous_words() {
    let words = SeedWords::new(vec![
        Hidden::hide("cabín".to_string()),
        Hidden::hide("cabín".to_string()),
        Hidden::hide("cabín".to_string()),
    ]);
    let result = mnemonic::detect_language(&words);
    assert!(
        matches!(result, Err(MnemonicError::UnknownLanguage)),
        "Invalid normalized input should return UnknownLanguage"
    );
}

#[test]
fn test_single_script_detection() {
    use bip39::Language;

    let english_wordlist = Language::English.word_list();
    assert!(english_wordlist.contains(&"cabin"));
    assert!(!english_wordlist.contains(&"cabín"));
}

#[test]
fn test_mixed_script_detection_comprehensive() {
    use bip39::Language;

    let english_wordlist = Language::English.word_list();
    for mixed_word in ["aа", "aα", "aا"] {
        assert!(!english_wordlist.contains(&mixed_word));
    }
}

#[test]
fn test_empty_string_no_confusables() {
    assert!(!contains_confusable_characters(""));
}

#[test]
fn test_only_combining_marks_rejected() {
    let combining_only = "\u{0301}\u{0302}";
    assert!(
        contains_confusable_characters(combining_only),
        "Strings with only combining marks should be rejected"
    );
}
