use super::{
    decode_bincode, decode_object_record_bounded, decrypt_object_record, encode_object_id_be,
    is_valid_mode, is_valid_tofu_lvl, object_id_from_be_bytes, validate_index_update,
    EncryptedObjectPayload, IndexManifestEntry, IndexTable, KeysPayload, ObjectKindId,
    ReadableDatabase, ReadableTable, ScanStatePayload, StealthMetaPayload, TableDefinition,
    TofuPinsPayload, WalletError, WalletResult, WalletSession, INDEX_MANIFEST_TABLE,
    META_KEYS_OBJECT_ID, META_SCAN_STATE_OBJECT_ID, META_STEALTH_META_OBJECT_ID, META_TABLE,
    META_TOFU_PINS_OBJECT_ID, OBJECTS_TABLE, PAYLOAD_VERSION_KEYS, PAYLOAD_VERSION_SCAN_STATE,
    PAYLOAD_VERSION_STEALTH_META, PAYLOAD_VERSION_TOFU_PINS,
};

// Phase 047 lands index-query primitives before later wallet-service waves call them.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct IndexedObjectPage {
    pub(crate) object_ids: Vec<u128>,
    pub(crate) next_cursor: Option<Vec<u8>>,
    pub(crate) has_more: bool,
}

/// Read and decrypt an object payload from a `.wlt` file.
///
/// This decrypts the stored `EncryptedObjectRecord` under `DATA_KEY` using the
/// canonical object AAD (`wallet_id_aad16 || object_id_be || payload_version_be`).
///
/// Authentication failures are intentionally bounded as `WalletError::InvalidPassword`.
pub fn read_object_by_id(
    session: &WalletSession,
    object_id: u128,
) -> WalletResult<EncryptedObjectPayload> {
    let read_txn = session
        .db
        .begin_read()
        .map_err(|e| WalletError::InvalidConfig(format!("redb begin_read failed: {e}")))?;

    let objects = read_txn
        .open_table(OBJECTS_TABLE)
        .map_err(|e| WalletError::InvalidConfig(format!("redb open objects failed: {e}")))?;

    let key = encode_object_id_be(object_id);
    let record_bytes = objects
        .get(key.as_slice())
        .map_err(|e| WalletError::InvalidConfig(format!("redb objects read failed: {e}")))?
        .ok_or_else(|| WalletError::InvalidConfig("object not found".to_string()))?;
    let record = decode_object_record_bounded(record_bytes.value())?;

    decrypt_object_record(
        &session.opened.wallet_id,
        &session.opened.derived_keys,
        object_id,
        &record,
    )
}

pub(crate) fn read_objects_by_index(
    session: &WalletSession,
    table: IndexTable,
    semantic_prefix: &[u8],
    limit: usize,
    cursor: Option<Vec<u8>>,
) -> WalletResult<IndexedObjectPage> {
    if let Some(cursor_bytes) = cursor.as_deref() {
        let (cursor_table, _, _) = crate::db::index_codecs::decode_index_key(cursor_bytes)?;
        if cursor_table != table {
            return Err(WalletError::InvalidConfig(
                "index cursor table mismatch".to_string(),
            ));
        }
    }

    let exact_semantic = if semantic_prefix.is_empty() {
        None
    } else {
        crate::db::index_codecs::validate_index_semantic_key(semantic_prefix)?;
        Some(semantic_prefix)
    };

    if limit == 0 {
        return Ok(IndexedObjectPage {
            object_ids: Vec::new(),
            next_cursor: None,
            has_more: false,
        });
    }

    let read_txn = session
        .db
        .begin_read()
        .map_err(|e| WalletError::InvalidConfig(format!("redb begin_read failed: {e}")))?;
    let def: TableDefinition<&[u8], &[u8]> = TableDefinition::new(table.store_name());
    let index = read_txn.open_table(def).map_err(|e| {
        WalletError::InvalidConfig(format!("redb open {} failed: {e}", table.store_name()))
    })?;

    let iter = if let Some(cursor_bytes) = cursor.as_deref() {
        index
            .range::<&[u8]>((
                std::ops::Bound::Excluded(cursor_bytes),
                std::ops::Bound::Unbounded,
            ))
            .map_err(|e| WalletError::InvalidConfig(format!("redb index range failed: {e}")))?
    } else {
        index
            .iter()
            .map_err(|e| WalletError::InvalidConfig(format!("redb index iter failed: {e}")))?
    };

    let mut object_ids = Vec::with_capacity(limit);
    let mut next_cursor = None;
    let mut has_more = false;

    for row in iter {
        let (key_guard, _value_guard) =
            row.map_err(|e| WalletError::InvalidConfig(format!("redb index read failed: {e}")))?;
        let key_bytes = key_guard.value();
        let (_, _, object_id) = crate::db::index_codecs::decode_index_key(key_bytes)?;

        if !index_row_matches_query(session, table, exact_semantic, key_bytes, object_id)? {
            continue;
        }

        if object_ids.len() == limit {
            has_more = true;
            break;
        }

        object_ids.push(object_id);
        next_cursor = Some(key_bytes.to_vec());
    }

    Ok(IndexedObjectPage {
        object_ids,
        next_cursor: has_more.then_some(next_cursor).flatten(),
        has_more,
    })
}

pub(crate) fn validate_object_index_rows(
    session: &WalletSession,
    object_id: u128,
) -> WalletResult<()> {
    let read_txn = session
        .db
        .begin_read()
        .map_err(|e| WalletError::InvalidConfig(format!("redb begin_read failed: {e}")))?;
    let manifest = read_txn
        .open_table(INDEX_MANIFEST_TABLE)
        .map_err(|e| WalletError::InvalidConfig(format!("redb open index_manifest failed: {e}")))?;
    let manifest_key = encode_object_id_be(object_id);
    let Some(raw_manifest) = manifest
        .get(manifest_key.as_slice())
        .map_err(|e| WalletError::InvalidConfig(format!("redb index_manifest read failed: {e}")))?
    else {
        return Ok(());
    };
    let entries: Vec<IndexManifestEntry> = decode_bincode(raw_manifest.value())?;

    for entry in entries {
        let def: TableDefinition<&[u8], &[u8]> = TableDefinition::new(entry.table.store_name());
        let table = read_txn.open_table(def).map_err(|e| {
            WalletError::InvalidConfig(format!(
                "redb open {} failed: {e}",
                entry.table.store_name()
            ))
        })?;
        let Some(value) = table
            .get(entry.key.as_slice())
            .map_err(|e| WalletError::InvalidConfig(format!("redb index read failed: {e}")))?
        else {
            return Err(WalletError::InvalidConfig(
                "index manifest row missing from index table".to_string(),
            ));
        };
        let (decoded_table, _, decoded_object_id) =
            crate::db::index_codecs::decode_index_key(entry.key.as_slice())?;
        if decoded_table != entry.table || decoded_object_id != object_id {
            return Err(WalletError::InvalidConfig(
                "index manifest row drifted from owning object".to_string(),
            ));
        }
        validate_index_update(entry.table, entry.key.as_slice(), value.value())?;
    }

    Ok(())
}

fn index_row_matches_query(
    session: &WalletSession,
    table: IndexTable,
    exact_semantic: Option<&[u8]>,
    key_bytes: &[u8],
    object_id: u128,
) -> WalletResult<bool> {
    let Some(semantic_key) = exact_semantic else {
        return Ok(true);
    };

    // Mode A privacy keys HMAC the semantic key together with the object id,
    // so partial-prefix scans are not safely reconstructable. The only honest
    // non-empty query supported here is an exact canonical semantic key match.
    let expected = crate::db::index_codecs::encode_index_key_mode(
        session.opened.derived_keys.index_key.reveal(),
        crate::db::index_codecs::IndexKeyMode::A,
        table,
        semantic_key,
        object_id,
    )?;
    Ok(expected.as_slice() == key_bytes)
}

pub fn read_stealth_meta(session: &WalletSession) -> WalletResult<Option<StealthMetaPayload>> {
    let read_txn = session
        .db
        .begin_read()
        .map_err(|e| WalletError::InvalidConfig(format!("redb begin_read failed: {e}")))?;

    let meta = read_txn
        .open_table(META_TABLE)
        .map_err(|e| WalletError::InvalidConfig(format!("redb open meta failed: {e}")))?;

    let object_id = match meta
        .get(META_STEALTH_META_OBJECT_ID)
        .map_err(|e| WalletError::InvalidConfig(format!("redb meta read failed: {e}")))?
    {
        Some(raw) => object_id_from_be_bytes(raw.value())?,
        None => return Ok(None),
    };

    let payload = read_object_by_id(session, object_id)?;
    if payload.kind_id != ObjectKindId::StealthMeta as u8
        || payload.payload_version != PAYLOAD_VERSION_STEALTH_META
    {
        return Err(WalletError::InvalidConfig(
            "stealth meta payload invalid".to_string(),
        ));
    }

    let decoded: StealthMetaPayload = decode_bincode(&payload.data)?;
    if !is_valid_mode(&decoded.receiver_mode) {
        return Err(WalletError::InvalidConfig(
            "stealth meta mode invalid".to_string(),
        ));
    }

    Ok(Some(decoded))
}

pub fn read_scan_state(session: &WalletSession) -> WalletResult<Option<ScanStatePayload>> {
    let read_txn = session
        .db
        .begin_read()
        .map_err(|e| WalletError::InvalidConfig(format!("redb begin_read failed: {e}")))?;

    let meta = read_txn
        .open_table(META_TABLE)
        .map_err(|e| WalletError::InvalidConfig(format!("redb open meta failed: {e}")))?;

    let object_id = match meta
        .get(META_SCAN_STATE_OBJECT_ID)
        .map_err(|e| WalletError::InvalidConfig(format!("redb meta read failed: {e}")))?
    {
        Some(raw) => object_id_from_be_bytes(raw.value())?,
        None => return Ok(None),
    };

    let payload = read_object_by_id(session, object_id)?;
    if payload.kind_id != ObjectKindId::ScanState as u8
        || payload.payload_version != PAYLOAD_VERSION_SCAN_STATE
    {
        return Err(WalletError::InvalidConfig(
            "scan state payload invalid".to_string(),
        ));
    }

    decode_bincode(&payload.data).map(Some)
}

pub fn read_keys_payload(session: &WalletSession) -> WalletResult<Option<KeysPayload>> {
    let read_txn = session
        .db
        .begin_read()
        .map_err(|e| WalletError::InvalidConfig(format!("redb begin_read failed: {e}")))?;

    let meta = read_txn
        .open_table(META_TABLE)
        .map_err(|e| WalletError::InvalidConfig(format!("redb open meta failed: {e}")))?;

    let object_id = match meta
        .get(META_KEYS_OBJECT_ID)
        .map_err(|e| WalletError::InvalidConfig(format!("redb meta read failed: {e}")))?
    {
        Some(raw) => object_id_from_be_bytes(raw.value())?,
        None => return Ok(None),
    };

    let payload = read_object_by_id(session, object_id)?;
    if payload.kind_id != ObjectKindId::Keys as u8
        || payload.payload_version != PAYLOAD_VERSION_KEYS
    {
        return Err(WalletError::InvalidConfig(
            "keys payload invalid".to_string(),
        ));
    }

    decode_bincode(&payload.data).map(Some)
}

pub fn read_tofu_pins(session: &WalletSession) -> WalletResult<Option<TofuPinsPayload>> {
    let read_txn = session
        .db
        .begin_read()
        .map_err(|e| WalletError::InvalidConfig(format!("redb begin_read failed: {e}")))?;

    let meta = read_txn
        .open_table(META_TABLE)
        .map_err(|e| WalletError::InvalidConfig(format!("redb open meta failed: {e}")))?;

    let object_id = match meta
        .get(META_TOFU_PINS_OBJECT_ID)
        .map_err(|e| WalletError::InvalidConfig(format!("redb meta read failed: {e}")))?
    {
        Some(raw) => object_id_from_be_bytes(raw.value())?,
        None => return Ok(None),
    };

    let payload = read_object_by_id(session, object_id)?;
    if payload.kind_id != ObjectKindId::TofuPins as u8
        || payload.payload_version != PAYLOAD_VERSION_TOFU_PINS
    {
        return Err(WalletError::InvalidConfig(
            "tofu pins payload invalid".to_string(),
        ));
    }

    let decoded: TofuPinsPayload = decode_bincode(&payload.data)?;
    for pin in &decoded.pins {
        if !is_valid_tofu_lvl(pin.trust_level) {
            return Err(WalletError::InvalidConfig(
                "tofu trust level invalid".to_string(),
            ));
        }
    }

    Ok(Some(decoded))
}
