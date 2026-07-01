//! In-process RPC transport for native builds.
//!
//! This transport calls a shared [`RpcDispatcher`] directly.
//! It exists as a local testing helper and in-process adapter; it does not own
//! remote peer identity, authentication, retry policy, or connection lifecycle.

use crate::{RpcDispatcher, RpcError, RpcTransport};
use async_trait::async_trait;
use std::sync::Arc;
use z00z_utils::codec::Value;

/// Local (in-process) RPC transport.
#[derive(Clone)]
pub struct LocalRpcTransport {
    dispatcher: Arc<RpcDispatcher>,
}

impl LocalRpcTransport {
    /// Create a new local transport backed by the provided dispatcher.
    pub fn new(dispatcher: Arc<RpcDispatcher>) -> Self {
        Self { dispatcher }
    }

    /// Get the underlying dispatcher.
    pub fn dispatcher(&self) -> &Arc<RpcDispatcher> {
        &self.dispatcher
    }
}

#[async_trait(?Send)]
impl RpcTransport for LocalRpcTransport {
    async fn call(&self, method: &str, params: Value) -> Result<Value, RpcError> {
        self.dispatcher.dispatch(method, params).await
    }
}
