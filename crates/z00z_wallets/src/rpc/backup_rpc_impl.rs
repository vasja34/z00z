//! Backup RPC implementations backed by `WalletService`.

use async_trait::async_trait;
use jsonrpsee::core::RpcResult;
use jsonrpsee::types::ErrorObjectOwned;
use std::sync::Arc;
use z00z_crypto::expert::encoding::SafePassword;
use z00z_utils::time::TimeProvider;

use super::backup_rpc::BackupRpcServer;
use crate::{
    rpc::error_mapping::map_wallet_error_to_rpc,
    rpc::types::{
        backup::{
            PersistBackupSettings, RuntimeBackupSettingsResponse, RuntimeCreateBackupResponse,
            RuntimeListBackupsResponse, RuntimeRestoreBackupResponse,
        },
        common::PersistWalletId,
        security::{RuntimeRateLimitError, SecurityErrorCode},
        wallet::SessionToken,
    },
    services::{VerifiedSession, WalletService},
    WalletError,
};

/// Backup RPC service implementation for manifest-backed `.wlt` plus JSONL
/// backup and restore flows.
pub struct BackupRpcImpl {
    service: Arc<WalletService>,
}

impl BackupRpcImpl {
    pub fn new(service: Arc<WalletService>) -> Self {
        Self { service }
    }

    pub fn with_dependencies<T: TimeProvider>(
        service: Arc<WalletService>,
        _time_provider: T,
    ) -> Self {
        Self { service }
    }

    #[cfg(test)]
    async fn backup_create_rate_limit_precheck(
        &self,
        wallet_id: &PersistWalletId,
    ) -> Result<(), ErrorObjectOwned> {
        const MAX_REQUESTS: u32 = 1;
        match self
            .service
            .backup_create_rate_limit_precheck(wallet_id)
            .await
        {
            Ok(_) => Ok(()),
            Err(WalletError::RateLimited {
                retry_after_seconds,
            }) => {
                let data = RuntimeRateLimitError {
                    method: "wallet.backup.create_backup".to_string(),
                    tier: "wallet.backup.create_backup".to_string(),
                    current_count: 1,
                    max_requests: MAX_REQUESTS,
                    window_seconds: 3600,
                    retry_after_seconds,
                };

                Err(ErrorObjectOwned::owned(
                    SecurityErrorCode::RateLimitExceeded.code(),
                    SecurityErrorCode::RateLimitExceeded.message().to_string(),
                    Some(data),
                ))
            }
            Err(other) => Err(map_wallet_error_to_rpc(other)),
        }
    }

    pub(crate) async fn verify_touch_cap(
        &self,
        session: SessionToken,
    ) -> Result<VerifiedSession, ErrorObjectOwned> {
        #[cfg(all(test, feature = "os_hardening"))]
        {
            Ok(VerifiedSession::new(&session))
        }

        #[cfg(not(all(test, feature = "os_hardening")))]
        {
            self.service
                .check_auto_lock()
                .await
                .map_err(map_wallet_error_to_rpc)?;

            self.service
                .verify_session(&session)
                .await
                .map_err(map_wallet_error_to_rpc)
        }
    }

    /// Update activity timestamp.
    async fn update_activity(&self, wallet_id: &PersistWalletId) {
        let _ = self.service.update_activity(wallet_id).await;
    }

    pub(crate) async fn create_backup_checked(
        &self,
        cap: VerifiedSession,
        password: String,
        destination: Option<String>,
    ) -> RpcResult<RuntimeCreateBackupResponse> {
        let wallet_id = cap.wallet_id().clone();

        let response = self
            .service
            .create_backup(&wallet_id, SafePassword::from(password), destination)
            .await
            .map_err(|e| match e {
                WalletError::RateLimited {
                    retry_after_seconds,
                } => {
                    let data = RuntimeRateLimitError {
                        method: "wallet.backup.create_backup".to_string(),
                        tier: "wallet.backup.create_backup".to_string(),
                        current_count: 1,
                        max_requests: 1,
                        window_seconds: 3600,
                        retry_after_seconds,
                    };

                    ErrorObjectOwned::owned(
                        SecurityErrorCode::RateLimitExceeded.code(),
                        SecurityErrorCode::RateLimitExceeded.message().to_string(),
                        Some(data),
                    )
                }
                WalletError::InvalidPassword => ErrorObjectOwned::owned(
                    SecurityErrorCode::AuthenticationFailed.code(),
                    SecurityErrorCode::AuthenticationFailed
                        .message()
                        .to_string(),
                    None::<()>,
                ),
                other => map_wallet_error_to_rpc(other),
            })?;

        self.update_activity(&wallet_id).await;

        Ok(response)
    }

    pub(crate) async fn list_backups_checked(
        &self,
        cap: VerifiedSession,
        cursor: Option<String>,
        limit: Option<u32>,
    ) -> RpcResult<RuntimeListBackupsResponse> {
        let wallet_id = cap.wallet_id().clone();

        let response = self
            .service
            .list_backups(&wallet_id, cursor, limit)
            .await
            .map_err(map_wallet_error_to_rpc)?;

        self.update_activity(&wallet_id).await;

        Ok(response)
    }

    pub(crate) async fn configure_backup_checked(
        &self,
        cap: VerifiedSession,
        settings: Option<PersistBackupSettings>,
    ) -> RpcResult<RuntimeBackupSettingsResponse> {
        let wallet_id = cap.wallet_id().clone();

        let response = self
            .service
            .configure_backup_settings(&wallet_id, settings)
            .await
            .map_err(map_wallet_error_to_rpc)?;

        self.update_activity(&wallet_id).await;

        Ok(response)
    }
}

#[async_trait]
impl BackupRpcServer for BackupRpcImpl {
    async fn create_backup(
        &self,
        session: SessionToken,
        password: String,
        destination: Option<String>,
    ) -> RpcResult<RuntimeCreateBackupResponse> {
        let cap = self.verify_touch_cap(session).await?;
        self.create_backup_checked(cap, password, destination).await
    }

    async fn list_backups(
        &self,
        session: SessionToken,
        cursor: Option<String>,
        limit: Option<u32>,
    ) -> RpcResult<RuntimeListBackupsResponse> {
        let cap = self.verify_touch_cap(session).await?;
        self.list_backups_checked(cap, cursor, limit).await
    }

    async fn restore_backup(
        &self,
        backup_path: String,
        password: String,
        _wallet_name: Option<String>,
    ) -> RpcResult<RuntimeRestoreBackupResponse> {
        let response = self
            .service
            .restore_backup(backup_path, SafePassword::from(password), _wallet_name)
            .await
            .map_err(|e| match e {
                WalletError::InvalidPassword => ErrorObjectOwned::owned(
                    SecurityErrorCode::AuthenticationFailed.code(),
                    SecurityErrorCode::AuthenticationFailed
                        .message()
                        .to_string(),
                    None::<()>,
                ),
                other => map_wallet_error_to_rpc(other),
            })?;

        Ok(response)
    }

    async fn configure_backup_settings(
        &self,
        session: SessionToken,
        settings: Option<PersistBackupSettings>,
    ) -> RpcResult<RuntimeBackupSettingsResponse> {
        let cap = self.verify_touch_cap(session).await?;
        self.configure_backup_checked(cap, settings).await
    }
}

#[cfg(test)]
#[path = "test_backup_impl.rs"]
mod tests;
