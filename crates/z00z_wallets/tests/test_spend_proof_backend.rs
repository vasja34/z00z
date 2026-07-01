#[path = "test_inc/test_spend_proof_support.inc"]
mod test_spend_proof_support;

use z00z_core::AssetClass;
use z00z_crypto::{domains::AssetIdDomain, hash_zk::hash_zk};
use z00z_storage::settlement::{
    DefinitionId, RightClass, RightLeaf, SerialId, SettlementPath, SettlementStateRoot,
    SettlementStore, StoreItem, TerminalId,
};
use z00z_wallets::tx::{
    asset_wire_to_leaf, audit_asset_class_outcome, audit_asset_class_total,
    default_spend_proof_backend, AssetClassAuditEntry, AssetClassAuditErr,
    AssetClassAuditMismatchClass, AssetClassAuditStatus, AssetClassAuditTarget,
    SpendMembershipWitness, SpendProofArtifact, SpendProofBackend, SpendProofBackendError,
    SpendProofStmt,
};
use z00z_wallets::{
    key::{derive_owner_handle, derive_view_secret_key, ReceiverSecret},
    stealth::ecdh::{compute_dh_receiver, decode_r_pub},
    stealth::kdf::{compute_owner_tag, derive_k_dh},
};

fn forged_artifact(stmt: &SpendProofStmt) -> SpendProofArtifact {
    let statement_hash = *blake3::hash(&stmt.statement).as_bytes();
    let mut public_hash_bytes = Vec::with_capacity(b"z00z.spend.public.hash.v1".len() + 32);
    public_hash_bytes.extend_from_slice(b"z00z.spend.public.hash.v1");
    public_hash_bytes.extend_from_slice(&statement_hash);
    let public_hash = *blake3::hash(&public_hash_bytes).as_bytes();

    let suite_id = b"regular_spend_theorem_bpplus";
    let mut theorem_preimage =
        Vec::with_capacity(b"z00z.spend.theorem.proof.v1".len() + suite_id.len() + 64);
    theorem_preimage.extend_from_slice(b"z00z.spend.theorem.proof.v1");
    theorem_preimage.extend_from_slice(suite_id);
    theorem_preimage.extend_from_slice(&statement_hash);
    theorem_preimage.extend_from_slice(&public_hash);
    let theorem_bytes = blake3::hash(&theorem_preimage).as_bytes().to_vec();

    let mut proof_bytes = Vec::new();
    proof_bytes.extend_from_slice(b"z00z.spend.proof.backend.v2");
    proof_bytes.push(suite_id.len() as u8);
    proof_bytes.extend_from_slice(suite_id);
    proof_bytes.extend_from_slice(&statement_hash);
    proof_bytes.extend_from_slice(&public_hash);
    proof_bytes.extend_from_slice(&(theorem_bytes.len() as u32).to_le_bytes());
    proof_bytes.extend_from_slice(&theorem_bytes);

    SpendProofArtifact {
        proof_hex: hex::encode(proof_bytes),
        pub_hash_hex: hex::encode(public_hash),
    }
}

fn audit_entry() -> (AssetClassAuditEntry, SpendProofStmt) {
    let (tx, _) = test_spend_proof_support::canonical_public_contract_tx();
    let output = tx.outputs[0].clone();
    let wire = output.asset_wire.clone().to_wire().expect("output wire");
    let mut leaf = asset_wire_to_leaf(&wire).expect("output leaf");
    leaf.asset_id = output.asset_wire.leaf_ad_id.expect("leaf ad id");

    let path = SettlementPath::new(
        DefinitionId::new(output.asset_wire.definition.id),
        SerialId::new(output.asset_wire.serial_id),
        TerminalId::new(leaf.asset_id),
    );
    let mut store = SettlementStore::new();
    let item = StoreItem::new(path, leaf.clone()).expect("store item");
    store.put_settlement_item(item).expect("put item");
    let scan = store.settlement_proof_scan(&path).expect("proof scan");

    (
        AssetClassAuditEntry { scan, output },
        test_spend_proof_support::canonical_proof_stmt(),
    )
}

fn canonical_audit_entry() -> (AssetClassAuditEntry, SpendProofStmt) {
    audit_entry()
}

fn bytes(mark: u8) -> [u8; 32] {
    [mark; 32]
}

fn right_path(mark: u8) -> SettlementPath {
    SettlementPath::new(
        DefinitionId::new(bytes(mark.wrapping_add(1))),
        SerialId::new(u32::from(mark) + 1),
        TerminalId::new(bytes(mark)),
    )
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

#[test]
fn test_backend_roundtrip() {
    let (tx, _) = test_spend_proof_support::canonical_public_contract_tx();
    let spend = tx.proof.spend.as_ref().expect("spend proof");
    let stmt = test_spend_proof_support::canonical_proof_stmt();
    let backend = default_spend_proof_backend();

    assert_eq!(spend.proof_suite, backend.suite_id());

    let artifact = backend
        .prove(&stmt, &test_spend_proof_support::canonical_proof_witness())
        .expect("prove artifact");

    assert_eq!(artifact.proof_hex, spend.proof_hex);

    backend.verify(&stmt, &artifact).expect("verify artifact");
}

#[test]
fn test_audit_accepts_canonical_entry() {
    let (entry, stmt) = canonical_audit_entry();
    let report = audit_asset_class_total(
        AssetClass::Coin,
        entry.scan.settlement_root(),
        std::slice::from_ref(&entry),
        Some(&stmt),
        Some(&entry.output.asset_wire.commitment),
    )
    .expect("canonical audit");

    assert_eq!(report.asset_class, AssetClass::Coin);
    assert_eq!(report.semantic_root, entry.scan.settlement_root());
    assert_eq!(report.leaf_count, 1);
    assert_eq!(report.total_commitment, entry.output.asset_wire.commitment);
    assert_eq!(report.mismatch_class, None);
    assert_eq!(
        report.target,
        AssetClassAuditTarget::ExpectedTotalCommitment {
            expected_total: entry.output.asset_wire.commitment
        }
    );
}

#[test]
fn test_hjmt_audit_scan() {
    let (entry, stmt) = audit_entry();
    let report = audit_asset_class_total(
        AssetClass::Coin,
        entry.scan.settlement_root(),
        std::slice::from_ref(&entry),
        Some(&stmt),
        Some(&entry.output.asset_wire.commitment),
    )
    .expect("hjmt audit");

    assert_eq!(report.semantic_root, entry.scan.settlement_root());
    assert_eq!(report.backend_root, entry.scan.backend_root());
    assert_eq!(report.root_bind, entry.scan.root_bind());
    assert_eq!(report.leaf_count, 1);

    let mut tampered_entry = entry.clone();
    tampered_entry.scan = tampered_entry.scan.with_root_bind(1, [0x42; 32]);
    let err = audit_asset_class_total(
        AssetClass::Coin,
        entry.scan.settlement_root(),
        std::slice::from_ref(&tampered_entry),
        Some(&stmt),
        Some(&tampered_entry.output.asset_wire.commitment),
    )
    .expect_err("hjmt root-bind drift must reject");

    assert_eq!(err, AssetClassAuditErr::RootBind { entry_index: 0 });
}

#[test]
fn test_audit_outcome_passes_target() {
    let (entry, stmt) = canonical_audit_entry();
    let outcome = audit_asset_class_outcome(
        AssetClass::Coin,
        entry.scan.settlement_root(),
        std::slice::from_ref(&entry),
        Some(&stmt),
        AssetClassAuditTarget::CheckpointEquation {
            expected_total: entry.output.asset_wire.commitment.clone(),
            checkpoint_id: "checkpoint-043-11".to_string(),
        },
    );

    assert_eq!(outcome.status, AssetClassAuditStatus::Pass);
    assert_eq!(outcome.mismatch_class, None);
    assert_eq!(outcome.entry_index, None);
    assert_eq!(outcome.report.asset_class, AssetClass::Coin);
    assert_eq!(outcome.report.mismatch_class, None);
    assert_eq!(
        outcome.report.target,
        AssetClassAuditTarget::CheckpointEquation {
            expected_total: entry.output.asset_wire.commitment,
            checkpoint_id: "checkpoint-043-11".to_string()
        }
    );
}

#[test]
fn test_audit_outcome_fails_typed() {
    let (entry, stmt) = canonical_audit_entry();
    let wrong_total = &entry.output.asset_wire.commitment + &entry.output.asset_wire.commitment;
    let outcome = audit_asset_class_outcome(
        AssetClass::Coin,
        entry.scan.settlement_root(),
        std::slice::from_ref(&entry),
        Some(&stmt),
        AssetClassAuditTarget::ExpectedTotalCommitment {
            expected_total: wrong_total,
        },
    );

    assert_eq!(outcome.status, AssetClassAuditStatus::FailClosed);
    assert_eq!(
        outcome.mismatch_class,
        Some(AssetClassAuditMismatchClass::TargetMismatch)
    );
    assert_eq!(outcome.entry_index, Some(0));
    assert_eq!(
        outcome.report.mismatch_class,
        Some(AssetClassAuditMismatchClass::TargetMismatch)
    );
}

#[test]
fn test_audit_rejects_wrong_root() {
    let (entry, stmt) = canonical_audit_entry();
    let err = audit_asset_class_total(
        AssetClass::Coin,
        SettlementStateRoot::settlement_v1([0xEE; 32]),
        std::slice::from_ref(&entry),
        Some(&stmt),
        Some(&entry.output.asset_wire.commitment),
    )
    .expect_err("wrong semantic root must reject");

    assert_eq!(err, AssetClassAuditErr::Membership { entry_index: 0 });
}

#[test]
fn test_audit_rejects_missing_spend() {
    let (entry, _) = canonical_audit_entry();
    let err = audit_asset_class_total(
        AssetClass::Coin,
        entry.scan.settlement_root(),
        std::slice::from_ref(&entry),
        None,
        Some(&entry.output.asset_wire.commitment),
    )
    .expect_err("missing spend proof must reject");

    assert_eq!(err, AssetClassAuditErr::MissingEvidence("spend proof"));
}

#[test]
fn test_audit_rejects_missing_total() {
    let (entry, stmt) = canonical_audit_entry();
    let err = audit_asset_class_total(
        AssetClass::Coin,
        entry.scan.settlement_root(),
        std::slice::from_ref(&entry),
        Some(&stmt),
        None,
    )
    .expect_err("missing total commitment must reject");

    assert_eq!(
        err,
        AssetClassAuditErr::MissingEvidence("expected total commitment")
    );
}

#[test]
fn test_audit_rejects_total_drift() {
    let (entry, stmt) = canonical_audit_entry();
    let wrong_total = &entry.output.asset_wire.commitment + &entry.output.asset_wire.commitment;
    let err = audit_asset_class_total(
        AssetClass::Coin,
        entry.scan.settlement_root(),
        std::slice::from_ref(&entry),
        Some(&stmt),
        Some(&wrong_total),
    )
    .expect_err("wrong commitment total must reject");

    assert_eq!(err, AssetClassAuditErr::Target { entry_index: 0 });
}

#[test]
fn test_audit_rejects_empty_entries() {
    let (entry, stmt) = canonical_audit_entry();
    let err = audit_asset_class_total(
        AssetClass::Coin,
        SettlementStateRoot::settlement_v1([0xAA; 32]),
        &[],
        Some(&stmt),
        Some(&entry.output.asset_wire.commitment),
    )
    .expect_err("empty entry list must reject");

    assert_eq!(err, AssetClassAuditErr::MissingEvidence("asset entries"));
}

#[test]
fn test_audit_root_bind_drift() {
    let (entry, stmt) = canonical_audit_entry();
    let mut tampered_entry = entry.clone();
    tampered_entry.scan = tampered_entry.scan.clone().with_root_bind(1, [0x42; 32]);

    let err = audit_asset_class_total(
        AssetClass::Coin,
        entry.scan.settlement_root(),
        std::slice::from_ref(&tampered_entry),
        Some(&stmt),
        Some(&tampered_entry.output.asset_wire.commitment),
    )
    .expect_err("tampered root bind must reject");

    assert_eq!(err, AssetClassAuditErr::RootBind { entry_index: 0 });
}

#[test]
fn test_audit_rejects_leaf_mismatch() {
    let (entry, stmt) = canonical_audit_entry();
    let mut tampered_entry = entry.clone();
    tampered_entry.output.asset_wire.owner_tag = Some([0x41; 32]);

    let err = audit_asset_class_total(
        AssetClass::Coin,
        entry.scan.settlement_root(),
        std::slice::from_ref(&tampered_entry),
        Some(&stmt),
        Some(&tampered_entry.output.asset_wire.commitment),
    )
    .expect_err("tampered output leaf must reject");

    assert_eq!(err, AssetClassAuditErr::Leaf { entry_index: 0 });
}

#[test]
fn test_audit_leaf_hash_mismatch() {
    let (entry, stmt) = canonical_audit_entry();
    let mut tampered_entry = entry.clone();
    let mut wrong_leaf_hash = tampered_entry.scan.terminal_leaf_hash();
    wrong_leaf_hash[0] ^= 0x01;
    tampered_entry.scan = tampered_entry.scan.with_terminal_leaf_hash(wrong_leaf_hash);

    let err = audit_asset_class_total(
        AssetClass::Coin,
        entry.scan.settlement_root(),
        std::slice::from_ref(&tampered_entry),
        Some(&stmt),
        Some(&tampered_entry.output.asset_wire.commitment),
    )
    .expect_err("tampered leaf hash must reject");

    assert_eq!(err, AssetClassAuditErr::Hash { entry_index: 0 });
    assert_eq!(
        err.mismatch_class(),
        AssetClassAuditMismatchClass::HashMismatch
    );
}

#[test]
fn test_audit_spend_proof_mismatch() {
    let (entry, mut stmt) = canonical_audit_entry();
    stmt.output_leaves[0].owner_tag = [0x41; 32];

    let err = audit_asset_class_total(
        AssetClass::Coin,
        entry.scan.settlement_root(),
        std::slice::from_ref(&entry),
        Some(&stmt),
        Some(&entry.output.asset_wire.commitment),
    )
    .expect_err("tampered spend proof must reject");

    assert_eq!(err, AssetClassAuditErr::SpendProof { entry_index: 0 });
}

#[test]
fn test_audit_rejects_duplicate_entry() {
    let (entry, mut stmt) = canonical_audit_entry();
    stmt.output_leaves.push(stmt.output_leaves[0].clone());
    let entries = vec![entry.clone(), entry.clone()];
    let expected_total = &entry.output.asset_wire.commitment + &entry.output.asset_wire.commitment;
    let err = audit_asset_class_total(
        AssetClass::Coin,
        entry.scan.settlement_root(),
        &entries,
        Some(&stmt),
        Some(&expected_total),
    )
    .expect_err("duplicate audit entry must reject");

    assert_eq!(err, AssetClassAuditErr::DuplicateEntry { entry_index: 1 });
    assert_eq!(
        err.mismatch_class(),
        AssetClassAuditMismatchClass::DuplicateEntry
    );
    assert_eq!(err.entry_index(), Some(1));
}

#[test]
fn test_audit_asset_class_mismatch() {
    let (entry, stmt) = canonical_audit_entry();
    let mut tampered_entry = entry.clone();
    tampered_entry.output.asset_wire.definition.class = AssetClass::Token;

    let err = audit_asset_class_total(
        AssetClass::Coin,
        entry.scan.settlement_root(),
        std::slice::from_ref(&tampered_entry),
        Some(&stmt),
        Some(&tampered_entry.output.asset_wire.commitment),
    )
    .expect_err("wrong asset class must reject");

    assert_eq!(err, AssetClassAuditErr::AssetClass { entry_index: 0 });
}

#[test]
fn test_backend_decodes_wire_artifact() {
    let (tx, _) = test_spend_proof_support::canonical_public_contract_tx();
    let spend = tx.proof.spend.as_ref().expect("spend proof");
    let artifact = SpendProofArtifact::from_wire_hex(&spend.proof_hex).expect("decode artifact");

    assert!(!artifact.pub_hash_hex.is_empty());
}

#[test]
fn test_backend_rejects_statement_drift() {
    let (tx, _) = test_spend_proof_support::canonical_public_contract_tx();
    let spend = tx.proof.spend.as_ref().expect("spend proof");
    let mut stmt = test_spend_proof_support::canonical_proof_stmt();
    let mut statement = stmt.statement.clone();
    let last = statement.last_mut().expect("statement bytes");
    *last ^= 0x01;
    stmt.statement = statement;
    let artifact = SpendProofArtifact::from_wire_hex(&spend.proof_hex).expect("decode artifact");
    let backend = default_spend_proof_backend();

    let err = backend
        .verify(&stmt, &artifact)
        .expect_err("drifted statement must reject");

    assert_eq!(err, SpendProofBackendError::StatementMismatch);
}

#[test]
fn test_backend_theorem_payload_drift() {
    let (tx, _) = test_spend_proof_support::canonical_public_contract_tx();
    let spend = tx.proof.spend.as_ref().expect("spend proof");
    let mut artifact =
        SpendProofArtifact::from_wire_hex(&spend.proof_hex).expect("decode artifact");
    let mut proof_blob = hex::decode(&artifact.proof_hex).expect("proof hex");
    let last = proof_blob.last_mut().expect("theorem payload byte");
    *last ^= 0x01;
    artifact.proof_hex = hex::encode(proof_blob);
    let stmt = test_spend_proof_support::canonical_proof_stmt();
    let backend = default_spend_proof_backend();

    let err = backend
        .verify(&stmt, &artifact)
        .expect_err("tampered theorem payload must reject");

    assert_eq!(err, SpendProofBackendError::TheoremRelationMismatch);
}

#[test]
fn test_backend_rejects_forged_range() {
    let mut stmt = test_spend_proof_support::canonical_proof_stmt();
    stmt.output_leaves[0].range_proof.clear();
    let artifact = forged_artifact(&stmt);
    let backend = default_spend_proof_backend();

    let err = backend
        .verify(&stmt, &artifact)
        .expect_err("backend verify must reject a forged public range relation");

    assert_eq!(err, SpendProofBackendError::RangeRelationMismatch);
}

#[test]
fn test_backend_rejects_forged_balance() {
    let mut stmt = test_spend_proof_support::canonical_proof_stmt();
    stmt.output_leaves.push(stmt.output_leaves[0].clone());
    let artifact = forged_artifact(&stmt);
    let backend = default_spend_proof_backend();

    let err = backend
        .verify(&stmt, &artifact)
        .expect_err("backend verify must reject a forged public balance relation");

    assert_eq!(err, SpendProofBackendError::TheoremRelationMismatch);
}

#[test]
fn test_backend_rejects_forged_overlap() {
    let mut stmt = test_spend_proof_support::canonical_proof_stmt();
    stmt.output_leaves[0].asset_id = stmt.input_leaves[0].leaf_ad_id;
    let artifact = forged_artifact(&stmt);
    let backend = default_spend_proof_backend();

    let err = backend
        .verify(&stmt, &artifact)
        .expect_err("backend verify must reject forged input/output leaf overlap");

    assert_eq!(err, SpendProofBackendError::TheoremRelationMismatch);
}

#[test]
fn test_backend_rejects_bad_payload() {
    let err = SpendProofArtifact::from_wire_hex(&hex::encode(b"bad-proof-payload"))
        .expect_err("malformed payload must reject");

    assert_eq!(err, SpendProofBackendError::InvalidProofPayload);
}

#[test]
fn test_backend_rejects_empty_witness() {
    let stmt = test_spend_proof_support::canonical_proof_stmt();
    let backend = default_spend_proof_backend();
    let mut witness = test_spend_proof_support::canonical_proof_witness();
    witness.input_s_in.clear();

    let err = backend
        .prove(&stmt, &witness)
        .expect_err("empty witness must reject");

    assert_eq!(err, SpendProofBackendError::EmptyWitness);
}

#[test]
fn test_backend_rejects_bad_witness() {
    let stmt = test_spend_proof_support::canonical_proof_stmt();
    let backend = default_spend_proof_backend();
    let mut unrelated_witness = test_spend_proof_support::canonical_proof_witness();
    unrelated_witness.receiver_secret =
        ReceiverSecret::from_bytes([0x22; 32]).expect("receiver secret");
    unrelated_witness.input_s_in = vec![[0x33; 32]];
    let err = backend
        .prove(&stmt, &unrelated_witness)
        .expect_err("unrelated witness must reject");

    assert_eq!(
        err,
        SpendProofBackendError::WitnessRelationMismatch,
        "backend proof generation must reject witnesses that do not satisfy theorem public inputs"
    );
}

#[test]
fn test_backend_statement_only_stmt() {
    let stmt = SpendProofStmt::new(vec![0xA5]).expect("statement-only stmt");
    let witness = test_spend_proof_support::canonical_proof_witness();
    let backend = default_spend_proof_backend();

    let err = backend
        .prove(&stmt, &witness)
        .expect_err("statement-only typed statement must reject");

    assert_eq!(err, SpendProofBackendError::StatementShapeMismatch);
}

#[test]
fn test_backend_prev_root_tamper() {
    let mut stmt = test_spend_proof_support::canonical_proof_stmt();
    let mut root = stmt.prev_root.into_bytes();
    root[0] ^= 0x01;
    stmt.prev_root = z00z_storage::settlement::CheckRoot::new(root);
    let witness = test_spend_proof_support::canonical_proof_witness();
    let backend = default_spend_proof_backend();

    let err = backend
        .prove(&stmt, &witness)
        .expect_err("prev_root tamper must reject");

    assert_eq!(err, SpendProofBackendError::MembershipWitnessMismatch);
}

#[test]
fn test_backend_membership_proof_tamper() {
    let stmt = test_spend_proof_support::canonical_proof_stmt();
    let mut witness = test_spend_proof_support::canonical_proof_witness();
    witness.membership[0].proof[0] ^= 0x01;
    let backend = default_spend_proof_backend();

    let err = backend
        .prove(&stmt, &witness)
        .expect_err("membership proof tamper must reject");

    assert_eq!(err, SpendProofBackendError::MembershipWitnessMismatch);
}

#[test]
fn test_backend_membership_path_tamper() {
    let stmt = test_spend_proof_support::canonical_proof_stmt();
    let mut witness = test_spend_proof_support::canonical_proof_witness();
    let path = witness.membership[0].path;
    witness.membership[0].path = SettlementPath::new(
        path.definition_id,
        z00z_storage::settlement::SerialId::new(path.serial_id.get() + 1),
        path.terminal_id(),
    );
    let backend = default_spend_proof_backend();

    let err = backend
        .prove(&stmt, &witness)
        .expect_err("membership path tamper must reject");

    assert_eq!(err, SpendProofBackendError::MembershipWitnessMismatch);
}

#[test]
fn test_ctor_rejects_path_drift() {
    let witness = test_spend_proof_support::canonical_proof_witness()
        .membership
        .into_iter()
        .next()
        .expect("membership");
    let path = witness.path;
    let err = SpendMembershipWitness::new(
        SettlementPath::new(
            path.definition_id,
            z00z_storage::settlement::SerialId::new(path.serial_id.get() + 1),
            path.terminal_id(),
        ),
        witness.leaf,
        witness.proof,
        witness.proof_item,
    )
    .expect_err("constructor must reject path drift before backend validation");

    assert_eq!(err, SpendProofBackendError::MembershipWitnessMismatch);
}

#[test]
fn test_backend_membership_leaf_tamper() {
    let stmt = test_spend_proof_support::canonical_proof_stmt();
    let mut witness = test_spend_proof_support::canonical_proof_witness();
    witness.membership[0].leaf.owner_tag[0] ^= 0x01;
    let backend = default_spend_proof_backend();

    let err = backend
        .prove(&stmt, &witness)
        .expect_err("membership leaf tamper must reject");

    assert_eq!(err, SpendProofBackendError::MembershipWitnessMismatch);
}

#[test]
fn test_ctor_rejects_leaf_drift() {
    let witness = test_spend_proof_support::canonical_proof_witness()
        .membership
        .into_iter()
        .next()
        .expect("membership");
    let mut leaf = witness.leaf.clone();
    leaf.owner_tag[0] ^= 0x01;
    let err = SpendMembershipWitness::new(witness.path, leaf, witness.proof, witness.proof_item)
        .expect_err("constructor must reject leaf drift before backend validation");

    assert_eq!(err, SpendProofBackendError::MembershipWitnessMismatch);
}

#[test]
fn test_membership_rejects_right_leaf() {
    let asset_witness = test_spend_proof_support::canonical_proof_witness()
        .membership
        .into_iter()
        .next()
        .expect("membership");
    let path = right_path(0x21);
    let leaf = right_leaf(0x21);
    let mut store = SettlementStore::new();
    store
        .put_settlement_item(StoreItem::new(path, leaf).expect("right item"))
        .expect("put right item");
    let proof_item = store.settlement_proof_item(&path).expect("proof item");
    let proof = store
        .settlement_proof_blob(&path)
        .expect("proof blob")
        .encode()
        .expect("proof bytes");

    let err = SpendMembershipWitness::new(path, asset_witness.leaf, proof, proof_item)
        .expect_err("right-leaf proof must reject before spend ownership logic");

    assert_eq!(err, SpendProofBackendError::MembershipWitnessMismatch);
}

#[test]
fn test_backend_rejects_nullifier_tamper() {
    let mut stmt = test_spend_proof_support::canonical_proof_stmt();
    stmt.nullifiers[0][0] ^= 0x01;
    let witness = test_spend_proof_support::canonical_proof_witness();
    let backend = default_spend_proof_backend();

    let err = backend
        .prove(&stmt, &witness)
        .expect_err("nullifier tamper must reject");

    assert_eq!(err, SpendProofBackendError::WitnessRelationMismatch);
}

#[test]
fn test_backend_rejects_balance_tamper() {
    let mut stmt = test_spend_proof_support::canonical_proof_stmt();
    let extra_output = stmt.output_leaves[0].clone();
    stmt.output_leaves.push(extra_output);
    let witness = test_spend_proof_support::canonical_proof_witness();
    let backend = default_spend_proof_backend();

    let err = backend
        .prove(&stmt, &witness)
        .expect_err("balance tamper must reject");

    assert_eq!(err, SpendProofBackendError::WitnessRelationMismatch);
}

#[test]
fn test_backend_range_relation_tamper() {
    let mut stmt = test_spend_proof_support::canonical_proof_stmt();
    stmt.output_leaves[0].range_proof.clear();
    let witness = test_spend_proof_support::canonical_proof_witness();
    let backend = default_spend_proof_backend();

    let err = backend
        .prove(&stmt, &witness)
        .expect_err("range relation tamper must reject");

    assert_eq!(err, SpendProofBackendError::RangeRelationMismatch);
}

#[test]
fn test_backend_wrong_prefix_artifact() {
    let stmt = test_spend_proof_support::canonical_proof_stmt();
    let rejected_artifact = test_spend_proof_support::wrong_prefix_artifact();
    let backend = default_spend_proof_backend();

    let err = backend
        .verify(&stmt, &rejected_artifact)
        .expect_err("wrong prefix artifact must reject");

    assert_eq!(err, SpendProofBackendError::InvalidProofPayload);
}

#[test]
fn test_backend_noncanonical_suite_artifact() {
    let stmt = test_spend_proof_support::canonical_proof_stmt();
    let artifact = test_spend_proof_support::noncanonical_suite_artifact();
    let backend = default_spend_proof_backend();

    let err = backend
        .verify(&stmt, &artifact)
        .expect_err("noncanonical suite artifact must reject");

    assert_eq!(err, SpendProofBackendError::UnsupportedSuite);
}

#[test]
fn test_artifact_decoder_noncanonical_suite() {
    let artifact = test_spend_proof_support::noncanonical_suite_artifact();
    let err = SpendProofArtifact::from_wire_hex(&artifact.proof_hex)
        .expect_err("noncanonical suite must reject at decode");

    assert_eq!(err, SpendProofBackendError::UnsupportedSuite);
}

#[test]
fn test_support_public_inputs_aligned() {
    let (tx, _) = test_spend_proof_support::canonical_public_contract_tx();
    let spend = tx.proof.spend.expect("spend proof");
    let witness = test_spend_proof_support::canonical_proof_witness();
    let proof_input = spend.inputs.first().expect("proof input");
    let s_in = witness.input_s_in[0];
    let receiver_secret =
        z00z_wallets::key::ReceiverSecret::from_bytes(test_spend_proof_support::recv_sec())
            .expect("receiver secret");

    let view_sk = derive_view_secret_key(&receiver_secret).expect("view key");
    let owner_handle = derive_owner_handle(&receiver_secret);
    let r_pub = decode_r_pub(
        &hex::decode(&proof_input.r_pub_hex)
            .expect("r_pub hex")
            .try_into()
            .expect("r_pub bytes"),
    )
    .expect("r_pub point");
    let dh = compute_dh_receiver(&view_sk, &r_pub).expect("dh");
    let k_in = derive_k_dh(&dh);
    let expected_owner_tag = compute_owner_tag(&owner_handle, &k_in);
    let expected_leaf_ad_id = hash_zk::<AssetIdDomain>("", &[&s_in]);
    let expected_nullifier =
        z00z_wallets::tx::derive_spend_nullifier(test_spend_proof_support::CHAIN_ID, &s_in);

    assert_eq!(
        hex::encode(expected_owner_tag),
        proof_input.owner_tag_hex,
        "owner_tag theorem public input drifted"
    );
    assert_eq!(
        hex::encode(expected_leaf_ad_id),
        proof_input.leaf_ad_id_hex,
        "leaf_ad_id theorem public input drifted"
    );
    assert_eq!(
        hex::encode(expected_nullifier),
        proof_input.nullifier_hex,
        "nullifier theorem public input drifted"
    );
}
