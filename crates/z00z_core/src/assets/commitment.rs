use z00z_crypto::{create_commitment, CryptoError, Z00ZCommitment, Z00ZScalar};

pub fn commit_amount(value: u64, blinding: &Z00ZScalar) -> Result<Z00ZCommitment, CryptoError> {
    create_commitment(value, blinding)
}

pub fn verify_commitment_opening(
    commitment: &Z00ZCommitment,
    amount: u64,
    blinding: &Z00ZScalar,
) -> Result<bool, CryptoError> {
    let expected = create_commitment(amount, blinding)?;
    Ok(expected.as_bytes() == commitment.as_bytes())
}
