//! Payment Reference (PayRef) for Z00Z.
//!
//! A PayRef is a deterministic, verifiable 32-byte identifier useful for:
//! - receipts and audits
//! - offline payment proofs
//! - correlating outputs without leaking wallet internals
//!
//! PayRef is derived from public transaction context (e.g. block hash + output hash)
//! using domain-separated hashing.
//!
//! ## Uniqueness Guarantees
//!
//! PayRef uniqueness depends on:
//! - **Block hash uniqueness**: Cryptographic hash of block header
//! - **Output hash uniqueness**: Cryptographic hash of output
//! - **Domain separation**: PAYREF_DOMAIN prevents cross-protocol collisions
//!
//! **Collision probability**: ~2^-256 (effectively zero)
//!
//! ## Use Cases
//!
//! ### Receipts
//! Store PayRef as receipt identifier. User can verify payment by checking blockchain.
//!
//! ### Offline Proofs
//! Prove payment without full transaction data. Just provide PayRef + signature.
//!
//! ### Correlation
//! Link outputs across different wallets without revealing wallet structure.
//!
//! ## Format
//!
//! - **Full**: 64 hex characters (0-9a-f)
//! - **Short**: 8 chars + "…" + 8 chars (17 total)
//! - **Compact**: 8 chars + "…" + 8 chars (for display)

use z00z_crypto::CryptoError;

use crate::domains::hashing::compute_pay_ref;

/// Size (bytes) of a PayRef.
pub const PAY_REF_SIZE: usize = 32;

/// Domain separation tag for PayRef generation.
///
/// This constant is part of the on-wire contract.
pub const PAY_REF_DOMAIN: &str = "z00z.wallets.tx.pay_ref.v1";

/// Generate a PayRef from `(block_hash, output_hash)`.
///
/// Both inputs are expected to be 32-byte hashes.
pub fn generate(block_hash: &[u8; 32], output_hash: &[u8; 32]) -> [u8; PAY_REF_SIZE] {
    compute_pay_ref(block_hash, output_hash)
}

/// Parse a PayRef from a hex string.
///
/// Accepted formats:
/// - 64 hex characters
/// - optional "0x" prefix
pub fn parse_hex(s: &str) -> Result<[u8; PAY_REF_SIZE], CryptoError> {
    let s = s.strip_prefix("0x").unwrap_or(s);
    if s.len() != PAY_REF_SIZE * 2 {
        return Err(CryptoError::InvalidParameters {
            param: "payref_hex_length",
        });
    }

    let mut out = [0u8; PAY_REF_SIZE];
    for (i, byte) in out.iter_mut().enumerate() {
        let hi = decode_hex_nibble(s.as_bytes()[2 * i])?;
        let lo = decode_hex_nibble(s.as_bytes()[2 * i + 1])?;
        *byte = (hi << 4) | lo;
    }
    Ok(out)
}

/// Verify a PayRef against the original inputs.
///
/// Returns true if the PayRef matches the generated value from block_hash and output_hash.
///
/// # Example
///
/// ```
/// use z00z_wallets::tx::pay_ref::{generate, verify};
///
/// let block_hash = [1u8; 32];
/// let output_hash = [2u8; 32];
/// let payref = generate(&block_hash, &output_hash);
///
/// assert!(verify(&payref, &block_hash, &output_hash));
/// assert!(!verify(&payref, &[3u8; 32], &output_hash));
/// ```
pub fn verify(payref: &[u8; PAY_REF_SIZE], block_hash: &[u8; 32], output_hash: &[u8; 32]) -> bool {
    generate(block_hash, output_hash) == *payref
}

/// Format a PayRef as a full hex string.
///
/// Returns 64 lowercase hex characters (no prefix).
///
/// # Example
///
/// ```
/// use z00z_wallets::tx::pay_ref::format_full;
///
/// let payref = [0xABu8; 32];
/// let full = format_full(&payref);
/// assert_eq!(full.len(), 64);
/// assert!(full.chars().all(|c| c.is_ascii_hexdigit()));
/// ```
pub fn format_full(payref: &[u8; PAY_REF_SIZE]) -> String {
    to_lower_hex(payref)
}

/// Format a PayRef as a short, human-friendly string.
///
/// Format: first 8 hex chars + "…" + last 8 hex chars.
/// Total: 17 characters (8 + 1 + 8).
///
/// # Example
///
/// ```
/// use z00z_wallets::tx::pay_ref::format_short;
///
/// let payref = [0x12u8; 32];
/// let short = format_short(&payref);
/// assert_eq!(short, "12121212…12121212");
/// ```
pub fn format_short(payref: &[u8; PAY_REF_SIZE]) -> String {
    let full = to_lower_hex(payref);
    let head = &full[..8];
    let tail = &full[full.len() - 8..];
    format!("{head}…{tail}")
}

/// Format a PayRef as a compact string (same as short).
///
/// Alias for format_short() for different display contexts.
pub fn format_compact(payref: &[u8; PAY_REF_SIZE]) -> String {
    format_short(payref)
}

fn decode_hex_nibble(b: u8) -> Result<u8, CryptoError> {
    match b {
        b'0'..=b'9' => Ok(b - b'0'),
        b'a'..=b'f' => Ok(b - b'a' + 10),
        b'A'..=b'F' => Ok(b - b'A' + 10),
        _ => Err(CryptoError::InvalidParameters {
            param: "hex_character",
        }),
    }
}

fn to_lower_hex(bytes: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut out = vec![0u8; bytes.len() * 2];
    for (i, &b) in bytes.iter().enumerate() {
        out[2 * i] = HEX[(b >> 4) as usize];
        out[2 * i + 1] = HEX[(b & 0x0f) as usize];
    }
    // Best-effort: output is ASCII, but avoid panicking in production.
    match String::from_utf8(out) {
        Ok(s) => s,
        Err(err) => String::from_utf8_lossy(&err.into_bytes()).into_owned(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_is_deterministic() {
        let block = [1u8; 32];
        let out = [2u8; 32];
        assert_eq!(generate(&block, &out), generate(&block, &out));
    }

    #[test]
    fn test_verify_works_correctly_r735() {
        let block = [1u8; 32];
        let out = [2u8; 32];
        let payref = generate(&block, &out);

        // Correct inputs
        assert!(verify(&payref, &block, &out));

        // Wrong block hash
        assert!(!verify(&payref, &[3u8; 32], &out));

        // Wrong output hash
        assert!(!verify(&payref, &block, &[3u8; 32]));

        // Both wrong
        assert!(!verify(&payref, &[3u8; 32], &[3u8; 32]));
    }

    #[test]
    fn test_parse_hex_case_insensitive() {
        let bytes = [0xABu8; 32];
        let hex = "abababababababababababababababababababababababababababababababab";
        let parsed = parse_hex(hex).unwrap();
        assert_eq!(parsed, bytes);

        let parsed2 = parse_hex(&hex.to_uppercase()).unwrap();
        assert_eq!(parsed2, bytes);

        let parsed3 = parse_hex(&format!("0x{hex}")).unwrap();
        assert_eq!(parsed3, bytes);
    }

    #[test]
    fn test_format_short_expected_shape() {
        let bytes = [0x12u8; 32];
        let s = format_short(&bytes);
        assert!(s.contains('…'));
        assert_eq!(s.chars().count(), 8 + 1 + 8);
        assert_eq!(s, "12121212…12121212");
    }

    #[test]
    fn test_format_full_64_chars() {
        let bytes = [0xABu8; 32];
        let full = format_full(&bytes);
        assert_eq!(full.len(), 64);
        assert!(full.chars().all(|c| c.is_ascii_hexdigit()));
        assert!(full.chars().all(|c| c.is_lowercase()));
    }

    #[test]
    fn test_format_compact_equals_short() {
        let bytes = [0x42u8; 32];
        let short = format_short(&bytes);
        let compact = format_compact(&bytes);
        assert_eq!(short, compact);
    }

    #[test]
    fn test_different_inputs_different_payrefs() {
        let block1 = [1u8; 32];
        let out1 = [2u8; 32];
        let block2 = [3u8; 32];
        let out2 = [4u8; 32];

        let payref1 = generate(&block1, &out1);
        let payref2 = generate(&block2, &out2);
        let payref3 = generate(&block1, &out2);

        assert_ne!(payref1, payref2);
        assert_ne!(payref1, payref3);
        assert_ne!(payref2, payref3);
    }

    #[test]
    fn test_parse_hex_validates_length() {
        // Too short
        assert!(parse_hex("abc123").is_err());

        // Too long
        let long = "ab".repeat(33);
        assert!(parse_hex(&long).is_err());

        // Just right
        let correct = "ab".repeat(32);
        assert!(parse_hex(&correct).is_ok());
    }

    #[test]
    fn test_parse_hex_validates_characters() {
        // Valid hex
        let valid1 = "0123456789abcdef".repeat(4);
        assert!(parse_hex(&valid1).is_ok());
        let valid2 = "0123456789ABCDEF".repeat(4);
        assert!(parse_hex(&valid2).is_ok());

        // Invalid characters
        let invalid1 = "g".repeat(64);
        assert!(parse_hex(&invalid1).is_err());
        let invalid2 = "z".repeat(64);
        assert!(parse_hex(&invalid2).is_err());
        let invalid3 = "00".repeat(31) + "gg";
        assert!(parse_hex(&invalid3).is_err());
    }

    #[test]
    fn test_format_short_preserves_uniqueness() {
        // Two different payrefs should have different short formats
        let payref1 = generate(&[1u8; 32], &[2u8; 32]);
        let payref2 = generate(&[3u8; 32], &[4u8; 32]);

        let short1 = format_short(&payref1);
        let short2 = format_short(&payref2);

        assert_ne!(short1, short2);
    }

    #[test]
    fn test_roundtrip_generate_parse_hex() {
        let block = [0x42u8; 32];
        let out = [0x99u8; 32];

        let payref = generate(&block, &out);
        let hex = format_full(&payref);
        let parsed = parse_hex(&hex).unwrap();

        assert_eq!(payref, parsed);
    }

    #[test]
    fn test_roundtrip_generate_short_parse() {
        let block = [0x55u8; 32];
        let out = [0xAAu8; 32];

        let payref = generate(&block, &out);
        let short = format_short(&payref);

        // Can't directly parse short format, but verify it's derived correctly
        let full = format_full(&payref);
        let short_from_full = format_short(&parse_hex(&full).unwrap());

        assert_eq!(short, short_from_full);
    }

    #[test]
    fn test_constants() {
        assert_eq!(PAY_REF_SIZE, 32);
        assert_eq!(PAY_REF_DOMAIN, "z00z.wallets.tx.pay_ref.v1");
    }

    #[test]
    fn test_edge_case_like_inputs() {
        // All zeros
        let zero = [0u8; 32];
        let payref = generate(&zero, &zero);
        assert_eq!(payref.len(), 32);

        // Verify works
        assert!(verify(&payref, &zero, &zero));
    }

    #[test]
    fn test_edge_case_max_values() {
        // All 0xFF
        let max = [0xFFu8; 32];
        let payref = generate(&max, &max);
        assert_eq!(payref.len(), 32);

        // Verify works
        assert!(verify(&payref, &max, &max));
    }
}
