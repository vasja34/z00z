#[cfg(test)]
mod tests {
    use super::*;

    fn contains_confusable_characters(s: &str) -> bool {
        if s.is_empty() {
            return false;
        }

        let has_base = s
            .nfkd()
            .any(|c| !unicode_normalization::char::is_combining_mark(c));

        !has_base
    }

    fn phrase_with_first_word(first: &str) -> String {
        format!(
            "{first} abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon",
        )
    }

    fn best_language_pair() -> (MnemonicLanguage, MnemonicLanguage) {
        let languages = MnemonicLanguage::ALL;
        let mut best_pair: Option<(MnemonicLanguage, MnemonicLanguage)> = None;
        let mut best_count = 0usize;

        for (idx, &left) in languages.iter().enumerate() {
            let left_set: std::collections::HashSet<&str> =
                left.word_list().iter().copied().collect();
            for &right in languages.iter().skip(idx + 1) {
                let shared = right
                    .word_list()
                    .iter()
                    .filter(|word| left_set.contains(*word))
                    .count();
                if shared > best_count {
                    best_count = shared;
                    best_pair = Some((left, right));
                }
            }
        }

        best_pair.expect("at least one language pair must exist")
    }

    fn find_ambiguous_seed_words(
        primary: MnemonicLanguage,
        secondary: MnemonicLanguage,
        max_counter: u32,
    ) -> SeedWords {
        let primary_set: std::collections::HashSet<&str> =
            primary.word_list().iter().copied().collect();
        let secondary_set: std::collections::HashSet<&str> =
            secondary.word_list().iter().copied().collect();

        for counter in 0u32..max_counter {
            let mut entropy = [0u8; 16];
            entropy[..4].copy_from_slice(&counter.to_le_bytes());
            let mnemonic = Mnemonic::from_entropy_in(primary, &entropy).unwrap();
            let phrase = mnemonic.to_string();
            let all_shared = phrase
                .split(' ')
                .all(|word| primary_set.contains(word) && secondary_set.contains(word));
            if all_shared && Mnemonic::parse_in(secondary, &phrase).is_ok() {
                return SeedWords::from_str(&phrase).unwrap();
            }
        }

        panic!("no ambiguous phrase found in search")
    }

    fn find_unambiguous_phrase_words(primary: MnemonicLanguage, max_counter: u32) -> Vec<String> {
        for counter in 0u32..max_counter {
            let mut entropy = [0u8; 16];
            entropy[..4].copy_from_slice(&counter.to_le_bytes());
            let mnemonic = match Mnemonic::from_entropy_in(primary, &entropy) {
                Ok(mnemonic) => mnemonic,
                Err(_) => continue,
            };

            let words = mnemonic
                .to_string()
                .split(' ')
                .map(|w| w.to_string())
                .collect::<Vec<_>>();

            let hidden_words = words
                .iter()
                .map(|w| Hidden::hide(w.clone()))
                .collect::<Vec<_>>();
            let seed_words = SeedWords::new(hidden_words);

            if matches!(mnemonic::detect_language(&seed_words), Ok(lang) if lang == primary) {
                return words;
            }
        }

        panic!("no unambiguous phrase found in search")
    }

    fn assert_unknown_language_repeated_word(word: &str) {
        let invalid_words = SeedWords::new(vec![
            Hidden::hide(word.to_string()),
            Hidden::hide(word.to_string()),
            Hidden::hide(word.to_string()),
        ]);
        let result = mnemonic::detect_language(&invalid_words);
        assert!(
            matches!(result, Err(MnemonicError::UnknownLanguage)),
            "Invalid word set should return UnknownLanguage"
        );
    }

    include!("test_seed_backup_format_basic.rs");
    include!("test_seed_backup_format_language.rs");
}
