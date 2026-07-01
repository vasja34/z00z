impl WalletService {
    pub(crate) fn apply_hardening() {
        #[cfg(all(feature = "os_hardening", not(target_arch = "wasm32")))]
        {
            let report = z00z_utils::os_hardening::apply_best_effort();
            z00z_utils::logger::Logger::info(
                &z00z_utils::logger::TracingLogger,
                &format!(
                    "wallet os hardening applied: core_dumps_disabled={}, non_dumpable={}, notes={}",
                    report.core_dumps_disabled,
                    report.non_dumpable,
                    report.notes.len()
                ),
            );
            for note in report.notes {
                z00z_utils::logger::Logger::debug(
                    &z00z_utils::logger::TracingLogger,
                    &format!("wallet os hardening note: {}", note),
                );
            }
        }
    }

    pub(crate) const MAX_GAP_SCAN_ADDRESSES: u32 = 1_000_000;

    pub(crate) fn default_entropy() -> Arc<dyn WalletEntropy> {
        Arc::new(WalletEntropyFromRngProvider::new(SystemRngProvider))
    }

    pub(crate) fn default_output_dir() -> PathBuf {
        resolve_wallet_output_dir()
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub(crate) fn cleanup_lock_file_best_effort(&self, wallet_id: &PersistWalletId) {
        use fs2::FileExt as _;
        use z00z_utils::io::{remove_file, File};

        let wlt_path = self.wlt_file_path(wallet_id);

        // Avoid deleting a lock file that is currently held by this process.
        if crate::db::is_lock_held_local(&wlt_path) {
            return;
        }

        let lock_path = {
            let mut os = wlt_path.as_os_str().to_os_string();
            os.push(".lock");
            PathBuf::from(os)
        };

        if !lock_path.exists() {
            return;
        }

        let lock_file = match File::options()
            .create(true)
            .read(true)
            .write(true)
            .truncate(false)
            .open(&lock_path)
        {
            Ok(file) => file,
            Err(_) => return,
        };

        if lock_file.try_lock_exclusive().is_err() {
            return;
        }

        let _ = lock_file.unlock();
        let _ = remove_file(&lock_path);
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub(crate) fn cleanup_stale_locks(&self) {
        use z00z_utils::io::read_dir;

        let Ok(entries) = read_dir(&self.output_dir) else {
            return;
        };

        for entry in entries {
            let Some(name) = entry.file_name().and_then(|s| s.to_str()) else {
                continue;
            };

            if !name.starts_with("wallet_") || !name.ends_with(".wlt.lock") {
                continue;
            }

            let Some(stem) = name.strip_suffix(".lock") else {
                continue;
            };

            let wallet_id = PersistWalletId(stem.to_string());
            self.cleanup_lock_file_best_effort(&wallet_id);
        }
    }

    pub(crate) fn output_dir(&self) -> &Path {
        &self.output_dir
    }

    pub(crate) fn fill_entropy(&self, dest: &mut [u8]) {
        self.entropy.fill_bytes(dest);
    }
}
