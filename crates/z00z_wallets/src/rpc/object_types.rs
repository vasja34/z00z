use crate::db::redb_store::{OwnedAssetSource, OwnedAssetStatus};
use crate::db::{
    OwnedObjectFamily, OwnedObjectSource, OwnedRightStatus, OwnedVoucherStatus,
    WalletPolicyAvailability,
};
use crate::rpc::types::common::{PersistTxId, PersistWalletId, RuntimePaginatedResponse};
use serde::{Deserialize, Serialize};
use z00z_core::{actions::ActionPoolDescriptorV1, assets::AssetWire, policies::PolicyDescriptorV1};
use z00z_storage::settlement::{
    ObjectDeltaSetV1, ObjectValidatorVerdict, ObjectWitnessBundleV1, RightLeaf, RightWitnessRefV1,
    RuntimeObjectPackageV1, SettlementActionV1, VoucherLeaf,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimeObjectListFilter {
    pub account_id: Option<u128>,
    pub family: Option<OwnedObjectFamily>,
    pub policy_availability: Option<WalletPolicyAvailability>,
    pub holder_commitment_hex: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimeObjectPolicyState {
    pub policy_id_hex: Option<String>,
    pub availability: WalletPolicyAvailability,
    pub manual_review: bool,
    pub quarantine_reason: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimeAssetObjectDetail {
    pub status: OwnedAssetStatus,
    pub source: OwnedAssetSource,
    pub asset_wire: AssetWire,
    pub spend_ref: Option<PersistTxId>,
    pub quarantined: bool,
    pub quarantine_reason: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimeVoucherObjectDetail {
    pub status: OwnedVoucherStatus,
    pub source: OwnedObjectSource,
    pub voucher_leaf: VoucherLeaf,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimeRightObjectDetail {
    pub status: OwnedRightStatus,
    pub source: OwnedObjectSource,
    pub right_leaf: RightLeaf,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimeObjectRecord {
    pub object_id: Option<u128>,
    pub wallet_id: PersistWalletId,
    pub account_id: Option<u128>,
    pub family: OwnedObjectFamily,
    pub stable_id_hex: String,
    pub labels: Vec<String>,
    pub last_updated_ms: u64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub policy: Option<RuntimeObjectPolicyState>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub asset: Option<RuntimeAssetObjectDetail>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub voucher: Option<RuntimeVoucherObjectDetail>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub right: Option<RuntimeRightObjectDetail>,
}

pub type RuntimeListObjectsResponse = RuntimePaginatedResponse<RuntimeObjectRecord>;
pub type RuntimeListVoucherClaimsResponse = RuntimePaginatedResponse<RuntimeObjectRecord>;
pub type RuntimeListRightInventoryResponse = RuntimePaginatedResponse<RuntimeObjectRecord>;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimeObjectPackageRequest {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub stable_id_hex: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub issue_asset_id_hex: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub issue_reserve_hex: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub create_terminal_id_hex: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub selected_action: Option<SettlementActionV1>,
    pub action_label: String,
    pub policy_descriptor: PolicyDescriptorV1,
    pub action_pool: ActionPoolDescriptorV1,
    #[serde(default)]
    pub required_rights: Vec<RightWitnessRefV1>,
    pub object_witnesses: ObjectWitnessBundleV1,
    pub delta_set: ObjectDeltaSetV1,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimeObjectPackagePreviewResponse {
    pub stable_id_hex: String,
    pub family: OwnedObjectFamily,
    pub action_label: String,
    pub package: RuntimeObjectPackageV1,
    pub verdict: ObjectValidatorVerdict,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimeObjectPackageBuildResponse {
    pub stable_id_hex: String,
    pub family: OwnedObjectFamily,
    pub action_label: String,
    pub package: RuntimeObjectPackageV1,
}
