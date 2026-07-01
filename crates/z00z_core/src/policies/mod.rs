//! Canonical policy vocabulary for Phase 059 object semantics.

mod condition_descriptor;
mod policy_descriptor;
mod policy_id;
mod policy_template;

pub use condition_descriptor::{ConditionDescriptorV1, ConditionKindV1, ConditionTrustTierV1};
pub use policy_descriptor::{
    native_cash_policy_descriptor, validate_native_cash_policy_descriptor,
    AttestationRequirementV1, ConservationContributionV1, ExpiryRuleV1, PolicyDescriptorV1,
    ReplayProtectionV1, UnknownPolicyHandlingV1, UnknownPolicyValidatorV1, UnknownPolicyWalletV1,
};
pub use policy_id::PolicyId;
pub use policy_template::{PolicyConfigEntryV1, PolicyTemplateV1};

#[cfg(test)]
mod test_policy_descriptor;
