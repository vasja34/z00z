#[cfg(not(target_arch = "wasm32"))]
use jsonrpsee::{core::RpcResult, proc_macros::rpc};

#[cfg(not(target_arch = "wasm32"))]
use super::super::types::{
    common::PersistWalletId,
    object::{
        RuntimeListObjectsResponse, RuntimeListRightInventoryResponse,
        RuntimeListVoucherClaimsResponse, RuntimeObjectListFilter,
        RuntimeObjectPackageBuildResponse, RuntimeObjectPackagePreviewResponse,
        RuntimeObjectPackageRequest,
    },
    wallet::SessionToken,
};

#[cfg(not(target_arch = "wasm32"))]
use crate::db::{OwnedRightStatus, OwnedVoucherStatus};

/// Typed object RPC surface for voucher and right inventory plus package
/// building.
///
/// This namespace is the wallet-visible typed-object authority plane. It must
/// not create a second persistence story beside `.wlt` plus `WalletExportPack`,
/// and it must not cause `wallet.asset.*` to absorb voucher/right semantics as
/// cash.
#[cfg(not(target_arch = "wasm32"))]
#[rpc(server, client)]
pub trait ObjectRpc {
    #[method(name = "wallet.object.list_objects")]
    async fn list_objects(
        &self,
        wallet_id: PersistWalletId,
        limit: Option<usize>,
        cursor: Option<String>,
        filter: Option<RuntimeObjectListFilter>,
    ) -> RpcResult<RuntimeListObjectsResponse>;

    #[method(name = "wallet.object.list_vouchers")]
    async fn list_vouchers(
        &self,
        wallet_id: PersistWalletId,
        limit: Option<usize>,
        cursor: Option<String>,
        status: Option<OwnedVoucherStatus>,
    ) -> RpcResult<RuntimeListVoucherClaimsResponse>;

    #[method(name = "wallet.object.list_rights")]
    async fn list_rights(
        &self,
        wallet_id: PersistWalletId,
        limit: Option<usize>,
        cursor: Option<String>,
        status: Option<OwnedRightStatus>,
    ) -> RpcResult<RuntimeListRightInventoryResponse>;

    #[method(name = "wallet.object.preview_package")]
    async fn preview_package(
        &self,
        session: SessionToken,
        request: RuntimeObjectPackageRequest,
    ) -> RpcResult<RuntimeObjectPackagePreviewResponse>;

    #[method(name = "wallet.object.build_package")]
    async fn build_package(
        &self,
        session: SessionToken,
        request: RuntimeObjectPackageRequest,
    ) -> RpcResult<RuntimeObjectPackageBuildResponse>;

    #[method(name = "wallet.object.accept_voucher")]
    async fn accept_voucher(
        &self,
        session: SessionToken,
        request: RuntimeObjectPackageRequest,
    ) -> RpcResult<RuntimeObjectPackageBuildResponse>;

    #[method(name = "wallet.object.reject_voucher")]
    async fn reject_voucher(
        &self,
        session: SessionToken,
        request: RuntimeObjectPackageRequest,
    ) -> RpcResult<RuntimeObjectPackageBuildResponse>;

    #[method(name = "wallet.object.redeem_voucher")]
    async fn redeem_voucher(
        &self,
        session: SessionToken,
        request: RuntimeObjectPackageRequest,
    ) -> RpcResult<RuntimeObjectPackageBuildResponse>;

    #[method(name = "wallet.object.refund_voucher")]
    async fn refund_voucher(
        &self,
        session: SessionToken,
        request: RuntimeObjectPackageRequest,
    ) -> RpcResult<RuntimeObjectPackageBuildResponse>;

    #[method(name = "wallet.object.transfer_voucher")]
    async fn transfer_voucher(
        &self,
        session: SessionToken,
        request: RuntimeObjectPackageRequest,
    ) -> RpcResult<RuntimeObjectPackageBuildResponse>;

    #[method(name = "wallet.object.delegate_right")]
    async fn delegate_right(
        &self,
        session: SessionToken,
        request: RuntimeObjectPackageRequest,
    ) -> RpcResult<RuntimeObjectPackageBuildResponse>;

    #[method(name = "wallet.object.consume_right")]
    async fn consume_right(
        &self,
        session: SessionToken,
        request: RuntimeObjectPackageRequest,
    ) -> RpcResult<RuntimeObjectPackageBuildResponse>;

    #[method(name = "wallet.object.revoke_right")]
    async fn revoke_right(
        &self,
        session: SessionToken,
        request: RuntimeObjectPackageRequest,
    ) -> RpcResult<RuntimeObjectPackageBuildResponse>;

    #[method(name = "wallet.object.challenge_right")]
    async fn challenge_right(
        &self,
        session: SessionToken,
        request: RuntimeObjectPackageRequest,
    ) -> RpcResult<RuntimeObjectPackageBuildResponse>;
}
