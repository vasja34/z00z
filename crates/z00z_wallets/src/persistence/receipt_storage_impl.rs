//! File-backed receipt storage implementation.

use super::{Receipt, ReceiptStorage, ReceiptStorageError, ReceiptStorageResult};
use std::path::PathBuf;
use z00z_utils::{
    codec::{Codec, JsonCodec},
    io::{read_dir, read_file, remove_file, write_file},
};

/// File-backed receipt storage implementation.
///
/// Stores receipts as individual JSON files in a base directory:
/// `{base_dir}/{receipt_id}.json`
pub struct ReceiptStorageImpl {
    base_dir: PathBuf,
}

impl ReceiptStorageImpl {
    /// Creates a new file-backed receipt storage.
    ///
    /// # Arguments
    ///
    /// * `base_dir` - Base directory for receipt storage
    pub fn new(base_dir: impl Into<PathBuf>) -> Self {
        Self {
            base_dir: base_dir.into(),
        }
    }

    /// Validates receipt_id to prevent path traversal attacks.
    fn validate_receipt_id(receipt_id: &str) -> ReceiptStorageResult<()> {
        if receipt_id.is_empty()
            || receipt_id.contains('/')
            || receipt_id.contains('\\')
            || receipt_id.contains("..")
        {
            return Err(ReceiptStorageError::InvalidReceiptId(
                receipt_id.to_string(),
            ));
        }
        Ok(())
    }

    fn receipt_path(&self, receipt_id: &str) -> ReceiptStorageResult<PathBuf> {
        Self::validate_receipt_id(receipt_id)?;
        Ok(self.base_dir.join(format!("{receipt_id}.json")))
    }
}

impl ReceiptStorage for ReceiptStorageImpl {
    fn put(&mut self, receipt: Receipt) -> ReceiptStorageResult<()> {
        let path = self.receipt_path(&receipt.receipt_id)?;

        let codec = JsonCodec;
        let data = codec
            .serialize(&receipt)
            .map_err(|e| ReceiptStorageError::Serialization(e.to_string()))?;

        write_file(&path, &data).map_err(ReceiptStorageError::Io)?;
        Ok(())
    }

    fn get(&self, receipt_id: &str) -> ReceiptStorageResult<Receipt> {
        let path = self.receipt_path(receipt_id)?;

        let data = read_file(&path).map_err(|e| match e {
            z00z_utils::io::IoError::Io(io_err)
                if io_err.kind() == std::io::ErrorKind::NotFound =>
            {
                ReceiptStorageError::NotFound(receipt_id.to_string())
            }
            _ => ReceiptStorageError::Io(e),
        })?;

        let codec = JsonCodec;
        let receipt = codec
            .deserialize(&data)
            .map_err(|e| ReceiptStorageError::Serialization(e.to_string()))?;

        Ok(receipt)
    }

    fn list(&self) -> ReceiptStorageResult<Vec<Receipt>> {
        let mut receipts = Vec::new();

        let paths = read_dir(&self.base_dir).map_err(ReceiptStorageError::Io)?;

        for path in paths {
            // Skip non-JSON files
            if path.extension().and_then(|s| s.to_str()) != Some("json") {
                continue;
            }

            // Extract receipt_id from filename
            let receipt_id = path
                .file_stem()
                .and_then(|s| s.to_str())
                .ok_or_else(|| ReceiptStorageError::Database("Invalid filename".to_string()))?;

            // Read and deserialize
            if let Ok(receipt) = self.get(receipt_id) {
                receipts.push(receipt);
            }
        }

        Ok(receipts)
    }

    fn find_by_tx(&self, tx_hash: &str) -> ReceiptStorageResult<Vec<Receipt>> {
        let all_receipts = self.list()?;

        let filtered: Vec<Receipt> = all_receipts
            .into_iter()
            .filter(|r| r.tx_hash == tx_hash)
            .collect();

        Ok(filtered)
    }

    fn delete(&mut self, receipt_id: &str) -> ReceiptStorageResult<()> {
        let path = self.receipt_path(receipt_id)?;

        remove_file(&path).map_err(|e| match e {
            z00z_utils::io::IoError::Io(io_err)
                if io_err.kind() == std::io::ErrorKind::NotFound =>
            {
                ReceiptStorageError::NotFound(receipt_id.to_string())
            }
            _ => ReceiptStorageError::Io(e),
        })?;

        Ok(())
    }
}

#[cfg(all(test, not(target_arch = "wasm32")))]
mod tests {
    use super::*;

    fn create_test_receipt(id: &str, tx_hash: &str) -> Receipt {
        Receipt {
            receipt_id: id.to_string(),
            tx_hash: tx_hash.to_string(),
            amount: 1000,
            recipient: "test_recipient".to_string(),
            timestamp_ms: 1234567890,
            proof: None,
        }
    }

    #[test]
    fn test_put_get_roundtrip() {
        let dir = tempfile::tempdir().unwrap();
        let mut store = ReceiptStorageImpl::new(dir.path());

        let receipt = create_test_receipt("receipt1", "tx_hash_1");
        store.put(receipt.clone()).unwrap();

        let loaded = store.get("receipt1").unwrap();
        assert_eq!(loaded, receipt);
    }

    #[test]
    fn test_delete_removes_receipt() {
        let dir = tempfile::tempdir().unwrap();
        let mut store = ReceiptStorageImpl::new(dir.path());

        let receipt = create_test_receipt("receipt2", "tx_hash_2");
        store.put(receipt.clone()).unwrap();

        store.delete("receipt2").unwrap();

        let result = store.get("receipt2");
        assert!(matches!(result, Err(ReceiptStorageError::NotFound(_))));
    }

    #[test]
    fn test_delete_missing_not_found() {
        let dir = tempfile::tempdir().unwrap();
        let mut store = ReceiptStorageImpl::new(dir.path());

        let result = store.delete("nonexistent");
        assert!(matches!(result, Err(ReceiptStorageError::NotFound(_))));
    }

    #[test]
    fn test_find_tx_filters_correctly() {
        let dir = tempfile::tempdir().unwrap();
        let mut store = ReceiptStorageImpl::new(dir.path());

        let r1 = create_test_receipt("receipt1", "tx_hash_1");
        let r2 = create_test_receipt("receipt2", "tx_hash_2");
        let r3 = create_test_receipt("receipt3", "tx_hash_1");

        store.put(r1.clone()).unwrap();
        store.put(r2.clone()).unwrap();
        store.put(r3.clone()).unwrap();

        let found = store.find_by_tx("tx_hash_1").unwrap();
        assert_eq!(found.len(), 2);
        assert!(found.iter().any(|r| r.receipt_id == "receipt1"));
        assert!(found.iter().any(|r| r.receipt_id == "receipt3"));
    }

    #[test]
    fn test_invalid_receipt_id_rejected() {
        let dir = tempfile::tempdir().unwrap();
        let mut store = ReceiptStorageImpl::new(dir.path());

        let invalid_ids = vec!["../etc/passwd", "/absolute/path", "contains\\backslash", ""];

        for id in invalid_ids {
            let receipt = create_test_receipt(id, "tx_hash");
            let result = store.put(receipt);
            assert!(matches!(
                result,
                Err(ReceiptStorageError::InvalidReceiptId(_))
            ));
        }
    }

    #[test]
    fn test_list_returns_all_receipts() {
        let dir = tempfile::tempdir().unwrap();
        let mut store = ReceiptStorageImpl::new(dir.path());

        let r1 = create_test_receipt("receipt1", "tx1");
        let r2 = create_test_receipt("receipt2", "tx2");
        let r3 = create_test_receipt("receipt3", "tx3");

        store.put(r1.clone()).unwrap();
        store.put(r2.clone()).unwrap();
        store.put(r3.clone()).unwrap();

        let all_receipts = store.list().unwrap();
        assert_eq!(all_receipts.len(), 3);

        let ids: Vec<String> = all_receipts.iter().map(|r| r.receipt_id.clone()).collect();
        assert!(ids.contains(&"receipt1".to_string()));
        assert!(ids.contains(&"receipt2".to_string()));
        assert!(ids.contains(&"receipt3".to_string()));
    }
}
