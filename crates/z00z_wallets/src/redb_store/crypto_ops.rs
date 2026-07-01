use super::{
    aad_object, decode_bincode, decode_bincode_bounded, encode_bincode,
    is_supported_payload_version, unwrap_object_payload_with_header, validate_seed_main_record,
    wrap_object_payload_with_header, AeadEnvelope, BincodeCodec, Codec, EncryptedObjectPayload,
    EncryptedObjectRecord, MasterKeyRecord, PersistWalletId, ReadableDatabase, ReadableTable,
    SafePassword, SecretBytes, SecretsKind, SecretsRecord, SecureRngProvider,
    SeedMainEntropyPayload, SeedPhrase24, TimeProvider, WalletDerivedKeys, WalletError,
    WalletRedbKeyManager, WalletResult, WalletSession, MAX_COMPRESSED_BYTES, META_WALLET_INTEGRITY,
    OBJECT_PAYLOAD_HEADER_LEN, SECRETS_MASTER_KEY, SECRETS_SEED_MAIN,
    SECRETS_SEED_MAIN_REVEALED_AT, SECRETS_TABLE, WALLET_OBJECT_PAYLOAD_INVALID,
    WALLET_SECRET_INVALID,
};
#[cfg(feature = "test-params-fast")]
use crate::db::redb_store::codecs::generate_24_bytes;

#[path = "seed_reveal.rs"]
mod seed_reveal;

pub use self::seed_reveal::{reveal_seed_phrase, reveal_seed_phrase_once};

pub(super) fn seal_for_store(
    _rng: &mut impl rand::RngCore,
    key: &[u8; 32],
    aad: &[u8],
    plaintext: &[u8],
) -> WalletResult<Vec<u8>> {
    #[cfg(feature = "test-params-fast")]
    {
        use z00z_crypto::aead::test_only::seal_with_nonce_TEST_ONLY;
        let nonce = generate_24_bytes(_rng);
        seal_with_nonce_TEST_ONLY(key, aad, plaintext, nonce)
            .map_err(|_| WalletError::InvalidConfig("encrypt failed".to_string()))
    }

    #[cfg(not(feature = "test-params-fast"))]
    {
        use z00z_crypto::aead::seal;
        seal(key, aad, plaintext)
            .map_err(|_| WalletError::InvalidConfig("encrypt failed".to_string()))
    }
}

pub(super) fn encrypt_secret_record(
    rng: &mut impl rand::RngCore,
    wallet_id: &PersistWalletId,
    secret_name: &str,
    master_key: &[u8; 32],
    plaintext: &[u8],
) -> WalletResult<AeadEnvelope> {
    let aad = crate::db::wallet_store_crypto::aad_secret(wallet_id.0.as_bytes(), secret_name);

    let envelope_bytes = seal_for_store(rng, master_key, &aad, plaintext)
        .map_err(|_| WalletError::InvalidConfig("encrypt secret record failed".to_string()))?;
    Ok(AeadEnvelope {
        envelope: envelope_bytes,
    })
}

pub(super) fn encrypt_object_record(
    rng: &mut impl rand::RngCore,
    wallet_id: &PersistWalletId,
    data_key: &[u8; 32],
    object_id: u128,
    payload_version: u16,
    kind_id: u8,
    payload_bytes: Vec<u8>,
) -> WalletResult<EncryptedObjectRecord> {
    let mut container = EncryptedObjectPayload {
        payload_version,
        kind_id,
        data: payload_bytes,
    };

    let plaintext = encode_bincode(&container)?;
    container.data.fill(0);

    let mut payload_with_header = wrap_object_payload_with_header(plaintext)?;

    let aad = aad_object(wallet_id.0.as_bytes(), object_id, payload_version);

    let envelope = match seal_for_store(rng, data_key, &aad, &payload_with_header) {
        Ok(envelope_bytes) => AeadEnvelope {
            envelope: envelope_bytes,
        },
        Err(e) => {
            payload_with_header.fill(0);
            return Err(WalletError::InvalidConfig(format!(
                "encrypt object failed: {e}"
            )));
        }
    };

    payload_with_header.fill(0);

    Ok(EncryptedObjectRecord {
        envelope,
        payload_version,
    })
}

pub(super) fn map_bounded_auth_error(err: z00z_crypto::CryptoError) -> WalletError {
    match err {
        z00z_crypto::CryptoError::CryptoOperationFailed => WalletError::InvalidPassword,
        _ => WalletError::InvalidPassword,
    }
}

pub(super) fn decrypt_envelope_bounded(
    envelope: &AeadEnvelope,
    key: &[u8; 32],
    aad: &[u8],
) -> WalletResult<SecretBytes> {
    use z00z_crypto::aead::open;
    open(key, aad, &envelope.envelope)
        .map(SecretBytes::new)
        .map_err(map_bounded_auth_error)
}

pub(super) fn decrypt_secret_record(
    wallet_id: &PersistWalletId,
    secret_name: &str,
    master_key: &[u8; 32],
    record: &SecretsRecord,
) -> WalletResult<SecretBytes> {
    let aad = crate::db::wallet_store_crypto::aad_secret(wallet_id.0.as_bytes(), secret_name);
    decrypt_envelope_bounded(&record.envelope, master_key, &aad)
}

pub(super) fn decrypt_secret_record_post_unlock(
    wallet_id: &PersistWalletId,
    secret_name: &str,
    master_key: &[u8; 32],
    record: &SecretsRecord,
) -> WalletResult<SecretBytes> {
    let aad = crate::db::wallet_store_crypto::aad_secret(wallet_id.0.as_bytes(), secret_name);
    use z00z_crypto::aead::open;
    open(master_key, &aad, &record.envelope.envelope)
        .map(SecretBytes::new)
        .map_err(|_| WalletError::InvalidConfig(WALLET_SECRET_INVALID.to_string()))
}

/// Verify that the provided password matches the active session.
pub fn verify_password_for_session(
    session: &WalletSession,
    password: &SafePassword,
) -> WalletResult<()> {
    use subtle::ConstantTimeEq;

    let read_txn = session
        .db
        .begin_read()
        .map_err(|e| WalletError::InvalidConfig(format!("redb begin_read failed: {e}")))?;

    let secrets = read_txn
        .open_table(SECRETS_TABLE)
        .map_err(|e| WalletError::InvalidConfig(format!("redb open secrets failed: {e}")))?;

    let master_key_record_bytes = secrets
        .get(SECRETS_MASTER_KEY)
        .map_err(|e| WalletError::InvalidConfig(format!("redb secrets read failed: {e}")))?
        .ok_or_else(|| WalletError::InvalidConfig("missing secrets.master_key".to_string()))?;

    let mut master_key_record: MasterKeyRecord = decode_bincode_bounded(
        master_key_record_bytes.value(),
        WalletError::InvalidPassword,
    )?;

    if let Some(record_kdf) = master_key_record.kdf_params.as_ref() {
        if record_kdf != &session.opened.kdf_params {
            return Err(WalletError::InvalidConfig(
                "kdf params mismatch".to_string(),
            ));
        }
    } else {
        master_key_record.kdf_params = Some(session.opened.kdf_params.clone());
    }

    let km = WalletRedbKeyManager::new();

    let computed_master_key = km
        .unwrap_master_key(
            &session.opened.wallet_id,
            password,
            &session.opened.kdf_params,
            &master_key_record,
        )
        .map_err(|_| WalletError::InvalidPassword)?;

    let ok = computed_master_key
        .reveal()
        .ct_eq(session.opened.master_key.reveal())
        .unwrap_u8()
        != 0;

    if !ok {
        return Err(WalletError::InvalidPassword);
    }

    Ok(())
}

pub(super) fn commit_redb_write_txn_flush(
    session: &WalletSession,
    write_txn: redb::WriteTransaction,
) -> WalletResult<()> {
    write_txn
        .commit()
        .map_err(|e| WalletError::InvalidConfig(format!("redb commit failed: {e}")))?;

    session.flush_if_zstd()?;
    Ok(())
}

pub(super) fn update_wallet_integrity(
    meta: &mut redb::Table<'_, &str, &[u8]>,
    save_seq: u64,
) -> WalletResult<()> {
    use crate::domains::hashing::{canonicalize_bytes, WalletIntegrityHasher};

    let codec = BincodeCodec;
    let bytes = codec
        .serialize(&save_seq)
        .map_err(|e| WalletError::SerializationError(e.to_string()))?;

    let hash = WalletIntegrityHasher::new_with_label("wallet_integrity")
        .chain(canonicalize_bytes(&bytes))
        .finalize();

    let mut mac = [0u8; 32];
    mac.copy_from_slice(&hash.as_ref()[..32]);

    meta.insert(META_WALLET_INTEGRITY, mac.as_slice())
        .map_err(|e| WalletError::InvalidConfig(format!("redb meta insert failed: {e}")))?;
    Ok(())
}

pub(super) fn decrypt_object_record(
    wallet_id: &PersistWalletId,
    derived: &WalletDerivedKeys,
    object_id: u128,
    record: &EncryptedObjectRecord,
) -> WalletResult<EncryptedObjectPayload> {
    let min_envelope_size = 1 + 24 + 16;
    if record.envelope.envelope.len() < min_envelope_size {
        return Err(WalletError::InvalidConfig(
            WALLET_OBJECT_PAYLOAD_INVALID.to_string(),
        ));
    }

    let ciphertext_with_tag_len = record.envelope.envelope.len() - 1 - 24;
    let ciphertext_len = ciphertext_with_tag_len - 16;

    if ciphertext_len > OBJECT_PAYLOAD_HEADER_LEN + MAX_COMPRESSED_BYTES {
        return Err(WalletError::InvalidConfig(
            WALLET_OBJECT_PAYLOAD_INVALID.to_string(),
        ));
    }

    let aad = aad_object(wallet_id.0.as_bytes(), object_id, record.payload_version);
    let mut plaintext =
        decrypt_envelope_bounded(&record.envelope, derived.data_key.reveal(), &aad)?;
    let mut container_bytes = match unwrap_object_payload_with_header(&plaintext) {
        Ok(bytes) => bytes,
        Err(e) => {
            plaintext.wipe();
            return Err(e);
        }
    };

    let payload: EncryptedObjectPayload = match decode_bincode_bounded(
        &container_bytes,
        WalletError::InvalidConfig(WALLET_OBJECT_PAYLOAD_INVALID.to_string()),
    ) {
        Ok(payload) => payload,
        Err(e) => {
            plaintext.wipe();
            container_bytes.fill(0);
            return Err(e);
        }
    };

    plaintext.wipe();
    container_bytes.fill(0);

    if payload.payload_version != record.payload_version {
        return Err(WalletError::InvalidConfig(
            "object payload_version mismatch".to_string(),
        ));
    }

    if !is_supported_payload_version(payload.kind_id, payload.payload_version) {
        return Err(WalletError::UnsupportedVersion(
            payload.payload_version as u32,
        ));
    }

    Ok(payload)
}
