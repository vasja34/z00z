use super::{
    apply_receiver_filter, async_trait, audit_event, build_pub_export, check_rotate_confirm,
    check_rotate_password, decode_cursor, decode_request_compact, encode_cursor, finish_rotate,
    format_receiver_handle, invalid_params, make_req_params, map_req_decode_err,
    map_req_validate_err, not_found, parse_payment_id, req_response, validate_limit,
    validate_req_response, wallet_chain_id, AuditResult, Bip44Path, ErrorObjectOwned, KeyRpcImpl,
    KeyRpcServer, PaymentRequest, PersistReceiverInfo, ReceiverCardRecord, RpcResult,
    RuntimeCreatePaymentRequestResponse, RuntimeDeriveReceiverResponse,
    RuntimeGetReceiverCardResponse, RuntimeLabelReceiverResponse, RuntimeListReceiversResponse,
    RuntimeOperationStatus, RuntimePaymentRequestMetaInput, RuntimePubMaterialExportResponse,
    RuntimeReceiverFilter, RuntimeRotateKeyResponse, RuntimeValidatePaymentRequestResponse,
    RuntimeValidateReceiverCardResponse, RuntimeValidationResult, SessionToken,
    ValidatePaymentRequest, ValidateReceiverCard,
};
use crate::services::{VerifiedSession, VerifiedSessionNoTouch};

include!("key_rpc_server_derive.rs");
include!("key_rpc_server_requests.rs");
include!("key_rpc_server_admin.rs");

#[async_trait]
impl KeyRpcServer for KeyRpcImpl {
    async fn derive_receiver(
        &self,
        session: SessionToken,
        path: String,
    ) -> RpcResult<RuntimeDeriveReceiverResponse> {
        let cap = self.verify_no_touch_cap(session).await?;
        self.derive_receiver_checked(cap, path).await
    }

    async fn get_receiver_card(
        &self,
        session: SessionToken,
    ) -> RpcResult<RuntimeGetReceiverCardResponse> {
        let cap = self.verify_touch_cap(session).await?;
        self.get_receiver_card_checked(cap).await
    }

    async fn create_payment_request(
        &self,
        session: SessionToken,
        amount: Option<u64>,
        expiry_secs: u64,
        metadata: Option<RuntimePaymentRequestMetaInput>,
    ) -> RpcResult<RuntimeCreatePaymentRequestResponse> {
        let cap = self.verify_touch_cap(session).await?;
        self.create_payment_request_checked(cap, amount, expiry_secs, metadata)
            .await
    }

    async fn validate_payment_request(
        &self,
        session: SessionToken,
        request_compact: String,
    ) -> RpcResult<RuntimeValidatePaymentRequestResponse> {
        let cap = self.verify_touch_cap(session).await?;
        self.validate_payment_request_checked(cap, request_compact)
            .await
    }

    async fn export_public_material(
        &self,
        session: SessionToken,
        account: u32,
        password: String,
    ) -> RpcResult<RuntimePubMaterialExportResponse> {
        let cap = self.verify_touch_cap(session).await?;
        self.export_public_material_checked(cap, account, password)
            .await
    }

    async fn rotate_master_key(
        &self,
        session: SessionToken,
        password: String,
        confirmation: String,
    ) -> RpcResult<RuntimeRotateKeyResponse> {
        let cap = self.verify_rotate_cap(session).await?;
        self.rotate_master_key_checked(cap, password, confirmation)
            .await
    }

    async fn list_receivers(
        &self,
        session: SessionToken,
        limit: Option<usize>,
        cursor: Option<String>,
        filter: Option<RuntimeReceiverFilter>,
    ) -> RpcResult<RuntimeListReceiversResponse> {
        let cap = self.verify_touch_cap(session).await?;
        self.list_receivers_checked(cap, limit, cursor, filter)
            .await
    }

    async fn validate_receiver_card(
        &self,
        card_compact: String,
    ) -> RpcResult<RuntimeValidateReceiverCardResponse> {
        self.validate_receiver_card_impl(card_compact).await
    }

    async fn label_receiver(
        &self,
        session: SessionToken,
        receiver_id: String,
        label: String,
    ) -> RpcResult<RuntimeLabelReceiverResponse> {
        let cap = self.verify_touch_cap(session).await?;
        self.label_receiver_checked(cap, receiver_id, label).await
    }
}
