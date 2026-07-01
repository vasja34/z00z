use std::sync::{Mutex, OnceLock};

use redb::{Database, ReadableTable, TableDefinition};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use tempfile::tempdir;
use z00z_core::assets::AssetLeaf;
use z00z_crypto::{expert::hash_domain, hash_zk::hash_zk};
use z00z_storage::settlement::{
    AdaptiveProofErr, BucketEpoch, BucketId, BucketPolicy, ClaimNullRec, DefinitionId,
    FeeReplayRec, HjmtProofFamily, MergeProof, PolicyTransitionProof, RightClass, RightLeaf,
    SerialId, SettlementLeafFamily, SettlementPath, SettlementStore, SettlementStoreError,
    SplitProof, StoreItem, TerminalId, TerminalLeaf,
};
use z00z_utils::codec::{BincodeCodec, Codec};

const META_TABLE: TableDefinition<&[u8], &[u8]> = TableDefinition::new("settlement_meta");
const AST_ROW_TABLE: TableDefinition<&[u8], &[u8]> = TableDefinition::new("settlement_rows");
const PATH_ROW_TABLE: TableDefinition<&[u8], &[u8]> = TableDefinition::new("settlement_paths");
const CLAIM_NULL_TABLE: TableDefinition<&[u8], &[u8]> =
    TableDefinition::new("settlement_claim_nulls");
const FEE_REPLAY_TABLE: TableDefinition<&[u8], &[u8]> =
    TableDefinition::new("settlement_fee_replays");
const SNAP_TABLE: TableDefinition<&[u8], &[u8]> = TableDefinition::new("settlement_cp_snaps");
const DRAFT_TABLE: TableDefinition<&[u8], &[u8]> = TableDefinition::new("settlement_cp_drafts");
const CHECK_TABLE: TableDefinition<&[u8], &[u8]> = TableDefinition::new("settlement_cp_checks");
const EXEC_TABLE: TableDefinition<&[u8], &[u8]> = TableDefinition::new("settlement_cp_execs");
const LINK_TABLE: TableDefinition<&[u8], &[u8]> = TableDefinition::new("settlement_cp_links");
const HJMT_TERMINAL_ROW_TABLE: TableDefinition<&[u8], &[u8]> =
    TableDefinition::new("settlement_hjmt_terminal_rows");
const HJMT_SETTLEMENT_PATH_TABLE: TableDefinition<&[u8], &[u8]> =
    TableDefinition::new("settlement_hjmt_settlement_path_rows");
const HJMT_JOURNAL_TABLE: TableDefinition<&[u8], &[u8]> =
    TableDefinition::new("settlement_hjmt_journal");
const HJMT_ROOT_TABLE: TableDefinition<&[u8], &[u8]> =
    TableDefinition::new("settlement_hjmt_roots");
const DB_FILE: &str = "settlement_state.redb";
const BACKEND_ENV: &str = "Z00Z_SETTLEMENT_BACKEND_MODE";
const BUCKET_BITS_ENV: &str = "Z00Z_SETTLEMENT_BUCKET_BITS";
const KEY_STATE: &[u8] = b"state_meta";

hash_domain!(TestHjmtJournalDom, "z00z.storage.asset.hjmt.journal.v1", 1);

type HjmtBucketKey = (DefinitionId, SerialId, BucketId);

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
struct StateMetaWire {
    version: u64,
    state_root: [u8; 32],
    flat_root: [u8; 32],
    snap_id: [u8; 32],
    draft_id: [u8; 32],
    check_id: [u8; 32],
    exec_id: [u8; 32],
    def_root: Option<[u8; 32]>,
    fee_replay_count: u64,
    fee_replay_digest: [u8; 32],
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum HjmtCommitStatus {
    Prepared,
    ChildrenCommitted,
    ParentsCommitted,
    RootPublished,
}

impl HjmtCommitStatus {
    fn rank(self) -> u8 {
        match self {
            Self::Prepared => 0,
            Self::ChildrenCommitted => 1,
            Self::ParentsCommitted => 2,
            Self::RootPublished => 3,
        }
    }

    fn from_rank(rank: u8) -> Self {
        match rank {
            0 => Self::Prepared,
            1 => Self::ChildrenCommitted,
            2 => Self::ParentsCommitted,
            3 => Self::RootPublished,
            other => panic!("unsupported journal rank {other}"),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct HjmtJournalWire {
    version: u64,
    bucket_epoch: u64,
    bucket_policy_id: [u8; 32],
    root_generation: u8,
    proof_version: u16,
    previous_semantic_state_root: [u8; 32],
    next_semantic_state_root: [u8; 32],
    touched_definitions: Vec<DefinitionId>,
    touched_serials: Vec<(DefinitionId, SerialId)>,
    touched_buckets: Vec<HjmtBucketKey>,
    fee_replay_count: u64,
    fee_replay_digest: [u8; 32],
    fee_replay_digests: Vec<[u8; 32]>,
    child_commit_digest: [u8; 32],
    parent_commit_digest: [u8; 32],
    status: HjmtCommitStatus,
}

fn env_lock() -> &'static Mutex<()> {
    static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    LOCK.get_or_init(|| Mutex::new(()))
}

fn db(root: &std::path::Path) -> Database {
    Database::create(root.join(DB_FILE)).expect("open db")
}

fn table_len(root: &std::path::Path, table_def: TableDefinition<&[u8], &[u8]>) -> usize {
    let db = db(root);
    let read = db.begin_read().expect("begin read");
    match read.open_table(table_def) {
        Ok(table) => table.iter().expect("iterate table").count(),
        Err(redb::TableError::TableDoesNotExist(_)) => 0,
        Err(err) => panic!("open table: {err}"),
    }
}

fn bytes(value: u8) -> [u8; 32] {
    [value; 32]
}

fn empty_fee_replay_digest() -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(0u64.to_be_bytes());
    hasher.finalize().into()
}

fn test_item(mark: u8) -> StoreItem {
    let core = AssetLeaf::dummy_for_scan(u32::from(mark));
    let leaf = TerminalLeaf::from(core.clone());
    let path = SettlementPath::new(
        DefinitionId::new(bytes(mark)),
        SerialId::new(core.serial_id),
        TerminalId::new(core.asset_id),
    );
    StoreItem::new(path, leaf).expect("test item")
}

fn item_for_path(path: SettlementPath) -> StoreItem {
    let mut core = AssetLeaf::dummy_for_scan(path.serial_id.get());
    core.asset_id = path.terminal_id().into_bytes();
    let leaf = TerminalLeaf::from(core);
    StoreItem::new(path, leaf).expect("path test item")
}

fn sibling_path_same_bucket(
    store: &SettlementStore,
    base: SettlementPath,
    start_mark: u8,
) -> SettlementPath {
    let target_bucket = store.bucket_policy().derive_bucket_id(base);
    for mark in start_mark..=u8::MAX {
        let candidate = SettlementPath::new(
            base.definition_id,
            base.serial_id,
            TerminalId::new(bytes(mark)),
        );
        if candidate != base && store.bucket_policy().derive_bucket_id(candidate) == target_bucket {
            return candidate;
        }
    }
    panic!("missing same-bucket sibling path fixture");
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

fn right_item(mark: u8) -> StoreItem {
    StoreItem::new(right_path(mark), right_leaf(mark)).expect("right test item")
}

fn asset_path(definition: u8, serial: u32, asset: u8) -> SettlementPath {
    SettlementPath::new(
        DefinitionId::new(bytes(definition)),
        SerialId::new(serial),
        TerminalId::new(bytes(asset)),
    )
}

fn put_item(store: &mut SettlementStore, path: SettlementPath) {
    store
        .put_settlement_item(item_for_path(path))
        .expect("put settlement item");
}

fn split_ready_count(store: &SettlementStore) -> usize {
    usize::try_from(store.bucket_policy().min_bucket_count()).expect("usize") + 1
}

fn split_ready_paths(store: &mut SettlementStore) -> Vec<SettlementPath> {
    let policy = store.bucket_policy();
    let first = asset_path(41, 9, 1);
    let target_bucket = first.bucket_id(policy);
    let needed = split_ready_count(store);
    let mut selected = vec![first];
    for seed in 2..=255 {
        let candidate = asset_path(41, 9, seed);
        if candidate.bucket_id(policy) == target_bucket {
            selected.push(candidate);
            if selected.len() == needed {
                break;
            }
        }
    }

    assert_eq!(
        selected.len(),
        needed,
        "failed to find same-bucket split paths"
    );
    for candidate in &selected {
        put_item(store, *candidate);
    }
    assert!(
        store.split_proof(&selected[0]).is_ok(),
        "split-ready fixture must build a split proof"
    );
    selected
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
        let candidate = asset_path(33, 11, seed);
        put_item(store, candidate);
        let bucket = candidate.bucket_id(store.bucket_policy());
        let sibling = sibling_bucket_id(bucket, bucket_bits);
        if let Some(other) = first_paths.get(&sibling).copied() {
            if store.merge_proof(&other, &candidate).is_ok() {
                return (other, candidate);
            }
        }
        first_paths.entry(bucket).or_insert(candidate);
    }

    panic!("failed to find sibling bucket pair");
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

fn legacy_asset_row_key(version: u64, path: SettlementPath) -> Vec<u8> {
    let mut key = Vec::with_capacity(76);
    key.extend_from_slice(&version.to_be_bytes());
    key.extend_from_slice(path.definition_id.as_bytes());
    key.extend_from_slice(&path.serial_id.get().to_be_bytes());
    key.extend_from_slice(path.terminal_id().as_bytes());
    key
}

fn legacy_path_row_key(version: u64, path: SettlementPath) -> Vec<u8> {
    let mut key = Vec::with_capacity(40);
    key.extend_from_slice(&version.to_be_bytes());
    key.extend_from_slice(path.terminal_id().as_bytes());
    key
}

fn seed_legacy_only_state(root: &std::path::Path) {
    let codec = BincodeCodec;
    let legacy_path = asset_path(7, 1, 7);
    let meta = StateMetaWire {
        version: 1,
        state_root: bytes(7),
        flat_root: bytes(8),
        snap_id: [0u8; 32],
        draft_id: [0u8; 32],
        check_id: [0u8; 32],
        exec_id: [0u8; 32],
        def_root: None,
        fee_replay_count: 0,
        fee_replay_digest: empty_fee_replay_digest(),
    };
    let meta_bytes = codec.serialize(&meta).expect("serialize state meta");

    let db = db(root);
    let write = db.begin_write().expect("begin write");
    {
        let mut meta_table = write.open_table(META_TABLE).expect("meta table");
        meta_table
            .insert(KEY_STATE, meta_bytes.as_slice())
            .expect("state meta row");
    }
    {
        let mut settlement_rows = write.open_table(AST_ROW_TABLE).expect("asset row table");
        let asset_payload = b"legacy-asset";
        settlement_rows
            .insert(
                legacy_asset_row_key(1, legacy_path).as_slice(),
                asset_payload.as_slice(),
            )
            .expect("legacy asset row");
        drop(settlement_rows);

        let mut path_rows = write.open_table(PATH_ROW_TABLE).expect("path row table");
        let path_payload = b"legacy-path";
        path_rows
            .insert(
                legacy_path_row_key(1, legacy_path).as_slice(),
                path_payload.as_slice(),
            )
            .expect("legacy path row");
        drop(path_rows);

        let _ = write.open_table(CLAIM_NULL_TABLE).expect("claim row table");
        let _ = write
            .open_table(FEE_REPLAY_TABLE)
            .expect("fee replay table");
    }
    write.commit().expect("commit legacy-only state");
}

fn corrupt_state_flat_root(root: &std::path::Path) {
    let codec = BincodeCodec;
    let db = db(root);
    let write = db.begin_write().expect("begin write");
    {
        let mut table = write.open_table(META_TABLE).expect("meta table");
        let mut meta: StateMetaWire = codec
            .deserialize(
                table
                    .get(KEY_STATE)
                    .expect("meta get")
                    .expect("meta row")
                    .value(),
            )
            .expect("deserialize meta");
        meta.flat_root[0] ^= 0xFF;
        let bytes = codec.serialize(&meta).expect("serialize meta");
        table
            .insert(KEY_STATE, bytes.as_slice())
            .expect("rewrite meta");
    }
    write.commit().expect("commit meta tamper");
}

fn load_state_meta(root: &std::path::Path) -> StateMetaWire {
    let codec = BincodeCodec;
    let db = db(root);
    let read = db.begin_read().expect("begin read");
    let table = read.open_table(META_TABLE).expect("meta table");
    codec
        .deserialize(
            table
                .get(KEY_STATE)
                .expect("meta get")
                .expect("meta row")
                .value(),
        )
        .expect("deserialize state meta")
}

fn decode_hjmt_terminal_key(key: &[u8], version: u64) -> Option<(SettlementPath, BucketId)> {
    if key.len() != 108 {
        return None;
    }

    let mut ver = [0u8; 8];
    ver.copy_from_slice(&key[..8]);
    if u64::from_be_bytes(ver) != version {
        return None;
    }

    let mut definition = [0u8; 32];
    definition.copy_from_slice(&key[8..40]);
    let mut serial = [0u8; 4];
    serial.copy_from_slice(&key[40..44]);
    let mut bucket = [0u8; 32];
    bucket.copy_from_slice(&key[44..76]);
    let mut terminal = [0u8; 32];
    terminal.copy_from_slice(&key[76..108]);

    Some((
        SettlementPath::new(
            DefinitionId::new(definition),
            SerialId::new(u32::from_be_bytes(serial)),
            TerminalId::new(terminal),
        ),
        BucketId::new(bucket),
    ))
}

fn decode_hjmt_settlement_path_key(key: &[u8], version: u64) -> Option<TerminalId> {
    if key.len() != 40 {
        return None;
    }

    let mut ver = [0u8; 8];
    ver.copy_from_slice(&key[..8]);
    if u64::from_be_bytes(ver) != version {
        return None;
    }

    let mut terminal = [0u8; 32];
    terminal.copy_from_slice(&key[8..40]);
    Some(TerminalId::new(terminal))
}

fn put_u32(out: &mut Vec<u8>, len: usize) {
    let len = u32::try_from(len).expect("u32 len");
    out.extend_from_slice(&len.to_be_bytes());
}

fn take_u32(bytes: &[u8], pos: &mut usize) -> usize {
    let end = pos.saturating_add(4);
    let mut raw = [0u8; 4];
    raw.copy_from_slice(&bytes[*pos..end]);
    *pos = end;
    u32::from_be_bytes(raw) as usize
}

fn take_32(bytes: &[u8], pos: &mut usize) -> [u8; 32] {
    let end = pos.saturating_add(32);
    let mut raw = [0u8; 32];
    raw.copy_from_slice(&bytes[*pos..end]);
    *pos = end;
    raw
}

fn take_serial(bytes: &[u8], pos: &mut usize) -> SerialId {
    let end = pos.saturating_add(4);
    let mut raw = [0u8; 4];
    raw.copy_from_slice(&bytes[*pos..end]);
    *pos = end;
    SerialId::new(u32::from_be_bytes(raw))
}

fn take_definitions(bytes: &[u8], pos: &mut usize) -> Vec<DefinitionId> {
    let count = take_u32(bytes, pos);
    let mut out = Vec::with_capacity(count);
    for _ in 0..count {
        out.push(DefinitionId::new(take_32(bytes, pos)));
    }
    out
}

fn take_serials(bytes: &[u8], pos: &mut usize) -> Vec<(DefinitionId, SerialId)> {
    let count = take_u32(bytes, pos);
    let mut out = Vec::with_capacity(count);
    for _ in 0..count {
        out.push((
            DefinitionId::new(take_32(bytes, pos)),
            take_serial(bytes, pos),
        ));
    }
    out
}

fn take_buckets(bytes: &[u8], pos: &mut usize) -> Vec<HjmtBucketKey> {
    let count = take_u32(bytes, pos);
    let mut out = Vec::with_capacity(count);
    for _ in 0..count {
        out.push((
            DefinitionId::new(take_32(bytes, pos)),
            take_serial(bytes, pos),
            BucketId::new(take_32(bytes, pos)),
        ));
    }
    out
}

fn take_fee_replay_digests(bytes: &[u8], pos: &mut usize) -> Vec<[u8; 32]> {
    let count = take_u32(bytes, pos);
    let mut out = Vec::with_capacity(count);
    for _ in 0..count {
        out.push(take_32(bytes, pos));
    }
    out
}

fn encode_journal(entry: &HjmtJournalWire) -> Vec<u8> {
    let mut out = Vec::with_capacity(220);
    out.extend_from_slice(&entry.version.to_be_bytes());
    out.extend_from_slice(&entry.bucket_epoch.to_be_bytes());
    out.extend_from_slice(&entry.bucket_policy_id);
    out.push(entry.root_generation);
    out.extend_from_slice(&entry.proof_version.to_be_bytes());
    out.extend_from_slice(&entry.previous_semantic_state_root);
    out.extend_from_slice(&entry.next_semantic_state_root);
    out.extend_from_slice(&entry.child_commit_digest);
    out.extend_from_slice(&entry.parent_commit_digest);
    out.extend_from_slice(&entry.fee_replay_count.to_be_bytes());
    out.extend_from_slice(&entry.fee_replay_digest);
    out.push(entry.status.rank());
    put_u32(&mut out, entry.touched_definitions.len());
    for definition_id in &entry.touched_definitions {
        out.extend_from_slice(definition_id.as_bytes());
    }
    put_u32(&mut out, entry.touched_serials.len());
    for (definition_id, serial_id) in &entry.touched_serials {
        out.extend_from_slice(definition_id.as_bytes());
        out.extend_from_slice(&serial_id.get().to_be_bytes());
    }
    put_u32(&mut out, entry.touched_buckets.len());
    for (definition_id, serial_id, bucket_id) in &entry.touched_buckets {
        out.extend_from_slice(definition_id.as_bytes());
        out.extend_from_slice(&serial_id.get().to_be_bytes());
        out.extend_from_slice(bucket_id.as_bytes());
    }
    put_u32(&mut out, entry.fee_replay_digests.len());
    for digest in &entry.fee_replay_digests {
        out.extend_from_slice(digest);
    }
    out
}

fn decode_journal(bytes: &[u8]) -> HjmtJournalWire {
    let mut version = [0u8; 8];
    version.copy_from_slice(&bytes[..8]);
    let mut bucket_epoch = [0u8; 8];
    bucket_epoch.copy_from_slice(&bytes[8..16]);
    let mut bucket_policy_id = [0u8; 32];
    bucket_policy_id.copy_from_slice(&bytes[16..48]);
    let root_generation = bytes[48];
    let mut proof_version = [0u8; 2];
    proof_version.copy_from_slice(&bytes[49..51]);
    let mut previous_root = [0u8; 32];
    previous_root.copy_from_slice(&bytes[51..83]);
    let mut next_root = [0u8; 32];
    next_root.copy_from_slice(&bytes[83..115]);
    let mut child_digest = [0u8; 32];
    child_digest.copy_from_slice(&bytes[115..147]);
    let mut parent_digest = [0u8; 32];
    parent_digest.copy_from_slice(&bytes[147..179]);
    let mut fee_replay_count = [0u8; 8];
    fee_replay_count.copy_from_slice(&bytes[179..187]);
    let mut fee_replay_digest = [0u8; 32];
    fee_replay_digest.copy_from_slice(&bytes[187..219]);

    let mut pos = 220;
    let touched_definitions = take_definitions(bytes, &mut pos);
    let touched_serials = take_serials(bytes, &mut pos);
    let touched_buckets = take_buckets(bytes, &mut pos);
    let fee_replay_digests = take_fee_replay_digests(bytes, &mut pos);

    HjmtJournalWire {
        version: u64::from_be_bytes(version),
        bucket_epoch: u64::from_be_bytes(bucket_epoch),
        bucket_policy_id,
        root_generation,
        proof_version: u16::from_be_bytes(proof_version),
        previous_semantic_state_root: previous_root,
        next_semantic_state_root: next_root,
        touched_definitions,
        touched_serials,
        touched_buckets,
        fee_replay_count: u64::from_be_bytes(fee_replay_count),
        fee_replay_digest,
        fee_replay_digests,
        child_commit_digest: child_digest,
        parent_commit_digest: parent_digest,
        status: HjmtCommitStatus::from_rank(bytes[219]),
    }
}

fn load_terminal_rows(
    root: &std::path::Path,
    version: u64,
) -> Vec<(SettlementPath, BucketId, Vec<u8>)> {
    let db = db(root);
    let read = db.begin_read().expect("begin read");
    let table = read
        .open_table(HJMT_TERMINAL_ROW_TABLE)
        .expect("terminal table");
    let mut rows = Vec::new();
    for entry in table.iter().expect("terminal iter") {
        let (key, value) = entry.expect("terminal entry");
        if let Some((path, bucket_id)) = decode_hjmt_terminal_key(key.value(), version) {
            rows.push((path, bucket_id, value.value().to_vec()));
        }
    }
    rows
}

fn load_settlement_path_rows(
    root: &std::path::Path,
    version: u64,
) -> Vec<(SettlementPath, Vec<u8>)> {
    let codec = BincodeCodec;
    let db = db(root);
    let read = db.begin_read().expect("begin read");
    let table = read
        .open_table(HJMT_SETTLEMENT_PATH_TABLE)
        .expect("settlement path table");
    let mut rows = Vec::new();
    for entry in table.iter().expect("path iter") {
        let (key, value) = entry.expect("path entry");
        let Some(terminal_id) = decode_hjmt_settlement_path_key(key.value(), version) else {
            continue;
        };
        let path: SettlementPath = codec
            .deserialize(value.value())
            .expect("deserialize settlement path");
        assert_eq!(path.terminal_id, terminal_id);
        rows.push((path, value.value().to_vec()));
    }
    rows
}

fn load_child_root_rows(root: &std::path::Path, version: u64) -> Vec<(Vec<u8>, [u8; 32])> {
    let db = db(root);
    let read = db.begin_read().expect("begin read");
    let table = read.open_table(HJMT_ROOT_TABLE).expect("root table");
    let mut rows = Vec::new();
    for entry in table.iter().expect("root iter") {
        let (key, value) = entry.expect("root entry");
        let key = key.value().to_vec();
        if !key.starts_with(&version.to_be_bytes()) || key.get(8).copied() != Some(4) {
            continue;
        }
        let mut root = [0u8; 32];
        root.copy_from_slice(value.value());
        rows.push((key, root));
    }
    rows
}

fn hjmt_child_digest_for_test(
    terminal_rows: &[(SettlementPath, BucketId, Vec<u8>)],
    settlement_path_rows: &[(SettlementPath, Vec<u8>)],
    child_root_rows: &[(Vec<u8>, [u8; 32])],
) -> [u8; 32] {
    let mut terminal_rows = terminal_rows.to_vec();
    let mut settlement_path_rows = settlement_path_rows.to_vec();
    let mut child_root_rows = child_root_rows.to_vec();
    let claim_rows: Vec<ClaimNullRec> = Vec::new();
    let fee_rows: Vec<FeeReplayRec> = Vec::new();
    terminal_rows.sort_by_key(|(path, bucket_id, _)| (*path, *bucket_id));
    settlement_path_rows.sort_by_key(|(path, _)| *path);
    child_root_rows.sort_by(|left, right| left.0.cmp(&right.0));

    let codec = BincodeCodec;
    let payload = codec
        .serialize(&(
            terminal_rows,
            settlement_path_rows,
            claim_rows,
            fee_rows,
            child_root_rows,
        ))
        .expect("serialize child digest payload");
    hash_zk::<TestHjmtJournalDom>("children", &[payload.as_slice()])
}

fn refresh_child_digest(root: &std::path::Path, version: u64) {
    let terminal_rows = load_terminal_rows(root, version);
    let settlement_path_rows = load_settlement_path_rows(root, version);
    let child_root_rows = load_child_root_rows(root, version);
    let digest =
        hjmt_child_digest_for_test(&terminal_rows, &settlement_path_rows, &child_root_rows);

    let db = db(root);
    let write = db.begin_write().expect("begin write");
    {
        let mut table = write.open_table(HJMT_JOURNAL_TABLE).expect("journal table");
        let entry_bytes = table
            .get(version.to_be_bytes().as_slice())
            .expect("journal get")
            .expect("journal row")
            .value()
            .to_vec();
        let mut journal = decode_journal(&entry_bytes);
        journal.child_commit_digest = digest;
        let bytes = encode_journal(&journal);
        table
            .insert(version.to_be_bytes().as_slice(), bytes.as_slice())
            .expect("journal update");
    }
    write.commit().expect("commit journal update");
}

fn corrupt_first_terminal_bucket(root: &std::path::Path) -> u64 {
    let db = db(root);
    let write = db.begin_write().expect("begin write");
    let version;
    {
        let mut table = write
            .open_table(HJMT_TERMINAL_ROW_TABLE)
            .expect("terminal table");
        let (old_key, value) = {
            let entry = table
                .iter()
                .expect("terminal iter")
                .next()
                .expect("terminal row")
                .expect("terminal entry");
            (entry.0.value().to_vec(), entry.1.value().to_vec())
        };
        version = u64::from_be_bytes(old_key[..8].try_into().expect("version bytes"));
        let mut new_key = old_key.clone();
        new_key[44] ^= 1;
        table
            .remove(old_key.as_slice())
            .expect("remove terminal row");
        table
            .insert(new_key.as_slice(), value.as_slice())
            .expect("insert corrupted terminal row");
    }
    write.commit().expect("commit terminal corruption");
    version
}

fn drop_version_terminal_row(root: &std::path::Path, version: u64) {
    let db = db(root);
    let write = db.begin_write().expect("begin write");
    {
        let mut table = write
            .open_table(HJMT_TERMINAL_ROW_TABLE)
            .expect("terminal table");
        let key = table
            .iter()
            .expect("terminal iter")
            .find_map(|entry| {
                let (key, _) = entry.expect("terminal entry");
                let key_bytes = key.value().to_vec();
                let row_version =
                    u64::from_be_bytes(key_bytes[..8].try_into().expect("version bytes"));
                (row_version == version).then_some(key_bytes)
            })
            .expect("terminal row for version");
        table.remove(key.as_slice()).expect("remove terminal row");
    }
    write.commit().expect("commit terminal row removal");
}

fn published_versions(root: &std::path::Path) -> Vec<u64> {
    let db = db(root);
    let read = db.begin_read().expect("begin read");
    let table = read.open_table(HJMT_JOURNAL_TABLE).expect("journal table");
    let mut versions = Vec::new();
    for entry in table.iter().expect("journal iter") {
        let (_, value) = entry.expect("journal entry");
        let journal = decode_journal(value.value());
        if journal.status == HjmtCommitStatus::RootPublished {
            versions.push(journal.version);
        }
    }
    versions.sort_unstable();
    versions
}

fn drop_first_settlement_path_row(root: &std::path::Path) -> u64 {
    let db = db(root);
    let write = db.begin_write().expect("begin write");
    let version;
    {
        let mut table = write
            .open_table(HJMT_SETTLEMENT_PATH_TABLE)
            .expect("settlement path table");
        let key = {
            let entry = table
                .iter()
                .expect("path iter")
                .next()
                .expect("path row")
                .expect("path entry");
            entry.0.value().to_vec()
        };
        version = u64::from_be_bytes(key[..8].try_into().expect("version bytes"));
        table.remove(key.as_slice()).expect("remove path row");
    }
    write.commit().expect("commit path row removal");
    version
}

#[test]
fn test_reload_uses_settle_rows() -> Result<(), Box<dyn std::error::Error>> {
    let guard = env_lock().lock().expect("env lock");
    let previous_mode = std::env::var(BACKEND_ENV).ok();
    std::env::set_var(BACKEND_ENV, "hjmt");

    let result = (|| -> Result<(), Box<dyn std::error::Error>> {
        let temp = tempdir()?;
        let first = test_item(41);
        let second = test_item(42);
        let first_path = first.path();
        let second_path = second.path();

        let mut store = SettlementStore::load(temp.path())?;
        let _ = store.put_settlement_item(first.clone())?;
        let root_after_put = store.put_settlement_item(second.clone())?;
        drop(store);

        assert_eq!(table_len(temp.path(), AST_ROW_TABLE), 0);
        assert_eq!(table_len(temp.path(), PATH_ROW_TABLE), 0);
        assert!(table_len(temp.path(), HJMT_TERMINAL_ROW_TABLE) > 0);
        assert!(table_len(temp.path(), HJMT_SETTLEMENT_PATH_TABLE) > 0);

        let mut reloaded = SettlementStore::load(temp.path())?;
        assert_eq!(
            reloaded
                .settlement_proof_blob(&second_path)?
                .item()
                .settlement_root(),
            root_after_put,
        );
        assert_eq!(
            reloaded.get_settlement_item(&second_path)?,
            Some(second.clone())
        );

        let root_after_delete = reloaded.del_settlement_item(&first_path)?;
        drop(reloaded);

        let final_store = SettlementStore::load(temp.path())?;
        assert_eq!(
            final_store
                .settlement_proof_blob(&second_path)?
                .item()
                .settlement_root(),
            root_after_delete,
        );
        assert_eq!(final_store.get_settlement_item(&first_path)?, None);
        assert_eq!(final_store.get_settlement_item(&second_path)?, Some(second));
        Ok(())
    })();

    if let Some(value) = previous_mode {
        std::env::set_var(BACKEND_ENV, value);
    } else {
        std::env::remove_var(BACKEND_ENV);
    }
    drop(guard);
    result
}

#[test]
fn test_plain_commit_skips_checkpoint() -> Result<(), Box<dyn std::error::Error>> {
    let guard = env_lock().lock().expect("env lock");
    let previous_mode = std::env::var(BACKEND_ENV).ok();
    std::env::set_var(BACKEND_ENV, "hjmt");

    let result = (|| -> Result<(), Box<dyn std::error::Error>> {
        let temp = tempdir()?;
        let first = test_item(51);
        let second = test_item(52);
        let second_path = second.path();

        let mut store = SettlementStore::load(temp.path())?;
        let _ = store.put_settlement_item(first)?;
        let root = store.put_settlement_item(second.clone())?;
        drop(store);

        let meta = load_state_meta(temp.path());
        assert_eq!(meta.snap_id, [0u8; 32]);
        assert_eq!(meta.draft_id, [0u8; 32]);
        assert_eq!(meta.check_id, [0u8; 32]);
        assert_eq!(meta.exec_id, [0u8; 32]);
        assert_eq!(table_len(temp.path(), SNAP_TABLE), 0);
        assert_eq!(table_len(temp.path(), DRAFT_TABLE), 0);
        assert_eq!(table_len(temp.path(), CHECK_TABLE), 0);
        assert_eq!(table_len(temp.path(), EXEC_TABLE), 0);
        assert_eq!(table_len(temp.path(), LINK_TABLE), 0);

        let reloaded = SettlementStore::load(temp.path())?;
        assert_eq!(reloaded.get_settlement_item(&second_path)?, Some(second));
        assert_eq!(reloaded.settlement_root()?, root);
        Ok(())
    })();

    if let Some(value) = previous_mode {
        std::env::set_var(BACKEND_ENV, value);
    } else {
        std::env::remove_var(BACKEND_ENV);
    }
    drop(guard);
    result
}

#[test]
fn test_reload_roundtrips_right_leaf() -> Result<(), Box<dyn std::error::Error>> {
    let guard = env_lock().lock().expect("env lock");
    let previous_mode = std::env::var(BACKEND_ENV).ok();
    std::env::set_var(BACKEND_ENV, "hjmt");

    let result = (|| -> Result<(), Box<dyn std::error::Error>> {
        let temp = tempdir()?;
        let item = right_item(61);
        let path = item.path();

        let mut store = SettlementStore::load(temp.path())?;
        let root_after_put = store.put_settlement_item(item.clone())?;
        drop(store);

        assert_eq!(table_len(temp.path(), AST_ROW_TABLE), 0);
        assert_eq!(table_len(temp.path(), PATH_ROW_TABLE), 0);
        assert!(table_len(temp.path(), HJMT_TERMINAL_ROW_TABLE) > 0);
        assert!(table_len(temp.path(), HJMT_SETTLEMENT_PATH_TABLE) > 0);

        let reloaded = SettlementStore::load(temp.path())?;
        assert_eq!(reloaded.get_settlement_item(&path)?, Some(item));
        assert_eq!(
            reloaded
                .settlement_proof_blob(&path)?
                .item()
                .settlement_root(),
            root_after_put,
        );
        Ok(())
    })();

    if let Some(value) = previous_mode {
        std::env::set_var(BACKEND_ENV, value);
    } else {
        std::env::remove_var(BACKEND_ENV);
    }
    drop(guard);
    result
}

#[test]
fn test_reload_del_proof_valid() -> Result<(), Box<dyn std::error::Error>> {
    let guard = env_lock().lock().expect("env lock");
    let previous_mode = std::env::var(BACKEND_ENV).ok();
    std::env::set_var(BACKEND_ENV, "hjmt");

    let result = (|| -> Result<(), Box<dyn std::error::Error>> {
        let temp = tempdir()?;
        let deleted_path = SettlementPath::new(
            DefinitionId::new(bytes(81)),
            SerialId::new(1),
            TerminalId::new(bytes(11)),
        );
        let deleted_item = item_for_path(deleted_path);

        let mut store = SettlementStore::load(temp.path())?;
        let _ = store.put_settlement_item(deleted_item.clone())?;
        let surviving_path = sibling_path_same_bucket(&store, deleted_path, 12);
        let surviving_item = item_for_path(surviving_path);
        let _ = store.put_settlement_item(surviving_item)?;
        let root_after_delete = store.del_settlement_item(&deleted_path)?;
        drop(store);

        let reloaded = SettlementStore::load(temp.path())?;
        let proof = reloaded.settlement_proof_blob(&deleted_path)?;

        assert_eq!(proof.hjmt_proof_family(), Some(HjmtProofFamily::Deletion));
        assert_eq!(proof.item().settlement_root(), root_after_delete);
        reloaded.validate_settlement_proof_blob(&proof)?;
        Ok(())
    })();

    if let Some(value) = previous_mode {
        std::env::set_var(BACKEND_ENV, value);
    } else {
        std::env::remove_var(BACKEND_ENV);
    }
    drop(guard);
    result
}

#[test]
fn test_reload_miss_proof_valid() -> Result<(), Box<dyn std::error::Error>> {
    let guard = env_lock().lock().expect("env lock");
    let previous_mode = std::env::var(BACKEND_ENV).ok();
    std::env::set_var(BACKEND_ENV, "hjmt");

    let result = (|| -> Result<(), Box<dyn std::error::Error>> {
        let temp = tempdir()?;
        let present_path = SettlementPath::new(
            DefinitionId::new(bytes(82)),
            SerialId::new(1),
            TerminalId::new(bytes(14)),
        );
        let missing_path = SettlementPath::new(
            DefinitionId::new(bytes(82)),
            SerialId::new(1),
            TerminalId::new(bytes(13)),
        );

        let mut store = SettlementStore::load(temp.path())?;
        let _ = store.put_settlement_item(item_for_path(present_path))?;
        drop(store);

        let reloaded = SettlementStore::load(temp.path())?;
        let absence = reloaded
            .settlement_nonexistence_proof_blob(&missing_path, SettlementLeafFamily::Terminal)?;
        assert_eq!(
            absence.hjmt_proof_family(),
            Some(HjmtProofFamily::NonExistence)
        );
        reloaded.validate_settlement_nonexistence_proof_blob(
            &absence,
            SettlementLeafFamily::Terminal,
        )?;
        drop(reloaded);

        let reopened = SettlementStore::load(temp.path())?;
        let reopened_absence = reopened
            .settlement_nonexistence_proof_blob(&missing_path, SettlementLeafFamily::Terminal)?;
        reopened.validate_settlement_nonexistence_proof_blob(
            &reopened_absence,
            SettlementLeafFamily::Terminal,
        )?;
        Ok(())
    })();

    if let Some(value) = previous_mode {
        std::env::set_var(BACKEND_ENV, value);
    } else {
        std::env::remove_var(BACKEND_ENV);
    }
    drop(guard);
    result
}

#[test]
fn test_rejects_corrupt_overwrite_history() -> Result<(), Box<dyn std::error::Error>> {
    let guard = env_lock().lock().expect("env lock");
    let previous_mode = std::env::var(BACKEND_ENV).ok();
    std::env::set_var(BACKEND_ENV, "hjmt");

    let result = (|| -> Result<(), Box<dyn std::error::Error>> {
        let temp = tempdir()?;
        let path = SettlementPath::new(
            DefinitionId::new(bytes(91)),
            SerialId::new(1),
            TerminalId::new(bytes(19)),
        );

        let first = item_for_path(path);
        let mut overwritten_core = AssetLeaf::dummy_for_scan(path.serial_id.get());
        overwritten_core.asset_id = path.terminal_id().into_bytes();
        let mut overwritten_leaf = TerminalLeaf::from(overwritten_core);
        overwritten_leaf.owner_tag[0] ^= 0x33;
        let overwritten = StoreItem::new(path, overwritten_leaf).expect("overwrite item");

        let mut store = SettlementStore::load(temp.path())?;
        let _ = store.put_settlement_item(first)?;
        let _ = store.put_settlement_item(overwritten)?;
        let _ = store.del_settlement_item(&path)?;
        drop(store);

        let versions = published_versions(temp.path());
        assert!(
            versions.len() >= 3,
            "expected put/overwrite/delete versions"
        );
        drop_version_terminal_row(temp.path(), versions[1]);

        let reloaded = SettlementStore::load(temp.path())?;
        let err = reloaded
            .settlement_proof_blob(&path)
            .expect_err("corrupt overwrite history must reject deletion proof generation");
        assert!(
            err.to_string()
                .contains("hjmt child commit digest mismatch"),
            "unexpected corruption error: {err}"
        );
        Ok(())
    })();

    if let Some(value) = previous_mode {
        std::env::set_var(BACKEND_ENV, value);
    } else {
        std::env::remove_var(BACKEND_ENV);
    }
    drop(guard);
    result
}

#[test]
fn test_reload_rejects_legacy_state() -> Result<(), Box<dyn std::error::Error>> {
    let guard = env_lock().lock().expect("env lock");
    let previous_mode = std::env::var(BACKEND_ENV).ok();
    std::env::set_var(BACKEND_ENV, "hjmt");

    let result = (|| -> Result<(), Box<dyn std::error::Error>> {
        let temp = tempdir()?;
        seed_legacy_only_state(temp.path());

        let err = match SettlementStore::load(temp.path()) {
            Ok(_) => panic!("legacy-only RedB state must reject"),
            Err(err) => err,
        };
        assert!(matches!(
            err,
            SettlementStoreError::UnsupportedGeneration(message)
                if message.contains("legacy simple-jmt rows")
        ));
        Ok(())
    })();

    if let Some(value) = previous_mode {
        std::env::set_var(BACKEND_ENV, value);
    } else {
        std::env::remove_var(BACKEND_ENV);
    }
    drop(guard);
    result
}

#[test]
fn test_flat_root_metadata_drift() -> Result<(), Box<dyn std::error::Error>> {
    let guard = env_lock().lock().expect("env lock");
    let previous_mode = std::env::var(BACKEND_ENV).ok();
    std::env::set_var(BACKEND_ENV, "hjmt");

    let result = (|| -> Result<(), Box<dyn std::error::Error>> {
        let temp = tempdir()?;
        let item = right_item(73);

        let mut store = SettlementStore::load(temp.path())?;
        let _ = store.put_settlement_item(item)?;
        drop(store);

        corrupt_state_flat_root(temp.path());

        let err = match SettlementStore::load(temp.path()) {
            Ok(_) => panic!("flat_root drift must reject"),
            Err(err) => err,
        };
        assert!(
            err.to_string()
                .contains("hjmt reload flat_root does not match persisted metadata"),
            "unexpected reload error: {err}"
        );
        Ok(())
    })();

    if let Some(value) = previous_mode {
        std::env::set_var(BACKEND_ENV, value);
    } else {
        std::env::remove_var(BACKEND_ENV);
    }
    drop(guard);
    result
}

#[test]
fn test_bucket_mismatch_post_reseal() -> Result<(), Box<dyn std::error::Error>> {
    let guard = env_lock().lock().expect("env lock");
    let previous_mode = std::env::var(BACKEND_ENV).ok();
    std::env::set_var(BACKEND_ENV, "hjmt");

    let result = (|| -> Result<(), Box<dyn std::error::Error>> {
        let temp = tempdir()?;
        let item = right_item(71);

        let mut store = SettlementStore::load(temp.path())?;
        let _ = store.put_settlement_item(item)?;
        drop(store);

        let version = corrupt_first_terminal_bucket(temp.path());
        refresh_child_digest(temp.path(), version);

        let err = match SettlementStore::load(temp.path()) {
            Ok(_) => panic!("bucket mismatch reload must reject"),
            Err(err) => err,
        };
        assert!(
            err.to_string()
                .contains("hjmt child commit digest mismatch")
                || err.to_string().contains(
                    "hjmt terminal row bucket id does not match committed settlement path"
                ),
            "unexpected reload error: {err}"
        );
        Ok(())
    })();

    if let Some(value) = previous_mode {
        std::env::set_var(BACKEND_ENV, value);
    } else {
        std::env::remove_var(BACKEND_ENV);
    }
    drop(guard);
    result
}

#[test]
fn test_path_drift_post_reseal() -> Result<(), Box<dyn std::error::Error>> {
    let guard = env_lock().lock().expect("env lock");
    let previous_mode = std::env::var(BACKEND_ENV).ok();
    std::env::set_var(BACKEND_ENV, "hjmt");

    let result = (|| -> Result<(), Box<dyn std::error::Error>> {
        let temp = tempdir()?;
        let item = test_item(72);

        let mut store = SettlementStore::load(temp.path())?;
        let _ = store.put_settlement_item(item)?;
        drop(store);

        let version = drop_first_settlement_path_row(temp.path());
        refresh_child_digest(temp.path(), version);

        let err = match SettlementStore::load(temp.path()) {
            Ok(_) => panic!("settlement path drift reload must reject"),
            Err(err) => err,
        };
        assert!(
            err.to_string()
                .contains("hjmt child commit digest mismatch")
                || err
                    .to_string()
                    .contains("hjmt settlement path index drift from committed terminal rows"),
            "unexpected reload error: {err}"
        );
        Ok(())
    })();

    if let Some(value) = previous_mode {
        std::env::set_var(BACKEND_ENV, value);
    } else {
        std::env::remove_var(BACKEND_ENV);
    }
    drop(guard);
    result
}

#[test]
fn test_hist_split_proof_valid() -> Result<(), Box<dyn std::error::Error>> {
    let guard = env_lock().lock().expect("env lock");
    let previous_mode = std::env::var(BACKEND_ENV).ok();
    let previous_bucket_bits = std::env::var(BUCKET_BITS_ENV).ok();
    std::env::set_var(BACKEND_ENV, "hjmt");
    std::env::set_var(BUCKET_BITS_ENV, "1");

    let result = (|| -> Result<(), Box<dyn std::error::Error>> {
        let temp = tempdir()?;
        let mut store = SettlementStore::load(temp.path())?;
        let split_paths = split_ready_paths(&mut store);
        let first = split_paths[0];
        let second = split_paths[1];
        let proof = store.split_proof(&first).expect("split proof");
        drop(store);

        let mut reloaded = SettlementStore::load(temp.path())?;
        reloaded
            .validate_split_proof(&proof)
            .expect("reloaded split proof validation");
        put_item(&mut reloaded, asset_path(98, 1, 201));
        reloaded
            .validate_split_proof(&proof)
            .expect("historical split proof after reload");

        let tampered_epoch = SplitProof {
            prior_epoch: BucketEpoch::new(proof.prior_epoch.get() + 1),
            ..proof
        };
        let err = reloaded
            .validate_split_proof(&tampered_epoch)
            .expect_err("wrong split epoch must reject");
        assert!(matches!(err, AdaptiveProofErr::WrongEpoch));
        assert_ne!(first, second);
        Ok(())
    })();

    if let Some(value) = previous_mode {
        std::env::set_var(BACKEND_ENV, value);
    } else {
        std::env::remove_var(BACKEND_ENV);
    }
    if let Some(value) = previous_bucket_bits {
        std::env::set_var(BUCKET_BITS_ENV, value);
    } else {
        std::env::remove_var(BUCKET_BITS_ENV);
    }
    drop(guard);
    result
}

#[test]
fn test_keeps_merge_xfer_valid() -> Result<(), Box<dyn std::error::Error>> {
    let guard = env_lock().lock().expect("env lock");
    let previous_mode = std::env::var(BACKEND_ENV).ok();
    let previous_bucket_bits = std::env::var(BUCKET_BITS_ENV).ok();
    std::env::set_var(BACKEND_ENV, "hjmt");
    std::env::set_var(BUCKET_BITS_ENV, "2");

    let result = (|| -> Result<(), Box<dyn std::error::Error>> {
        let temp = tempdir()?;
        let mut store = SettlementStore::load(temp.path())?;
        let (left, right) = sibling_bucket_pair(&mut store);
        let merge = store.merge_proof(&left, &right).expect("merge proof");
        let next_policy = next_policy(&store);
        let transition = store
            .policy_transition_proof(next_policy)
            .expect("policy transition proof");
        drop(store);

        let mut reloaded = SettlementStore::load(temp.path())?;
        reloaded
            .validate_merge_proof(&merge)
            .expect("reloaded merge proof validation");
        reloaded
            .validate_policy_transition_proof(&transition, next_policy)
            .expect("reloaded policy transition validation");

        put_item(&mut reloaded, asset_path(99, 2, 202));
        reloaded
            .validate_merge_proof(&merge)
            .expect("historical merge proof after reload");
        reloaded
            .validate_policy_transition_proof(&transition, next_policy)
            .expect("historical policy transition after reload");

        let tampered_merge = MergeProof {
            prior_epoch: BucketEpoch::new(merge.prior_epoch.get() + 1),
            ..merge
        };
        let err = reloaded
            .validate_merge_proof(&tampered_merge)
            .expect_err("wrong merge epoch must reject");
        assert!(matches!(err, AdaptiveProofErr::WrongEpoch));

        let tampered_transition = PolicyTransitionProof {
            prior_epoch: BucketEpoch::new(transition.prior_epoch.get() + 1),
            ..transition
        };
        let err = reloaded
            .validate_policy_transition_proof(&tampered_transition, next_policy)
            .expect_err("wrong transition epoch must reject");
        assert!(matches!(err, AdaptiveProofErr::WrongEpoch));
        Ok(())
    })();

    if let Some(value) = previous_mode {
        std::env::set_var(BACKEND_ENV, value);
    } else {
        std::env::remove_var(BACKEND_ENV);
    }
    if let Some(value) = previous_bucket_bits {
        std::env::set_var(BUCKET_BITS_ENV, value);
    } else {
        std::env::remove_var(BUCKET_BITS_ENV);
    }
    drop(guard);
    result
}
