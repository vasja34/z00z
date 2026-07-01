use super::{
    decode_bincode, generate_16_bytes, is_zstd_magic_bytes, to_hex, try_lock_wallet_file,
    zstd_decode_bounded_to_writer, Arc, BufWriter, Database, Path, PersistWalletDiscovery,
    PersistWalletId, ReadableDatabase, SystemRngProvider, SystemTimeProvider, TimeProvider,
    WalletError, WalletIo, WalletResult, Write, Z00ZWalletIo, MAX_WLT_DECOMPRESSED_BYTES,
    META_SCHEMA_VERSION, META_TABLE, META_WALLET_CHAIN, META_WALLET_ID, META_WALLET_INITIALIZED,
    META_WALLET_NETWORK, REDB_WALLET_SCHEMA_VERSION, WALLET_META_INVALID,
};

pub fn discover_wallet_store(path: &Path) -> WalletResult<PersistWalletDiscovery> {
    let time_provider: Arc<dyn TimeProvider> = Arc::new(SystemTimeProvider);
    let io = Arc::new(Z00ZWalletIo);

    discover_wlt_with_deps(path, time_provider.as_ref(), io)
}

pub(crate) fn discover_wlt_with_deps(
    path: &Path,
    time_provider: &dyn TimeProvider,
    io: Arc<dyn WalletIo>,
) -> WalletResult<PersistWalletDiscovery> {
    let _file_lock = try_lock_wallet_file(path, time_provider, io.clone())?;

    // Check zstd magic bytes by streaming just the header first (no full read).
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

    // Ensure /dev/shm exists for tmpfs work file.
    let shm_dir = Path::new("/dev/shm");
    if !io.path_exists(shm_dir)? {
        return Err(WalletError::InvalidConfig(
            "/dev/shm is required to open zstd .wlt without writing plaintext to disk".to_string(),
        ));
    }

    // Generate unique work file path.
    let mut tmp_rng = SystemRngProvider.rng();
    let tmp_id = generate_16_bytes(&mut tmp_rng);
    let file_name = path
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("wallet.wlt");
    let work_name = format!("{file_name}.work.{}", to_hex(&tmp_id));
    let work_path = shm_dir.join(work_name);

    // Best-effort cleanup of stale work file.
    io.remove_file_best_effort(&work_path);

    // Stream-decompress .wlt directly to tmpfs work file (no full-buffer Vec).
    // Ensure work file is hardened (0o600) and cleaned up on failure.
    let open_result = (|| -> WalletResult<()> {
        use std::io::BufReader;

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

    // On failure, ensure no plaintext work file remains.
    if open_result.is_err() {
        io.remove_file_best_effort(&work_path);
        // Convert inner () error to PersistWalletDiscovery error type.
        return open_result.map(|_| PersistWalletDiscovery {
            wallet_id: PersistWalletId::default(),
            network: String::new(),
            chain: String::new(),
        });
    }

    // Open RedB from the tmpfs work file.
    let db = Database::open(&work_path).map_err(|e| {
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

    let get_bounded = |key: &str| -> WalletResult<Vec<u8>> {
        meta.get(key)
            .map_err(|_| WalletError::InvalidConfig(WALLET_META_INVALID.to_string()))?
            .map(|g| g.value().to_vec())
            .ok_or_else(|| WalletError::InvalidConfig(WALLET_META_INVALID.to_string()))
    };

    // Keep discovery strict and deterministic (bounds + typed decode).
    let stored_wallet_id_bytes = get_bounded(META_WALLET_ID)?;
    let wallet_id: PersistWalletId = decode_bincode(stored_wallet_id_bytes.as_slice())
        .map_err(|_| WalletError::InvalidConfig(WALLET_META_INVALID.to_string()))?;

    let stored_chain_bytes = get_bounded(META_WALLET_CHAIN)?;
    let chain: String = decode_bincode(stored_chain_bytes.as_slice())
        .map_err(|_| WalletError::InvalidConfig(WALLET_META_INVALID.to_string()))?;

    let stored_network_bytes = get_bounded(META_WALLET_NETWORK)?;
    let network: String = decode_bincode(stored_network_bytes.as_slice())
        .map_err(|_| WalletError::InvalidConfig(WALLET_META_INVALID.to_string()))?;

    let schema_version_bytes = get_bounded(META_SCHEMA_VERSION)?;
    let schema_version: u32 = decode_bincode(schema_version_bytes.as_slice())
        .map_err(|_| WalletError::InvalidConfig(WALLET_META_INVALID.to_string()))?;
    if schema_version > REDB_WALLET_SCHEMA_VERSION {
        return Err(WalletError::UnsupportedVersion(schema_version));
    }

    let initialized_bytes = get_bounded(META_WALLET_INITIALIZED)?;
    let initialized: u8 = decode_bincode(initialized_bytes.as_slice())
        .map_err(|_| WalletError::InvalidConfig("wallet file is not initialized".to_string()))?;
    if initialized != 1 {
        return Err(WalletError::InvalidConfig(
            "wallet file is not initialized".to_string(),
        ));
    }

    drop(meta);
    drop(read_txn);
    drop(db);
    io.remove_file_best_effort(&work_path);

    Ok(PersistWalletDiscovery {
        wallet_id,
        network,
        chain,
    })
}
