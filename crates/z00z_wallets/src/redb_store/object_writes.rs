use super::{
    allocate_object_id, bump_wallet_write_meta, commit_redb_write_txn_flush, decode_bincode,
    encode_bincode, encode_object_id_be, encrypt_object_record, is_supported_payload_version,
    store_object, validate_index_update, EncryptedObjectRecord, IndexKeyBytes, IndexManifestEntry,
    IndexTable, IndexUpdate, ReadableTable, SecureRngProvider, TableDefinition, TimeProvider,
    ValidatedIndexUpdate, WalletError, WalletResult, WalletSession, INDEX_MANIFEST_TABLE,
    META_TABLE, OBJECTS_TABLE,
};

/// Encrypt and write an object record to the `objects` table, and bump `meta["wallet.save_seq"]`.
///
/// The object insert and the `save_seq` update are committed together in exactly one RedB
/// `write_transaction`.
///
/// Task 4.5: The object id is generated inside the write transaction and returned to the caller.
pub fn write_object<R: SecureRngProvider>(
    session: &WalletSession,
    kind_id: u8,
    payload_version: u16,
    payload_bytes: Vec<u8>,
    index_updates: &[IndexUpdate],
    rng_provider: R,
) -> WalletResult<u128> {
    if !is_supported_payload_version(kind_id, payload_version) {
        return Err(WalletError::UnsupportedVersion(payload_version as u32));
    }

    let write_txn = session
        .db
        .begin_write()
        .map_err(|e| WalletError::InvalidConfig(format!("redb begin_write failed: {e}")))?;

    let mut rng = rng_provider.rng();
    let object_id = {
        let objects = write_txn
            .open_table(OBJECTS_TABLE)
            .map_err(|e| WalletError::InvalidConfig(format!("redb open objects failed: {e}")))?;
        allocate_object_id(&objects, &mut rng)?
    };

    let record = encrypt_object_record(
        &mut rng,
        &session.opened.wallet_id,
        session.opened.derived_keys.data_key.reveal(),
        object_id,
        payload_version,
        kind_id,
        payload_bytes,
    )?;

    let _new_save_seq =
        write_object_with_indexes(session, &write_txn, object_id, &record, index_updates)?;

    commit_redb_write_txn_flush(session, write_txn)?;
    Ok(object_id)
}

// Phase 047 lands object-rewrite primitives ahead of later wallet-service cutovers.
#[allow(clippy::too_many_arguments)]
pub(crate) fn write_object_by_id<R: SecureRngProvider>(
    session: &WalletSession,
    object_id: u128,
    kind_id: u8,
    payload_version: u16,
    payload_bytes: Vec<u8>,
    index_updates: &[IndexUpdate],
    rng_provider: R,
) -> WalletResult<u64> {
    write_payload_at_id(
        session,
        object_id,
        kind_id,
        payload_version,
        payload_bytes,
        index_updates,
        rng_provider,
        true,
    )
}

#[allow(clippy::too_many_arguments)]
pub(super) fn write_payload_at_id<R: SecureRngProvider>(
    session: &WalletSession,
    object_id: u128,
    kind_id: u8,
    payload_version: u16,
    payload_bytes: Vec<u8>,
    index_updates: &[IndexUpdate],
    rng_provider: R,
    require_exists: bool,
) -> WalletResult<u64> {
    if !is_supported_payload_version(kind_id, payload_version) {
        return Err(WalletError::UnsupportedVersion(payload_version as u32));
    }

    let write_txn = session
        .db
        .begin_write()
        .map_err(|e| WalletError::InvalidConfig(format!("redb begin_write failed: {e}")))?;

    if require_exists {
        let objects = write_txn
            .open_table(OBJECTS_TABLE)
            .map_err(|e| WalletError::InvalidConfig(format!("redb open objects failed: {e}")))?;
        let key = encode_object_id_be(object_id);
        let exists = objects
            .get(key.as_slice())
            .map_err(|e| WalletError::InvalidConfig(format!("redb objects read failed: {e}")))?
            .is_some();
        if !exists {
            return Err(WalletError::InvalidConfig("object not found".to_string()));
        }
    }

    let mut rng = rng_provider.rng();
    let record = encrypt_object_record(
        &mut rng,
        &session.opened.wallet_id,
        session.opened.derived_keys.data_key.reveal(),
        object_id,
        payload_version,
        kind_id,
        payload_bytes,
    )?;

    let new_save_seq =
        write_object_with_indexes(session, &write_txn, object_id, &record, index_updates)?;

    commit_redb_write_txn_flush(session, write_txn)?;
    Ok(new_save_seq)
}

pub(super) fn write_object_with_indexes(
    session: &WalletSession,
    write_txn: &redb::WriteTransaction,
    object_id: u128,
    record: &EncryptedObjectRecord,
    index_updates: &[IndexUpdate],
) -> WalletResult<u64> {
    write_object_with_index_key(
        write_txn,
        object_id,
        record,
        index_updates,
        session.opened.derived_keys.index_key.reveal(),
        session.time_provider.as_ref(),
    )
}

pub(super) fn write_object_with_index_key(
    write_txn: &redb::WriteTransaction,
    object_id: u128,
    record: &EncryptedObjectRecord,
    index_updates: &[IndexUpdate],
    index_key: &[u8; 32],
    time_provider: &dyn TimeProvider,
) -> WalletResult<u64> {
    {
        let mut objects = write_txn
            .open_table(OBJECTS_TABLE)
            .map_err(|e| WalletError::InvalidConfig(format!("redb open objects failed: {e}")))?;
        store_object(&mut objects, object_id, record)?;
    }

    let mut validated_updates = Vec::new();
    for update in index_updates {
        crate::db::index_codecs::validate_index_semantic_key(&update.semantic_key)?;

        let key_bytes = crate::db::index_codecs::encode_index_key_mode(
            index_key,
            crate::db::index_codecs::IndexKeyMode::A,
            update.table,
            &update.semantic_key,
            object_id,
        )?;

        validate_index_update(
            update.table,
            key_bytes.as_slice(),
            update.value.0.as_slice(),
        )?;

        validated_updates.push(ValidatedIndexUpdate {
            table: update.table,
            key: IndexKeyBytes::new(update.table, key_bytes)?,
            value: update.value.clone(),
        });
    }

    if !validated_updates.is_empty() {
        apply_index_updates(write_txn, object_id, &validated_updates)?;
    }

    let mut meta = write_txn
        .open_table(META_TABLE)
        .map_err(|e| WalletError::InvalidConfig(format!("redb open meta failed: {e}")))?;

    bump_wallet_write_meta(&mut meta, time_provider)
}

fn apply_index_updates(
    write_txn: &redb::WriteTransaction,
    object_id: u128,
    validated_updates: &[ValidatedIndexUpdate],
) -> WalletResult<()> {
    let mut manifest_table = write_txn
        .open_table(INDEX_MANIFEST_TABLE)
        .map_err(|e| WalletError::InvalidConfig(format!("redb open index_manifest failed: {e}")))?;

    let manifest_key = encode_object_id_be(object_id);
    let existing_manifest = manifest_table
        .get(manifest_key.as_slice())
        .map_err(|e| WalletError::InvalidConfig(format!("redb index_manifest read failed: {e}")))?
        .map(|guard| guard.value().to_vec());

    if let Some(raw) = existing_manifest {
        let entries: Vec<IndexManifestEntry> = decode_bincode(&raw)?;
        for entry in entries {
            remove_index_entry(write_txn, entry.table, entry.key.as_slice())?;
        }
    }

    let new_entries: Vec<IndexManifestEntry> = validated_updates
        .iter()
        .map(|update| IndexManifestEntry {
            table: update.table,
            key: update.key.0.clone(),
        })
        .collect();

    for update in validated_updates {
        insert_index_entry(
            write_txn,
            update.table,
            update.key.0.as_slice(),
            update.value.0.as_slice(),
        )?;
    }

    manifest_table
        .insert(
            manifest_key.as_slice(),
            encode_bincode(&new_entries)?.as_slice(),
        )
        .map_err(|e| {
            WalletError::InvalidConfig(format!("redb index_manifest insert failed: {e}"))
        })?;

    Ok(())
}

fn remove_index_entry(
    write_txn: &redb::WriteTransaction,
    table: IndexTable,
    key: &[u8],
) -> WalletResult<()> {
    let def: TableDefinition<&[u8], &[u8]> = TableDefinition::new(table.store_name());
    let mut table = write_txn.open_table(def).map_err(|e| {
        WalletError::InvalidConfig(format!("redb open {} failed: {e}", table.store_name()))
    })?;

    table
        .remove(key)
        .map_err(|e| WalletError::InvalidConfig(format!("redb index remove failed: {e}")))?;
    Ok(())
}

fn insert_index_entry(
    write_txn: &redb::WriteTransaction,
    table: IndexTable,
    key: &[u8],
    value: &[u8],
) -> WalletResult<()> {
    let def: TableDefinition<&[u8], &[u8]> = TableDefinition::new(table.store_name());
    let mut table = write_txn.open_table(def).map_err(|e| {
        WalletError::InvalidConfig(format!("redb open {} failed: {e}", table.store_name()))
    })?;

    table
        .insert(key, value)
        .map_err(|e| WalletError::InvalidConfig(format!("redb index insert failed: {e}")))?;
    Ok(())
}
