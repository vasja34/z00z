use z00z_core::assets::AssetLeaf;
use z00z_storage::settlement::{
    DefinitionId, RightAction, RightActionCtx, RightClass, RightErr, RightLeaf, SerialId,
    SettlementLeaf, SettlementPath, TerminalId, TerminalLeaf,
};
use z00z_utils::codec::{BincodeCodec, Codec, JsonCodec};

fn bytes(value: u8) -> [u8; 32] {
    [value; 32]
}

fn right_leaf(mark: u8) -> RightLeaf {
    RightLeaf {
        version: 1,
        terminal_id: TerminalId::new(bytes(mark)),
        right_class: RightClass::MachineCapability,
        issuer_scope: bytes(mark.wrapping_add(1)),
        provider_scope: bytes(mark.wrapping_add(2)),
        holder_commitment: bytes(mark.wrapping_add(3)),
        control_commitment: bytes(mark.wrapping_add(4)),
        beneficiary_commitment: bytes(mark.wrapping_add(5)),
        payload_commitment: bytes(mark.wrapping_add(6)),
        valid_from: 10,
        valid_until: 20,
        challenge_from: 12,
        challenge_until: 18,
        use_nonce: bytes(mark.wrapping_add(6)),
        revocation_policy_id: bytes(mark.wrapping_add(7)),
        transition_policy_id: bytes(mark.wrapping_add(8)),
        challenge_policy_id: bytes(mark.wrapping_add(9)),
        disclosure_policy_id: bytes(mark.wrapping_add(10)),
        retention_policy_id: bytes(mark.wrapping_add(11)),
    }
}

fn delegated_right(prior: RightLeaf, mark: u8) -> RightLeaf {
    let mut next = prior;
    next.holder_commitment = bytes(mark.wrapping_add(20));
    next.control_commitment = bytes(mark.wrapping_add(21));
    next.beneficiary_commitment = bytes(mark.wrapping_add(22));
    next.use_nonce = bytes(mark.wrapping_add(23));
    next.valid_from = prior.valid_from + 1;
    next.valid_until = prior.valid_until - 1;
    next.challenge_from = prior.challenge_from + 1;
    next.challenge_until = prior.challenge_until - 1;
    next
}

#[test]
fn test_canon_encoding_golden() {
    let codec = BincodeCodec;
    let right = right_leaf(8);
    let leaf = SettlementLeaf::Right(right);
    let mut expected = vec![2u8];
    expected.extend(codec.serialize(&right).expect("right bincode"));

    assert_eq!(leaf.encode().expect("leaf encode"), expected);
}

#[test]
fn test_right_path_mismatch_rejects() {
    let right = right_leaf(9);
    let err = right
        .check_path(SettlementPath::new(
            DefinitionId::new(bytes(1)),
            SerialId::new(2),
            TerminalId::new(bytes(7)),
        ))
        .expect_err("path mismatch");
    assert_eq!(err, RightErr::PathTerminalMix);
}

#[test]
fn test_wrong_right_class_rejects() {
    let codec = JsonCodec;
    let right = right_leaf(10);
    let mut json = String::from_utf8(codec.serialize(&right).expect("right json")).expect("utf8");
    json = json.replace("\"machine_capability\"", "\"wrong_class\"");
    assert!(codec.deserialize::<RightLeaf>(json.as_bytes()).is_err());
}

#[test]
fn test_expired_transition_rejects() {
    let prior = right_leaf(11);
    let next = prior;
    let err = next
        .validate_action(
            RightAction::Transfer,
            RightActionCtx {
                now: 25,
                expected_holder: Some(next.holder_commitment),
                ..RightActionCtx::default()
            },
            Some(&prior),
        )
        .expect_err("expired transfer");
    assert_eq!(err, RightErr::Expired);
}

#[test]
fn test_revoked_transition_rejects() {
    let prior = right_leaf(12);
    let next = prior;
    let err = next
        .validate_action(
            RightAction::Transfer,
            RightActionCtx {
                now: 15,
                expected_holder: Some(next.holder_commitment),
                revoked: true,
                ..RightActionCtx::default()
            },
            Some(&prior),
        )
        .expect_err("revoked transfer");
    assert_eq!(err, RightErr::Revoked);
}

#[test]
fn test_one_time_replay_rejects() {
    let prior = right_leaf(13);
    let next = prior;
    let err = next
        .validate_action(
            RightAction::Consume,
            RightActionCtx {
                now: 15,
                expected_holder: Some(next.holder_commitment),
                seen_use_nonce: Some(next.use_nonce),
                ..RightActionCtx::default()
            },
            Some(&prior),
        )
        .expect_err("replay");
    assert_eq!(err, RightErr::OneTimeReplay);
}

#[test]
fn test_wrong_holder_binding_rejects() {
    let prior = right_leaf(14);
    let next = prior;
    let err = next
        .validate_action(
            RightAction::Transfer,
            RightActionCtx {
                now: 15,
                expected_holder: Some(bytes(0xEE)),
                ..RightActionCtx::default()
            },
            Some(&prior),
        )
        .expect_err("holder mix");
    assert_eq!(err, RightErr::HolderControlMix);
}

#[test]
fn test_shaped_bytes_confusion_rejects() {
    let right = SettlementLeaf::Right(right_leaf(15));
    let asset = SettlementLeaf::Terminal(TerminalLeaf::from(AssetLeaf::dummy_for_scan(15)));

    assert_ne!(
        right.encode().expect("right encode"),
        asset.encode().expect("asset encode")
    );
    assert!(right.as_terminal().is_none());
}

#[test]
fn test_fields_absent_right_leaf() {
    let codec = JsonCodec;
    let right = right_leaf(16);
    let json = String::from_utf8(codec.serialize(&right).expect("right json")).expect("utf8");
    let injected = json.replacen(
        "}",
        ",\"payer_commitment\":[1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1]}",
        1,
    );
    assert!(codec.deserialize::<RightLeaf>(injected.as_bytes()).is_err());
}

#[test]
fn test_missing_transition_policy_rejects() {
    let prior = right_leaf(17);
    let mut next = prior;
    next.transition_policy_id = [0u8; 32];
    let err = next
        .validate_action(
            RightAction::Transfer,
            RightActionCtx {
                now: 15,
                expected_holder: Some(next.holder_commitment),
                ..RightActionCtx::default()
            },
            Some(&prior),
        )
        .expect_err("transition policy must reject");
    assert_eq!(err, RightErr::TransitionPolicyMix);
}

#[test]
fn test_right_delegate_accepts_narrow() {
    let prior = right_leaf(19);
    let next = delegated_right(prior, 19);

    next.validate_action(
        RightAction::Transfer,
        RightActionCtx {
            now: 15,
            expected_holder: Some(next.holder_commitment),
            expected_control: Some(next.control_commitment),
            ..RightActionCtx::default()
        },
        Some(&prior),
    )
    .expect("narrow delegation must stay valid");
}

#[test]
fn test_right_delegate_validity_wide() {
    let prior = right_leaf(20);
    let mut next = delegated_right(prior, 20);
    next.valid_until = prior.valid_until + 1;

    let err = next
        .validate_action(
            RightAction::Transfer,
            RightActionCtx {
                now: 15,
                expected_holder: Some(next.holder_commitment),
                expected_control: Some(next.control_commitment),
                ..RightActionCtx::default()
            },
            Some(&prior),
        )
        .expect_err("wider validity must reject");
    assert_eq!(err, RightErr::TransitionMix);
}

#[test]
fn test_right_delegate_policy_drift() {
    let prior = right_leaf(21);
    let mut next = delegated_right(prior, 21);
    next.transition_policy_id = bytes(0xFE);

    let err = next
        .validate_action(
            RightAction::Transfer,
            RightActionCtx {
                now: 15,
                expected_holder: Some(next.holder_commitment),
                expected_control: Some(next.control_commitment),
                ..RightActionCtx::default()
            },
            Some(&prior),
        )
        .expect_err("policy drift must reject");
    assert_eq!(err, RightErr::TransitionMix);
}

#[test]
fn test_right_delegate_scope_drift() {
    let prior = right_leaf(22);
    let mut next = delegated_right(prior, 22);
    next.provider_scope = bytes(0xFD);

    let err = next
        .validate_action(
            RightAction::Transfer,
            RightActionCtx {
                now: 15,
                expected_holder: Some(next.holder_commitment),
                expected_control: Some(next.control_commitment),
                ..RightActionCtx::default()
            },
            Some(&prior),
        )
        .expect_err("scope drift must reject");
    assert_eq!(err, RightErr::TransitionMix);
}

#[test]
fn test_challenge_outside_window_rejects() {
    let prior = right_leaf(18);
    let next = prior;
    let err = next
        .validate_action(
            RightAction::Challenge,
            RightActionCtx {
                now: 19,
                expected_holder: Some(next.holder_commitment),
                ..RightActionCtx::default()
            },
            Some(&prior),
        )
        .expect_err("challenge window must reject");
    assert_eq!(err, RightErr::ChallengeWindow);
}
