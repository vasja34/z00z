use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};
use z00z_crypto::DomainHasher;
use z00z_utils::codec::to_canonical_json_bytes;

use crate::{
    config_name::validate_underscore_name, domains::ActionPoolDescriptorHashDomain, AssetError,
    ObjectFamily,
};

use super::{
    ActionDescriptorV1, ActionId, ActionPoolId, LifecycleEffectV1, RequiredSignatureV1,
    WitnessRequirementV1,
};

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ActionPoolDescriptorV1 {
    pub label: String,
    pub actions: BTreeSet<ActionDescriptorV1>,
}

impl ActionPoolDescriptorV1 {
    pub fn validate(&self) -> Result<(), AssetError> {
        if self.label.trim().is_empty() {
            return Err(AssetError::InvalidAsset(
                "action pool label must not be empty".into(),
            ));
        }
        validate_underscore_name("action_pool.label", self.label.as_str())?;

        if self.actions.is_empty() {
            return Err(AssetError::InvalidAsset(
                "action pool must contain at least one action".into(),
            ));
        }

        let mut seen_ids = BTreeSet::new();
        for action in &self.actions {
            action.validate()?;
            let action_id = action.action_id()?;
            if !seen_ids.insert(action_id) {
                return Err(AssetError::InvalidAsset(
                    format!("duplicate action id in action pool: {action_id}").into(),
                ));
            }
        }

        Ok(())
    }

    pub fn canonical_bytes(&self) -> Result<Vec<u8>, AssetError> {
        self.validate()?;
        to_canonical_json_bytes(self)
            .map_err(|err| AssetError::Serialization(err.to_string().into()))
    }

    pub fn action_pool_id(&self) -> Result<ActionPoolId, AssetError> {
        let bytes = self.canonical_bytes()?;
        let mut hasher = DomainHasher::<ActionPoolDescriptorHashDomain>::new_with_label("pool");
        hasher.update(bytes);
        let digest = hasher.finalize();
        let mut id = [0u8; 32];
        id.copy_from_slice(&digest.as_ref()[..32]);
        Ok(ActionPoolId::new(id))
    }

    pub fn action_ids(&self) -> Result<BTreeSet<ActionId>, AssetError> {
        self.actions
            .iter()
            .map(ActionDescriptorV1::action_id)
            .collect()
    }
}

#[must_use]
pub fn fixed_cash_action_pool_descriptor() -> ActionPoolDescriptorV1 {
    let owner_only = BTreeSet::from([WitnessRequirementV1::Signature(RequiredSignatureV1::Owner)]);

    let actions = BTreeSet::from([
        ActionDescriptorV1 {
            label: "cash_merge".to_string(),
            allowed_input_families: BTreeSet::from([ObjectFamily::Asset]),
            allowed_output_families: BTreeSet::from([ObjectFamily::Asset]),
            lifecycle_effect: LifecycleEffectV1::NoStateChange,
            witness_requirements: owner_only.clone(),
            receiver_must_accept: false,
            preserves_beneficiary: true,
            preserves_refund_authority: true,
        },
        ActionDescriptorV1 {
            label: "cash_split".to_string(),
            allowed_input_families: BTreeSet::from([ObjectFamily::Asset]),
            allowed_output_families: BTreeSet::from([ObjectFamily::Asset]),
            lifecycle_effect: LifecycleEffectV1::NoStateChange,
            witness_requirements: owner_only.clone(),
            receiver_must_accept: false,
            preserves_beneficiary: true,
            preserves_refund_authority: true,
        },
        ActionDescriptorV1 {
            label: "cash_transfer".to_string(),
            allowed_input_families: BTreeSet::from([ObjectFamily::Asset]),
            allowed_output_families: BTreeSet::from([ObjectFamily::Asset]),
            lifecycle_effect: LifecycleEffectV1::Transfer,
            witness_requirements: owner_only.clone(),
            receiver_must_accept: false,
            preserves_beneficiary: true,
            preserves_refund_authority: true,
        },
    ]);

    ActionPoolDescriptorV1 {
        label: "cash_policy_v1".to_string(),
        actions,
    }
}

pub fn reject_custom_native_cash_pool(
    action_pool: &ActionPoolDescriptorV1,
) -> Result<(), AssetError> {
    let expected = fixed_cash_action_pool_descriptor().action_pool_id()?;
    let actual = action_pool.action_pool_id()?;
    if actual != expected {
        return Err(AssetError::InvalidAsset(
            format!(
                "native cash must keep the fixed cash action pool; expected {expected}, got {actual}"
            )
            .into(),
        ));
    }
    Ok(())
}
