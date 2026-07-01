use jsonrpsee::types::ErrorObjectOwned;
use z00z_core::{
    assets::{registry::AssetId, AssetWire},
    Asset,
};

use crate::{rpc::types::common::PersistWalletId, services::WalletService};

pub(crate) fn to_asset_wire(asset: &Asset) -> AssetWire {
    AssetWire::from_asset(asset)
}

pub(crate) async fn load_assets_from_storage(
    service: &WalletService,
    wallet_id: &PersistWalletId,
) -> Result<Vec<Asset>, ErrorObjectOwned> {
    service
        .list_claimed_assets_live_cache(wallet_id)
        .await
        .map_err(|error| {
            ErrorObjectOwned::owned(
                -32603,
                format!("wallet claimed assets error: {error}"),
                None::<()>,
            )
        })
}

pub(crate) async fn load_all_assets(
    service: &WalletService,
) -> Result<Vec<Asset>, ErrorObjectOwned> {
    service.list_claimed_all().await.map_err(|error| {
        ErrorObjectOwned::owned(
            -32603,
            format!("Failed to load assets: {error}"),
            None::<()>,
        )
    })
}

pub(crate) fn unknown_wallet_asset_err(label: &str) -> ErrorObjectOwned {
    ErrorObjectOwned::owned(-32602, label.to_string(), None::<()>)
}

pub(crate) async fn load_wallet_asset(
    service: &WalletService,
    wallet_id: &PersistWalletId,
    asset_id: AssetId,
    err_label: &str,
) -> Result<Asset, ErrorObjectOwned> {
    let stored = load_assets_from_storage(service, wallet_id).await?;
    stored
        .into_iter()
        .find(|asset| asset.asset_id() == asset_id)
        .ok_or_else(|| unknown_wallet_asset_err(err_label))
}

pub(crate) async fn load_wallet_assets_by_ids(
    service: &WalletService,
    wallet_id: &PersistWalletId,
    asset_ids: &[AssetId],
) -> Result<Vec<Asset>, ErrorObjectOwned> {
    let stored = load_assets_from_storage(service, wallet_id).await?;
    Ok(stored
        .into_iter()
        .filter(|asset| {
            asset_ids
                .iter()
                .any(|asset_id| asset.asset_id() == *asset_id)
        })
        .collect())
}
