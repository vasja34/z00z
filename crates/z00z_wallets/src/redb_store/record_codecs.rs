use super::{
    decode_encrypted_object_record, decode_object_id_be, encode_object_id_be, mnemonic,
    BincodeCodec, Codec, Deserialize, EncryptedObjectRecord, FromStr, ReadableTable, RngCoreExt,
    SecretsKind, SecretsRecord, SeedMainEntropyPayload, SeedPhrase24, SeedWords, Serialize,
    WalletError, WalletResult,
};

pub(crate) fn encode_bincode<T: Serialize>(value: &T) -> WalletResult<Vec<u8>> {
    let codec = BincodeCodec;
    codec
        .serialize(value)
        .map_err(|e| WalletError::InvalidConfig(format!("bincode serialize failed: {e}")))
}

pub(crate) fn decode_bincode<T: for<'de> Deserialize<'de>>(bytes: &[u8]) -> WalletResult<T> {
    let codec = BincodeCodec;
    codec
        .deserialize(bytes)
        .map_err(|e| WalletError::InvalidConfig(format!("bincode deserialize failed: {e}")))
}

pub(crate) const WALLET_SECRET_INVALID: &str = "wallet secret record invalid";
#[cfg(feature = "wallet_debug_tools")]
pub(crate) const WALLET_MASTER_KEY_INVALID: &str = "wallet master key record invalid";
pub(crate) const WALLET_OBJECT_PAYLOAD_INVALID: &str = "wallet object payload invalid";

// Object payload size caps (DoS bounds).
// These caps apply to the bincode-encoded `EncryptedObjectPayload` container bytes.
pub(crate) const MAX_OBJECT_BYTES: usize = 4 * 1024 * 1024;
pub(crate) const MAX_COMPRESSED_BYTES: usize = 4 * 1024 * 1024;

// Versioned payload header (inside encryption).
// Layout (little-endian):
// magic[4] || version(u8) || flags(u8) || algo(u8) || reserved(u8)
// || uncompressed_len(u32) || stored_len(u32) || stored_bytes[stored_len]
pub(crate) const OBJECT_PAYLOAD_MAGIC: [u8; 4] = *b"ZWL1";
pub(crate) const OBJECT_PAYLOAD_HEADER_VERSION: u8 = 1;
pub(crate) const OBJECT_PAYLOAD_HEADER_LEN: usize = 16;

pub(crate) const OBJECT_PAYLOAD_FLAG_COMPRESSED: u8 = 0x01;

pub(crate) const OBJECT_PAYLOAD_ALGO_ZSTD: u8 = 1;
pub(crate) const OBJECT_PAYLOAD_ALGO_LZ4: u8 = 2;

pub(crate) fn wrap_object_payload_with_header(container_bytes: Vec<u8>) -> WalletResult<Vec<u8>> {
    if container_bytes.len() > MAX_OBJECT_BYTES {
        return Err(WalletError::InvalidParams(
            "object payload too large".to_string(),
        ));
    }

    let uncompressed_len: u32 = container_bytes
        .len()
        .try_into()
        .map_err(|_| WalletError::InvalidParams("object payload too large".to_string()))?;

    let algo = OBJECT_PAYLOAD_ALGO_ZSTD;
    let flags = OBJECT_PAYLOAD_FLAG_COMPRESSED;
    let mut stored_bytes = container_bytes;

    let compressed = z00z_utils::compression::zstd_compress(&stored_bytes)
        .map_err(|_| WalletError::InvalidConfig(WALLET_OBJECT_PAYLOAD_INVALID.to_string()))?;

    stored_bytes.fill(0);
    stored_bytes = compressed;

    if stored_bytes.len() > MAX_COMPRESSED_BYTES {
        return Err(WalletError::InvalidParams(
            "object payload too large".to_string(),
        ));
    }

    let stored_len: u32 = stored_bytes
        .len()
        .try_into()
        .map_err(|_| WalletError::InvalidParams("object payload too large".to_string()))?;

    let mut out = Vec::with_capacity(OBJECT_PAYLOAD_HEADER_LEN + stored_bytes.len());
    out.extend_from_slice(&OBJECT_PAYLOAD_MAGIC);
    out.push(OBJECT_PAYLOAD_HEADER_VERSION);
    out.push(flags);
    out.push(algo);
    out.push(0u8);
    out.extend_from_slice(&uncompressed_len.to_le_bytes());
    out.extend_from_slice(&stored_len.to_le_bytes());
    out.extend_from_slice(&stored_bytes);

    stored_bytes.fill(0);

    Ok(out)
}

pub(crate) fn unwrap_object_payload_with_header(
    payload_with_header: &[u8],
) -> WalletResult<Vec<u8>> {
    if payload_with_header.len() < OBJECT_PAYLOAD_HEADER_LEN
        || payload_with_header[..4] != OBJECT_PAYLOAD_MAGIC
    {
        return Err(WalletError::InvalidConfig(
            WALLET_OBJECT_PAYLOAD_INVALID.to_string(),
        ));
    }

    let version = payload_with_header[4];
    if version != OBJECT_PAYLOAD_HEADER_VERSION {
        return Err(WalletError::InvalidConfig(
            WALLET_OBJECT_PAYLOAD_INVALID.to_string(),
        ));
    }

    let flags = payload_with_header[5];
    let algo = payload_with_header[6];

    let mut uncompressed_len_bytes = [0u8; 4];
    uncompressed_len_bytes.copy_from_slice(&payload_with_header[8..12]);
    let uncompressed_len = u32::from_le_bytes(uncompressed_len_bytes) as usize;

    let mut stored_len_bytes = [0u8; 4];
    stored_len_bytes.copy_from_slice(&payload_with_header[12..16]);
    let stored_len = u32::from_le_bytes(stored_len_bytes) as usize;

    if uncompressed_len > MAX_OBJECT_BYTES || stored_len > MAX_COMPRESSED_BYTES {
        return Err(WalletError::InvalidConfig(
            WALLET_OBJECT_PAYLOAD_INVALID.to_string(),
        ));
    }

    if payload_with_header.len() != OBJECT_PAYLOAD_HEADER_LEN + stored_len {
        return Err(WalletError::InvalidConfig(
            WALLET_OBJECT_PAYLOAD_INVALID.to_string(),
        ));
    }

    let stored = &payload_with_header[OBJECT_PAYLOAD_HEADER_LEN..];

    if (flags & OBJECT_PAYLOAD_FLAG_COMPRESSED) == 0 {
        return Err(WalletError::InvalidConfig(
            WALLET_OBJECT_PAYLOAD_INVALID.to_string(),
        ));
    }

    match algo {
        OBJECT_PAYLOAD_ALGO_ZSTD => {
            let out = z00z_utils::compression::zstd_decompress_bounded(stored, uncompressed_len)
                .map_err(|_| {
                    WalletError::InvalidConfig(WALLET_OBJECT_PAYLOAD_INVALID.to_string())
                })?;

            if out.len() != uncompressed_len {
                return Err(WalletError::InvalidConfig(
                    WALLET_OBJECT_PAYLOAD_INVALID.to_string(),
                ));
            }

            Ok(out)
        }
        OBJECT_PAYLOAD_ALGO_LZ4 => {
            let out = z00z_utils::compression::lz4_decompress_bounded(stored, uncompressed_len)
                .map_err(|_| {
                    WalletError::InvalidConfig(WALLET_OBJECT_PAYLOAD_INVALID.to_string())
                })?;

            if out.len() != uncompressed_len {
                return Err(WalletError::InvalidConfig(
                    WALLET_OBJECT_PAYLOAD_INVALID.to_string(),
                ));
            }

            Ok(out)
        }
        _ => Err(WalletError::InvalidConfig(
            WALLET_OBJECT_PAYLOAD_INVALID.to_string(),
        )),
    }
}

pub(crate) fn decode_bincode_bounded<T: for<'de> Deserialize<'de>>(
    bytes: &[u8],
    err: WalletError,
) -> WalletResult<T> {
    let codec = BincodeCodec;
    codec.deserialize(bytes).map_err(|_| err)
}

pub(crate) fn decode_object_record_bounded(bytes: &[u8]) -> WalletResult<EncryptedObjectRecord> {
    decode_encrypted_object_record(bytes).map_err(|_| WalletError::InvalidPassword)
}

pub(crate) fn validate_seed_main_record(
    record: &SecretsRecord,
    err: WalletError,
) -> WalletResult<()> {
    if record.kind != SecretsKind::Seed {
        return Err(err);
    }

    if record.label != "main" {
        return Err(err);
    }

    if record.version != 1 {
        return Err(err);
    }

    Ok(())
}

pub(crate) fn validate_seed_plaintext_unlock(plaintext: &[u8]) -> WalletResult<()> {
    if let Ok(payload) = decode_bincode::<SeedMainEntropyPayload>(plaintext) {
        if payload.entropy_bytes.len() != 32 {
            return Err(WalletError::InvalidPassword);
        }

        SeedPhrase24::from_bip39_entropy_bytes(
            &payload.entropy_bytes,
            payload.mnemonic_language.to_mnemonic_language(),
        )
        .map_err(|_| WalletError::InvalidPassword)?;

        return Ok(());
    }

    let s = std::str::from_utf8(plaintext).map_err(|_| WalletError::InvalidPassword)?;
    let words = SeedWords::from_str(s).map_err(|_| WalletError::InvalidPassword)?;
    let matches = mnemonic::suggest_language(&words);
    let language = match matches.as_slice() {
        [only] => *only,
        _ => return Err(WalletError::InvalidPassword),
    };
    let phrase =
        SeedPhrase24::from_words(language, words).map_err(|_| WalletError::InvalidPassword)?;
    let mut entropy = phrase
        .to_bip39_entropy_bytes()
        .map_err(|_| WalletError::InvalidPassword)?;
    let ok = entropy.len() == 32;
    entropy.fill(0);
    if !ok {
        return Err(WalletError::InvalidPassword);
    }

    Ok(())
}

pub(crate) fn decode_seed_plaintext_phrase24(plaintext: &[u8]) -> WalletResult<SeedPhrase24> {
    if let Ok(payload) = decode_bincode::<SeedMainEntropyPayload>(plaintext) {
        if payload.entropy_bytes.len() != 32 {
            return Err(WalletError::InvalidPassword);
        }

        return SeedPhrase24::from_bip39_entropy_bytes(
            &payload.entropy_bytes,
            payload.mnemonic_language.to_mnemonic_language(),
        )
        .map_err(|_| WalletError::InvalidPassword);
    }

    let s = std::str::from_utf8(plaintext).map_err(|_| WalletError::InvalidPassword)?;
    let words = SeedWords::from_str(s).map_err(|_| WalletError::InvalidPassword)?;
    let matches = mnemonic::suggest_language(&words);
    let language = match matches.as_slice() {
        [only] => *only,
        _ => return Err(WalletError::InvalidPassword),
    };
    SeedPhrase24::from_words(language, words).map_err(|_| WalletError::InvalidPassword)
}

pub(crate) fn object_id_to_be_bytes(object_id: u128) -> [u8; 16] {
    encode_object_id_be(object_id)
}

pub(crate) fn object_id_from_be_bytes(bytes: &[u8]) -> WalletResult<u128> {
    decode_object_id_be(bytes)
}

pub(crate) fn generate_object_id(rng: &mut impl rand::RngCore) -> u128 {
    let mut buf = [0u8; 16];
    rng.fill_bytes_ext(&mut buf);
    u128::from_be_bytes(buf)
}

pub(crate) fn allocate_object_id(
    objects: &redb::Table<'_, &[u8], &[u8]>,
    rng: &mut impl rand::RngCore,
) -> WalletResult<u128> {
    for _ in 0..64 {
        let candidate = generate_object_id(rng);
        let key = encode_object_id_be(candidate);
        let exists = objects
            .get(key.as_slice())
            .map_err(|e| WalletError::InvalidConfig(format!("redb objects read failed: {e}")))?
            .is_some();
        if !exists {
            return Ok(candidate);
        }
    }

    Err(WalletError::InvalidConfig(
        "object id allocation failed".to_string(),
    ))
}

pub(crate) fn generate_16_bytes(rng: &mut impl rand::RngCore) -> [u8; 16] {
    let mut buf = [0u8; 16];
    rng.fill_bytes_ext(&mut buf);
    buf
}

#[cfg(feature = "test-params-fast")]
pub(crate) fn generate_24_bytes(rng: &mut impl rand::RngCore) -> [u8; 24] {
    let mut buf = [0u8; 24];
    rng.fill_bytes_ext(&mut buf);
    buf
}
