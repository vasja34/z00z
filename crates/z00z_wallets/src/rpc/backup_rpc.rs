//! Backup RPC method definitions - all methods from rpc_naming-spec
//!
//! This module defines the complete JSON-RPC 2.0 interface for backup operations.

#[cfg(not(target_arch = "wasm32"))]
use jsonrpsee::{core::RpcResult, proc_macros::rpc};

#[cfg(not(target_arch = "wasm32"))]
use super::super::types::{
    backup::{
        PersistBackupSettings, RuntimeBackupSettingsResponse, RuntimeCreateBackupResponse,
        RuntimeListBackupsResponse, RuntimeRestoreBackupResponse,
    },
    wallet::SessionToken,
};

/// Backup RPC trait defining backup management operations.
///
/// # JSON-RPC 2.0 Methods
///
/// Uses the standardized `kernel.service.method` naming from `rpc_naming-spec`.
#[cfg(not(target_arch = "wasm32"))]
#[rpc(server, client)]
pub trait BackupRpc {
    /// Create wallet backup
    ///
    /// # JSON-RPC Method
    /// ```json
    /// {"jsonrpc": "2.0", "method": "wallet.backup.create_backup", "params": {"session": {"token": "***", "wallet_id": "...", "created_at": 0, "expires_at": 0, "last_activity_at": 0, "permissions": ["all"]}, "password": "***", "destination": null}, "id": 1}
    /// ```
    #[method(name = "wallet.backup.create_backup")]
    async fn create_backup(
        &self,
        session: SessionToken,
        password: String,
        destination: Option<String>,
    ) -> RpcResult<RuntimeCreateBackupResponse>;

    /// List backups for a wallet
    ///
    /// # JSON-RPC Method
    /// ```json
    /// {"jsonrpc": "2.0", "method": "wallet.backup.list_backups", "params": {"session": {"token": "***", "wallet_id": "...", "created_at": 0, "expires_at": 0, "last_activity_at": 0, "permissions": ["all"]}, "cursor": null, "limit": 50}, "id": 1}
    /// ```
    #[method(name = "wallet.backup.list_backups")]
    async fn list_backups(
        &self,
        session: SessionToken,
        cursor: Option<String>,
        limit: Option<u32>,
    ) -> RpcResult<RuntimeListBackupsResponse>;

    /// Restore from backup
    ///
    /// # JSON-RPC Method
    /// ```json
    /// {"jsonrpc": "2.0", "method": "wallet.backup.restore_backup", "params": {"backup_path": "...", "password": "***", "wallet_name": null}, "id": 1}
    /// ```
    #[method(name = "wallet.backup.restore_backup")]
    async fn restore_backup(
        &self,
        backup_path: String,
        password: String,
        wallet_name: Option<String>,
    ) -> RpcResult<RuntimeRestoreBackupResponse>;

    /// Configure backup preferences
    ///
    /// # JSON-RPC Method
    /// ```json
    /// {"jsonrpc": "2.0", "method": "wallet.backup.configure_backup", "params": {"session": {"token": "***", "wallet_id": "...", "created_at": 0, "expires_at": 0, "last_activity_at": 0, "permissions": ["all"]}, "settings": {...}}, "id": 1}
    /// ```
    #[method(name = "wallet.backup.configure_backup")]
    async fn configure_backup_settings(
        &self,
        session: SessionToken,
        settings: Option<PersistBackupSettings>,
    ) -> RpcResult<RuntimeBackupSettingsResponse>;
}
