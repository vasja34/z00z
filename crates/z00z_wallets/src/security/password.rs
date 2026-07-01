use std::collections::HashSet;

use once_cell::sync::Lazy;
use regex::Regex;
use serde::{Deserialize, Serialize};

use crate::domains::hashing::compute_password_bloom;
use crate::rpc::types::common::RuntimeValidationResult;

/// Password strength requirements (DTO-only).
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct RuntimePasswordPolicy {
    /// Policy version (for future migration / client UI messaging).
    pub version: u32,
    /// Minimum password length.
    pub min_length: usize,
    /// Recommended password length (for UI strength guidance).
    pub recommended_length: usize,
    /// Maximum accepted length.
    pub max_length: usize,
}

impl Default for RuntimePasswordPolicy {
    fn default() -> Self {
        Self {
            version: 1,
            min_length: 14,
            recommended_length: 20,
            max_length: 128,
        }
    }
}

/// Password validation and scoring service.
#[derive(Debug, Clone, Copy)]
pub struct PasswordValidator {
    policy: RuntimePasswordPolicy,
}

impl Default for PasswordValidator {
    fn default() -> Self {
        Self::new(RuntimePasswordPolicy::default())
    }
}

impl PasswordValidator {
    // Per specs/006-z00z-wallets/validation-rules-very-strong.md:
    // ship a denylist without keeping a large plaintext list in the binary.
    const DENYLIST_BLOOM: &'static [u8] = include_bytes!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/src/config/password_denylist.bloom"
    ));
    const DENYLIST_BLOOM_K: u64 = 7;

    /// Create a validator for a specific policy.
    pub fn new(policy: RuntimePasswordPolicy) -> Self {
        Self { policy }
    }

    /// Return the currently configured policy.
    pub fn policy(&self) -> RuntimePasswordPolicy {
        self.policy
    }

    /// Compute deterministic password strength score.
    ///
    /// Score is a pure function of password length and character classes.
    /// Range: 0..=100.
    pub fn strength_score(&self, password: &str) -> u8 {
        let mut score: u32 = 0;

        // Length contribution (0..=60).
        // 14 chars => 21 points; 20+ chars => 60 points.
        let len = password.chars().count() as u32;
        score += (len.saturating_mul(3)).min(60);

        // Character classes contribution (0..=40).
        let has_upper = password.chars().any(|c| c.is_uppercase());
        let has_lower = password.chars().any(|c| c.is_lowercase());
        let has_digit = password.chars().any(|c| c.is_ascii_digit());
        let has_special = password.chars().any(|c| !c.is_alphanumeric());

        score += (has_upper as u32) * 10;
        score += (has_lower as u32) * 10;
        score += (has_digit as u32) * 10;
        score += (has_special as u32) * 10;

        score.min(100) as u8
    }

    /// Validate a password and return a serializable result.
    pub fn validate_result(&self, password: &str) -> RuntimeValidationResult {
        match self.validate(password) {
            Ok(()) => RuntimeValidationResult::valid(),
            Err(msg) => RuntimeValidationResult::invalid(msg),
        }
    }

    /// Validate password against policy.
    pub fn validate(&self, password: &str) -> Result<(), &'static str> {
        // Reject leading/trailing whitespace (spec).
        if password != password.trim() {
            return Err("Password cannot have leading/trailing whitespace");
        }

        let length = password.chars().count();
        if length < self.policy.min_length {
            return Err("Password too short");
        }
        if length > self.policy.max_length {
            return Err("Password too long");
        }

        // Reject immediately if denylisted (Bloom filter).
        if self.is_denylisted(password) {
            return Err("Password is too common");
        }

        // Accept either a good passphrase or a good complex password.
        let words: Vec<&str> = password
            .split_whitespace()
            .filter(|w| !w.is_empty())
            .collect();
        let unique_words: HashSet<String> = words.iter().map(|w| w.to_ascii_lowercase()).collect();

        let passphrase_ok = length >= 20
            && words.len() >= 4
            && words.iter().all(|w| !w.is_empty())
            && unique_words.len() >= 4;

        // Weak patterns: repeats and low uniqueness.
        if Self::max_same_char_run(password) > 3 {
            return Err("Password has repeated characters");
        }

        let (unique_chars, unique_ratio) = Self::uniqueness_metrics(password);
        if unique_chars < 6 || unique_ratio < 0.35 {
            return Err("Password has low uniqueness");
        }

        // Normalize leetspeak and re-run detectors.
        let lowered = password.to_ascii_lowercase();
        let normalized = Self::leet_normalize_ascii(&lowered);

        if Self::has_sequence_run_gte(&normalized, 5) {
            return Err("Password contains sequences");
        }
        if Self::has_keyboard_walk(&normalized) {
            return Err("Password contains keyboard patterns");
        }
        if Self::has_repeated_substring_period_lte(&normalized, 4) {
            return Err("Password contains repeated patterns");
        }
        if Self::looks_date_like(&normalized) {
            return Err("Password looks like a date");
        }
        if Self::is_word_short_tail(&lowered) || Self::is_word_short_tail(&normalized) {
            return Err("Password matches a common pattern");
        }

        if passphrase_ok {
            let log10_guesses = Self::estimate_log10_guesses(password);
            if log10_guesses < 10.0 {
                return Err("Password is too guessable");
            }
            return Ok(());
        }

        // Weak patterns: single-class passwords (applies to non-passphrases).
        if Self::has_only_one_class(password) {
            return Err("Password is too simple");
        }

        // Complex password mode.
        if Self::classes_count(password) < 3 {
            return Err("Password must mix character classes");
        }

        if unique_chars < 8 || unique_ratio < 0.40 {
            return Err("Password has low uniqueness");
        }

        if Self::max_same_char_run(password) > 3 {
            return Err("Password has repeated characters");
        }

        let log10_guesses = Self::estimate_log10_guesses(password);
        if log10_guesses < 10.0 {
            return Err("Password is too guessable");
        }
        Ok(())
    }
}

include!("password_checks.rs");

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_policy_rejects_common() {
        let validator = PasswordValidator::default();
        assert!(validator.validate("Password123!").is_err());
    }

    #[test]
    fn test_policy_rejects_whitespace() {
        let validator = PasswordValidator::default();
        let err = validator.validate(" StrongPassw0rd! ").unwrap_err();
        assert_eq!(err, "Password cannot have leading/trailing whitespace");
    }

    #[test]
    fn test_passphrase_accepts_short_words() {
        let validator = PasswordValidator::default();
        // Includes 1-letter word "a"; should be acceptable per very-strong passphrase mode.
        let pw = "She is baking a cake";
        assert!(validator.validate(pw).is_ok());
    }

    #[test]
    fn test_rejects_word_tail() {
        let validator = PasswordValidator::default();
        assert!(PasswordValidator::is_word_short_tail("longpassword2025!"));
        assert!(validator.validate("LongPassword2025!").is_err());
    }
}
