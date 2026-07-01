use tari_crypto::tari_utilities::ByteArray;

use crate::CryptoError;

use super::{Z00ZCommitment, MAX_PROOF_SIZE, MAX_PROOF_SIZE_EXTENDED};

pub fn validate_amount(amount: u64) -> Result<(), CryptoError> {
    validate_transfer_amount(amount)
}

pub fn validate_transfer_amount(amount: u64) -> Result<(), CryptoError> {
    if amount == 0 {
        return Err(CryptoError::InvalidParameters { param: "amount" });
    }
    Ok(())
}

pub fn validate_asset_amount(amount: u64, allow_zero: bool) -> Result<(), CryptoError> {
    if amount == 0 && !allow_zero {
        return Err(CryptoError::InvalidParameters { param: "amount" });
    }
    Ok(())
}

pub fn validate_amount_relaxed(_amount: u64) -> Result<(), CryptoError> {
    Ok(())
}

pub fn validate_proof_size(proof_size: usize, version: u32) -> Result<(), CryptoError> {
    let max_size = match version {
        1 => MAX_PROOF_SIZE,
        2 => MAX_PROOF_SIZE_EXTENDED,
        _ => {
            return Err(CryptoError::InvalidParameters {
                param: "protocol version",
            });
        }
    };

    if proof_size > max_size {
        return Err(CryptoError::InvalidParameters {
            param: "proof size (exceeds limit)",
        });
    }

    Ok(())
}

pub fn validate_commitment_non_zero(commitment: &Z00ZCommitment) -> Result<(), CryptoError> {
    let bytes = commitment.reveal().as_bytes();
    if bytes.iter().all(|&byte| byte == 0) {
        return Err(CryptoError::InvalidParameters {
            param: "commitment (zero)",
        });
    }
    Ok(())
}
