use tari_crypto::{
    commitment::ExtensionDegree,
    extended_range_proof::{ExtendedMask, ExtendedRangeProofService, ExtendedWitness, Statement},
    ristretto::bulletproofs_plus::RistrettoAggregatedPublicStatement,
};

use crate::{
    error::CryptoError,
    types::{AGGREGATION_FACTOR, MAX_PROOF_SIZE, RANGE_PROOF_BITS},
    RangeProof, Z00ZCommitment, Z00ZScalar,
};

use super::backend_init::{bulletproof_service, COMMITMENT_FACTORY};

pub(crate) fn create_range_proof_impl(
    amount: u64,
    blinding: &Z00ZScalar,
    bits: usize,
    minimum_value_promise: u64,
) -> Result<RangeProof, CryptoError> {
    if !(1..=64).contains(&bits) {
        return Err(CryptoError::InvalidParameters { param: "bits" });
    }
    if bits != RANGE_PROOF_BITS {
        return Err(CryptoError::InvalidParameters { param: "bits" });
    }
    if amount < minimum_value_promise {
        return Err(CryptoError::InvalidParameters {
            param: "minimum_value_promise",
        });
    }

    let mask = ExtendedMask::assign(
        ExtensionDegree::DefaultPedersen,
        vec![blinding.inner().clone()],
    )
    .map_err(|_| CryptoError::ProofGenerationFailed)?;
    let witness = ExtendedWitness::new(mask, amount, minimum_value_promise);
    let proof = bulletproof_service()
        .construct_extended_proof(vec![witness], None)
        .map_err(|_| CryptoError::ProofGenerationFailed)?;

    let _ = &*COMMITMENT_FACTORY;
    Ok(proof)
}

pub(crate) fn verify_range_proof_impl(
    proof: &RangeProof,
    commitment: &Z00ZCommitment,
    bits: usize,
    aggregation_factor: usize,
    minimum_value_promise: u64,
) -> Result<(), CryptoError> {
    if !(1..=64).contains(&bits) {
        return Err(CryptoError::InvalidParameters { param: "bits" });
    }
    if aggregation_factor == 0 || aggregation_factor > 8 {
        return Err(CryptoError::InvalidParameters {
            param: "aggregation_factor",
        });
    }

    if proof.len() > MAX_PROOF_SIZE {
        return Err(CryptoError::ProofVerificationFailed);
    }
    if bits != RANGE_PROOF_BITS {
        return Err(CryptoError::InvalidParameters { param: "bits" });
    }
    if aggregation_factor != AGGREGATION_FACTOR {
        return Err(CryptoError::InvalidParameters {
            param: "aggregation_factor",
        });
    }

    let statement = RistrettoAggregatedPublicStatement {
        statements: vec![Statement {
            commitment: commitment.reveal().clone(),
            minimum_value_promise,
        }],
    };

    if bulletproof_service()
        .verify_batch(vec![proof], vec![&statement])
        .is_ok()
    {
        Ok(())
    } else {
        Err(CryptoError::ProofVerificationFailed)
    }
}
