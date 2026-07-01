use super::BackendInfo;
use crate::error::CryptoError;
use crate::types::{Z00ZCommitment, Z00ZScalar};
use crate::RangeProof;

/// Universal cryptographic backend trait.
///
/// This trait abstracts cryptographic operations from their implementation,
/// allowing Z00Z to work with different crypto backends transparently.
pub(crate) trait CryptoBackend: Send + Sync + 'static {
    /// Create a Pedersen commitment for an amount with a given blinding factor.
    fn create_commitment(&self, amount: u64, blinding: &Z00ZScalar) -> Z00ZCommitment;

    /// Generate a range proof for an amount with given blinding factor.
    fn create_range_proof(
        &self,
        amount: u64,
        blinding: &Z00ZScalar,
        bits: usize,
        minimum_value_promise: u64,
    ) -> Result<RangeProof, CryptoError>;

    /// Verify a range proof against a commitment.
    fn verify_range_proof(
        &self,
        proof: &RangeProof,
        commitment: &Z00ZCommitment,
        bits: usize,
        aggregation_factor: usize,
        minimum_value_promise: u64,
    ) -> Result<(), CryptoError>;

    /// Batch verify multiple range proofs simultaneously.
    fn batch_verify_range_proofs(
        &self,
        proofs: &[&RangeProof],
        commitments: &[&Z00ZCommitment],
        bits: usize,
        aggregation_factor: usize,
        minimum_value_promises: &[u64],
    ) -> Result<(), CryptoError>;

    /// Get information about this backend implementation.
    fn backend_info(&self) -> BackendInfo;

    /// Derive a hash from domain and data using backend-specific hasher.
    fn derive_hash(&self, domain: &[u8], data: &[&[u8]]) -> [u8; 32];
}
