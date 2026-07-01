use std::{collections::BTreeMap, sync::Arc};

use jsonrpsee::{core::RpcResult, types::ErrorObjectOwned};
use tokio::sync::RwLock;
use z00z_core::assets::AssetWire;
use z00z_core::{assets::registry::AssetId, Asset};

use crate::rpc::types::{asset::RuntimeAssetMetadataResponse, common::PersistWalletId};

pub(crate) type AssetListCache = Arc<RwLock<BTreeMap<PersistWalletId, AssetListCacheValue>>>;
pub(crate) type AssetMetadataCache = Arc<RwLock<BTreeMap<AssetId, AssetMetadataCacheValue>>>;

#[derive(Debug, Clone)]
pub(crate) struct AssetListCacheValue {
    pub(crate) created_at: u64,
    pub(crate) assets: Vec<AssetWire>,
}

#[derive(Debug, Clone)]
pub(crate) struct AssetMetadataCacheValue {
    pub(crate) created_at: u64,
    pub(crate) metadata: RuntimeAssetMetadataResponse,
}

pub(crate) async fn get_cached_assets(
    cache: &AssetListCache,
    now_ms: u64,
    wallet_id: &PersistWalletId,
    ttl_ms: u64,
    max_wallets: usize,
) -> Vec<AssetWire> {
    {
        let cache_guard = cache.read().await;
        if let Some(entry) = cache_guard.get(wallet_id) {
            if now_ms.saturating_sub(entry.created_at) <= ttl_ms {
                return entry.assets.clone();
            }

            let stale = entry.assets.clone();
            drop(cache_guard);

            let mut cache = cache.write().await;
            cache.insert(
                wallet_id.clone(),
                AssetListCacheValue {
                    created_at: now_ms,
                    assets: stale.clone(),
                },
            );
            return stale;
        }
    }

    {
        let mut cache = cache.write().await;
        cache.insert(
            wallet_id.clone(),
            AssetListCacheValue {
                created_at: now_ms,
                assets: Vec::new(),
            },
        );

        if cache.len() > max_wallets {
            if let Some(oldest_key) = cache
                .iter()
                .min_by_key(|(_key, value)| value.created_at)
                .map(|(key, _value)| key.clone())
            {
                cache.remove(&oldest_key);
            }
        }
    }

    Vec::new()
}

pub(crate) async fn get_cached_metadata(
    cache: &AssetMetadataCache,
    now_ms: u64,
    asset_id: &AssetId,
    ttl_ms: u64,
    max_entries: usize,
    stored: &[Asset],
    build_metadata: fn(&Asset) -> RuntimeAssetMetadataResponse,
) -> RpcResult<RuntimeAssetMetadataResponse> {
    {
        let cache = cache.read().await;
        if let Some(entry) = cache.get(asset_id) {
            if now_ms.saturating_sub(entry.created_at) <= ttl_ms {
                return Ok(entry.metadata.clone());
            }
        }
    }

    let Some(asset) = stored.iter().find(|item| item.asset_id() == *asset_id) else {
        return Err(ErrorObjectOwned::owned(
            -32602,
            "Unknown asset_id".to_string(),
            None::<()>,
        ));
    };

    let metadata = build_metadata(asset);
    let canonical_id = metadata.asset.asset_id;

    {
        let mut cache = cache.write().await;
        cache.insert(
            canonical_id,
            AssetMetadataCacheValue {
                created_at: now_ms,
                metadata: metadata.clone(),
            },
        );

        if cache.len() > max_entries {
            if let Some(oldest_key) = cache
                .iter()
                .min_by_key(|(_key, value)| value.created_at)
                .map(|(key, _value)| *key)
            {
                cache.remove(&oldest_key);
            }
        }
    }

    Ok(metadata)
}
