use super::{
    keys::{definition_key, serial_key, terminal_key},
    model::{terminal_leaf_hash, SettlementModel},
    DefinitionId, DefinitionRootLeaf, ModelErr, ProofItem, RightErr, RootErr, SerialId,
    SerialRootLeaf, SettlementLeaf, SettlementPath, SettlementStateRoot, StoreItem, TerminalId,
    TxDigest,
};
use proptest::prelude::*;
use serde::Serialize;
use z00z_core::assets::AssetLeaf;
use z00z_core::vouchers::{VoucherLifecycleV1, VoucherValidityWindowV1};
use z00z_utils::codec::{BincodeCodec, Codec};

use super::{
    ObjectDeltaSetV1, SettlementActionV1, SettlementObjectDeltaV1, TerminalLeaf, VoucherAction,
    VoucherActionCtx, VoucherBackingRef, VoucherLeaf,
};

fn fee_envelope(mark: u8) -> super::FeeEnvelope {
    let budget_units = u64::from(mark) + 3;
    let support_ref = Some([mark.wrapping_add(11); 32]);
    super::FeeEnvelope {
        version: 1,
        payer_commitment: [mark.wrapping_add(12); 32],
        sponsor_commitment: [mark.wrapping_add(13); 32],
        budget_units,
        budget_commitment: super::FeeEnvelope::budget_bind(budget_units, support_ref),
        domain_id: [mark.wrapping_add(14); 32],
        expires_at: 100,
        nonce: [mark.wrapping_add(15); 32],
        transition_id: [mark.wrapping_add(16); 32],
        replay_key: [mark.wrapping_add(17); 32],
        support_ref,
        failure_policy_id: [mark.wrapping_add(18); 32],
    }
}

fn leaf(path: &SettlementPath, mark: u8) -> TerminalLeaf {
    let mut leaf = AssetLeaf {
        asset_id: path.terminal_id().into_bytes(),
        serial_id: path.serial_id.get(),
        ..AssetLeaf::default()
    };
    leaf.owner_tag[0] = mark;
    leaf.c_amount[0] = mark;
    TerminalLeaf::from(leaf)
}

fn path(def_mark: u8, serial_id: u32, asset_mark: u8) -> SettlementPath {
    SettlementPath::new(
        DefinitionId::new([def_mark; 32]),
        SerialId::new(serial_id),
        TerminalId::new([asset_mark; 32]),
    )
}

fn build_root_from_path(path: SettlementPath, mark: u8) -> SettlementStateRoot {
    let mut model = SettlementModel::new();
    model
        .put_leaf(StoreItem::new(path, leaf(&path, mark)).expect("store item"))
        .expect("root")
}

fn root_many(items: &[StoreItem]) -> SettlementStateRoot {
    let mut model = SettlementModel::new();
    let mut root = SettlementStateRoot::settlement_v1([0u8; 32]);

    for item in items.iter().cloned() {
        root = model.put_leaf(item).expect("put");
    }

    root
}

fn item(def_mark: u8, serial_id: u32, asset_mark: u8, mark: u8) -> StoreItem {
    let path = path(def_mark, serial_id, asset_mark);
    StoreItem::new(path, leaf(&path, mark)).expect("item")
}

fn voucher_path(mark: u8) -> SettlementPath {
    SettlementPath::new(
        DefinitionId::new([mark.wrapping_add(40); 32]),
        SerialId::new(u32::from(mark) + 1),
        TerminalId::new([mark; 32]),
    )
}

fn voucher_leaf(mark: u8) -> VoucherLeaf {
    VoucherLeaf {
        version: 1,
        terminal_id: TerminalId::new([mark; 32]),
        issuer_commitment: [mark.wrapping_add(1); 32],
        holder_commitment: [mark.wrapping_add(2); 32],
        beneficiary_commitment: [mark.wrapping_add(3); 32],
        refund_target_commitment: [mark.wrapping_add(4); 32],
        backing: VoucherBackingRef::ReserveCommitment([mark.wrapping_add(5); 32]),
        face_value: 95,
        remaining_value: 95,
        policy_id: [mark.wrapping_add(6); 32],
        action_pool_id: [mark.wrapping_add(7); 32],
        lifecycle: VoucherLifecycleV1::Active,
        validity: VoucherValidityWindowV1 {
            valid_from: 10,
            valid_until: 50,
        },
        receiver_must_accept: true,
        allow_reject: true,
        replay_nonce: [mark.wrapping_add(8); 32],
        disclosure_commitment: Some([mark.wrapping_add(9); 32]),
        audit_commitment: Some([mark.wrapping_add(10); 32]),
    }
}

#[test]
fn test_delta_rejects_value_right() {
    let path = path(11, 1, 9);
    let right_delta = SettlementObjectDeltaV1::created(
        path,
        SettlementLeaf::Right(super::RightLeaf {
            version: 1,
            terminal_id: path.terminal_id,
            right_class: super::RightClass::MachineCapability,
            issuer_scope: [1u8; 32],
            provider_scope: [2u8; 32],
            holder_commitment: [3u8; 32],
            control_commitment: [4u8; 32],
            beneficiary_commitment: [5u8; 32],
            payload_commitment: [6u8; 32],
            valid_from: 10,
            valid_until: 20,
            challenge_from: 0,
            challenge_until: 0,
            use_nonce: [7u8; 32],
            revocation_policy_id: [8u8; 32],
            transition_policy_id: [9u8; 32],
            challenge_policy_id: [10u8; 32],
            disclosure_policy_id: [11u8; 32],
            retention_policy_id: [12u8; 32],
        }),
        Some(1),
    );
    let delta = ObjectDeltaSetV1::new(
        SettlementActionV1::Right(super::RightAction::Create),
        [9u8; 32],
        None,
        Vec::new(),
        vec![right_delta],
        Vec::new(),
        None,
        SettlementStateRoot::settlement_v1([1u8; 32]),
        SettlementStateRoot::settlement_v1([2u8; 32]),
    );
    let err = delta.check_contract().expect_err("right value must reject");
    assert!(err
        .to_string()
        .contains("right deltas must not carry declared value units"));
}

#[test]
fn test_delta_accepts_redeem_shape() {
    let voucher_path = voucher_path(41);
    let mut prior = voucher_leaf(41);
    prior.lifecycle = VoucherLifecycleV1::Active;
    let mut residual = prior.clone();
    residual.remaining_value = 60;
    residual.lifecycle = VoucherLifecycleV1::PartiallyRedeemed;

    let asset_path = path(51, 1, 61);
    let asset = leaf(&asset_path, 7);
    let delta = ObjectDeltaSetV1::new(
        SettlementActionV1::Voucher(VoucherAction::RedeemPartial),
        prior.policy_id,
        Some(VoucherActionCtx {
            now: 20,
            expected_holder: Some(prior.holder_commitment),
            expected_beneficiary: Some(prior.beneficiary_commitment),
            acceptance_confirmed: true,
            ..VoucherActionCtx::default()
        }),
        Vec::new(),
        vec![SettlementObjectDeltaV1::created(
            asset_path,
            SettlementLeaf::Terminal(asset),
            Some(35),
        )],
        vec![SettlementObjectDeltaV1::updated(
            voucher_path,
            SettlementLeaf::Voucher(prior),
            SettlementLeaf::Voucher(residual),
            None,
        )],
        None,
        SettlementStateRoot::settlement_v1([1u8; 32]),
        SettlementStateRoot::settlement_v1([2u8; 32]),
    );
    delta
        .check_contract()
        .expect("partial redeem delta contract");
}

#[test]
fn test_delta_rejects_redeem_mismatch() {
    let voucher_path = voucher_path(43);
    let mut prior = voucher_leaf(43);
    prior.lifecycle = VoucherLifecycleV1::Active;
    let mut residual = prior.clone();
    residual.remaining_value = 60;
    residual.lifecycle = VoucherLifecycleV1::PartiallyRedeemed;

    let asset_path = path(53, 1, 63);
    let asset = leaf(&asset_path, 9);
    let delta = ObjectDeltaSetV1::new(
        SettlementActionV1::Voucher(VoucherAction::RedeemPartial),
        prior.policy_id,
        Some(VoucherActionCtx {
            now: 20,
            expected_holder: Some(prior.holder_commitment),
            expected_beneficiary: Some(prior.beneficiary_commitment),
            acceptance_confirmed: true,
            ..VoucherActionCtx::default()
        }),
        Vec::new(),
        vec![SettlementObjectDeltaV1::created(
            asset_path,
            SettlementLeaf::Terminal(asset),
            Some(34),
        )],
        vec![SettlementObjectDeltaV1::updated(
            voucher_path,
            SettlementLeaf::Voucher(prior),
            SettlementLeaf::Voucher(residual),
            None,
        )],
        None,
        SettlementStateRoot::settlement_v1([5u8; 32]),
        SettlementStateRoot::settlement_v1([6u8; 32]),
    );

    let err = delta
        .check_contract()
        .expect_err("mismatched voucher value accounting must reject");
    assert!(
        err.to_string().contains("conservation mismatch"),
        "unexpected error: {err}"
    );
}

#[test]
fn test_delta_rejects_refund_target() {
    let voucher_path = voucher_path(45);
    let pending = VoucherLeaf {
        lifecycle: VoucherLifecycleV1::PendingAcceptance,
        ..voucher_leaf(45)
    };
    let refund_path = SettlementPath::new(
        DefinitionId::new(match pending.backing {
            VoucherBackingRef::ReserveCommitment(backing)
            | VoucherBackingRef::GenesisReserve(backing) => backing,
            VoucherBackingRef::ConsumedAsset { definition_id, .. } => definition_id,
        }),
        SerialId::new(1),
        TerminalId::new([0xAA; 32]),
    );
    let delta = ObjectDeltaSetV1::new(
        SettlementActionV1::Voucher(VoucherAction::Reject),
        pending.policy_id,
        Some(VoucherActionCtx {
            now: 20,
            expected_holder: Some(pending.holder_commitment),
            expected_beneficiary: Some(pending.beneficiary_commitment),
            expected_refund_target: Some(pending.refund_target_commitment),
            policy_allows_reject: true,
            ..VoucherActionCtx::default()
        }),
        vec![SettlementObjectDeltaV1::deleted(
            voucher_path,
            SettlementLeaf::Voucher(pending.clone()),
            None,
        )],
        vec![SettlementObjectDeltaV1::created(
            refund_path,
            SettlementLeaf::Terminal(leaf(&refund_path, 11)),
            Some(pending.remaining_value),
        )],
        Vec::new(),
        None,
        SettlementStateRoot::settlement_v1([9u8; 32]),
        SettlementStateRoot::settlement_v1([10u8; 32]),
    );

    let err = delta
        .check_contract()
        .expect_err("refund target mismatch must reject");
    assert!(
        err.to_string().contains("refund output target"),
        "unexpected error: {err}"
    );
}

#[test]
fn test_delta_rejects_refund_ctx() {
    let voucher_path = voucher_path(46);
    let pending = VoucherLeaf {
        lifecycle: VoucherLifecycleV1::PendingAcceptance,
        backing: VoucherBackingRef::ConsumedAsset {
            definition_id: [0x46; 32],
            serial_id: 7,
        },
        ..voucher_leaf(46)
    };
    let refund_path = SettlementPath::new(
        DefinitionId::new([0x47; 32]),
        SerialId::new(7),
        TerminalId::new(pending.refund_target_commitment),
    );
    let delta = ObjectDeltaSetV1::new(
        SettlementActionV1::Voucher(VoucherAction::Reject),
        pending.policy_id,
        Some(VoucherActionCtx {
            now: 20,
            expected_holder: Some(pending.holder_commitment),
            expected_beneficiary: Some(pending.beneficiary_commitment),
            expected_refund_target: Some(pending.refund_target_commitment),
            policy_allows_reject: true,
            ..VoucherActionCtx::default()
        }),
        vec![SettlementObjectDeltaV1::deleted(
            voucher_path,
            SettlementLeaf::Voucher(pending.clone()),
            None,
        )],
        vec![SettlementObjectDeltaV1::created(
            refund_path,
            SettlementLeaf::Terminal(leaf(&refund_path, 12)),
            Some(pending.remaining_value),
        )],
        Vec::new(),
        None,
        SettlementStateRoot::settlement_v1([11u8; 32]),
        SettlementStateRoot::settlement_v1([12u8; 32]),
    );

    let err = delta
        .check_contract()
        .expect_err("restricted source mismatch must reject");
    assert!(
        err.to_string().contains("source context"),
        "unexpected error: {err}"
    );
}

#[test]
fn test_delta_rejects_reserve_ctx() {
    let voucher_path = voucher_path(47);
    let pending = VoucherLeaf {
        lifecycle: VoucherLifecycleV1::PendingAcceptance,
        backing: VoucherBackingRef::ReserveCommitment([0x47; 32]),
        ..voucher_leaf(47)
    };
    let refund_path = SettlementPath::new(
        DefinitionId::new([0x48; 32]),
        SerialId::new(1),
        TerminalId::new(pending.refund_target_commitment),
    );
    let delta = ObjectDeltaSetV1::new(
        SettlementActionV1::Voucher(VoucherAction::Reject),
        pending.policy_id,
        Some(VoucherActionCtx {
            now: 20,
            expected_holder: Some(pending.holder_commitment),
            expected_beneficiary: Some(pending.beneficiary_commitment),
            expected_refund_target: Some(pending.refund_target_commitment),
            policy_allows_reject: true,
            ..VoucherActionCtx::default()
        }),
        vec![SettlementObjectDeltaV1::deleted(
            voucher_path,
            SettlementLeaf::Voucher(pending.clone()),
            None,
        )],
        vec![SettlementObjectDeltaV1::created(
            refund_path,
            SettlementLeaf::Terminal(leaf(&refund_path, 13)),
            Some(pending.remaining_value),
        )],
        Vec::new(),
        None,
        SettlementStateRoot::settlement_v1([13u8; 32]),
        SettlementStateRoot::settlement_v1([14u8; 32]),
    );

    let err = delta
        .check_contract()
        .expect_err("reserve source mismatch must reject");
    assert!(
        err.to_string().contains("reserve backing"),
        "unexpected error: {err}"
    );
}

#[test]
fn test_delta_rejects_bad_fee() {
    let voucher_path = voucher_path(44);
    let pending = VoucherLeaf {
        lifecycle: VoucherLifecycleV1::PendingAcceptance,
        ..voucher_leaf(44)
    };
    let mut envelope = fee_envelope(44);
    envelope.budget_commitment = super::FeeEnvelope::budget_bind(
        envelope.budget_units.saturating_add(1),
        envelope.support_ref,
    );
    let delta = ObjectDeltaSetV1::new(
        SettlementActionV1::Voucher(VoucherAction::Issue),
        pending.policy_id,
        Some(VoucherActionCtx::default()),
        Vec::new(),
        vec![SettlementObjectDeltaV1::created(
            voucher_path,
            SettlementLeaf::Voucher(pending),
            None,
        )],
        Vec::new(),
        Some(envelope),
        SettlementStateRoot::settlement_v1([7u8; 32]),
        SettlementStateRoot::settlement_v1([8u8; 32]),
    );

    let err = delta
        .check_contract()
        .expect_err("malformed fee envelope must reject at package-contract time");
    assert!(
        err.to_string()
            .contains("fee envelope budget commitment mismatch"),
        "unexpected error: {err}"
    );
}

#[test]
fn test_delta_accepts_reserve_issue() {
    let voucher_path = voucher_path(42);
    let pending = VoucherLeaf {
        lifecycle: VoucherLifecycleV1::PendingAcceptance,
        ..voucher_leaf(42)
    };
    let delta = ObjectDeltaSetV1::new(
        SettlementActionV1::Voucher(VoucherAction::Issue),
        pending.policy_id,
        Some(VoucherActionCtx::default()),
        Vec::new(),
        vec![SettlementObjectDeltaV1::created(
            voucher_path,
            SettlementLeaf::Voucher(pending),
            None,
        )],
        Vec::new(),
        None,
        SettlementStateRoot::settlement_v1([3u8; 32]),
        SettlementStateRoot::settlement_v1([4u8; 32]),
    );
    delta
        .check_contract()
        .expect("reserve-backed issue delta contract");
}

#[test]
fn test_root_changes_def() {
    let left = build_root_from_path(path(1, 7, 9), 1);
    let right = build_root_from_path(path(2, 7, 9), 1);
    assert_ne!(left, right);
}

#[test]
fn test_root_changes_serial() {
    let left = build_root_from_path(path(1, 7, 9), 1);
    let right = build_root_from_path(path(1, 8, 9), 1);
    assert_ne!(left, right);
}

#[test]
fn test_root_changes_asset() {
    let left = build_root_from_path(path(1, 7, 9), 1);
    let right = build_root_from_path(path(1, 7, 8), 1);
    assert_ne!(left, right);
}

#[test]
fn test_leaf_id_match() {
    let path = path(1, 7, 9);
    let mut bad_leaf = leaf(&path, 1);
    bad_leaf.asset_id = [3u8; 32];

    let err = StoreItem::new(path, bad_leaf).unwrap_err();

    assert!(matches!(err, ModelErr::Right(RightErr::PathTerminalMix)));
}

#[test]
fn test_leaf_serial_match() {
    let path = path(1, 7, 9);
    let mut bad_leaf = leaf(&path, 1);
    bad_leaf.serial_id = 99;

    let err = StoreItem::new(path, bad_leaf).unwrap_err();

    assert!(matches!(err, ModelErr::PathSerialMix));
}

#[test]
fn test_path_vector() {
    let path_a = path(1, 7, 9);
    let path_b = path(2, 7, 5);
    let path_c = path(1, 7, 8);

    let mut model = SettlementModel::new();
    model
        .put_leaf(StoreItem::new(path_a, leaf(&path_a, 1)).expect("item a"))
        .expect("put a");
    model
        .put_leaf(StoreItem::new(path_b, leaf(&path_b, 2)).expect("item b"))
        .expect("put b");
    model
        .put_leaf(StoreItem::new(path_c, leaf(&path_c, 3)).expect("item c"))
        .expect("put c");

    let proof_a = model.proof_case(&path_a).expect("proof a");
    let proof_b = model.proof_case(&path_b).expect("proof b");
    let proof_c = model.proof_case(&path_c).expect("proof c");

    assert_eq!(proof_a.root(), proof_b.root());
    assert_eq!(proof_a.root(), proof_c.root());
    assert_eq!(proof_a.ser_leaf(), proof_c.ser_leaf());
    assert_ne!(proof_a.ser_leaf(), proof_b.ser_leaf());
    assert_ne!(proof_a.path(), proof_b.path());
    assert_ne!(proof_a.path(), proof_c.path());
}

#[test]
fn test_proof_def_path_mix() {
    let path = path(1, 7, 9);
    let root = build_root_from_path(path, 3);
    let err = ProofItem::new_settlement(
        root,
        path,
        DefinitionRootLeaf {
            definition_id: DefinitionId::new([2u8; 32]),
            definition_root: [4u8; 32],
        },
        SerialRootLeaf {
            definition_id: path.definition_id,
            serial_id: path.serial_id,
            serial_root: [5u8; 32],
        },
        leaf(&path, 3),
    )
    .unwrap_err();

    assert!(matches!(err, ModelErr::PathDefMix));
}

#[test]
fn test_proof_ser_path_mix() {
    let path = path(1, 7, 9);
    let root = build_root_from_path(path, 6);
    let err = ProofItem::new_settlement(
        root,
        path,
        DefinitionRootLeaf {
            definition_id: path.definition_id,
            definition_root: [4u8; 32],
        },
        SerialRootLeaf {
            definition_id: path.definition_id,
            serial_id: SerialId::new(8),
            serial_root: [5u8; 32],
        },
        leaf(&path, 6),
    )
    .unwrap_err();

    assert!(matches!(err, ModelErr::PathSerMix));
}

#[test]
fn test_tx_root_reject() {
    let err = TxDigest::new([7u8; 32]).to_check().unwrap_err();
    assert!(matches!(err, RootErr::TxRootMix));
}

#[test]
fn test_def_leaf_encode() {
    let leaf = DefinitionRootLeaf {
        definition_id: DefinitionId::new([7u8; 32]),
        definition_root: [9u8; 32],
    };

    let bytes = leaf.encode();

    assert_eq!(bytes.len(), 64);
    assert_eq!(&bytes[..32], leaf.definition_id.as_bytes());
    assert_eq!(&bytes[32..], &leaf.definition_root);
}

#[test]
fn test_ser_leaf_encode() {
    let leaf = SerialRootLeaf {
        definition_id: DefinitionId::new([5u8; 32]),
        serial_id: SerialId::new(11),
        serial_root: [3u8; 32],
    };

    let bytes = leaf.encode();

    assert_eq!(bytes.len(), 68);
    assert_eq!(&bytes[..32], leaf.definition_id.as_bytes());
    assert_eq!(&bytes[32..36], &11u32.to_le_bytes());
    assert_eq!(&bytes[36..], &leaf.serial_root);
}

#[test]
fn test_def_key_diff() {
    let left = DefinitionId::new([1u8; 32]);
    let right = DefinitionId::new([2u8; 32]);

    assert_ne!(definition_key(left), definition_key(right));
}

#[test]
fn test_def_leaf_diff() {
    let left = DefinitionRootLeaf {
        definition_id: DefinitionId::new([1u8; 32]),
        definition_root: [9u8; 32],
    };
    let right = DefinitionRootLeaf {
        definition_id: DefinitionId::new([2u8; 32]),
        definition_root: [9u8; 32],
    };

    assert_ne!(left.encode(), right.encode());
}

#[test]
fn test_def_update_scope() {
    let path_a = path(1, 7, 9);
    let path_b = path(2, 7, 5);
    let path_c = path(1, 8, 6);

    let mut model = SettlementModel::new();
    model
        .put_leaf(StoreItem::new(path_a, leaf(&path_a, 1)).expect("item a"))
        .expect("put a");
    model
        .put_leaf(StoreItem::new(path_b, leaf(&path_b, 2)).expect("item b"))
        .expect("put b");

    let proof_b_1 = model.proof_case(&path_b).expect("proof b1");
    let root_1 = proof_b_1.root();

    model
        .put_leaf(StoreItem::new(path_c, leaf(&path_c, 3)).expect("item c"))
        .expect("put c");

    let proof_b_2 = model.proof_case(&path_b).expect("proof b2");

    assert_ne!(root_1, proof_b_2.root());
    assert_eq!(proof_b_1.def_leaf(), proof_b_2.def_leaf());
    assert_eq!(proof_b_1.ser_leaf(), proof_b_2.ser_leaf());
}

#[test]
fn test_ser_key_diff() {
    let serial_id = SerialId::new(7);
    let left = serial_key(DefinitionId::new([1u8; 32]), serial_id);
    let right = serial_key(DefinitionId::new([2u8; 32]), serial_id);

    assert_ne!(left, right);
}

#[test]
fn test_asset_key_diff() {
    let left = terminal_key(TerminalId::new([1u8; 32]));
    let right = terminal_key(TerminalId::new([2u8; 32]));

    assert_ne!(left, right);
}

#[test]
fn test_ser_proof_diff() {
    let path_a = path(1, 7, 9);
    let path_b = path(2, 7, 9);

    let mut model = SettlementModel::new();
    model
        .put_leaf(StoreItem::new(path_a, leaf(&path_a, 1)).expect("item a"))
        .expect("put a");
    model
        .put_leaf(StoreItem::new(path_b, leaf(&path_b, 2)).expect("item b"))
        .expect("put b");

    let proof_a = model.proof_case(&path_a).expect("proof a");
    let proof_b = model.proof_case(&path_b).expect("proof b");

    assert_ne!(proof_a.ser_leaf(), proof_b.ser_leaf());
}

#[test]
fn test_serial_prune() {
    let path_a = path(1, 7, 9);
    let path_b = path(1, 8, 5);

    let mut model = SettlementModel::new();
    model
        .put_leaf(StoreItem::new(path_a, leaf(&path_a, 1)).expect("item a"))
        .expect("put a");
    model
        .put_leaf(StoreItem::new(path_b, leaf(&path_b, 2)).expect("item b"))
        .expect("put b");

    let proof_b_1 = model.proof_case(&path_b).expect("proof b1");
    model.del_leaf(&path_a).expect("del a");
    let proof_b_2 = model.proof_case(&path_b).expect("proof b2");

    assert!(matches!(model.proof_case(&path_a), Err(ModelErr::NoSerial)));
    assert_eq!(proof_b_1.ser_leaf(), proof_b_2.ser_leaf());
    assert_ne!(proof_b_1.def_leaf(), proof_b_2.def_leaf());
}

#[test]
fn test_terminal_proof_diff() {
    let path_a = path(1, 7, 9);
    let path_b = path(1, 7, 8);
    let mut leaf_a = leaf(&path_a, 1);
    let mut leaf_b = leaf(&path_a, 1);
    leaf_a.asset_id = path_a.terminal_id().into_bytes();
    leaf_b.asset_id = path_b.terminal_id().into_bytes();

    let mut model = SettlementModel::new();
    model
        .put_leaf(StoreItem::new(path_a, leaf_a).expect("item a"))
        .expect("put a");
    model
        .put_leaf(StoreItem::new(path_b, leaf_b).expect("item b"))
        .expect("put b");

    let proof_a = model.proof_case(&path_a).expect("proof a");
    let proof_b = model.proof_case(&path_b).expect("proof b");

    assert_ne!(proof_a.path(), proof_b.path());
    assert_ne!(proof_a.leaf().terminal_id(), proof_b.leaf().terminal_id());
}

#[test]
fn test_asset_hash_canon() {
    #[derive(Serialize)]
    struct Wrap<'a> {
        tag: &'a str,
        leaf: &'a TerminalLeaf,
    }

    let path = path(1, 7, 9);
    let leaf = leaf(&path, 1);
    let codec = BincodeCodec;
    let payload = codec.serialize(&leaf).expect("payload");
    let wrap_a = codec
        .serialize(&Wrap {
            tag: "left",
            leaf: &leaf,
        })
        .expect("wrap a");
    let wrap_b = codec
        .serialize(&Wrap {
            tag: "right",
            leaf: &leaf,
        })
        .expect("wrap b");
    let settlement_leaf = SettlementLeaf::from(leaf.clone());

    assert_ne!(wrap_a, wrap_b);
    assert_eq!(
        terminal_leaf_hash(&settlement_leaf).expect("hash a"),
        terminal_leaf_hash(&settlement_leaf).expect("hash b")
    );
    assert_ne!(payload, wrap_a);
    assert_ne!(payload, wrap_b);
}

proptest! {
    #[test]
    fn test_rebuild_idx_keeps_root(
        marks in proptest::collection::vec((1u8..4, 1u32..4, 1u8..8, 1u8..16), 1..8)
    ) {
        let mut seen = std::collections::BTreeSet::new();
        let mut left = Vec::new();
        let mut right = Vec::new();

        for (def_mark, serial_id, asset_mark, mark) in marks {
            if !seen.insert((def_mark, serial_id, asset_mark)) {
                continue;
            }
            let item = item(def_mark, serial_id, asset_mark, mark);
            left.push(item.clone());
            right.insert(0, item);
        }

        prop_assert_eq!(root_many(&left), root_many(&right));
    }
}
