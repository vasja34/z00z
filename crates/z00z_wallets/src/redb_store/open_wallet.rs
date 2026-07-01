use super::{
    decode_bincode, decode_bincode_bounded, decode_object_id_be, decode_object_record_bounded,
    decode_seed_plaintext_phrase24, decrypt_object_record, decrypt_secret_record,
    decrypt_secret_record_post_unlock, finalize_rotation_marker_on_db, generate_16_bytes,
    is_zstd_magic_bytes, migrate_index_format_if_needed, read_wallet_meta_header, to_hex,
    try_lock_wallet_file, validate_seed_main_record, validate_seed_plaintext_unlock,
    verify_archived_wallet_copy, zstd_decode_bounded_to_writer, Arc, Database, MasterKeyRecord,
    OpenedWallet, Path, PersistWalletId, ReadableDatabase, ReadableTable, SafePassword,
    SecretsRecord, SystemRngProvider, TimeProvider, WalletDerivedKeys, WalletError, WalletIdentity,
    WalletIo, WalletRedbKeyManager, WalletResult, WalletSession, WltBacking, Write,
    AAD_SECRET_VERSION, HKDF_INFO_VERSION, INDEX_FORMAT_VERSION_HMAC, MAX_WLT_DECOMPRESSED_BYTES,
    META_AAD_SECRET_VERSION, META_HKDF_INFO_VERSION, META_INDEX_FORMAT_VERSION, META_TABLE,
    OBJECTS_TABLE, SECRETS_MASTER_KEY, SECRETS_SEED_MAIN, SECRETS_TABLE, WALLET_META_INVALID,
};

struct TmpfsWorkGuard {
    io: Arc<dyn WalletIo>,
    path: std::path::PathBuf,
    armed: bool,
}

impl TmpfsWorkGuard {
    fn new(io: Arc<dyn WalletIo>, path: std::path::PathBuf) -> Self {
        Self {
            io,
            path,
            armed: true,
        }
    }

    fn disarm(&mut self) {
        self.armed = false;
    }
}

impl Drop for TmpfsWorkGuard {
    fn drop(&mut self) {
        if self.armed {
            self.io.remove_file_best_effort(&self.path);
        }
    }
}

pub(crate) fn open_wlt_with_deps(
    path: &Path,
    wallet_id: &PersistWalletId,
    password: &SafePassword,
    identity: &WalletIdentity,
    time_provider: Arc<dyn TimeProvider>,
    io: Arc<dyn WalletIo>,
) -> WalletResult<WalletSession> {
    let file_lock = try_lock_wallet_file(path, time_provider.as_ref(), io.clone())?;

    // Detect zstd-by-content `.wlt` by checking magic bytes first
    // We need to read just the first 4 bytes to check magic
    let mut header_buf = [0u8; 4];
    {
        use std::io::Read;
        let mut file = std::fs::File::open(path)
            .map_err(|e| WalletError::InvalidConfig(format!("failed to open .wlt file: {e}")))?;
        file.read_exact(&mut header_buf)
            .map_err(|e| WalletError::InvalidConfig(format!("failed to read .wlt header: {e}")))?;
    }

    if !is_zstd_magic_bytes(&header_buf) {
        return Err(WalletError::InvalidConfig(
            "wallet .wlt must be zstd-compressed (development mode)".to_string(),
        ));
    }

    // Check /dev/shm availability
    let shm_dir = Path::new("/dev/shm");
    if !io.path_exists(shm_dir)? {
        return Err(WalletError::InvalidConfig(
            "/dev/shm is required to open zstd .wlt without writing plaintext to disk".to_string(),
        ));
    }

    // Generate unique work file path
    let mut tmp_rng = SystemRngProvider.rng();
    let tmp_id = generate_16_bytes(&mut tmp_rng);
    let file_name = path
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("wallet.wlt");
    let work_name = format!("{file_name}.work.{}", to_hex(&tmp_id));
    let work_path = shm_dir.join(work_name);

    io.remove_file_best_effort(&work_path);
    let mut work_guard = TmpfsWorkGuard::new(io.clone(), work_path.clone());

    // Stream-decompress .wlt directly to tmpfs work file.
    // This avoids allocating full Vec<u8> for decompressed data.
    // If decode fails, we must cleanup the partially written plaintext work file.
    let open_result = (|| -> WalletResult<()> {
        use std::io::{BufReader, BufWriter};

        let input_file = std::fs::File::open(path).map_err(|e| {
            WalletError::InvalidConfig(format!("failed to open .wlt for reading: {e}"))
        })?;
        let mut reader = BufReader::new(input_file);

        let output_file = std::fs::File::create(&work_path)
            .map_err(|e| WalletError::InvalidConfig(format!("failed to create work file: {e}")))?;
        // Permission hardening must go through the I/O boundary.
        io.set_private_file_permissions(&work_path)?;
        let mut writer = BufWriter::new(output_file);

        zstd_decode_bounded_to_writer(&mut reader, &mut writer, MAX_WLT_DECOMPRESSED_BYTES)
            .map_err(|e| WalletError::InvalidConfig(format!("wallet zstd payload invalid: {e}")))?;

        writer
            .flush()
            .map_err(|e| WalletError::InvalidConfig(format!("failed to flush work file: {e}")))?;

        Ok(())
    })();

    open_result?;

    let open_path = work_path.clone();
    let backing = WltBacking::ZstdTmpfs {
        original_path: path.to_path_buf(),
        work_path,
    };

    let db = Database::open(&open_path).map_err(|e| {
        let msg = e.to_string();
        let msg_lower = msg.to_ascii_lowercase();
        if msg_lower.contains("already open") || msg_lower.contains("cannot acquire lock") {
            WalletError::WalletInUse
        } else {
            WalletError::InvalidConfig(format!("redb open failed: {msg}"))
        }
    })?;

    let read_txn = db
        .begin_read()
        .map_err(|e| WalletError::InvalidConfig(format!("redb begin_read failed: {e}")))?;

    let meta = read_txn
        .open_table(META_TABLE)
        .map_err(|e| WalletError::InvalidConfig(format!("redb open meta failed: {e}")))?;
    let secrets = read_txn
        .open_table(SECRETS_TABLE)
        .map_err(|e| WalletError::InvalidConfig(format!("redb open secrets failed: {e}")))?;

    let header = read_wallet_meta_header(&meta, wallet_id, identity)?;
    let wallet_id = header.wallet_id;
    let schema_version = header.schema_version;
    let kdf_params = header.kdf_params;

    let stored_index_format_version = match meta
        .get(META_INDEX_FORMAT_VERSION)
        .map_err(|_| WalletError::InvalidConfig(WALLET_META_INVALID.to_string()))?
    {
        Some(g) => decode_bincode::<u32>(g.value())
            .map_err(|_| WalletError::InvalidConfig(WALLET_META_INVALID.to_string()))?,
        None => INDEX_FORMAT_VERSION_HMAC,
    };

    let stored_aad_secret_version = match meta
        .get(META_AAD_SECRET_VERSION)
        .map_err(|_| WalletError::InvalidConfig(WALLET_META_INVALID.to_string()))?
    {
        Some(g) => decode_bincode::<u32>(g.value())
            .map_err(|_| WalletError::InvalidConfig(WALLET_META_INVALID.to_string()))?,
        None => {
            return Err(WalletError::InvalidConfig(
                "missing secret AAD format version".to_string(),
            ));
        }
    };
    if stored_aad_secret_version != AAD_SECRET_VERSION {
        return Err(WalletError::InvalidConfig(
            "unsupported secret AAD format version".to_string(),
        ));
    }

    let stored_hkdf_info_version = match meta
        .get(META_HKDF_INFO_VERSION)
        .map_err(|_| WalletError::InvalidConfig(WALLET_META_INVALID.to_string()))?
    {
        Some(g) => decode_bincode::<u32>(g.value())
            .map_err(|_| WalletError::InvalidConfig(WALLET_META_INVALID.to_string()))?,
        None => {
            return Err(WalletError::InvalidConfig(
                "missing hkdf info version".to_string(),
            ));
        }
    };
    if stored_hkdf_info_version != HKDF_INFO_VERSION {
        return Err(WalletError::InvalidConfig(
            "unsupported hkdf info version".to_string(),
        ));
    }

    let master_key_record_bytes = secrets
        .get(SECRETS_MASTER_KEY)
        .map_err(|e| WalletError::InvalidConfig(format!("redb secrets read failed: {e}")))?
        .ok_or_else(|| WalletError::InvalidConfig("missing secrets.master_key".to_string()))?;
    let mut master_key_record: MasterKeyRecord = decode_bincode(master_key_record_bytes.value())
        .map_err(|_| WalletError::InvalidPassword)?;

    if let Some(record_kdf) = master_key_record.kdf_params.as_ref() {
        if record_kdf != &kdf_params {
            return Err(WalletError::InvalidConfig(
                "kdf params mismatch".to_string(),
            ));
        }
    } else {
        master_key_record.kdf_params = Some(kdf_params.clone());
    }
    let active_kdf_params = master_key_record
        .kdf_params
        .clone()
        .unwrap_or_else(|| kdf_params.clone());

    let km = WalletRedbKeyManager::new();

    let master_key_hidden = km
        .unwrap_master_key(&wallet_id, password, &active_kdf_params, &master_key_record)
        .map_err(|err| match err {
            crate::key::WalletRedbKeyManagerError::InvalidParameters(message) => {
                WalletError::InvalidConfig(format!("unsupported wallet kdf parameters: {message}"))
            }
            _ => WalletError::InvalidPassword,
        })?;

    // Strict: a valid wallet must contain the main seed secret record.
    // Any missing/corrupted/unsupported seed record must fail closed as a bounded auth failure
    // during open/unlock.
    let seed_main_raw = secrets
        .get(SECRETS_SEED_MAIN)
        .map_err(|e| WalletError::InvalidConfig(format!("redb secrets read failed: {e}")))?
        .ok_or(WalletError::InvalidPassword)?
        .value()
        .to_vec();
    let seed_record: SecretsRecord =
        decode_bincode_bounded(&seed_main_raw, WalletError::InvalidPassword)?;
    validate_seed_main_record(&seed_record, WalletError::InvalidPassword)?;
    let mut seed_plaintext = decrypt_secret_record(
        &wallet_id,
        SECRETS_SEED_MAIN,
        master_key_hidden.reveal(),
        &seed_record,
    )?;
    validate_seed_plaintext_unlock(&seed_plaintext)?;

    // Decode the seed phrase from the decrypted seed secret and derive BIP-39 seed bytes.
    // Any failure is treated as a bounded auth failure.
    let seed_phrase = decode_seed_plaintext_phrase24(&seed_plaintext)?;
    let seed_bip39_hidden = seed_phrase
        .to_bip39_seed("")
        .map_err(|_| WalletError::InvalidPassword)?;
    drop(seed_phrase);

    seed_plaintext.wipe();

    // RedB does not allow a write transaction while a read transaction is alive.
    // Drop the initial read handles before any migration.
    drop(secrets);
    drop(meta);
    drop(read_txn);

    migrate_index_format_if_needed(
        &db,
        io.as_ref(),
        path,
        &open_path,
        stored_index_format_version,
    )?;

    // Fail closed on any tampering/corruption of object records.
    let read_txn = db
        .begin_read()
        .map_err(|e| WalletError::InvalidConfig(format!("redb begin_read failed: {e}")))?;

    let meta = read_txn
        .open_table(META_TABLE)
        .map_err(|e| WalletError::InvalidConfig(format!("redb open meta failed: {e}")))?;

    let hkdf_info_version = match meta
        .get(META_HKDF_INFO_VERSION)
        .map_err(|_| WalletError::InvalidConfig(WALLET_META_INVALID.to_string()))?
    {
        Some(g) => decode_bincode::<u32>(g.value())
            .map_err(|_| WalletError::InvalidConfig(WALLET_META_INVALID.to_string()))?,
        None => {
            return Err(WalletError::InvalidConfig(
                "missing hkdf info version".to_string(),
            ));
        }
    };
    if hkdf_info_version != HKDF_INFO_VERSION {
        return Err(WalletError::InvalidConfig(
            "unsupported hkdf info version".to_string(),
        ));
    }

    let rotation_in_progress = super::meta::rotation_in_progress(&meta)?;

    let derived_keys = km
        .derive_wallet_keys(&master_key_hidden)
        .map_err(|e| WalletError::InvalidConfig(format!("key derivation failed: {e}")))?;

    let objects = read_txn
        .open_table(OBJECTS_TABLE)
        .map_err(|e| WalletError::InvalidConfig(format!("redb open objects failed: {e}")))?;
    validate_objects_on_open(&wallet_id, &derived_keys, &objects)?;

    if rotation_in_progress {
        let secrets = read_txn
            .open_table(SECRETS_TABLE)
            .map_err(|e| WalletError::InvalidConfig(format!("redb open secrets failed: {e}")))?;

        for row in secrets
            .iter()
            .map_err(|e| WalletError::InvalidConfig(format!("redb secrets iter failed: {e}")))?
        {
            let (name, value) = row.map_err(|e| {
                WalletError::InvalidConfig(format!("redb secrets read failed: {e}"))
            })?;
            let secret_name = name.value();
            if secret_name == SECRETS_MASTER_KEY {
                continue;
            }

            let record: SecretsRecord = decode_bincode_bounded(
                value.value(),
                WalletError::InvalidConfig("wallet secret invalid".to_string()),
            )?;
            let mut plaintext = decrypt_secret_record_post_unlock(
                &wallet_id,
                secret_name,
                master_key_hidden.reveal(),
                &record,
            )?;
            plaintext.wipe();
        }
    }

    if rotation_in_progress {
        drop(objects);
        drop(meta);
        drop(read_txn);
        verify_archived_wallet_copy(
            path,
            &wallet_id,
            &active_kdf_params,
            password,
            identity,
            master_key_hidden.reveal(),
            &derived_keys,
            false,
            Arc::clone(&time_provider),
            Arc::clone(&io),
        )?;
        finalize_rotation_marker_on_db(&db, &backing, io.as_ref(), time_provider.as_ref())?;
    }

    #[cfg(all(feature = "os_hardening", not(target_arch = "wasm32")))]
    let master_key = z00z_utils::os_hardening::OwnedLockedBytes::new_best_effort_with(|slot| {
        slot.copy_from_slice(master_key_hidden.reveal());
    });

    #[cfg(all(feature = "os_hardening", not(target_arch = "wasm32")))]
    let seed_bip39 = z00z_utils::os_hardening::OwnedLockedBytes::new_best_effort_with(|slot| {
        slot.copy_from_slice(seed_bip39_hidden.reveal());
    });

    #[cfg(all(feature = "os_hardening", not(target_arch = "wasm32")))]
    drop(master_key_hidden);

    #[cfg(all(feature = "os_hardening", not(target_arch = "wasm32")))]
    drop(seed_bip39_hidden);

    #[cfg(not(all(feature = "os_hardening", not(target_arch = "wasm32"))))]
    let master_key = master_key_hidden;

    #[cfg(not(all(feature = "os_hardening", not(target_arch = "wasm32"))))]
    let seed_bip39 = seed_bip39_hidden;

    let opened = OpenedWallet {
        wallet_id,
        schema_version,
        kdf_params: active_kdf_params,
        master_key,
        derived_keys,
        seed_bip39,
        _file_lock: file_lock,
    };

    work_guard.disarm();

    Ok(WalletSession {
        db,
        opened,
        time_provider,
        io,
        backing,
    })
}

pub(crate) fn validate_objects_on_open(
    wallet_id: &PersistWalletId,
    derived: &WalletDerivedKeys,
    objects: &redb::ReadOnlyTable<&[u8], &[u8]>,
) -> WalletResult<()> {
    use zeroize::Zeroize;

    let it = objects
        .iter()
        .map_err(|e| WalletError::InvalidConfig(format!("redb objects iter failed: {e}")))?;

    for row in it {
        let (key, value) =
            row.map_err(|e| WalletError::InvalidConfig(format!("redb objects read failed: {e}")))?;

        let object_id = decode_object_id_be(key.value())?;
        let record = decode_object_record_bounded(value.value())?;

        // Decrypt and validate the record, then scrub the plaintext payload.
        let mut payload = decrypt_object_record(wallet_id, derived, object_id, &record)?;
        payload.data.zeroize();
    }

    Ok(())
}
