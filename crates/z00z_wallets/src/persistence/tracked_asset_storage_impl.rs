//! AssetStorage implementation.
//!
//! Detection and scan orchestration stay owned by `core/receiver/*`.
//! Phase 037 keeps wallet-native claimed-asset persistence as the canonical
//! receive target; this file remains a separate asset storage adapter for
//! storage metadata flows and does not define a third receive persistence
//! layer.

use super::{AssetStorage, AssetStorageError};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use z00z_core::assets::Asset;
use z00z_crypto::expert::encoding::to_hex;
use z00z_utils::{
    codec::Codec,
    codec::JsonCodec,
    config::{ConfigSource, EnvConfig},
    io::{create_dir_all, path_exists, read_dir, read_file, write_file},
};

type Result<T> = std::result::Result<T, AssetStorageError>;
const FAIL_ASSET_SAVE_ENV: &str = "Z00Z_FAIL_ASSET_SAVE";

/// Internal storage record with wallet-local spending metadata.
///
/// This is an INTERNAL implementation detail, not exposed in public API.
/// It wraps core `Asset` with wallet-specific spending tracking and must not
/// be treated as authoritative protocol spent state. The protocol consumed
/// state is carried by checkpointed storage roots, asset-leaf membership, and
/// spent deltas.
#[derive(Debug, Clone, Serialize, Deserialize)]
struct AssetRecord {
    wallet_id: String,

    /// Core asset (from z00z_core)
    asset: Asset,

    /// Wallet-local spending status flag.
    is_spent: bool,

    /// Wallet-local block height when this record was marked spent.
    spent_at_height: Option<u64>,
}

/// In-memory asset storage implementation using HashMap.
///
/// Persistent deployments use per-wallet JSON files behind the same trait.
#[derive(Debug)]
pub struct AssetStorageImpl {
    /// Database path (":memory:" for in-memory)
    db_path: PathBuf,

    /// In-memory storage with optional file-backed snapshots per wallet.
    /// Key: asset_id.to_string(), Value: JSON serialized AssetRecord
    storage: std::collections::HashMap<String, Vec<u8>>,
}

impl AssetStorageImpl {
    fn is_mem(&self) -> bool {
        self.db_path.to_string_lossy() == ":memory:"
    }

    fn mk_key(wallet_id: &str, asset_id: &[u8; 32]) -> String {
        format!("{}:{}", wallet_id, to_hex(asset_id))
    }

    fn key_prefix(wallet_id: &str) -> String {
        format!("{wallet_id}:")
    }

    fn wallet_from_key(key: &str) -> Option<&str> {
        key.split_once(':').map(|(wallet_id, _)| wallet_id)
    }

    fn store_dir(&self) -> Option<PathBuf> {
        if self.is_mem() {
            return None;
        }
        Some(self.db_path.with_extension("wallets"))
    }

    fn wallet_file(&self, wallet_id: &str) -> Option<PathBuf> {
        self.store_dir()
            .map(|dir| dir.join(format!("{wallet_id}.json")))
    }

    fn save_map(&self) -> Result<()> {
        if self.is_mem() {
            return Ok(());
        }
        if EnvConfig.get(FAIL_ASSET_SAVE_ENV).ok().flatten().as_deref() == Some("1") {
            return Err(AssetStorageError::Database(
                "injected save fail".to_string(),
            ));
        }

        let Some(dir) = self.store_dir() else {
            return Ok(());
        };
        create_dir_all(&dir).map_err(|e| AssetStorageError::Database(e.to_string()))?;

        let mut by_wallet: std::collections::HashMap<
            String,
            std::collections::HashMap<String, Vec<u8>>,
        > = std::collections::HashMap::new();
        for (key, value) in &self.storage {
            let Some(wallet_id) = Self::wallet_from_key(key) else {
                continue;
            };
            let entry = by_wallet.entry(wallet_id.to_string()).or_default();
            entry.insert(key.clone(), value.clone());
        }

        for (wallet_id, wallet_map) in by_wallet {
            let bytes = JsonCodec
                .serialize(&wallet_map)
                .map_err(|e| AssetStorageError::Serialization(e.to_string()))?;
            let file = self.wallet_file(&wallet_id).ok_or_else(|| {
                AssetStorageError::Database("wallet file path missing".to_string())
            })?;
            write_file(&file, &bytes).map_err(|e| AssetStorageError::Database(e.to_string()))?;
        }

        Ok(())
    }

    fn load_map(&mut self) -> Result<()> {
        if self.is_mem() {
            return Ok(());
        }

        let Some(dir) = self.store_dir() else {
            return Ok(());
        };
        if !path_exists(&dir).map_err(|e| AssetStorageError::Database(e.to_string()))? {
            return Ok(());
        }

        self.storage.clear();
        for path in read_dir(&dir).map_err(|e| AssetStorageError::Database(e.to_string()))? {
            if path.extension().and_then(|ext| ext.to_str()) != Some("json") {
                continue;
            }
            let bytes = read_file(&path).map_err(|e| AssetStorageError::Database(e.to_string()))?;
            if bytes.is_empty() {
                continue;
            }
            let part: std::collections::HashMap<String, Vec<u8>> = JsonCodec
                .deserialize(&bytes)
                .map_err(|e| AssetStorageError::Deserialization(e.to_string()))?;
            for (key, val) in part {
                self.storage.insert(key, val);
            }
        }
        Ok(())
    }

    /// Create new in-memory asset storage
    ///
    /// # Arguments
    ///
    /// * `db_path` - Database path (":memory:" for testing)
    /// # Examples
    ///
    /// ```ignore
    /// use z00z_wallets::persistence::AssetStorageImpl;
    ///
    /// let storage = AssetStorageImpl::new(":memory:").unwrap();
    /// ```
    pub fn new(db_path: impl Into<PathBuf>) -> Result<Self> {
        let db_path = db_path.into();

        let mut this = Self {
            db_path,
            storage: std::collections::HashMap::new(),
        };
        this.load_map()?;
        Ok(this)
    }
}

impl AssetStorage for AssetStorageImpl {
    fn put_for_wallet(&mut self, wallet_id: &str, asset: Asset) -> Result<()> {
        let key = Self::mk_key(wallet_id, &asset.asset_id());

        // Check if already exists
        if self.storage.contains_key(&key) {
            return Err(AssetStorageError::AlreadyExists(key));
        }

        // Wrap asset with spending metadata
        let record = AssetRecord {
            wallet_id: wallet_id.to_string(),
            asset,
            is_spent: false,
            spent_at_height: None,
        };

        // Serialize using z00z_utils::codec (ONE SOURCE OF TRUTH)
        let codec = JsonCodec;
        let data = codec
            .serialize(&record)
            .map_err(|e| AssetStorageError::Serialization(e.to_string()))?;

        // Store
        self.storage.insert(key.clone(), data);
        if let Err(err) = self.save_map() {
            self.storage.remove(&key);
            return Err(err);
        }

        Ok(())
    }

    fn get_for_wallet(&self, wallet_id: &str, asset_id: &[u8; 32]) -> Result<Asset> {
        let key = Self::mk_key(wallet_id, asset_id);

        let data = self
            .storage
            .get(&key)
            .ok_or_else(|| AssetStorageError::NotFound(key.clone()))?;

        // Deserialize using z00z_utils::codec
        let codec = JsonCodec;
        let record: AssetRecord = codec
            .deserialize(data)
            .map_err(|e| AssetStorageError::Deserialization(e.to_string()))?;

        Ok(record.asset)
    }

    fn list_unspent(&self) -> Result<Vec<Asset>> {
        let codec = JsonCodec;
        let mut unspent = Vec::new();

        for data in self.storage.values() {
            let record: AssetRecord = codec
                .deserialize(data)
                .map_err(|e| AssetStorageError::Deserialization(e.to_string()))?;

            if !record.is_spent {
                unspent.push(record.asset);
            }
        }

        Ok(unspent)
    }

    fn list_unspent_for_wallet(&self, wallet_id: &str) -> Result<Vec<Asset>> {
        let codec = JsonCodec;
        let mut out = Vec::new();
        let prefix = Self::key_prefix(wallet_id);

        for (key, data) in &self.storage {
            if !key.starts_with(&prefix) {
                continue;
            }
            let record: AssetRecord = codec
                .deserialize(data)
                .map_err(|e| AssetStorageError::Deserialization(e.to_string()))?;
            if !record.is_spent {
                out.push(record.asset);
            }
        }

        Ok(out)
    }

    fn list_spent(&self) -> Result<Vec<Asset>> {
        let codec = JsonCodec;
        let mut spent = Vec::new();

        for data in self.storage.values() {
            let record: AssetRecord = codec
                .deserialize(data)
                .map_err(|e| AssetStorageError::Deserialization(e.to_string()))?;

            if record.is_spent {
                spent.push(record.asset);
            }
        }

        Ok(spent)
    }

    fn list_spent_for_wallet(&self, wallet_id: &str) -> Result<Vec<Asset>> {
        let codec = JsonCodec;
        let mut out = Vec::new();
        let prefix = Self::key_prefix(wallet_id);

        for (key, data) in &self.storage {
            if !key.starts_with(&prefix) {
                continue;
            }
            let record: AssetRecord = codec
                .deserialize(data)
                .map_err(|e| AssetStorageError::Deserialization(e.to_string()))?;
            if record.is_spent {
                out.push(record.asset);
            }
        }

        Ok(out)
    }

    fn mark_spent_for_wallet(
        &mut self,
        wallet_id: &str,
        asset_id: &[u8; 32],
        spent_at_height: u64,
    ) -> Result<()> {
        let key = Self::mk_key(wallet_id, asset_id);

        // Get existing record
        let data = self
            .storage
            .get(&key)
            .ok_or_else(|| AssetStorageError::NotFound(key.clone()))?;

        let codec = JsonCodec;
        let mut record: AssetRecord = codec
            .deserialize(data)
            .map_err(|e| AssetStorageError::Deserialization(e.to_string()))?;

        // Check if already spent
        if record.is_spent {
            return Err(AssetStorageError::AlreadySpent(key));
        }

        // Update spending status
        record.is_spent = true;
        record.spent_at_height = Some(spent_at_height);

        // Re-serialize
        let data = codec
            .serialize(&record)
            .map_err(|e| AssetStorageError::Serialization(e.to_string()))?;

        // Update storage
        self.storage.insert(key.clone(), data);
        self.save_map()?;

        Ok(())
    }

    fn get_balance(&self) -> Result<u64> {
        let unspent = self.list_unspent()?;
        let balance = unspent.iter().map(|a| a.amount()).sum();

        Ok(balance)
    }

    fn get_balance_for_wallet(&self, wallet_id: &str) -> Result<u64> {
        let unspent = self.list_unspent_for_wallet(wallet_id)?;
        Ok(unspent.iter().map(|a| a.amount()).sum())
    }

    fn remove_for_wallet(&mut self, wallet_id: &str, asset_id: &[u8; 32]) -> Result<()> {
        let key = Self::mk_key(wallet_id, asset_id);

        if self.storage.remove(&key).is_none() {
            return Err(AssetStorageError::NotFound(key));
        }
        self.save_map()?;

        Ok(())
    }
}

#[cfg(test)]
#[path = "test_tracked_asset_storage_impl.rs"]
mod tests;
