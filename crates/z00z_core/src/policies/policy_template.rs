use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};

use crate::{
    actions::{ActionPoolDescriptorV1, RequiredSignatureV1},
    config_name::{validate_domain_name, validate_underscore_name},
    rights::RightRequirementV1,
    AssetError, ObjectFamily,
};

use super::{
    AttestationRequirementV1, ConditionDescriptorV1, ConservationContributionV1, ExpiryRuleV1,
    PolicyDescriptorV1, ReplayProtectionV1, UnknownPolicyHandlingV1,
};

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PolicyTemplateV1 {
    pub label: String,
    pub primary_family: ObjectFamily,
    pub allowed_input_families: BTreeSet<ObjectFamily>,
    pub allowed_output_families: BTreeSet<ObjectFamily>,
    #[serde(default)]
    pub conditions: BTreeSet<ConditionDescriptorV1>,
    #[serde(default)]
    pub required_rights: BTreeSet<RightRequirementV1>,
    #[serde(default)]
    pub required_signatures: BTreeSet<RequiredSignatureV1>,
    #[serde(default)]
    pub required_attestations: BTreeSet<AttestationRequirementV1>,
    pub expiry_rule: ExpiryRuleV1,
    pub replay_protection: ReplayProtectionV1,
    pub conservation: ConservationContributionV1,
    #[serde(default)]
    pub unknown_policy_handling: UnknownPolicyHandlingV1,
}

impl PolicyTemplateV1 {
    pub fn materialize(
        &self,
        action_pool: &ActionPoolDescriptorV1,
        domain_name: &str,
    ) -> Result<PolicyDescriptorV1, AssetError> {
        let descriptor = PolicyDescriptorV1 {
            label: self.label.clone(),
            domain_name: domain_name.to_string(),
            primary_family: self.primary_family,
            allowed_input_families: self.allowed_input_families.clone(),
            allowed_output_families: self.allowed_output_families.clone(),
            action_pool_id: action_pool.action_pool_id()?,
            action_ids: action_pool.action_ids()?,
            conditions: self.conditions.clone(),
            required_rights: self.required_rights.clone(),
            required_signatures: self.required_signatures.clone(),
            required_attestations: self.required_attestations.clone(),
            expiry_rule: self.expiry_rule,
            replay_protection: self.replay_protection,
            conservation: self.conservation,
            unknown_policy_handling: self.unknown_policy_handling.clone(),
        };
        descriptor.validate()?;
        Ok(descriptor)
    }
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PolicyConfigEntryV1 {
    pub domain_name: String,
    pub action_pool: ActionPoolDescriptorV1,
    pub template: PolicyTemplateV1,
}

impl PolicyConfigEntryV1 {
    pub fn validate(&self) -> Result<(), AssetError> {
        validate_domain_name("policy.domain_name", self.domain_name.as_str())?;
        validate_underscore_name("policy.label", self.template.label.as_str())?;
        validate_underscore_name("policy.action_pool.label", self.action_pool.label.as_str())?;
        self.materialize().map(|_| ())
    }

    pub fn materialize(&self) -> Result<PolicyDescriptorV1, AssetError> {
        self.template
            .materialize(&self.action_pool, self.domain_name.as_str())
    }
}
