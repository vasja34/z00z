//! Wallet RPC implementations backed by `WalletService`.

use async_trait::async_trait;
use jsonrpsee::core::RpcResult;
use jsonrpsee::types::ErrorObjectOwned;
use std::sync::Arc;

use super::wallet_rpc::WalletRpcServer;
use crate::{
    rpc::error_mapping::map_wallet_error_to_rpc,
    rpc::types::common::{PersistWalletId, RuntimeOperationStatus},
    rpc::types::security::{RuntimeRateLimitError, SecurityErrorCode},
    rpc::types::wallet::{
        RuntimeLockWalletResponse, RuntimeShowSeedPhraseResponse, SessionToken,
        WalletLifecycleEvent,
    },
    services::{VerifiedSessionNoTouch, WalletService},
    wallet::{WalletErrorStage, WalletPublicError},
};

/// Wallet RPC service implementation.
pub struct WalletRpcImpl {
    service: Arc<WalletService>,
}

impl WalletRpcImpl {
    pub fn new(service: Arc<WalletService>) -> Self {
        Self { service }
    }

    pub(crate) async fn verify_no_touch_cap(
        &self,
        session: SessionToken,
    ) -> Result<VerifiedSessionNoTouch, ErrorObjectOwned> {
        self.service
            .verify_session_no_touch(&session)
            .await
            .map_err(map_wallet_error_to_rpc)
    }

    pub(crate) async fn lock_wallet_checked(
        &self,
        cap: VerifiedSessionNoTouch,
    ) -> RpcResult<RuntimeLockWalletResponse> {
        let wallet_id = cap.wallet_id().clone();
        self.service
            .lock_wallet(&wallet_id)
            .await
            .map_err(map_wallet_error_to_rpc)?;

        Ok(RuntimeLockWalletResponse {
            status: RuntimeOperationStatus {
                success: true,
                message: String::new(),
            },
            wallet_id,
        })
    }

    pub(crate) async fn show_seed_phrase_checked(
        &self,
        cap: VerifiedSessionNoTouch,
        password: String,
        confirmation: String,
    ) -> RpcResult<RuntimeShowSeedPhraseResponse> {
        match self
            .service
            .show_seed_phrase_precheck(cap.wallet_id())
            .await
            .map_err(map_wallet_error_to_rpc)?
        {
            crate::services::RateLimitPrecheck::Allowed => {}
            crate::services::RateLimitPrecheck::RateLimited {
                retry_after_seconds,
                current_count,
                max_requests,
            } => {
                let data = RuntimeRateLimitError {
                    method: "wallet.session.show_seed_phrase".to_string(),
                    tier: "show_seed_phrase".to_string(),
                    current_count,
                    max_requests,
                    window_seconds: 60,
                    retry_after_seconds,
                };

                return Err(ErrorObjectOwned::owned(
                    SecurityErrorCode::RateLimitExceeded.code(),
                    SecurityErrorCode::RateLimitExceeded.message().to_string(),
                    Some(data),
                ));
            }
        }

        let safe_password = z00z_crypto::expert::encoding::SafePassword::from(password);

        match self
            .service
            .show_seed_phrase(cap.session(), safe_password, confirmation)
            .await
        {
            Ok(response) => Ok(response),
            Err(crate::WalletError::InvalidPassword) => Err(ErrorObjectOwned::owned(
                SecurityErrorCode::AuthenticationFailed.code(),
                SecurityErrorCode::AuthenticationFailed
                    .message()
                    .to_string(),
                None::<()>,
            )),
            Err(crate::WalletError::RateLimited {
                retry_after_seconds,
            }) => {
                let data = RuntimeRateLimitError {
                    method: "wallet.session.show_seed_phrase".to_string(),
                    tier: "show_seed_phrase".to_string(),
                    current_count: 0,
                    max_requests: 0,
                    window_seconds: 60,
                    retry_after_seconds,
                };

                Err(ErrorObjectOwned::owned(
                    SecurityErrorCode::RateLimitExceeded.code(),
                    SecurityErrorCode::RateLimitExceeded.message().to_string(),
                    Some(data),
                ))
            }
            Err(error) => Err(map_wallet_error_to_rpc(error)),
        }
    }
}

#[async_trait]
impl WalletRpcServer for WalletRpcImpl {
    async fn lock_wallet(&self, session: SessionToken) -> RpcResult<RuntimeLockWalletResponse> {
        let cap = self.verify_no_touch_cap(session).await?;
        self.lock_wallet_checked(cap).await
    }

    async fn show_seed_phrase(
        &self,
        session: SessionToken,
        password: String,
        confirmation: String,
    ) -> RpcResult<RuntimeShowSeedPhraseResponse> {
        let cap = self.verify_no_touch_cap(session).await?;
        self.show_seed_phrase_checked(cap, password, confirmation)
            .await
    }

    async fn unlock_wallet(
        &self,
        id: PersistWalletId,
        password: String,
    ) -> RpcResult<SessionToken> {
        let safe_password = z00z_crypto::expert::encoding::SafePassword::from(password);
        // Always delegate to the service, even for empty passwords, so unlock attempts and
        // backoff/rate-limit behavior remain a single source of truth.
        match self
            .service
            .unlock_wallet_in_memory(&id, &safe_password)
            .await
        {
            Ok(token) => Ok(token),
            Err(crate::WalletError::InvalidPassword) => Err(ErrorObjectOwned::owned(
                SecurityErrorCode::AuthenticationFailed.code(),
                SecurityErrorCode::AuthenticationFailed
                    .message()
                    .to_string(),
                None::<()>,
            )),
            Err(crate::WalletError::RateLimited {
                retry_after_seconds,
            }) => {
                let data = RuntimeRateLimitError {
                    method: "wallet.session.unlock_wallet".to_string(),
                    tier: "unlock".to_string(),
                    current_count: 0,
                    max_requests: 0,
                    window_seconds: 60,
                    retry_after_seconds,
                };

                Err(ErrorObjectOwned::owned(
                    SecurityErrorCode::RateLimitExceeded.code(),
                    SecurityErrorCode::RateLimitExceeded.message().to_string(),
                    Some(data),
                ))
            }
            Err(error) => {
                let public = error.to_public_error(WalletErrorStage::UnlockOpen);
                match public {
                    WalletPublicError::InvalidPassword | WalletPublicError::CorruptedWallet => {
                        Err(ErrorObjectOwned::owned(
                            SecurityErrorCode::AuthenticationFailed.code(),
                            SecurityErrorCode::AuthenticationFailed
                                .message()
                                .to_string(),
                            None::<()>,
                        ))
                    }
                    WalletPublicError::WalletLocked => {
                        Err(ErrorObjectOwned::owned(-32003, "Wallet locked", None::<()>))
                    }
                    WalletPublicError::UnsupportedFormat => Err(ErrorObjectOwned::owned(
                        -32023,
                        "Unsupported format",
                        None::<()>,
                    )),
                    WalletPublicError::IoFailure => {
                        Err(ErrorObjectOwned::owned(-32022, "I/O error", None::<()>))
                    }
                }
            }
        }
    }

    async fn on_lifecycle_event(&self, event: WalletLifecycleEvent) -> RpcResult<()> {
        self.service
            .on_lifecycle_event(event)
            .await
            .map_err(map_wallet_error_to_rpc)?;

        Ok(())
    }
}

#[cfg(all(test, not(target_arch = "wasm32")))]
#[path = "test_wallet_impl.rs"]
mod tests;
