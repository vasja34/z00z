/// Encodes card as compact URL-safe base64.
pub fn encode_card_compact(card: &ReceiverCard) -> String {
    URL_SAFE_NO_PAD.encode(card.canonical_encoding())
}

/// Decodes card from compact URL-safe base64.
pub fn decode_card_compact(encoded: &str) -> Result<ReceiverCard, ReceiverCardError> {
    let raw = URL_SAFE_NO_PAD
        .decode(encoded)
        .map_err(|_| ReceiverCardError::InvalidCardBytes)?;
    ReceiverCard::from_untrusted_bytes(&raw)
}

/// Format owner handle as a human-readable receiver display string.
pub fn format_receiver_handle(owner_handle: &[u8; 32]) -> Result<String, ReceiverCardError> {
    bech32::encode("z00z", owner_handle.to_base32(), Variant::Bech32m)
        .map_err(|_| ReceiverCardError::InvalidCardString)
}

/// Prove ownership of a receiver card using identity key and challenge.
pub fn prove_ownership(
    card: &ReceiverCard,
    identity_sk: &Z00ZScalar,
    challenge: &[u8; 32],
) -> Result<[u8; 64], ReceiverCardError> {
    let expected_pk = Z00ZRistrettoPoint::from_secret_key(identity_sk);
    if expected_pk.as_bytes() != card.identity_pk {
        return Err(ReceiverCardError::KeyMismatch);
    }

    let sig = sign_identity(identity_sk, challenge, &card.owner_handle)
        .map_err(|_| ReceiverCardError::CryptoFailed)?;
    Ok(sig_to_bytes(&sig))
}

fn current_unix_timestamp_fail_closed() -> u64 {
    SystemTimeProvider.try_unix_timestamp().unwrap_or(u64::MAX)
}

fn parse_meta(bytes: &[u8], pos: &mut usize) -> Result<CardMetadata, ReceiverCardError> {
    let display_name = parse_opt_string(bytes, pos)?;

    let valid_until = match read_u8(bytes, pos)? {
        0 => None,
        1 => Some(read_u64(bytes, pos)?),
        _ => return Err(ReceiverCardError::InvalidCardFlag),
    };

    let contact = parse_opt_string(bytes, pos)?;
    let created_at = read_u64(bytes, pos)?;

    Ok(CardMetadata {
        created_at,
        display_name,
        valid_until,
        contact,
    })
}

fn parse_card_id(bytes: &[u8], pos: &mut usize) -> Result<Option<[u8; 16]>, ReceiverCardError> {
    let flag = read_u8(bytes, pos)?;
    if flag == 0 {
        return Ok(None);
    }
    if flag != 1 {
        return Err(ReceiverCardError::InvalidCardFlag);
    }

    Ok(Some(read_arr::<16>(bytes, pos)?))
}

fn parse_card_meta(
    bytes: &[u8],
    pos: &mut usize,
) -> Result<Option<CardMetadata>, ReceiverCardError> {
    let flag = read_u8(bytes, pos)?;
    if flag == 0 {
        return Ok(None);
    }
    if flag != 1 {
        return Err(ReceiverCardError::InvalidCardFlag);
    }

    Ok(Some(parse_meta(bytes, pos)?))
}

fn parse_opt_string(bytes: &[u8], pos: &mut usize) -> Result<Option<String>, ReceiverCardError> {
    let flag = read_u8(bytes, pos)?;
    if flag == 0 {
        return Ok(None);
    }
    if flag != 1 {
        return Err(ReceiverCardError::InvalidCardFlag);
    }

    let len = read_u32(bytes, pos)?;
    if len > STR_LEN_MAX {
        return Err(ReceiverCardError::InvalidCardSize);
    }
    let len_usize = usize::try_from(len).map_err(|_| ReceiverCardError::InvalidCardSize)?;
    let end = pos
        .checked_add(len_usize)
        .ok_or(ReceiverCardError::InvalidCardSize)?;
    if end > bytes.len() {
        return Err(ReceiverCardError::InvalidCardBytes);
    }
    let value =
        std::str::from_utf8(&bytes[*pos..end]).map_err(|_| ReceiverCardError::InvalidCardString)?;
    *pos = end;
    Ok(Some(value.to_string()))
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

fn read_u8(bytes: &[u8], pos: &mut usize) -> Result<u8, ReceiverCardError> {
    if *pos >= bytes.len() {
        return Err(ReceiverCardError::InvalidCardBytes);
    }
    let value = bytes[*pos];
    *pos += 1;
    Ok(value)
}

fn read_u32(bytes: &[u8], pos: &mut usize) -> Result<u32, ReceiverCardError> {
    let value = read_arr::<4>(bytes, pos)?;
    Ok(u32::from_le_bytes(value))
}

fn read_u64(bytes: &[u8], pos: &mut usize) -> Result<u64, ReceiverCardError> {
    let value = read_arr::<8>(bytes, pos)?;
    Ok(u64::from_le_bytes(value))
}

fn read_arr<const N: usize>(bytes: &[u8], pos: &mut usize) -> Result<[u8; N], ReceiverCardError> {
    let end = pos
        .checked_add(N)
        .ok_or(ReceiverCardError::InvalidCardSize)?;
    if end > bytes.len() {
        return Err(ReceiverCardError::InvalidCardBytes);
    }
    let part = bytes[*pos..end]
        .try_into()
        .map_err(|_| ReceiverCardError::InvalidCardBytes)?;
    *pos = end;
    Ok(part)
}

fn decode_card_public_key(bytes: &[u8; 32]) -> Result<Z00ZRistrettoPoint, ReceiverCardError> {
    Z00ZRistrettoPoint::try_from_bytes(*bytes).map_err(|error| match error {
        z00z_crypto::CryptoError::IdentityPoint => ReceiverCardError::IdentityPoint,
        _ => ReceiverCardError::InvalidPublicKey,
    })
}

fn sig_to_bytes(sig: &Z00ZSchnorrSignature) -> [u8; 64] {
    let mut bytes = [0u8; 64];
    bytes[..32].copy_from_slice(sig.get_public_nonce().as_bytes());
    bytes[32..].copy_from_slice(sig.get_signature().as_bytes());
    bytes
}

fn sig_from_bytes(bytes: &[u8; 64]) -> Result<Z00ZSchnorrSignature, ReceiverCardError> {
    let nonce = Z00ZRistrettoPoint::try_from_bytes(
        bytes[..32]
            .try_into()
            .map_err(|_| ReceiverCardError::InvalidSignature)?,
    )
    .map_err(|_| ReceiverCardError::InvalidSignature)?;
    let s = Z00ZScalar::try_from_bytes(
        bytes[32..]
            .try_into()
            .map_err(|_| ReceiverCardError::InvalidSignature)?,
    )
        .map_err(|_| ReceiverCardError::InvalidSignature)?;
    validate_scalar_nonzero(&s).map_err(|_| ReceiverCardError::InvalidSignature)?;
    Ok(Z00ZSchnorrSignature::new(
        nonce.reveal().clone(),
        s.reveal().clone(),
    ))
}