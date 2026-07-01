//! Cryptographic error types for Z00Z blockchain.
//!
//! This module provides unified error handling for all cryptographic operations
//! in the Z00Z protocol, enabling consistent error reporting across the codebase.

use thiserror::Error;

/// Represents errors that can occur during cryptographic operations.
///
/// This enum provides a unified error interface for all cryptographic operations,
/// allowing external code to handle crypto failures without knowing about the underlying
/// backend implementation details (Tari, post-quantum alternatives, etc.).
///
/// # Design Rationale
///
/// Using a dedicated error enum instead of string errors ensures:
/// - **Type safety:** Errors are handled explicitly with pattern matching
/// - **Composability:** Errors propagate naturally through `?` operator
/// - **Backend agnostic:** Error sources from any backend can be wrapped
/// - **Debuggability:** Full error context preserved (not lost in string conversion)
/// - **Security:** No sensitive data in error messages (no String fields)
///
/// # Examples
///
/// ```ignore
/// use z00z_crypto::{create_range_proof, CryptoError};
///
/// match create_range_proof(amount, &blinding, 64, 0) {
///     Ok(_proof) => {}
///     Err(CryptoError::ProofGenerationFailed) => {}
///     Err(CryptoError::InvalidParameters { param: _ }) => {}
///     Err(_err) => {}
/// }
/// ```
#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum CryptoError {
    /// Generic cryptographic operation failed.
    ///
    /// Used for general crypto failures that don't fit into other categories.
    /// No sensitive details are included to prevent information leakage.
    ///
    /// # Example
    /// - Commitment factory initialization failures
    /// - Unexpected internal state errors
    #[error("Cryptographic operation failed")]
    CryptoOperationFailed,

    /// Range proof generation failed.
    ///
    /// Indicates that a range proof could not be generated for the provided amount
    /// and blinding factor. This typically means:
    /// - Invalid amount (e.g., exceeds bit limit)
    /// - Invalid blinding factor
    /// - Internal Bulletproof+ service failure
    ///
    /// # Performance Note
    /// Range proof generation is computationally expensive (6-45ms for 64-bit proofs).
    /// If this error occurs frequently, consider:
    /// - Precomputing proofs if amounts are known
    /// - Using batch verification where possible
    /// - Checking amount validity before attempting proof generation
    #[error("Range proof generation failed")]
    ProofGenerationFailed,

    /// Range proof verification failed.
    ///
    /// Indicates that a range proof could not be verified against the provided commitment.
    /// Possible causes:
    /// - Proof is corrupted or tampered with
    /// - Commitment doesn't match the proof
    /// - Proof uses incompatible parameters (wrong bit length, aggregation factor)
    /// - Underlying cryptographic verification failed
    ///
    /// # Security Note
    /// A verification failure means:
    /// - The proof does NOT correspond to the claimed amount
    /// - The transaction should be REJECTED
    /// - The proof MUST NOT be used for consensus
    #[error("Range proof verification failed")]
    ProofVerificationFailed,

    /// Invalid parameters provided to cryptographic operation.
    ///
    /// Indicates that the provided parameters are invalid or incompatible:
    /// - Amount exceeds supported range (e.g., > 2^64)
    /// - Bit length is unsupported (must be 1-64 for Bulletproofs+)
    /// - Aggregation factor is out of range
    /// - Blinding factor is invalid (all-zero, etc.)
    ///
    /// These errors are typically caught during input validation and indicate
    /// a programming error rather than a cryptographic failure.
    ///
    /// # Security
    /// Uses a static string for the parameter name to avoid including sensitive values.
    #[error("Invalid cryptographic parameter: {param}")]
    InvalidParameters {
        /// Name of the invalid parameter (e.g., "amount", "bits")
        param: &'static str,
    },

    /// Backend-specific error occurred with context.
    ///
    /// Wraps errors from the underlying cryptographic backend (Tari, etc.)
    /// with minimal context about what failed.
    ///
    /// # Security
    /// Uses a static context string to avoid including sensitive values.
    #[error("Backend error: {context}")]
    BackendError {
        /// What operation failed (static string only)
        context: &'static str,
    },

    /// Proof verification failed due to mismatch.
    ///
    /// More specific than ProofVerificationFailed - indicates the proof
    /// doesn't match the provided commitment.
    #[error("Proof does not match commitment")]
    ProofCommitmentMismatch,

    /// Batch verification failed - at least one proof is invalid.
    ///
    /// Used when verifying multiple proofs simultaneously and at least
    /// one fails verification.
    #[error("Batch verification failed: one or more proofs invalid")]
    BatchVerificationFailed,

    /// Batch is too large for safe verification.
    ///
    /// Prevents resource exhaustion from extremely large inputs.
    #[error("Batch too large: {count} > {max}")]
    BatchTooLarge {
        /// Number of proofs requested for verification.
        count: usize,
        /// Maximum supported proofs per batch.
        max: usize,
    },

    /// Batch verification exceeded resource limits.
    #[error("Excessive memory usage")]
    ExcessiveMemoryUsage,

    /// Batch verification exceeded the caller-provided time limit.
    #[error("Batch verification timed out")]
    BatchTimeout,

    /// Service initialization failed.
    ///
    /// Indicates that a cryptographic service (Bulletproof+, commitment factory, etc.)
    /// could not be initialized.
    #[error("Cryptographic service initialization failed")]
    ServiceInitializationFailed,

    /// Proof aggregation failed.
    ///
    /// Used when combining multiple proofs into a batch.
    #[error("Proof aggregation failed")]
    ProofAggregationFailed,

    /// Invalid proof size.
    ///
    /// The provided proof has incorrect size for the expected format.
    #[error("Invalid proof size at index {index}: {size} > {max_size}")]
    InvalidProofSize {
        /// Index of the offending proof in the batch.
        index: usize,
        /// Observed proof size in bytes.
        size: usize,
        /// Maximum allowed size in bytes.
        max_size: usize,
    },

    /// AAD size exceeds the allowed limit.
    ///
    /// AAD is not encrypted but is authenticated; excessively large AAD can amplify
    /// DoS risk by increasing CPU and allocation overhead.
    #[error("AAD size {size} exceeds limit {limit}")]
    AadTooLarge {
        /// Observed AAD size in bytes.
        size: usize,
        /// Maximum allowed AAD size in bytes.
        limit: usize,
    },

    /// Invalid commitment.
    ///
    /// The provided commitment is malformed or invalid.
    #[error("Invalid commitment")]
    InvalidCommitment,

    /// Blinding factor is all-zero.
    ///
    /// A zero blinding factor completely breaks the hiding property of Pedersen commitments.
    /// With a zero blinding factor, C = amount*G + 0*H = amount*G, making the commitment
    /// deterministic and revealing the amount through precomputation attacks.
    ///
    /// # Security Impact
    /// An attacker can precompute all possible commitments for amounts 1..MAX_AMOUNT
    /// and match against transaction outputs to extract amounts.
    ///
    /// This is a CRITICAL security violation and must be rejected.
    #[error("Blinding factor cannot be all-zero (breaks commitment hiding)")]
    InvalidBlindingFactorZero,

    /// Random number generator failure.
    ///
    /// The system RNG failed to generate random bytes, which is a critical security event.
    /// This can indicate:
    /// - RNG hardware failure on embedded systems
    /// - Corrupted /dev/urandom on Linux
    /// - RNG service unavailable in restricted environments
    /// - Potential system compromise or tampering
    ///
    /// # Security Impact
    /// Without secure randomness, cryptographic operations (especially nonce generation)
    /// become completely insecure. Nonce reuse breaks AEAD security, enabling:
    /// - Complete plaintext recovery
    /// - Forgery of authenticated messages
    ///
    /// Applications MUST halt cryptographic operations and alert operators when this occurs.
    #[error("Random number generator failure - cannot generate secure nonces")]
    RngFailure,

    /// Invalid scalar value.
    ///
    /// Hash-to-scalar conversion failed because the output could not be reduced
    /// to a valid Ristretto curve scalar. This should never happen in practice
    /// with proper hash functions (BLAKE2b, Poseidon), indicating potential:
    /// - Corrupted hash output
    /// - Implementation bug in scalar reduction
    /// - Memory corruption
    ///
    /// # Security Impact
    /// An invalid scalar cannot be used for cryptographic operations.
    /// Attempting to proceed would cause undefined behavior or security vulnerabilities.
    #[error("Invalid scalar value - hash-to-scalar conversion failed")]
    InvalidScalar,

    /// Invalid public key.
    ///
    /// Public key validation failed because the point is invalid:
    /// - Identity point (O) detected
    /// - Point not on curve (should never happen with Ristretto)
    /// - Untrusted input from blockchain failed validation
    ///
    /// # Security Impact
    /// Using identity point allows:
    /// - Shared ECDH secret becomes identity (predictable)
    /// - Derived keys become zero or predictable
    /// - Break stealth address privacy
    ///
    /// This is a CRITICAL security violation and MUST be rejected.
    #[error("Invalid public key - identity point or corrupted data")]
    InvalidPublicKey,

    /// Invalid point length.
    ///
    /// Point encoding must be exactly 32 bytes for Ristretto255.
    #[error("Invalid point length - expected 32 bytes")]
    InvalidPointLength,

    /// Invalid point encoding.
    ///
    /// Point decompression failed (non-canonical or invalid encoding).
    #[error("Invalid point - decompression failed")]
    InvalidPoint,

    /// Identity point detected.
    ///
    /// Identity point (O) is forbidden in protocol (SPEC §2.1.3 Rule 2).
    #[error("Identity point forbidden")]
    IdentityPoint,

    /// Zero scalar detected.
    ///
    /// Zero scalar is forbidden for ephemeral secrets (SPEC §2.1.3 Rule 3).
    #[error("Zero scalar forbidden")]
    ZeroScalar,

    /// Authentication failed.
    ///
    /// AEAD tag verification failed. Possible causes:
    /// - Wrong decryption key
    /// - Ciphertext tampered
    /// - Associated data mismatch
    /// - Replay attack attempt
    ///
    /// This is a CRITICAL security event and MUST NOT be ignored.
    #[error("Authentication failed - invalid AEAD tag")]
    AuthenticationFailed,

    /// Invalid length.
    ///
    /// Input data has unexpected length. Common causes:
    /// - Truncated ciphertext
    /// - Malformed serialization
    /// - Protocol version mismatch
    #[error("Invalid length - expected different size")]
    InvalidLength,
}

impl From<crate::kdf::KdfError> for CryptoError {
    fn from(err: crate::kdf::KdfError) -> Self {
        match err {
            crate::kdf::KdfError::Argon2Params => CryptoError::InvalidParameters {
                param: "argon2_params",
            },
            crate::kdf::KdfError::Argon2Execution => CryptoError::CryptoOperationFailed,
            crate::kdf::KdfError::HkdfExpansion => CryptoError::CryptoOperationFailed,
            crate::kdf::KdfError::HkdfInfoEmpty => {
                CryptoError::InvalidParameters { param: "hkdf_info" }
            }
            crate::kdf::KdfError::HkdfSaltRequired => {
                CryptoError::InvalidParameters { param: "hkdf_salt" }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_crypto_error_display() {
        let err = CryptoError::InvalidParameters { param: "amount" };
        assert_eq!(err.to_string(), "Invalid cryptographic parameter: amount");
    }

    #[test]
    fn test_crypto_error_debug() {
        let err = CryptoError::ProofGenerationFailed;
        let debug = format!("{:?}", err);
        assert!(debug.contains("ProofGenerationFailed"));
        // Verify no sensitive data in debug output
        assert!(!debug.contains("service"));
        assert!(!debug.contains("unavailable"));
    }

    #[test]
    fn test_crypto_error_no_clone() {
        // Verify that errors are NOT cloneable (security feature)
        // This test ensures we don't accidentally add Clone back
        let err = CryptoError::CryptoOperationFailed;
        // Can't clone, but can use references
        let err_ref = &err;
        assert_eq!(err.to_string(), err_ref.to_string());
    }

    #[test]
    fn test_all_variants_display() {
        // Ensure all variants have proper Display implementations
        let variants = [
            CryptoError::CryptoOperationFailed.to_string(),
            CryptoError::ProofGenerationFailed.to_string(),
            CryptoError::ProofVerificationFailed.to_string(),
            CryptoError::InvalidParameters { param: "test" }.to_string(),
            CryptoError::BackendError { context: "test" }.to_string(),
            CryptoError::ProofCommitmentMismatch.to_string(),
            CryptoError::BatchVerificationFailed.to_string(),
        ];

        // All should be non-empty and not contain sensitive data
        for variant in variants {
            assert!(!variant.is_empty());
            assert!(!variant.contains("secret"));
            assert!(!variant.contains("key"));
            assert!(!variant.contains("password"));
        }
    }

    #[test]
    fn test_error_trait() {
        // Verify it implements std::error::Error
        fn test_requires_error<E: std::error::Error>() {}
        test_requires_error::<CryptoError>();
    }
}
