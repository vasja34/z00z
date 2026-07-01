use std::collections::{BTreeMap, BTreeSet};

use crate::{
    actions::{
        reject_custom_native_cash_pool, ActionDescriptorV1, ActionPoolDescriptorV1,
        LifecycleEffectV1, RequiredSignatureV1, WitnessRequirementV1,
    },
    policies::{
        native_cash_policy_descriptor, validate_native_cash_policy_descriptor,
        AttestationRequirementV1, ConditionDescriptorV1, ConditionKindV1, ConditionTrustTierV1,
        ConservationContributionV1, ExpiryRuleV1, PolicyDescriptorV1, ReplayProtectionV1,
        UnknownPolicyHandlingV1, UnknownPolicyValidatorV1, UnknownPolicyWalletV1,
    },
    rights::{RightActionV1, RightRequirementV1, RightScopeV1},
    AssetError, ObjectFamily, ObjectRoleV1,
};
use z00z_utils::codec::{Codec, JsonCodec};

fn sample_voucher_action(label: &str) -> ActionDescriptorV1 {
    ActionDescriptorV1 {
        label: label.to_string(),
        allowed_input_families: BTreeSet::from([ObjectFamily::Voucher]),
        allowed_output_families: BTreeSet::from([ObjectFamily::Asset, ObjectFamily::Voucher]),
        lifecycle_effect: LifecycleEffectV1::PartialRedeem,
        witness_requirements: BTreeSet::from([
            WitnessRequirementV1::AcceptanceProof,
            WitnessRequirementV1::RightReference("right_redeem".to_string()),
            WitnessRequirementV1::Signature(RequiredSignatureV1::Holder),
        ]),
        receiver_must_accept: true,
        preserves_beneficiary: true,
        preserves_refund_authority: true,
    }
}

fn sample_policy_descriptor() -> Result<PolicyDescriptorV1, AssetError> {
    let action_a = sample_voucher_action("voucher_accept");
    let action_b = sample_voucher_action("voucher_redeem");
    let action_pool = ActionPoolDescriptorV1 {
        label: "voucher_policy_v1".to_string(),
        actions: BTreeSet::from([action_b.clone(), action_a.clone()]),
    };

    Ok(PolicyDescriptorV1 {
        label: "voucher_policy_v1".to_string(),
        domain_name: "z00z.core.policies.voucher_policy.test.v1".to_string(),
        primary_family: ObjectFamily::Voucher,
        allowed_input_families: BTreeSet::from([ObjectFamily::Asset, ObjectFamily::Voucher]),
        allowed_output_families: BTreeSet::from([ObjectFamily::Asset, ObjectFamily::Voucher]),
        action_pool_id: action_pool.action_pool_id()?,
        action_ids: action_pool.action_ids()?,
        conditions: BTreeSet::from([ConditionDescriptorV1 {
            label: "valid_until".to_string(),
            kind: ConditionKindV1::TimeWindow,
            trust_tier: ConditionTrustTierV1::Deterministic,
            verifier: None,
            metadata: BTreeMap::from([("field".to_string(), "valid_until".to_string())]),
        }]),
        required_rights: BTreeSet::from([RightRequirementV1 {
            right_policy: "right_redeem".to_string(),
            allowed_actions: BTreeSet::from([RightActionV1::Use]),
            scope: RightScopeV1::ObjectFamily(ObjectFamily::Voucher),
            max_uses: Some(1),
            delegation_allowed: false,
            attenuation_only: true,
        }]),
        required_signatures: BTreeSet::from([RequiredSignatureV1::Holder]),
        required_attestations: BTreeSet::from([AttestationRequirementV1 {
            label: "issuer_attestation".to_string(),
            verifier: "validator_attestor".to_string(),
        }]),
        expiry_rule: ExpiryRuleV1::ValidUntil,
        replay_protection: ReplayProtectionV1::Nonce,
        conservation: ConservationContributionV1::ConditionalValue,
        unknown_policy_handling: UnknownPolicyHandlingV1::default(),
    })
}

#[test]
fn test_object_family_variants() {
    let names = [
        String::from_utf8(JsonCodec.serialize(&ObjectFamily::Asset).unwrap()).unwrap(),
        String::from_utf8(JsonCodec.serialize(&ObjectFamily::Voucher).unwrap()).unwrap(),
        String::from_utf8(JsonCodec.serialize(&ObjectFamily::Right).unwrap()).unwrap(),
    ];
    assert_eq!(names, ["\"asset\"", "\"voucher\"", "\"right\""]);

    let fee_role =
        String::from_utf8(JsonCodec.serialize(&ObjectRoleV1::FeeEnvelope).unwrap()).unwrap();
    assert_eq!(fee_role, "\"fee_envelope\"");
}

#[test]
fn test_action_pool_canonical_order() -> Result<(), Box<dyn std::error::Error>> {
    let action_a = sample_voucher_action("voucher_accept");
    let action_b = sample_voucher_action("voucher_redeem");

    let pool_left = ActionPoolDescriptorV1 {
        label: "voucher_policy_v1".to_string(),
        actions: BTreeSet::from([action_a.clone(), action_b.clone()]),
    };
    let pool_right = ActionPoolDescriptorV1 {
        label: "voucher_policy_v1".to_string(),
        actions: BTreeSet::from([action_b, action_a]),
    };

    assert_eq!(pool_left.canonical_bytes()?, pool_right.canonical_bytes()?);
    assert_eq!(pool_left.action_pool_id()?, pool_right.action_pool_id()?);
    assert_eq!(pool_left.action_ids()?, pool_right.action_ids()?);
    Ok(())
}

#[test]
fn test_policy_descriptor_canonical_bytes() -> Result<(), Box<dyn std::error::Error>> {
    let descriptor = sample_policy_descriptor()?;
    let bytes1 = descriptor.canonical_bytes()?;
    let bytes2 = descriptor.canonical_bytes()?;

    assert_eq!(bytes1, bytes2);
    assert_eq!(descriptor.policy_id()?, descriptor.policy_id()?);
    Ok(())
}

#[test]
fn test_policy_hash_changes() -> Result<(), Box<dyn std::error::Error>> {
    let descriptor = sample_policy_descriptor()?;
    let mut changed = sample_policy_descriptor()?;
    changed
        .required_attestations
        .insert(AttestationRequirementV1 {
            label: "second_attestation".to_string(),
            verifier: "validator_attestor_2".to_string(),
        });

    assert_ne!(descriptor.policy_id()?, changed.policy_id()?);
    Ok(())
}

#[test]
fn test_policy_owner_path_contract() -> Result<(), Box<dyn std::error::Error>> {
    const ASSETS_MOD: &str = include_str!("mod.rs");

    assert!(
        !ASSETS_MOD.contains("pub use crate::policies::{"),
        "assets facade must not re-export policy owner contracts",
    );

    let descriptor = native_cash_policy_descriptor()?;
    validate_native_cash_policy_descriptor(&descriptor)?;
    Ok(())
}

#[test]
fn test_native_cash_pool_reject() -> Result<(), Box<dyn std::error::Error>> {
    let arbitrary_pool = ActionPoolDescriptorV1 {
        label: "cash_policy_mutated".to_string(),
        actions: BTreeSet::from([ActionDescriptorV1 {
            label: "cash_transfer_with_accept".to_string(),
            allowed_input_families: BTreeSet::from([ObjectFamily::Asset]),
            allowed_output_families: BTreeSet::from([ObjectFamily::Asset, ObjectFamily::Voucher]),
            lifecycle_effect: LifecycleEffectV1::Transfer,
            witness_requirements: BTreeSet::from([WitnessRequirementV1::Signature(
                RequiredSignatureV1::Owner,
            )]),
            receiver_must_accept: false,
            preserves_beneficiary: true,
            preserves_refund_authority: true,
        }]),
    };

    let err = reject_custom_native_cash_pool(&arbitrary_pool).unwrap_err();
    assert!(
        err.to_string()
            .contains("native cash must keep the fixed cash action pool"),
        "unexpected error: {err}",
    );

    let mut native_policy = native_cash_policy_descriptor()?;
    native_policy.action_pool_id = arbitrary_pool.action_pool_id()?;
    native_policy.action_ids = arbitrary_pool.action_ids()?;

    let err = validate_native_cash_policy_descriptor(&native_policy).unwrap_err();
    assert!(
        err.to_string()
            .contains("native cash policy must stay fixed"),
        "unexpected error: {err}",
    );

    Ok(())
}

#[test]
fn test_unknown_policy_handling_default() {
    let handling = UnknownPolicyHandlingV1::default();
    assert_eq!(handling.validator, UnknownPolicyValidatorV1::FailClosed);
    assert_eq!(handling.wallet, UnknownPolicyWalletV1::Quarantine);
}
