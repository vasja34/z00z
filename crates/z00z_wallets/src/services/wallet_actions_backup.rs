#[derive(Clone)]
struct WalletRestorePack {
    profile: crate::db::WalletProfilePayload,
    owned_assets: Vec<crate::db::OwnedAssetPayload>,
    owned_objects: Vec<crate::db::WalletInventoryPayload>,
    scan_state: Option<crate::db::ScanStatePayload>,
    stealth_meta: Option<crate::db::StealthMetaPayload>,
    tofu_pins: Option<crate::db::TofuPinsPayload>,
    keys: Option<crate::db::KeysPayload>,
    seed_phrase: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
enum RestoreMarkStage {
    Prepared,
    HistoryStep,
    WltStep,
    PublishStep,
    Published,
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
struct WalletRestoreMark {
    stage: RestoreMarkStage,
    live_wlt: PathBuf,
    staged_wlt: PathBuf,
    wlt_backup: PathBuf,
    had_wlt: bool,
    live_hist: Option<PathBuf>,
    staged_hist: Option<PathBuf>,
    hist_backup: Option<PathBuf>,
    had_hist: bool,
}

impl WalletRestoreMark {
    fn new(
        live_wlt: PathBuf,
        staged_wlt: PathBuf,
        wlt_backup: PathBuf,
        had_wlt: bool,
        live_hist: Option<PathBuf>,
        staged_hist: Option<PathBuf>,
        hist_backup: Option<PathBuf>,
        had_hist: bool,
    ) -> Self {
        Self {
            stage: RestoreMarkStage::Prepared,
            live_wlt,
            staged_wlt,
            wlt_backup,
            had_wlt,
            live_hist,
            staged_hist,
            hist_backup,
            had_hist,
        }
    }
}

struct BackupCreateLimitGuard {
    limits: std::sync::Arc<
        tokio::sync::RwLock<
            std::collections::BTreeMap<PersistWalletId, BackupCreateRateLimitState>,
        >,
    >,
    wallet_id: PersistWalletId,
    armed: bool,
}

impl BackupCreateLimitGuard {
    fn new(
        limits: std::sync::Arc<
            tokio::sync::RwLock<
                std::collections::BTreeMap<PersistWalletId, BackupCreateRateLimitState>,
            >,
        >,
        wallet_id: PersistWalletId,
    ) -> Self {
        Self {
            limits,
            wallet_id,
            armed: true,
        }
    }

    fn disarm(&mut self) {
        self.armed = false;
    }
}

impl Drop for BackupCreateLimitGuard {
    fn drop(&mut self) {
        if !self.armed {
            return;
        }

        if let Ok(handle) = tokio::runtime::Handle::try_current() {
            let limits = std::sync::Arc::clone(&self.limits);
            let wallet_id = self.wallet_id.clone();
            handle.spawn(async move {
                let mut map = limits.write().await;
                if let Some(state) = map.get_mut(&wallet_id) {
                    state.in_progress = false;
                }
            });
        }
    }
}

const RESTORE_ATOMIC_FAILPOINT_ENV: &str = "Z00Z_TEST_RESTORE_ATOMIC_FAILPOINT";

fn is_restore_atomic_test_process() -> bool {
    std::env::current_exe()
        .ok()
        .and_then(|path| {
            path.file_stem()
                .and_then(|stem| stem.to_str())
                .map(str::to_owned)
        })
        .map(|stem| {
            stem == "test_wallet_restore_atomic"
                || stem.starts_with("test_wallet_restore_atomic-")
        })
        .unwrap_or(false)
}

fn restore_atomic_failpoint_is(name: &str) -> bool {
    if !is_restore_atomic_test_process() {
        return false;
    }

    std::env::var_os(RESTORE_ATOMIC_FAILPOINT_ENV)
        .and_then(|value| value.into_string().ok())
        .as_deref()
        == Some(name)
}

impl WalletService {
    fn sanitize_backup_wallet_id(wallet_id: &PersistWalletId) -> String {
        wallet_id
            .0
            .chars()
            .map(|c| {
                if c.is_ascii_alphanumeric() || c == '-' || c == '_' {
                    c
                } else {
                    '_'
                }
            })
            .collect()
    }

    fn default_backup_settings(
        &self,
        wallet_id: &PersistWalletId,
    ) -> WalletResult<PersistBackupSettings> {
        let defaults = crate::services::wallet_runtime_config::resolve_wallet_backup_defaults()?;
        let backup_location = defaults
            .base_directory
            .join(Self::sanitize_backup_wallet_id(wallet_id))
            .to_string_lossy()
            .to_string();
        Ok(PersistBackupSettings {
            auto_backup_enabled: defaults.auto_backup_enabled,
            backup_interval_hours: defaults.backup_interval_hours,
            backup_location,
            encrypt_backups: defaults.encrypt_backups,
        })
    }

    async fn get_backup_settings(
        &self,
        wallet_id: &PersistWalletId,
    ) -> WalletResult<PersistBackupSettings> {
        let store = self.backup_settings.read().await;
        store
            .get(wallet_id)
            .cloned()
            .map(Ok)
            .unwrap_or_else(|| self.default_backup_settings(wallet_id))
    }

    async fn set_backup_settings(
        &self,
        wallet_id: &PersistWalletId,
        settings: PersistBackupSettings,
    ) {
        let mut store = self.backup_settings.write().await;
        store.insert(wallet_id.clone(), settings);
    }

    /// Precheck rate limit for `wallet.backup.create_backup`.
    ///
    /// Allows at most one backup creation per wallet per hour.
    pub async fn backup_create_rate_limit_precheck(
        &self,
        wallet_id: &PersistWalletId,
    ) -> WalletResult<u64> {
        let window_ms = crate::services::wallet_runtime_config::resolve_wallet_backup_defaults()?
            .create_rate_limit_window_ms;
        let now_ms = self.require_now_ms()?;
        let mut limits = self.backup_create_limits.write().await;
        let state = limits
            .entry(wallet_id.clone())
            .or_insert(BackupCreateRateLimitState {
                last_created_at: 0,
                in_progress: false,
            });

        if state.last_created_at != 0 && now_ms.saturating_sub(state.last_created_at) < window_ms {
            let retry_ms = window_ms.saturating_sub(now_ms.saturating_sub(state.last_created_at));
            let retry_after_seconds = retry_ms.div_ceil(1_000) as u32;
            return Err(WalletError::RateLimited {
                retry_after_seconds,
            });
        }

        if state.in_progress {
            return Err(WalletError::RateLimited {
                retry_after_seconds: 1,
            });
        }

        state.in_progress = true;
        Ok(now_ms)
    }

    async fn backup_create_rate_limit_finish(
        &self,
        wallet_id: &PersistWalletId,
        started_at_ms: u64,
        committed: bool,
    ) {
        let mut limits = self.backup_create_limits.write().await;
        let state = limits
            .entry(wallet_id.clone())
            .or_insert(BackupCreateRateLimitState {
                last_created_at: 0,
                in_progress: false,
            });
        if committed {
            state.last_created_at = started_at_ms;
        }
        state.in_progress = false;
    }

    fn ensure_backup_dir(path: &Path) -> WalletResult<()> {
        create_dir_all(path).map_err(|e| {
            WalletError::InvalidConfig(format!("Failed to create backup directory: {e}"))
        })
    }

    async fn collect_tx_history_jsonl(
        &self,
        wallet_id: &PersistWalletId,
    ) -> WalletResult<(Vec<u8>, Vec<crate::persistence::tx::TxRecord>)> {
        let live_path = self.wallet_history_jsonl_path(wallet_id);
        let wallet_id = wallet_id.clone();
        tokio::task::spawn_blocking(move || {
            let history_lock =
                crate::persistence::tx::tx_history_path_lock(&live_path).map_err(|e| {
                    WalletError::InvalidConfig(format!(
                        "Backup tx-history export lock acquisition failed: {e}"
                    ))
                })?;
            let _history_guard = history_lock.lock().map_err(|e| {
                WalletError::InvalidConfig(format!(
                    "Backup tx-history export lock acquisition failed: {e}"
                ))
            })?;

            if let Some(parent) = live_path.parent() {
                z00z_utils::io::create_dir_all(parent).map_err(|e| {
                    WalletError::InvalidConfig(format!(
                        "Backup tx-history export directory setup failed: {e}"
                    ))
                })?;
            }

            if !z00z_utils::io::path_exists(&live_path).map_err(|e| {
                WalletError::InvalidConfig(format!("Backup tx-history export failed: {e}"))
            })? {
                z00z_utils::io::write_file(&live_path, &[]).map_err(|e| {
                    WalletError::InvalidConfig(format!(
                        "Backup tx-history export initialization failed: {e}"
                    ))
                })?;
            }

            let history_bytes = z00z_utils::io::read_file(&live_path).map_err(|e| {
                WalletError::InvalidConfig(format!("Backup tx-history export failed: {e}"))
            })?;
            Self::validate_tx_history_bytes(&wallet_id, &history_bytes).map_err(|e| {
                WalletError::InvalidConfig(format!(
                    "Backup tx-history JSONL validation failed: {e}"
                ))
            })?;
            let history_view =
                crate::backup::decode_tx_history_jsonl(&history_bytes).map_err(|e| {
                    WalletError::InvalidConfig(format!(
                        "Backup tx-history JSONL validation failed: {e}"
                    ))
                })?;

            Ok((history_bytes, history_view))
        })
        .await
        .map_err(|_| {
            WalletError::InvalidConfig("Backup tx-history export task failed".to_string())
        })?
    }

    pub(crate) fn validate_tx_history_bytes(
        wallet_id: &PersistWalletId,
        history_bytes: &[u8],
    ) -> Result<(), String> {
        if history_bytes.len() as u64 > crate::persistence::tx::MAX_TX_HISTORY_JSONL_BYTES {
            return Err(format!(
                "tx-history JSONL file too large: {} bytes",
                history_bytes.len()
            ));
        }

        let rows = crate::backup::decode_tx_history_rows(history_bytes)?;
        let expected_stem = Self::wallet_stem(wallet_id);

        if let Some(row) = rows.iter().find(|row| row.wallet_stem != expected_stem) {
            return Err(format!(
                "tx-history wallet stem mismatch: expected {expected_stem}, found {}",
                row.wallet_stem
            ));
        }

        Ok(())
    }

    #[cfg(test)]
    pub(crate) fn import_tx_history_jsonl(
        &self,
        wallet_id: &PersistWalletId,
        history_path: &Path,
    ) -> WalletResult<()> {
        let history_bytes = z00z_utils::io::read_file(history_path).map_err(|e| {
            WalletError::InvalidConfig(format!("Backup tx-history JSONL import failed: {e}"))
        })?;
        Self::validate_tx_history_bytes(wallet_id, &history_bytes).map_err(|e| {
            WalletError::InvalidConfig(format!("Backup tx-history JSONL import failed: {e}"))
        })?;
        self.write_tx_history_jsonl_bytes(wallet_id, &history_bytes)
    }

    pub(crate) fn write_tx_history_jsonl_bytes(
        &self,
        wallet_id: &PersistWalletId,
        history_bytes: &[u8],
    ) -> WalletResult<()> {
        Self::validate_tx_history_bytes(wallet_id, history_bytes).map_err(|e| {
            WalletError::InvalidConfig(format!("Backup tx-history JSONL import failed: {e}"))
        })?;

        let history_path = self.wallet_history_jsonl_path(wallet_id);
        let history_lock =
            crate::persistence::tx::tx_history_path_lock(&history_path).map_err(|e| {
                WalletError::InvalidConfig(format!(
                    "Backup tx-history lock acquisition failed: {e}"
                ))
            })?;
        let _history_guard = history_lock.lock().map_err(|e| {
            WalletError::InvalidConfig(format!("Backup tx-history lock acquisition failed: {e}"))
        })?;

        if let Some(parent) = history_path.parent() {
            z00z_utils::io::create_dir_all(parent).map_err(|e| {
                WalletError::InvalidConfig(format!(
                    "Backup tx-history import directory setup failed: {e}"
                ))
            })?;
        }
        z00z_utils::io::atomic_write_file_private(&history_path, history_bytes).map_err(|e| {
            WalletError::InvalidConfig(format!("Backup tx-history import failed: {e}"))
        })
    }

    fn restore_claimed_assets_from_payloads(
        payloads: &[crate::db::OwnedAssetPayload],
    ) -> WalletResult<Vec<Asset>> {
        payloads
            .iter()
            .filter(|payload| payload.is_live_claimed_status())
            .map(|payload| payload.clone().validate_invariants())
            .collect()
    }

    fn validate_backup_owned_objects(
        payloads: Vec<crate::db::WalletInventoryPayload>,
        wallet_id: &PersistWalletId,
    ) -> WalletResult<Vec<crate::db::WalletInventoryPayload>> {
        let mut dedup_ids = std::collections::BTreeSet::new();
        let mut canonical_objects = Vec::with_capacity(payloads.len());

        for payload in payloads {
            let canonical = match payload {
                crate::db::OwnedObjectPayload::Asset(_) => {
                    return Err(WalletError::InvalidConfig(
                        "backup owned_objects must not carry asset variants".to_string(),
                    ));
                }
                crate::db::OwnedObjectPayload::Voucher(payload) => {
                    let payload = payload.migrate_to_current()?;
                    payload.verify_checksum()?;
                    payload.validate_invariants()?;
                    if payload.wallet_id != *wallet_id {
                        return Err(WalletError::InvalidConfig(
                            "owned voucher wallet id mismatch in backup payload".to_string(),
                        ));
                    }
                    crate::db::OwnedObjectPayload::Voucher(payload)
                }
                crate::db::OwnedObjectPayload::Right(payload) => {
                    let payload = payload.migrate_to_current()?;
                    payload.verify_checksum()?;
                    payload.validate_invariants()?;
                    if payload.wallet_id != *wallet_id {
                        return Err(WalletError::InvalidConfig(
                            "owned right wallet id mismatch in backup payload".to_string(),
                        ));
                    }
                    crate::db::OwnedObjectPayload::Right(payload)
                }
            };

            let family_tag = match canonical.family() {
                crate::db::OwnedObjectFamily::Asset => 1u8,
                crate::db::OwnedObjectFamily::Voucher => 2u8,
                crate::db::OwnedObjectFamily::Right => 3u8,
            };
            let dedup_key = (family_tag, canonical.stable_object_key());
            if !dedup_ids.insert(dedup_key) {
                return Err(WalletError::InvalidConfig(
                    "duplicate owned object stable key in backup payload".to_string(),
                ));
            }
            canonical_objects.push(canonical);
        }

        Ok(canonical_objects)
    }

    fn validate_backup_manifest(
        manifest: &crate::db::BackupManifestPayload,
        wallet_id: &PersistWalletId,
        identity: &WalletIdentity,
        owned_assets: &[crate::db::OwnedAssetPayload],
        owned_objects: &[crate::db::WalletInventoryPayload],
        scan_state: Option<&crate::db::ScanStatePayload>,
        stealth_meta: Option<&crate::db::StealthMetaPayload>,
        tofu_pins: Option<&crate::db::TofuPinsPayload>,
        keys: Option<&crate::db::KeysPayload>,
        history_len: Option<usize>,
    ) -> WalletResult<()> {
        manifest.verify_checksum()?;
        if manifest.version != crate::db::BackupManifestPayload::VERSION {
            return Err(WalletError::UnsupportedVersion(manifest.version));
        }
        if manifest.wallet_id != *wallet_id {
            return Err(WalletError::InvalidConfig(
                "backup manifest wallet id mismatch".to_string(),
            ));
        }
        if manifest.network != identity.network {
            return Err(WalletError::InvalidConfig(
                "backup manifest network mismatch".to_string(),
            ));
        }
        if manifest.chain != identity.chain {
            return Err(WalletError::InvalidConfig(
                "backup manifest chain mismatch".to_string(),
            ));
        }
        if manifest.profile_count != 1 {
            return Err(WalletError::InvalidConfig(
                "backup manifest profile count mismatch".to_string(),
            ));
        }
        if manifest.owned_asset_count != owned_assets.len() as u32 {
            return Err(WalletError::InvalidConfig(
                "backup manifest owned asset count mismatch".to_string(),
            ));
        }
        if manifest.owned_object_count != owned_objects.len() as u32 {
            return Err(WalletError::InvalidConfig(
                "backup manifest owned object count mismatch".to_string(),
            ));
        }
        if manifest.scan_state_count != u32::from(scan_state.is_some()) {
            return Err(WalletError::InvalidConfig(
                "backup manifest scan state count mismatch".to_string(),
            ));
        }
        if manifest.stealth_meta_count != u32::from(stealth_meta.is_some()) {
            return Err(WalletError::InvalidConfig(
                "backup manifest stealth meta count mismatch".to_string(),
            ));
        }
        if manifest.tofu_pins_count != u32::from(tofu_pins.is_some()) {
            return Err(WalletError::InvalidConfig(
                "backup manifest tofu pins count mismatch".to_string(),
            ));
        }
        if manifest.key_ref_count
            != keys
                .map(|payload| payload.signing_keys.len() as u32)
                .unwrap_or(0)
        {
            return Err(WalletError::InvalidConfig(
                "backup manifest key ref count mismatch".to_string(),
            ));
        }
        if let Some(history_len) = history_len {
            if manifest.tx_record_count != history_len as u32 {
                return Err(WalletError::InvalidConfig(
                    "backup manifest tx record count mismatch".to_string(),
                ));
            }
        } else if manifest.tx_record_count != 0 {
            return Err(WalletError::InvalidConfig(
                "backup manifest tx record count mismatch".to_string(),
            ));
        }
        if manifest.has_tx_history_sidecar != history_len.is_some() {
            return Err(WalletError::InvalidConfig(
                "backup manifest tx-history plane mismatch".to_string(),
            ));
        }
        if manifest.tx_history_plane != crate::db::BackupManifestPayload::TX_HISTORY_JSONL {
            return Err(WalletError::InvalidConfig(
                "backup manifest tx-history plane invalid".to_string(),
            ));
        }

        Ok(())
    }

    fn validate_wallet_restore_pack(
        export_pack: crate::wallet::persistence::WalletExportPack,
        identity: &WalletIdentity,
        history_bytes: Option<&[u8]>,
    ) -> WalletResult<WalletRestorePack> {
        let crate::wallet::persistence::WalletExportPack {
            version: _,
            manifest,
            wallet_profile,
            owned_assets,
            owned_objects,
            scan_state,
            stealth_meta,
            tofu_pins,
            keys,
            tx_history_plane,
            seed_phrase,
            wallet_identity,
        } = export_pack;

        if seed_phrase.trim().is_empty() {
            return Err(WalletError::InvalidParams(
                "Invalid backup payload".to_string(),
            ));
        }

        if let Some(export_identity) = wallet_identity.as_ref() {
            if export_identity.network != identity.network {
                return Err(WalletError::InvalidConfig(
                    "wallet export network mismatch".to_string(),
                ));
            }
            if export_identity.chain != identity.chain {
                return Err(WalletError::InvalidConfig(
                    "wallet export chain mismatch".to_string(),
                ));
            }
        }

        let profile = wallet_profile.ok_or_else(|| {
            WalletError::InvalidConfig("backup payload missing wallet profile".to_string())
        })?;
        let profile = profile.migrate_to_current()?;
        profile.verify_checksum()?;

        if let Some(bytes) = history_bytes {
            Self::validate_tx_history_bytes(&profile.wallet_id, bytes).map_err(|e| {
                WalletError::InvalidConfig(format!("Backup tx-history JSONL import failed: {e}"))
            })?;
        }

        let mut dedup_ids = std::collections::BTreeSet::new();
        let mut canonical_assets = Vec::with_capacity(owned_assets.len());
        for payload in owned_assets {
            let canonical = payload.migrate_to_current()?;
            canonical.verify_checksum()?;
            let _ = canonical.validate_invariants()?;
            if canonical.wallet_id != profile.wallet_id {
                return Err(WalletError::InvalidConfig(
                    "owned asset wallet id mismatch in backup payload".to_string(),
                ));
            }
            if !dedup_ids.insert(canonical.asset_id) {
                return Err(WalletError::InvalidConfig(
                    "duplicate owned asset id in backup payload".to_string(),
                ));
            }
            canonical_assets.push(canonical);
        }

        let manifest = manifest.ok_or_else(|| {
            WalletError::InvalidConfig("backup payload missing manifest".to_string())
        })?;
        let history_len = history_bytes
            .map(crate::backup::decode_tx_history_jsonl)
            .transpose()
            .map_err(|e| {
                WalletError::InvalidConfig(format!("Backup tx-history JSONL import failed: {e}"))
            })?
            .map(|records| records.len());
        let canonical_objects =
            Self::validate_backup_owned_objects(owned_objects, &profile.wallet_id)?;
        Self::validate_backup_manifest(
            &manifest,
            &profile.wallet_id,
            identity,
            canonical_assets.as_slice(),
            canonical_objects.as_slice(),
            scan_state.as_ref(),
            stealth_meta.as_ref(),
            tofu_pins.as_ref(),
            keys.as_ref(),
            history_len,
        )?;
        if tx_history_plane.as_deref() != Some(crate::db::BackupManifestPayload::TX_HISTORY_JSONL) {
            return Err(WalletError::InvalidConfig(
                "wallet export tx-history plane invalid".to_string(),
            ));
        }

        Ok(WalletRestorePack {
            profile,
            owned_assets: canonical_assets,
            owned_objects: canonical_objects,
            scan_state,
            stealth_meta,
            tofu_pins,
            keys,
            seed_phrase,
        })
    }

    fn suffix_path(path: &Path, suffix: &str) -> PathBuf {
        let file_name = path
            .file_name()
            .map(|value| value.to_string_lossy().to_string())
            .unwrap_or_else(|| "wallet".to_string());
        path.with_file_name(format!("{file_name}{suffix}"))
    }

    fn staged_wlt_path(path: &Path) -> PathBuf {
        Self::suffix_path(path, ".restore.tmp")
    }

    fn staged_history_path(path: &Path) -> PathBuf {
        Self::suffix_path(path, ".restore.tmp")
    }

    fn restore_mark_path(path: &Path) -> PathBuf {
        Self::suffix_path(path, ".restore.json")
    }

    fn wlt_backup_path(path: &Path) -> PathBuf {
        Self::suffix_path(path, ".bak")
    }

    fn history_backup_path(path: &Path) -> PathBuf {
        Self::suffix_path(path, ".bak")
    }

    fn read_existing_file_bytes(
        path: &Path,
        read_failed_context: &str,
    ) -> WalletResult<Option<Vec<u8>>> {
        match z00z_utils::io::read_file(path) {
            Ok(bytes) => Ok(Some(bytes)),
            Err(z00z_utils::io::IoError::Io(err))
                if matches!(
                    err.kind(),
                    std::io::ErrorKind::NotFound | std::io::ErrorKind::IsADirectory
                ) =>
            {
                Ok(None)
            }
            Err(err) => Err(WalletError::InvalidConfig(format!(
                "{read_failed_context}: {err}"
            ))),
        }
    }

    fn cleanup_restore_path(path: &Path) {
        crate::db::wallet_io::remove_file_best_effort(path);

        let mut lock_path = path.as_os_str().to_os_string();
        lock_path.push(".lock");
        crate::db::wallet_io::remove_file_best_effort(Path::new(&lock_path));

        let mut tmp_path = path.as_os_str().to_os_string();
        tmp_path.push(".tmp");
        crate::db::wallet_io::remove_file_best_effort(Path::new(&tmp_path));
    }

    fn cleanup_wlt_temp(path: &Path) {
        Self::cleanup_restore_path(path);
    }

    fn write_restore_mark(path: &Path, mark: &WalletRestoreMark) -> WalletResult<()> {
        use z00z_utils::codec::{Codec, JsonCodec};

        if let Some(parent) = path.parent() {
            z00z_utils::io::create_dir_all(parent).map_err(|error| {
                WalletError::InvalidConfig(format!(
                    "Backup restore mark directory create failed: {error}"
                ))
            })?;
        }
        let bytes = JsonCodec.serialize(mark).map_err(|error| {
            WalletError::InvalidConfig(format!("Backup restore mark encode failed: {error}"))
        })?;
        z00z_utils::io::atomic_write_file_private(path, &bytes).map_err(|error| {
            WalletError::InvalidConfig(format!("Backup restore mark write failed: {error}"))
        })
    }

    fn load_restore_mark(path: &Path) -> WalletResult<Option<WalletRestoreMark>> {
        use z00z_utils::codec::{Codec, JsonCodec};

        let bytes = match z00z_utils::io::read_file(path) {
            Ok(bytes) => bytes,
            Err(z00z_utils::io::IoError::Io(err))
                if matches!(
                    err.kind(),
                    std::io::ErrorKind::NotFound | std::io::ErrorKind::IsADirectory
                ) =>
            {
                return Ok(None);
            }
            Err(error) => {
                return Err(WalletError::InvalidConfig(format!(
                    "Backup restore mark read failed: {error}"
                )));
            }
        };

        JsonCodec.deserialize(&bytes).map(Some).map_err(|error| {
            WalletError::InvalidConfig(format!("Backup restore mark decode failed: {error}"))
        })
    }

    fn rollback_restore_file(
        live_path: &Path,
        backup_path: &Path,
        had_live: bool,
        context: &str,
    ) -> WalletResult<()> {
        if had_live {
            let backup_bytes = z00z_utils::io::read_file(backup_path).map_err(|error| {
                WalletError::InvalidConfig(format!("{context}: {error}"))
            })?;
            z00z_utils::io::atomic_write_file_private(live_path, &backup_bytes).map_err(
                |error| WalletError::InvalidConfig(format!("{context}: {error}")),
            )?;
        } else {
            crate::db::wallet_io::remove_file_best_effort(live_path);
        }
        Ok(())
    }

    fn cleanup_restore_mark(mark_path: &Path, mark: &WalletRestoreMark) {
        Self::cleanup_restore_path(mark_path);
        Self::cleanup_wlt_temp(&mark.staged_wlt);
        Self::cleanup_restore_path(&mark.wlt_backup);
        if let Some(path) = mark.staged_hist.as_ref() {
            Self::cleanup_restore_path(path);
        }
        if let Some(path) = mark.hist_backup.as_ref() {
            Self::cleanup_restore_path(path);
        }
    }

    fn resume_restore_mark(&self, mark_path: &Path) -> WalletResult<()> {
        let Some(mark) = Self::load_restore_mark(mark_path)? else {
            return Ok(());
        };

        let history_lock = if let Some(path) = mark.live_hist.as_ref() {
            Some(
                crate::persistence::tx::tx_history_path_lock(path).map_err(|error| {
                    WalletError::InvalidConfig(format!(
                        "Backup restore tx-history lock failed: {error}"
                    ))
                })?,
            )
        } else {
            None
        };
        let history_guard = if let Some(lock) = history_lock.as_ref() {
            Some(lock.lock().map_err(|error| {
                WalletError::InvalidConfig(format!("Backup restore tx-history lock failed: {error}"))
            })?)
        } else {
            None
        };

        match mark.stage {
            RestoreMarkStage::Prepared => {}
            RestoreMarkStage::HistoryStep => {
                if let (Some(live_path), Some(backup_path)) =
                    (mark.live_hist.as_ref(), mark.hist_backup.as_ref())
                {
                    Self::rollback_restore_file(
                        live_path,
                        backup_path,
                        mark.had_hist,
                        "Backup restore history rollback failed",
                    )?;
                }
            }
            RestoreMarkStage::WltStep | RestoreMarkStage::PublishStep => {
                if let (Some(live_path), Some(backup_path)) =
                    (mark.live_hist.as_ref(), mark.hist_backup.as_ref())
                {
                    Self::rollback_restore_file(
                        live_path,
                        backup_path,
                        mark.had_hist,
                        "Backup restore history rollback failed",
                    )?;
                }
                Self::rollback_restore_file(
                    &mark.live_wlt,
                    &mark.wlt_backup,
                    mark.had_wlt,
                    "Backup restore .wlt rollback failed",
                )?;
            }
            RestoreMarkStage::Published => {}
        }

        drop(history_guard);
        drop(history_lock);
        Self::cleanup_restore_mark(mark_path, &mark);
        Ok(())
    }

    fn stage_history_restore(path: &Path, history_bytes: &[u8]) -> WalletResult<()> {
        crate::backup::decode_tx_history_jsonl(history_bytes).map_err(|e| {
            WalletError::InvalidConfig(format!("Backup tx-history JSONL import failed: {e}"))
        })?;

        if let Some(parent) = path.parent() {
            z00z_utils::io::create_dir_all(parent).map_err(|e| {
                WalletError::InvalidConfig(format!(
                    "Backup restore staging directory setup failed: {e}"
                ))
            })?;
        }

        z00z_utils::io::atomic_write_file_private(path, history_bytes).map_err(|e| {
            WalletError::InvalidConfig(format!("Backup restore staging tx-history failed: {e}"))
        })
    }

    async fn stage_wlt_restore(
        &self,
        path: &Path,
        restore: &WalletRestorePack,
        password: &SafePassword,
        identity: &WalletIdentity,
    ) -> WalletResult<()> {
        use z00z_utils::codec::{BincodeCodec, Codec};

        let wlt_store = Arc::clone(&self.wlt_store);
        let stage_path = path.to_path_buf();
        let stage_profile = restore.profile.clone();
        let stage_wallet_id = stage_profile.wallet_id.clone();
        let stage_password = password.clone();
        let stage_seed_phrase = restore.seed_phrase.clone();
        let stage_identity = identity.clone();
        let stage_scan_state = restore.scan_state.clone();
        let stage_stealth_meta = restore.stealth_meta.clone();
        let stage_tofu_pins = restore.tofu_pins.clone();
        let stage_keys = restore.keys.clone();
        let stage_owned_assets = restore.owned_assets.clone();
        let stage_owned_objects = restore.owned_objects.clone();
        let profile_bytes = BincodeCodec
            .serialize(&stage_profile)
            .map_err(|e| WalletError::InvalidConfig(format!("Binary serialization failed: {e}")))?;

        tokio::task::spawn_blocking(move || {
            wlt_store.create_wallet_store(
                &stage_path,
                &stage_wallet_id,
                &stage_password,
                &stage_seed_phrase,
                &stage_identity,
            )?;
            let session = wlt_store.open_wallet_store(
                &stage_path,
                &stage_wallet_id,
                &stage_password,
                &stage_identity,
            )?;
            let _ = wlt_store.write_wallet_profile(&session, profile_bytes)?;

            if let Some(payload) = stage_scan_state.as_ref() {
                let _ = crate::db::upsert_scan_state(&session, payload, SystemRngProvider)?;
            }
            if let Some(payload) = stage_stealth_meta.as_ref() {
                let _ = crate::db::upsert_stealth_meta(&session, payload, SystemRngProvider)?;
            }
            if let Some(payload) = stage_tofu_pins.as_ref() {
                let _ = crate::db::upsert_tofu_pins(&session, payload, SystemRngProvider)?;
            }
            if let Some(payload) = stage_keys.as_ref() {
                let _ = crate::db::upsert_keys_payload(&session, payload, SystemRngProvider)?;
            }
            crate::db::wallet_asset_store()
                .replace_payloads_for_restore(&session, stage_owned_assets.as_slice())?;
            for payload in stage_owned_objects {
                match payload {
                    crate::db::WalletInventoryPayload::Asset(_) => {
                        return Err(WalletError::InvalidConfig(
                            "backup owned_objects must not carry asset variants".to_string(),
                        ));
                    }
                    crate::db::WalletInventoryPayload::Voucher(payload) => {
                        let _ =
                            crate::db::object_inventory_store().put_voucher(&session, payload)?;
                    }
                    crate::db::WalletInventoryPayload::Right(payload) => {
                        let _ = crate::db::object_inventory_store().put_right(&session, payload)?;
                    }
                }
            }
            Ok(())
        })
        .await
        .map_err(|_| WalletError::InvalidConfig(".wlt staging task failed".to_string()))?
    }

    pub(crate) async fn restore_wallet_pack_atomic(
        &self,
        export_pack: crate::wallet::persistence::WalletExportPack,
        password: &SafePassword,
        wallet_name: Option<&str>,
        identity: &WalletIdentity,
        history_bytes: Option<&[u8]>,
    ) -> WalletResult<PersistWalletId> {
        let mut restore = Self::validate_wallet_restore_pack(export_pack, identity, history_bytes)?;
        if let Some(name) = wallet_name.filter(|value| !value.trim().is_empty()) {
            restore.profile.name = name.to_string();
            restore.profile.checksum = Some(restore.profile.compute_checksum());
        }

        let wallet_id = restore.profile.wallet_id.clone();
        let final_wlt = self.wlt_file_path(&wallet_id);
        let staged_wlt = Self::staged_wlt_path(&final_wlt);
        let wlt_backup = Self::wlt_backup_path(&final_wlt);
        let mark_path = Self::restore_mark_path(&final_wlt);
        let final_history = history_bytes.map(|_| self.wallet_history_jsonl_path(&wallet_id));
        let staged_history = final_history
            .as_ref()
            .map(|path| Self::staged_history_path(path));
        let history_backup = final_history
            .as_ref()
            .map(|path| Self::history_backup_path(path));

        self.resume_restore_mark(&mark_path)?;
        Self::cleanup_wlt_temp(&staged_wlt);
        Self::cleanup_restore_path(&wlt_backup);
        Self::cleanup_restore_path(&mark_path);
        if let Some(path) = staged_history.as_ref() {
            Self::cleanup_restore_path(path);
        }
        if let Some(path) = history_backup.as_ref() {
            Self::cleanup_restore_path(path);
        }

        let stage_outcome = async {
            self.stage_wlt_restore(&staged_wlt, &restore, password, identity)
                .await?;
            if let (Some(path), Some(bytes)) = (staged_history.as_ref(), history_bytes) {
                Self::stage_history_restore(path, bytes)?;
            }
            Ok::<(), WalletError>(())
        }
        .await;
        if let Err(err) = stage_outcome {
            Self::cleanup_wlt_temp(&staged_wlt);
            Self::cleanup_restore_path(&wlt_backup);
            Self::cleanup_restore_path(&mark_path);
            if let Some(path) = staged_history.as_ref() {
                Self::cleanup_restore_path(path);
            }
            if let Some(path) = history_backup.as_ref() {
                Self::cleanup_restore_path(path);
            }
            return Err(err);
        }

        let staged_wlt_bytes = crate::db::wallet_io::read_file(&staged_wlt)?;
        let orig_wlt =
            Self::read_existing_file_bytes(&final_wlt, "Backup restore .wlt read failed")?;
        let history_lock = if let Some(path) = final_history.as_ref() {
            Some(
                crate::persistence::tx::tx_history_path_lock(path).map_err(|e| {
                    WalletError::InvalidConfig(format!(
                        "Backup restore tx-history lock failed: {e}"
                    ))
                })?,
            )
        } else {
            None
        };
        let history_guard = if let Some(lock) = history_lock.as_ref() {
            Some(lock.lock().map_err(|e| {
                WalletError::InvalidConfig(format!("Backup restore tx-history lock failed: {e}"))
            })?)
        } else {
            None
        };
        let orig_history = if let Some(path) = final_history.as_ref() {
            Self::read_existing_file_bytes(path, "Backup restore history read failed")?
        } else {
            None
        };

        if let Some(bytes) = orig_wlt.as_ref() {
            crate::db::wallet_io::atomic_write_file_private(&wlt_backup, bytes).map_err(|e| {
                WalletError::InvalidConfig(format!("Backup restore .wlt backup failed: {e}"))
            })?;
        }

        if let (Some(bytes), Some(path)) = (orig_history.as_ref(), history_backup.as_ref()) {
            z00z_utils::io::atomic_write_file_private(path, bytes).map_err(|e| {
                WalletError::InvalidConfig(format!("Backup restore history backup failed: {e}"))
            })?;
        }

        let mut mark = WalletRestoreMark::new(
            final_wlt.clone(),
            staged_wlt.clone(),
            wlt_backup.clone(),
            orig_wlt.is_some(),
            final_history.clone(),
            staged_history.clone(),
            history_backup.clone(),
            orig_history.is_some(),
        );
        Self::write_restore_mark(&mark_path, &mark)?;

        if let (Some(final_history), Some(staged_history)) =
            (final_history.as_ref(), staged_history.as_ref())
        {
            mark.stage = RestoreMarkStage::HistoryStep;
            Self::write_restore_mark(&mark_path, &mark)?;
            let staged_history_bytes = z00z_utils::io::read_file(staged_history).map_err(|e| {
                WalletError::InvalidConfig(format!(
                    "Backup restore staged tx-history read failed: {e}"
                ))
            })?;
            let history_commit_err = if restore_atomic_failpoint_is("history_commit") {
                Some("restore atomic failpoint: history_commit".to_string())
            } else {
                z00z_utils::io::atomic_write_file_private(final_history, &staged_history_bytes)
                    .err()
                    .map(|err| err.to_string())
            };
            if let Some(err) = history_commit_err {
                drop(history_guard);
                drop(history_lock);
                let roll_err = self.resume_restore_mark(&mark_path).err();
                Self::cleanup_wlt_temp(&staged_wlt);
                let mut detail = format!("Backup restore history commit failed: {err}");
                if let Some(roll_err) = roll_err {
                    detail.push_str(&format!("; {roll_err}"));
                }
                return Err(WalletError::InvalidConfig(detail));
            }
            if restore_atomic_failpoint_is("crash_after_history") {
                return Err(WalletError::InvalidConfig(
                    "restore atomic failpoint: crash_after_history".to_string(),
                ));
            }
        }

        mark.stage = RestoreMarkStage::WltStep;
        Self::write_restore_mark(&mark_path, &mark)?;
        let wlt_commit_err = if restore_atomic_failpoint_is("wlt_commit") {
            Some("restore atomic failpoint: wlt_commit".to_string())
        } else {
            crate::db::wallet_io::atomic_write_file_private(&final_wlt, &staged_wlt_bytes)
                .err()
                .map(|err| err.to_string())
        };
        if let Some(err) = wlt_commit_err {
            drop(history_guard);
            drop(history_lock);
            let roll_err = self.resume_restore_mark(&mark_path).err();
            Self::cleanup_wlt_temp(&staged_wlt);
            let mut detail = format!("Backup restore .wlt commit failed: {err}");
            if let Some(roll_err) = roll_err {
                detail.push_str(&format!("; {roll_err}"));
            }
            return Err(WalletError::InvalidConfig(detail));
        }

        if restore_atomic_failpoint_is("crash_after_wlt") {
            return Err(WalletError::InvalidConfig(
                "restore atomic failpoint: crash_after_wlt".to_string(),
            ));
        }

        mark.stage = RestoreMarkStage::PublishStep;
        Self::write_restore_mark(&mark_path, &mark)?;
        drop(history_guard);
        drop(history_lock);

        let publish_result = if restore_atomic_failpoint_is("publish") {
            Err(WalletError::InvalidConfig(
                "restore atomic failpoint: publish".to_string(),
            ))
        } else {
            self.publish_restored_wallet_pack(&restore, identity).await
        };
        let restored_wallet_id = match publish_result {
            Ok(wallet_id) => wallet_id,
            Err(err) => {
                let roll_err = self.resume_restore_mark(&mark_path).err();
                let mut detail = format!("Backup restore in-memory publish failed: {err}");
                if let Some(roll_err) = roll_err {
                    detail.push_str(&format!("; {roll_err}"));
                }
                return Err(WalletError::InvalidConfig(detail));
            }
        };

        mark.stage = RestoreMarkStage::Published;
        Self::write_restore_mark(&mark_path, &mark)?;
        Self::cleanup_restore_mark(&mark_path, &mark);

        Ok(restored_wallet_id)
    }

    async fn restore_with_history_atomic(
        &self,
        export_pack: crate::wallet::persistence::WalletExportPack,
        password: &SafePassword,
        wallet_name: Option<&str>,
        identity: &WalletIdentity,
        history_bytes: &[u8],
    ) -> WalletResult<PersistWalletId> {
        self.restore_wallet_pack_atomic(
            export_pack,
            password,
            wallet_name,
            identity,
            Some(history_bytes),
        )
        .await
    }

    async fn publish_restored_wallet_pack(
        &self,
        restore: &WalletRestorePack,
        identity: &WalletIdentity,
    ) -> WalletResult<PersistWalletId> {
        let wallet_id = self.restore_profile(restore.profile.clone()).await?;
        let claimed_assets =
            Self::restore_claimed_assets_from_payloads(restore.owned_assets.as_slice())?;
        self.install_claimed_assets(&wallet_id, claimed_assets)
            .await;

        let mut identities = self.wallet_identities.write().await;
        identities.insert(wallet_id.clone(), identity.clone());
        Ok(wallet_id)
    }

    fn resolve_backup_destination(
        wallet_id: &PersistWalletId,
        settings: &PersistBackupSettings,
        destination: &Option<String>,
        now_ms: u64,
    ) -> WalletResult<(PathBuf, PathBuf)> {
        let dir = if let Some(dest) = destination {
            if dest.ends_with('/') {
                PathBuf::from(dest)
            } else {
                let path = PathBuf::from(dest);
                if path.extension().is_some() {
                    let parent = path
                        .parent()
                        .map(|value| value.to_path_buf())
                        .unwrap_or_else(|| PathBuf::from("."));
                    return Ok((path, parent));
                }
                path
            }
        } else {
            PathBuf::from(&settings.backup_location)
        };

        let safe_wallet = Self::sanitize_backup_wallet_id(wallet_id);
        let tmp_name = format!("backup-{}-tmp-{}.json", safe_wallet, now_ms);
        let tmp_path = dir.join(tmp_name);
        Ok((tmp_path, dir))
    }

    /// Configure backup settings for a wallet.
    ///
    /// If settings are not provided, returns the currently effective settings.
    pub async fn configure_backup_settings(
        &self,
        wallet_id: &PersistWalletId,
        settings: Option<PersistBackupSettings>,
    ) -> WalletResult<RuntimeBackupSettingsResponse> {
        if let Some(settings) = settings {
            self.set_backup_settings(wallet_id, settings).await;
        }

        Ok(RuntimeBackupSettingsResponse {
            settings: self.get_backup_settings(wallet_id).await?,
        })
    }

    /// Create encrypted backup file and canonical tx-history JSONL for a wallet.
    ///
    /// Backup container is created as JSON with encrypted payload.
    pub async fn create_backup(
        &self,
        wallet_id: &PersistWalletId,
        password: SafePassword,
        destination: Option<String>,
    ) -> WalletResult<RuntimeCreateBackupResponse> {
        let rate_limit_started_at = self.backup_create_rate_limit_precheck(wallet_id).await?;
        let mut rate_limit_guard = BackupCreateLimitGuard::new(
            std::sync::Arc::clone(&self.backup_create_limits),
            wallet_id.clone(),
        );

        let result = async {
            if password.reveal().is_empty() {
                return Err(WalletError::InvalidPassword);
            }

            let settings = self.get_backup_settings(wallet_id).await?;
            let now_ms = self.now_ms();
            let (tmp_path, dir) =
                Self::resolve_backup_destination(wallet_id, &settings, &destination, now_ms)?;
            Self::ensure_backup_dir(&dir)?;

            let identity = self.resolve_persisted_wallet_identity(wallet_id).await?;

            let export_pack = self.build_backup_export_pack(wallet_id, &password).await?;
            let (history_jsonl, forensic_history) =
                self.collect_tx_history_jsonl(wallet_id).await?;

            let exporter = BackupExporterImpl::new_with_forensic_history(
                wallet_id.0.clone(),
                identity.network,
                identity.chain,
                export_pack,
                forensic_history,
                SystemTimeProvider,
                SystemRngProvider,
            );

            let metadata = exporter
                .export_with_history_bytes(
                    tmp_path.to_string_lossy().as_ref(),
                    &password,
                    &history_jsonl,
                )
                .map_err(|e| WalletError::InvalidConfig(format!("Backup export failed: {e}")))?;

            let safe_wallet = Self::sanitize_backup_wallet_id(wallet_id);
            let final_name = format!("backup-{}-{}.json", safe_wallet, metadata.created_at);
            let final_path = dir.join(final_name);

            rename_file(&tmp_path, &final_path).map_err(|e| {
                WalletError::InvalidConfig(format!("Failed to finalize backup file: {e}"))
            })?;

            Ok(RuntimeCreateBackupResponse {
                status: crate::rpc::types::common::RuntimeOperationStatus {
                    success: true,
                    message: "OK".to_string(),
                },
                backup_path: final_path.to_string_lossy().to_string(),
                encrypted: true,
            })
        }
        .await;

        self.backup_create_rate_limit_finish(wallet_id, rate_limit_started_at, result.is_ok())
            .await;
        rate_limit_guard.disarm();

        result
    }

    /// List available backup files for a wallet.
    ///
    /// Supports cursor-based pagination over backup file names.
    pub async fn list_backups(
        &self,
        wallet_id: &PersistWalletId,
        cursor: Option<String>,
        limit: Option<u32>,
    ) -> WalletResult<RuntimeListBackupsResponse> {
        let settings = self.get_backup_settings(wallet_id).await?;
        let dir = PathBuf::from(&settings.backup_location);

        let mut sorted = match read_dir(&dir) {
            Ok(entries) => entries,
            Err(_) => {
                return Ok(RuntimeListBackupsResponse {
                    items: Vec::new(),
                    next_cursor: None,
                    has_more: false,
                    total_count: Some(0),
                });
            }
        };

        sorted.sort_by(|a, b| a.file_name().cmp(&b.file_name()));

        let importer = BackupImporterImpl::new();
        let mut items = Vec::new();
        let mut started = cursor.is_none();
        let max_items = limit.unwrap_or(50).clamp(1, 200) as usize;
        let mut next_cursor = None;
        let mut total_count = 0usize;

        for path in sorted {
            let file_name = match path.file_name().and_then(|name| name.to_str()) {
                Some(value) => value.to_string(),
                None => continue,
            };

            let metadata = match importer.read_metadata(path.to_string_lossy().as_ref()) {
                Ok(value) => value,
                Err(_) => continue,
            };

            if metadata.wallet_id != wallet_id.0 {
                continue;
            }

            total_count = total_count.saturating_add(1);

            if let Some(expected) = &cursor {
                if !started {
                    if &file_name == expected {
                        started = true;
                    }
                    continue;
                }
            }

            if items.len() >= max_items {
                next_cursor = Some(file_name);
                break;
            }

            let size_bytes = z00z_utils::io::file_len(&path).unwrap_or(0);
            items.push(PersistBackupInfo {
                id: file_name,
                wallet_id: PersistWalletId(wallet_id.0.clone()),
                created_at: metadata.created_at,
                size_bytes,
                encrypted: true,
            });
        }

        Ok(RuntimeListBackupsResponse {
            items,
            next_cursor: next_cursor.clone(),
            has_more: next_cursor.is_some(),
            total_count: Some(total_count),
        })
    }

    /// Restore wallet data from an encrypted backup file.
    ///
    /// Returns imported wallet id on successful decryption and parsing.
    pub async fn restore_backup(
        &self,
        backup_path: String,
        password: SafePassword,
        wallet_name: Option<String>,
    ) -> WalletResult<RuntimeRestoreBackupResponse> {
        self.restore_backup_with_mode(
            backup_path,
            password,
            wallet_name,
            crate::backup::ForensicImportMode::WalletPlusHistory,
        )
        .await
    }

    /// Restore a backup with an explicit forensic import mode.
    pub async fn restore_backup_with_mode(
        &self,
        backup_path: String,
        password: SafePassword,
        wallet_name: Option<String>,
        mode: crate::backup::ForensicImportMode,
    ) -> WalletResult<RuntimeRestoreBackupResponse> {
        if password.reveal().is_empty() {
            return Err(WalletError::InvalidPassword);
        }

        let importer = BackupImporterImpl::new();
        let imported = importer
            .import_with_mode(&backup_path, &password, mode)
            .map_err(|e| {
                let message = e.to_string();
                if message.to_lowercase().contains("decryption failed") {
                    WalletError::InvalidPassword
                } else {
                    WalletError::InvalidConfig(format!("Backup restore failed: {message}"))
                }
            })?;

        let imported_wallet_id = PersistWalletId(imported.wallet_id.clone());

        let wallet_id = match mode {
            crate::backup::ForensicImportMode::WalletOnly => {
                let export_pack = imported.export_pack.ok_or_else(|| {
                    WalletError::InvalidConfig(
                        "Backup restore payload missing export pack".to_string(),
                    )
                })?;

                if imported.chain.trim().is_empty() {
                    return Err(WalletError::InvalidConfig(
                        "Backup restore requires a chain-bound backup payload".to_string(),
                    ));
                }

                let identity = WalletIdentity {
                    network: imported.network,
                    chain: imported.chain,
                };
                self.restore_wallet_export_pack(
                    export_pack,
                    &password,
                    wallet_name.as_deref(),
                    &identity,
                )
                .await?
            }
            crate::backup::ForensicImportMode::WalletPlusHistory => {
                let export_pack = imported.export_pack.ok_or_else(|| {
                    WalletError::InvalidConfig(
                        "Backup restore payload missing export pack".to_string(),
                    )
                })?;

                if imported.chain.trim().is_empty() {
                    return Err(WalletError::InvalidConfig(
                        "Backup restore requires a chain-bound backup payload".to_string(),
                    ));
                }

                let identity = WalletIdentity {
                    network: imported.network,
                    chain: imported.chain,
                };
                self.restore_with_history_atomic(
                    export_pack,
                    &password,
                    wallet_name.as_deref(),
                    &identity,
                    &imported.transactions,
                )
                .await?
            }
            crate::backup::ForensicImportMode::TxHistoryOnly => imported_wallet_id,
        };

        if matches!(mode, crate::backup::ForensicImportMode::TxHistoryOnly) {
            self.write_tx_history_jsonl_bytes(&wallet_id, &imported.transactions)?;
        }

        Ok(RuntimeRestoreBackupResponse {
            status: crate::rpc::types::common::RuntimeOperationStatus {
                success: true,
                message: "OK".to_string(),
            },
            wallet_id,
        })
    }
}
