use z00z_core::assets::AssetLeaf;
use z00z_core::vouchers::{VoucherLifecycleV1, VoucherValidityWindowV1};
use z00z_crypto::{
    expert::{hash_domain, traits::DomainSeparation},
    poseidon2_hash,
};
use z00z_storage::settlement::{
    DefinitionId, RightClass, RightLeaf, SerialId, SettlementLeaf, SettlementPath,
    SettlementPathErr, TerminalId, TerminalLeaf, VoucherBackingRef, VoucherLeaf,
};
use z00z_utils::codec::{BincodeCodec, Codec};

hash_domain!(TestStorTerminalDom, "z00z.storage.terminal", 1);

fn bytes(value: u8) -> [u8; 32] {
    [value; 32]
}

fn payload_hash(payload: &[u8]) -> [u8; 32] {
    let domain = TestStorTerminalDom::domain_separation_tag("").into_bytes();
    poseidon2_hash(domain.as_slice(), &[payload])
}

fn asset_leaf(index: u32) -> TerminalLeaf {
    TerminalLeaf::from(AssetLeaf::dummy_for_scan(index))
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

fn voucher_leaf(mark: u8) -> VoucherLeaf {
    VoucherLeaf {
        version: 1,
        terminal_id: TerminalId::new(bytes(mark)),
        issuer_commitment: bytes(mark.wrapping_add(12)),
        holder_commitment: bytes(mark.wrapping_add(13)),
        beneficiary_commitment: bytes(mark.wrapping_add(14)),
        refund_target_commitment: bytes(mark.wrapping_add(15)),
        backing: VoucherBackingRef::ReserveCommitment(bytes(mark.wrapping_add(16))),
        face_value: 90,
        remaining_value: 65,
        policy_id: bytes(mark.wrapping_add(17)),
        action_pool_id: bytes(mark.wrapping_add(18)),
        lifecycle: VoucherLifecycleV1::Active,
        validity: VoucherValidityWindowV1 {
            valid_from: 10,
            valid_until: 20,
        },
        receiver_must_accept: true,
        allow_reject: true,
        replay_nonce: bytes(mark.wrapping_add(19)),
        disclosure_commitment: Some(bytes(mark.wrapping_add(20))),
        audit_commitment: Some(bytes(mark.wrapping_add(21))),
    }
}

#[test]
fn test_family_markers_commit_roundtrip() {
    let asset = SettlementLeaf::Terminal(asset_leaf(11));
    let right = SettlementLeaf::Right(right_leaf(19));
    let voucher = SettlementLeaf::Voucher(voucher_leaf(27));

    let asset_bytes = asset.encode().expect("asset encode");
    let right_bytes = right.encode().expect("right encode");
    let voucher_bytes = voucher.encode().expect("voucher encode");

    assert_eq!(asset.family_tag(), 1);
    assert_eq!(right.family_tag(), 2);
    assert_eq!(voucher.family_tag(), 3);
    assert_eq!(asset_bytes[0], 1);
    assert_eq!(right_bytes[0], 2);
    assert_eq!(voucher_bytes[0], 3);
    assert_ne!(asset_bytes, right_bytes);
    assert_ne!(asset_bytes, voucher_bytes);
    assert_ne!(right_bytes, voucher_bytes);
    assert_ne!(payload_hash(&asset_bytes), payload_hash(&right_bytes));
    assert_ne!(payload_hash(&asset_bytes), payload_hash(&voucher_bytes));
    assert_ne!(payload_hash(&right_bytes), payload_hash(&voucher_bytes));
    assert_eq!(
        SettlementLeaf::decode(&asset_bytes).expect("asset decode"),
        asset
    );
    assert_eq!(
        SettlementLeaf::decode(&right_bytes).expect("right decode"),
        right
    );
    assert_eq!(
        SettlementLeaf::decode(&voucher_bytes).expect("voucher decode"),
        voucher
    );
}

#[test]
fn test_path_rejects_terminal_mismatch() {
    let leaf = asset_leaf(7);
    let asset_id = TerminalId::new(leaf.asset_id);
    let settlement_path =
        SettlementPath::new(DefinitionId::new(bytes(1)), SerialId::new(3), asset_id);
    let good = SettlementLeaf::Terminal(leaf.clone());

    good.check_path(settlement_path).expect("asset path ok");

    let err = good
        .check_path(SettlementPath::new(
            settlement_path.definition_id,
            settlement_path.serial_id,
            TerminalId::new(bytes(9)),
        ))
        .expect_err("terminal mismatch must reject");
    assert_eq!(
        err.to_string(),
        "settlement path terminal id does not match leaf terminal id"
    );
}

#[test]
fn test_voucher_rejects_terminal() {
    let leaf = voucher_leaf(31);
    let settlement_path = SettlementPath::new(
        DefinitionId::new(bytes(7)),
        SerialId::new(9),
        leaf.terminal_id,
    );
    let good = SettlementLeaf::Voucher(leaf.clone());

    good.check_path(settlement_path).expect("voucher path ok");

    let err = good
        .check_path(SettlementPath::new(
            settlement_path.definition_id,
            settlement_path.serial_id,
            TerminalId::new(bytes(99)),
        ))
        .expect_err("voucher terminal mismatch must reject");
    assert_eq!(
        err.to_string(),
        "settlement path terminal id does not match leaf terminal id"
    );
}

#[test]
fn test_duplicate_terminal_id_rejects() {
    let terminal = TerminalId::new(bytes(5));
    let paths = [
        SettlementPath::new(DefinitionId::new(bytes(1)), SerialId::new(1), terminal),
        SettlementPath::new(DefinitionId::new(bytes(2)), SerialId::new(2), terminal),
    ];

    let err = SettlementPath::check_unique(&paths).expect_err("dup terminal id");
    assert_eq!(err, SettlementPathErr::DupTerminalId);
}

#[test]
fn test_zero_terminal_path_rejects() {
    let err = SettlementPath::new(
        DefinitionId::new(bytes(1)),
        SerialId::new(1),
        TerminalId::new([0u8; 32]),
    )
    .check()
    .expect_err("zero terminal id");
    assert_eq!(err, SettlementPathErr::ZeroTerminalId);
}

#[test]
fn test_no_family_tag_reject() {
    let codec = BincodeCodec;
    let leaf = asset_leaf(5);
    let raw = codec.serialize(&leaf).expect("raw asset bytes");
    assert!(SettlementLeaf::decode(&raw).is_err());
}
