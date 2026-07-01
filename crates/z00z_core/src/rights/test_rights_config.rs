use std::collections::{BTreeMap, BTreeSet};

use crate::rights::{
    RightActionV1, RightClassConfig, RightPolicyV1, RightRequirementV1, RightScopeV1,
    RightsConfigEntry,
};

fn sample_right_config() -> RightsConfigEntry {
    RightsConfigEntry {
        id: "right_community_redeem".to_string(),
        right_class: RightClassConfig::ServiceEntitlement,
        issuer_scope: "issuer_test".to_string(),
        provider_scope: "provider_test".to_string(),
        holder_fixture: "wallet_bob".to_string(),
        control_fixture: "wallet_alice".to_string(),
        beneficiary_fixture: Some("wallet_charlie".to_string()),
        count: 1,
        domain_name: "rights.test.v1".to_string(),
        valid_from: 0,
        valid_until: 42,
        challenge_from: 0,
        challenge_until: 0,
        revocation_policy_id: "policy_revoke".to_string(),
        transition_policy_id: "policy_transition".to_string(),
        challenge_policy_id: "policy_challenge".to_string(),
        disclosure_policy_id: "policy_disclosure".to_string(),
        retention_policy_id: "policy_retention".to_string(),
        payload_commitment_seed: "payload_seed".to_string(),
        metadata: Some(BTreeMap::from([(
            "purpose".to_string(),
            "voucher redemption".to_string(),
        )])),
    }
}

#[test]
fn test_rights_reexport_shape() -> Result<(), Box<dyn std::error::Error>> {
    sample_right_config().validate()?;

    let requirement = RightRequirementV1 {
        right_policy: "right_community_redeem".to_string(),
        allowed_actions: BTreeSet::from([RightActionV1::Use]),
        scope: RightScopeV1::ObjectFamily(crate::ObjectFamily::Voucher),
        max_uses: Some(1),
        delegation_allowed: false,
        attenuation_only: true,
    };
    requirement.validate()?;

    Ok(())
}

#[test]
fn test_rights_reject_missing_purpose() {
    let mut right = sample_right_config();
    right.metadata = Some(BTreeMap::new());
    let err = right.validate().unwrap_err();
    assert!(
        err.to_string()
            .contains("rights.metadata.purpose must not be empty"),
        "unexpected error: {err}",
    );
}

#[test]
fn test_right_policy_zero_value() {
    let mut policy = RightPolicyV1 {
        right_class: RightClassConfig::ServiceEntitlement,
        allowed_actions: BTreeSet::from([RightActionV1::Use, RightActionV1::Delegate]),
        delegation_allowed: true,
        zero_value_only: false,
    };

    let err = policy.validate().unwrap_err();
    assert!(
        err.to_string()
            .contains("right policies must remain zero-value"),
        "unexpected error: {err}",
    );

    policy.zero_value_only = true;
    policy.validate().unwrap();
}

#[test]
fn test_rights_owner_path() {
    const RIGHTS_MOD: &str = include_str!("mod.rs");
    const ASSETS_MOD: &str = include_str!("../assets/mod.rs");

    assert!(
        RIGHTS_MOD.contains("mod config;"),
        "rights module must keep local config ownership",
    );
    assert!(
        RIGHTS_MOD.contains(
            "pub use config::{load_rights_from_yaml, RightClassConfig, RightsConfigEntry};"
        ),
        "rights module must export the canonical rights-config owner path",
    );
    assert!(
        !RIGHTS_MOD.contains("pub use crate::assets::right_config"),
        "rights module must not revert to assets-owned config exports",
    );
    assert!(
        !ASSETS_MOD.contains("pub mod right_config;"),
        "assets module must not keep a compatibility right_config submodule",
    );
    assert!(
        !ASSETS_MOD.contains("pub use crate::rights::{RightClassConfig, RightsConfigEntry};"),
        "assets module must not keep a cross-family rights re-export shim",
    );
}
