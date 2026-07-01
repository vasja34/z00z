use tari_crypto::extended_range_proof::{ExtendedRangeProofService, Statement};
use tari_crypto::ristretto::bulletproofs_plus::RistrettoAggregatedPublicStatement;

use crate::{
    error::CryptoError,
    types::{
        AGGREGATION_FACTOR, MAX_BATCH_MEMORY, MAX_BATCH_PROOF_COUNT, MAX_PROOF_SIZE,
        RANGE_PROOF_BITS,
    },
    RangeProof, Z00ZCommitment,
};

use super::backend_init::bulletproof_service;

fn validate_batch_inputs(
    proofs: &[&RangeProof],
    commitments: &[&Z00ZCommitment],
    minimum_value_promises: &[u64],
) -> Result<(), CryptoError> {
    if proofs.len() != commitments.len() {
        return Err(CryptoError::InvalidParameters {
            param: "proof_commitment_mismatch",
        });
    }
    if proofs.len() != minimum_value_promises.len() {
        return Err(CryptoError::InvalidParameters {
            param: "minimum_value_promises_length_mismatch",
        });
    }
    if proofs.is_empty() {
        #[cfg(all(feature = "logging", not(target_arch = "wasm32")))]
        {
            use z00z_utils::logger::{Logger, StdoutLogger};

            const EMPTY_BATCH_MSG: &str =
                "batch_verify_range_proofs called with empty proof set - possible DoS attempt or caller bug";
            let logger = StdoutLogger;
            logger.warn(EMPTY_BATCH_MSG);
        }
        return Ok(());
    }
    if proofs.len() > MAX_BATCH_PROOF_COUNT {
        return Err(CryptoError::BatchTooLarge {
            count: proofs.len(),
            max: MAX_BATCH_PROOF_COUNT,
        });
    }
    Ok(())
}

fn validate_batch_params(bits: usize, aggregation_factor: usize) -> Result<(), CryptoError> {
    if !(1..=64).contains(&bits) {
        return Err(CryptoError::InvalidParameters { param: "bits" });
    }
    if aggregation_factor == 0 || aggregation_factor > 8 {
        return Err(CryptoError::InvalidParameters {
            param: "aggregation_factor",
        });
    }
    if bits != RANGE_PROOF_BITS {
        return Err(CryptoError::InvalidParameters { param: "bits" });
    }
    if aggregation_factor != AGGREGATION_FACTOR {
        return Err(CryptoError::InvalidParameters {
            param: "aggregation_factor",
        });
    }
    Ok(())
}

fn validate_proof_sizes(proofs: &[&RangeProof], max_size: usize) -> Result<(), CryptoError> {
    let mut all_valid = true;
    let mut first_error_idx = usize::MAX;
    let mut first_error_sz = 0usize;

    for (index, proof) in proofs.iter().enumerate() {
        let len = proof.len();
        let is_invalid = len == 0 || len > max_size;
        all_valid &= !is_invalid;

        let should_record = is_invalid && (first_error_idx == usize::MAX);
        let mask = (should_record as usize).wrapping_neg();

        #[allow(clippy::identity_op)]
        {
            first_error_idx = (mask & index) | (!mask & first_error_idx);
            first_error_sz = (mask & len) | (!mask & first_error_sz);
        }
    }

    if !all_valid {
        return Err(CryptoError::InvalidProofSize {
            index: first_error_idx,
            size: first_error_sz,
            max_size,
        });
    }
    Ok(())
}

fn validate_batch_memory(
    proofs: &[&RangeProof],
    commitments: &[&Z00ZCommitment],
) -> Result<(), CryptoError> {
    let estimated_memory = proofs.iter().map(|p| p.len()).sum::<usize>() + commitments.len() * 32;
    if estimated_memory > MAX_BATCH_MEMORY {
        return Err(CryptoError::ExcessiveMemoryUsage);
    }
    Ok(())
}

fn build_statements(
    commitments: &[&Z00ZCommitment],
    minimum_value_promises: &[u64],
) -> Vec<RistrettoAggregatedPublicStatement> {
    commitments
        .iter()
        .zip(minimum_value_promises.iter())
        .map(
            |(commitment, &minimum_value_promise)| RistrettoAggregatedPublicStatement {
                statements: vec![Statement {
                    commitment: commitment.reveal().clone(),
                    minimum_value_promise,
                }],
            },
        )
        .collect()
}

pub(crate) fn batch_verify_range_proofs_impl(
    proofs: &[&RangeProof],
    commitments: &[&Z00ZCommitment],
    bits: usize,
    aggregation_factor: usize,
    minimum_value_promises: &[u64],
) -> Result<(), CryptoError> {
    validate_batch_inputs(proofs, commitments, minimum_value_promises)?;
    if proofs.is_empty() {
        return Ok(());
    }
    validate_batch_params(bits, aggregation_factor)?;
    validate_proof_sizes(proofs, MAX_PROOF_SIZE)?;
    validate_batch_memory(proofs, commitments)?;

    let statements = build_statements(commitments, minimum_value_promises);
    let statement_refs: Vec<_> = statements.iter().collect();

    bulletproof_service()
        .verify_batch(proofs.to_vec(), statement_refs)
        .map_err(|_| CryptoError::ProofVerificationFailed)
}
