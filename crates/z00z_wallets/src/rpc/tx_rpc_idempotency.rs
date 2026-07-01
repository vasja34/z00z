use std::sync::Arc;

use tokio::sync::RwLock;
use z00z_utils::codec::{Codec, JsonCodec};

use crate::rpc::types::{
    common::PersistWalletId,
    security::{IdempotencyKey, RuntimeIdempotencyCacheEntry},
    tx::RuntimeSendTxResponse,
};

pub(crate) const TX_SEND_IDEMPOTENCY_TTL_MS: u64 = 10 * 60 * 1_000;
pub(crate) type TxIdempotencyCache = Arc<RwLock<Vec<RuntimeIdempotencyCacheEntry>>>;

pub(crate) async fn idempotency_get(
    cache: &TxIdempotencyCache,
    now_ms: u64,
    wallet_id: &PersistWalletId,
    key: &IdempotencyKey,
) -> Option<RuntimeSendTxResponse> {
    let mut cache = cache.write().await;
    cache.retain(|entry| entry.expires_at > now_ms);

    let entry = cache
        .iter()
        .find(|entry| {
            &entry.wallet_id == wallet_id
                && &entry.key == key
                && entry.method == "wallet.tx.send_transaction"
        })
        .cloned()?;

    JsonCodec.deserialize(entry.response.as_bytes()).ok()
}

pub(crate) async fn idempotency_put(
    cache: &TxIdempotencyCache,
    now_ms: u64,
    wallet_id: &PersistWalletId,
    key: IdempotencyKey,
    resp: &RuntimeSendTxResponse,
) {
    let expires_at = now_ms.saturating_add(TX_SEND_IDEMPOTENCY_TTL_MS);
    let response =
        String::from_utf8(JsonCodec.serialize(resp).unwrap_or_default()).unwrap_or_default();

    let mut cache = cache.write().await;
    cache.retain(|entry| entry.expires_at > now_ms);

    if let Some(existing) = cache.iter_mut().find(|entry| {
        &entry.wallet_id == wallet_id
            && entry.method == "wallet.tx.send_transaction"
            && entry.key == key
    }) {
        existing.response = response;
        existing.created_at = now_ms;
        existing.expires_at = expires_at;
        return;
    }

    cache.push(RuntimeIdempotencyCacheEntry {
        key,
        wallet_id: wallet_id.clone(),
        method: "wallet.tx.send_transaction".to_string(),
        response,
        created_at: now_ms,
        expires_at,
    });
}
