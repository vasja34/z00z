use std::sync::Arc;

use tokio::sync::RwLock;

use crate::rpc::types::common::PersistWalletId;

pub(crate) const TX_SEND_RATE_LIMIT_MAX: u32 = 10;
pub(crate) const TX_SEND_RATE_LIMIT_WINDOW: u32 = 60;
pub(crate) const BUILD_TX_RATE_LIMIT_MAX: u32 = 20;
pub(crate) const BUILD_TX_RATE_LIMIT_WINDOW: u32 = 60;

pub(crate) type TxSendRateLimitStore = Arc<RwLock<Vec<TxSendRateLimitEntry>>>;
pub(crate) type TxBuildRateLimitStore = Arc<RwLock<Vec<TxBuildRateLimitEntry>>>;

#[derive(Debug, Clone)]
pub(crate) struct TxSendRateLimitEntry {
    pub(crate) wallet_id: PersistWalletId,
    pub(crate) window_start_ms: u64,
    pub(crate) current_count: u32,
}

#[derive(Debug, Clone)]
pub(crate) struct TxBuildRateLimitEntry {
    pub(crate) wallet_id: PersistWalletId,
    pub(crate) window_start_ms: u64,
    pub(crate) current_count: u32,
}

pub(crate) async fn tx_send_precheck(
    rate_limits: &TxSendRateLimitStore,
    now_ms: u64,
    wallet_id: &PersistWalletId,
) -> Result<(), (u32, u32, u32)> {
    let window_ms = u64::from(TX_SEND_RATE_LIMIT_WINDOW).saturating_mul(1_000);

    let mut limits = rate_limits.write().await;
    let entry = if let Some(existing) = limits
        .iter_mut()
        .find(|entry| &entry.wallet_id == wallet_id)
    {
        existing
    } else {
        limits.push(TxSendRateLimitEntry {
            wallet_id: wallet_id.clone(),
            window_start_ms: now_ms,
            current_count: 0,
        });

        match limits.last_mut() {
            Some(entry) => entry,
            None => return Ok(()),
        }
    };

    if now_ms.saturating_sub(entry.window_start_ms) >= window_ms {
        entry.window_start_ms = now_ms;
        entry.current_count = 0;
    }

    if entry.current_count >= TX_SEND_RATE_LIMIT_MAX {
        let elapsed_ms = now_ms.saturating_sub(entry.window_start_ms);
        let retry_after_seconds = window_ms.saturating_sub(elapsed_ms).div_ceil(1_000) as u32;
        return Err((
            retry_after_seconds,
            entry.current_count,
            TX_SEND_RATE_LIMIT_MAX,
        ));
    }

    entry.current_count = entry.current_count.saturating_add(1);
    Ok(())
}

pub(crate) async fn tx_build_precheck(
    rate_limits: &TxBuildRateLimitStore,
    now_ms: u64,
    wallet_id: &PersistWalletId,
) -> Result<(), (u32, u32, u32)> {
    let window_ms = u64::from(BUILD_TX_RATE_LIMIT_WINDOW).saturating_mul(1_000);

    let mut limits = rate_limits.write().await;
    let entry = if let Some(existing) = limits
        .iter_mut()
        .find(|entry| &entry.wallet_id == wallet_id)
    {
        existing
    } else {
        limits.push(TxBuildRateLimitEntry {
            wallet_id: wallet_id.clone(),
            window_start_ms: now_ms,
            current_count: 0,
        });

        match limits.last_mut() {
            Some(entry) => entry,
            None => return Ok(()),
        }
    };

    if now_ms.saturating_sub(entry.window_start_ms) >= window_ms {
        entry.window_start_ms = now_ms;
        entry.current_count = 0;
    }

    if entry.current_count >= BUILD_TX_RATE_LIMIT_MAX {
        let elapsed_ms = now_ms.saturating_sub(entry.window_start_ms);
        let retry_after_seconds = window_ms.saturating_sub(elapsed_ms).div_ceil(1_000) as u32;
        return Err((
            retry_after_seconds,
            entry.current_count,
            BUILD_TX_RATE_LIMIT_MAX,
        ));
    }

    entry.current_count = entry.current_count.saturating_add(1);
    Ok(())
}
