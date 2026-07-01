use super::{
    asset_rpc_balance, asset_rpc_caches, asset_rpc_registry, Asset, AssetId, AssetRpcImpl,
    AssetWire, ErrorObjectOwned, PersistWalletId, ReceiveReject, RpcResult,
    RuntimeAssetDetailsResponse, RuntimeAssetMetadataResponse, RuntimeAssetRef,
    ASSET_LIST_CACHE_MAX_WALLETS, ASSET_LIST_CACHE_TTL_MS, ASSET_METADATA_CACHE_MAX_ENTRIES,
    ASSET_METADATA_CACHE_TTL_MS,
};

impl AssetRpcImpl {
    pub(super) fn to_asset_wire(asset: &z00z_core::assets::Asset) -> AssetWire {
        asset_rpc_registry::to_asset_wire(asset)
    }

    pub(super) async fn load_assets_from_storage(
        &self,
        wallet_id: &PersistWalletId,
    ) -> Result<Vec<z00z_core::assets::Asset>, ErrorObjectOwned> {
        asset_rpc_registry::load_assets_from_storage(&self.service, wallet_id).await
    }

    pub(super) async fn load_all_assets(
        &self,
    ) -> Result<Vec<z00z_core::assets::Asset>, ErrorObjectOwned> {
        asset_rpc_registry::load_all_assets(&self.service).await
    }

    pub(super) fn unknown_wallet_asset_err(label: &str) -> ErrorObjectOwned {
        asset_rpc_registry::unknown_wallet_asset_err(label)
    }

    pub(super) async fn load_wallet_asset(
        &self,
        wallet_id: &PersistWalletId,
        asset_id: AssetId,
        err_label: &str,
    ) -> Result<Asset, ErrorObjectOwned> {
        asset_rpc_registry::load_wallet_asset(&self.service, wallet_id, asset_id, err_label).await
    }

    pub(super) async fn load_wallet_assets_by_ids(
        &self,
        wallet_id: &PersistWalletId,
        asset_ids: &[AssetId],
    ) -> Result<Vec<Asset>, ErrorObjectOwned> {
        asset_rpc_registry::load_wallet_assets_by_ids(&self.service, wallet_id, asset_ids).await
    }

    pub(super) fn runtime_asset_ref(asset: &Asset) -> RuntimeAssetRef {
        asset_rpc_balance::runtime_asset_ref(asset)
    }

    pub(super) fn asset_matches_query_id(asset: &Asset, asset_id: AssetId) -> bool {
        asset_rpc_balance::asset_matches_query_id(asset, asset_id)
    }

    pub(super) fn build_details_from_asset(
        asset: &Asset,
    ) -> RpcResult<RuntimeAssetDetailsResponse> {
        asset_rpc_balance::build_details_from_asset(asset)
    }
    pub(super) async fn get_cached_assets(&self, wallet_id: &PersistWalletId) -> Vec<AssetWire> {
        asset_rpc_caches::get_cached_assets(
            &self.asset_list_cache,
            self.now_ms(),
            wallet_id,
            ASSET_LIST_CACHE_TTL_MS,
            ASSET_LIST_CACHE_MAX_WALLETS,
        )
        .await
    }

    pub(super) async fn lookup_receive_asset(
        &self,
        wallet_id: &PersistWalletId,
        asset_id: AssetId,
    ) -> Result<Asset, ErrorObjectOwned> {
        let cached = self.get_cached_assets(wallet_id).await;
        if let Some(asset) = cached.iter().find_map(|item| {
            let asset = item.clone().to_asset().ok()?;
            Self::asset_matches_query_id(&asset, asset_id).then_some(asset)
        }) {
            return Ok(asset);
        }

        let wallet_assets = self.load_assets_from_storage(wallet_id).await?;
        if let Some(asset) = wallet_assets
            .iter()
            .find(|item| Self::asset_matches_query_id(item, asset_id))
        {
            return Ok(asset.clone());
        }

        if cached.iter().any(|item| item.definition.id == asset_id)
            || wallet_assets
                .iter()
                .any(|item| item.definition.id == asset_id)
        {
            return Err(Self::recv_err(ReceiveReject::InvalidInput));
        }

        Err(Self::recv_err(ReceiveReject::NotMine))
    }

    pub(super) async fn get_cached_metadata(
        &self,
        asset_id: &AssetId,
    ) -> RpcResult<RuntimeAssetMetadataResponse> {
        let stored = self.load_all_assets().await?;
        asset_rpc_caches::get_cached_metadata(
            &self.asset_metadata_cache,
            self.now_ms(),
            asset_id,
            ASSET_METADATA_CACHE_TTL_MS,
            ASSET_METADATA_CACHE_MAX_ENTRIES,
            &stored,
            asset_rpc_balance::build_metadata_from_asset,
        )
        .await
    }
}
