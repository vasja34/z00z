//! File-backed wallet metadata store.

use super::{WalletStorage, WalletStorageError, WalletStorageResult};
use crate::wallet::WalletRecord;
use std::path::{Path, PathBuf};
use z00z_utils::io::{
    create_dir_all, load_json_bounded, read_dir, remove_file, save_json, IoError,
};

const MAX_WALLET_FILE_SIZE: u64 = 10 * 1024 * 1024;

/// File-backed wallet storage implementation.
///
/// Stores one JSON file per wallet, keyed by wallet_id.
#[derive(Debug, Clone)]
pub struct WalletStorageImpl {
    base_dir: PathBuf,
}

impl WalletStorageImpl {
    /// Create a new wallet storage rooted at `base_dir`.
    pub fn new(base_dir: impl Into<PathBuf>) -> Self {
        Self {
            base_dir: base_dir.into(),
        }
    }

    fn validate_wallet_id(wallet_id: &str) -> WalletStorageResult<()> {
        if wallet_id.is_empty()
            || wallet_id.contains('/')
            || wallet_id.contains('\\')
            || wallet_id.contains("..")
        {
            return Err(WalletStorageError::InvalidWalletId(wallet_id.to_string()));
        }
        Ok(())
    }

    fn wallet_path(&self, wallet_id: &str) -> WalletStorageResult<PathBuf> {
        Self::validate_wallet_id(wallet_id)?;
        Ok(self.base_dir.join(format!("{wallet_id}.json")))
    }

    fn is_not_found(err: &IoError) -> bool {
        matches!(err, IoError::Io(e) if e.kind() == std::io::ErrorKind::NotFound)
    }

    fn ensure_base_dir(&self) -> WalletStorageResult<()> {
        create_dir_all(&self.base_dir)?;
        Ok(())
    }

    fn list_wallet_files(&self) -> Result<Vec<PathBuf>, IoError> {
        read_dir(&self.base_dir)
    }

    fn is_wallet_metadata_file(path: &Path) -> bool {
        path.extension().and_then(|s| s.to_str()) == Some("json")
    }
}

impl WalletStorage for WalletStorageImpl {
    fn save(&mut self, record: WalletRecord) -> WalletStorageResult<()> {
        self.ensure_base_dir()?;

        let wallet_id = record.wallet_id().to_persist_wallet_id().0;
        Self::validate_wallet_id(&wallet_id)?;
        let path = self.wallet_path(&wallet_id)?;

        if self.exists(&wallet_id) {
            return Err(WalletStorageError::AlreadyExists(wallet_id));
        }

        save_json(path, &record)?;
        Ok(())
    }

    fn load(&self, wallet_id: &str) -> WalletStorageResult<WalletRecord> {
        self.ensure_base_dir()?;

        Self::validate_wallet_id(wallet_id)?;
        let path = self.wallet_path(wallet_id)?;

        match load_json_bounded(path, MAX_WALLET_FILE_SIZE) {
            Ok(value) => Ok(value),
            Err(e) if Self::is_not_found(&e) => {
                Err(WalletStorageError::NotFound(wallet_id.to_string()))
            }
            Err(e) => Err(WalletStorageError::Io(e)),
        }
    }

    fn list(&self) -> WalletStorageResult<Vec<WalletRecord>> {
        self.ensure_base_dir()?;

        let mut items = Vec::new();
        for path in self.list_wallet_files()? {
            if !Self::is_wallet_metadata_file(&path) {
                continue;
            }

            let record: WalletRecord = load_json_bounded(&path, MAX_WALLET_FILE_SIZE)?;
            items.push(record);
        }

        Ok(items)
    }

    fn delete(&mut self, wallet_id: &str) -> WalletStorageResult<()> {
        self.ensure_base_dir()?;

        Self::validate_wallet_id(wallet_id)?;
        let path = self.wallet_path(wallet_id)?;

        match remove_file(path) {
            Ok(()) => Ok(()),
            Err(e) if Self::is_not_found(&e) => {
                Err(WalletStorageError::NotFound(wallet_id.to_string()))
            }
            Err(e) => Err(WalletStorageError::Io(e)),
        }
    }

    fn exists(&self, wallet_id: &str) -> bool {
        let path = match self.wallet_path(wallet_id) {
            Ok(path) => path,
            Err(_) => return false,
        };

        match load_json_bounded::<WalletRecord>(&path, MAX_WALLET_FILE_SIZE) {
            Ok(_) => true,
            Err(e) if Self::is_not_found(&e) => false,
            Err(_e) => false,
        }
    }
}

#[cfg(all(test, not(target_arch = "wasm32")))]
mod tests {
    use super::*;
    use crate::wallet::{ChainId, WalletId, WalletKernel, WalletSystemMetadata, WalletUserFields};

    fn sample_record(wallet_id: WalletId) -> WalletRecord {
        let kernel = WalletKernel::new(wallet_id, ChainId::TESTNET);
        WalletRecord::new(
            kernel,
            WalletUserFields {
                wallet_name: "Test Wallet".to_string(),
                memo: Some("memo".to_string()),
            },
            WalletSystemMetadata {
                created_at: 123,
                updated_at: 456,
            },
        )
    }

    #[test]
    fn test_save_load_roundtrip() {
        let dir = tempfile::tempdir().unwrap();
        let mut store = WalletStorageImpl::new(dir.path());

        let record = sample_record(WalletId([1u8; 32]));
        let wallet_id = record.wallet_id().to_persist_wallet_id().0;
        store.save(record.clone()).unwrap();

        let loaded = store.load(&wallet_id).unwrap();
        assert_eq!(loaded, record);

        // TempDir cleans up automatically.
    }

    #[test]
    fn test_save_duplicate_fails() {
        let dir = tempfile::tempdir().unwrap();
        let mut store = WalletStorageImpl::new(dir.path());

        let record = sample_record(WalletId([2u8; 32]));
        store.save(record.clone()).unwrap();
        let err = store.save(record).unwrap_err();
        assert!(matches!(err, WalletStorageError::AlreadyExists(_)));

        // TempDir cleans up automatically.
    }

    #[test]
    fn test_delete_missing_not_found() {
        let dir = tempfile::tempdir().unwrap();
        let mut store = WalletStorageImpl::new(dir.path());

        let err = store.delete("missing").unwrap_err();
        assert!(matches!(err, WalletStorageError::NotFound(_)));
    }

    #[test]
    fn test_invalid_wallet_id_rejected() {
        let dir = tempfile::tempdir().unwrap();
        let store = WalletStorageImpl::new(dir.path());

        let err = store.load("../bad").unwrap_err();
        assert!(matches!(err, WalletStorageError::InvalidWalletId(_)));
        assert!(!store.exists("../bad"));
    }

    #[test]
    fn test_list_returns_all_wallets() {
        let dir = tempfile::tempdir().unwrap();
        let mut store = WalletStorageImpl::new(dir.path());

        let r1 = sample_record(WalletId([10u8; 32]));
        let r2 = sample_record(WalletId([11u8; 32]));
        let id1 = r1.wallet_id().to_persist_wallet_id().0;
        let id2 = r2.wallet_id().to_persist_wallet_id().0;
        store.save(r1).unwrap();
        store.save(r2).unwrap();

        let mut wallets = store.list().unwrap();
        wallets.sort_by_key(|a| a.wallet_id().to_hex());

        assert_eq!(wallets.len(), 2);
        assert_eq!(wallets[0].wallet_id().to_persist_wallet_id().0, id1);
        assert_eq!(wallets[1].wallet_id().to_persist_wallet_id().0, id2);

        // TempDir cleans up automatically.
    }
}
