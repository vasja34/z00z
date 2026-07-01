//! Asset storage trait and wallet asset storage implementation
//!
//! Manages unspent assets for wallet balance and asset selection.

use thiserror::Error;
use z00z_core::assets::Asset;

/// Errors that can occur during asset storage operations
#[derive(Debug, Error)]
pub enum AssetStorageError {
    /// Database operation failed
    #[error("Database error: {0}")]
    Database(String),

    /// Asset not found in store
    #[error("Asset not found: {0}")]
    NotFound(String),

    /// Asset already exists (duplicate insert)
    #[error("Asset already exists: {0}")]
    AlreadyExists(String),

    /// Serialization failed
    #[error("Serialization error: {0}")]
    Serialization(String),

    /// Deserialization failed
    #[error("Deserialization error: {0}")]
    Deserialization(String),

    /// Asset already spent (cannot spend twice)
    #[error("Asset already spent: {0}")]
    AlreadySpent(String),
}

/// Result type for asset storage operations.
pub type AssetStorageResult<T> = std::result::Result<T, AssetStorageError>;

/// Asset storage trait for wallet Assets.
///
/// Phase 037 keeps wallet-native claimed-asset persistence as the current
/// receive target; this trait remains a future-unification seam for storage
/// metadata flows, not a second receive authority.
///
/// Manages unspent Assets for balance tracking
/// and asset selection. Provides CRUD operations with spending status.
///
/// # Architecture Compliance
///
/// - ✅ Uses z00z_utils::codec for serialization (ONE SOURCE OF TRUTH)
/// - ✅ This trait is side-effect free and has no logging dependencies
/// - ✅ Trait-based design for testability (mock implementations)
/// - ✅ No direct std::fs or serde_json usage
///
/// # Examples
///
/// ```ignore
/// use z00z_wallets::persistence::{AssetStorage, AssetStorageImpl};
/// use z00z_core::assets::Asset;
///
/// let mut storage = AssetStorageImpl::new("wallet.db")?;
///
/// // Add unspent asset
/// storage.put(tracked_asset)?;
///
/// // List unspent assets
/// let unspent = storage.list_unspent()?;
///
/// // Mark as spent
/// storage.mark_spent(&asset_id, 1001)?;
/// ```
pub trait AssetStorage {
    /// Add new asset to store for a specific wallet.
    fn put_for_wallet(&mut self, wallet_id: &str, asset: Asset) -> AssetStorageResult<()>;

    /// Get wallet asset by ID.
    fn get_for_wallet(&self, wallet_id: &str, asset_id: &[u8; 32]) -> AssetStorageResult<Asset>;

    /// List unspent assets for one wallet.
    fn list_unspent_for_wallet(&self, wallet_id: &str) -> AssetStorageResult<Vec<Asset>>;

    /// List spent assets for one wallet.
    fn list_spent_for_wallet(&self, wallet_id: &str) -> AssetStorageResult<Vec<Asset>>;

    /// Mark wallet asset as spent.
    fn mark_spent_for_wallet(
        &mut self,
        wallet_id: &str,
        asset_id: &[u8; 32],
        spent_at_height: u64,
    ) -> AssetStorageResult<()>;

    /// Get total unspent balance for one wallet.
    fn get_balance_for_wallet(&self, wallet_id: &str) -> AssetStorageResult<u64>;

    /// Remove wallet asset by id.
    fn remove_for_wallet(&mut self, wallet_id: &str, asset_id: &[u8; 32])
        -> AssetStorageResult<()>;

    /// Add new asset to store
    ///
    /// # Arguments
    ///
    /// * `asset` - Asset to store
    ///
    /// # Returns
    ///
    /// * `Ok(())` - Asset stored successfully
    /// * `Err(AssetStorageError::AlreadyExists)` - Asset ID already in storage
    /// * `Err(AssetStorageError::Database)` - Database operation failed
    fn put(&mut self, asset: Asset) -> AssetStorageResult<()> {
        self.put_for_wallet("__global__", asset)
    }

    /// Get asset by ID
    ///
    /// # Arguments
    ///
    /// * `asset_id` - Asset identifier (32-byte hash from Asset.asset_id())
    ///
    /// # Returns
    ///
    /// * `Ok(Asset)` - Asset found
    /// * `Err(AssetNotFound)` - Asset not in store
    fn get(&self, asset_id: &[u8; 32]) -> AssetStorageResult<Asset> {
        self.get_for_wallet("__global__", asset_id)
    }

    /// List all unspent assets
    ///
    /// # Returns
    ///
    /// * `Ok(Vec<Asset>)` - All unspent assets
    /// * `Err(AssetStorageError::Database)` - Database query failed
    fn list_unspent(&self) -> AssetStorageResult<Vec<Asset>>;

    /// List all spent assets
    ///
    /// # Returns
    ///
    /// * `Ok(Vec<Asset>)` - All spent assets
    /// * `Err(AssetStorageError::Database)` - Database query failed
    fn list_spent(&self) -> AssetStorageResult<Vec<Asset>>;

    /// Mark asset as spent
    ///
    /// # Arguments
    ///
    /// * `asset_id` - Asset identifier (32-byte hash from Asset.asset_id())
    /// * `spent_at_height` - Block height when spent
    ///
    /// # Returns
    ///
    /// * `Ok(())` - Asset marked as spent
    /// * `Err(AssetStorageError::NotFound)` - Asset not in storage
    /// * `Err(AssetStorageError::AlreadySpent)` - Asset already spent
    fn mark_spent(&mut self, asset_id: &[u8; 32], spent_at_height: u64) -> AssetStorageResult<()> {
        self.mark_spent_for_wallet("__global__", asset_id, spent_at_height)
    }

    /// Get total balance (sum of unspent asset amounts)
    ///
    /// NOTE: In production, amounts are hidden by commitments.
    /// This is a placeholder for testing.
    ///
    /// # Returns
    ///
    /// * `Ok(u64)` - Total unspent balance
    /// * `Err(AssetStorageError::Database)` - Database query failed
    fn get_balance(&self) -> AssetStorageResult<u64>;

    /// Remove asset from store (dangerous - use with caution)
    ///
    /// # Arguments
    ///
    /// * `asset_id` - Asset identifier (32-byte hash from Asset.asset_id())
    ///
    /// # Returns
    ///
    /// * `Ok(())` - Asset removed
    /// * `Err(AssetStorageError::NotFound)` - Asset not in storage
    fn remove(&mut self, asset_id: &[u8; 32]) -> AssetStorageResult<()> {
        self.remove_for_wallet("__global__", asset_id)
    }
}
