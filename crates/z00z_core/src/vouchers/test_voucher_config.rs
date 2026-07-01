// Owner-module tests stay under `crate::vouchers`, not `crate::assets`.

use crate::{
    actions::ActionPoolId,
    policies::PolicyId,
    vouchers::{
        VoucherAcceptanceTermsV1, VoucherBackingReferenceV1, VoucherConfigEntry,
        VoucherLifecycleV1, VoucherValidityWindowV1,
    },
};

fn sample_voucher_config() -> VoucherConfigEntry {
    VoucherConfigEntry {
        id: "voucher_aid_community".to_string(),
        domain_name: "z00z.core.vouchers.aid.community.v1".to_string(),
        issuer_fixture: "wallet_alice".to_string(),
        holder_fixture: "wallet_bob".to_string(),
        beneficiary_fixture: "wallet_charlie".to_string(),
        backing: VoucherBackingReferenceV1::ConsumedAsset {
            definition_id: [0x11u8; 32],
            serial_id: 7,
        },
        face_value: 600,
        remaining_value: 350,
        policy_id: PolicyId::new([0x22u8; 32]),
        action_pool_id: ActionPoolId::new([0x33u8; 32]),
        lifecycle: VoucherLifecycleV1::Active,
        validity: VoucherValidityWindowV1 {
            valid_from: 10,
            valid_until: 42,
        },
        acceptance: VoucherAcceptanceTermsV1 {
            receiver_must_accept: true,
            allow_reject: true,
            refund_target_fixture: "wallet_alice".to_string(),
        },
        replay_nonce: [0x44u8; 32],
        disclosure_commitment: Some([0x55u8; 32]),
        audit_commitment: Some([0x66u8; 32]),
    }
}

#[test]
fn test_voucher_config_validates_shape() -> Result<(), Box<dyn std::error::Error>> {
    let config = sample_voucher_config();
    config.validate()?;
    assert_ne!(config.holder_fixture, config.beneficiary_fixture);
    Ok(())
}

#[test]
fn test_voucher_rejects_zero_reserve() {
    let mut config = sample_voucher_config();
    config.backing = VoucherBackingReferenceV1::ReserveCommitment([0u8; 32]);
    let err = config.validate().unwrap_err();
    assert!(
        err.to_string()
            .contains("voucher reserve commitment must not be zero"),
        "unexpected error: {err}",
    );
}

#[test]
fn test_voucher_rejects_residual_overflow() {
    let mut config = sample_voucher_config();
    config.remaining_value = config.face_value + 1;
    let err = config.validate().unwrap_err();
    assert!(
        err.to_string()
            .contains("voucher remaining_value must not exceed face_value"),
        "unexpected error: {err}",
    );
}

#[test]
fn test_voucher_requires_acceptance() {
    let mut config = sample_voucher_config();
    config.acceptance.receiver_must_accept = false;
    let err = config.validate().unwrap_err();
    assert!(
        err.to_string()
            .contains("voucher acceptance must stay explicit for the receiver"),
        "unexpected error: {err}",
    );
}

#[test]
fn test_voucher_rejects_zero_nonce() {
    let mut config = sample_voucher_config();
    config.replay_nonce = [0u8; 32];
    let err = config.validate().unwrap_err();
    assert!(
        err.to_string()
            .contains("voucher replay nonce must not be zero"),
        "unexpected error: {err}",
    );
}
