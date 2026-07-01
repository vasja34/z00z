use super::{
    allocate_object_id, commit_redb_write_txn_flush, decode_object_id_be,
    decode_object_record_bounded, decrypt_object_record, encode_object_id_be,
    encrypt_object_record, object_id_from_be_bytes, object_id_to_be_bytes,
    write_object_with_indexes, ObjectKindId, ReadableDatabase, ReadableTable, SecretBytes,
    SecureRngProvider, WalletError, WalletResult, WalletSession, META_TABLE,
    META_WALLET_PROFILE_OBJECT_ID, OBJECTS_TABLE, PAYLOAD_VERSION_WALLET_PROFILE,
};

fn ensure_singleton_object_id(
    write_txn: &redb::WriteTransaction,
    meta_key: &str,
    rng: &mut impl rand::RngCore,
) -> WalletResult<u128> {
    {
        let meta = write_txn
            .open_table(META_TABLE)
            .map_err(|e| WalletError::InvalidConfig(format!("redb open meta failed: {e}")))?;
        let existing_raw = meta
            .get(meta_key)
            .map_err(|e| WalletError::InvalidConfig(format!("redb meta read failed: {e}")))?
            .map(|guard| guard.value().to_vec());
        if let Some(raw) = existing_raw {
            return object_id_from_be_bytes(&raw);
        }
    }

    let object_id = {
        let objects = write_txn
            .open_table(OBJECTS_TABLE)
            .map_err(|e| WalletError::InvalidConfig(format!("redb open objects failed: {e}")))?;
        allocate_object_id(&objects, rng)?
    };

    let mut meta = write_txn
        .open_table(META_TABLE)
        .map_err(|e| WalletError::InvalidConfig(format!("redb open meta failed: {e}")))?;
    meta.insert(meta_key, object_id_to_be_bytes(object_id).as_slice())
        .map_err(|e| WalletError::InvalidConfig(format!("redb meta insert failed: {e}")))?;
    Ok(object_id)
}

/// Persist an encrypted wallet profile payload into the `.wlt` file.
pub fn write_wallet_profile<R: SecureRngProvider>(
    session: &WalletSession,
    profile_bytes: Vec<u8>,
    rng_provider: R,
) -> WalletResult<u64> {
    let write_txn = session
        .db
        .begin_write()
        .map_err(|e| WalletError::InvalidConfig(format!("redb begin_write failed: {e}")))?;

    let mut rng = rng_provider.rng();
    let profile_object_id =
        ensure_singleton_object_id(&write_txn, META_WALLET_PROFILE_OBJECT_ID, &mut rng)?;

    let record = encrypt_object_record(
        &mut rng,
        &session.opened.wallet_id,
        session.opened.derived_keys.data_key.reveal(),
        profile_object_id,
        PAYLOAD_VERSION_WALLET_PROFILE,
        ObjectKindId::WalletProfile as u8,
        profile_bytes,
    )?;

    let new_save_seq =
        write_object_with_indexes(session, &write_txn, profile_object_id, &record, &[])?;

    commit_redb_write_txn_flush(session, write_txn)?;
    Ok(new_save_seq)
}

/// Read and decrypt the persisted wallet profile bytes from the `.wlt` file.
pub fn read_wallet_profile(session: &WalletSession) -> WalletResult<SecretBytes> {
    let read_txn = session
        .db
        .begin_read()
        .map_err(|e| WalletError::InvalidConfig(format!("redb begin_read failed: {e}")))?;

    let meta = read_txn
        .open_table(META_TABLE)
        .map_err(|e| WalletError::InvalidConfig(format!("redb open meta failed: {e}")))?;

    let Some(profile_id_bytes) = meta
        .get(META_WALLET_PROFILE_OBJECT_ID)
        .map_err(|e| WalletError::InvalidConfig(format!("redb meta read failed: {e}")))?
    else {
        return Err(WalletError::NotFound(0));
    };
    let profile_object_id = decode_object_id_be(profile_id_bytes.value())?;

    let objects = read_txn
        .open_table(OBJECTS_TABLE)
        .map_err(|e| WalletError::InvalidConfig(format!("redb open objects failed: {e}")))?;

    let key = encode_object_id_be(profile_object_id);
    let record_bytes = objects
        .get(key.as_slice())
        .map_err(|e| WalletError::InvalidConfig(format!("redb objects read failed: {e}")))?
        .ok_or_else(|| WalletError::InvalidConfig("wallet profile object not found".to_string()))?;
    let record = decode_object_record_bounded(record_bytes.value())?;

    let payload = decrypt_object_record(
        &session.opened.wallet_id,
        &session.opened.derived_keys,
        profile_object_id,
        &record,
    )?;

    if payload.kind_id != ObjectKindId::WalletProfile as u8 {
        return Err(WalletError::InvalidConfig(
            "wallet profile object kind mismatch".to_string(),
        ));
    }
    if payload.payload_version != PAYLOAD_VERSION_WALLET_PROFILE {
        return Err(WalletError::UnsupportedVersion(
            payload.payload_version as u32,
        ));
    }

    Ok(SecretBytes::new(payload.data))
}
