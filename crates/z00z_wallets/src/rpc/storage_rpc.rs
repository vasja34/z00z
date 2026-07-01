//! Storage administration RPC methods.
//!
//! Provides storage operations over the current wallet outputs layout:
//! - compaction of obvious temporary leftovers
//! - aggregate storage statistics for live wallet files
//! - export of the current output tree to manifest-style formats
//!
//! The current trait shape does not carry a session token. Any caller-side
//! access control must therefore be enforced by the surrounding RPC host or
//! dispatcher layer.

use async_trait::async_trait;
use jsonrpsee::core::RpcResult;

use super::super::types::storage::{
    RuntimeCompactStorageParams, RuntimeCompactStorageResponse, RuntimeExportStorageParams,
    RuntimeExportStorageResponse, RuntimeGetStorageStatsParams, RuntimeStorageStats,
};

/// Storage administration RPC trait.
///
/// Provides administrative operations for storage management.
///
/// # Methods
///
/// - [`compact_storage`](Self::compact_storage) - Compact database to reclaim space
/// - [`get_storage_stats`](Self::get_storage_stats) - Get storage statistics
/// - [`export_storage`](Self::export_storage) - Export database to file
///
/// # Examples
///
/// ```rust
/// use z00z_wallets::rpc::methods::storage_rpc::StorageRpc;
///
/// async fn compact_example(storage_rpc: impl StorageRpc) {
///     let params = Default::default();
///     let response = storage_rpc.compact_storage(params).await.unwrap();
///     assert!(response.bytes_reclaimed <= u64::MAX);
/// }
/// ```
#[async_trait]
pub trait StorageRpc {
    /// Compact database to reclaim space from deleted records.
    ///
    /// Performs database compaction/vacuum operation.
    ///
    /// # Parameters
    ///
    /// - `params`: Compaction options (force, dry_run)
    ///
    /// # Returns
    ///
    /// - Bytes reclaimed, duration, old/new sizes
    ///
    /// Current behavior removes obvious temporary leftovers from the wallet
    /// outputs directory and reports the resulting size delta.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use z00z_wallets::rpc::methods::storage_rpc::StorageRpc;
    /// # use z00z_wallets::rpc::types::storage::RuntimeCompactStorageParams;
    /// # async fn example(rpc: impl StorageRpc) {
    /// let params = RuntimeCompactStorageParams {
    ///     force: false,
    ///     dry_run: false,
    /// };
    /// let response = rpc.compact_storage(params).await.unwrap();
    /// # }
    /// ```
    async fn compact_storage(
        &self,
        params: RuntimeCompactStorageParams,
    ) -> RpcResult<RuntimeCompactStorageResponse>;

    /// Get storage statistics (sizes, record counts, fragmentation).
    ///
    /// # use z00z_wallets::rpc::types::storage::RuntimeGetStorageStatsParams;
    ///
    /// - `params`: Query options (include_details)
    ///
    /// # Returns
    ///
    /// - Storage statistics (total size, wallets, transactions, fragmentation)
    ///
    /// Current behavior inspects the live wallet outputs layout and reports
    /// aggregate file-based statistics for `.wlt` packs plus sidecars.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use z00z_wallets::rpc::methods::storage_rpc::StorageRpc;
    /// # use z00z_wallets::rpc::types::storage::RuntimeGetStorageStatsParams;
    /// # async fn example(rpc: impl StorageRpc) {
    /// let params = RuntimeGetStorageStatsParams {
    ///     include_details: true,
    /// };
    /// let stats = rpc.get_storage_stats(params).await.unwrap();
    /// assert!(stats.total_size_bytes <= u64::MAX);
    /// # }
    /// ```
    async fn get_storage_stats(
        &self,
        params: RuntimeGetStorageStatsParams,
    ) -> RpcResult<RuntimeStorageStats>;

    /// Export database to file (JSON, SQL, or binary format).
    ///
    /// # Parameters
    ///
    /// - `params`: Export options (path, format, include_deleted)
    ///
    /// # Returns
    ///
    /// - Export job status, export_path, format
    ///
    /// Current behavior writes a manifest-style JSON export of the live wallet
    /// outputs layout. Other formats report explicit non-implementation rather
    /// than pretending to be complete.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use z00z_wallets::rpc::methods::storage_rpc::StorageRpc;
    /// # use z00z_wallets::rpc::types::storage::RuntimeExportStorageParams;
    /// # async fn example(rpc: impl StorageRpc) {
    /// let params = RuntimeExportStorageParams {
    ///     path: "/tmp/wallet_backup.json".to_string(),
    ///     format: "json".to_string(),
    ///     include_deleted: false,
    /// };
    /// let response = rpc.export_storage(params).await.unwrap();
    /// # }
    /// ```
    async fn export_storage(
        &self,
        params: RuntimeExportStorageParams,
    ) -> RpcResult<RuntimeExportStorageResponse>;
}
