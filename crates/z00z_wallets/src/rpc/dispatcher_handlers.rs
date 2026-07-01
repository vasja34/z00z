//! Shared helpers and parameter structs for dispatcher wiring.
//!
//! Important: the wiring modules must keep explicit `dispatcher.register_*` calls with literal
//! method strings so the static audit scripts can detect registrations.

#![cfg(not(target_arch = "wasm32"))]

use crate::rpc::types::{common::PersistWalletId, wallet::SessionToken};
use jsonrpsee::types::ErrorObjectOwned;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use z00z_networks_rpc::RpcError;
use z00z_utils::codec::{Codec, JsonCodec, Value};

type JsonHandlerFuture = Pin<Box<dyn Future<Output = Result<Value, RpcError>> + Send + 'static>>;

type TypedHandlerFuture<Res> =
    Pin<Box<dyn Future<Output = Result<Res, RpcError>> + Send + 'static>>;

#[derive(Debug, Deserialize)]
pub(crate) struct WalletIdParams {
    pub(crate) wallet_id: PersistWalletId,
}

#[derive(Debug, Deserialize)]
pub(crate) struct WalletIdPasswordParams {
    pub(crate) wallet_id: PersistWalletId,
    pub(crate) password: String,
}

#[derive(Debug, Deserialize)]
pub(crate) struct NoArgs {}

#[derive(Debug, Deserialize)]
pub(crate) struct SessionArgs<T> {
    pub(crate) session: SessionToken,
    #[serde(flatten)]
    pub(crate) args: T,
}

pub(crate) fn typed_handler_ok<R, Req, Res, F, Fut>(
    rpc: Arc<R>,
    f: F,
) -> impl Fn(Req) -> TypedHandlerFuture<Res> + Send + Sync + 'static
where
    R: Send + Sync + 'static,
    Req: DeserializeOwned + Send + 'static,
    Res: Send + 'static,
    F: Fn(Arc<R>, Req) -> Fut + Send + Sync + 'static,
    Fut: Future<Output = Res> + Send + 'static,
{
    let f = Arc::new(f);
    move |req: Req| {
        let rpc = Arc::clone(&rpc);
        let f = Arc::clone(&f);
        Box::pin(async move { Ok((f.as_ref())(rpc, req).await) })
    }
}

pub(crate) fn typed_handler_map_err<R, Req, Res, E, F, Fut, Map>(
    rpc: Arc<R>,
    f: F,
    map_err: Map,
) -> impl Fn(Req) -> TypedHandlerFuture<Res> + Send + Sync + 'static
where
    R: Send + Sync + 'static,
    Req: DeserializeOwned + Send + 'static,
    Res: Send + 'static,
    E: Send + 'static,
    F: Fn(Arc<R>, Req) -> Fut + Send + Sync + 'static,
    Fut: Future<Output = Result<Res, E>> + Send + 'static,
    Map: Fn(E) -> RpcError + Send + Sync + 'static,
{
    let f = Arc::new(f);
    let map_err = Arc::new(map_err);

    move |req: Req| {
        let rpc = Arc::clone(&rpc);
        let f = Arc::clone(&f);
        let map_err = Arc::clone(&map_err);

        Box::pin(async move {
            (f.as_ref())(rpc, req)
                .await
                .map_err(|e| (map_err.as_ref())(e))
        })
    }
}

pub(crate) fn map_error_object_owned(err: ErrorObjectOwned) -> RpcError {
    // Convert JSON-RPC error objects into stable in-process `RpcError` categories.
    // This is used by dispatcher wiring, where we bridge jsonrpsee-based RPC impls
    // into the generic in-process transport.
    match err.code() {
        // JSON-RPC invalid params.
        -32602 => RpcError::InvalidParams(err.message().to_string()),

        // Wallet-reserved stable codes.
        -32003 => RpcError::WalletLocked,
        -32007 => RpcError::AuthFailed,

        // Security error codes (see `types::security::SecurityErrorCode`).
        -32401 => RpcError::AuthFailed,
        -32402 => RpcError::SessionExpired,
        -32403 => RpcError::SessionInvalid,
        -32429 => RpcError::RateLimited(
            err.data()
                .map(|data| data.to_string())
                .filter(|data| !data.is_empty())
                .unwrap_or_else(|| err.message().to_string()),
        ),

        _ => RpcError::RequestFailed(err.to_string()),
    }
}

pub(crate) fn typed_handler_jsonrpsee_err<R, Req, Res, F, Fut>(
    rpc: Arc<R>,
    f: F,
) -> impl Fn(Req) -> TypedHandlerFuture<Res> + Send + Sync + 'static
where
    R: Send + Sync + 'static,
    Req: DeserializeOwned + Send + 'static,
    Res: Send + 'static,
    F: Fn(Arc<R>, Req) -> Fut + Send + Sync + 'static,
    Fut: Future<Output = Result<Res, ErrorObjectOwned>> + Send + 'static,
{
    typed_handler_map_err(rpc, f, map_error_object_owned)
}

pub(crate) fn typed_handler_cap<R, Req, Res, Cap, Verify, VerifyFut, F, Fut>(
    rpc: Arc<R>,
    verify: Verify,
    f: F,
) -> impl Fn(SessionArgs<Req>) -> TypedHandlerFuture<Res> + Send + Sync + 'static
where
    R: Send + Sync + 'static,
    Req: DeserializeOwned + Send + 'static,
    Res: Send + 'static,
    Cap: Send + 'static,
    Verify: Fn(Arc<R>, SessionToken) -> VerifyFut + Send + Sync + 'static,
    VerifyFut: Future<Output = Result<Cap, ErrorObjectOwned>> + Send + 'static,
    F: Fn(Arc<R>, Cap, Req) -> Fut + Send + Sync + 'static,
    Fut: Future<Output = Result<Res, ErrorObjectOwned>> + Send + 'static,
{
    let verify = Arc::new(verify);
    let f = Arc::new(f);

    move |params: SessionArgs<Req>| {
        let rpc = Arc::clone(&rpc);
        let verify = Arc::clone(&verify);
        let f = Arc::clone(&f);

        Box::pin(async move {
            let cap = (verify.as_ref())(Arc::clone(&rpc), params.session)
                .await
                .map_err(map_error_object_owned)?;
            (f.as_ref())(rpc, cap, params.args)
                .await
                .map_err(map_error_object_owned)
        })
    }
}

pub(crate) fn json_handler<R, F, Fut>(
    rpc: Arc<R>,
    f: F,
) -> impl Fn(Value) -> JsonHandlerFuture + Send + Sync + 'static
where
    R: Send + Sync + 'static,
    F: Fn(Arc<R>, Value) -> Fut + Send + Sync + 'static,
    Fut: Future<Output = Result<Value, RpcError>> + Send + 'static,
{
    move |params: Value| {
        let rpc = Arc::clone(&rpc);
        Box::pin(f(rpc, params))
    }
}

pub(crate) fn json_typed_handler<R, P, Res, F, Fut>(
    rpc: Arc<R>,
    f: F,
) -> impl Fn(Value) -> JsonHandlerFuture + Send + Sync + 'static
where
    R: Send + Sync + 'static,
    P: DeserializeOwned + Send + 'static,
    Res: Serialize + Send + 'static,
    F: Fn(Arc<R>, P) -> Fut + Send + Sync + 'static,
    Fut: Future<Output = Result<Res, RpcError>> + Send + 'static,
{
    let f = Arc::new(f);
    move |params: Value| {
        let rpc = Arc::clone(&rpc);
        let f = Arc::clone(&f);
        Box::pin(async move {
            let p: P = parse_params(params)?;
            let res = (f.as_ref())(rpc, p).await?;
            serialize_result(res)
        })
    }
}

pub(crate) fn json_typed_handler_jsonrpsee_err<R, P, Res, F, Fut>(
    rpc: Arc<R>,
    f: F,
) -> impl Fn(Value) -> JsonHandlerFuture + Send + Sync + 'static
where
    R: Send + Sync + 'static,
    P: DeserializeOwned + Send + 'static,
    Res: Serialize + Send + 'static,
    F: Fn(Arc<R>, P) -> Fut + Send + Sync + 'static,
    Fut: Future<Output = Result<Res, ErrorObjectOwned>> + Send + 'static,
{
    let f = Arc::new(f);
    json_typed_handler(rpc, move |rpc, p| {
        let f = Arc::clone(&f);
        async move { (f.as_ref())(rpc, p).await.map_err(map_error_object_owned) }
    })
}

pub(crate) fn parse_params<P>(params: Value) -> Result<P, RpcError>
where
    P: DeserializeOwned,
{
    JsonCodec
        .serialize(&params)
        .and_then(|bytes| JsonCodec.deserialize::<P>(&bytes))
        .map_err(|_| RpcError::InvalidParams("params deserialization failed".to_string()))
}

pub(crate) fn serialize_result<T>(result: T) -> Result<Value, RpcError>
where
    T: Serialize,
{
    JsonCodec
        .serialize(&result)
        .and_then(|bytes| JsonCodec.deserialize::<Value>(&bytes))
        .map_err(|_| RpcError::Internal("result serialization failed".to_string()))
}

#[cfg(test)]
mod tests {
    use super::parse_params;
    use jsonrpsee::types::ErrorObjectOwned;
    use serde::Deserialize;
    use z00z_networks_rpc::RpcError;
    use z00z_utils::codec::json;

    #[test]
    fn test_required_field_returns_invalid() {
        #[derive(Debug, Deserialize)]
        struct RequiredFieldParams {
            required: String,
        }

        let err = parse_params::<RequiredFieldParams>(json!({})).unwrap_err();
        match err {
            RpcError::InvalidParams(_) => {}
            other => panic!("expected InvalidParams, got {other:?}"),
        }

        let ok = parse_params::<RequiredFieldParams>(json!({"required": "x"})).unwrap();
        assert_eq!(ok.required, "x");
    }

    #[test]
    fn test_unknown_fields_ignored_default() {
        let params = json!({"wallet_id": "wallet-123", "extra": 1});
        let parsed = parse_params::<super::WalletIdParams>(params).expect("params should parse");
        assert_eq!(parsed.wallet_id.0, "wallet-123");
    }

    #[test]
    fn test_id_needs_wallet_id() {
        let err = parse_params::<super::WalletIdParams>(json!({"id": "wallet-abc"}))
            .expect_err("legacy id field must reject");
        match err {
            RpcError::InvalidParams(_) => {}
            other => panic!("expected InvalidParams, got {other:?}"),
        }

        let parsed = parse_params::<super::WalletIdParams>(json!({"wallet_id": "wallet-abc"}))
            .expect("params should parse with wallet_id");
        assert_eq!(parsed.wallet_id.0, "wallet-abc");
    }

    #[test]
    fn test_password_needs_wallet_id() {
        let err = parse_params::<super::WalletIdPasswordParams>(json!({
            "id": "wallet-abc",
            "password": "pw"
        }))
        .expect_err("legacy id field must reject");
        match err {
            RpcError::InvalidParams(_) => {}
            other => panic!("expected InvalidParams, got {other:?}"),
        }

        let parsed = parse_params::<super::WalletIdPasswordParams>(json!({
            "wallet_id": "wallet-abc",
            "password": "pw"
        }))
        .expect("params should parse with wallet_id");
        assert_eq!(parsed.wallet_id.0, "wallet-abc");
        assert_eq!(parsed.password, "pw");
    }

    #[test]
    fn test_preserves_rate_limit_class() {
        let err = super::map_error_object_owned(ErrorObjectOwned::owned(
            -32429,
            "Rate limit exceeded: too many requests",
            Some(json!({"retry_after_seconds": 42})),
        ));

        match err {
            RpcError::RateLimited(message) => {
                assert!(message.contains("retry_after_seconds"));
            }
            other => panic!("expected RateLimited, got {other:?}"),
        }
    }
}
