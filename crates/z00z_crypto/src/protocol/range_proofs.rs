//! Range proof wrappers for Phase 5.

use thiserror::Error;

use crate::{
    batch_verify_range_proofs, create_range_proof, verify_range_proof, CryptoError, RangeProof,
    Z00ZScalar,
};

use crate::protocol::commitments::{generate_blinding_factor, Commitment, CommitmentErr};

const DEFAULT_BITS: usize = 64;
const DEFAULT_AGG: usize = 1;
const DEFAULT_MIN: u64 = 0;
#[cfg(test)]
const TYPICAL_MAX_SIZE: usize = 1024;

#[derive(Debug, Error)]
pub enum RangeProofErr {
    #[error("range proof generation failed")]
    GenFail,
    #[error("range proof verification failed")]
    VerifyFail,
    #[error("batch verification failed")]
    BatchFail,
}

impl From<CryptoError> for RangeProofErr {
    fn from(_: CryptoError) -> Self {
        RangeProofErr::VerifyFail
    }
}

impl From<CommitmentErr> for RangeProofErr {
    fn from(_: CommitmentErr) -> Self {
        RangeProofErr::GenFail
    }
}

/// Real Bulletproofs+ verification seam for the current accepted boundary; not a finished end-to-end trustless theorem.
#[derive(Debug, Clone)]
pub struct AssetRangeProof {
    proof_bytes: RangeProof,
}

impl AssetRangeProof {
    pub fn new(value: u64, blinding: &Z00ZScalar) -> Result<Self, RangeProofErr> {
        let proof = create_range_proof(value, blinding, DEFAULT_BITS, DEFAULT_MIN)
            .map_err(|_| RangeProofErr::GenFail)?;
        Ok(Self { proof_bytes: proof })
    }

    pub fn size_bytes(&self) -> usize {
        self.proof_bytes.len()
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        self.proof_bytes.clone()
    }

    pub fn verify(&self, commitment: &Commitment) -> Result<bool, RangeProofErr> {
        verify_range_proof(
            &self.proof_bytes,
            commitment.as_point(),
            DEFAULT_BITS,
            DEFAULT_AGG,
            DEFAULT_MIN,
        )
        .map(|_| true)
        .map_err(|_| RangeProofErr::VerifyFail)
    }
}

/// Asset output with commitment and proof.
#[derive(Debug, Clone)]
pub struct AssetOutputProof {
    pub commitment: Commitment,
    pub proof: AssetRangeProof,
}

impl AssetOutputProof {
    pub fn new(value: u64) -> Result<Self, RangeProofErr> {
        let blinding = generate_blinding_factor()?;
        let commitment = Commitment::new_with_blinding(value, &blinding)?;
        let proof = AssetRangeProof::new(value, &blinding)?;
        Ok(Self { commitment, proof })
    }

    pub fn verify(&self) -> Result<bool, RangeProofErr> {
        self.proof.verify(&self.commitment)
    }
}

pub fn verify_asset_output_proofs_batch(
    outputs: &[AssetOutputProof],
) -> Result<bool, RangeProofErr> {
    let proofs: Vec<&RangeProof> = outputs.iter().map(|item| &item.proof.proof_bytes).collect();
    let commitments: Vec<&crate::Z00ZCommitment> = outputs
        .iter()
        .map(|item| item.commitment.as_point())
        .collect();
    let mins = vec![DEFAULT_MIN; outputs.len()];

    batch_verify_range_proofs(&proofs, &commitments, DEFAULT_BITS, DEFAULT_AGG, &mins)
        .map(|_| true)
        .map_err(|_| RangeProofErr::BatchFail)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_proof_valid_value() {
        let value = 1000u64;
        let blind = Z00ZScalar::one();
        let proof = AssetRangeProof::new(value, &blind).expect("proof");
        assert!(proof.size_bytes() > 0);
    }

    #[test]
    fn test_proof_zero_value() {
        let value = 0u64;
        let blind = Z00ZScalar::one();
        let proof = AssetRangeProof::new(value, &blind).expect("proof");
        let commitment = Commitment::new_with_blinding(value, &blind).expect("commitment");
        assert!(proof.verify(&commitment).expect("verify"));
    }

    #[test]
    fn test_proof_max_value() {
        let value = u64::MAX;
        let blind = Z00ZScalar::one();
        let result = AssetRangeProof::new(value, &blind);
        assert!(result.is_ok());
    }

    #[test]
    fn test_proof_repeat_valid() {
        let value = 1000u64;
        let blind = Z00ZScalar::one();
        let p1 = AssetRangeProof::new(value, &blind).expect("p1");
        let p2 = AssetRangeProof::new(value, &blind).expect("p2");
        let commitment = Commitment::new_with_blinding(value, &blind).expect("commitment");
        assert!(p1.verify(&commitment).expect("verify p1"));
        assert!(p2.verify(&commitment).expect("verify p2"));
    }

    #[test]
    fn test_proof_reject_tamper() {
        let value = 1000u64;
        let blind = Z00ZScalar::one();
        let proof = AssetRangeProof::new(value, &blind).expect("proof");
        let commitment = Commitment::new_with_blinding(value, &blind).expect("commitment");

        let mut tampered = proof.to_bytes();
        tampered[0] ^= 0x01;
        let bad = AssetRangeProof {
            proof_bytes: tampered,
        };

        assert!(bad.verify(&commitment).is_err());
    }

    #[test]
    fn test_proof_size_guard() {
        let value = 1000u64;
        let blind = Z00ZScalar::one();
        let proof = AssetRangeProof::new(value, &blind).expect("proof");
        assert!(proof.size_bytes() < TYPICAL_MAX_SIZE);
    }

    #[test]
    fn test_batch_verify_ok() {
        let mut outputs = Vec::new();
        for value in [10u64, 20, 30, 40] {
            outputs.push(AssetOutputProof::new(value).expect("output"));
        }
        assert!(verify_asset_output_proofs_batch(&outputs).expect("batch"));
    }
}
