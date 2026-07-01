#[cfg(feature = "wallet_debug_tools")]
use super::super::tables::{
    INDEX_ACCOUNT_BY_LABEL_TABLE, INDEX_ASSET_DEF_SYMBOL_TABLE, INDEX_ASSET_DEF_TABLE,
    INDEX_ASSET_SPENTFLAG_TABLE, INDEX_PENDING_STATUS_EXPIRY_TABLE, INDEX_RECEIPT_BY_TXHASH_TABLE,
    INDEX_RECEIVER_BY_KIND_TABLE, INDEX_TRACKED_ASSET_SPENTFLAG_TABLE, INDEX_TX_BY_STATUS_TABLE,
    INDEX_TX_BY_TIME_TABLE, INDEX_WALLET_ID_TABLE, OWNED_ASSET_ID_TABLE, OWNED_ASSET_SCAN_TABLE,
    OWNED_ASSET_STATUS_TABLE, OWNED_ASSET_TX_TABLE, OWNED_DEF_STATUS_TABLE,
    OWNED_OBJECT_FAMILY_TABLE, OWNED_OBJECT_HOLDER_TABLE, OWNED_OBJECT_POLICY_TABLE,
    OWNED_OBJECT_STATUS_TABLE, OWNED_RIGHT_ID_TABLE, OWNED_VOUCHER_ID_TABLE,
};
#[cfg(feature = "wallet_debug_tools")]
use super::super::{
    decode_bincode_bounded, decode_object_id_be, decode_object_record_bounded,
    decode_seed_plaintext_phrase24, decrypt_object_record, decrypt_secret_record,
    open_wallet_store, MasterKeyRecord, PersistWalletId, SafePassword, SecretsRecord, WalletError,
    WalletIdentity, WalletRedbKeyManager, WalletResult, INDEX_MANIFEST_TABLE, META_TABLE,
    OBJECTS_TABLE, SECRETS_MASTER_KEY, SECRETS_SEED_MAIN, SECRETS_TABLE, WALLET_SECRET_INVALID,
};
#[cfg(feature = "wallet_debug_tools")]
use super::debug_types::{
    b64, decode_meta_value, decode_object_json, DebugIndexKey, DebugMetaEntry, DebugObjectEntry,
    DebugSecretEntry, DebugTableError, DebugTableRow, DebugWalletDump,
};
#[cfg(feature = "wallet_debug_tools")]
use crate::db::index_codecs::decode_index_key;
#[cfg(feature = "wallet_debug_tools")]
use redb::{ReadableDatabase, ReadableTable};
#[cfg(feature = "wallet_debug_tools")]
use std::collections::BTreeMap;
#[cfg(feature = "wallet_debug_tools")]
use std::path::Path;
#[cfg(feature = "wallet_debug_tools")]
use subtle::ConstantTimeEq;
#[cfg(feature = "wallet_debug_tools")]
use z00z_utils::{
    codec::{Codec, JsonCodec},
    io::atomic_write_file_private,
};

#[cfg(feature = "wallet_debug_tools")]
use super::super::codecs::WALLET_MASTER_KEY_INVALID;

#[cfg(feature = "wallet_debug_tools")]
pub fn debug_export_wallet(
    wlt_path: &Path,
    wallet_id: &PersistWalletId,
    password: &SafePassword,
    identity: &WalletIdentity,
    out_path: &Path,
) -> WalletResult<()> {
    let session = open_wallet_store(wlt_path, wallet_id, password, identity)?;

    let read_txn = session
        .db
        .begin_read()
        .map_err(|e| WalletError::InvalidConfig(format!("redb begin_read failed: {e}")))?;

    let meta_table = read_txn
        .open_table(META_TABLE)
        .map_err(|e| WalletError::InvalidConfig(format!("redb open meta failed: {e}")))?;
    let secrets_table = read_txn
        .open_table(SECRETS_TABLE)
        .map_err(|e| WalletError::InvalidConfig(format!("redb open secrets failed: {e}")))?;
    let objects_table = read_txn
        .open_table(OBJECTS_TABLE)
        .map_err(|e| WalletError::InvalidConfig(format!("redb open objects failed: {e}")))?;

    let mut meta_entries = Vec::new();
    for row in meta_table
        .iter()
        .map_err(|e| WalletError::InvalidConfig(format!("redb meta iter failed: {e}")))?
    {
        let (k, v) =
            row.map_err(|e| WalletError::InvalidConfig(format!("redb meta read failed: {e}")))?;
        let key = k.value().to_string();
        let raw = v.value();
        meta_entries.push(DebugMetaEntry {
            key: key.clone(),
            raw_b64: b64(raw),
            decoded: decode_meta_value(&key, raw),
        });
    }
    meta_entries.sort_by(|a, b| a.key.cmp(&b.key));

    let mut secret_entries = Vec::new();
    let master_key = session.opened.master_key.reveal();
    for row in secrets_table
        .iter()
        .map_err(|e| WalletError::InvalidConfig(format!("redb secrets iter failed: {e}")))?
    {
        let (k, v) =
            row.map_err(|e| WalletError::InvalidConfig(format!("redb secrets read failed: {e}")))?;
        let name = k.value().to_string();
        let raw = v.value();

        if name == SECRETS_MASTER_KEY {
            let record_res: WalletResult<MasterKeyRecord> = decode_bincode_bounded(
                raw,
                WalletError::InvalidConfig(WALLET_MASTER_KEY_INVALID.to_string()),
            );

            let entry = match record_res {
                Ok(record) => {
                    let km = WalletRedbKeyManager::new();
                    let kdf = record.kdf_params.clone().ok_or_else(|| {
                        WalletError::InvalidConfig(
                            "missing secrets.master_key.kdf_params".to_string(),
                        )
                    });

                    match kdf.and_then(|kdf_params| {
                        km.unwrap_master_key(wallet_id, password, &kdf_params, &record)
                            .map_err(|e| WalletError::InvalidConfig(e.to_string()))
                            .map(|mk| (mk, kdf_params.version))
                    }) {
                        Ok((unwrapped, kdf_version)) => {
                            let revealed = unwrapped.reveal();
                            let matches_opened = revealed.ct_eq(master_key).unwrap_u8() == 1;
                            let record_error = if matches_opened {
                                None
                            } else {
                                Some("master key mismatch against opened session".to_string())
                            };

                            DebugSecretEntry {
                                name,
                                kind: "MasterKey".to_string(),
                                label: String::new(),
                                version: kdf_version,
                                record_raw_b64: b64(raw),
                                record_error,
                                plaintext_b64: b64(revealed),
                                plaintext_utf8: None,
                                seed_phrase: None,
                            }
                        }
                        Err(e) => DebugSecretEntry {
                            name,
                            kind: "MasterKey".to_string(),
                            label: String::new(),
                            version: 0,
                            record_raw_b64: b64(raw),
                            record_error: Some(e.to_string()),
                            plaintext_b64: String::new(),
                            plaintext_utf8: None,
                            seed_phrase: None,
                        },
                    }
                }
                Err(e) => DebugSecretEntry {
                    name,
                    kind: "decode_failed".to_string(),
                    label: String::new(),
                    version: 0,
                    record_raw_b64: b64(raw),
                    record_error: Some(e.to_string()),
                    plaintext_b64: String::new(),
                    plaintext_utf8: None,
                    seed_phrase: None,
                },
            };

            secret_entries.push(entry);
            continue;
        }

        let record_res: WalletResult<SecretsRecord> = decode_bincode_bounded(
            raw,
            WalletError::InvalidConfig(WALLET_SECRET_INVALID.to_string()),
        );

        let (record, record_error) = match record_res {
            Ok(r) => (Some(r), None),
            Err(e) => (None, Some(e.to_string())),
        };

        let (plaintext_b64, plaintext_utf8, seed_phrase, kind, label, version, decrypt_error) =
            if let Some(record) = record {
                match decrypt_secret_record(wallet_id, &name, master_key, &record) {
                    Ok(plaintext) => {
                        let plaintext_utf8 = std::str::from_utf8(plaintext.as_ref())
                            .ok()
                            .map(|s| s.to_string());

                        let seed_phrase = if name == SECRETS_SEED_MAIN {
                            decode_seed_plaintext_phrase24(plaintext.as_ref())
                                .ok()
                                .map(|p| p.with_phrase(|s| s.to_string()))
                        } else {
                            None
                        };

                        (
                            b64(plaintext.as_ref()),
                            plaintext_utf8,
                            seed_phrase,
                            format!("{:?}", record.kind),
                            record.label,
                            record.version,
                            None,
                        )
                    }
                    Err(e) => (
                        String::new(),
                        None,
                        None,
                        format!("{:?}", record.kind),
                        record.label,
                        record.version,
                        Some(e.to_string()),
                    ),
                }
            } else {
                (
                    String::new(),
                    None,
                    None,
                    "decode_failed".to_string(),
                    String::new(),
                    0,
                    None,
                )
            };

        let record_error = record_error.or(decrypt_error);

        secret_entries.push(DebugSecretEntry {
            name,
            kind,
            label,
            version,
            record_raw_b64: b64(raw),
            record_error,
            plaintext_b64,
            plaintext_utf8,
            seed_phrase,
        });
    }
    secret_entries.sort_by(|a, b| a.name.cmp(&b.name));

    let mut object_entries = Vec::new();
    for row in objects_table
        .iter()
        .map_err(|e| WalletError::InvalidConfig(format!("redb objects iter failed: {e}")))?
    {
        let (k, v) =
            row.map_err(|e| WalletError::InvalidConfig(format!("redb objects read failed: {e}")))?;
        let object_id = decode_object_id_be(k.value())?;
        let record = decode_object_record_bounded(v.value())?;
        let payload =
            decrypt_object_record(wallet_id, &session.opened.derived_keys, object_id, &record)?;

        let decoded = decode_object_json(payload.kind_id, payload.payload_version, &payload.data);
        object_entries.push(DebugObjectEntry {
            object_id_hex: format!("0x{object_id:032x}"),
            kind_id: payload.kind_id,
            payload_version: payload.payload_version,
            payload_len: payload.data.len(),
            payload_data_b64: b64(&payload.data),
            decoded,
        });
    }
    object_entries.sort_by(|a, b| a.object_id_hex.cmp(&b.object_id_hex));

    let mut tables: BTreeMap<String, Vec<DebugTableRow>> = BTreeMap::new();

    let mut index_keys: Vec<DebugIndexKey> = Vec::new();

    let mut table_errors: Vec<DebugTableError> = Vec::new();

    macro_rules! dump_index_table {
        ($table_def:ident, $table_name:literal) => {{
            match read_txn.open_table($table_def) {
                Ok(table) => {
                    let mut rows = Vec::new();
                    match table.iter() {
                        Ok(iter) => {
                            for row in iter {
                                let (k, v) = match row {
                                    Ok(pair) => pair,
                                    Err(e) => {
                                        table_errors.push(DebugTableError {
                                            table_name: $table_name.to_string(),
                                            error: format!("redb index read failed: {e}"),
                                        });
                                        continue;
                                    }
                                };

                                let key_bytes = k.value();
                                let value_bytes = v.value();

                                rows.push(DebugTableRow {
                                    key_b64: b64(key_bytes),
                                    value_b64: b64(value_bytes),
                                });

                                if let Ok((decoded_table, semantic, object_id)) =
                                    decode_index_key(key_bytes)
                                {
                                    index_keys.push(DebugIndexKey {
                                        table_name: $table_name.to_string(),
                                        table: format!("{:?}", decoded_table),
                                        object_id_hex: format!("0x{object_id:032x}"),
                                        semantic_b64: b64(&semantic),
                                    });
                                }
                            }
                        }
                        Err(e) => {
                            table_errors.push(DebugTableError {
                                table_name: $table_name.to_string(),
                                error: format!("redb index iter failed: {e}"),
                            });
                        }
                    }

                    tables.insert($table_name.to_string(), rows);
                }
                Err(e) => {
                    if !matches!(e, redb::TableError::TableDoesNotExist(_)) {
                        table_errors.push(DebugTableError {
                            table_name: $table_name.to_string(),
                            error: format!("redb open {} failed: {e}", stringify!($table_def)),
                        });
                    }
                    tables.insert($table_name.to_string(), Vec::new());
                }
            }
        }};
    }

    dump_index_table!(INDEX_ACCOUNT_BY_LABEL_TABLE, "index.account_by_label");
    dump_index_table!(INDEX_RECEIVER_BY_KIND_TABLE, "index.receiver_by_kind");
    dump_index_table!(INDEX_ASSET_DEF_SYMBOL_TABLE, "index.asset_def_by_symbol");
    dump_index_table!(INDEX_ASSET_DEF_TABLE, "index.asset_def");
    dump_index_table!(INDEX_ASSET_SPENTFLAG_TABLE, "index.asset_spentflag");
    dump_index_table!(
        INDEX_TRACKED_ASSET_SPENTFLAG_TABLE,
        "index.tracked_asset_spentflag"
    );
    dump_index_table!(INDEX_TX_BY_STATUS_TABLE, "index.tx_by_status");
    dump_index_table!(INDEX_TX_BY_TIME_TABLE, "index.tx_by_time");
    dump_index_table!(
        INDEX_PENDING_STATUS_EXPIRY_TABLE,
        "index.pending_status_expiry"
    );
    dump_index_table!(INDEX_RECEIPT_BY_TXHASH_TABLE, "index.receipt_by_txhash");
    dump_index_table!(INDEX_WALLET_ID_TABLE, "index.wallet_id");
    dump_index_table!(OWNED_ASSET_ID_TABLE, "index.owned_asset_by_id");
    dump_index_table!(OWNED_DEF_STATUS_TABLE, "index.owned_asset_by_def_status");
    dump_index_table!(OWNED_ASSET_STATUS_TABLE, "index.owned_asset_by_status");
    dump_index_table!(OWNED_ASSET_TX_TABLE, "index.owned_asset_by_tx");
    dump_index_table!(OWNED_ASSET_SCAN_TABLE, "index.owned_asset_by_scan");
    dump_index_table!(OWNED_OBJECT_FAMILY_TABLE, "index.owned_object_by_family");
    dump_index_table!(OWNED_OBJECT_STATUS_TABLE, "index.owned_object_by_status");
    dump_index_table!(OWNED_OBJECT_POLICY_TABLE, "index.owned_object_by_policy");
    dump_index_table!(OWNED_OBJECT_HOLDER_TABLE, "index.owned_object_by_holder");
    dump_index_table!(OWNED_VOUCHER_ID_TABLE, "index.owned_voucher_by_id");
    dump_index_table!(OWNED_RIGHT_ID_TABLE, "index.owned_right_by_id");
    dump_index_table!(INDEX_MANIFEST_TABLE, "index.manifest");

    index_keys.sort_by(|a, b| {
        a.table_name
            .cmp(&b.table_name)
            .then_with(|| a.object_id_hex.cmp(&b.object_id_hex))
    });

    let dump = DebugWalletDump {
        wlt_path: wlt_path.display().to_string(),
        wallet_id: wallet_id.0.clone(),
        schema_version: session.opened.schema_version,
        meta: meta_entries,
        secrets: Vec::new(),
        secrets_redacted: true,
        objects: object_entries,
        tables,
        index_keys,
        table_errors,
    };

    let json = JsonCodec
        .serialize_pretty(&dump)
        .map_err(|e| WalletError::InvalidConfig(format!("debug export json failed: {e}")))?;

    atomic_write_file_private(out_path, &json)
        .map_err(|e| WalletError::InvalidConfig(format!("debug export write failed: {e}")))?;

    Ok(())
}
