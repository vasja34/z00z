//! Asset RPC types matching z00z_core::assets structures.
//!
//! `wallet.asset.*` is the cash-only projection namespace. Voucher and Right
//! inventory must stay on `wallet.object.*`, and voucher/right ids must be
//! rejected instead of being projected as spendable value.
//!
//! All asset domain types (AssetWire/AssetDefinition/AssetId/AssetClass) must
//! come from z00z_core.

use crate::rpc::types::common::{
    PersistTxId, RuntimeAssetAmount, RuntimeAssetRef, RuntimeOperationStatus,
    RuntimePaginatedAssetsResponse,
};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

use z00z_core::assets::registry::AssetId;
use z00z_core::assets::{AssetClass, AssetDefinition, AssetWire};

/// Asset list filter for asset.list_assets
///
/// Optional filters to narrow down asset list.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimeAssetListFilter {
    /// Filter by asset class (Coin/Token/Nft/Void)
    pub asset_class: Option<AssetClass>,
    /// Filter by minimum balance (amount)
    pub min_balance: Option<u64>,
}

/// Response for asset.list_assets method.
///
/// This is a paginated response wrapper serialized with an `assets` field.
pub type RuntimeListAssetsResponse = RuntimePaginatedAssetsResponse<AssetWire>;

/// Asset balance response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimeAssetBalanceResponse {
    #[serde(flatten)]
    pub asset: RuntimeAssetRef,
    pub total: u64,
    pub available: u64,
    pub pending: u64,
    pub decimals: u8,
}

/// Asset metadata response (from AssetDefinition)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimeAssetMetadataResponse {
    #[serde(flatten)]
    pub asset: RuntimeAssetRef,
    pub name: String,
    pub decimals: u8,
    pub domain_name: String,
    pub version: u8,
    pub metadata: Option<BTreeMap<String, String>>,
}

/// Detailed asset information response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimeAssetDetailsResponse {
    #[serde(flatten)]
    pub asset: RuntimeAssetRef,
    pub definition: AssetDefinition,
    pub total_serials: u32,
    pub nominal_per_serial: u64,
    pub total_supply: u64,
    pub policy_flags: u8,
    pub crypto_version: u8,
}

/// Import asset response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimeImportAssetResponse {
    #[serde(flatten)]
    pub asset: RuntimeAssetRef,
    #[serde(flatten)]
    pub status: RuntimeOperationStatus,
    pub is_inserted: bool,
    pub asset_already_exists: bool,
}

/// Merge assets response (combine multiple asset commitments)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimeMergeAssetsResponse {
    #[serde(flatten)]
    pub asset: RuntimeAssetRef,
    pub merged_count: usize,
    pub total_amount: u64,
    pub tx_id: Option<PersistTxId>,
}

/// Split asset response (create multiple asset commitments from one)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimeSplitAssetResponse {
    pub original_asset_id: AssetId,
    pub splits: Vec<RuntimeAssetAmount>,
    pub tx_id: Option<PersistTxId>,
}

/// Stake assets response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimeStakeAssetsResponse {
    pub stake_id: String,
    #[serde(flatten)]
    pub asset: RuntimeAssetRef,
    pub amount: u64,
    /// milliseconds since Unix epoch
    pub start_time: u64,
    /// milliseconds since Unix epoch
    pub end_time: u64,
    pub apy: f64,
}

/// Unstake assets response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimeUnstakeAssetsResponse {
    pub stake_id: String,
    #[serde(flatten)]
    pub asset: RuntimeAssetRef,
    pub amount: u64,
    pub reward: u64,
    /// milliseconds since Unix epoch
    pub unstaked_at: u64,
}

/// Swap assets response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimeSwapAssetsResponse {
    pub from_asset_id: AssetId,
    pub from_serial_id: u32,
    pub from_symbol: String,
    pub from_class: AssetClass,
    pub to_asset_id: AssetId,
    pub to_serial_id: u32,
    pub to_symbol: String,
    pub to_class: AssetClass,
    pub from_amount: u64,
    pub to_amount: u64,
    pub exchange_rate: f64,
    pub fee: u64,
    pub tx_id: PersistTxId,
}

/// Send asset response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimeSendAssetResponse {
    pub tx_id: PersistTxId,
    #[serde(flatten)]
    pub asset: RuntimeAssetRef,
    pub owner_handle: String,
    pub amount: u64,
    pub recipient: String,
    pub fee: u64,
    pub status: String,
}

/// Receive asset response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimeReceiveAssetResponse {
    #[serde(flatten)]
    pub asset: RuntimeAssetRef,
    /// Stable receive status vocabulary.
    pub status: String,
    pub owner_handle: String,
    pub view_key: String,
    /// milliseconds since Unix epoch
    pub expires_at: Option<u64>,
}

/// Add asset response (import to wallet)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimeAddAssetResponse {
    #[serde(flatten)]
    pub asset: RuntimeAssetRef,
    #[serde(flatten)]
    pub status: RuntimeOperationStatus,
}

#[cfg(test)]
mod tests {
    use super::RuntimeListAssetsResponse;
    use z00z_utils::codec::{Codec, JsonCodec, Value};

    #[test]
    fn test_asset_list_schema_flat() {
        let response = RuntimeListAssetsResponse {
            items: Vec::new(),
            next_cursor: None,
            has_more: false,
            total_count: Some(0),
        };

        let value = JsonCodec
            .serialize(&response)
            .and_then(|bytes| JsonCodec.deserialize::<Value>(&bytes))
            .expect("serialization must succeed");
        let obj = value.as_object().expect("must serialize to JSON object");

        assert!(obj.contains_key("assets"), "must keep `assets` field");
        assert!(!obj.contains_key("items"), "must not expose `items` field");
        assert!(obj.contains_key("next_cursor"));
        assert!(obj.contains_key("has_more"));
        assert!(obj.contains_key("total_count"));
    }
}
