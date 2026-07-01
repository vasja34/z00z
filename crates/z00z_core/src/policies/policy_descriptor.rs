use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};
use z00z_crypto::DomainHasher;
use z00z_utils::codec::to_canonical_json_bytes;

use crate::{
    actions::{fixed_cash_action_pool_descriptor, ActionId, ActionPoolId, RequiredSignatureV1},
    config_name::{validate_domain_name, validate_underscore_name},
    domains::PolicyDescriptorHashDomain,
    rights::RightRequirementV1,
    AssetError, ObjectFamily,
};

use super::{ConditionDescriptorV1, PolicyId};

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ConservationContributionV1 {
    FinalValue,
    ConditionalValue,
    ZeroValueAuthority,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ExpiryRuleV1 {
    None,
    ValidUntil,
    FixedEpoch,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReplayProtectionV1 {
    None,
    Nonce,
    NonceAndRoot,
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AttestationRequirementV1 {
    pub label: String,
    pub verifier: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UnknownPolicyValidatorV1 {
    FailClosed,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UnknownPolicyWalletV1 {
    Quarantine,
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct UnknownPolicyHandlingV1 {
    pub validator: UnknownPolicyValidatorV1,
    pub wallet: UnknownPolicyWalletV1,
}

impl Default for UnknownPolicyHandlingV1 {
    fn default() -> Self {
        Self {
            validator: UnknownPolicyValidatorV1::FailClosed,
            wallet: UnknownPolicyWalletV1::Quarantine,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PolicyDescriptorV1 {
    pub label: String,
    pub domain_name: String,
    pub primary_family: ObjectFamily,
    pub allowed_input_families: BTreeSet<ObjectFamily>,
    pub allowed_output_families: BTreeSet<ObjectFamily>,
    pub action_pool_id: ActionPoolId,
    pub action_ids: BTreeSet<ActionId>,
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

impl PolicyDescriptorV1 {
    pub fn validate(&self) -> Result<(), AssetError> {
        if self.label.trim().is_empty() {
            return Err(AssetError::InvalidAsset(
                "policy descriptor label must not be empty".into(),
            ));
        }
        validate_underscore_name("policy.label", self.label.as_str())?;
        validate_domain_name("policy.domain_name", self.domain_name.as_str())?;

        if self.allowed_input_families.is_empty() || self.allowed_output_families.is_empty() {
            return Err(AssetError::InvalidAsset(
                "policy descriptor must declare input and output families".into(),
            ));
        }

        if self.action_ids.is_empty() {
            return Err(AssetError::InvalidAsset(
                "policy descriptor must declare at least one action".into(),
            ));
        }

        for condition in &self.conditions {
            condition.validate()?;
        }

        for right in &self.required_rights {
            right.validate()?;
        }

        for attestation in &self.required_attestations {
            if attestation.label.trim().is_empty() || attestation.verifier.trim().is_empty() {
                return Err(AssetError::InvalidAsset(
                    "attestation requirements must keep non-empty label and verifier".into(),
                ));
            }
            validate_underscore_name("policy.attestation.label", attestation.label.as_str())?;
            validate_underscore_name("policy.attestation.verifier", attestation.verifier.as_str())?;
        }

        match self.primary_family {
            ObjectFamily::Asset => {
                if self.conservation != ConservationContributionV1::FinalValue {
                    return Err(AssetError::InvalidAsset(
                        "asset policies must conserve final value".into(),
                    ));
                }
            }
            ObjectFamily::Voucher => {
                if self.conservation != ConservationContributionV1::ConditionalValue {
                    return Err(AssetError::InvalidAsset(
                        "voucher policies must conserve conditional value".into(),
                    ));
                }
            }
            ObjectFamily::Right => {
                if self.conservation != ConservationContributionV1::ZeroValueAuthority {
                    return Err(AssetError::InvalidAsset(
                        "right policies must remain zero-value authority".into(),
                    ));
                }
            }
        }

        Ok(())
    }

    pub fn canonical_bytes(&self) -> Result<Vec<u8>, AssetError> {
        self.validate()?;
        to_canonical_json_bytes(self)
            .map_err(|err| AssetError::Serialization(err.to_string().into()))
    }

    pub fn policy_id(&self) -> Result<PolicyId, AssetError> {
        let bytes = self.canonical_bytes()?;
        let mut hasher = DomainHasher::<PolicyDescriptorHashDomain>::new_with_label("policy");
        hasher.update(bytes);
        let digest = hasher.finalize();
        let mut id = [0u8; 32];
        id.copy_from_slice(&digest.as_ref()[..32]);
        Ok(PolicyId::new(id))
    }
}

pub fn native_cash_policy_descriptor() -> Result<PolicyDescriptorV1, AssetError> {
    let action_pool = fixed_cash_action_pool_descriptor();
    Ok(PolicyDescriptorV1 {
        label: "cash_policy_v1".to_string(),
        domain_name: "z00z.core.policies.cash.v1".to_string(),
        primary_family: ObjectFamily::Asset,
        allowed_input_families: BTreeSet::from([ObjectFamily::Asset]),
        allowed_output_families: BTreeSet::from([ObjectFamily::Asset]),
        action_pool_id: action_pool.action_pool_id()?,
        action_ids: action_pool.action_ids()?,
        conditions: BTreeSet::new(),
        required_rights: BTreeSet::new(),
        required_signatures: BTreeSet::from([RequiredSignatureV1::Owner]),
        required_attestations: BTreeSet::new(),
        expiry_rule: ExpiryRuleV1::None,
        replay_protection: ReplayProtectionV1::NonceAndRoot,
        conservation: ConservationContributionV1::FinalValue,
        unknown_policy_handling: UnknownPolicyHandlingV1::default(),
    })
}

pub fn validate_native_cash_policy_descriptor(
    descriptor: &PolicyDescriptorV1,
) -> Result<(), AssetError> {
    let expected = native_cash_policy_descriptor()?;
    let expected_id = expected.policy_id()?;
    let actual_id = descriptor.policy_id()?;
    if expected_id != actual_id {
        return Err(AssetError::InvalidAsset(
            format!("native cash policy must stay fixed; expected {expected_id}, got {actual_id}")
                .into(),
        ));
    }
    Ok(())
}
