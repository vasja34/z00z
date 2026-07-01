//! Z00Z-owned seed phrase primitives.
//!
//! 📌 Goal:
//! - Provide a stable mnemonic/seed-word toolkit under `z00z_crypto`.
//! - Prefer maintainability: use the upstream `bip39` crate wordlists.
//!
//! 🔒 SECURITY:
//! - All seed words are stored in `Hidden<String>` to prevent accidental logging
//! - Mnemonic phrases must never be stored or returned as plain `String` in internal APIs
//! - Implements `Zeroize` to clear memory on drop
//! - Validates minimum entropy (128 bits) to prevent weak seeds
//! - Uses domain-separated operations for cryptographic security
//! - Does NOT strip diacritics/combining marks as an acceptance mechanism (strict BIP-39 wordlist matching)
//!
//! ⚙️ Language Policy:
//! - Callers MUST specify the mnemonic language explicitly when validating or deriving a seed.
//! - Validation is performed only in the provided language; it never rejects phrases just because
//!   they are ambiguous across multiple wordlists.
//! - Use `mnemonic::suggest_language()` when the language is unknown.
//! - Prefer `SeedPhrase24::parse_in(...)` over `.parse()` to avoid implicit language detection.
//!
//! 🔎 Mixed-Script Note:
//! - Mixed-script / homoglyph protection is NOT a production validation layer.
//! - Production validation relies on the BIP-39 wordlists (`bip39::Mnemonic::parse_in(...)`).
//! - Any mixed-script examples in tests are wordlist integrity checks, not additional runtime policy.
//!
//! ## 🔐 KDF Parameters (Password-Based Key Derivation)
//!
//! **OWASP 2023 Argon2id Recommendations:**
//! - **Minimum for any device:** 128 MiB memory, 2 iterations, 1 thread
//! - **Recommended for desktop:** 256 MiB memory, 3 iterations, 4 threads
//!
//! **Z00Z Implementation:**
//! - `Argon2idParams::MOBILE`: 128 MiB, 4 iterations, 4 threads (meets OWASP 2023 minimum)
//! - `Argon2idParams::DEFAULT`: 128 MiB, 4 iterations, 4 threads (balanced)
//! - `Argon2idParams::HIGH_SECURITY`: 256 MiB, 8 iterations, 16 threads (high security)
//!
//! **Reference:** <https://cheatsheetseries.owasp.org/cheatsheets/Password_Storage_Cheat_Sheet.html#argon2id>

use std::{fmt, str::FromStr};

use subtle::{Choice, ConstantTimeEq};
use thiserror::Error;
use unicode_normalization::UnicodeNormalization;
use zeroize::{Zeroize, Zeroizing};

use bip39::Mnemonic;
use z00z_core::genesis::ChainType;
use z00z_crypto::expert::encoding::SafePassword;
use z00z_crypto::kdf::Argon2Params;
use z00z_crypto::{DomainHasher, Hidden};
use z00z_utils::logger::{Logger, TracingLogger};

use crate::domains::CipherSeedAadTagDomain;
use crate::rpc::types::common::RuntimeValidationResult;

include!("seed_entropy.rs");

/// Maximum allowed passphrase length to prevent memory exhaustion attacks.
/// Reduced from 256 to 128 for DoS protection (6.4 KB → 3.2 KB in UTF-8).
pub const MAX_PASSPHRASE_LENGTH: usize = 128;
/// Maximum allowed wallet_id length (u8 length prefix).
pub const MAX_WALLET_ID_LEN: usize = u8::MAX as usize;
/// Maximum allowed purpose length (u8 length prefix).
pub const MAX_PURPOSE_LEN: usize = u8::MAX as usize;

/// Supported mnemonic languages.
pub type MnemonicLanguage = bip39::Language;

include!("seed_mnemonic.rs");
include!("seed_cipher.rs");
include!("seed_backup_format.rs");
