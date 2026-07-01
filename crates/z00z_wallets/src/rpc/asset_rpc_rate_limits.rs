use std::{collections::BTreeMap, sync::Arc};

use tokio::sync::RwLock;

use crate::rpc::types::common::PersistWalletId;

pub(crate) const ASSET_SEND_RATE_LIMIT_MAX: u32 = 10;
pub(crate) const ASSET_SEND_RATE_LIMIT_WINDOW: u32 = 60;

pub(crate) type AssetSendRateLimitMap =
    Arc<RwLock<BTreeMap<PersistWalletId, AssetSendRateLimitState>>>;

#[derive(Debug, Clone)]
pub(crate) struct AssetSendRateLimitState {
    pub(crate) window_start_ms: u64,
    pub(crate) current_count: u32,
}

pub(crate) async fn asset_send_precheck(
    rate_limits: &AssetSendRateLimitMap,
    now_ms: u64,
    wallet_id: &PersistWalletId,
) -> Result<(), (u32, u32, u32)> {
    let window_ms = u64::from(ASSET_SEND_RATE_LIMIT_WINDOW).saturating_mul(1_000);

    let mut limits = rate_limits.write().await;
    let entry = limits
        .entry(wallet_id.clone())
        .or_insert_with(|| AssetSendRateLimitState {
            window_start_ms: now_ms,
            current_count: 0,
        });

    if now_ms.saturating_sub(entry.window_start_ms) >= window_ms {
        entry.window_start_ms = now_ms;
        entry.current_count = 0;
    }

    if entry.current_count >= ASSET_SEND_RATE_LIMIT_MAX {
        let elapsed_ms = now_ms.saturating_sub(entry.window_start_ms);
        let retry_after_seconds = window_ms.saturating_sub(elapsed_ms).div_ceil(1_000) as u32;
        return Err((
            retry_after_seconds,
            entry.current_count,
            ASSET_SEND_RATE_LIMIT_MAX,
        ));
    }

    entry.current_count = entry.current_count.saturating_add(1);
    Ok(())
}
