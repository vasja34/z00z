use jsonrpsee::{core::RpcResult, types::ErrorObjectOwned};
use z00z_core::{assets::registry::AssetId, Asset};

use crate::rpc::types::{
    asset::{RuntimeAssetDetailsResponse, RuntimeAssetMetadataResponse},
    common::RuntimeAssetRef,
};

pub(crate) fn runtime_asset_ref(asset: &Asset) -> RuntimeAssetRef {
    RuntimeAssetRef {
        asset_id: asset.asset_id(),
        serial_id: asset.serial_id,
        symbol: asset.definition.symbol.clone(),
        class: asset.definition.class,
    }
}

pub(crate) fn asset_matches_query_id(asset: &Asset, asset_id: AssetId) -> bool {
    asset.asset_id() == asset_id
}

pub(crate) fn build_metadata_from_asset(asset: &Asset) -> RuntimeAssetMetadataResponse {
    RuntimeAssetMetadataResponse {
        asset: runtime_asset_ref(asset),
        name: asset.definition.name.clone(),
        decimals: asset.definition.decimals,
        domain_name: asset.definition.domain_name.clone(),
        version: asset.definition.version,
        metadata: asset.definition.metadata.clone(),
    }
}

pub(crate) fn build_details_from_asset(asset: &Asset) -> RpcResult<RuntimeAssetDetailsResponse> {
    let total_supply = u64::from(asset.definition.serials)
        .checked_mul(asset.definition.nominal)
        .ok_or_else(|| {
            ErrorObjectOwned::owned(
                -32602,
                "Invalid asset definition: total_supply overflow".to_string(),
                None::<()>,
            )
        })?;

    Ok(RuntimeAssetDetailsResponse {
        asset: runtime_asset_ref(asset),
        definition: (*asset.definition).clone(),
        total_serials: asset.definition.serials,
        nominal_per_serial: asset.definition.nominal,
        total_supply,
        policy_flags: asset.definition.policy_flags,
        crypto_version: asset.definition.crypto_version,
    })
}
