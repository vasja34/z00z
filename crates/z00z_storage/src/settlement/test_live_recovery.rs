use redb::{Database, ReadableTable, TableDefinition};
use tempfile::tempdir;
use z00z_core::assets::{AssetLeaf, AssetPackPlain};
use z00z_core::vouchers::{VoucherLifecycleV1, VoucherValidityWindowV1};
use z00z_crypto::{expert::encoding::to_hex, ZkPackEncrypted};
use z00z_utils::codec::{BincodeCodec, Codec, JsonCodec};

use crate::checkpoint::{CheckpointExecOut, CheckpointExecTx, CheckpointInRef};
use crate::settlement::{
    BucketId, BucketPolicy, CheckpointPublicationV1, FeeActorCtx, FeeEnvelope, FeeReplayKey,
    FeeSupportCtx, PolicySetCommitmentV1, PublicationModeTagV1, RightActionCtx, RightClass,
    RightLeaf, RootGenerationTagV1, SettlementActionV1, SettlementExecHandoff, SettlementLeaf,
    SettlementLeafFamily, SettlementListReq, SettlementPath, SettlementRecoveryState,
    SettlementRouteCtx, ShardRootLeafV1, TerminalId, TerminalLeaf, VoucherAction, VoucherActionCtx,
    VoucherBackingRef, VoucherLeaf,
};

use super::{
    hjmt_config::SettlementBackendMode,
    hjmt_journal::{decode_journal, encode_journal, HjmtCommitStatus},
    store::{test_env_lock, TEST_HJMT_INJ_STAGE_ENV},
    ClaimNullTx, ClaimNullifier, DefinitionId, SerialId, SettlementStore, StoreItem, StoreOp,
};

const CLAIM_NULL_TABLE: TableDefinition<&[u8], &[u8]> =
    TableDefinition::new("settlement_claim_nulls");
const META_TABLE: TableDefinition<&[u8], &[u8]> = TableDefinition::new("settlement_meta");
const EXEC_TABLE: TableDefinition<&[u8], &[u8]> = TableDefinition::new("settlement_cp_execs");
const HJMT_JOURNAL_TABLE: TableDefinition<&[u8], &[u8]> =
    TableDefinition::new("settlement_hjmt_journal");
const HJMT_PENDING_META_TABLE: TableDefinition<&[u8], &[u8]> =
    TableDefinition::new("settlement_hjmt_pending_meta");
const HJMT_ROOT_TABLE: TableDefinition<&[u8], &[u8]> =
    TableDefinition::new("settlement_hjmt_roots");
const DB_FILE: &str = "settlement_state.redb";
const KEY_ACTIVE: &[u8] = b"active_version";
const JOURNAL_STATUS_OFFSET: usize = 219;

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
struct StateMetaWire {
    version: u64,
    state_root: [u8; 32],
    flat_root: [u8; 32],
    snap_id: [u8; 32],
    draft_id: [u8; 32],
    check_id: [u8; 32],
    exec_id: [u8; 32],
    def_root: Option<[u8; 32]>,
    #[serde(default)]
    fee_replay_count: u64,
    #[serde(default)]
    fee_replay_digest: [u8; 32],
}

fn path(definition: u8, serial: u32, asset: u8) -> SettlementPath {
    SettlementPath::new(
        DefinitionId::new([definition; 32]),
        SerialId::new(serial),
        TerminalId::new([asset; 32]),
    )
}

fn leaf(path: SettlementPath, value: u64) -> TerminalLeaf {
    let payload = AssetPackPlain {
        value,
        blinding: [3u8; 32],
        s_out: [4u8; 32],
    }
    .to_bytes();

    AssetLeaf {
        asset_id: path.terminal_id().into_bytes(),
        serial_id: path.serial_id.get(),
        r_pub: [1u8; 32],
        owner_tag: [2u8; 32],
        c_amount: [5u8; 32],
        enc_pack: ZkPackEncrypted {
            version: 1,
            ciphertext: payload,
            tag: [0u8; 16],
        },
        range_proof: vec![9u8; 4],
        tag16: 11,
    }
    .into()
}

fn item(path: SettlementPath, value: u64) -> StoreItem {
    StoreItem::new(path, leaf(path, value)).expect("store item")
}

fn bytes(value: u8) -> [u8; 32] {
    [value; 32]
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
        use_nonce: bytes(mark.wrapping_add(6)),
        revocation_policy_id: bytes(mark.wrapping_add(7)),
        transition_policy_id: bytes(mark.wrapping_add(8)),
        challenge_policy_id: bytes(mark.wrapping_add(9)),
        disclosure_policy_id: bytes(mark.wrapping_add(10)),
        retention_policy_id: bytes(mark.wrapping_add(11)),
    }
}

fn voucher_path(mark: u8) -> SettlementPath {
    SettlementPath::new(
        DefinitionId::new(bytes(mark.wrapping_add(12))),
        SerialId::new(u32::from(mark) + 1),
        TerminalId::new(bytes(mark)),
    )
}

fn voucher_leaf(mark: u8) -> VoucherLeaf {
    VoucherLeaf {
        version: 1,
        terminal_id: TerminalId::new(bytes(mark)),
        issuer_commitment: bytes(mark.wrapping_add(13)),
        holder_commitment: bytes(mark.wrapping_add(14)),
        beneficiary_commitment: bytes(mark.wrapping_add(15)),
        refund_target_commitment: bytes(mark.wrapping_add(16)),
        backing: VoucherBackingRef::ReserveCommitment(bytes(mark.wrapping_add(17))),
        face_value: 150,
        remaining_value: 95,
        policy_id: bytes(mark.wrapping_add(18)),
        action_pool_id: bytes(mark.wrapping_add(19)),
        lifecycle: VoucherLifecycleV1::Active,
        validity: VoucherValidityWindowV1 {
            valid_from: 10,
            valid_until: 40,
        },
        receiver_must_accept: true,
        allow_reject: true,
        replay_nonce: bytes(mark.wrapping_add(20)),
        disclosure_commitment: Some(bytes(mark.wrapping_add(21))),
        audit_commitment: Some(bytes(mark.wrapping_add(22))),
    }
}

fn voucher_item(mark: u8) -> StoreItem {
    StoreItem::new(
        voucher_path(mark),
        SettlementLeaf::Voucher(voucher_leaf(mark)),
    )
    .expect("voucher item")
}

fn right_ctx(leaf: &RightLeaf, now: u64) -> RightActionCtx {
    RightActionCtx {
        now,
        expected_holder: Some(leaf.holder_commitment),
        expected_control: Some(leaf.control_commitment),
        ..RightActionCtx::default()
    }
}

fn fee_actor(mark: u8, now: u64) -> FeeActorCtx {
    FeeActorCtx {
        now,
        payer_commitment: Some(bytes(mark.wrapping_add(40))),
        sponsor_commitment: None,
    }
}

fn fee_envelope(mark: u8, support: FeeSupportCtx) -> FeeEnvelope {
    let support_ref = Some(bytes(mark.wrapping_add(41)));
    let budget_units = support.required_units.saturating_add(1);
    FeeEnvelope {
        version: 1,
        payer_commitment: bytes(mark.wrapping_add(40)),
        sponsor_commitment: [0u8; 32],
        budget_units,
        budget_commitment: FeeEnvelope::budget_bind(budget_units, support_ref),
        domain_id: support.domain_id,
        expires_at: 80,
        nonce: bytes(mark.wrapping_add(42)),
        transition_id: support.transition_id,
        replay_key: bytes(mark.wrapping_add(43)),
        support_ref,
        failure_policy_id: bytes(mark.wrapping_add(44)),
    }
}

fn fee_put_ops(
    path: SettlementPath,
    leaf: RightLeaf,
) -> Result<Vec<StoreOp>, Box<dyn std::error::Error>> {
    Ok(vec![StoreOp::Put(Box::new(StoreItem::new(path, leaf)?))])
}

fn fee_del_ops(path: SettlementPath) -> Vec<StoreOp> {
    vec![StoreOp::Delete(path)]
}

fn exec_tx(path: SettlementPath, value: u64, proof: &[u8]) -> CheckpointExecTx {
    CheckpointExecTx::new(
        vec![CheckpointInRef::new(
            path.terminal_id().into_bytes(),
            path.serial_id,
        )],
        vec![CheckpointExecOut::new(path.definition_id, leaf(path, value)).expect("exec out")],
        proof.to_vec(),
    )
    .expect("exec tx")
}

fn exec_handoff_tx(input: SettlementPath, outputs: &[StoreItem], proof: &[u8]) -> CheckpointExecTx {
    let outputs = outputs
        .iter()
        .map(|item| {
            CheckpointExecOut::new(
                item.path().definition_id,
                item.terminal_leaf().expect("terminal output").clone(),
            )
            .expect("exec out")
        })
        .collect();

    CheckpointExecTx::new(
        vec![CheckpointInRef::new(
            input.terminal_id().into_bytes(),
            input.serial_id,
        )],
        outputs,
        proof.to_vec(),
    )
    .expect("exec tx")
}

fn publication_policy_set_digest(recovery: &SettlementRecoveryState) -> [u8; 32] {
    PolicySetCommitmentV1::singleton_live(
        u64::from(recovery.bucket_policy_generation),
        recovery.bucket_policy_id,
        recovery.version,
    )
    .digest()
    .expect("policy-set digest")
}

fn publication_leaf(
    shard_id: u32,
    recovery: &SettlementRecoveryState,
    route_table_digest: [u8; 32],
    journal_checkpoint: u64,
    shard_epoch: u64,
    local_sequence: u64,
) -> ShardRootLeafV1 {
    ShardRootLeafV1::new(
        shard_id,
        recovery.state_root.into_bytes(),
        shard_epoch,
        14,
        route_table_digest,
        publication_policy_set_digest(recovery),
        journal_checkpoint,
        local_sequence,
        0,
    )
}

fn publication(
    publication_checkpoint: u64,
    route_table_digest: [u8; 32],
    prior_public_root: crate::settlement::SettlementStateRoot,
    shard_leaves: Vec<ShardRootLeafV1>,
) -> Result<CheckpointPublicationV1, Box<dyn std::error::Error>> {
    let publication = CheckpointPublicationV1::new(
        RootGenerationTagV1::RootGeneration1,
        PublicationModeTagV1::CheckpointWindow,
        publication_checkpoint,
        route_table_digest,
        prior_public_root,
        shard_leaves,
    );
    publication.check_contract_v1()?;
    Ok(publication)
}

fn claim(seed: u8) -> ClaimNullTx {
    ClaimNullTx {
        nullifier: ClaimNullifier::new([seed; 32]),
        claim_id_hex: format!("{seed:02x}").repeat(32),
        chain_id: u32::from(seed),
        tx_digest_hex: format!("{:02x}", seed.wrapping_add(1)).repeat(32),
    }
}

fn db(root: &std::path::Path) -> Database {
    Database::create(root.join(DB_FILE)).expect("open db")
}

fn active_version(root: &std::path::Path) -> u64 {
    let db = db(root);
    let read = db.begin_read().expect("begin read");
    let table = read.open_table(META_TABLE).expect("meta table");
    let bytes = table
        .get(KEY_ACTIVE)
        .expect("active version get")
        .expect("active version");
    let mut raw = [0u8; 8];
    raw.copy_from_slice(bytes.value());
    u64::from_be_bytes(raw)
}

fn pending_meta(root: &std::path::Path, version: u64) -> StateMetaWire {
    let db = db(root);
    let read = db.begin_read().expect("begin read");
    let table = read
        .open_table(HJMT_PENDING_META_TABLE)
        .expect("pending meta table");
    let bytes = table
        .get(version.to_be_bytes().as_slice())
        .expect("pending meta get")
        .expect("pending meta row")
        .value()
        .to_vec();
    let codec = BincodeCodec;
    codec.deserialize(&bytes).expect("pending meta decode")
}

fn claim_null_key(version: u64, nullifier: ClaimNullifier) -> Vec<u8> {
    let mut key = Vec::with_capacity(40);
    key.extend_from_slice(&version.to_be_bytes());
    key.extend_from_slice(nullifier.as_bytes());
    key
}

fn remove_claim_row(root: &std::path::Path, version: u64, nullifier: ClaimNullifier) {
    let db = db(root);
    let write = db.begin_write().expect("begin write");
    {
        let mut table = write.open_table(CLAIM_NULL_TABLE).expect("claim table");
        table
            .remove(claim_null_key(version, nullifier).as_slice())
            .expect("remove claim row");
    }
    write.commit().expect("commit claim removal");
}

fn corrupt_parent_root(root: &std::path::Path, version: u64) {
    let db = db(root);
    let write = db.begin_write().expect("begin write");
    {
        let mut table = write.open_table(HJMT_ROOT_TABLE).expect("root table");
        let mut keys = Vec::new();
        for entry in table.iter().expect("iterate root rows") {
            let (key, _) = entry.expect("root row");
            if key.value().starts_with(&version.to_be_bytes()) {
                keys.push(key.value().to_vec());
            }
        }
        let key = keys.first().expect("hjmt parent root row");
        table
            .insert(key.as_slice(), [0x5Au8; 32].as_slice())
            .expect("tamper parent root");
    }
    write.commit().expect("commit parent root tamper");
}

fn corrupt_journal_status(root: &std::path::Path, version: u64, status: u8) {
    let db = db(root);
    let write = db.begin_write().expect("begin write");
    {
        let mut table = write.open_table(HJMT_JOURNAL_TABLE).expect("journal table");
        let bytes = table
            .get(version.to_be_bytes().as_slice())
            .expect("read journal")
            .expect("journal row")
            .value()
            .to_vec();
        let mut entry = decode_journal(&bytes).expect("decode journal");
        let bytes = match status {
            0 => {
                entry.status = HjmtCommitStatus::Prepared;
                encode_journal(&entry)
            }
            1 => {
                entry.status = HjmtCommitStatus::ChildrenCommitted;
                encode_journal(&entry)
            }
            2 => {
                entry.status = HjmtCommitStatus::ParentsCommitted;
                encode_journal(&entry)
            }
            3 => {
                entry.status = HjmtCommitStatus::RootPublished;
                encode_journal(&entry)
            }
            _ => {
                let mut encoded = encode_journal(&entry);
                encoded[JOURNAL_STATUS_OFFSET] = status;
                encoded
            }
        };
        table
            .insert(version.to_be_bytes().as_slice(), bytes.as_slice())
            .expect("tamper journal status");
    }
    write.commit().expect("commit journal tamper");
}

fn corrupt_journal_root_generation(root: &std::path::Path, version: u64, root_generation: u8) {
    let db = db(root);
    let write = db.begin_write().expect("begin write");
    {
        let mut table = write.open_table(HJMT_JOURNAL_TABLE).expect("journal table");
        let bytes = table
            .get(version.to_be_bytes().as_slice())
            .expect("read journal")
            .expect("journal row")
            .value()
            .to_vec();
        let mut entry = decode_journal(&bytes).expect("decode journal");
        entry.root_generation = root_generation;
        let bytes = encode_journal(&entry);
        table
            .insert(version.to_be_bytes().as_slice(), bytes.as_slice())
            .expect("tamper journal root generation");
    }
    write.commit().expect("commit journal generation tamper");
}

fn corrupt_journal_proof_version(root: &std::path::Path, version: u64, proof_version: u16) {
    let db = db(root);
    let write = db.begin_write().expect("begin write");
    {
        let mut table = write.open_table(HJMT_JOURNAL_TABLE).expect("journal table");
        let bytes = table
            .get(version.to_be_bytes().as_slice())
            .expect("read journal")
            .expect("journal row")
            .value()
            .to_vec();
        let mut entry = decode_journal(&bytes).expect("decode journal");
        entry.proof_version = proof_version;
        let bytes = encode_journal(&entry);
        table
            .insert(version.to_be_bytes().as_slice(), bytes.as_slice())
            .expect("tamper journal proof version");
    }
    write.commit().expect("commit journal proof version tamper");
}

fn corrupt_journal_policy(root: &std::path::Path, version: u64, bucket_policy_id: [u8; 32]) {
    let db = db(root);
    let write = db.begin_write().expect("begin write");
    {
        let mut table = write.open_table(HJMT_JOURNAL_TABLE).expect("journal table");
        let bytes = table
            .get(version.to_be_bytes().as_slice())
            .expect("read journal")
            .expect("journal row")
            .value()
            .to_vec();
        let mut entry = decode_journal(&bytes).expect("decode journal");
        entry.bucket_policy_id = bucket_policy_id;
        let bytes = encode_journal(&entry);
        table
            .insert(version.to_be_bytes().as_slice(), bytes.as_slice())
            .expect("tamper journal policy");
    }
    write.commit().expect("commit journal policy tamper");
}

fn remove_exec_row(root: &std::path::Path, exec_id: [u8; 32]) {
    let db = db(root);
    let write = db.begin_write().expect("begin write");
    {
        let mut table = write.open_table(EXEC_TABLE).expect("exec table");
        table
            .remove(&exec_id[..])
            .expect("remove exec row")
            .expect("pending exec row");
    }
    write.commit().expect("commit exec removal");
}

fn same_bucket_paths(store: &mut SettlementStore, needed: usize) -> Vec<SettlementPath> {
    let policy = store.bucket_policy();
    let first = path(61, 31, 1);
    let target_bucket = first.bucket_id(policy);
    let mut selected = vec![(1u8, first)];
    for seed in 2..=255 {
        let candidate = path(61, 31, seed);
        if candidate.bucket_id(policy) == target_bucket {
            selected.push((seed, candidate));
            if selected.len() == needed {
                break;
            }
        }
    }

    assert_eq!(selected.len(), needed, "failed to find same-bucket paths");
    for (seed, candidate) in &selected {
        store
            .put_settlement_item(item(*candidate, 6_100 + u64::from(*seed)))
            .expect("seed candidate bucket item");
    }
    selected.into_iter().map(|(_, path)| path).collect()
}

fn sibling_bucket_id(bucket_id: BucketId, bucket_bits: u8) -> BucketId {
    let mut bytes = bucket_id.into_bytes();
    let bit_index = bucket_bits - 1;
    let byte_index = usize::from(bit_index / 8);
    let bit_mask = 1u8 << (7 - (bit_index % 8));
    bytes[byte_index] ^= bit_mask;
    BucketId::new(bytes)
}

fn sibling_bucket_pair(store: &mut SettlementStore) -> (SettlementPath, SettlementPath) {
    let mut first_paths = std::collections::BTreeMap::<BucketId, SettlementPath>::new();
    let bucket_bits = store.bucket_policy().bucket_bits();

    for seed in 1..=128 {
        let candidate = path(62, 32, seed);
        store
            .put_settlement_item(item(candidate, 6_200 + u64::from(seed)))
            .expect("seed sibling bucket item");
        let bucket = candidate.bucket_id(store.bucket_policy());
        let sibling = sibling_bucket_id(bucket, bucket_bits);
        if let Some(other) = first_paths.get(&sibling).copied() {
            if store.merge_proof(&other, &candidate).is_ok() {
                return (other, candidate);
            }
        }
        first_paths.entry(bucket).or_insert(candidate);
    }

    panic!("failed to find sibling bucket pair")
}

fn trigger_split_path(
    store: &mut SettlementStore,
    target: SettlementPath,
    start_seed: u8,
) -> SettlementPath {
    let policy = store.bucket_policy();
    let threshold = usize::try_from(store.bucket_policy().min_bucket_count()).expect("usize") + 1;
    let current = usize::try_from(
        store
            .bucket_occupancy_metric(&target)
            .expect("target occupancy metric")
            .exact_count,
    )
    .expect("usize");
    let needed = threshold.saturating_sub(current);
    let target_bucket = target.bucket_id(policy);
    assert!(needed > 0, "target is already split-ready");
    let mut selected = Vec::with_capacity(needed);
    for seed in start_seed..=255 {
        let candidate = path(61, 31, seed);
        if candidate.bucket_id(policy) == target_bucket {
            selected.push((seed, candidate));
            if selected.len() == needed {
                break;
            }
        }
    }

    assert_eq!(selected.len(), needed, "failed to reach the split trigger");
    for (seed, candidate) in &selected {
        store
            .put_settlement_item(item(*candidate, 6_100 + u64::from(*seed)))
            .expect("seed split trigger item");
    }
    assert!(
        store.split_proof(&target).is_ok(),
        "failed to reach the split trigger"
    );
    *selected
        .last()
        .map(|(_, path)| path)
        .expect("split trigger path")
}

fn next_policy(store: &SettlementStore) -> BucketPolicy {
    BucketPolicy::new(
        store.bucket_policy().bucket_bits() + 1,
        store.bucket_policy().min_bucket_count(),
        store.bucket_policy().max_target_leaf_count(),
        store.bucket_policy().compatibility_generation() + 1,
    )
    .expect("next bucket policy")
}

fn voucher_ctx(leaf: &VoucherLeaf, now: u64) -> VoucherActionCtx {
    VoucherActionCtx {
        now,
        expected_holder: Some(leaf.holder_commitment),
        expected_beneficiary: Some(leaf.beneficiary_commitment),
        expected_refund_target: Some(leaf.refund_target_commitment),
        ..VoucherActionCtx::default()
    }
}

#[test]
fn test_hjmt_reload_claim_rows() -> Result<(), Box<dyn std::error::Error>> {
    let guard = test_env_lock().lock().expect("env lock");
    std::env::remove_var(TEST_HJMT_INJ_STAGE_ENV);
    let temp = tempdir()?;
    let claim = claim(38);

    let mut store =
        SettlementStore::load_with_backend_mode(temp.path(), SettlementBackendMode::Hjmt)?;
    let root = store.apply_settlement_claim_ops(
        vec![StoreOp::Put(Box::new(item(path(38, 14, 1), 3_801)))],
        std::slice::from_ref(&claim),
    )?;
    assert!(store.settlement_claim_null_rec(&claim.nullifier)?.is_some());
    drop(store);

    let reloaded =
        SettlementStore::load_with_backend_mode(temp.path(), SettlementBackendMode::Hjmt)?;
    assert_eq!(reloaded.settlement_root()?, root);
    assert!(reloaded
        .settlement_claim_null_rec(&claim.nullifier)?
        .is_some());
    drop(reloaded);

    remove_claim_row(temp.path(), 1, claim.nullifier);
    let err =
        match SettlementStore::load_with_backend_mode(temp.path(), SettlementBackendMode::Hjmt) {
            Ok(_) => panic!("claim replay row drift must reject"),
            Err(err) => err,
        };
    assert!(
        err.to_string()
            .contains("hjmt child commit digest mismatch"),
        "{err}"
    );
    drop(guard);
    Ok(())
}

#[test]
fn test_hjmt_reload_listing() -> Result<(), Box<dyn std::error::Error>> {
    let guard = test_env_lock().lock().expect("env lock");
    std::env::remove_var(TEST_HJMT_INJ_STAGE_ENV);
    let temp = tempdir()?;

    let item = voucher_item(94);
    let path = item.path();
    let leaf = item.voucher_leaf()?.clone();

    let mut store =
        SettlementStore::load_with_backend_mode(temp.path(), SettlementBackendMode::Hjmt)?;
    let root = store.put_settlement_item(item.clone())?;
    drop(store);

    let reloaded =
        SettlementStore::load_with_backend_mode(temp.path(), SettlementBackendMode::Hjmt)?;
    assert_eq!(reloaded.settlement_root()?, root);
    let recovered_item = reloaded
        .get_settlement_item(&path)?
        .expect("recovered voucher item");
    assert_eq!(recovered_item.path(), path);
    assert_eq!(recovered_item.voucher_leaf()?, &leaf);
    assert_eq!(
        reloaded.list_settlement(SettlementListReq::all(8))?.items(),
        &[item]
    );

    let proof = reloaded.settlement_proof_blob(&path)?;
    reloaded.validate_settlement_proof_blob(&proof)?;
    drop(reloaded);

    let mut deleting =
        SettlementStore::load_with_backend_mode(temp.path(), SettlementBackendMode::Hjmt)?;
    let delete_root = deleting.apply_settlement_ops(vec![StoreOp::Delete(path)])?;
    drop(deleting);

    let recovered =
        SettlementStore::load_with_backend_mode(temp.path(), SettlementBackendMode::Hjmt)?;
    assert_eq!(recovered.settlement_root()?, delete_root);
    assert!(recovered.get_settlement_item(&path)?.is_none());
    assert!(recovered
        .list_settlement(SettlementListReq::all(8))?
        .items()
        .is_empty());

    let absence =
        recovered.settlement_nonexistence_proof_blob(&path, SettlementLeafFamily::Voucher)?;
    recovered
        .validate_settlement_nonexistence_proof_blob(&absence, SettlementLeafFamily::Voucher)?;

    drop(guard);
    Ok(())
}

#[test]
fn test_hjmt_reload_delta_history() -> Result<(), Box<dyn std::error::Error>> {
    let guard = test_env_lock().lock().expect("env lock");
    std::env::remove_var(TEST_HJMT_INJ_STAGE_ENV);
    let temp = tempdir()?;

    let path = voucher_path(95);
    let mut pending = voucher_leaf(95);
    pending.lifecycle = VoucherLifecycleV1::PendingAcceptance;

    let mut store =
        SettlementStore::load_with_backend_mode(temp.path(), SettlementBackendMode::Hjmt)?;
    let issue_ops = vec![StoreOp::Put(Box::new(StoreItem::new(
        path,
        SettlementLeaf::Voucher(pending.clone()),
    )?))];
    let issue_root = store.issue_voucher_with_fee(
        None,
        path,
        pending.clone(),
        VoucherActionCtx::default(),
        fee_envelope(95, store.fee_support_ctx(&issue_ops)?),
        fee_actor(95, 15),
    )?;
    let issue_ver = store.hjmt_roots.version;
    let issue_delta = store.latest_object_delta().cloned().expect("issue delta");
    assert_eq!(
        issue_delta.selected_action,
        SettlementActionV1::Voucher(VoucherAction::Issue)
    );
    assert_eq!(issue_delta.expected_new_root, issue_root);

    let mut active = pending.clone();
    active.lifecycle = VoucherLifecycleV1::Active;
    let accept_ops = vec![StoreOp::Put(Box::new(StoreItem::new(
        path,
        SettlementLeaf::Voucher(active.clone()),
    )?))];
    let accept_root = store.accept_voucher_with_fee(
        path,
        active.clone(),
        VoucherActionCtx {
            acceptance_confirmed: true,
            ..voucher_ctx(&active, 16)
        },
        fee_envelope(96, store.fee_support_ctx(&accept_ops)?),
        fee_actor(96, 16),
    )?;
    let accept_ver = store.hjmt_roots.version;
    let accept_delta = store.latest_object_delta().cloned().expect("accept delta");
    assert_eq!(
        accept_delta.selected_action,
        SettlementActionV1::Voucher(VoucherAction::Accept)
    );
    assert_eq!(accept_delta.prior_root, issue_root);
    assert_eq!(accept_delta.expected_new_root, accept_root);
    drop(store);

    let reloaded =
        SettlementStore::load_with_backend_mode(temp.path(), SettlementBackendMode::Hjmt)?;
    assert_eq!(reloaded.latest_object_delta(), Some(&accept_delta));
    assert_eq!(
        reloaded.object_delta_for_version(issue_ver)?,
        Some(issue_delta)
    );
    assert_eq!(
        reloaded.object_delta_for_version(accept_ver)?,
        Some(accept_delta)
    );
    assert_eq!(reloaded.settlement_root_for_version(issue_ver)?, issue_root);
    assert_eq!(
        reloaded.settlement_root_for_version(accept_ver)?,
        accept_root
    );

    drop(guard);
    Ok(())
}

#[test]
fn test_hjmt_rejects_parent_drift() -> Result<(), Box<dyn std::error::Error>> {
    let guard = test_env_lock().lock().expect("env lock");
    std::env::remove_var(TEST_HJMT_INJ_STAGE_ENV);
    let temp = tempdir()?;

    let mut store =
        SettlementStore::load_with_backend_mode(temp.path(), SettlementBackendMode::Hjmt)?;
    store.put_settlement_item(item(path(36, 12, 1), 3_601))?;
    drop(store);

    corrupt_parent_root(temp.path(), 1);
    let err =
        match SettlementStore::load_with_backend_mode(temp.path(), SettlementBackendMode::Hjmt) {
            Ok(_) => panic!("tampered parent row must reject"),
            Err(err) => err,
        };
    assert!(
        err.to_string()
            .contains("hjmt parent commit digest mismatch"),
        "{err}"
    );
    drop(guard);
    Ok(())
}

#[test]
fn test_hjmt_rejects_journal_drift() -> Result<(), Box<dyn std::error::Error>> {
    let guard = test_env_lock().lock().expect("env lock");
    std::env::remove_var(TEST_HJMT_INJ_STAGE_ENV);
    let temp = tempdir()?;

    let mut store =
        SettlementStore::load_with_backend_mode(temp.path(), SettlementBackendMode::Hjmt)?;
    store.put_settlement_item(item(path(37, 13, 1), 3_701))?;
    drop(store);

    corrupt_journal_status(temp.path(), 1, 0);
    let err =
        match SettlementStore::load_with_backend_mode(temp.path(), SettlementBackendMode::Hjmt) {
            Ok(_) => panic!("regressed journal status must reject"),
            Err(err) => err,
        };
    assert!(
        err.to_string()
            .contains("hjmt journal active metadata is not root-published"),
        "{err}"
    );

    corrupt_journal_status(temp.path(), 1, 255);
    let err =
        match SettlementStore::load_with_backend_mode(temp.path(), SettlementBackendMode::Hjmt) {
            Ok(_) => panic!("unsupported journal status must reject"),
            Err(err) => err,
        };
    assert!(
        err.to_string()
            .contains("hjmt journal status byte is unsupported"),
        "{err}"
    );
    drop(guard);
    Ok(())
}

#[test]
fn test_rejects_stale_bucket_rows() -> Result<(), Box<dyn std::error::Error>> {
    let guard = test_env_lock().lock().expect("env lock");
    std::env::remove_var(TEST_HJMT_INJ_STAGE_ENV);
    std::env::set_var("Z00Z_SETTLEMENT_BUCKET_BITS", "1");
    let temp = tempdir()?;

    let mut store =
        SettlementStore::load_with_backend_mode(temp.path(), SettlementBackendMode::Hjmt)?;
    store.put_settlement_item(item(path(51, 21, 1), 5_101))?;
    drop(store);

    corrupt_journal_policy(temp.path(), 1, [0xA5; 32]);
    let err =
        match SettlementStore::load_with_backend_mode(temp.path(), SettlementBackendMode::Hjmt) {
            Ok(_) => panic!("stale bucket policy rows must reject"),
            Err(err) => err,
        };
    assert!(
        err.to_string()
            .contains("hjmt persisted bucket policy id does not match active bucket policy"),
        "{err}"
    );

    std::env::remove_var("Z00Z_SETTLEMENT_BUCKET_BITS");
    drop(guard);
    Ok(())
}

#[test]
fn test_hjmt_rejects_contract_drift() -> Result<(), Box<dyn std::error::Error>> {
    let guard = test_env_lock().lock().expect("env lock");
    std::env::remove_var(TEST_HJMT_INJ_STAGE_ENV);

    let generation_temp = tempdir()?;
    let mut generation_store = SettlementStore::load_with_backend_mode(
        generation_temp.path(),
        SettlementBackendMode::Hjmt,
    )?;
    generation_store.put_settlement_item(item(path(52, 22, 1), 5_201))?;
    drop(generation_store);

    corrupt_journal_root_generation(generation_temp.path(), 1, 255);
    let err = match SettlementStore::load_with_backend_mode(
        generation_temp.path(),
        SettlementBackendMode::Hjmt,
    ) {
        Ok(_) => panic!("unsupported journal root generation must reject"),
        Err(err) => err,
    };
    assert!(
        err.to_string()
            .contains("hjmt journal root generation is unsupported"),
        "{err}"
    );

    let proof_temp = tempdir()?;
    let mut proof_store =
        SettlementStore::load_with_backend_mode(proof_temp.path(), SettlementBackendMode::Hjmt)?;
    proof_store.put_settlement_item(item(path(53, 23, 1), 5_301))?;
    drop(proof_store);

    corrupt_journal_proof_version(proof_temp.path(), 1, u16::MAX);
    let err = match SettlementStore::load_with_backend_mode(
        proof_temp.path(),
        SettlementBackendMode::Hjmt,
    ) {
        Ok(_) => panic!("unsupported journal proof version must reject"),
        Err(err) => err,
    };
    assert!(
        err.to_string()
            .contains("hjmt journal proof version is unsupported"),
        "{err}"
    );

    drop(guard);
    Ok(())
}

#[test]
fn test_create_publishes_leaf_replay() -> Result<(), Box<dyn std::error::Error>> {
    let guard = test_env_lock().lock().expect("env lock");
    std::env::remove_var(TEST_HJMT_INJ_STAGE_ENV);
    let temp = tempdir()?;

    let path = right_path(91);
    let leaf = right_leaf(91);
    let mut store =
        SettlementStore::load_with_backend_mode(temp.path(), SettlementBackendMode::Hjmt)?;
    let envelope = fee_envelope(91, store.fee_support_ctx(&fee_put_ops(path, leaf)?)?);
    let replay_key = FeeReplayKey::new(envelope.replay_key);

    std::env::set_var(TEST_HJMT_INJ_STAGE_ENV, "parents");
    let err = store
        .create_right_with_fee(
            path,
            leaf,
            right_ctx(&leaf, 15),
            envelope,
            fee_actor(91, 15),
        )
        .expect_err("parent-stage injection must fail before right publication");
    assert!(
        err.to_string()
            .contains("hjmt journal injection after ParentsCommitted"),
        "{err}"
    );
    std::env::remove_var(TEST_HJMT_INJ_STAGE_ENV);
    drop(store);

    let recovered =
        SettlementStore::load_with_backend_mode(temp.path(), SettlementBackendMode::Hjmt)?;
    let recovered_item = recovered
        .get_settlement_item(&path)?
        .expect("recovered right item");
    assert_eq!(recovered_item.path(), path);
    assert_eq!(recovered_item.right_leaf()?, &leaf);
    assert!(recovered.fee_replay_rec(&replay_key)?.is_some());

    let proof = recovered.settlement_proof_blob(&path)?;
    recovered.validate_settlement_proof_blob(&proof)?;

    drop(guard);
    Ok(())
}

#[test]
fn test_delete_keeps_absence_proof() -> Result<(), Box<dyn std::error::Error>> {
    let guard = test_env_lock().lock().expect("env lock");
    std::env::remove_var(TEST_HJMT_INJ_STAGE_ENV);
    let temp = tempdir()?;

    let path = right_path(92);
    let leaf = right_leaf(92);
    let mut seed =
        SettlementStore::load_with_backend_mode(temp.path(), SettlementBackendMode::Hjmt)?;
    let create_env = fee_envelope(92, seed.fee_support_ctx(&fee_put_ops(path, leaf)?)?);
    seed.create_right_with_fee(
        path,
        leaf,
        right_ctx(&leaf, 15),
        create_env,
        fee_actor(92, 15),
    )?;
    drop(seed);

    let mut interrupted =
        SettlementStore::load_with_backend_mode(temp.path(), SettlementBackendMode::Hjmt)?;
    let delete_env = fee_envelope(93, interrupted.fee_support_ctx(&fee_del_ops(path))?);
    let replay_key = FeeReplayKey::new(delete_env.replay_key);
    std::env::set_var(TEST_HJMT_INJ_STAGE_ENV, "parents");
    let err = interrupted
        .consume_right_with_fee(path, right_ctx(&leaf, 15), delete_env, fee_actor(93, 15))
        .expect_err("parent-stage injection must fail before right deletion publish");
    assert!(
        err.to_string()
            .contains("hjmt journal injection after ParentsCommitted"),
        "{err}"
    );
    std::env::remove_var(TEST_HJMT_INJ_STAGE_ENV);
    drop(interrupted);

    let recovered =
        SettlementStore::load_with_backend_mode(temp.path(), SettlementBackendMode::Hjmt)?;
    assert!(recovered.get_settlement_item(&path)?.is_none());
    assert!(recovered.fee_replay_rec(&replay_key)?.is_some());

    let deletion = recovered.settlement_proof_blob(&path)?;
    recovered.validate_settlement_proof_blob(&deletion)?;

    let absence =
        recovered.settlement_nonexistence_proof_blob(&path, SettlementLeafFamily::Right)?;
    recovered.validate_settlement_nonexistence_proof_blob(&absence, SettlementLeafFamily::Right)?;

    drop(guard);
    Ok(())
}

#[test]
fn test_split_state_pre_publish() -> Result<(), Box<dyn std::error::Error>> {
    let guard = test_env_lock().lock().expect("env lock");
    std::env::remove_var(TEST_HJMT_INJ_STAGE_ENV);
    std::env::set_var("Z00Z_SETTLEMENT_BUCKET_BITS", "1");
    let temp = tempdir()?;

    let mut seed = SettlementStore::load(temp.path())?;
    let steady_count = usize::try_from(seed.bucket_policy().min_bucket_count()).expect("usize");
    let paths = same_bucket_paths(&mut seed, steady_count);
    let baseline_root = seed.settlement_root()?;
    let first = paths[0];
    let err = seed
        .split_proof(&first)
        .expect_err("two members must stay below the split trigger");
    assert_eq!(
        err.to_string(),
        "adaptive split is ineligible under the current bucket occupancy"
    );
    drop(seed);

    let mut trigger = SettlementStore::load(temp.path())?;
    let pending_path = trigger_split_path(&mut trigger, paths[0], 129);
    let split_root = trigger.settlement_root()?;
    let split = trigger
        .split_proof(&first)
        .expect("triggered split proof before interruption");
    let next_policy = next_policy(&trigger);
    let transition = trigger
        .policy_transition_proof(next_policy)
        .expect("policy transition proof before interruption");
    drop(trigger);

    let mut interrupted = SettlementStore::load(temp.path())?;
    std::env::set_var(TEST_HJMT_INJ_STAGE_ENV, "children");
    let err = interrupted
        .put_settlement_item(item(pending_path, 6_300))
        .expect_err("children-stage injection must fail");
    assert!(err
        .to_string()
        .contains("hjmt journal injection after ChildrenCommitted"));
    std::env::remove_var(TEST_HJMT_INJ_STAGE_ENV);
    drop(interrupted);

    let recovered = SettlementStore::load(temp.path())?;
    assert_ne!(split_root, baseline_root);
    assert_eq!(recovered.settlement_root()?, split_root);
    assert_eq!(recovered.split_proof(&first)?, split);
    recovered
        .validate_split_proof(&split)
        .expect("recovered split proof validates");
    assert_eq!(recovered.policy_transition_proof(next_policy)?, transition);
    recovered
        .validate_policy_transition_proof(&transition, next_policy)
        .expect("recovered policy transition validates");

    std::env::remove_var("Z00Z_SETTLEMENT_BUCKET_BITS");
    drop(guard);
    Ok(())
}

#[test]
fn test_merge_state_pre_publish() -> Result<(), Box<dyn std::error::Error>> {
    let guard = test_env_lock().lock().expect("env lock");
    std::env::remove_var(TEST_HJMT_INJ_STAGE_ENV);
    std::env::set_var("Z00Z_SETTLEMENT_BUCKET_BITS", "2");
    let temp = tempdir()?;

    let mut seed = SettlementStore::load(temp.path())?;
    let (left_path, right_path) = sibling_bucket_pair(&mut seed);
    let left = left_path;
    let right = right_path;
    let baseline_root = seed.settlement_root()?;
    let merge = seed
        .merge_proof(&left, &right)
        .expect("merge proof before interruption");
    let next_policy = next_policy(&seed);
    let transition = seed
        .policy_transition_proof(next_policy)
        .expect("policy transition proof before interruption");
    drop(seed);

    let mut interrupted = SettlementStore::load(temp.path())?;
    std::env::set_var(TEST_HJMT_INJ_STAGE_ENV, "children");
    let err = interrupted
        .put_settlement_item(item(left_path, 6_900))
        .expect_err("children-stage injection must fail");
    assert!(err
        .to_string()
        .contains("hjmt journal injection after ChildrenCommitted"));
    std::env::remove_var(TEST_HJMT_INJ_STAGE_ENV);
    drop(interrupted);

    let recovered = SettlementStore::load(temp.path())?;
    assert_eq!(recovered.settlement_root()?, baseline_root);
    assert_eq!(recovered.merge_proof(&left, &right)?, merge);
    recovered
        .validate_merge_proof(&merge)
        .expect("recovered merge proof validates");
    assert_eq!(recovered.policy_transition_proof(next_policy)?, transition);
    recovered
        .validate_policy_transition_proof(&transition, next_policy)
        .expect("recovered policy transition validates");

    std::env::remove_var("Z00Z_SETTLEMENT_BUCKET_BITS");
    drop(guard);
    Ok(())
}

#[test]
fn test_recovery_restores_xfer_proof() -> Result<(), Box<dyn std::error::Error>> {
    let guard = test_env_lock().lock().expect("env lock");
    std::env::remove_var(TEST_HJMT_INJ_STAGE_ENV);
    std::env::set_var("Z00Z_SETTLEMENT_BUCKET_BITS", "1");
    let temp = tempdir()?;

    let mut seed = SettlementStore::load(temp.path())?;
    let steady_count = usize::try_from(seed.bucket_policy().min_bucket_count()).expect("usize");
    let paths = same_bucket_paths(&mut seed, steady_count);
    let first = paths[0];
    drop(seed);

    let mut trigger = SettlementStore::load(temp.path())?;
    let pending_path = trigger_split_path(&mut trigger, paths[0], 129);
    drop(trigger);

    let mut interrupted = SettlementStore::load(temp.path())?;
    std::env::set_var(TEST_HJMT_INJ_STAGE_ENV, "parents");
    let err = interrupted
        .put_settlement_item(item(pending_path, 6_400))
        .expect_err("parent-stage injection must fail");
    assert!(err
        .to_string()
        .contains("hjmt journal injection after ParentsCommitted"));
    std::env::remove_var(TEST_HJMT_INJ_STAGE_ENV);
    drop(interrupted);

    let recovered = SettlementStore::load(temp.path())?;
    let split = recovered
        .split_proof(&first)
        .expect("recovered split proof");
    recovered
        .validate_split_proof(&split)
        .expect("recovered split proof validates");
    let next_policy = next_policy(&recovered);
    let transition = recovered
        .policy_transition_proof(next_policy)
        .expect("recovered policy transition proof");
    recovered
        .validate_policy_transition_proof(&transition, next_policy)
        .expect("recovered policy transition validates");
    let root = recovered.settlement_root()?;
    drop(recovered);

    let reloaded = SettlementStore::load(temp.path())?;
    assert_eq!(reloaded.settlement_root()?, root);
    assert_eq!(reloaded.split_proof(&first)?, split);
    assert_eq!(reloaded.policy_transition_proof(next_policy)?, transition);

    std::env::remove_var("Z00Z_SETTLEMENT_BUCKET_BITS");
    drop(guard);
    Ok(())
}

#[test]
fn test_hjmt_rolls_back_journal() -> Result<(), Box<dyn std::error::Error>> {
    let guard = test_env_lock().lock().expect("env lock");
    std::env::remove_var(TEST_HJMT_INJ_STAGE_ENV);

    for stage in ["prepared", "children", "parents"] {
        let temp = tempdir()?;
        let mut seed =
            SettlementStore::load_with_backend_mode(temp.path(), SettlementBackendMode::Hjmt)?;
        let seed_item = item(path(34, 10, 1), 3_401);
        let seed_root = seed.put_settlement_item(seed_item.clone())?;
        drop(seed);

        let pending_item = item(path(35, 11, 2), 3_502);
        std::env::set_var(TEST_HJMT_INJ_STAGE_ENV, stage);
        let mut store =
            SettlementStore::load_with_backend_mode(temp.path(), SettlementBackendMode::Hjmt)?;
        let err = store
            .put_settlement_item(pending_item.clone())
            .expect_err("injected hjmt stage must fail");
        assert!(err.to_string().contains("hjmt journal injection"));
        std::env::remove_var(TEST_HJMT_INJ_STAGE_ENV);
        drop(store);

        let mut recovered =
            SettlementStore::load_with_backend_mode(temp.path(), SettlementBackendMode::Hjmt)?;
        if stage == "parents" {
            assert_eq!(
                recovered
                    .list_settlement(SettlementListReq::all(8))?
                    .items(),
                &[seed_item.clone(), pending_item.clone()]
            );
        } else {
            assert_eq!(recovered.settlement_root()?, seed_root);
            assert_eq!(
                recovered
                    .list_settlement(SettlementListReq::all(8))?
                    .items(),
                &[seed_item.clone()]
            );
        }
        let root = if stage == "parents" {
            recovered.settlement_root()?
        } else {
            recovered.put_settlement_item(pending_item.clone())?
        };
        drop(recovered);

        let reloaded =
            SettlementStore::load_with_backend_mode(temp.path(), SettlementBackendMode::Hjmt)?;
        assert_eq!(reloaded.settlement_root()?, root);
        drop(reloaded);

        let reloaded_again =
            SettlementStore::load_with_backend_mode(temp.path(), SettlementBackendMode::Hjmt)?;
        assert_eq!(reloaded_again.settlement_root()?, root);
    }

    std::env::remove_var(TEST_HJMT_INJ_STAGE_ENV);
    drop(guard);
    Ok(())
}

#[test]
fn exec_handoff_recovers_on_crash() -> Result<(), Box<dyn std::error::Error>> {
    let guard = test_env_lock().lock().expect("env lock");
    std::env::remove_var(TEST_HJMT_INJ_STAGE_ENV);
    let temp = tempdir()?;

    let spent_path = path(40, 16, 1);
    let scope_path = path(41, 17, 2);
    let sibling_path = path(41, 17, 3);
    let route = SettlementRouteCtx::new([0x56; 32], 6, 14, [0x79; 32]);

    let route_json = JsonCodec.serialize_pretty(&route)?;
    let route_roundtrip: SettlementRouteCtx = JsonCodec.deserialize(&route_json)?;
    assert_eq!(route_roundtrip, route);
    for field in [
        "\"batch_id\"",
        "\"shard_id\"",
        "\"routing_generation\"",
        "\"route_table_digest\"",
    ] {
        assert!(
            std::str::from_utf8(&route_json)?.contains(field),
            "route export missing {field}"
        );
    }

    let mut seed =
        SettlementStore::load_with_backend_mode(temp.path(), SettlementBackendMode::Hjmt)?;
    let seed_root = seed.put_settlement_item(item(spent_path, 4_001))?;
    drop(seed);

    let scope_item = item(scope_path, 4_102);
    let sibling_item = item(sibling_path, 4_103);
    let ops = vec![
        StoreOp::Delete(spent_path),
        StoreOp::Put(Box::new(scope_item.clone())),
        StoreOp::Put(Box::new(sibling_item.clone())),
    ];
    let txs = vec![exec_handoff_tx(
        spent_path,
        &[scope_item.clone(), sibling_item.clone()],
        b"crash-after-durable-journal-advance",
    )];

    std::env::set_var(TEST_HJMT_INJ_STAGE_ENV, "parents");
    let mut store =
        SettlementStore::load_with_backend_mode(temp.path(), SettlementBackendMode::Hjmt)?;
    let err = store
        .apply_exec_handoff(SettlementExecHandoff::new(route, ops, txs))
        .expect_err("parent-stage crash injection must fail after durable journal advance");
    assert!(
        err.to_string()
            .contains("hjmt journal injection after ParentsCommitted"),
        "{err}"
    );
    std::env::remove_var(TEST_HJMT_INJ_STAGE_ENV);
    drop(store);

    let recovered =
        SettlementStore::load_with_backend_mode(temp.path(), SettlementBackendMode::Hjmt)?;
    let recovered_root = recovered.settlement_root()?;
    assert_ne!(recovered_root, seed_root);
    assert!(recovered.get_settlement_item(&spent_path)?.is_none());
    assert_eq!(
        recovered
            .get_settlement_item(&scope_path)?
            .expect("scope item present")
            .path(),
        scope_path
    );
    assert_eq!(
        recovered
            .get_settlement_item(&sibling_path)?
            .expect("sibling item present")
            .path(),
        sibling_path
    );

    let expected_version = recovered.hjmt_roots.version;
    let recovery = recovered.recovery_state()?;
    assert_eq!(recovery.version, expected_version);
    assert_eq!(recovery.state_root, recovered_root);
    assert_ne!(recovery.journal_lineage, [0u8; 32]);
    assert_eq!(recovery.route, Some(route));
    drop(recovered);

    let recovery_json = JsonCodec.serialize_pretty(&recovery)?;
    let recovery_roundtrip: SettlementRecoveryState = JsonCodec.deserialize(&recovery_json)?;
    assert_eq!(recovery_roundtrip, recovery);
    for field in [
        "\"version\"",
        "\"state_root\"",
        "\"root_generation\"",
        "\"proof_version\"",
        "\"bucket_policy_generation\"",
        "\"bucket_policy_id\"",
        "\"journal_lineage\"",
        "\"route\"",
    ] {
        assert!(
            std::str::from_utf8(&recovery_json)?.contains(field),
            "recovery export missing {field}"
        );
    }

    std::env::remove_var(TEST_HJMT_INJ_STAGE_ENV);
    drop(guard);
    Ok(())
}

#[test]
fn test_parent_crash_exact_root() -> Result<(), Box<dyn std::error::Error>> {
    let guard = test_env_lock().lock().expect("env lock");
    std::env::remove_var(TEST_HJMT_INJ_STAGE_ENV);
    let temp = tempdir()?;

    let spent_path = path(43, 18, 1);
    let scope_path = path(43, 18, 2);
    let sibling_path = path(43, 18, 3);
    let route = SettlementRouteCtx::new([0x66; 32], 7, 14, [0x7A; 32]);

    let mut seed =
        SettlementStore::load_with_backend_mode(temp.path(), SettlementBackendMode::Hjmt)?;
    seed.put_settlement_item(item(spent_path, 5_001))?;
    let prior_recovery = seed.recovery_state()?;
    drop(seed);

    let scope_item = item(scope_path, 5_102);
    let sibling_item = item(sibling_path, 5_103);
    let ops = vec![
        StoreOp::Delete(spent_path),
        StoreOp::Put(Box::new(scope_item.clone())),
        StoreOp::Put(Box::new(sibling_item.clone())),
    ];
    let txs = vec![exec_handoff_tx(
        spent_path,
        &[scope_item, sibling_item],
        b"fov-g-003-parent-stage-crash",
    )];

    std::env::set_var(TEST_HJMT_INJ_STAGE_ENV, "parents");
    let mut store =
        SettlementStore::load_with_backend_mode(temp.path(), SettlementBackendMode::Hjmt)?;
    let err = store
        .apply_exec_handoff(SettlementExecHandoff::new(route, ops, txs))
        .expect_err("parent-stage crash injection must fail after durable journal advance");
    assert!(
        err.to_string()
            .contains("hjmt journal injection after ParentsCommitted"),
        "{err}"
    );
    std::env::remove_var(TEST_HJMT_INJ_STAGE_ENV);
    drop(store);

    let recovered =
        SettlementStore::load_with_backend_mode(temp.path(), SettlementBackendMode::Hjmt)?;
    let later_recovery = recovered.recovery_state()?;
    drop(recovered);

    let route_table_digest = [0x24; 32];
    let prior = publication(
        40,
        route_table_digest,
        crate::settlement::SettlementStateRoot::settlement_v1([0x55; 32]),
        vec![
            publication_leaf(3, &prior_recovery, route_table_digest, 40, 7, 1),
            publication_leaf(4, &prior_recovery, route_table_digest, 40, 7, 2),
        ],
    )?;
    let later = publication(
        41,
        route_table_digest,
        prior.public_root_v1()?,
        vec![
            publication_leaf(3, &later_recovery, route_table_digest, 41, 8, 1),
            publication_leaf(4, &later_recovery, route_table_digest, 41, 8, 2),
        ],
    )?;

    later
        .check_monotonic_successor_v1(&prior)
        .expect("durable successor publication must remain monotonic");

    let prior_root = prior.public_root_v1()?;
    let later_root = later.public_root_v1()?;
    assert_ne!(prior_root, later_root);
    assert_eq!(later.prior_public_root, prior_root);
    let prior_root_hex = to_hex(&prior_root.into_bytes());
    let later_root_hex = to_hex(&later_root.into_bytes());
    assert_eq!(
        prior_root_hex,
        "d0b3cf4bee78a86f5056f9691a77880c77e2a9b61313569e3509b1a85d0f1633"
    );
    assert_eq!(
        later_root_hex,
        "7517087833a6f97609352beb5e117a96407411c6a67178886ca3d6f12f092c2b"
    );

    drop(guard);
    Ok(())
}

#[test]
fn test_ckpt_drift_pre_publish() -> Result<(), Box<dyn std::error::Error>> {
    let guard = test_env_lock().lock().expect("env lock");
    std::env::remove_var(TEST_HJMT_INJ_STAGE_ENV);
    let temp = tempdir()?;

    let pending_path = path(39, 15, 1);
    let mut seed =
        SettlementStore::load_with_backend_mode(temp.path(), SettlementBackendMode::Hjmt)?;
    seed.put_settlement_item(item(pending_path, 3_901))?;
    drop(seed);

    std::env::set_var(TEST_HJMT_INJ_STAGE_ENV, "parents");
    let mut store =
        SettlementStore::load_with_backend_mode(temp.path(), SettlementBackendMode::Hjmt)?;
    let result = store.apply_attested_settlement_ops(
        vec![StoreOp::Put(Box::new(item(pending_path, 4_002)))],
        vec![exec_tx(pending_path, 4_002, b"pending-checkpoint-drift")],
    );
    std::env::remove_var(TEST_HJMT_INJ_STAGE_ENV);
    let err = result.expect_err("parent-stage injection must fail");
    assert!(
        err.to_string()
            .contains("hjmt journal injection after ParentsCommitted"),
        "{err}"
    );
    drop(store);

    let pending = pending_meta(temp.path(), 2);
    remove_exec_row(temp.path(), pending.exec_id);
    assert_eq!(active_version(temp.path()), 1);

    let err =
        match SettlementStore::load_with_backend_mode(temp.path(), SettlementBackendMode::Hjmt) {
            Ok(_) => panic!("pending checkpoint drift must reject before publish"),
            Err(err) => err,
        };
    assert!(
        err.to_string()
            .contains("missing canonical exec row for checkpoint metadata"),
        "{err}"
    );
    assert_eq!(active_version(temp.path()), 1);

    drop(guard);
    Ok(())
}
