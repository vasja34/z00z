use std::sync::Arc;

use hex::{decode_to_slice, encode};
use tokio::sync::RwLock;
use z00z_core::assets::registry::AssetId;

pub(crate) type AssetStakeCounter = Arc<RwLock<u64>>;

const STAKE_PREFIX: &str = "stake_";

pub(crate) async fn next_stake_id(
    counter: &AssetStakeCounter,
    now_ms: u64,
    asset_id: AssetId,
    amount: u64,
) -> String {
    let mut counter = counter.write().await;
    *counter = counter.saturating_add(1);
    format!(
        "{STAKE_PREFIX}{now_ms}_{}_{asset_hex}_{amount}",
        *counter,
        asset_hex = encode(asset_id),
    )
}

pub(crate) fn parse_stake_id(stake_id: &str) -> Option<(AssetId, u64)> {
    let suffix = stake_id.strip_prefix(STAKE_PREFIX)?;
    let mut parts = suffix.splitn(4, '_');
    let issued_at_ms = parts.next()?;
    let counter = parts.next()?;
    let asset_hex = parts.next()?;
    let amount = parts.next()?;

    let _ = issued_at_ms.parse::<u64>().ok()?;
    let _ = counter.parse::<u64>().ok()?;

    let mut asset_id = [0u8; 32];
    decode_to_slice(asset_hex, &mut asset_id).ok()?;
    let amount = amount.parse::<u64>().ok()?;
    Some((asset_id, amount))
}
