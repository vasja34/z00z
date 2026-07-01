use serde::{Deserialize, Serialize};
use sha2::Sha256;
use std::time::{Duration, Instant};
use z00z_core::{
    assets::AssetLeaf,
    vouchers::{VoucherLifecycleV1, VoucherValidityWindowV1},
};
use z00z_storage::settlement::{
    chk_blob_settlement, chk_blob_settlement_inclusion, hjmt_default_child_commitment,
    hjmt_default_value_commitment, BucketPolicy, BucketRootLeaf, DefinitionId, DefinitionRootLeaf,
    HjmtProofFamily, ProofBlob, ProofChkErr, ProofItem, RightClass, RightLeaf, SerialId,
    SerialRootLeaf, SettlementLeaf, SettlementLeafFamily, SettlementPath, SettlementStateRoot,
    SettlementStore, SettlementStoreError, StoreItem, StoreOp, TerminalId, TerminalLeaf,
    VoucherBackingRef, VoucherLeaf, HJMT_DEFAULT_COMMITMENT_VERSION,
};
use z00z_utils::codec::{BincodeCodec, Codec};

#[derive(Clone, Debug, Serialize, Deserialize)]
struct MirrorHjmtPriorProofEnvelope {
    version: u64,
    settlement_root: SettlementStateRoot,
    backend_root: [u8; 32],
    root_bind_ver: u8,
    root_bind: [u8; 32],
    journal_digest: Option<[u8; 32]>,
    checkpoint_bind: Option<[u8; 32]>,
    definition_root_leaf: DefinitionRootLeaf,
    serial_root_leaf: SerialRootLeaf,
    bucket_root_leaf: BucketRootLeaf,
    definition_proof: Vec<u8>,
    serial_proof: Vec<u8>,
    bucket_proof: Vec<u8>,
    terminal_proof: Vec<u8>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct MirrorHjmtProofEnvelope {
    version: u8,
    family: HjmtProofFamily,
    leaf_family: SettlementLeafFamily,
    journal_checkpoint: Option<u64>,
    journal_digest: Option<[u8; 32]>,
    checkpoint_bind: Option<[u8; 32]>,
    default_commitment_version: Option<u8>,
    default_commitment: Option<[u8; 32]>,
    default_child_commitment: Option<[u8; 32]>,
    bucket_policy: BucketPolicy,
    bucket_root_leaf: BucketRootLeaf,
    bucket_proof: Vec<u8>,
    prior: Option<MirrorHjmtPriorProofEnvelope>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct MirrorProofBlob {
    item: ProofItem,
    terminal_leaf_hash: [u8; 32],
    backend_root: [u8; 32],
    root_bind_ver: u8,
    root_bind: [u8; 32],
    definition_proof: Vec<u8>,
    serial_proof: Vec<u8>,
    terminal_proof: Vec<u8>,
    hjmt: Option<MirrorHjmtProofEnvelope>,
}

fn bytes(value: u8) -> [u8; 32] {
    [value; 32]
}

fn settlement_leaf_hash(leaf: SettlementLeaf) -> [u8; 32] {
    jmt::ValueHash::with::<Sha256>(&leaf.encode().expect("encode settlement leaf")).0
}

fn leaf_for_path(path: SettlementPath) -> TerminalLeaf {
    let mut core = AssetLeaf::dummy_for_scan(path.serial_id.get());
    core.asset_id = path.terminal_id().into_bytes();
    TerminalLeaf::from(core)
}

fn item_for_path(path: SettlementPath) -> StoreItem {
    StoreItem::new(path, leaf_for_path(path)).expect("store item")
}

fn rewrite_prior_blob(
    blob: &ProofBlob,
    rewrite: impl FnOnce(&mut MirrorHjmtPriorProofEnvelope),
) -> ProofBlob {
    let codec = BincodeCodec;
    let bytes = blob.encode().expect("encode proof blob");
    let mut mirror: MirrorProofBlob = codec.deserialize(&bytes).expect("decode proof blob");
    let prior = mirror
        .hjmt
        .as_mut()
        .and_then(|hjmt| hjmt.prior.as_mut())
        .expect("deletion proof must carry prior context");
    rewrite(prior);
    let bytes = codec
        .serialize(&mirror)
        .expect("encode rewritten proof blob");
    ProofBlob::decode(&bytes).expect("decode rewritten proof blob")
}

#[cfg(feature = "test-params-fast")]
fn assert_tampered_prior_context_rejects(
    store: &SettlementStore,
    blob: &ProofBlob,
    deleted_path: SettlementPath,
    deleted_item: &StoreItem,
) {
    let mut alt_store = SettlementStore::new();
    alt_store
        .apply_settlement_ops(vec![StoreOp::Put(Box::new(deleted_item.clone()))])
        .expect("seed alternate prior target");
    let alt_unrelated_path = SettlementPath::new(
        DefinitionId::new(bytes(54)),
        SerialId::new(9),
        TerminalId::new(bytes(67)),
    );
    alt_store
        .apply_settlement_ops(vec![StoreOp::Put(Box::new(item_for_path(
            alt_unrelated_path,
        )))])
        .expect("advance alternate prior state");
    let alt_prior_blob = alt_store
        .settlement_proof_blob(&deleted_path)
        .expect("alternate prior inclusion blob");
    assert_ne!(
        alt_prior_blob.backend_root(),
        blob.hjmt_prior_backend_root()
            .expect("deletion proof must carry prior backend root")
    );

    let stale_prior = blob.clone().with_hjmt_prior_blob(&alt_prior_blob);
    let err = store
        .validate_settlement_proof_blob(&stale_prior)
        .expect_err("tampered prior backend root must reject");
    assert!(matches!(
        err,
        SettlementStoreError::Proof(ProofChkErr::PriorRootMix)
    ));
}

#[cfg(not(feature = "test-params-fast"))]
fn assert_tampered_prior_context_rejects(
    _store: &SettlementStore,
    _blob: &ProofBlob,
    _deleted_path: SettlementPath,
    _deleted_item: &StoreItem,
) {
}

fn empty_marker_leaf(path: SettlementPath) -> TerminalLeaf {
    TerminalLeaf::from(AssetLeaf {
        asset_id: path.terminal_id().into_bytes(),
        serial_id: path.serial_id.get(),
        r_pub: [0u8; 32],
        owner_tag: [0u8; 32],
        c_amount: [0u8; 32],
        enc_pack: z00z_crypto::ZkPackEncrypted {
            version: 1,
            ciphertext: Vec::new(),
            tag: [0u8; 16],
        },
        range_proof: Vec::new(),
        tag16: 0,
    })
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
        use_nonce: bytes(mark.wrapping_add(7)),
        revocation_policy_id: bytes(mark.wrapping_add(8)),
        transition_policy_id: bytes(mark.wrapping_add(9)),
        challenge_policy_id: bytes(mark.wrapping_add(10)),
        disclosure_policy_id: bytes(mark.wrapping_add(11)),
        retention_policy_id: bytes(mark.wrapping_add(12)),
    }
}

fn right_item(mark: u8) -> StoreItem {
    let path = SettlementPath::new(
        DefinitionId::new(bytes(mark.wrapping_add(21))),
        SerialId::new(u32::from(mark) + 1),
        TerminalId::new(bytes(mark)),
    );
    StoreItem::new(path, SettlementLeaf::Right(right_leaf(mark))).expect("right store item")
}

fn voucher_leaf(mark: u8) -> VoucherLeaf {
    VoucherLeaf {
        version: 1,
        terminal_id: TerminalId::new(bytes(mark)),
        issuer_commitment: bytes(mark.wrapping_add(31)),
        holder_commitment: bytes(mark.wrapping_add(32)),
        beneficiary_commitment: bytes(mark.wrapping_add(33)),
        refund_target_commitment: bytes(mark.wrapping_add(34)),
        backing: VoucherBackingRef::ReserveCommitment(bytes(mark.wrapping_add(35))),
        face_value: 120,
        remaining_value: 80,
        policy_id: bytes(mark.wrapping_add(36)),
        action_pool_id: bytes(mark.wrapping_add(37)),
        lifecycle: VoucherLifecycleV1::Active,
        validity: VoucherValidityWindowV1 {
            valid_from: 10,
            valid_until: 30,
        },
        receiver_must_accept: true,
        allow_reject: true,
        replay_nonce: bytes(mark.wrapping_add(38)),
        disclosure_commitment: Some(bytes(mark.wrapping_add(39))),
        audit_commitment: Some(bytes(mark.wrapping_add(40))),
    }
}

fn voucher_item(mark: u8) -> StoreItem {
    let path = SettlementPath::new(
        DefinitionId::new(bytes(mark.wrapping_add(41))),
        SerialId::new(u32::from(mark) + 2),
        TerminalId::new(bytes(mark)),
    );
    StoreItem::new(path, SettlementLeaf::Voucher(voucher_leaf(mark))).expect("voucher store item")
}

fn sibling_path_same_bucket(
    store: &SettlementStore,
    base: SettlementPath,
    start_seed: u16,
) -> SettlementPath {
    same_bucket_paths(store, base, start_seed, 2)[1]
}

fn same_bucket_paths(
    store: &SettlementStore,
    base: SettlementPath,
    start_seed: u16,
    need: usize,
) -> Vec<SettlementPath> {
    let target_bucket = store.bucket_policy().derive_bucket_id(base);
    let def_mark = base.definition_id.into_bytes()[0];
    let serial_mark = base.serial_id.get() as u8;
    let mut out = vec![base];
    for seed in start_seed..=u16::MAX {
        let mut terminal = [0u8; 32];
        terminal[0] = (seed >> 8) as u8;
        terminal[1] = seed as u8;
        terminal[2] = def_mark;
        terminal[3] = serial_mark;
        let candidate = SettlementPath::new(
            base.definition_id,
            base.serial_id,
            TerminalId::new(terminal),
        );
        if !out.contains(&candidate)
            && store.bucket_policy().derive_bucket_id(candidate) == target_bucket
        {
            out.push(candidate);
            if out.len() == need {
                return out;
            }
        }
    }
    panic!("missing same-bucket sibling path fixture set");
}

#[test]
fn test_miss_blob_needs_leaffam() {
    let mut store = SettlementStore::new();
    let present_path = SettlementPath::new(
        DefinitionId::new(bytes(41)),
        SerialId::new(1),
        TerminalId::new(bytes(7)),
    );
    let present_item = item_for_path(present_path);
    let root = store
        .apply_settlement_ops(vec![StoreOp::Put(Box::new(present_item))])
        .expect("seed settlement state");

    let missing_path = sibling_path_same_bucket(&store, present_path, 8);
    let blob = store
        .settlement_nonexistence_proof_blob(&missing_path, SettlementLeafFamily::Terminal)
        .expect("nonexistence proof blob");

    assert_eq!(
        blob.hjmt_proof_family(),
        Some(HjmtProofFamily::NonExistence)
    );
    assert_eq!(
        blob.hjmt_leaf_family(),
        Some(SettlementLeafFamily::Terminal)
    );
    assert_eq!(blob.hjmt_journal_checkpoint(), Some(1));
    assert_eq!(
        blob.hjmt_default_commitment(),
        Some(hjmt_default_value_commitment())
    );
    assert_eq!(
        blob.hjmt_default_child_commitment(),
        Some(hjmt_default_child_commitment())
    );
    assert_eq!(
        blob.hjmt_default_commitment_version(),
        Some(HJMT_DEFAULT_COMMITMENT_VERSION)
    );
    assert!(blob.hjmt_journal_digest().is_some());
    let err = store
        .validate_settlement_proof_blob(&blob)
        .expect_err("generic settlement validator must reject family-free nonexistence checks");
    assert!(matches!(
        err,
        SettlementStoreError::Proof(ProofChkErr::ProofFamilyMix)
    ));
    store
        .validate_settlement_nonexistence_proof_blob(&blob, SettlementLeafFamily::Terminal)
        .expect("store-context nonexistence validation");

    let checked = chk_blob_settlement(
        &blob.encode().expect("encode blob"),
        root,
        &missing_path,
        blob.item().def_leaf(),
        blob.item().ser_leaf(),
        blob.item()
            .terminal_leaf()
            .expect("marker asset leaf")
            .clone(),
    )
    .expect("static nonexistence validation");
    assert_eq!(
        checked.hjmt_proof_family(),
        Some(HjmtProofFamily::NonExistence)
    );

    let err = store
        .settlement_nonexistence_proof_blob(&present_path, SettlementLeafFamily::Terminal)
        .expect_err("present path must not produce a nonexistence proof");
    assert!(matches!(err, SettlementStoreError::PathMiss));

    let err = store
        .validate_settlement_proof_blob(&blob.clone().with_hjmt_default_commitment(Some([0u8; 32])))
        .expect_err("tampered default commitment must reject");
    assert!(matches!(
        err,
        SettlementStoreError::Proof(ProofChkErr::DefaultCommitmentMix)
    ));

    let err = store
        .validate_settlement_proof_blob(
            &blob
                .clone()
                .with_hjmt_default_child_commitment(Some([1u8; 32])),
        )
        .expect_err("tampered child default commitment must reject");
    assert!(matches!(
        err,
        SettlementStoreError::Proof(ProofChkErr::DefaultCommitmentMix)
    ));

    let err = store
        .validate_settlement_proof_blob(
            &blob
                .clone()
                .with_hjmt_default_commitment_version(Some(HJMT_DEFAULT_COMMITMENT_VERSION + 1)),
        )
        .expect_err("tampered default commitment version must reject");
    assert!(matches!(
        err,
        SettlementStoreError::Proof(ProofChkErr::DefaultCommitmentMix)
    ));

    let static_err = chk_blob_settlement(
        &blob
            .clone()
            .with_hjmt_journal_digest(Some([2u8; 32]))
            .encode()
            .expect("encode blob"),
        root,
        &missing_path,
        blob.item().def_leaf(),
        blob.item().ser_leaf(),
        blob.item()
            .terminal_leaf()
            .expect("marker asset leaf")
            .clone(),
    )
    .expect_err("tampered journal digest must reject statically");
    assert_eq!(static_err, ProofChkErr::JournalCheckpointMix);

    let wrong_marker = SettlementLeafFamily::Right.marker_leaf(missing_path);
    let wrong_item = z00z_storage::settlement::ProofItem::new_settlement(
        blob.item().settlement_root(),
        missing_path,
        blob.item().def_leaf(),
        blob.item().ser_leaf(),
        wrong_marker.clone(),
    )
    .expect("wrong marker item");
    let retagged = blob
        .clone()
        .rebind(wrong_item)
        .with_hjmt_leaf_family(SettlementLeafFamily::Right);

    let static_err = chk_blob_settlement(
        &retagged.encode().expect("encode blob"),
        root,
        &missing_path,
        retagged.item().def_leaf(),
        retagged.item().ser_leaf(),
        wrong_marker,
    )
    .expect_err("retagged nonexistence family must reject statically");
    assert_eq!(static_err, ProofChkErr::LeafHashMix);

    let err = store
        .validate_settlement_nonexistence_proof_blob(&retagged, SettlementLeafFamily::Terminal)
        .expect_err("retagged nonexistence family must reject in store context");
    assert!(matches!(
        err,
        SettlementStoreError::Proof(ProofChkErr::LeafHashMix | ProofChkErr::LeafMix)
    ));

    let mut wrong_same_family = empty_marker_leaf(missing_path);
    wrong_same_family.owner_tag[0] ^= 0x5a;
    let wrong_same_family_hash = jmt::ValueHash::with::<Sha256>(
        &SettlementLeaf::from(wrong_same_family.clone())
            .encode()
            .expect("encode wrong same-family leaf"),
    )
    .0;
    let wrong_same_family_item = blob.item().clone();
    let wrong_same_family_item = z00z_storage::settlement::ProofItem::new_settlement(
        wrong_same_family_item.settlement_root(),
        missing_path,
        wrong_same_family_item.def_leaf(),
        wrong_same_family_item.ser_leaf(),
        SettlementLeaf::from(wrong_same_family.clone()),
    )
    .expect("same-family wrong marker item");
    let wrong_same_family_blob = blob
        .clone()
        .rebind(wrong_same_family_item)
        .with_terminal_leaf_hash(wrong_same_family_hash);

    let static_err = chk_blob_settlement(
        &wrong_same_family_blob.encode().expect("encode blob"),
        root,
        &missing_path,
        wrong_same_family_blob.item().def_leaf(),
        wrong_same_family_blob.item().ser_leaf(),
        SettlementLeaf::from(wrong_same_family),
    )
    .expect_err("same-family non-marker leaf must reject statically");
    assert_eq!(static_err, ProofChkErr::LeafMix);

    let other_marker = empty_marker_leaf(present_path);
    let tampered_item = z00z_storage::settlement::ProofItem::new_settlement(
        blob.item().settlement_root(),
        present_path,
        blob.item().def_leaf(),
        blob.item().ser_leaf(),
        SettlementLeaf::from(other_marker.clone()),
    )
    .expect("tampered path item");
    let tampered_blob =
        blob.clone()
            .rebind(tampered_item)
            .with_terminal_leaf_hash(settlement_leaf_hash(SettlementLeaf::from(
                other_marker.clone(),
            )));

    let static_err = chk_blob_settlement(
        &tampered_blob.encode().expect("encode blob"),
        root,
        &present_path,
        tampered_blob.item().def_leaf(),
        tampered_blob.item().ser_leaf(),
        SettlementLeaf::from(other_marker),
    )
    .expect_err("present-key nonexistence claim must reject statically");
    assert_eq!(static_err, ProofChkErr::TerminalProofMix);

    let wrong_policy = BucketPolicy::new(
        BucketPolicy::DEFAULT_BUCKET_BITS,
        BucketPolicy::DEFAULT_MIN_BUCKET_COUNT,
        BucketPolicy::DEFAULT_MAX_TARGET_LEAF_COUNT,
        BucketPolicy::DEFAULT_COMPATIBILITY_GENERATION + 1,
    )
    .expect("wrong bucket policy");
    let static_err = chk_blob_settlement(
        &blob
            .clone()
            .with_hjmt_bucket_policy(wrong_policy)
            .encode()
            .expect("encode blob"),
        root,
        &missing_path,
        blob.item().def_leaf(),
        blob.item().ser_leaf(),
        blob.item()
            .terminal_leaf()
            .expect("marker asset leaf")
            .clone(),
    )
    .expect_err("wrong bucket policy epoch must reject statically");
    assert!(matches!(
        static_err,
        ProofChkErr::BucketMix | ProofChkErr::BucketPolicyMix
    ));
}

#[test]
fn test_miss_blob_empty_tree() {
    let store = SettlementStore::new();
    let missing_path = SettlementPath::new(
        DefinitionId::new(bytes(91)),
        SerialId::new(7),
        TerminalId::new(bytes(3)),
    );
    let blob = store
        .settlement_nonexistence_proof_blob(&missing_path, SettlementLeafFamily::Terminal)
        .expect("empty-tree nonexistence proof blob");

    assert_eq!(
        blob.hjmt_proof_family(),
        Some(HjmtProofFamily::NonExistence)
    );
    assert_eq!(
        blob.hjmt_leaf_family(),
        Some(SettlementLeafFamily::Terminal)
    );
    store
        .validate_settlement_nonexistence_proof_blob(&blob, SettlementLeafFamily::Terminal)
        .expect("store-context empty-tree nonexistence validation");

    let checked = chk_blob_settlement(
        &blob.encode().expect("encode blob"),
        blob.item().settlement_root(),
        &missing_path,
        blob.item().def_leaf(),
        blob.item().ser_leaf(),
        empty_marker_leaf(missing_path),
    )
    .expect("static empty-tree nonexistence validation");
    assert_eq!(
        checked.hjmt_proof_family(),
        Some(HjmtProofFamily::NonExistence)
    );
}

#[test]
fn test_miss_blob_serial_lane() {
    let mut store = SettlementStore::new();
    let present_path = SettlementPath::new(
        DefinitionId::new(bytes(92)),
        SerialId::new(1),
        TerminalId::new(bytes(17)),
    );
    store
        .apply_settlement_ops(vec![StoreOp::Put(Box::new(item_for_path(present_path)))])
        .expect("seed settlement state");

    let missing_path = SettlementPath::new(
        present_path.definition_id,
        SerialId::new(2),
        TerminalId::new(bytes(18)),
    );
    let blob = store
        .settlement_nonexistence_proof_blob(&missing_path, SettlementLeafFamily::Terminal)
        .expect("missing-serial nonexistence proof blob");

    store
        .validate_settlement_nonexistence_proof_blob(&blob, SettlementLeafFamily::Terminal)
        .expect("store-context missing-serial nonexistence validation");

    let checked = chk_blob_settlement(
        &blob.encode().expect("encode blob"),
        blob.item().settlement_root(),
        &missing_path,
        blob.item().def_leaf(),
        blob.item().ser_leaf(),
        empty_marker_leaf(missing_path),
    )
    .expect("static missing-serial nonexistence validation");
    assert_eq!(
        checked.hjmt_proof_family(),
        Some(HjmtProofFamily::NonExistence)
    );
}

#[test]
fn test_miss_blob_right_leaffam() {
    let mut store = SettlementStore::new();
    let present_item = right_item(94);
    let present_path = present_item.path();
    store
        .apply_settlement_ops(vec![StoreOp::Put(Box::new(present_item))])
        .expect("seed right settlement state");

    let missing_path = sibling_path_same_bucket(&store, present_path, 95);
    let blob = store
        .settlement_nonexistence_proof_blob(&missing_path, SettlementLeafFamily::Right)
        .expect("right nonexistence proof blob");

    assert_eq!(
        blob.hjmt_proof_family(),
        Some(HjmtProofFamily::NonExistence)
    );
    assert_eq!(blob.hjmt_leaf_family(), Some(SettlementLeafFamily::Right));
    store
        .validate_settlement_nonexistence_proof_blob(&blob, SettlementLeafFamily::Right)
        .expect("store-context right nonexistence validation");

    let marker = SettlementLeafFamily::Right.marker_leaf(missing_path);
    let checked = chk_blob_settlement(
        &blob.encode().expect("encode blob"),
        blob.item().settlement_root(),
        &missing_path,
        blob.item().def_leaf(),
        blob.item().ser_leaf(),
        marker,
    )
    .expect("static right nonexistence validation");
    assert_eq!(
        checked.hjmt_proof_family(),
        Some(HjmtProofFamily::NonExistence)
    );
}

#[test]
fn test_miss_blob_voucher_leaffam() {
    let mut store = SettlementStore::new();
    let present_item = voucher_item(96);
    let present_path = present_item.path();
    store
        .apply_settlement_ops(vec![StoreOp::Put(Box::new(present_item))])
        .expect("seed voucher settlement state");

    let missing_path = sibling_path_same_bucket(&store, present_path, 97);
    let blob = store
        .settlement_nonexistence_proof_blob(&missing_path, SettlementLeafFamily::Voucher)
        .expect("voucher nonexistence proof blob");

    assert_eq!(
        blob.hjmt_proof_family(),
        Some(HjmtProofFamily::NonExistence)
    );
    assert_eq!(blob.hjmt_leaf_family(), Some(SettlementLeafFamily::Voucher));
    store
        .validate_settlement_nonexistence_proof_blob(&blob, SettlementLeafFamily::Voucher)
        .expect("store-context voucher nonexistence validation");

    let marker = SettlementLeafFamily::Voucher.marker_leaf(missing_path);
    let checked = chk_blob_settlement(
        &blob.encode().expect("encode blob"),
        blob.item().settlement_root(),
        &missing_path,
        blob.item().def_leaf(),
        blob.item().ser_leaf(),
        marker,
    )
    .expect("static voucher nonexistence validation");
    assert_eq!(
        checked.hjmt_proof_family(),
        Some(HjmtProofFamily::NonExistence)
    );
}

#[test]
fn test_miss_root_replay_rejects() {
    let mut store = SettlementStore::new();
    let seeded_path = SettlementPath::new(
        DefinitionId::new(bytes(87)),
        SerialId::new(1),
        TerminalId::new(bytes(30)),
    );
    store
        .apply_settlement_ops(vec![StoreOp::Put(Box::new(item_for_path(seeded_path)))])
        .expect("seed baseline settlement state");

    let missing_path = SettlementPath::new(
        DefinitionId::new(bytes(88)),
        SerialId::new(1),
        TerminalId::new(bytes(31)),
    );
    let stale_blob = store
        .settlement_nonexistence_proof_blob(&missing_path, SettlementLeafFamily::Terminal)
        .expect("stale nonexistence blob");

    let unrelated_path = SettlementPath::new(
        DefinitionId::new(bytes(89)),
        SerialId::new(1),
        TerminalId::new(bytes(32)),
    );
    store
        .apply_settlement_ops(vec![StoreOp::Put(Box::new(item_for_path(unrelated_path)))])
        .expect("advance store root and checkpoint");
    let current_blob = store
        .settlement_nonexistence_proof_blob(&missing_path, SettlementLeafFamily::Terminal)
        .expect("current nonexistence blob");

    let forged = ProofBlob::new_forest(
        current_blob.item().clone(),
        stale_blob.terminal_leaf_hash(),
        stale_blob.backend_root(),
        stale_blob.hjmt_bucket_policy().expect("bucket policy"),
        stale_blob
            .hjmt_bucket_root_leaf()
            .expect("bucket root leaf"),
        stale_blob.definition_proof().to_vec(),
        stale_blob.serial_proof().to_vec(),
        stale_blob
            .hjmt_bucket_proof()
            .expect("bucket proof")
            .to_vec(),
        stale_blob.terminal_proof().to_vec(),
        HjmtProofFamily::NonExistence,
        current_blob.hjmt_journal_checkpoint(),
        current_blob.hjmt_journal_digest(),
    )
    .with_hjmt_default_commitment(stale_blob.hjmt_default_commitment())
    .with_hjmt_leaf_family(SettlementLeafFamily::Terminal);

    let err = store
        .validate_settlement_nonexistence_proof_blob(&forged, SettlementLeafFamily::Terminal)
        .expect_err("rebound stale nonexistence proof must reject");
    assert!(matches!(
        err,
        SettlementStoreError::Proof(ProofChkErr::RootBindMix)
    ));
}

#[test]
fn test_inserted_path_rejects() {
    let mut store = SettlementStore::new();
    let present_path = SettlementPath::new(
        DefinitionId::new(bytes(90)),
        SerialId::new(1),
        TerminalId::new(bytes(33)),
    );
    store
        .apply_settlement_ops(vec![StoreOp::Put(Box::new(item_for_path(present_path)))])
        .expect("seed settlement state");

    let missing_path = sibling_path_same_bucket(&store, present_path, 91);
    let stale_blob = store
        .settlement_nonexistence_proof_blob(&missing_path, SettlementLeafFamily::Terminal)
        .expect("baseline nonexistence blob");

    store
        .apply_settlement_ops(vec![StoreOp::Put(Box::new(item_for_path(missing_path)))])
        .expect("insert formerly absent path");

    let err = store
        .validate_settlement_nonexistence_proof_blob(&stale_blob, SettlementLeafFamily::Terminal)
        .expect_err("stale nonexistence proof must reject after insert");
    assert!(matches!(
        err,
        SettlementStoreError::Proof(_) | SettlementStoreError::PathMiss
    ));

    let err = store
        .settlement_nonexistence_proof_blob(&missing_path, SettlementLeafFamily::Terminal)
        .expect_err("present path must not produce nonexistence proof after insert");
    assert!(matches!(err, SettlementStoreError::PathMiss));
}

#[test]
fn test_miss_path_tamper_rejects() {
    let mut store = SettlementStore::new();
    let present_path = SettlementPath::new(
        DefinitionId::new(bytes(90)),
        SerialId::new(1),
        TerminalId::new(bytes(33)),
    );
    let root = store
        .apply_settlement_ops(vec![StoreOp::Put(Box::new(item_for_path(present_path)))])
        .expect("seed baseline settlement state");

    let missing_path = sibling_path_same_bucket(&store, present_path, 34);
    let blob = store
        .settlement_nonexistence_proof_blob(&missing_path, SettlementLeafFamily::Terminal)
        .expect("baseline nonexistence blob");

    let tampered_path = (35..=u8::MAX)
        .map(|mark| {
            SettlementPath::new(
                present_path.definition_id,
                present_path.serial_id,
                TerminalId::new(bytes(mark)),
            )
        })
        .find(|candidate| {
            *candidate != present_path
                && *candidate != missing_path
                && store.bucket_policy().derive_bucket_id(*candidate)
                    != store.bucket_policy().derive_bucket_id(missing_path)
        })
        .expect("different-bucket alternate missing path");
    let marker_leaf = empty_marker_leaf(tampered_path);
    let tampered_item = z00z_storage::settlement::ProofItem::new_settlement(
        blob.item().settlement_root(),
        tampered_path,
        blob.item().def_leaf(),
        blob.item().ser_leaf(),
        SettlementLeaf::from(marker_leaf.clone()),
    )
    .expect("tampered missing-path item");
    let tampered_blob =
        blob.clone()
            .rebind(tampered_item)
            .with_terminal_leaf_hash(settlement_leaf_hash(SettlementLeaf::from(
                marker_leaf.clone(),
            )));

    let static_err = chk_blob_settlement(
        &tampered_blob.encode().expect("encode blob"),
        root,
        &tampered_path,
        tampered_blob.item().def_leaf(),
        tampered_blob.item().ser_leaf(),
        SettlementLeaf::from(marker_leaf.clone()),
    )
    .expect_err("tampered nonexistence path must reject statically");
    assert_eq!(static_err, ProofChkErr::BucketMix);

    let err = store
        .validate_settlement_nonexistence_proof_blob(&tampered_blob, SettlementLeafFamily::Terminal)
        .expect_err("tampered nonexistence path must reject");
    assert!(matches!(
        err,
        SettlementStoreError::Proof(ProofChkErr::BucketMix)
    ));
}

#[test]
fn test_incl_blob_right_leaf() {
    let mut store = SettlementStore::new();
    let right_item = right_item(61);
    let right_path = right_item.path();
    let right_leaf = *right_item.right_leaf().expect("right leaf fixture");
    let root = store
        .apply_settlement_ops(vec![StoreOp::Put(Box::new(right_item))])
        .expect("seed right settlement state");

    let blob = store
        .settlement_proof_blob(&right_path)
        .expect("right inclusion proof blob");

    assert_eq!(blob.hjmt_proof_family(), Some(HjmtProofFamily::Inclusion));
    assert_eq!(blob.hjmt_leaf_family(), Some(SettlementLeafFamily::Right));
    store
        .validate_settlement_proof_blob(&blob)
        .expect("store-context right inclusion validation");
    let err = store
        .validate_settlement_nonexistence_proof_blob(&blob, SettlementLeafFamily::Right)
        .expect_err("inclusion proof must not pass nonexistence validation");
    assert!(matches!(
        err,
        SettlementStoreError::Proof(ProofChkErr::ProofFamilyMix)
    ));

    let checked = chk_blob_settlement(
        &blob.encode().expect("encode blob"),
        root,
        &right_path,
        blob.item().def_leaf(),
        blob.item().ser_leaf(),
        right_leaf,
    )
    .expect("static right inclusion validation");
    assert_eq!(
        checked.hjmt_proof_family(),
        Some(HjmtProofFamily::Inclusion)
    );
    assert_eq!(
        checked.hjmt_leaf_family(),
        Some(SettlementLeafFamily::Right)
    );

    let legacy_like = ProofBlob::new(
        blob.item().clone(),
        blob.terminal_leaf_hash(),
        blob.backend_root(),
        blob.definition_proof().to_vec(),
        blob.serial_proof().to_vec(),
        blob.terminal_proof().to_vec(),
    );

    let inclusion_err = chk_blob_settlement_inclusion(
        &legacy_like.encode().expect("encode blob"),
        root,
        &right_path,
        legacy_like.item().def_leaf(),
        legacy_like.item().ser_leaf(),
        right_leaf,
    )
    .expect_err("membership verifier must reject envelope-less blob");
    assert_eq!(inclusion_err, ProofChkErr::ProofFamilyMix);

    let err = store
        .validate_settlement_proof_blob(&legacy_like)
        .expect_err("store validator must reject envelope-less blob");
    assert!(matches!(
        err,
        SettlementStoreError::Proof(ProofChkErr::ProofFamilyMix)
    ));
}

#[test]
fn test_incl_blob_voucher_leaf() {
    let mut store = SettlementStore::new();
    let voucher_item = voucher_item(63);
    let voucher_path = voucher_item.path();
    let voucher_leaf = voucher_item
        .voucher_leaf()
        .expect("voucher leaf fixture")
        .clone();
    let root = store
        .apply_settlement_ops(vec![StoreOp::Put(Box::new(voucher_item))])
        .expect("seed voucher settlement state");

    let blob = store
        .settlement_proof_blob(&voucher_path)
        .expect("voucher inclusion proof blob");

    assert_eq!(blob.hjmt_proof_family(), Some(HjmtProofFamily::Inclusion));
    assert_eq!(blob.hjmt_leaf_family(), Some(SettlementLeafFamily::Voucher));
    store
        .validate_settlement_proof_blob(&blob)
        .expect("store-context voucher inclusion validation");
    let err = store
        .validate_settlement_nonexistence_proof_blob(&blob, SettlementLeafFamily::Voucher)
        .expect_err("inclusion proof must not pass voucher nonexistence validation");
    assert!(matches!(
        err,
        SettlementStoreError::Proof(ProofChkErr::ProofFamilyMix)
    ));

    let checked = chk_blob_settlement(
        &blob.encode().expect("encode blob"),
        root,
        &voucher_path,
        blob.item().def_leaf(),
        blob.item().ser_leaf(),
        voucher_leaf.clone(),
    )
    .expect("static voucher inclusion validation");
    assert_eq!(
        checked.hjmt_proof_family(),
        Some(HjmtProofFamily::Inclusion)
    );
    assert_eq!(
        checked.hjmt_leaf_family(),
        Some(SettlementLeafFamily::Voucher)
    );

    let legacy_like = ProofBlob::new(
        blob.item().clone(),
        blob.terminal_leaf_hash(),
        blob.backend_root(),
        blob.definition_proof().to_vec(),
        blob.serial_proof().to_vec(),
        blob.terminal_proof().to_vec(),
    );

    let inclusion_err = chk_blob_settlement_inclusion(
        &legacy_like.encode().expect("encode blob"),
        root,
        &voucher_path,
        legacy_like.item().def_leaf(),
        legacy_like.item().ser_leaf(),
        voucher_leaf,
    )
    .expect_err("membership verifier must reject envelope-less voucher blob");
    assert_eq!(inclusion_err, ProofChkErr::ProofFamilyMix);

    let err = store
        .validate_settlement_proof_blob(&legacy_like)
        .expect_err("store validator must reject envelope-less voucher blob");
    assert!(matches!(
        err,
        SettlementStoreError::Proof(ProofChkErr::ProofFamilyMix)
    ));
}

#[test]
fn test_incl_blob_asset_leaf() {
    let mut store = SettlementStore::new();
    let asset_path = SettlementPath::new(
        DefinitionId::new(bytes(62)),
        SerialId::new(1),
        TerminalId::new(bytes(23)),
    );
    let asset_item = item_for_path(asset_path);
    let asset_leaf = asset_item
        .terminal_leaf()
        .expect("asset leaf fixture")
        .clone();
    let root = store
        .apply_settlement_ops(vec![StoreOp::Put(Box::new(asset_item))])
        .expect("seed asset settlement state");

    let blob = store
        .settlement_proof_blob(&asset_path)
        .expect("asset inclusion proof blob");

    assert_eq!(blob.hjmt_proof_family(), Some(HjmtProofFamily::Inclusion));
    assert_eq!(
        blob.hjmt_leaf_family(),
        Some(SettlementLeafFamily::Terminal)
    );
    store
        .validate_settlement_proof_blob(&blob)
        .expect("store-context asset inclusion validation");

    let checked = chk_blob_settlement_inclusion(
        &blob.encode().expect("encode blob"),
        root,
        &asset_path,
        blob.item().def_leaf(),
        blob.item().ser_leaf(),
        asset_leaf,
    )
    .expect("static asset inclusion validation");
    assert_eq!(
        checked.hjmt_proof_family(),
        Some(HjmtProofFamily::Inclusion)
    );
    assert_eq!(
        checked.hjmt_leaf_family(),
        Some(SettlementLeafFamily::Terminal)
    );
}

#[test]
fn test_incl_root_replay_rejects() {
    let mut store = SettlementStore::new();
    let target_path = SettlementPath::new(
        DefinitionId::new(bytes(82)),
        SerialId::new(1),
        TerminalId::new(bytes(25)),
    );
    let target_item = item_for_path(target_path);
    store
        .apply_settlement_ops(vec![StoreOp::Put(Box::new(target_item.clone()))])
        .expect("seed target inclusion path");
    let stale_blob = store
        .settlement_proof_blob(&target_path)
        .expect("stale inclusion blob");

    let unrelated_path = SettlementPath::new(
        DefinitionId::new(bytes(83)),
        SerialId::new(1),
        TerminalId::new(bytes(26)),
    );
    store
        .apply_settlement_ops(vec![StoreOp::Put(Box::new(item_for_path(unrelated_path)))])
        .expect("advance store root and checkpoint");
    let current_blob = store
        .settlement_proof_blob(&target_path)
        .expect("current inclusion blob");

    let forged = ProofBlob::new_forest(
        current_blob.item().clone(),
        stale_blob.terminal_leaf_hash(),
        stale_blob.backend_root(),
        stale_blob.hjmt_bucket_policy().expect("bucket policy"),
        stale_blob
            .hjmt_bucket_root_leaf()
            .expect("bucket root leaf"),
        stale_blob.definition_proof().to_vec(),
        stale_blob.serial_proof().to_vec(),
        stale_blob
            .hjmt_bucket_proof()
            .expect("bucket proof")
            .to_vec(),
        stale_blob.terminal_proof().to_vec(),
        HjmtProofFamily::Inclusion,
        current_blob.hjmt_journal_checkpoint(),
        current_blob.hjmt_journal_digest(),
    );

    let err = store
        .validate_settlement_proof_blob(&forged)
        .expect_err("rebound stale inclusion proof must reject");
    assert!(matches!(
        err,
        SettlementStoreError::Proof(ProofChkErr::RootBindMix)
    ));
}

#[test]
fn test_blob_binds_state_ckpt() {
    let mut store = SettlementStore::new();
    let deleted_path = SettlementPath::new(
        DefinitionId::new(bytes(52)),
        SerialId::new(1),
        TerminalId::new(bytes(11)),
    );
    let deleted_item = item_for_path(deleted_path);
    store
        .apply_settlement_ops(vec![StoreOp::Put(Box::new(deleted_item.clone()))])
        .expect("seed deleted item");

    let surviving_path = sibling_path_same_bucket(&store, deleted_path, 12);
    let surviving_item = item_for_path(surviving_path);
    store
        .apply_settlement_ops(vec![StoreOp::Put(Box::new(surviving_item))])
        .expect("seed surviving sibling");

    let root = store
        .apply_settlement_ops(vec![StoreOp::Delete(deleted_path)])
        .expect("delete target item");
    let blob = store
        .settlement_proof_blob(&deleted_path)
        .expect("deletion proof blob");

    assert_eq!(blob.hjmt_proof_family(), Some(HjmtProofFamily::Deletion));
    assert_eq!(blob.hjmt_journal_checkpoint(), Some(3));
    store
        .validate_settlement_proof_blob(&blob)
        .expect("store-context deletion validation");

    let checked = chk_blob_settlement(
        &blob.encode().expect("encode blob"),
        root,
        &deleted_path,
        blob.item().def_leaf(),
        blob.item().ser_leaf(),
        deleted_item
            .terminal_leaf()
            .expect("deleted asset leaf")
            .clone(),
    )
    .expect("static deletion validation");
    assert_eq!(checked.hjmt_proof_family(), Some(HjmtProofFamily::Deletion));

    let inclusion_err = chk_blob_settlement_inclusion(
        &blob.encode().expect("encode blob"),
        root,
        &deleted_path,
        blob.item().def_leaf(),
        blob.item().ser_leaf(),
        deleted_item
            .terminal_leaf()
            .expect("deleted asset leaf")
            .clone(),
    )
    .expect_err("membership verifier must reject deletion family");
    assert_eq!(inclusion_err, ProofChkErr::ProofFamilyMix);

    let scan_err = store
        .settlement_proof_scan(&deleted_path)
        .expect_err("scan surface must reject deletion family");
    assert!(matches!(
        scan_err,
        SettlementStoreError::Proof(ProofChkErr::ProofFamilyMix)
    ));

    let static_err = chk_blob_settlement(
        &blob
            .clone()
            .with_hjmt_journal_checkpoint(None)
            .encode()
            .expect("encode blob"),
        root,
        &deleted_path,
        blob.item().def_leaf(),
        blob.item().ser_leaf(),
        deleted_item
            .terminal_leaf()
            .expect("deleted asset leaf")
            .clone(),
    )
    .expect_err("missing journal checkpoint must reject statically");
    assert_eq!(static_err, ProofChkErr::JournalCheckpointMix);

    let static_err = chk_blob_settlement(
        &blob
            .clone()
            .with_hjmt_journal_checkpoint(Some(2))
            .encode()
            .expect("encode blob"),
        root,
        &deleted_path,
        blob.item().def_leaf(),
        blob.item().ser_leaf(),
        deleted_item
            .terminal_leaf()
            .expect("deleted asset leaf")
            .clone(),
    )
    .expect_err("stale journal checkpoint must reject statically");
    assert_eq!(static_err, ProofChkErr::JournalCheckpointMix);

    let err = store
        .validate_settlement_proof_blob(&blob.clone().with_hjmt_journal_checkpoint(Some(2)))
        .expect_err("stale journal checkpoint must reject");
    assert!(matches!(
        err,
        SettlementStoreError::Proof(ProofChkErr::JournalCheckpointMix)
    ));

    assert_tampered_prior_context_rejects(&store, &blob, deleted_path, &deleted_item);

    let mut wrong_old_leaf = deleted_item
        .terminal_leaf()
        .expect("deleted asset leaf")
        .clone();
    wrong_old_leaf.owner_tag[0] ^= 0x5a;
    let wrong_old = SettlementLeaf::from(wrong_old_leaf.clone());
    let wrong_old_item = z00z_storage::settlement::ProofItem::new_settlement(
        blob.item().settlement_root(),
        deleted_path,
        blob.item().def_leaf(),
        blob.item().ser_leaf(),
        wrong_old.clone(),
    )
    .expect("wrong old leaf item");
    let wrong_old_blob = blob
        .clone()
        .rebind(wrong_old_item)
        .with_terminal_leaf_hash(settlement_leaf_hash(wrong_old.clone()));
    let static_err = chk_blob_settlement(
        &wrong_old_blob.encode().expect("encode blob"),
        root,
        &deleted_path,
        wrong_old_blob.item().def_leaf(),
        wrong_old_blob.item().ser_leaf(),
        wrong_old,
    )
    .expect_err("wrong deleted leaf must reject against prior proof");
    assert_eq!(static_err, ProofChkErr::PriorTerminalProofMix);
}

#[test]
fn test_blob_last_leaf_xfer() {
    let mut store = SettlementStore::new();
    let deleted_path = SettlementPath::new(
        DefinitionId::new(bytes(93)),
        SerialId::new(1),
        TerminalId::new(bytes(19)),
    );
    let deleted_item = item_for_path(deleted_path);
    store
        .apply_settlement_ops(vec![StoreOp::Put(Box::new(deleted_item.clone()))])
        .expect("seed deleted item");

    let root = store
        .apply_settlement_ops(vec![StoreOp::Delete(deleted_path)])
        .expect("delete last live item");
    let blob = store
        .settlement_proof_blob(&deleted_path)
        .expect("last-leaf deletion proof blob");

    assert_eq!(blob.hjmt_proof_family(), Some(HjmtProofFamily::Deletion));
    store
        .validate_settlement_proof_blob(&blob)
        .expect("store-context last-leaf deletion validation");

    let checked = chk_blob_settlement(
        &blob.encode().expect("encode blob"),
        root,
        &deleted_path,
        blob.item().def_leaf(),
        blob.item().ser_leaf(),
        deleted_item
            .terminal_leaf()
            .expect("deleted asset leaf")
            .clone(),
    )
    .expect("static last-leaf deletion validation");
    assert_eq!(checked.hjmt_proof_family(), Some(HjmtProofFamily::Deletion));
}

#[test]
fn test_blob_right_leaf_xfer() {
    let mut store = SettlementStore::new();
    let deleted_item = right_item(74);
    let deleted_path = deleted_item.path();
    let deleted_leaf = *deleted_item.right_leaf().expect("deleted right leaf");
    store
        .apply_settlement_ops(vec![StoreOp::Put(Box::new(deleted_item))])
        .expect("seed deleted right");

    let root = store
        .apply_settlement_ops(vec![StoreOp::Delete(deleted_path)])
        .expect("delete right leaf");
    let blob = store
        .settlement_proof_blob(&deleted_path)
        .expect("right deletion proof blob");

    assert_eq!(blob.hjmt_proof_family(), Some(HjmtProofFamily::Deletion));
    assert_eq!(blob.hjmt_leaf_family(), Some(SettlementLeafFamily::Right));
    store
        .validate_settlement_proof_blob(&blob)
        .expect("store-context right deletion validation");

    let checked = chk_blob_settlement(
        &blob.encode().expect("encode blob"),
        root,
        &deleted_path,
        blob.item().def_leaf(),
        blob.item().ser_leaf(),
        SettlementLeaf::Right(deleted_leaf),
    )
    .expect("static right deletion validation");
    assert_eq!(checked.hjmt_proof_family(), Some(HjmtProofFamily::Deletion));
}

#[test]
fn test_blob_voucher_leaf_delete() {
    let mut store = SettlementStore::new();
    let deleted_item = voucher_item(76);
    let deleted_path = deleted_item.path();
    let deleted_leaf = deleted_item
        .voucher_leaf()
        .expect("deleted voucher leaf")
        .clone();
    store
        .apply_settlement_ops(vec![StoreOp::Put(Box::new(deleted_item))])
        .expect("seed deleted voucher");

    let root = store
        .apply_settlement_ops(vec![StoreOp::Delete(deleted_path)])
        .expect("delete voucher leaf");
    let blob = store
        .settlement_proof_blob(&deleted_path)
        .expect("voucher deletion proof blob");

    assert_eq!(blob.hjmt_proof_family(), Some(HjmtProofFamily::Deletion));
    assert_eq!(blob.hjmt_leaf_family(), Some(SettlementLeafFamily::Voucher));
    store
        .validate_settlement_proof_blob(&blob)
        .expect("store-context voucher deletion validation");

    let checked = chk_blob_settlement(
        &blob.encode().expect("encode blob"),
        root,
        &deleted_path,
        blob.item().def_leaf(),
        blob.item().ser_leaf(),
        SettlementLeaf::Voucher(deleted_leaf),
    )
    .expect("static voucher deletion validation");
    assert_eq!(checked.hjmt_proof_family(), Some(HjmtProofFamily::Deletion));
}

#[test]
fn test_bad_next_root_rejects() {
    let mut store = SettlementStore::new();
    let deleted_path = SettlementPath::new(
        DefinitionId::new(bytes(75)),
        SerialId::new(1),
        TerminalId::new(bytes(41)),
    );
    let deleted_item = item_for_path(deleted_path);
    store
        .apply_settlement_ops(vec![StoreOp::Put(Box::new(deleted_item.clone()))])
        .expect("seed deleted item");

    let surviving_path = sibling_path_same_bucket(&store, deleted_path, 12);
    store
        .apply_settlement_ops(vec![StoreOp::Put(Box::new(item_for_path(surviving_path)))])
        .expect("seed surviving sibling");
    let root = store
        .apply_settlement_ops(vec![StoreOp::Delete(deleted_path)])
        .expect("delete target item");
    let blob = store
        .settlement_proof_blob(&deleted_path)
        .expect("deletion proof blob");
    let mut wrong_root_bytes = root.into_bytes();
    wrong_root_bytes[0] ^= 0x5a;
    let wrong_root = z00z_storage::settlement::SettlementStateRoot::settlement_v1(wrong_root_bytes);

    let wrong_next_item = z00z_storage::settlement::ProofItem::new_settlement(
        wrong_root,
        deleted_path,
        blob.item().def_leaf(),
        blob.item().ser_leaf(),
        blob.item().leaf().clone(),
    )
    .expect("wrong next-root item");
    let wrong_next_blob = blob.clone().rebind(wrong_next_item);

    let static_err = chk_blob_settlement(
        &wrong_next_blob.encode().expect("encode blob"),
        root,
        &deleted_path,
        wrong_next_blob.item().def_leaf(),
        wrong_next_blob.item().ser_leaf(),
        deleted_item
            .terminal_leaf()
            .expect("deleted asset leaf")
            .clone(),
    )
    .expect_err("stale next root must reject statically");
    assert_eq!(static_err, ProofChkErr::RootGenerationMix);

    let err = store
        .validate_settlement_proof_blob(&wrong_next_blob)
        .expect_err("stale next root must reject");
    assert!(matches!(
        err,
        SettlementStoreError::Proof(ProofChkErr::RootGenerationMix)
    ));
}

#[test]
fn test_batch_records_size_time() {
    let mut store = SettlementStore::new();
    let included_path = SettlementPath::new(
        DefinitionId::new(bytes(77)),
        SerialId::new(1),
        TerminalId::new(bytes(33)),
    );
    let included_item = item_for_path(included_path);
    let root = store
        .apply_settlement_ops(vec![StoreOp::Put(Box::new(included_item.clone()))])
        .expect("seed included item");

    let missing_path = SettlementPath::new(
        included_path.definition_id,
        included_path.serial_id,
        TerminalId::new(bytes(34)),
    );
    let inclusion = store
        .settlement_proof_blob(&included_path)
        .expect("inclusion proof blob");
    let nonexistence = store
        .settlement_nonexistence_proof_blob(&missing_path, SettlementLeafFamily::Terminal)
        .expect("nonexistence proof blob");
    let inclusion_size = inclusion.encode().expect("encode inclusion").len();
    let nonexistence_size = nonexistence.encode().expect("encode nonexistence").len();
    assert!(inclusion_size > 0);
    assert!(nonexistence_size > 0);
    assert_ne!(
        inclusion.hjmt_proof_family(),
        nonexistence.hjmt_proof_family()
    );

    let started = Instant::now();
    store
        .validate_settlement_proof_blob(&inclusion)
        .expect("store-context inclusion validation");
    store
        .validate_settlement_nonexistence_proof_blob(&nonexistence, SettlementLeafFamily::Terminal)
        .expect("store-context nonexistence validation");
    let verify_time = started.elapsed();
    assert!(verify_time < Duration::from_secs(60));

    chk_blob_settlement_inclusion(
        &inclusion.encode().expect("encode inclusion"),
        root,
        &included_path,
        inclusion.item().def_leaf(),
        inclusion.item().ser_leaf(),
        included_item
            .terminal_leaf()
            .expect("included asset leaf")
            .clone(),
    )
    .expect("static inclusion validation");
    chk_blob_settlement(
        &nonexistence.encode().expect("encode nonexistence"),
        root,
        &missing_path,
        nonexistence.item().def_leaf(),
        nonexistence.item().ser_leaf(),
        empty_marker_leaf(missing_path),
    )
    .expect("static nonexistence validation");
}

#[test]
fn test_del_proof_size_time() {
    let mut store = SettlementStore::new();
    let deleted_path = SettlementPath::new(
        DefinitionId::new(bytes(52)),
        SerialId::new(1),
        TerminalId::new(bytes(11)),
    );
    let deleted_item = item_for_path(deleted_path);
    store
        .apply_settlement_ops(vec![StoreOp::Put(Box::new(deleted_item.clone()))])
        .expect("seed deleted item");
    let surviving_path = sibling_path_same_bucket(&store, deleted_path, 12);
    store
        .apply_settlement_ops(vec![StoreOp::Put(Box::new(item_for_path(surviving_path)))])
        .expect("seed surviving sibling");

    let root = store
        .apply_settlement_ops(vec![StoreOp::Delete(deleted_path)])
        .expect("delete target item");
    let blob = store
        .settlement_proof_blob(&deleted_path)
        .expect("deletion proof blob");

    let encoded = blob.encode().expect("encode deletion");
    assert!(!encoded.is_empty());
    assert_eq!(blob.hjmt_proof_family(), Some(HjmtProofFamily::Deletion));

    let started = Instant::now();
    store
        .validate_settlement_proof_blob(&blob)
        .expect("store-context deletion validation");
    let verify_time = started.elapsed();
    assert!(verify_time < Duration::from_secs(60));

    chk_blob_settlement(
        &encoded,
        root,
        &deleted_path,
        blob.item().def_leaf(),
        blob.item().ser_leaf(),
        deleted_item
            .terminal_leaf()
            .expect("deleted asset leaf")
            .clone(),
    )
    .expect("static deletion validation");
}

#[test]
fn test_definition_leaf_mismatch_rejects() {
    let mut store = SettlementStore::new();
    let deleted_path = SettlementPath::new(
        DefinitionId::new(bytes(53)),
        SerialId::new(1),
        TerminalId::new(bytes(12)),
    );
    let deleted_item = item_for_path(deleted_path);
    store
        .apply_settlement_ops(vec![StoreOp::Put(Box::new(deleted_item.clone()))])
        .expect("seed deleted item");
    let surviving_path = sibling_path_same_bucket(&store, deleted_path, 13);
    store
        .apply_settlement_ops(vec![StoreOp::Put(Box::new(item_for_path(surviving_path)))])
        .expect("seed surviving sibling");
    store
        .apply_settlement_ops(vec![StoreOp::Delete(deleted_path)])
        .expect("delete target item");
    let blob = store
        .settlement_proof_blob(&deleted_path)
        .expect("deletion proof blob");

    let tampered = rewrite_prior_blob(&blob, |prior| {
        prior.definition_root_leaf.definition_id = DefinitionId::new(bytes(200));
    });
    let err = store
        .validate_settlement_proof_blob(&tampered)
        .expect_err("tampered prior definition leaf must reject");
    assert!(matches!(
        err,
        SettlementStoreError::Proof(ProofChkErr::PriorDefMix)
    ));
}

#[test]
fn test_serial_leaf_mismatch_rejects() {
    let mut store = SettlementStore::new();
    let deleted_path = SettlementPath::new(
        DefinitionId::new(bytes(54)),
        SerialId::new(1),
        TerminalId::new(bytes(13)),
    );
    let deleted_item = item_for_path(deleted_path);
    store
        .apply_settlement_ops(vec![StoreOp::Put(Box::new(deleted_item.clone()))])
        .expect("seed deleted item");
    let surviving_path = sibling_path_same_bucket(&store, deleted_path, 14);
    store
        .apply_settlement_ops(vec![StoreOp::Put(Box::new(item_for_path(surviving_path)))])
        .expect("seed surviving sibling");
    store
        .apply_settlement_ops(vec![StoreOp::Delete(deleted_path)])
        .expect("delete target item");
    let blob = store
        .settlement_proof_blob(&deleted_path)
        .expect("deletion proof blob");

    let tampered = rewrite_prior_blob(&blob, |prior| {
        prior.serial_root_leaf.serial_id = SerialId::new(99);
    });
    let err = store
        .validate_settlement_proof_blob(&tampered)
        .expect_err("tampered prior serial leaf must reject");
    assert!(matches!(
        err,
        SettlementStoreError::Proof(ProofChkErr::PriorSerMix)
    ));
}

#[test]
fn test_bucket_policy_mismatch_rejects() {
    let mut store = SettlementStore::new();
    let deleted_path = SettlementPath::new(
        DefinitionId::new(bytes(55)),
        SerialId::new(1),
        TerminalId::new(bytes(14)),
    );
    let deleted_item = item_for_path(deleted_path);
    store
        .apply_settlement_ops(vec![StoreOp::Put(Box::new(deleted_item.clone()))])
        .expect("seed deleted item");
    let surviving_path = sibling_path_same_bucket(&store, deleted_path, 15);
    store
        .apply_settlement_ops(vec![StoreOp::Put(Box::new(item_for_path(surviving_path)))])
        .expect("seed surviving sibling");
    store
        .apply_settlement_ops(vec![StoreOp::Delete(deleted_path)])
        .expect("delete target item");
    let blob = store
        .settlement_proof_blob(&deleted_path)
        .expect("deletion proof blob");

    let tampered = rewrite_prior_blob(&blob, |prior| {
        prior.bucket_root_leaf.bucket_policy_id = bytes(201);
    });
    let err = store
        .validate_settlement_proof_blob(&tampered)
        .expect_err("tampered prior bucket policy must reject");
    assert!(matches!(
        err,
        SettlementStoreError::Proof(ProofChkErr::PriorBucketMix)
    ));
}

#[test]
fn test_family_roots_align() {
    let mut store = SettlementStore::new();
    let first = SettlementPath::new(
        DefinitionId::new(bytes(88)),
        SerialId::new(4),
        TerminalId::new(bytes(7)),
    );
    let paths = same_bucket_paths(&store, first, 8, 3);

    store
        .apply_settlement_ops(
            paths
                .iter()
                .map(|path| StoreOp::Put(Box::new(item_for_path(*path))))
                .collect(),
        )
        .expect("seed clustered inclusion paths");

    let batch = store
        .settlement_inclusion_batch_v1(&paths)
        .expect("live inclusion batch");
    let baseline = store
        .settlement_proof_blobs(&paths)
        .expect("independent proof baseline");

    assert_eq!(batch.path_table.len(), paths.len());
    assert_eq!(
        batch.header.settlement_root,
        baseline[0].item().settlement_root()
    );
    let total_refs: usize = batch
        .reference_table
        .iter()
        .map(|refs| refs.witness_indexes.len())
        .sum();
    assert!(batch.witness_dag.len() < total_refs);
    for blob in &baseline {
        store
            .validate_settlement_proof_blob(blob)
            .expect("baseline proof validation");
        assert_eq!(blob.item().settlement_root(), batch.header.settlement_root);
    }
}
