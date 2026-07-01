use super::{
    decode_bincode, encode_bincode, object_id_from_be_bytes, update_wallet_integrity, KdfParams,
    Path, PathBuf, PersistWalletId, ReadableTable, TimeProvider, WalletError, WalletIdentity,
    WalletResult, AAD_SECRET_VERSION, HKDF_INFO_VERSION, INDEX_FORMAT_VERSION_HMAC,
    META_AAD_SECRET_VERSION, META_APP_OBJECT_ID, META_CHAIN_OBJECT_ID, META_HKDF_INFO_VERSION,
    META_INDEX_FORMAT_VERSION, META_KEYS_OBJECT_ID, META_ROTATION_IN_PROGRESS,
    META_SCAN_STATE_OBJECT_ID, META_SCHEMA_VERSION, META_WALLET_CHAIN, META_WALLET_CREATED_AT,
    META_WALLET_ID, META_WALLET_INITIALIZED, META_WALLET_KDF, META_WALLET_NETWORK,
    META_WALLET_SAVE_SEQ, META_WALLET_UPDATED_AT, REDB_WALLET_SCHEMA_VERSION,
};

pub(super) fn store_required_meta(
    meta: &mut redb::Table<'_, &str, &[u8]>,
    wallet_id: &PersistWalletId,
    kdf_params: &KdfParams,
    identity: &WalletIdentity,
    now_ms: u64,
) -> WalletResult<()> {
    meta.insert(META_WALLET_ID, encode_bincode(wallet_id)?.as_slice())
        .map_err(|e| WalletError::InvalidConfig(format!("redb meta insert failed: {e}")))?;
    meta.insert(
        META_SCHEMA_VERSION,
        encode_bincode(&REDB_WALLET_SCHEMA_VERSION)?.as_slice(),
    )
    .map_err(|e| WalletError::InvalidConfig(format!("redb meta insert failed: {e}")))?;
    meta.insert(META_WALLET_KDF, encode_bincode(kdf_params)?.as_slice())
        .map_err(|e| WalletError::InvalidConfig(format!("redb meta insert failed: {e}")))?;

    // Initialization marker: required for crash-safety and deterministic open validation.
    let initialized: u8 = 1;
    meta.insert(
        META_WALLET_INITIALIZED,
        encode_bincode(&initialized)?.as_slice(),
    )
    .map_err(|e| WalletError::InvalidConfig(format!("redb meta insert failed: {e}")))?;

    meta.insert(META_WALLET_CREATED_AT, encode_bincode(&now_ms)?.as_slice())
        .map_err(|e| WalletError::InvalidConfig(format!("redb meta insert failed: {e}")))?;
    meta.insert(META_WALLET_UPDATED_AT, encode_bincode(&now_ms)?.as_slice())
        .map_err(|e| WalletError::InvalidConfig(format!("redb meta insert failed: {e}")))?;
    meta.insert(
        META_WALLET_CHAIN,
        encode_bincode(&identity.chain)?.as_slice(),
    )
    .map_err(|e| WalletError::InvalidConfig(format!("redb meta insert failed: {e}")))?;

    meta.insert(
        META_WALLET_NETWORK,
        encode_bincode(&identity.network)?.as_slice(),
    )
    .map_err(|e| WalletError::InvalidConfig(format!("redb meta insert failed: {e}")))?;

    // Index format marker: new wallets always start on the current format.
    meta.insert(
        META_INDEX_FORMAT_VERSION,
        encode_bincode(&INDEX_FORMAT_VERSION_HMAC)?.as_slice(),
    )
    .map_err(|e| WalletError::InvalidConfig(format!("redb meta insert failed: {e}")))?;

    // Secret AAD format marker: new wallets always start on the current format.
    meta.insert(
        META_AAD_SECRET_VERSION,
        encode_bincode(&AAD_SECRET_VERSION)?.as_slice(),
    )
    .map_err(|e| WalletError::InvalidConfig(format!("redb meta insert failed: {e}")))?;

    // HKDF info scheme marker: new wallets always start on the current format.
    meta.insert(
        META_HKDF_INFO_VERSION,
        encode_bincode(&HKDF_INFO_VERSION)?.as_slice(),
    )
    .map_err(|e| WalletError::InvalidConfig(format!("redb meta insert failed: {e}")))?;

    Ok(())
}

pub(super) fn wallet_tmp_path(path: &Path) -> PathBuf {
    let mut os = path.as_os_str().to_os_string();
    os.push(".tmp");
    PathBuf::from(os)
}

pub(super) fn store_wallet_save_seq(
    meta: &mut redb::Table<'_, &str, &[u8]>,
    save_seq: u64,
) -> WalletResult<()> {
    meta.insert(META_WALLET_SAVE_SEQ, encode_bincode(&save_seq)?.as_slice())
        .map_err(|e| WalletError::InvalidConfig(format!("redb meta insert failed: {e}")))?;
    Ok(())
}

pub(super) fn store_wallet_updated_ms(
    meta: &mut redb::Table<'_, &str, &[u8]>,
    updated_at_ms: u64,
) -> WalletResult<()> {
    meta.insert(
        META_WALLET_UPDATED_AT,
        encode_bincode(&updated_at_ms)?.as_slice(),
    )
    .map_err(|e| WalletError::InvalidConfig(format!("redb meta insert failed: {e}")))?;
    Ok(())
}

pub(super) fn store_rotation_in_progress(
    meta: &mut redb::Table<'_, &str, &[u8]>,
    started_at_ms: u64,
) -> WalletResult<()> {
    meta.insert(
        META_ROTATION_IN_PROGRESS,
        encode_bincode(&started_at_ms)?.as_slice(),
    )
    .map_err(|e| WalletError::InvalidConfig(format!("redb meta insert failed: {e}")))?;
    Ok(())
}

pub(super) fn rotation_in_progress(meta: &redb::ReadOnlyTable<&str, &[u8]>) -> WalletResult<bool> {
    let Some(raw) = meta
        .get(META_ROTATION_IN_PROGRESS)
        .map_err(|e| WalletError::InvalidConfig(format!("redb meta read failed: {e}")))?
    else {
        return Ok(false);
    };

    let _: u64 = decode_bincode(raw.value())
        .map_err(|_| WalletError::InvalidConfig("wallet rotation marker invalid".to_string()))?;

    Ok(true)
}

pub(super) fn clear_rotation_in_progress(
    meta: &mut redb::Table<'_, &str, &[u8]>,
) -> WalletResult<()> {
    meta.remove(META_ROTATION_IN_PROGRESS)
        .map_err(|e| WalletError::InvalidConfig(format!("redb meta remove failed: {e}")))?;
    Ok(())
}

pub(super) fn bump_wallet_write_meta(
    meta: &mut redb::Table<'_, &str, &[u8]>,
    time_provider: &dyn TimeProvider,
) -> WalletResult<u64> {
    let current_save_seq_raw = meta
        .get(META_WALLET_SAVE_SEQ)
        .map_err(|e| WalletError::InvalidConfig(format!("redb meta read failed: {e}")))?
        .ok_or_else(|| WalletError::InvalidConfig("missing meta.wallet.save_seq".to_string()))?
        .value()
        .to_vec();
    let current_save_seq: u64 = decode_bincode(&current_save_seq_raw)?;
    let next_save_seq = current_save_seq.saturating_add(1);

    let current_updated_at_raw = meta
        .get(META_WALLET_UPDATED_AT)
        .map_err(|e| WalletError::InvalidConfig(format!("redb meta read failed: {e}")))?
        .ok_or_else(|| WalletError::InvalidConfig("missing meta.wallet.updated_at".to_string()))?
        .value()
        .to_vec();
    let current_updated_at: u64 = decode_bincode(&current_updated_at_raw)?;

    let now_ms = time_provider.compat_unix_timestamp_millis();
    let next_updated_at = current_updated_at.max(now_ms);

    store_wallet_save_seq(meta, next_save_seq)?;
    store_wallet_updated_ms(meta, next_updated_at)?;
    update_wallet_integrity(meta, next_save_seq)?;

    Ok(next_save_seq)
}

pub(super) const WALLET_META_INVALID: &str = "wallet meta invalid";

const REQUIRED_META_KEYS_OPEN: &[&str] = &[
    META_WALLET_ID,
    META_WALLET_CHAIN,
    META_SCHEMA_VERSION,
    META_WALLET_KDF,
    META_WALLET_INITIALIZED,
];

const REQUIRED_META_KEYS_WRITE: &[&str] = &[
    META_WALLET_CREATED_AT,
    META_WALLET_UPDATED_AT,
    META_WALLET_SAVE_SEQ,
];

pub(crate) const REQUIRED_META_POINTER_KEYS_OPEN: &[&str] = &[
    META_SCAN_STATE_OBJECT_ID,
    META_APP_OBJECT_ID,
    META_CHAIN_OBJECT_ID,
    META_KEYS_OBJECT_ID,
];

pub(super) struct WalletMetaHeader {
    pub(super) wallet_id: PersistWalletId,
    pub(super) schema_version: u32,
    pub(super) kdf_params: KdfParams,
}

pub(super) fn read_wallet_meta_header(
    meta: &redb::ReadOnlyTable<&str, &[u8]>,
    expected_wallet_id: &PersistWalletId,
    identity: &WalletIdentity,
) -> WalletResult<WalletMetaHeader> {
    for key in REQUIRED_META_KEYS_OPEN
        .iter()
        .copied()
        .chain(REQUIRED_META_KEYS_WRITE.iter().copied())
    {
        meta.get(key)
            .map_err(|_| WalletError::InvalidConfig(WALLET_META_INVALID.to_string()))?
            .ok_or_else(|| WalletError::InvalidConfig(WALLET_META_INVALID.to_string()))?;
    }

    let get_bounded = |key: &str| -> WalletResult<Vec<u8>> {
        meta.get(key)
            .map_err(|_| WalletError::InvalidConfig(WALLET_META_INVALID.to_string()))?
            .map(|g| g.value().to_vec())
            .ok_or_else(|| WalletError::InvalidConfig(WALLET_META_INVALID.to_string()))
    };

    // Required object pointers: these are schema-level invariants and must be validated at open.
    // The values are stored as raw bytes (not bincode) and must be canonical 16-byte BE ObjectIds.
    for key in REQUIRED_META_POINTER_KEYS_OPEN.iter().copied() {
        let bytes = get_bounded(key)?;
        if bytes.len() != 16 {
            return Err(WalletError::InvalidConfig(WALLET_META_INVALID.to_string()));
        }
        let _ = object_id_from_be_bytes(&bytes)
            .map_err(|_| WalletError::InvalidConfig(WALLET_META_INVALID.to_string()))?;
    }

    let stored_wallet_id_bytes = get_bounded(META_WALLET_ID)?;
    let stored_wallet_id: PersistWalletId = decode_bincode(stored_wallet_id_bytes.as_slice())
        .map_err(|_| WalletError::InvalidConfig(WALLET_META_INVALID.to_string()))?;
    if &stored_wallet_id != expected_wallet_id {
        return Err(WalletError::InvalidConfig(WALLET_META_INVALID.to_string()));
    }

    // Validate operational meta keys early so failures are deterministic.
    let created_at_bytes = get_bounded(META_WALLET_CREATED_AT)?;
    let created_at: u64 = decode_bincode(created_at_bytes.as_slice())
        .map_err(|_| WalletError::InvalidConfig(WALLET_META_INVALID.to_string()))?;

    let updated_at_bytes = get_bounded(META_WALLET_UPDATED_AT)?;
    let updated_at: u64 = decode_bincode(updated_at_bytes.as_slice())
        .map_err(|_| WalletError::InvalidConfig(WALLET_META_INVALID.to_string()))?;

    if updated_at < created_at {
        return Err(WalletError::InvalidConfig(WALLET_META_INVALID.to_string()));
    }

    let save_seq_bytes = get_bounded(META_WALLET_SAVE_SEQ)?;
    let _save_seq: u64 = decode_bincode(save_seq_bytes.as_slice())
        .map_err(|_| WalletError::InvalidConfig(WALLET_META_INVALID.to_string()))?;

    let stored_chain_bytes = get_bounded(META_WALLET_CHAIN)?;
    let stored_chain: String = decode_bincode(stored_chain_bytes.as_slice())
        .map_err(|_| WalletError::InvalidConfig(WALLET_META_INVALID.to_string()))?;
    if stored_chain != identity.chain {
        return Err(WalletError::WalletChainMismatch {
            expected: identity.chain.clone(),
            actual: stored_chain,
        });
    }

    let stored_network_bytes = get_bounded(META_WALLET_NETWORK)?;
    let stored_network: String = decode_bincode(stored_network_bytes.as_slice())
        .map_err(|_| WalletError::InvalidConfig(WALLET_META_INVALID.to_string()))?;
    if stored_network != identity.network {
        return Err(WalletError::WalletNetworkMismatch {
            expected: identity.network.clone(),
            actual: stored_network,
        });
    }

    let schema_version_bytes = get_bounded(META_SCHEMA_VERSION)?;
    let schema_version: u32 = decode_bincode(schema_version_bytes.as_slice())
        .map_err(|_| WalletError::InvalidConfig(WALLET_META_INVALID.to_string()))?;
    if schema_version > REDB_WALLET_SCHEMA_VERSION {
        return Err(WalletError::UnsupportedVersion(schema_version));
    }

    let kdf_bytes = get_bounded(META_WALLET_KDF)?;
    let kdf_params: KdfParams =
        decode_bincode(kdf_bytes.as_slice()).map_err(|_| WalletError::InvalidPassword)?;
    // Structural proof: persisted KDF params are treated as untrusted input and validated
    // before any expensive KDF computation happens (Argon2 allocations/work are gated).
    match kdf_params.validate_untrusted_persisted() {
        Ok(()) => {}
        Err(z00z_crypto::CryptoError::InvalidParameters { param })
            if param.to_ascii_lowercase().contains("unsupported kdf")
                || param.to_ascii_lowercase().contains("kdf_algo") =>
        {
            return Err(WalletError::UnsupportedKdf(param.to_string()));
        }
        Err(_) => {
            return Err(WalletError::InvalidPassword);
        }
    }

    let initialized_bytes = get_bounded(META_WALLET_INITIALIZED)?;
    let initialized: u8 = decode_bincode(initialized_bytes.as_slice())
        .map_err(|_| WalletError::InvalidConfig("wallet file is not initialized".to_string()))?;
    if initialized != 1 {
        return Err(WalletError::InvalidConfig(
            "wallet file is not initialized".to_string(),
        ));
    }

    Ok(WalletMetaHeader {
        wallet_id: stored_wallet_id,
        schema_version,
        kdf_params,
    })
}
