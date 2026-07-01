//! KeyRpc implementation - HD key management RPC interface
//!
//! Provides JSON-RPC 2.0 methods for:
//! - BIP-44 key derivation
//! - Extended public key export
//! - Master key rotation
//! - Receiver listing
//! - Receiver validation
//! - Receiver labeling
//!
//! # Architecture Compliance
//!
//! ✅ Trait-based dependency injection (WalletService)
//! ✅ ONE SOURCE OF TRUTH (wraps key::KeyManager)
//! ✅ Session-bound live wallet-service routing
//! ✅ Security: `Hidden<T>` for sensitive data
//! ✅ No logging dependencies by policy

#[path = "key_rpc_server.rs"]
mod key_rpc_server;
#[path = "key_rpc_support.rs"]
mod key_rpc_support;
#[cfg(test)]
#[path = "test_key_impl.rs"]
mod tests;

use jsonrpsee::core::{async_trait, RpcResult};
use std::sync::Arc;

use self::key_rpc_support::{
    apply_receiver_filter, audit_event, build_pub_export, check_rotate_confirm,
    check_rotate_password, decode_cursor, encode_cursor, finish_rotate, invalid_params,
    make_req_params, map_req_decode_err, map_req_validate_err, not_found, parse_payment_id,
    req_response, validate_limit, validate_req_response, wallet_chain_id,
};
use crate::chain::ReceiverCardRecord;
use crate::key::Bip44Path;
use crate::receiver::receiver_card::format_receiver_handle;
use crate::receiver::request::decode_request_compact;
use crate::receiver::{PaymentRequest, ValidatePaymentRequest, ValidateReceiverCard};
use crate::rpc::methods::KeyRpcServer;
use crate::rpc::types::common::{RuntimeOperationStatus, RuntimeValidationResult};
use crate::rpc::types::key::{
    PersistReceiverInfo, RuntimeCreatePaymentRequestResponse, RuntimeDeriveReceiverResponse,
    RuntimeGetReceiverCardResponse, RuntimeLabelReceiverResponse, RuntimeListReceiversResponse,
    RuntimePaymentRequestMetaInput, RuntimePubMaterialExportResponse, RuntimeReceiverFilter,
    RuntimeRotateKeyResponse, RuntimeValidatePaymentRequestResponse,
    RuntimeValidateReceiverCardResponse,
};
use crate::rpc::types::security::AuditResult;
use crate::rpc::types::wallet::SessionToken;
use crate::services::{VerifiedSession, VerifiedSessionNoTouch, WalletService};
use jsonrpsee::types::ErrorObjectOwned;

/// KeyRpc live implementation.
///
/// Routes key derivation, xpub export, receiver flows, and master-key
/// maintenance through the current WalletService and session boundary.
///
/// # Examples
///
/// ```ignore
/// use z00z_wallets::rpc::methods::KeyRpcImpl;
/// use z00z_wallets::services::WalletService;
/// use std::sync::Arc;
///
/// let service = Arc::new(WalletService::new());
/// let key_rpc = KeyRpcImpl::new(service);
/// ```
pub struct KeyRpcImpl {
    /// Wallet service for key operations
    service: Arc<WalletService>,
}

impl KeyRpcImpl {
    /// Create new KeyRpc implementation
    pub fn new(wallet_service: Arc<WalletService>) -> Self {
        Self {
            service: wallet_service,
        }
    }

    pub(crate) async fn verify_touch_cap(
        &self,
        session: SessionToken,
    ) -> Result<VerifiedSession, ErrorObjectOwned> {
        self.service
            .verify_session(&session)
            .await
            .map_err(crate::rpc::error_mapping::map_wallet_error_to_rpc)
    }

    pub(crate) async fn verify_no_touch_cap(
        &self,
        session: SessionToken,
    ) -> Result<VerifiedSessionNoTouch, ErrorObjectOwned> {
        self.service
            .verify_session_no_touch(&session)
            .await
            .map_err(crate::rpc::error_mapping::map_wallet_error_to_rpc)
    }

    pub(crate) async fn verify_rotate_cap(
        &self,
        session: SessionToken,
    ) -> Result<VerifiedSessionNoTouch, ErrorObjectOwned> {
        let wallet_id = session.wallet_id.clone();
        match self.service.verify_session_no_touch(&session).await {
            Ok(cap) => Ok(cap),
            Err(error) => {
                let audit_wallet_id = match &error {
                    crate::WalletError::SessionExpired => Some(wallet_id.clone()),
                    _ => None,
                };
                audit_event(
                    &self.service,
                    audit_wallet_id,
                    "wallet.key.rotate_master_key",
                    AuditResult::Denied,
                    Some(format!(
                        "session_guard_failed={error},requested_wallet_id={}",
                        wallet_id.0
                    )),
                )
                .await;
                Err(crate::rpc::error_mapping::map_wallet_error_to_rpc(error))
            }
        }
    }
}

// NOTE: key RPC rate limiting is owned by WalletService (process-local).
