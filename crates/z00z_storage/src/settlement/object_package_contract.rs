use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize};
use z00z_core::{
    actions::{
        ActionDescriptorV1, ActionPoolDescriptorV1, RequiredSignatureV1, WitnessRequirementV1,
    },
    policies::{PolicyDescriptorV1, ReplayProtectionV1},
    ObjectFamily,
};

use super::{
    ObjectDeltaSetV1, SettlementActionV1, SettlementLeaf, SettlementStateRoot, SettlementStoreError,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RightWitnessStateV1 {
    Present,
    Missing,
    OutOfScope,
    Expired,
    Revoked,
    Consumed,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RightWitnessRefV1 {
    pub right_policy: String,
    pub witness_state: RightWitnessStateV1,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ObjectWitnessBundleV1 {
    #[serde(default)]
    pub signatures: BTreeSet<RequiredSignatureV1>,
    #[serde(default)]
    pub attestation_labels: BTreeSet<String>,
    pub has_acceptance_proof: bool,
    pub has_replay_nonce: bool,
    pub has_prior_root_binding: bool,
    pub has_disclosure_commitment: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RuntimeObjectPackageV1 {
    pub primary_family: ObjectFamily,
    pub selected_action: SettlementActionV1,
    pub selected_action_id: [u8; 32],
    pub policy_descriptor_hash: [u8; 32],
    pub action_pool_id: [u8; 32],
    #[serde(default)]
    pub required_rights: Vec<RightWitnessRefV1>,
    pub object_witnesses: ObjectWitnessBundleV1,
    pub delta_set: ObjectDeltaSetV1,
    pub fee_support_ref: Option<[u8; 32]>,
    pub prior_root: SettlementStateRoot,
    pub expected_new_root: SettlementStateRoot,
}

impl RuntimeObjectPackageV1 {
    #[must_use]
    pub const fn primary_family(&self) -> ObjectFamily {
        self.primary_family
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ObjectRejectCode {
    UnknownPolicy,
    UnknownAction,
    InvalidBacking,
    WrongFamilyProof,
    VoucherUsedAsCash,
    RightUsedAsValue,
    MissingRight,
    RightOutOfScope,
    RightExpired,
    RightRevoked,
    RightConsumed,
    Replay,
    DoubleRedeem,
    ResidualMismatch,
    ForcedAcceptance,
    StaleRoot,
    FeeBoundary,
    MissingSignature,
    MissingAttestation,
    ExpiredVoucherUse,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ObjectValidatorVerdict {
    pub family: ObjectFamily,
    pub selected_action: SettlementActionV1,
    pub policy_descriptor_hash: [u8; 32],
    pub action_pool_id: [u8; 32],
    pub selected_action_id: [u8; 32],
    pub prior_root: SettlementStateRoot,
    pub expected_new_root: SettlementStateRoot,
    pub reject: Option<ObjectRejectCode>,
}

impl ObjectValidatorVerdict {
    #[must_use]
    pub fn accepted(package: &RuntimeObjectPackageV1) -> Self {
        Self {
            family: package.primary_family,
            selected_action: package.selected_action,
            policy_descriptor_hash: package.policy_descriptor_hash,
            action_pool_id: package.action_pool_id,
            selected_action_id: package.selected_action_id,
            prior_root: package.prior_root,
            expected_new_root: package.expected_new_root,
            reject: None,
        }
    }

    #[must_use]
    pub fn rejected(package: &RuntimeObjectPackageV1, reject: ObjectRejectCode) -> Self {
        let mut item = Self::accepted(package);
        item.reject = Some(reject);
        item
    }
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct ObjectPolicyRegistryV1 {
    policies: BTreeMap<[u8; 32], RegisteredPolicyV1>,
}

impl ObjectPolicyRegistryV1 {
    pub fn register(
        &mut self,
        descriptor: PolicyDescriptorV1,
        action_pool: ActionPoolDescriptorV1,
    ) -> Result<(), String> {
        descriptor.validate().map_err(|err| err.to_string())?;
        action_pool.validate().map_err(|err| err.to_string())?;

        let policy_id = descriptor.policy_id().map_err(|err| err.to_string())?;
        let action_pool_id = action_pool
            .action_pool_id()
            .map_err(|err| err.to_string())?;
        if descriptor.action_pool_id != action_pool_id {
            return Err("policy descriptor action_pool_id does not match action pool".to_string());
        }

        let action_ids = action_pool.action_ids().map_err(|err| err.to_string())?;
        if descriptor.action_ids != action_ids {
            return Err("policy descriptor action ids do not match action pool".to_string());
        }

        self.policies.insert(
            policy_id.bytes(),
            RegisteredPolicyV1::new(descriptor, action_pool)?,
        );
        Ok(())
    }

    #[must_use]
    pub fn get(&self, policy_hash: [u8; 32]) -> Option<&RegisteredPolicyV1> {
        self.policies.get(&policy_hash)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RegisteredPolicyV1 {
    descriptor: PolicyDescriptorV1,
    action_pool: ActionPoolDescriptorV1,
    actions_by_id: BTreeMap<[u8; 32], ActionDescriptorV1>,
}

impl RegisteredPolicyV1 {
    fn new(
        descriptor: PolicyDescriptorV1,
        action_pool: ActionPoolDescriptorV1,
    ) -> Result<Self, String> {
        let mut actions_by_id = BTreeMap::new();
        for action in &action_pool.actions {
            let action_id = action.action_id().map_err(|err| err.to_string())?;
            actions_by_id.insert(action_id.bytes(), action.clone());
        }
        Ok(Self {
            descriptor,
            action_pool,
            actions_by_id,
        })
    }

    #[must_use]
    pub fn descriptor(&self) -> &PolicyDescriptorV1 {
        &self.descriptor
    }

    #[must_use]
    pub fn action(&self, action_id: [u8; 32]) -> Option<&ActionDescriptorV1> {
        self.actions_by_id.get(&action_id)
    }

    #[must_use]
    pub fn action_pool_id(&self) -> [u8; 32] {
        self.action_pool
            .action_pool_id()
            .expect("validated action pool")
            .bytes()
    }
}

pub fn inspect_object_package(
    package: &RuntimeObjectPackageV1,
    registry: &ObjectPolicyRegistryV1,
    published_prev_root: SettlementStateRoot,
    published_new_root: SettlementStateRoot,
) -> ObjectValidatorVerdict {
    let input_families = collect_input_families(package);
    let output_families = collect_output_families(package);

    if package.selected_action != package.delta_set.selected_action {
        return ObjectValidatorVerdict::rejected(package, ObjectRejectCode::UnknownAction);
    }
    if package.policy_descriptor_hash != package.delta_set.policy_descriptor_hash {
        return ObjectValidatorVerdict::rejected(package, ObjectRejectCode::UnknownPolicy);
    }
    if package.prior_root != package.delta_set.prior_root
        || package.expected_new_root != package.delta_set.expected_new_root
        || package.prior_root != published_prev_root
        || package.expected_new_root != published_new_root
    {
        return ObjectValidatorVerdict::rejected(package, ObjectRejectCode::StaleRoot);
    }
    if package.fee_support_ref
        != package
            .delta_set
            .fee_envelope
            .and_then(|item| item.support_ref)
    {
        return ObjectValidatorVerdict::rejected(package, ObjectRejectCode::FeeBoundary);
    }
    if has_value_bearing_right(&package.delta_set) {
        return ObjectValidatorVerdict::rejected(package, ObjectRejectCode::RightUsedAsValue);
    }

    let Some(policy) = registry.get(package.policy_descriptor_hash) else {
        return ObjectValidatorVerdict::rejected(package, ObjectRejectCode::UnknownPolicy);
    };
    if policy.action_pool_id() != package.action_pool_id
        || policy.descriptor().action_pool_id.bytes() != package.action_pool_id
    {
        return ObjectValidatorVerdict::rejected(package, ObjectRejectCode::UnknownAction);
    }
    let Some(action) = policy.action(package.selected_action_id) else {
        return ObjectValidatorVerdict::rejected(package, ObjectRejectCode::UnknownAction);
    };

    if package.primary_family == ObjectFamily::Voucher
        && policy.descriptor().primary_family == ObjectFamily::Asset
    {
        return ObjectValidatorVerdict::rejected(package, ObjectRejectCode::VoucherUsedAsCash);
    }
    if policy.descriptor().primary_family != package.primary_family {
        return ObjectValidatorVerdict::rejected(package, ObjectRejectCode::WrongFamilyProof);
    }
    if !family_set_allowed(&input_families, &action.allowed_input_families)
        || !family_set_allowed(&output_families, &action.allowed_output_families)
    {
        return ObjectValidatorVerdict::rejected(package, ObjectRejectCode::WrongFamilyProof);
    }
    if !family_set_allowed(&input_families, &policy.descriptor().allowed_input_families)
        || !family_set_allowed(
            &output_families,
            &policy.descriptor().allowed_output_families,
        )
    {
        return ObjectValidatorVerdict::rejected(package, ObjectRejectCode::WrongFamilyProof);
    }
    if let Some(reject) = check_required_right_input(action, &input_families) {
        return ObjectValidatorVerdict::rejected(package, reject);
    }
    if let Some(reject) = check_policy_signatures(policy.descriptor(), package) {
        return ObjectValidatorVerdict::rejected(package, reject);
    }
    if let Some(reject) = check_policy_attestations(policy.descriptor(), package) {
        return ObjectValidatorVerdict::rejected(package, reject);
    }
    if let Some(reject) = check_required_rights(policy.descriptor(), package) {
        return ObjectValidatorVerdict::rejected(package, reject);
    }
    if let Some(reject) = check_replay(policy.descriptor().replay_protection, package) {
        return ObjectValidatorVerdict::rejected(package, reject);
    }
    if let Some(reject) = check_action_requirements(action, package) {
        return ObjectValidatorVerdict::rejected(package, reject);
    }
    if let Err(err) = package.delta_set.validate_contract() {
        return ObjectValidatorVerdict::rejected(package, map_delta_error(&err));
    }

    ObjectValidatorVerdict::accepted(package)
}

fn family_set_allowed(used: &BTreeSet<ObjectFamily>, allowed: &BTreeSet<ObjectFamily>) -> bool {
    used.iter().all(|family| allowed.contains(family))
}

fn check_required_right_input(
    action: &ActionDescriptorV1,
    input_families: &BTreeSet<ObjectFamily>,
) -> Option<ObjectRejectCode> {
    let requires_right_input = action.allowed_input_families.contains(&ObjectFamily::Right)
        && action
            .witness_requirements
            .iter()
            .any(|requirement| matches!(requirement, WitnessRequirementV1::RightReference(_)));
    if requires_right_input && !input_families.contains(&ObjectFamily::Right) {
        return Some(ObjectRejectCode::MissingRight);
    }
    None
}

fn collect_input_families(package: &RuntimeObjectPackageV1) -> BTreeSet<ObjectFamily> {
    let deleted = package
        .delta_set
        .deleted_objects
        .iter()
        .filter_map(|delta| delta.prior_leaf.as_ref().map(leaf_family));
    let updated = package
        .delta_set
        .updated_objects
        .iter()
        .filter_map(|delta| delta.prior_leaf.as_ref().map(leaf_family));
    deleted.chain(updated).collect()
}

fn collect_output_families(package: &RuntimeObjectPackageV1) -> BTreeSet<ObjectFamily> {
    let created = package
        .delta_set
        .created_objects
        .iter()
        .filter_map(|delta| delta.next_leaf.as_ref().map(leaf_family));
    let updated = package
        .delta_set
        .updated_objects
        .iter()
        .filter_map(|delta| delta.next_leaf.as_ref().map(leaf_family));
    created.chain(updated).collect()
}

fn leaf_family(leaf: &SettlementLeaf) -> ObjectFamily {
    match leaf {
        SettlementLeaf::Terminal(_) => ObjectFamily::Asset,
        SettlementLeaf::Voucher(_) => ObjectFamily::Voucher,
        SettlementLeaf::Right(_) => ObjectFamily::Right,
    }
}

fn has_value_bearing_right(delta_set: &ObjectDeltaSetV1) -> bool {
    delta_set
        .deleted_objects
        .iter()
        .chain(delta_set.created_objects.iter())
        .chain(delta_set.updated_objects.iter())
        .any(|delta| {
            (matches!(delta.prior_leaf.as_ref(), Some(SettlementLeaf::Right(_)))
                || matches!(delta.next_leaf.as_ref(), Some(SettlementLeaf::Right(_))))
                && delta.declared_value_units.unwrap_or(0) != 0
        })
}

fn check_policy_signatures(
    descriptor: &PolicyDescriptorV1,
    package: &RuntimeObjectPackageV1,
) -> Option<ObjectRejectCode> {
    let need = descriptor
        .required_signatures
        .iter()
        .copied()
        .collect::<BTreeSet<RequiredSignatureV1>>();
    if !need.is_subset(&package.object_witnesses.signatures) {
        return Some(ObjectRejectCode::MissingSignature);
    }
    None
}

fn check_policy_attestations(
    descriptor: &PolicyDescriptorV1,
    package: &RuntimeObjectPackageV1,
) -> Option<ObjectRejectCode> {
    for requirement in &descriptor.required_attestations {
        if !package
            .object_witnesses
            .attestation_labels
            .contains(&requirement.label)
        {
            return Some(ObjectRejectCode::MissingAttestation);
        }
    }
    None
}

fn check_required_rights(
    descriptor: &PolicyDescriptorV1,
    package: &RuntimeObjectPackageV1,
) -> Option<ObjectRejectCode> {
    for right in &descriptor.required_rights {
        let Some(witness) = find_right(&package.required_rights, &right.right_policy) else {
            return Some(ObjectRejectCode::MissingRight);
        };
        match witness.witness_state {
            RightWitnessStateV1::Present => {}
            RightWitnessStateV1::Missing => return Some(ObjectRejectCode::MissingRight),
            RightWitnessStateV1::OutOfScope => return Some(ObjectRejectCode::RightOutOfScope),
            RightWitnessStateV1::Expired => return Some(ObjectRejectCode::RightExpired),
            RightWitnessStateV1::Revoked => return Some(ObjectRejectCode::RightRevoked),
            RightWitnessStateV1::Consumed => return Some(ObjectRejectCode::RightConsumed),
        }
    }
    None
}

fn check_replay(
    rule: ReplayProtectionV1,
    package: &RuntimeObjectPackageV1,
) -> Option<ObjectRejectCode> {
    match rule {
        ReplayProtectionV1::None => None,
        ReplayProtectionV1::Nonce if package.object_witnesses.has_replay_nonce => None,
        ReplayProtectionV1::Nonce => Some(ObjectRejectCode::Replay),
        ReplayProtectionV1::NonceAndRoot if !package.object_witnesses.has_replay_nonce => {
            Some(ObjectRejectCode::Replay)
        }
        ReplayProtectionV1::NonceAndRoot if !package.object_witnesses.has_prior_root_binding => {
            Some(ObjectRejectCode::StaleRoot)
        }
        ReplayProtectionV1::NonceAndRoot => None,
    }
}

fn check_action_requirements(
    action: &ActionDescriptorV1,
    package: &RuntimeObjectPackageV1,
) -> Option<ObjectRejectCode> {
    for requirement in &action.witness_requirements {
        match requirement {
            WitnessRequirementV1::Signature(signature)
                if !package.object_witnesses.signatures.contains(signature) =>
            {
                return Some(ObjectRejectCode::MissingSignature);
            }
            WitnessRequirementV1::VerifierAttestation(label)
                if !package.object_witnesses.attestation_labels.contains(label) =>
            {
                return Some(ObjectRejectCode::MissingAttestation);
            }
            WitnessRequirementV1::RightReference(label) => {
                let Some(witness) = find_right(&package.required_rights, label) else {
                    return Some(ObjectRejectCode::MissingRight);
                };
                match witness.witness_state {
                    RightWitnessStateV1::Present => {}
                    RightWitnessStateV1::Missing => return Some(ObjectRejectCode::MissingRight),
                    RightWitnessStateV1::OutOfScope => {
                        return Some(ObjectRejectCode::RightOutOfScope);
                    }
                    RightWitnessStateV1::Expired => return Some(ObjectRejectCode::RightExpired),
                    RightWitnessStateV1::Revoked => return Some(ObjectRejectCode::RightRevoked),
                    RightWitnessStateV1::Consumed => return Some(ObjectRejectCode::RightConsumed),
                }
            }
            WitnessRequirementV1::AcceptanceProof
                if !package.object_witnesses.has_acceptance_proof =>
            {
                return Some(ObjectRejectCode::ForcedAcceptance);
            }
            WitnessRequirementV1::ReplayNonce if !package.object_witnesses.has_replay_nonce => {
                return Some(ObjectRejectCode::Replay);
            }
            WitnessRequirementV1::PriorStateRoot
                if !package.object_witnesses.has_prior_root_binding =>
            {
                return Some(ObjectRejectCode::StaleRoot);
            }
            WitnessRequirementV1::DisclosureCommitment
                if !package.object_witnesses.has_disclosure_commitment =>
            {
                return Some(ObjectRejectCode::MissingAttestation);
            }
            WitnessRequirementV1::Signature(_)
            | WitnessRequirementV1::VerifierAttestation(_)
            | WitnessRequirementV1::AcceptanceProof
            | WitnessRequirementV1::ReplayNonce
            | WitnessRequirementV1::PriorStateRoot
            | WitnessRequirementV1::DisclosureCommitment => {}
        }
    }
    None
}

fn find_right<'a>(
    rights: &'a [RightWitnessRefV1],
    right_policy: &str,
) -> Option<&'a RightWitnessRefV1> {
    rights.iter().find(|item| item.right_policy == right_policy)
}

fn map_delta_error(err: &SettlementStoreError) -> ObjectRejectCode {
    let detail = err.to_string();
    if detail.contains("fee support") || detail.contains("fee envelope") {
        return ObjectRejectCode::FeeBoundary;
    }
    if detail.contains("right deltas must") {
        return ObjectRejectCode::RightUsedAsValue;
    }
    if detail.contains("expired voucher use") {
        return ObjectRejectCode::ExpiredVoucherUse;
    }
    if detail.contains("forced acceptance") || detail.contains("holder acceptance") {
        return ObjectRejectCode::ForcedAcceptance;
    }
    if detail.contains("double redeem") {
        return ObjectRejectCode::DoubleRedeem;
    }
    if detail.contains("declared value does not match voucher accounting")
        || detail.contains("conservation mismatch")
        || detail.contains("residual value is malformed")
    {
        return ObjectRejectCode::ResidualMismatch;
    }
    if detail.contains("voucher issue backing")
        || detail.contains("reserve backing")
        || detail.contains("positive-value voucher")
    {
        return ObjectRejectCode::InvalidBacking;
    }
    if detail.contains("policy hash") {
        return ObjectRejectCode::UnknownPolicy;
    }
    ObjectRejectCode::WrongFamilyProof
}
