//! RPC request dispatcher.
//!
//! The dispatcher owns method routing only. Authentication, peer identity,
//! retry policy, and transport connection lifecycle remain external inputs to
//! the callers that register handlers here.

use crate::error::RpcError;
use std::{
    collections::HashMap,
    future::Future,
    pin::Pin,
    sync::{Arc, RwLock},
};
use z00z_utils::codec::{Codec, JsonCodec, Value};

type HandlerFuture = Pin<Box<dyn Future<Output = Result<Value, RpcError>> + Send + 'static>>;
type HandlerFn = dyn Fn(Value) -> HandlerFuture + Send + Sync + 'static;

/// Dispatches RPC requests to appropriate service handlers.
pub struct RpcDispatcher {
    handlers: RwLock<HashMap<String, Arc<HandlerFn>>>,
}

impl RpcDispatcher {
    /// Create new dispatcher
    pub fn new() -> Self {
        Self {
            handlers: RwLock::new(HashMap::new()),
        }
    }

    /// Register a raw JSON handler for a method.
    ///
    /// The handler receives `params` as a JSON value and returns a JSON value result.
    pub fn register_method<F, Fut>(&self, method: impl Into<String>, handler: F)
    where
        F: Fn(Value) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Result<Value, RpcError>> + Send + 'static,
    {
        let method = method.into();
        let handler = Arc::new(move |params: Value| -> HandlerFuture { Box::pin(handler(params)) });
        if let Ok(mut handlers) = self.handlers.write() {
            handlers.insert(method, handler);
        }
    }

    /// Register a strongly-typed handler (serde-deserializes params, serde-serializes result).
    pub fn register_typed<Req, Res, F, Fut>(&self, method: impl Into<String>, handler: F)
    where
        Req: serde::de::DeserializeOwned + Send + 'static,
        Res: serde::Serialize + Send + 'static,
        F: Fn(Req) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Result<Res, RpcError>> + Send + 'static,
    {
        let handler = Arc::new(handler);
        self.register_method(method, move |params| {
            let handler = Arc::clone(&handler);
            async move {
                let codec = JsonCodec;
                let params_bytes = codec.serialize(&params).map_err(|e| {
                    RpcError::InvalidParams(format!("params serialization failed: {e}"))
                })?;
                let req: Req = codec.deserialize(&params_bytes).map_err(|e| {
                    RpcError::InvalidParams(format!("params deserialization failed: {e}"))
                })?;
                let res = (handler)(req).await?;
                let res_bytes = codec
                    .serialize(&res)
                    .map_err(|e| RpcError::Internal(format!("result serialization failed: {e}")))?;
                codec
                    .deserialize(&res_bytes)
                    .map_err(|e| RpcError::Internal(format!("result deserialization failed: {e}")))
            }
        });
    }

    /// Dispatch single RPC request
    pub async fn dispatch(&self, method: &str, params: Value) -> Result<Value, RpcError> {
        let handler = self
            .handlers
            .read()
            .map_err(|_| RpcError::Internal("dispatcher lock poisoned".to_string()))?
            .get(method)
            .cloned();

        let Some(handler) = handler else {
            return Err(RpcError::MethodNotFound(method.to_string()));
        };

        (handler)(params).await
    }
}

impl Default for RpcDispatcher {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use z00z_utils::codec::{Map, Value};

    #[derive(serde::Deserialize)]
    struct AddParams {
        a: i64,
        b: i64,
    }

    #[tokio::test]
    async fn test_dispatch_calls_typed_handler() {
        let dispatcher = RpcDispatcher::new();
        dispatcher.register_typed("math.add", |p: AddParams| async move { Ok(p.a + p.b) });

        let mut map = Map::new();
        map.insert("a".to_string(), Value::Number(2.into()));
        map.insert("b".to_string(), Value::Number(3.into()));
        let params = Value::Object(map);

        let result = dispatcher.dispatch("math.add", params).await.unwrap();

        assert_eq!(result, Value::Number(5.into()));
    }

    #[tokio::test]
    async fn test_dispatch_unknown_method_returns() {
        let dispatcher = RpcDispatcher::new();

        let empty = Value::Object(Map::new());
        let err = dispatcher.dispatch("nope", empty).await.unwrap_err();

        match err {
            RpcError::MethodNotFound(m) => assert_eq!(m, "nope"),
            other => panic!("unexpected error: {other:?}"),
        }
    }
}
