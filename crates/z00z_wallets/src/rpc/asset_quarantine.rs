use std::{collections::BTreeMap, sync::Arc};

use tokio::sync::RwLock;

use crate::rpc::types::common::PersistWalletId;

pub(crate) type AssetQuarantineStore = Arc<RwLock<BTreeMap<PersistWalletId, Vec<[u8; 32]>>>>;

pub(crate) async fn quarantine_ids(
    quarantined: &AssetQuarantineStore,
    wallet_id: &PersistWalletId,
) -> Vec<[u8; 32]> {
    let map = quarantined.read().await;
    map.get(wallet_id).cloned().unwrap_or_default()
}

pub(crate) async fn mark_quarantine(
    quarantined: &AssetQuarantineStore,
    wallet_id: &PersistWalletId,
    asset_id: [u8; 32],
) {
    let mut map = quarantined.write().await;
    let list = map.entry(wallet_id.clone()).or_insert_with(Vec::new);
    if !list.iter().any(|item| item == &asset_id) {
        list.push(asset_id);
    }
}
