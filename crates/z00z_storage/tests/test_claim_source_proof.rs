use z00z_crypto::{ClaimProofVer, CLAIM_ROOT_VERSION};
use z00z_storage::settlement::{
    ClaimSourceRoot, DefinitionId, ProofBlob, RightClass, RightLeaf, SerialId,
    SettlementLeafFamily, SettlementPath, SettlementStateRoot, SettlementStore, StoreItem,
    TerminalId, TerminalLeaf,
};

fn test_item_with_path(mark: u32, serial_raw: u32, asset_raw: u8) -> StoreItem {
    let def_id = DefinitionId::new([0x11; 32]);
    let serial_id = SerialId::new(serial_raw);
    let asset_id = TerminalId::new([asset_raw; 32]);
    let path = SettlementPath::new(def_id, serial_id, asset_id);
    let mut leaf = TerminalLeaf::dummy_for_scan(mark);
    leaf.asset_id = asset_id.into_bytes();
    leaf.serial_id = serial_id.get();
    StoreItem::new(path, leaf).expect("claim source item")
}

fn test_item(mark: u32) -> StoreItem {
    test_item_with_path(mark, 7, 0x22)
}

fn test_right_item(mark: u8) -> StoreItem {
    let path = SettlementPath::new(
        DefinitionId::new([mark.wrapping_add(1); 32]),
        SerialId::new(u32::from(mark) + 7),
        TerminalId::new([mark; 32]),
    );
    let leaf = RightLeaf {
        version: 1,
        terminal_id: path.terminal_id,
        right_class: RightClass::MachineCapability,
        issuer_scope: [mark.wrapping_add(2); 32],
        provider_scope: [mark.wrapping_add(3); 32],
        holder_commitment: [mark.wrapping_add(4); 32],
        control_commitment: [mark.wrapping_add(5); 32],
        beneficiary_commitment: [mark.wrapping_add(6); 32],
        payload_commitment: [mark.wrapping_add(7); 32],
        valid_from: 10,
        valid_until: 20,
        challenge_from: 12,
        challenge_until: 18,
        use_nonce: [mark.wrapping_add(8); 32],
        revocation_policy_id: [mark.wrapping_add(9); 32],
        transition_policy_id: [mark.wrapping_add(10); 32],
        challenge_policy_id: [mark.wrapping_add(11); 32],
        disclosure_policy_id: [mark.wrapping_add(12); 32],
        retention_policy_id: [mark.wrapping_add(13); 32],
    };
    StoreItem::new(path, leaf).expect("right claim source item")
}

#[test]
fn test_claim_source_root_proof() {
    let mut store = SettlementStore::new();
    let item = test_item(41);

    store.put_settlement_item(item).expect("put item");

    let expected_root = store.claim_source_root().expect("claim root");
    let (claim_root, claim_proof) = store
        .claim_source_contract_for_item(&test_item(41))
        .expect("claim contract");
    let proof_blob = ProofBlob::decode(claim_proof.proof_blob()).expect("proof blob");

    assert_eq!(claim_root, expected_root);
    assert_eq!(claim_root.root_version(), CLAIM_ROOT_VERSION);
    assert_eq!(claim_proof.root_version(), CLAIM_ROOT_VERSION);
    assert_eq!(claim_proof.proof_ver(), ClaimProofVer::V1);
    assert_eq!(claim_proof.source_root(), claim_root.into_bytes());
    assert_eq!(
        proof_blob.item().root().into_bytes(),
        claim_root.into_bytes()
    );
    assert_eq!(proof_blob.root_bind_ver(), 1);
    assert_ne!(proof_blob.root_bind(), [0u8; 32]);
}

#[test]
fn test_claim_source_scan_summary() {
    let mut store = SettlementStore::new();
    let item = test_item(44);
    let path = item.path();

    store.put_settlement_item(item.clone()).expect("put item");

    let (_, claim_proof) = store
        .claim_source_contract_for_item(&item)
        .expect("claim contract");
    let scan = ProofBlob::decode(claim_proof.proof_blob()).expect("claim proof blob");
    let claim_root = store.claim_source_root().expect("claim root");
    let settlement_root = store.settlement_root().expect("settlement root");
    let proof_blob = store.settlement_proof_blob(&path).expect("proof blob");

    assert_eq!(claim_root.settlement_root(), settlement_root);
    assert_eq!(scan.item().root(), claim_root.settlement_root());
    assert_eq!(scan.item().settlement_root(), claim_root.settlement_root());
    assert_eq!(scan.item().path(), path);
    assert_eq!(scan.item().leaf(), proof_blob.item().leaf());
    assert_eq!(scan.terminal_leaf_hash(), proof_blob.terminal_leaf_hash());
    assert_eq!(scan.backend_root(), proof_blob.backend_root());
    assert_eq!(scan.root_bind_ver(), proof_blob.root_bind_ver());
    assert_eq!(scan.root_bind(), proof_blob.root_bind());
}

#[test]
fn test_claim_source_root_bytes() {
    let root = ClaimSourceRoot::new(
        CLAIM_ROOT_VERSION,
        SettlementStateRoot::settlement_v1([0xAB; 32]),
    );
    let settlement_root = ClaimSourceRoot::new_settlement(
        CLAIM_ROOT_VERSION,
        SettlementStateRoot::settlement_v1([0xCD; 32]),
    );

    assert_eq!(root.root_version(), CLAIM_ROOT_VERSION);
    assert_eq!(root.into_bytes(), [0xAB; 32]);
    assert_eq!(root.root(), SettlementStateRoot::settlement_v1([0xAB; 32]));
    assert_eq!(
        settlement_root.settlement_root(),
        SettlementStateRoot::settlement_v1([0xCD; 32])
    );
}

#[test]
fn test_contract_matches_store_roundtrip() {
    let item = test_item(59);
    let mut store = SettlementStore::new();
    store.put_settlement_item(item.clone()).expect("put item");

    let expected_root = store.claim_source_root().expect("claim root");
    let (claim_root, claim_proof) = store
        .claim_source_contract_for_item(&item)
        .expect("storage-backed contract");

    assert_eq!(claim_root, expected_root);
    assert_eq!(claim_proof.source_root(), expected_root.into_bytes());
}

#[test]
fn test_keeps_right_family() {
    let mut store = SettlementStore::new();
    let asset = test_item(73);
    let right = test_right_item(0x33);

    store.put_settlement_item(asset).expect("put asset");
    store
        .put_settlement_item(right.clone())
        .expect("put right item");

    let settlement_root = store.settlement_root().expect("settlement root");
    let (claim_root, claim_proof) = store
        .claim_source_contract_for_item(&right)
        .expect("right claim contract");
    let proof_blob = ProofBlob::decode(claim_proof.proof_blob()).expect("claim proof blob");

    assert_eq!(claim_root.settlement_root(), settlement_root);
    assert_eq!(claim_proof.source_root(), claim_root.into_bytes());
    assert_eq!(
        proof_blob.hjmt_leaf_family(),
        Some(SettlementLeafFamily::Right)
    );
    assert_eq!(proof_blob.item().path(), right.path());
    assert_eq!(proof_blob.item().leaf(), right.leaf());
    assert!(proof_blob.item().terminal_leaf().is_err());
    assert_eq!(
        proof_blob.item().right_leaf().expect("typed right leaf"),
        right.right_leaf().expect("store right leaf")
    );
}

#[test]
fn test_right_path_local_noise() {
    let right = test_right_item(0x44);

    let mut quiet_store = SettlementStore::new();
    quiet_store
        .put_settlement_item(right.clone())
        .expect("put quiet right");
    let (quiet_root, quiet_claim) = quiet_store
        .claim_source_contract_for_item(&right)
        .expect("quiet claim contract");
    let quiet_blob = ProofBlob::decode(quiet_claim.proof_blob()).expect("quiet proof blob");
    let quiet_scan = quiet_store
        .settlement_proof_scan(&right.path())
        .expect("quiet proof scan");

    let mut noisy_store = SettlementStore::new();
    noisy_store
        .put_settlement_item(right.clone())
        .expect("put noisy right");
    noisy_store
        .put_settlement_item(test_item_with_path(0x91, 91, 0x31))
        .expect("put noisy asset 1");
    noisy_store
        .put_settlement_item(test_item_with_path(0x92, 92, 0x32))
        .expect("put noisy asset 2");
    noisy_store
        .put_settlement_item(test_item_with_path(0x93, 93, 0x33))
        .expect("put noisy asset 3");
    let (noisy_root, noisy_claim) = noisy_store
        .claim_source_contract_for_item(&right)
        .expect("noisy claim contract");
    let noisy_blob = ProofBlob::decode(noisy_claim.proof_blob()).expect("noisy proof blob");
    let noisy_scan = noisy_store
        .settlement_proof_scan(&right.path())
        .expect("noisy proof scan");

    assert_ne!(quiet_root, noisy_root);

    for proof_blob in [&quiet_blob, &noisy_blob] {
        assert_eq!(
            proof_blob.hjmt_leaf_family(),
            Some(SettlementLeafFamily::Right)
        );
        assert_eq!(proof_blob.item().path(), right.path());
        assert_eq!(proof_blob.item().leaf(), right.leaf());
        assert!(proof_blob.item().terminal_leaf().is_err());
    }

    assert_eq!(quiet_blob.item().def_leaf(), noisy_blob.item().def_leaf());
    assert_eq!(quiet_blob.item().ser_leaf(), noisy_blob.item().ser_leaf());
    assert_eq!(
        quiet_blob.terminal_leaf_hash(),
        noisy_blob.terminal_leaf_hash()
    );
    assert_eq!(quiet_scan.path(), noisy_scan.path());
    assert_eq!(quiet_scan.leaf(), noisy_scan.leaf());
    assert_eq!(quiet_scan.def_leaf(), noisy_scan.def_leaf());
    assert_eq!(quiet_scan.ser_leaf(), noisy_scan.ser_leaf());
    assert!(quiet_scan.terminal_leaf().is_err());
    assert!(noisy_scan.terminal_leaf().is_err());
}

#[test]
fn test_rejects_synthetic_item_authority() {
    let item = test_item_with_path(81, 7, 0x22);
    let sibling = test_item_with_path(82, 8, 0x23);

    let mut persisted_store = SettlementStore::new();
    persisted_store
        .put_settlement_item(item.clone())
        .expect("put item");
    persisted_store
        .put_settlement_item(sibling)
        .expect("put sibling");

    let (persisted_root, persisted_proof) = persisted_store
        .claim_source_contract_for_item(&item)
        .expect("persisted contract");

    let mut synthetic_store = SettlementStore::new();
    synthetic_store
        .put_settlement_item(item.clone())
        .expect("put synthetic item");
    let (synthetic_root, synthetic_proof) = synthetic_store
        .claim_source_contract_for_item(&item)
        .expect("synthetic contract");

    assert_ne!(persisted_root, synthetic_root);
    assert_ne!(persisted_proof, synthetic_proof);
}

#[test]
fn test_contract_rejects_missing_membership() {
    let store = SettlementStore::new();
    let err = store
        .claim_source_contract_for_item(&test_item(61))
        .expect_err("missing membership must fail closed");

    assert_eq!(err.to_string(), "settlement path is missing");
}

#[test]
fn test_rejects_stale_item_drift() {
    let mut store = SettlementStore::new();
    let live_item = test_item(71);
    let stale_item = test_item(72);
    store.put_settlement_item(live_item).expect("put item");

    let err = store
        .claim_source_contract_for_item(&stale_item)
        .expect_err("drifted item must fail closed");

    assert_eq!(err.to_string(), "settlement path is missing");
}
