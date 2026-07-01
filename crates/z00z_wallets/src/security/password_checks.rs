impl PasswordValidator {
    fn denylist_might_contain(candidate_ascii_lower: &str) -> bool {
        if candidate_ascii_lower.is_empty() {
            return false;
        }

        let m_bits = (Self::DENYLIST_BLOOM.len() as u64) * 8;
        let bytes = candidate_ascii_lower.as_bytes();
        let h1 = Self::denylist_h64(0, bytes);
        let h2 = Self::denylist_h64(1, bytes);

        for i in 0..Self::DENYLIST_BLOOM_K {
            let bit_index = h1.wrapping_add(i.wrapping_mul(h2)) % m_bits;
            let byte_index = (bit_index / 8) as usize;
            let mask = 1u8 << (bit_index % 8);

            if (Self::DENYLIST_BLOOM[byte_index] & mask) == 0 {
                return false;
            }
        }

        true
    }

    fn denylist_h64(prefix: u8, input: &[u8]) -> u64 {
        let out = compute_password_bloom(prefix, input);
        let mut bytes = [0u8; 8];
        bytes.copy_from_slice(&out[..8]);
        u64::from_le_bytes(bytes)
    }

    fn leet_normalize_ascii(s: &str) -> String {
        s.chars()
            .map(|c| match c {
                '0' => 'o',
                '1' => 'l',
                '3' => 'e',
                '5' | '$' => 's',
                '7' => 't',
                '@' => 'a',
                _ => c,
            })
            .collect()
    }

    fn strip_common_suffix(s: &str) -> &str {
        let mut cut = s.len();
        let mut removed = 0usize;
        for (idx, ch) in s.char_indices().rev() {
            if removed >= 6 {
                break;
            }
            if ch.is_ascii_digit() || (!ch.is_ascii_alphanumeric() && !ch.is_whitespace()) {
                cut = idx;
                removed += 1;
                continue;
            }
            break;
        }
        &s[..cut]
    }

    fn is_denylisted(&self, password: &str) -> bool {
        let trimmed = password.trim();
        if trimmed.is_empty() {
            return false;
        }

        let ascii_lower = trimmed.to_ascii_lowercase();
        if Self::denylist_might_contain(&ascii_lower) {
            return true;
        }
        let stripped = Self::strip_common_suffix(&ascii_lower);
        if stripped != ascii_lower && Self::denylist_might_contain(stripped) {
            return true;
        }

        let leet = Self::leet_normalize_ascii(&ascii_lower);
        if Self::denylist_might_contain(&leet) {
            return true;
        }
        let stripped_leet = Self::strip_common_suffix(&leet);
        if stripped_leet != leet && Self::denylist_might_contain(stripped_leet) {
            return true;
        }

        false
    }

    fn has_only_one_class(password: &str) -> bool {
        let mut has_alpha = false;
        let mut has_digit = false;
        let mut has_other = false;

        for c in password.chars() {
            if c.is_ascii_alphabetic() {
                has_alpha = true;
            } else if c.is_ascii_digit() {
                has_digit = true;
            } else if !c.is_whitespace() {
                has_other = true;
            }
        }

        let classes = (has_alpha as u8) + (has_digit as u8) + (has_other as u8);
        classes <= 1
    }

    fn max_same_char_run(password: &str) -> usize {
        let mut best = 0usize;
        let mut current = 0usize;
        let mut last: Option<char> = None;

        for c in password.chars() {
            if Some(c) == last {
                current += 1;
            } else {
                current = 1;
                last = Some(c);
            }
            best = best.max(current);
        }

        best
    }

    fn uniqueness_metrics(password: &str) -> (usize, f64) {
        let chars: Vec<char> = password.chars().filter(|c| !c.is_whitespace()).collect();
        if chars.is_empty() {
            return (0, 0.0);
        }
        let unique: HashSet<char> = chars.iter().copied().collect();
        (unique.len(), (unique.len() as f64) / (chars.len() as f64))
    }

    fn has_sequence_run_gte(password: &str, min_run: usize) -> bool {
        let mut run = 1usize;
        let mut prev: Option<i32> = None;

        for c in password.chars() {
            let value = if c.is_ascii_digit() {
                Some((c as u8 - b'0') as i32)
            } else if c.is_ascii_lowercase() {
                Some((c as u8 - b'a') as i32)
            } else {
                None
            };

            match (prev, value) {
                (Some(previous), Some(current)) if (current - previous).abs() == 1 => {
                    run += 1;
                    if run >= min_run {
                        return true;
                    }
                }
                (_, Some(_)) => {
                    run = 1;
                }
                _ => {
                    run = 1;
                }
            }

            prev = value;
        }

        false
    }

    fn has_repeated_substring_period_lte(password: &str, max_period: usize) -> bool {
        let bytes = password.as_bytes();
        for period in 2..=max_period {
            if bytes.len() < period * 3 {
                continue;
            }
            for start in 0..=bytes.len().saturating_sub(period * 3) {
                let a = &bytes[start..start + period];
                let b = &bytes[start + period..start + period * 2];
                let c = &bytes[start + period * 2..start + period * 3];
                if a == b && b == c {
                    return true;
                }
            }
        }
        false
    }

    fn has_keyboard_walk(password: &str) -> bool {
        const ROWS: [&str; 5] = [
            "qwertyuiop",
            "asdfghjkl",
            "zxcvbnm",
            "0123456789",
            "1q2w3e4r5t6y7u8i9o0p",
        ];

        for row in ROWS {
            let row_rev: String = row.chars().rev().collect();
            for window in 4..=6 {
                if row.len() < window {
                    continue;
                }
                for i in 0..=(row.len() - window) {
                    let pat = &row[i..i + window];
                    if password.contains(pat) {
                        return true;
                    }
                    let pat_rev = &row_rev[row.len() - i - window..row.len() - i];
                    if password.contains(pat_rev) {
                        return true;
                    }
                }
            }
        }

        false
    }

    fn looks_date_like(password: &str) -> bool {
        static DATE_RE: Lazy<Regex> = Lazy::new(|| {
            Regex::new(
                r"(?x)
                (?:\b\d{8}\b)
                |
                (?:\b\d{4}[-/._]\d{2}[-/._]\d{2}\b)
                |
                (?:\b\d{2}[-/._]\d{2}[-/._]\d{4}\b)
            ",
            )
            .expect("valid regex")
        });

        DATE_RE.is_match(password)
    }

    /// Detect a common bypass pattern: lowercase word + short numeric/symbol tail.
    pub fn is_word_short_tail(password: &str) -> bool {
        let mut iter = password.char_indices();

        let mut word_end = 0usize;
        let mut word_len = 0usize;
        for (idx, ch) in iter.by_ref() {
            if ch.is_ascii_lowercase() {
                word_len += 1;
                word_end = idx + ch.len_utf8();
                continue;
            }
            break;
        }

        if word_len < 3 {
            return false;
        }

        let tail = &password[word_end..];
        if tail.is_empty() || tail.chars().count() > 6 {
            return false;
        }

        let mut digits = 0usize;
        let mut symbols = 0usize;
        for ch in tail.chars() {
            if ch.is_ascii_digit() {
                digits += 1;
                continue;
            }
            if !ch.is_ascii_alphanumeric() && !ch.is_whitespace() {
                symbols += 1;
                continue;
            }
            return false;
        }

        (1..=4).contains(&digits) && (1..=2).contains(&symbols)
    }

    fn classes_count(password: &str) -> u8 {
        let has_upper = password.chars().any(|c| c.is_uppercase());
        let has_lower = password.chars().any(|c| c.is_lowercase());
        let has_digit = password.chars().any(|c| c.is_ascii_digit());
        let has_special = password.chars().any(|c| !c.is_alphanumeric());
        (has_upper as u8) + (has_lower as u8) + (has_digit as u8) + (has_special as u8)
    }

    fn estimate_log10_guesses(password: &str) -> f64 {
        let len = password.chars().filter(|c| !c.is_whitespace()).count() as f64;
        if len <= 0.0 {
            return 0.0;
        }

        let mut charset: f64 = 0.0;
        if password.chars().any(|c| c.is_ascii_lowercase()) {
            charset += 26.0;
        }
        if password.chars().any(|c| c.is_ascii_uppercase()) {
            charset += 26.0;
        }
        if password.chars().any(|c| c.is_ascii_digit()) {
            charset += 10.0;
        }
        if password
            .chars()
            .any(|c| !c.is_alphanumeric() && !c.is_whitespace())
        {
            charset += 32.0;
        }
        if password.chars().any(|c| c.is_whitespace()) {
            charset += 1.0;
        }

        if charset <= 1.0 {
            return 0.0;
        }

        let (_, unique_ratio) = Self::uniqueness_metrics(password);
        let effective_len = (len * unique_ratio.clamp(0.1, 1.0)).max(1.0);
        effective_len * charset.log10()
    }
}
