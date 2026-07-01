use super::{
    allocate_object_id, commit_redb_write_txn_flush, encode_bincode, encrypt_object_record,
    is_valid_mode, is_valid_tofu_lvl, object_id_from_be_bytes, object_id_to_be_bytes,
    read_stealth_meta, write_object_with_indexes, KeysPayload, ModeAuditEntry, ObjectKindId,
    ReadableTable, ScanStatePayload, SecureRngProvider, StealthMetaPayload, TofuPinsPayload,
    WalletError, WalletResult, WalletSession, META_KEYS_OBJECT_ID, META_SCAN_STATE_OBJECT_ID,
    META_STEALTH_META_OBJECT_ID, META_TABLE, META_TOFU_PINS_OBJECT_ID, OBJECTS_TABLE,
    PAYLOAD_VERSION_KEYS, PAYLOAD_VERSION_SCAN_STATE, PAYLOAD_VERSION_STEALTH_META,
    PAYLOAD_VERSION_TOFU_PINS,
};

pub fn upsert_scan_state<R: SecureRngProvider>(
    session: &WalletSession,
    payload: &ScanStatePayload,
    rng_provider: R,
) -> WalletResult<u128> {
    let payload_bytes = encode_bincode(payload)?;

    let write_txn = session
        .db
        .begin_write()
        .map_err(|e| WalletError::InvalidConfig(format!("redb begin_write failed: {e}")))?;

    let _object_id = upsert_scan_state_with_txn(session, &write_txn, payload_bytes, rng_provider)?;

    commit_redb_write_txn_flush(session, write_txn)?;
    Ok(_object_id)
}

pub fn upsert_keys_payload<R: SecureRngProvider>(
    session: &WalletSession,
    payload: &KeysPayload,
    rng_provider: R,
) -> WalletResult<u128> {
    let payload_bytes = encode_bincode(payload)?;

    let write_txn = session
        .db
        .begin_write()
        .map_err(|e| WalletError::InvalidConfig(format!("redb begin_write failed: {e}")))?;

    let mut rng = rng_provider.rng();
    let object_id = {
        let mut meta = write_txn
            .open_table(META_TABLE)
            .map_err(|e| WalletError::InvalidConfig(format!("redb open meta failed: {e}")))?;

        let existing = meta
            .get(META_KEYS_OBJECT_ID)
            .map_err(|e| WalletError::InvalidConfig(format!("redb meta read failed: {e}")))?
            .map(|raw| raw.value().to_vec());

        match existing {
            Some(raw) => object_id_from_be_bytes(raw.as_slice())?,
            None => {
                let objects = write_txn.open_table(OBJECTS_TABLE).map_err(|e| {
                    WalletError::InvalidConfig(format!("redb open objects failed: {e}"))
                })?;
                let created = allocate_object_id(&objects, &mut rng)?;
                meta.insert(
                    META_KEYS_OBJECT_ID,
                    object_id_to_be_bytes(created).as_slice(),
                )
                .map_err(|e| WalletError::InvalidConfig(format!("redb meta insert failed: {e}")))?;
                created
            }
        }
    };

    let record = encrypt_object_record(
        &mut rng,
        &session.opened.wallet_id,
        session.opened.derived_keys.data_key.reveal(),
        object_id,
        PAYLOAD_VERSION_KEYS,
        ObjectKindId::Keys as u8,
        payload_bytes,
    )?;

    let _save_seq = write_object_with_indexes(session, &write_txn, object_id, &record, &[])?;

    commit_redb_write_txn_flush(session, write_txn)?;
    Ok(object_id)
}

pub(crate) fn upsert_scan_state_with_txn<R: SecureRngProvider>(
    session: &WalletSession,
    write_txn: &redb::WriteTransaction,
    payload_bytes: Vec<u8>,
    rng_provider: R,
) -> WalletResult<u128> {
    let mut rng = rng_provider.rng();
    let object_id = {
        let mut meta = write_txn
            .open_table(META_TABLE)
            .map_err(|e| WalletError::InvalidConfig(format!("redb open meta failed: {e}")))?;

        let existing = meta
            .get(META_SCAN_STATE_OBJECT_ID)
            .map_err(|e| WalletError::InvalidConfig(format!("redb meta read failed: {e}")))?
            .map(|raw| raw.value().to_vec());

        match existing {
            Some(raw) => object_id_from_be_bytes(raw.as_slice())?,
            None => {
                let objects = write_txn.open_table(OBJECTS_TABLE).map_err(|e| {
                    WalletError::InvalidConfig(format!("redb open objects failed: {e}"))
                })?;
                let created = allocate_object_id(&objects, &mut rng)?;
                meta.insert(
                    META_SCAN_STATE_OBJECT_ID,
                    object_id_to_be_bytes(created).as_slice(),
                )
                .map_err(|e| WalletError::InvalidConfig(format!("redb meta insert failed: {e}")))?;
                created
            }
        }
    };

    let record = encrypt_object_record(
        &mut rng,
        &session.opened.wallet_id,
        session.opened.derived_keys.data_key.reveal(),
        object_id,
        PAYLOAD_VERSION_SCAN_STATE,
        ObjectKindId::ScanState as u8,
        payload_bytes,
    )?;

    let _save_seq = write_object_with_indexes(session, write_txn, object_id, &record, &[])?;
    Ok(object_id)
}

pub fn upsert_stealth_meta<R: SecureRngProvider>(
    session: &WalletSession,
    payload: &StealthMetaPayload,
    rng_provider: R,
) -> WalletResult<u128> {
    if !is_valid_mode(&payload.receiver_mode) {
        return Err(WalletError::InvalidParams(
            "invalid stealth receiver mode".to_string(),
        ));
    }

    let prev = read_stealth_meta(session)?;
    let mut next = payload.clone();
    if let Some(prev_meta) = prev {
        if prev_meta.receiver_mode != next.receiver_mode {
            let mut audit = prev_meta.mode_audit;
            audit.push(ModeAuditEntry {
                from_mode: prev_meta.receiver_mode,
                to_mode: next.receiver_mode.clone(),
                changed_at: session.time_provider.compat_unix_timestamp_millis(),
            });
            next.mode_audit = audit;
        } else if next.mode_audit.is_empty() {
            next.mode_audit = prev_meta.mode_audit;
        }
    }

    let payload_bytes = encode_bincode(&next)?;
    let write_txn = session
        .db
        .begin_write()
        .map_err(|e| WalletError::InvalidConfig(format!("redb begin_write failed: {e}")))?;

    let mut rng = rng_provider.rng();
    let object_id = {
        let mut meta = write_txn
            .open_table(META_TABLE)
            .map_err(|e| WalletError::InvalidConfig(format!("redb open meta failed: {e}")))?;

        let existing = meta
            .get(META_STEALTH_META_OBJECT_ID)
            .map_err(|e| WalletError::InvalidConfig(format!("redb meta read failed: {e}")))?
            .map(|raw| raw.value().to_vec());

        match existing {
            Some(raw) => object_id_from_be_bytes(raw.as_slice())?,
            None => {
                let objects = write_txn.open_table(OBJECTS_TABLE).map_err(|e| {
                    WalletError::InvalidConfig(format!("redb open objects failed: {e}"))
                })?;
                let created = allocate_object_id(&objects, &mut rng)?;
                meta.insert(
                    META_STEALTH_META_OBJECT_ID,
                    object_id_to_be_bytes(created).as_slice(),
                )
                .map_err(|e| WalletError::InvalidConfig(format!("redb meta insert failed: {e}")))?;
                created
            }
        }
    };

    let record = encrypt_object_record(
        &mut rng,
        &session.opened.wallet_id,
        session.opened.derived_keys.data_key.reveal(),
        object_id,
        PAYLOAD_VERSION_STEALTH_META,
        ObjectKindId::StealthMeta as u8,
        payload_bytes,
    )?;

    let _save_seq = write_object_with_indexes(session, &write_txn, object_id, &record, &[])?;

    commit_redb_write_txn_flush(session, write_txn)?;
    Ok(object_id)
}

pub fn upsert_tofu_pins<R: SecureRngProvider>(
    session: &WalletSession,
    payload: &TofuPinsPayload,
    rng_provider: R,
) -> WalletResult<u128> {
    for pin in &payload.pins {
        if !is_valid_tofu_lvl(pin.trust_level) {
            return Err(WalletError::InvalidParams(
                "invalid tofu trust level".to_string(),
            ));
        }
    }

    let payload_bytes = encode_bincode(payload)?;

    let write_txn = session
        .db
        .begin_write()
        .map_err(|e| WalletError::InvalidConfig(format!("redb begin_write failed: {e}")))?;

    let mut rng = rng_provider.rng();
    let object_id = {
        let mut meta = write_txn
            .open_table(META_TABLE)
            .map_err(|e| WalletError::InvalidConfig(format!("redb open meta failed: {e}")))?;

        let existing = meta
            .get(META_TOFU_PINS_OBJECT_ID)
            .map_err(|e| WalletError::InvalidConfig(format!("redb meta read failed: {e}")))?
            .map(|raw| raw.value().to_vec());

        match existing {
            Some(raw) => object_id_from_be_bytes(raw.as_slice())?,
            None => {
                let objects = write_txn.open_table(OBJECTS_TABLE).map_err(|e| {
                    WalletError::InvalidConfig(format!("redb open objects failed: {e}"))
                })?;
                let created = allocate_object_id(&objects, &mut rng)?;
                meta.insert(
                    META_TOFU_PINS_OBJECT_ID,
                    object_id_to_be_bytes(created).as_slice(),
                )
                .map_err(|e| WalletError::InvalidConfig(format!("redb meta insert failed: {e}")))?;
                created
            }
        }
    };

    let record = encrypt_object_record(
        &mut rng,
        &session.opened.wallet_id,
        session.opened.derived_keys.data_key.reveal(),
        object_id,
        PAYLOAD_VERSION_TOFU_PINS,
        ObjectKindId::TofuPins as u8,
        payload_bytes,
    )?;

    let _save_seq = write_object_with_indexes(session, &write_txn, object_id, &record, &[])?;

    commit_redb_write_txn_flush(session, write_txn)?;
    Ok(object_id)
}
