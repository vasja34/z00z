use super::{AssetConfigEntry, BTreeMap, BTreeSet, GenesisError};

use crate::{
    actions::{fixed_cash_action_pool_descriptor, ActionPoolDescriptorV1, ActionPoolId},
    assets::{policy_flags::GAS, AssetError, ObjectFamily},
    policies::{native_cash_policy_descriptor, PolicyConfigEntryV1, PolicyDescriptorV1, PolicyId},
};

pub const GENESIS_POLICIES_FILE: &str = "genesis_policies.json";

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct GenesisPolicyRecord {
    pub policy_index: u32,
    pub policy_id: PolicyId,
    pub action_pool_id: ActionPoolId,
    pub action_pool: ActionPoolDescriptorV1,
    pub descriptor: PolicyDescriptorV1,
}

impl GenesisPolicyRecord {
    pub fn validate(&self) -> Result<(), AssetError> {
        self.action_pool.validate()?;
        self.descriptor.validate()?;
        if self.action_pool_id != self.action_pool.action_pool_id()? {
            return Err(AssetError::InvalidAsset(
                "policy record action_pool_id drifted from descriptor".into(),
            ));
        }
        if self.policy_id != self.descriptor.policy_id()? {
            return Err(AssetError::InvalidAsset(
                "policy record policy_id drifted from descriptor".into(),
            ));
        }
        if self.descriptor.action_pool_id != self.action_pool_id {
            return Err(AssetError::InvalidAsset(
                "policy descriptor action_pool_id must match exported action pool".into(),
            ));
        }
        if self.descriptor.action_ids != self.action_pool.action_ids()? {
            return Err(AssetError::InvalidAsset(
                "policy descriptor action ids must match exported action pool".into(),
            ));
        }
        Ok(())
    }
}

pub(crate) fn policy_lookup(
    records: &[GenesisPolicyRecord],
) -> Result<BTreeMap<String, GenesisPolicyRecord>, GenesisError> {
    let mut lookup = BTreeMap::new();
    for record in records {
        record.validate()?;
        if lookup
            .insert(record.descriptor.label.clone(), record.clone())
            .is_some()
        {
            return Err(GenesisError::InvalidConfig(format!(
                "duplicate policy label in genesis packet: {}",
                record.descriptor.label
            )));
        }
    }
    Ok(lookup)
}

fn add_record(
    records: &mut Vec<GenesisPolicyRecord>,
    seen_policy_ids: &mut BTreeSet<PolicyId>,
    seen_action_pool_ids: &mut BTreeSet<ActionPoolId>,
    action_pool: ActionPoolDescriptorV1,
    descriptor: PolicyDescriptorV1,
) -> Result<(), GenesisError> {
    let action_pool_id = action_pool.action_pool_id()?;
    let policy_id = descriptor.policy_id()?;
    if !seen_policy_ids.insert(policy_id) {
        return Err(GenesisError::InvalidConfig(format!(
            "duplicate genesis policy id for {}",
            descriptor.label
        )));
    }
    if !seen_action_pool_ids.insert(action_pool_id) {
        return Err(GenesisError::InvalidConfig(format!(
            "duplicate genesis action pool id for {}",
            action_pool.label
        )));
    }
    records.push(GenesisPolicyRecord {
        policy_index: records.len() as u32,
        policy_id,
        action_pool_id,
        action_pool,
        descriptor,
    });
    Ok(())
}

fn has_native_cash_asset(assets: &[AssetConfigEntry]) -> bool {
    assets
        .iter()
        .any(|asset| asset.policy.asset_flags(asset.class) & GAS != 0)
}

pub fn generate_genesis_policies(
    assets: &[AssetConfigEntry],
    entries: &[PolicyConfigEntryV1],
) -> Result<Vec<GenesisPolicyRecord>, GenesisError> {
    let mut records = Vec::new();
    let mut seen_policy_ids = BTreeSet::new();
    let mut seen_action_pool_ids = BTreeSet::new();

    if has_native_cash_asset(assets) {
        let action_pool = fixed_cash_action_pool_descriptor();
        let descriptor = native_cash_policy_descriptor()?;
        add_record(
            &mut records,
            &mut seen_policy_ids,
            &mut seen_action_pool_ids,
            action_pool,
            descriptor,
        )?;
    }

    for entry in entries {
        if entry.template.primary_family == ObjectFamily::Asset {
            return Err(GenesisError::InvalidConfig(
                "asset-side custom genesis policy entries are forbidden; native cash policy is built in"
                    .to_string(),
            ));
        }
        let descriptor = entry.materialize()?;
        match descriptor.primary_family {
            ObjectFamily::Asset => {
                unreachable!("asset families must be rejected before materialize")
            }
            ObjectFamily::Voucher | ObjectFamily::Right => {}
        }
        add_record(
            &mut records,
            &mut seen_policy_ids,
            &mut seen_action_pool_ids,
            entry.action_pool.clone(),
            descriptor,
        )?;
    }

    Ok(records)
}
