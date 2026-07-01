//! File-backed scan state storage implementation.

use super::{ScanState, ScanStorage, ScanStorageError, ScanStorageResult};
use std::path::PathBuf;
use z00z_utils::{
    codec::{Codec, JsonCodec},
    io::{read_file, remove_file, write_file},
    time::TimeProvider,
};

/// File-backed scan state storage implementation.
///
/// Stores scan state as a single JSON file: `{base_dir}/scan_state.json`
///
/// Generic over:
/// - `T`: TimeProvider for timestamp generation
pub struct ScanStorageImpl<T: TimeProvider> {
    state_path: PathBuf,
    time_provider: T,
}

impl<T: TimeProvider> ScanStorageImpl<T> {
    /// Creates a new file-backed scan state storage.
    ///
    /// # Arguments
    ///
    /// * `base_dir` - Base directory for scan state storage
    /// * `time_provider` - Time provider for timestamps
    pub fn new(base_dir: impl Into<PathBuf>, time_provider: T) -> Self {
        let state_path = base_dir.into().join("scan_state.json");
        Self {
            state_path,
            time_provider,
        }
    }
}

impl<T: TimeProvider> ScanStorage for ScanStorageImpl<T> {
    fn save(&mut self, state: ScanState) -> ScanStorageResult<()> {
        let codec = JsonCodec;
        let data = codec
            .serialize(&state)
            .map_err(|e| ScanStorageError::Serialization(e.to_string()))?;

        write_file(&self.state_path, &data).map_err(ScanStorageError::Io)?;

        Ok(())
    }

    fn load(&self) -> ScanStorageResult<ScanState> {
        let data = read_file(&self.state_path).map_err(|e| match e {
            z00z_utils::io::IoError::Io(io_err)
                if io_err.kind() == std::io::ErrorKind::NotFound =>
            {
                ScanStorageError::NotFound
            }
            _ => ScanStorageError::Io(e),
        })?;

        let codec = JsonCodec;
        let state = codec
            .deserialize(&data)
            .map_err(|e| ScanStorageError::Serialization(e.to_string()))?;

        Ok(state)
    }

    fn update_last_scanned(&mut self, height: u64, hash: String) -> ScanStorageResult<()> {
        let mut state = self.load().or_else(|e| match e {
            ScanStorageError::NotFound => {
                // Initialize new state if not exists
                Ok(ScanState {
                    last_scanned_height: 0,
                    last_scanned_hash: String::new(),
                    last_scan_timestamp_ms: 0,
                    is_scanning: false,
                })
            }
            _ => Err(e),
        })?;

        state.last_scanned_height = height;
        state.last_scanned_hash = hash;
        state.last_scan_timestamp_ms = self.time_provider.compat_unix_timestamp_millis();

        self.save(state)
    }

    fn set_scanning(&mut self, is_scanning: bool) -> ScanStorageResult<()> {
        let mut state = self.load()?;
        state.is_scanning = is_scanning;
        self.save(state)
    }

    fn reset(&mut self) -> ScanStorageResult<()> {
        remove_file(&self.state_path).map_err(|e| match e {
            z00z_utils::io::IoError::Io(io_err)
                if io_err.kind() == std::io::ErrorKind::NotFound =>
            {
                ScanStorageError::NotFound
            }
            _ => ScanStorageError::Io(e),
        })?;
        Ok(())
    }
}

#[cfg(all(test, not(target_arch = "wasm32")))]
mod tests {
    use super::*;
    use z00z_utils::time::MockTimeProvider;

    #[test]
    fn test_save_load_roundtrip() {
        let dir = tempfile::tempdir().unwrap();
        let time = MockTimeProvider::from_unix_secs(1);
        let mut store = ScanStorageImpl::new(dir.path(), time);

        let state = ScanState {
            last_scanned_height: 100,
            last_scanned_hash: "hash_100".to_string(),
            last_scan_timestamp_ms: 1_234_567_890_000,
            is_scanning: false,
        };

        store.save(state.clone()).unwrap();

        let loaded = store.load().unwrap();
        assert_eq!(loaded, state);
    }

    #[test]
    fn test_load_uninitialized_fails() {
        let dir = tempfile::tempdir().unwrap();
        let time = MockTimeProvider::from_unix_secs(1);
        let store = ScanStorageImpl::new(dir.path(), time);

        let result = store.load();
        assert!(matches!(result, Err(ScanStorageError::NotFound)));
    }

    #[test]
    fn test_update_last_scanned_updates() {
        let dir = tempfile::tempdir().unwrap();
        let time = MockTimeProvider::from_unix_secs(1);
        let mut store = ScanStorageImpl::new(dir.path(), time);

        // Initialize state
        let initial = ScanState {
            last_scanned_height: 50,
            last_scanned_hash: "hash_50".to_string(),
            last_scan_timestamp_ms: 1000,
            is_scanning: false,
        };
        store.save(initial).unwrap();

        // Update
        store
            .update_last_scanned(100, "hash_100".to_string())
            .unwrap();

        let loaded = store.load().unwrap();
        assert_eq!(loaded.last_scanned_height, 100);
        assert_eq!(loaded.last_scanned_hash, "hash_100");
        assert!(loaded.last_scan_timestamp_ms > 0); // TimeProvider updated
    }

    #[test]
    fn test_update_last_scanned_creates() {
        let dir = tempfile::tempdir().unwrap();
        let time = MockTimeProvider::from_unix_secs(1);
        let mut store = ScanStorageImpl::new(dir.path(), time);

        // Update without prior initialization
        store
            .update_last_scanned(42, "hash_42".to_string())
            .unwrap();

        let loaded = store.load().unwrap();
        assert_eq!(loaded.last_scanned_height, 42);
        assert_eq!(loaded.last_scanned_hash, "hash_42");
    }

    #[test]
    fn test_set_scanning_toggles_flag() {
        let dir = tempfile::tempdir().unwrap();
        let time = MockTimeProvider::from_unix_secs(1);
        let mut store = ScanStorageImpl::new(dir.path(), time);

        let state = ScanState {
            last_scanned_height: 10,
            last_scanned_hash: "hash_10".to_string(),
            last_scan_timestamp_ms: 1000,
            is_scanning: false,
        };
        store.save(state).unwrap();

        store.set_scanning(true).unwrap();
        let loaded = store.load().unwrap();
        assert!(loaded.is_scanning);

        store.set_scanning(false).unwrap();
        let loaded = store.load().unwrap();
        assert!(!loaded.is_scanning);
    }

    #[test]
    fn test_reset_removes_state() {
        let dir = tempfile::tempdir().unwrap();
        let time = MockTimeProvider::from_unix_secs(1);
        let mut store = ScanStorageImpl::new(dir.path(), time);

        let state = ScanState {
            last_scanned_height: 100,
            last_scanned_hash: "hash_100".to_string(),
            last_scan_timestamp_ms: 1234567890,
            is_scanning: false,
        };
        store.save(state).unwrap();

        store.reset().unwrap();

        let result = store.load();
        assert!(matches!(result, Err(ScanStorageError::NotFound)));
    }

    #[test]
    fn test_reset_uninitialized_fails() {
        let dir = tempfile::tempdir().unwrap();
        let time = MockTimeProvider::from_unix_secs(1);
        let mut store = ScanStorageImpl::new(dir.path(), time);

        let result = store.reset();
        assert!(matches!(result, Err(ScanStorageError::NotFound)));
    }

    #[test]
    fn test_scan_resume_ckpt() {
        let dir = tempfile::tempdir().unwrap();

        let time_a = MockTimeProvider::from_unix_secs(1);
        let mut store_a = ScanStorageImpl::new(dir.path(), time_a);
        store_a
            .update_last_scanned(777, "block_hash_777".to_string())
            .unwrap();

        let time_b = MockTimeProvider::from_unix_secs(2);
        let store_b = ScanStorageImpl::new(dir.path(), time_b);
        let loaded = store_b.load().unwrap();

        assert_eq!(loaded.last_scanned_height, 777);
        assert_eq!(loaded.last_scanned_hash, "block_hash_777");
    }
}
