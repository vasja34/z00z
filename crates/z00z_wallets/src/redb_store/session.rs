use super::{
    bump_wallet_write_meta, commit_redb_write_txn_flush, decode_bincode, decode_bincode_bounded,
    decode_object_id_be, decode_object_record_bounded, decrypt_object_record,
    decrypt_secret_record_post_unlock, encode_bincode, encode_object_id_be, encrypt_object_record,
    encrypt_secret_record, generate_16_bytes, open_wlt_with_deps, to_hex,
    write_object_with_index_key, zstd_encode_to_writer, Arc, Database, HashSet, Hidden,
    IndexManifestEntry, IndexTable, IndexUpdate, KdfParams, Lazy, MasterKeyRecord, Mutex,
    ObjectKindId, OpenOptions, OwnedAssetPayload, Path, PathBuf, PersistWalletId, ReadableDatabase,
    ReadableTable, RngCoreExt, SafePassword, SecretsRecord, SecureRngProvider, SystemRngProvider,
    SystemTimeProvider, TableDefinition, TimeProvider, WalletDerivedKeys, WalletError,
    WalletIdentity, WalletIo, WalletRedbKeyManager, WalletResult, WltBacking, Z00ZWalletIo,
    INDEX_MANIFEST_TABLE, META_TABLE, META_WALLET_CHAIN, META_WALLET_NETWORK, OBJECTS_TABLE,
    PAYLOAD_VERSION_OWNED_ASSET, SECRETS_MASTER_KEY, SECRETS_TABLE, WLT_ZSTD_LEVEL,
};

#[cfg(test)]
use std::cell::Cell;

static OPEN_WALLET_LOCKS: Lazy<Mutex<HashSet<PathBuf>>> = Lazy::new(|| Mutex::new(HashSet::new()));
const WALLET_LOCK_OPEN_RETRIES: u32 = 20;
const WALLET_LOCK_OPEN_RETRY_MS: u64 = 25;

type IndexStateRow = (IndexTable, Vec<u8>, Vec<u8>);
type ObjectIndexStateSnapshot = (Option<Vec<u8>>, Vec<IndexStateRow>);

#[cfg(test)]
thread_local! {
    static ROTATE_MASTER_FP_COMMIT: Cell<bool> = const { Cell::new(false) };
}

#[cfg(test)]
pub(crate) fn set_rotate_master_fp_commit(enabled: bool) {
    ROTATE_MASTER_FP_COMMIT.with(|flag| flag.set(enabled));
}

#[cfg(test)]
fn take_rotate_master_fp_commit() -> bool {
    ROTATE_MASTER_FP_COMMIT.with(|flag| {
        let enabled = flag.get();
        if enabled {
            flag.set(false);
        }
        enabled
    })
}

pub(crate) fn is_lock_held_local(wallet_path: &Path) -> bool {
    let Ok(set) = OPEN_WALLET_LOCKS.lock() else {
        return true;
    };

    set.contains(wallet_path)
}

pub(super) struct WalletFileLockInner {
    wallet_path: PathBuf,
    lock_path: PathBuf,
    lock_file: std::fs::File,
    io: Arc<dyn WalletIo>,
}

impl std::fmt::Debug for WalletFileLockInner {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("WalletFileLockInner")
            .field("wallet_path", &self.wallet_path)
            .field("lock_path", &self.lock_path)
            .finish_non_exhaustive()
    }
}

impl Drop for WalletFileLockInner {
    fn drop(&mut self) {
        let _ = self.lock_file.unlock();

        self.io.remove_file_best_effort(&self.lock_path);

        if let Ok(mut set) = OPEN_WALLET_LOCKS.lock() {
            set.remove(&self.wallet_path);
        }
    }
}

pub(crate) fn flush_work_file_to_wallet(
    io: &dyn WalletIo,
    original_path: &Path,
    work_path: &Path,
) -> WalletResult<()> {
    let work_file = std::fs::File::open(work_path)
        .map_err(|e| WalletError::InvalidConfig(format!("failed to open work file: {e}")))?;

    io.atomic_write_file_streaming(original_path, &mut |mut out| {
        zstd_encode_to_writer(&mut &work_file, &mut out, WLT_ZSTD_LEVEL)
            .map_err(std::io::Error::other)
    })?;

    Ok(())
}

fn open_wallet_lock_file(lock_path: &Path) -> Result<std::fs::File, std::io::Error> {
    let mut last_error = None;

    for attempt in 0..=WALLET_LOCK_OPEN_RETRIES {
        match OpenOptions::new()
            .create(true)
            .truncate(false)
            .read(true)
            .write(true)
            .open(lock_path)
        {
            Ok(file) => return Ok(file),
            Err(err) if attempt < WALLET_LOCK_OPEN_RETRIES => {
                last_error = Some(err);
                std::thread::sleep(std::time::Duration::from_millis(WALLET_LOCK_OPEN_RETRY_MS));
            }
            Err(err) => return Err(err),
        }
    }

    Err(last_error.unwrap_or_else(|| std::io::Error::other("wallet lock open retry exhausted")))
}

pub(super) fn try_lock_wallet_file(
    path: &Path,
    time_provider: &dyn TimeProvider,
    io: Arc<dyn WalletIo>,
) -> WalletResult<Arc<WalletFileLockInner>> {
    use fs2::FileExt as _;
    use std::io::{Seek as _, SeekFrom, Write as _};

    let wallet_path = path.to_path_buf();

    {
        let mut set = OPEN_WALLET_LOCKS
            .lock()
            .map_err(|_| WalletError::WalletInUse)?;

        if set.contains(&wallet_path) {
            return Err(WalletError::WalletInUse);
        }
        set.insert(wallet_path.clone());
    }

    let lock_path = {
        let mut os = wallet_path.as_os_str().to_os_string();
        os.push(".lock");
        PathBuf::from(os)
    };

    let mut lock_file = match open_wallet_lock_file(&lock_path) {
        Ok(file) => file,
        Err(err) => {
            let mut set = OPEN_WALLET_LOCKS
                .lock()
                .map_err(|_| WalletError::WalletInUse)?;
            set.remove(&wallet_path);
            return Err(WalletError::Io(format!("wallet lock open failed: {err}")));
        }
    };

    if lock_file.try_lock_exclusive().is_err() {
        let mut set = OPEN_WALLET_LOCKS
            .lock()
            .map_err(|_| WalletError::WalletInUse)?;
        set.remove(&wallet_path);
        return Err(WalletError::WalletInUse);
    }

    let stamp = format!("{}\n", time_provider.compat_unix_timestamp_millis());
    let _ = lock_file.set_len(0);
    let _ = lock_file.seek(SeekFrom::Start(0));
    let _ = lock_file.write_all(stamp.as_bytes());
    let _ = lock_file.flush();

    let _ = io.set_private_file_permissions(&lock_path);

    Ok(Arc::new(WalletFileLockInner {
        wallet_path,
        lock_path,
        lock_file,
        io,
    }))
}

/// Result of opening (unlocking) a `.wlt` RedB wallet.
///
/// This keeps the decrypted `master_key` protected and provides derived keys
/// for subsequent decrypt/encrypt operations.
#[derive(Debug)]
pub struct OpenedWallet {
    pub wallet_id: PersistWalletId,
    pub schema_version: u32,
    pub kdf_params: KdfParams,

    #[cfg(all(feature = "os_hardening", not(target_arch = "wasm32")))]
    pub master_key: z00z_utils::os_hardening::OwnedLockedBytes<32>,
    #[cfg(not(all(feature = "os_hardening", not(target_arch = "wasm32"))))]
    pub master_key: Hidden<[u8; 32]>,
    pub derived_keys: WalletDerivedKeys,

    #[cfg(all(feature = "os_hardening", not(target_arch = "wasm32")))]
    pub seed_bip39: z00z_utils::os_hardening::OwnedLockedBytes<64>,
    #[cfg(not(all(feature = "os_hardening", not(target_arch = "wasm32"))))]
    pub seed_bip39: Hidden<[u8; 64]>,

    pub(super) _file_lock: Arc<WalletFileLockInner>,
}

/// Lock-guarded session handle for a `.wlt` RedB wallet.
///
/// 📌 Contract:
/// - The wallet `.lock` is held for the lifetime of this session.
/// - All `.wlt` reads/writes must go through this handle (non-bypassable by construction).
pub struct WalletSession {
    pub(super) db: Database,
    pub(super) opened: OpenedWallet,
    pub(super) time_provider: Arc<dyn TimeProvider>,
    pub(super) io: Arc<dyn WalletIo>,
    pub(super) backing: WltBacking,
}

struct RotationStateSnapshot {
    secrets_rows: Vec<(String, Vec<u8>)>,
    object_rows: Vec<(u128, Vec<u8>)>,
    index_manifest_rows: Vec<(u128, Vec<u8>)>,
    index_rows: Vec<(IndexTable, Vec<u8>, Vec<u8>)>,
}

impl std::fmt::Debug for WalletSession {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("WalletSession")
            .field("db", &self.db)
            .field("opened", &self.opened)
            .field("time_provider", &"TimeProvider")
            .finish()
    }
}

impl WalletSession {
    pub fn opened(&self) -> &OpenedWallet {
        &self.opened
    }

    pub(super) fn flush_if_zstd(&self) -> WalletResult<()> {
        let WltBacking::ZstdTmpfs {
            original_path,
            work_path,
        } = &self.backing;

        flush_work_file_to_wallet(self.io.as_ref(), original_path, work_path)
    }

    fn snapshot_rotation_state(&self) -> WalletResult<RotationStateSnapshot> {
        let read_txn = self
            .db
            .begin_read()
            .map_err(|e| WalletError::InvalidConfig(format!("redb begin_read failed: {e}")))?;

        let meta = read_txn
            .open_table(META_TABLE)
            .map_err(|e| WalletError::InvalidConfig(format!("redb open meta failed: {e}")))?;
        if super::meta::rotation_in_progress(&meta)? {
            return Err(WalletError::InvalidConfig(
                "wallet rotation already in progress".to_string(),
            ));
        }

        let secrets = read_txn
            .open_table(SECRETS_TABLE)
            .map_err(|e| WalletError::InvalidConfig(format!("redb open secrets failed: {e}")))?;
        let mut secrets_rows = Vec::new();
        for row in secrets
            .iter()
            .map_err(|e| WalletError::InvalidConfig(format!("redb secrets iter failed: {e}")))?
        {
            let (name, value) = row.map_err(|e| {
                WalletError::InvalidConfig(format!("redb secrets read failed: {e}"))
            })?;
            secrets_rows.push((name.value().to_string(), value.value().to_vec()));
        }

        let objects = read_txn
            .open_table(OBJECTS_TABLE)
            .map_err(|e| WalletError::InvalidConfig(format!("redb open objects failed: {e}")))?;
        let mut object_rows = Vec::new();
        let mut index_manifest_rows = Vec::new();
        let mut index_rows = Vec::new();
        for row in objects
            .iter()
            .map_err(|e| WalletError::InvalidConfig(format!("redb objects iter failed: {e}")))?
        {
            let (key, value) = row.map_err(|e| {
                WalletError::InvalidConfig(format!("redb objects read failed: {e}"))
            })?;
            let object_id = decode_object_id_be(key.value())?;
            object_rows.push((object_id, value.value().to_vec()));

            let (manifest_row, object_index_rows) =
                Self::snapshot_object_index_state(&read_txn, object_id)?;
            if let Some(raw_manifest) = manifest_row {
                index_manifest_rows.push((object_id, raw_manifest));
            }
            index_rows.extend(object_index_rows);
        }

        Ok(RotationStateSnapshot {
            secrets_rows,
            object_rows,
            index_manifest_rows,
            index_rows,
        })
    }

    fn snapshot_object_index_state(
        read_txn: &redb::ReadTransaction,
        object_id: u128,
    ) -> WalletResult<ObjectIndexStateSnapshot> {
        let manifest = match read_txn.open_table(INDEX_MANIFEST_TABLE) {
            Ok(table) => table,
            Err(redb::TableError::TableDoesNotExist(_)) => return Ok((None, Vec::new())),
            Err(e) => {
                return Err(WalletError::InvalidConfig(format!(
                    "redb open index_manifest failed: {e}"
                )));
            }
        };
        let manifest_key = encode_object_id_be(object_id);
        let Some(raw_manifest) = manifest.get(manifest_key.as_slice()).map_err(|e| {
            WalletError::InvalidConfig(format!("redb index_manifest read failed: {e}"))
        })?
        else {
            return Ok((None, Vec::new()));
        };

        let raw_manifest_bytes = raw_manifest.value().to_vec();
        let entries: Vec<IndexManifestEntry> = decode_bincode(&raw_manifest_bytes)?;
        let mut index_rows = Vec::with_capacity(entries.len());

        for entry in entries {
            let def: TableDefinition<&[u8], &[u8]> = TableDefinition::new(entry.table.store_name());
            let table = read_txn.open_table(def).map_err(|e| {
                WalletError::InvalidConfig(format!(
                    "redb open {} failed: {e}",
                    entry.table.store_name()
                ))
            })?;
            let value = table
                .get(entry.key.as_slice())
                .map_err(|e| WalletError::InvalidConfig(format!("redb index read failed: {e}")))?
                .ok_or_else(|| {
                    WalletError::InvalidConfig(
                        "index manifest row missing from index table".to_string(),
                    )
                })?;
            index_rows.push((entry.table, entry.key, value.value().to_vec()));
        }

        Ok((Some(raw_manifest_bytes), index_rows))
    }

    fn clear_object_index_state(
        write_txn: &redb::WriteTransaction,
        object_id: u128,
    ) -> WalletResult<()> {
        let mut manifest = match write_txn.open_table(INDEX_MANIFEST_TABLE) {
            Ok(table) => table,
            Err(redb::TableError::TableDoesNotExist(_)) => return Ok(()),
            Err(e) => {
                return Err(WalletError::InvalidConfig(format!(
                    "redb open index_manifest failed: {e}"
                )));
            }
        };
        let manifest_key = encode_object_id_be(object_id);
        let Some(raw_manifest) = manifest.get(manifest_key.as_slice()).map_err(|e| {
            WalletError::InvalidConfig(format!("redb index_manifest read failed: {e}"))
        })?
        else {
            return Ok(());
        };
        let raw_manifest_bytes = raw_manifest.value().to_vec();
        drop(raw_manifest);

        let entries: Vec<IndexManifestEntry> = decode_bincode(&raw_manifest_bytes)?;
        for entry in entries {
            let def: TableDefinition<&[u8], &[u8]> = TableDefinition::new(entry.table.store_name());
            let mut table = write_txn.open_table(def).map_err(|e| {
                WalletError::InvalidConfig(format!(
                    "redb open {} failed: {e}",
                    entry.table.store_name()
                ))
            })?;
            table.remove(entry.key.as_slice()).map_err(|e| {
                WalletError::InvalidConfig(format!("redb index remove failed: {e}"))
            })?;
        }

        manifest.remove(manifest_key.as_slice()).map_err(|e| {
            WalletError::InvalidConfig(format!("redb index_manifest remove failed: {e}"))
        })?;
        Ok(())
    }

    fn restore_snapshot_index_state(
        write_txn: &redb::WriteTransaction,
        snapshot: &RotationStateSnapshot,
    ) -> WalletResult<()> {
        for (table_kind, key, value) in &snapshot.index_rows {
            let def: TableDefinition<&[u8], &[u8]> = TableDefinition::new(table_kind.store_name());
            let mut table = write_txn.open_table(def).map_err(|e| {
                WalletError::InvalidConfig(format!(
                    "redb open {} failed: {e}",
                    table_kind.store_name()
                ))
            })?;
            table
                .insert(key.as_slice(), value.as_slice())
                .map_err(|e| {
                    WalletError::InvalidConfig(format!("redb index insert failed: {e}"))
                })?;
        }

        let mut manifest = write_txn.open_table(INDEX_MANIFEST_TABLE).map_err(|e| {
            WalletError::InvalidConfig(format!("redb open index_manifest failed: {e}"))
        })?;
        for (object_id, raw_manifest) in &snapshot.index_manifest_rows {
            let manifest_key = encode_object_id_be(*object_id);
            manifest
                .insert(manifest_key.as_slice(), raw_manifest.as_slice())
                .map_err(|e| {
                    WalletError::InvalidConfig(format!("redb index_manifest insert failed: {e}"))
                })?;
        }

        Ok(())
    }

    fn rotation_object_index_updates(
        object_id: u128,
        payload_version: u16,
        kind_id: u8,
        payload_bytes: &[u8],
    ) -> WalletResult<Vec<IndexUpdate>> {
        if kind_id == ObjectKindId::OwnedAsset as u8
            && payload_version == PAYLOAD_VERSION_OWNED_ASSET
        {
            let payload: OwnedAssetPayload = decode_bincode(payload_bytes)?;
            return super::owned_assets::owned_asset_index_updates(&payload, Some(object_id));
        }

        let owned_object_updates = super::owned_objects::rotation_owned_object_index_updates(
            object_id,
            payload_version,
            kind_id,
            payload_bytes,
        )?;
        if !owned_object_updates.is_empty() {
            return Ok(owned_object_updates);
        }

        Ok(Vec::new())
    }

    fn verify_object_index_state(
        read_txn: &redb::ReadTransaction,
        index_key: &[u8; 32],
        object_id: u128,
        expected_updates: &[IndexUpdate],
    ) -> WalletResult<()> {
        let manifest = match read_txn.open_table(INDEX_MANIFEST_TABLE) {
            Ok(table) => table,
            Err(redb::TableError::TableDoesNotExist(_)) if expected_updates.is_empty() => {
                return Ok(());
            }
            Err(redb::TableError::TableDoesNotExist(_)) => {
                return Err(WalletError::InvalidConfig(
                    "missing index manifest row for indexed object".to_string(),
                ));
            }
            Err(e) => {
                return Err(WalletError::InvalidConfig(format!(
                    "redb open index_manifest failed: {e}"
                )));
            }
        };
        let manifest_key = encode_object_id_be(object_id);
        let raw_manifest = manifest.get(manifest_key.as_slice()).map_err(|e| {
            WalletError::InvalidConfig(format!("redb index_manifest read failed: {e}"))
        })?;

        if expected_updates.is_empty() {
            if raw_manifest.is_some() {
                return Err(WalletError::InvalidConfig(
                    "unexpected index manifest row for unindexed object".to_string(),
                ));
            }
            return Ok(());
        }

        let raw_manifest = raw_manifest.ok_or_else(|| {
            WalletError::InvalidConfig("missing index manifest row for indexed object".to_string())
        })?;
        let manifest_entries: Vec<IndexManifestEntry> = decode_bincode(raw_manifest.value())?;
        if manifest_entries.len() != expected_updates.len() {
            return Err(WalletError::InvalidConfig(
                "index manifest entry count mismatch".to_string(),
            ));
        }

        for (entry, update) in manifest_entries.iter().zip(expected_updates.iter()) {
            let expected_key = crate::db::index_codecs::encode_index_key_mode(
                index_key,
                crate::db::index_codecs::IndexKeyMode::A,
                update.table,
                &update.semantic_key,
                object_id,
            )?;

            if entry.table != update.table || entry.key.as_slice() != expected_key.as_slice() {
                return Err(WalletError::InvalidConfig(
                    "index manifest row mismatch".to_string(),
                ));
            }

            let def: TableDefinition<&[u8], &[u8]> =
                TableDefinition::new(update.table.store_name());
            let table = read_txn.open_table(def).map_err(|e| {
                WalletError::InvalidConfig(format!(
                    "redb open {} failed: {e}",
                    update.table.store_name()
                ))
            })?;
            let value = table
                .get(expected_key.as_slice())
                .map_err(|e| WalletError::InvalidConfig(format!("redb index read failed: {e}")))?
                .ok_or_else(|| {
                    WalletError::InvalidConfig(
                        "index manifest row missing from index table".to_string(),
                    )
                })?;
            if value.value() != update.value.0.as_slice() {
                return Err(WalletError::InvalidConfig(
                    "index value mismatch".to_string(),
                ));
            }
        }

        Ok(())
    }

    fn install_rotated_keys(
        &mut self,
        new_master_key: Hidden<[u8; 32]>,
        new_derived_keys: WalletDerivedKeys,
    ) {
        #[cfg(all(feature = "os_hardening", not(target_arch = "wasm32")))]
        {
            self.opened.master_key =
                z00z_utils::os_hardening::OwnedLockedBytes::new_best_effort_with(|slot| {
                    slot.copy_from_slice(new_master_key.reveal());
                });
        }

        #[cfg(not(all(feature = "os_hardening", not(target_arch = "wasm32"))))]
        {
            self.opened.master_key = new_master_key;
        }

        self.opened.derived_keys = new_derived_keys;
    }

    fn verify_rotation_state_on_db(
        db: &Database,
        wallet_id: &PersistWalletId,
        kdf_params: &KdfParams,
        password: &SafePassword,
        expected_master_key: &[u8; 32],
        expected_derived_keys: &WalletDerivedKeys,
        expect_rotation_marker: bool,
    ) -> WalletResult<()> {
        let read_txn = db
            .begin_read()
            .map_err(|e| WalletError::InvalidConfig(format!("redb begin_read failed: {e}")))?;

        let meta = read_txn
            .open_table(META_TABLE)
            .map_err(|e| WalletError::InvalidConfig(format!("redb open meta failed: {e}")))?;
        let has_rotation_marker = super::meta::rotation_in_progress(&meta)?;
        if has_rotation_marker != expect_rotation_marker {
            return Err(WalletError::InvalidConfig(if expect_rotation_marker {
                "wallet rotation marker missing".to_string()
            } else {
                "wallet rotation marker unexpectedly present".to_string()
            }));
        }

        let secrets = read_txn
            .open_table(SECRETS_TABLE)
            .map_err(|e| WalletError::InvalidConfig(format!("redb open secrets failed: {e}")))?;

        let master_key_record_raw = secrets
            .get(SECRETS_MASTER_KEY)
            .map_err(|e| WalletError::InvalidConfig(format!("redb secrets read failed: {e}")))?
            .ok_or_else(|| WalletError::InvalidConfig("missing secrets.master_key".to_string()))?;
        let master_key_record: MasterKeyRecord = decode_bincode_bounded(
            master_key_record_raw.value(),
            WalletError::InvalidConfig("wallet master key invalid".to_string()),
        )?;

        let km = WalletRedbKeyManager::new();
        let unwrapped_master_key = km
            .unwrap_master_key(wallet_id, password, kdf_params, &master_key_record)
            .map_err(|_| {
                WalletError::InvalidConfig("wallet master key verification failed".to_string())
            })?;

        use subtle::ConstantTimeEq;
        if unwrapped_master_key
            .reveal()
            .ct_eq(expected_master_key)
            .unwrap_u8()
            == 0
        {
            return Err(WalletError::InvalidConfig(
                "wallet master key verification failed".to_string(),
            ));
        }

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
                wallet_id,
                secret_name,
                expected_master_key,
                &record,
            )?;
            plaintext.wipe();
        }

        let objects = read_txn
            .open_table(OBJECTS_TABLE)
            .map_err(|e| WalletError::InvalidConfig(format!("redb open objects failed: {e}")))?;
        super::open::validate_objects_on_open(wallet_id, expected_derived_keys, &objects)?;

        for row in objects
            .iter()
            .map_err(|e| WalletError::InvalidConfig(format!("redb objects iter failed: {e}")))?
        {
            let (key, value) = row.map_err(|e| {
                WalletError::InvalidConfig(format!("redb objects read failed: {e}"))
            })?;
            let object_id = decode_object_id_be(key.value())?;
            let record = decode_object_record_bounded(value.value())?;
            let payload =
                decrypt_object_record(wallet_id, expected_derived_keys, object_id, &record)?;
            let expected_updates = Self::rotation_object_index_updates(
                object_id,
                payload.payload_version,
                payload.kind_id,
                payload.data.as_slice(),
            )?;
            Self::verify_object_index_state(
                &read_txn,
                expected_derived_keys.index_key.reveal(),
                object_id,
                expected_updates.as_slice(),
            )?;
        }

        Ok(())
    }

    fn verify_rotation_state(
        &self,
        password: &SafePassword,
        expected_master_key: &[u8; 32],
        expected_derived_keys: &WalletDerivedKeys,
    ) -> WalletResult<()> {
        Self::verify_rotation_state_on_db(
            &self.db,
            &self.opened.wallet_id,
            &self.opened.kdf_params,
            password,
            expected_master_key,
            expected_derived_keys,
            true,
        )
    }

    fn verify_archived_rotation_state(
        &self,
        password: &SafePassword,
        expected_master_key: &[u8; 32],
        expected_derived_keys: &WalletDerivedKeys,
        expect_rotation_marker: bool,
    ) -> WalletResult<()> {
        let WltBacking::ZstdTmpfs { original_path, .. } = &self.backing;

        let read_txn = self
            .db
            .begin_read()
            .map_err(|e| WalletError::InvalidConfig(format!("redb begin_read failed: {e}")))?;
        let meta = read_txn
            .open_table(META_TABLE)
            .map_err(|e| WalletError::InvalidConfig(format!("redb open meta failed: {e}")))?;
        let stored_chain_bytes = meta
            .get(META_WALLET_CHAIN)
            .map_err(|e| WalletError::InvalidConfig(format!("redb chain read failed: {e}")))?
            .ok_or_else(|| WalletError::InvalidConfig("wallet chain missing".to_string()))?;
        let stored_network_bytes = meta
            .get(META_WALLET_NETWORK)
            .map_err(|e| WalletError::InvalidConfig(format!("redb network read failed: {e}")))?
            .ok_or_else(|| WalletError::InvalidConfig("wallet network missing".to_string()))?;
        let identity = WalletIdentity {
            chain: decode_bincode(stored_chain_bytes.value())
                .map_err(|_| WalletError::InvalidConfig("wallet chain invalid".to_string()))?,
            network: decode_bincode(stored_network_bytes.value())
                .map_err(|_| WalletError::InvalidConfig("wallet network invalid".to_string()))?,
        };
        drop(meta);
        drop(read_txn);

        verify_archived_wallet_copy(
            original_path,
            &self.opened.wallet_id,
            &self.opened.kdf_params,
            password,
            &identity,
            expected_master_key,
            expected_derived_keys,
            expect_rotation_marker,
            Arc::clone(&self.time_provider),
            Arc::clone(&self.io),
        )
    }

    fn restore_rotation_snapshot(&self, snapshot: &RotationStateSnapshot) -> WalletResult<()> {
        let write_txn = self
            .db
            .begin_write()
            .map_err(|e| WalletError::InvalidConfig(format!("redb begin_write failed: {e}")))?;

        for (object_id, _) in &snapshot.object_rows {
            Self::clear_object_index_state(&write_txn, *object_id)?;
        }

        {
            let mut secrets = write_txn.open_table(SECRETS_TABLE).map_err(|e| {
                WalletError::InvalidConfig(format!("redb open secrets failed: {e}"))
            })?;
            for (secret_name, raw_record) in &snapshot.secrets_rows {
                secrets
                    .insert(secret_name.as_str(), raw_record.as_slice())
                    .map_err(|e| {
                        WalletError::InvalidConfig(format!("redb secrets insert failed: {e}"))
                    })?;
            }
        }

        Self::restore_snapshot_index_state(&write_txn, snapshot)?;
        {
            let mut objects = write_txn.open_table(OBJECTS_TABLE).map_err(|e| {
                WalletError::InvalidConfig(format!("redb open objects failed: {e}"))
            })?;
            for (object_id, raw_record) in &snapshot.object_rows {
                let object_key = encode_object_id_be(*object_id);
                objects
                    .insert(object_key.as_slice(), raw_record.as_slice())
                    .map_err(|e| {
                        WalletError::InvalidConfig(format!("redb objects insert failed: {e}"))
                    })?;
            }
        }

        {
            let mut meta = write_txn
                .open_table(META_TABLE)
                .map_err(|e| WalletError::InvalidConfig(format!("redb open meta failed: {e}")))?;
            super::meta::clear_rotation_in_progress(&mut meta)?;
            let _ = bump_wallet_write_meta(&mut meta, self.time_provider.as_ref())?;
        }

        commit_redb_write_txn_flush(self, write_txn)
    }

    fn rollback_rotation_failure(
        &self,
        snapshot: &RotationStateSnapshot,
        error: WalletError,
    ) -> WalletResult<u32> {
        if let Err(rollback_error) = self.restore_rotation_snapshot(snapshot) {
            return Err(WalletError::InvalidConfig(format!(
                "wallet rotation rollback failed after {error}: {rollback_error}"
            )));
        }

        Err(error)
    }

    pub(crate) fn rotate_master_key_persisted<R: SecureRngProvider>(
        &mut self,
        password: &SafePassword,
        rng_provider: R,
    ) -> WalletResult<u32> {
        let snapshot = self.snapshot_rotation_state()?;
        let wallet_id = self.opened.wallet_id.clone();
        let old_master_key = zeroize::Zeroizing::new(*self.opened.master_key.reveal());
        let old_derived_keys = self.opened.derived_keys.clone();

        let mut rng = rng_provider.rng();
        let mut new_master_key_bytes = zeroize::Zeroizing::new([0u8; 32]);
        rng.fill_bytes_ext(&mut *new_master_key_bytes);
        let new_master_key = Hidden::hide(*new_master_key_bytes);

        let km = WalletRedbKeyManager::new();
        let new_master_key_record = km
            .wrap_master_key(
                &wallet_id,
                password,
                &new_master_key,
                &self.opened.kdf_params,
            )
            .map_err(|_| WalletError::InvalidConfig("master key wrap failed".to_string()))?;
        let new_derived_keys = km
            .derive_wallet_keys(&new_master_key)
            .map_err(|e| WalletError::InvalidConfig(format!("key derivation failed: {e}")))?;

        let write_txn = self
            .db
            .begin_write()
            .map_err(|e| WalletError::InvalidConfig(format!("redb begin_write failed: {e}")))?;

        let mut records_rewrapped: u32 = 0;

        {
            let mut meta = write_txn
                .open_table(META_TABLE)
                .map_err(|e| WalletError::InvalidConfig(format!("redb open meta failed: {e}")))?;
            super::meta::store_rotation_in_progress(
                &mut meta,
                self.time_provider.compat_unix_timestamp_millis(),
            )?;
        }

        {
            let mut secrets = write_txn.open_table(SECRETS_TABLE).map_err(|e| {
                WalletError::InvalidConfig(format!("redb open secrets failed: {e}"))
            })?;

            secrets
                .insert(
                    SECRETS_MASTER_KEY,
                    encode_bincode(&new_master_key_record)?.as_slice(),
                )
                .map_err(|e| {
                    WalletError::InvalidConfig(format!("redb secrets insert failed: {e}"))
                })?;
            records_rewrapped = records_rewrapped.saturating_add(1);

            for (secret_name, raw_record) in &snapshot.secrets_rows {
                if secret_name == SECRETS_MASTER_KEY {
                    continue;
                }

                let mut record: SecretsRecord = decode_bincode_bounded(
                    raw_record,
                    WalletError::InvalidConfig("wallet secret invalid".to_string()),
                )?;
                let mut plaintext = decrypt_secret_record_post_unlock(
                    &wallet_id,
                    secret_name,
                    &old_master_key,
                    &record,
                )?;
                let envelope = encrypt_secret_record(
                    &mut rng,
                    &wallet_id,
                    secret_name,
                    new_master_key.reveal(),
                    &plaintext,
                )?;
                plaintext.wipe();
                record.envelope = envelope;

                secrets
                    .insert(secret_name.as_str(), encode_bincode(&record)?.as_slice())
                    .map_err(|e| {
                        WalletError::InvalidConfig(format!("redb secrets insert failed: {e}"))
                    })?;
                records_rewrapped = records_rewrapped.saturating_add(1);
            }
        }

        for (object_id, raw_record) in &snapshot.object_rows {
            let record = decode_object_record_bounded(raw_record)?;
            let mut payload =
                decrypt_object_record(&wallet_id, &old_derived_keys, *object_id, &record)?;
            let index_updates = Self::rotation_object_index_updates(
                *object_id,
                payload.payload_version,
                payload.kind_id,
                payload.data.as_slice(),
            )?;
            let new_record = encrypt_object_record(
                &mut rng,
                &wallet_id,
                new_derived_keys.data_key.reveal(),
                *object_id,
                payload.payload_version,
                payload.kind_id,
                std::mem::take(&mut payload.data),
            )?;
            write_object_with_index_key(
                &write_txn,
                *object_id,
                &new_record,
                index_updates.as_slice(),
                new_derived_keys.index_key.reveal(),
                self.time_provider.as_ref(),
            )?;
            records_rewrapped = records_rewrapped.saturating_add(1);
        }

        {
            let mut meta = write_txn
                .open_table(META_TABLE)
                .map_err(|e| WalletError::InvalidConfig(format!("redb open meta failed: {e}")))?;
            let _ = bump_wallet_write_meta(&mut meta, self.time_provider.as_ref())?;
        }

        commit_redb_write_txn_flush(self, write_txn)?;

        if let Err(error) =
            self.verify_rotation_state(password, new_master_key.reveal(), &new_derived_keys)
        {
            return self.rollback_rotation_failure(&snapshot, error);
        }

        if let Err(error) = self.verify_archived_rotation_state(
            password,
            new_master_key.reveal(),
            &new_derived_keys,
            true,
        ) {
            return self.rollback_rotation_failure(&snapshot, error);
        }

        #[cfg(test)]
        if take_rotate_master_fp_commit() {
            return self.rollback_rotation_failure(
                &snapshot,
                WalletError::InvalidConfig("injected rotate_master_key failure".to_string()),
            );
        }

        if let Err(error) = self.verify_archived_rotation_state(
            password,
            new_master_key.reveal(),
            &new_derived_keys,
            false,
        ) {
            return self.rollback_rotation_failure(&snapshot, error);
        }

        if let Err(error) = finalize_rotation_marker_on_db(
            &self.db,
            &self.backing,
            self.io.as_ref(),
            self.time_provider.as_ref(),
        ) {
            return self.rollback_rotation_failure(&snapshot, error);
        }

        self.install_rotated_keys(new_master_key, new_derived_keys);

        Ok(records_rewrapped)
    }
}

pub(super) fn verify_archived_wallet_copy(
    original_path: &Path,
    wallet_id: &PersistWalletId,
    kdf_params: &KdfParams,
    password: &SafePassword,
    identity: &WalletIdentity,
    expected_master_key: &[u8; 32],
    expected_derived_keys: &WalletDerivedKeys,
    expect_rotation_marker: bool,
    time_provider: Arc<dyn TimeProvider>,
    io: Arc<dyn WalletIo>,
) -> WalletResult<()> {
    const MAX_WLT_DECOMPRESSED_BYTES: usize = 128 * 1024 * 1024;

    let compressed = io.read_file(original_path)?;
    let db_bytes =
        z00z_utils::compression::zstd_decompress_bounded(&compressed, MAX_WLT_DECOMPRESSED_BYTES)
            .map_err(|e| WalletError::InvalidConfig(format!("wallet zstd payload invalid: {e}")))?;

    let shm_dir = Path::new("/dev/shm");
    if !io.path_exists(shm_dir)? {
        return Err(WalletError::InvalidConfig(
            "/dev/shm is required for archived wallet verification".to_string(),
        ));
    }

    let mut tmp_rng = SystemRngProvider.rng();
    let tmp_id = generate_16_bytes(&mut tmp_rng);
    let file_name = original_path
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("wallet.wlt");
    let verify_path = shm_dir.join(format!("{file_name}.verify.{}", to_hex(&tmp_id)));
    let inspect_path = shm_dir.join(format!("{file_name}.inspect.{}", to_hex(&tmp_id)));

    io.remove_file_best_effort(&verify_path);
    io.remove_file_best_effort(&inspect_path);

    let inspect_write_result = io.atomic_write_file_streaming(&inspect_path, &mut |out| {
        use std::io::Write;
        out.write_all(&db_bytes)
    });
    if let Err(error) = inspect_write_result {
        io.remove_file_best_effort(&verify_path);
        io.remove_file_best_effort(&inspect_path);
        return Err(error);
    }
    if let Err(error) = io.set_private_file_permissions(&inspect_path) {
        io.remove_file_best_effort(&verify_path);
        io.remove_file_best_effort(&inspect_path);
        return Err(error);
    }

    let result = (|| -> WalletResult<()> {
        let inspect_db = Database::open(&inspect_path).map_err(|e| {
            WalletError::InvalidConfig(format!("redb open archived inspect failed: {e}"))
        })?;

        if expect_rotation_marker {
            flush_work_file_to_wallet(io.as_ref(), &verify_path, &inspect_path)?;
        } else {
            let temp_backing = WltBacking::ZstdTmpfs {
                original_path: verify_path.clone(),
                work_path: inspect_path.clone(),
            };
            finalize_rotation_marker_on_db(
                &inspect_db,
                &temp_backing,
                io.as_ref(),
                time_provider.as_ref(),
            )?;
        }
        io.set_private_file_permissions(&verify_path)?;

        WalletSession::verify_rotation_state_on_db(
            &inspect_db,
            wallet_id,
            kdf_params,
            password,
            expected_master_key,
            expected_derived_keys,
            expect_rotation_marker,
        )?;
        drop(inspect_db);

        let reopened = super::open::open_wlt_with_deps(
            &verify_path,
            wallet_id,
            password,
            identity,
            Arc::clone(&time_provider),
            Arc::clone(&io),
        )?;

        use subtle::ConstantTimeEq;
        if reopened
            .opened
            .master_key
            .reveal()
            .ct_eq(expected_master_key)
            .unwrap_u8()
            == 0
        {
            return Err(WalletError::InvalidConfig(
                "archived wallet master key mismatch".to_string(),
            ));
        }

        for (actual_key, expected_key, label) in [
            (
                reopened.opened.derived_keys.data_key.reveal(),
                expected_derived_keys.data_key.reveal(),
                "data",
            ),
            (
                reopened.opened.derived_keys.index_key.reveal(),
                expected_derived_keys.index_key.reveal(),
                "index",
            ),
            (
                reopened.opened.derived_keys.integrity_key.reveal(),
                expected_derived_keys.integrity_key.reveal(),
                "integrity",
            ),
        ] {
            if actual_key.ct_eq(expected_key).unwrap_u8() == 0 {
                return Err(WalletError::InvalidConfig(format!(
                    "archived wallet {label} key mismatch"
                )));
            }
        }

        drop(reopened);

        Ok(())
    })();

    io.remove_file_best_effort(&verify_path);
    io.remove_file_best_effort(&inspect_path);
    result
}

pub(super) fn finalize_rotation_marker_on_db(
    db: &Database,
    backing: &WltBacking,
    io: &dyn WalletIo,
    time_provider: &dyn TimeProvider,
) -> WalletResult<()> {
    let write_txn = db
        .begin_write()
        .map_err(|e| WalletError::InvalidConfig(format!("redb begin_write failed: {e}")))?;

    {
        let mut meta = write_txn
            .open_table(META_TABLE)
            .map_err(|e| WalletError::InvalidConfig(format!("redb open meta failed: {e}")))?;
        super::meta::clear_rotation_in_progress(&mut meta)?;
        let _ = bump_wallet_write_meta(&mut meta, time_provider)?;
    }

    write_txn
        .commit()
        .map_err(|e| WalletError::InvalidConfig(format!("redb commit failed: {e}")))?;

    let WltBacking::ZstdTmpfs {
        original_path,
        work_path,
    } = backing;
    flush_work_file_to_wallet(io, original_path, work_path)
}

impl Drop for WalletSession {
    fn drop(&mut self) {
        let WltBacking::ZstdTmpfs { work_path, .. } = &self.backing;
        self.io.remove_file_best_effort(work_path);
    }
}

/// Open an existing `.wlt` file and unlock it using `password`.
///
/// Authentication failures are intentionally mapped to `WalletError::InvalidPassword`
/// to keep error details bounded.
pub fn open_wallet_store(
    path: &Path,
    wallet_id: &PersistWalletId,
    password: &SafePassword,
    identity: &WalletIdentity,
) -> WalletResult<WalletSession> {
    let time_provider: Arc<dyn TimeProvider> = Arc::new(SystemTimeProvider);
    let io = Arc::new(Z00ZWalletIo);

    open_wlt_with_deps(path, wallet_id, password, identity, time_provider, io)
}
