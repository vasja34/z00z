fn decode_pk(bytes: &[u8; 32]) -> Result<Z00ZRistrettoPoint, PaymentRequestError> {
    Z00ZRistrettoPoint::try_from_bytes(*bytes).map_err(|error| match error {
        z00z_crypto::CryptoError::IdentityPoint => PaymentRequestError::IdentityPoint,
        _ => PaymentRequestError::InvalidPublicKey,
    })
}

fn sig_to_bytes(sig: &Z00ZSchnorrSignature) -> [u8; 64] {
    let mut bytes = [0u8; 64];
    bytes[..32].copy_from_slice(sig.get_public_nonce().as_bytes());
    bytes[32..].copy_from_slice(sig.get_signature().as_bytes());
    bytes
}

fn sig_from_bytes(bytes: &[u8; 64]) -> Result<Z00ZSchnorrSignature, PaymentRequestError> {
    let nonce = Z00ZRistrettoPoint::try_from_bytes(
        bytes[..32]
            .try_into()
            .map_err(|_| PaymentRequestError::InvalidSignature)?,
    )
    .map_err(|_| PaymentRequestError::InvalidSignature)?;
    let s = Z00ZScalar::try_from_bytes(
        bytes[32..]
            .try_into()
            .map_err(|_| PaymentRequestError::InvalidSignature)?,
    )
        .map_err(|_| PaymentRequestError::InvalidSignature)?;
    validate_scalar_nonzero(&s).map_err(|_| PaymentRequestError::InvalidSignature)?;
    Ok(Z00ZSchnorrSignature::new(
        nonce.reveal().clone(),
        s.reveal().clone(),
    ))
}

fn encode_opt_string(out: &mut Vec<u8>, value: &Option<String>) {
    match value {
        Some(text) => {
            out.push(1);
            let bytes = text.as_bytes();
            let len = u32::try_from(bytes.len()).unwrap_or(u32::MAX);
            out.extend_from_slice(&len.to_le_bytes());
            out.extend_from_slice(bytes);
        }
        None => out.push(0),
    }
}
