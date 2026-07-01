use std::sync::{Mutex, MutexGuard, OnceLock};
use std::{collections::BTreeMap, path::Path};

use crate::settlement::{
    DefinitionId, DefinitionRootLeaf, FeeActorCtx, FeeEnvelope, FeeSupportCtx, HjmtProofFamily,
    RightAction, RightActionCtx, RightClass, RightLeaf, SerialId, SerialRootLeaf, SettlementLeaf,
    SettlementLeafFamily, SettlementListReq, SettlementPath, SettlementStateRoot, SettlementStore,
    SettlementStoreError, StoreItem, StoreOp, TerminalId, TerminalLeaf,
};
use sha2::{Digest, Sha256};
use tempfile::TempDir;
use z00z_core::assets::{AssetLeaf, AssetPackPlain};
use z00z_crypto::{
    expert::{hash_domain, traits::DomainSeparation},
    poseidon2_hash, ZkPackEncrypted,
};
use z00z_utils::{
    codec::{Codec, JsonCodec},
    config::{ConfigSource, EnvConfig},
    io::{read_file, read_to_string},
};

const BACKEND_ENV: &str = "Z00Z_SETTLEMENT_BACKEND_MODE";
const BUCKET_BITS_ENV: &str = "Z00Z_SETTLEMENT_BUCKET_BITS";
const SCHED_CPU_ENV: &str = "Z00Z_STORAGE_SCHED_CPU";
const SCHED_QUEUE_ENV: &str = "Z00Z_STORAGE_SCHED_QUEUE";
const PROOF_BATCH_MODE_ENV: &str = "Z00Z_STORAGE_PROOF_BATCH_MODE";
const FIXTURE_JSON: &str = include_str!("../../tests/fixtures/test_settlement_corpus_fixture.json");
const FIXTURE_SHA256: [u8; 32] = [
    0xa1, 0xd1, 0xa6, 0x61, 0x51, 0x62, 0xc4, 0x1c, 0x0d, 0x32, 0x5f, 0x61, 0x1d, 0x9f, 0x7b, 0x3a,
    0x0d, 0x8d, 0x98, 0x12, 0x3b, 0x91, 0x32, 0xf6, 0xe5, 0x91, 0x57, 0xc8, 0xdb, 0x2a, 0xbc, 0x8e,
];

hash_domain!(AssetDom, "z00z.storage.settlement", 1);
hash_domain!(SerialDom, "z00z.storage.serial", 1);
hash_domain!(DefDom, "z00z.storage.definition", 1);
hash_domain!(StateDom, "z00z.storage.state", 1);

#[derive(Clone, Debug, PartialEq, Eq, serde::Deserialize)]
pub struct Fixture {
    pub version: u32,
    pub network: String,
    pub assets: Vec<AssetSeed>,
    pub rights: Vec<RightSeed>,
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Deserialize)]
pub struct AssetSeed {
    pub label: String,
    pub definition_mark: u8,
    pub serial_id: u32,
    pub terminal_mark: u8,
    pub value: u64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FixtureRightClass {
    MachineCapability,
    DataAccess,
    ServiceEntitlement,
    ValidatorMandate,
    OneTimeUse,
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Deserialize)]
pub struct RightSeed {
    pub label: String,
    pub definition_mark: u8,
    pub serial_id: u32,
    pub terminal_mark: u8,
    pub right_class: FixtureRightClass,
}

pub fn load_fixture() -> Fixture {
    let digest: [u8; 32] = Sha256::digest(FIXTURE_JSON.as_bytes()).into();
    assert_eq!(digest, FIXTURE_SHA256, "settlement corpus fixture drifted",);
    JsonCodec
        .deserialize(FIXTURE_JSON.as_bytes())
        .expect("settlement fixture")
}

pub fn bytes(value: u8) -> [u8; 32] {
    [value; 32]
}

pub fn right_class(class: FixtureRightClass) -> RightClass {
    match class {
        FixtureRightClass::MachineCapability => RightClass::MachineCapability,
        FixtureRightClass::DataAccess => RightClass::DataAccess,
        FixtureRightClass::ServiceEntitlement => RightClass::ServiceEntitlement,
        FixtureRightClass::ValidatorMandate => RightClass::ValidatorMandate,
        FixtureRightClass::OneTimeUse => RightClass::OneTimeUse,
    }
}

pub fn asset_path(seed: &AssetSeed) -> SettlementPath {
    SettlementPath::new(
        DefinitionId::new(bytes(seed.definition_mark)),
        SerialId::new(seed.serial_id),
        TerminalId::new(bytes(seed.terminal_mark)),
    )
}

pub fn asset_leaf(seed: &AssetSeed) -> TerminalLeaf {
    let path = asset_path(seed);
    let payload = AssetPackPlain {
        value: seed.value,
        blinding: bytes(seed.terminal_mark.wrapping_add(41)),
        s_out: bytes(seed.terminal_mark.wrapping_add(42)),
    }
    .to_bytes();

    AssetLeaf {
        asset_id: path.terminal_id().into_bytes(),
        serial_id: seed.serial_id,
        r_pub: bytes(seed.terminal_mark.wrapping_add(1)),
        owner_tag: bytes(seed.terminal_mark.wrapping_add(2)),
        c_amount: bytes(seed.terminal_mark.wrapping_add(3)),
        enc_pack: ZkPackEncrypted {
            version: 1,
            ciphertext: payload,
            tag: [0u8; 16],
        },
        range_proof: vec![seed.terminal_mark; 8],
        tag16: u16::from(seed.terminal_mark),
    }
    .into()
}

pub fn asset_item(seed: &AssetSeed) -> StoreItem {
    StoreItem::new(asset_path(seed), asset_leaf(seed)).expect("asset item")
}

pub fn right_path(seed: &RightSeed) -> SettlementPath {
    SettlementPath::new(
        DefinitionId::new(bytes(seed.definition_mark)),
        SerialId::new(seed.serial_id),
        TerminalId::new(bytes(seed.terminal_mark)),
    )
}

pub fn right_leaf(seed: &RightSeed) -> RightLeaf {
    let mark = seed.terminal_mark;
    RightLeaf {
        version: 1,
        terminal_id: TerminalId::new(bytes(mark)),
        right_class: right_class(seed.right_class),
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

pub fn transferred_right_leaf(prior: RightLeaf, mark: u8) -> RightLeaf {
    let mut next = prior;
    next.holder_commitment = bytes(mark.wrapping_add(30));
    next.control_commitment = bytes(mark.wrapping_add(31));
    next.beneficiary_commitment = bytes(mark.wrapping_add(32));
    next.use_nonce = bytes(mark.wrapping_add(33));
    next
}

pub fn right_item(seed: &RightSeed) -> StoreItem {
    StoreItem::new(right_path(seed), SettlementLeaf::Right(right_leaf(seed))).expect("right item")
}

pub fn right_ctx(leaf: &RightLeaf, now: u64) -> RightActionCtx {
    RightActionCtx {
        now,
        expected_holder: Some(leaf.holder_commitment),
        expected_control: Some(leaf.control_commitment),
        ..RightActionCtx::default()
    }
}

pub fn fee_actor(mark: u8, now: u64) -> FeeActorCtx {
    FeeActorCtx {
        now,
        payer_commitment: Some(fee_bytes(mark, 40)),
        sponsor_commitment: None,
    }
}

pub fn fee_put_ops(
    path: SettlementPath,
    leaf: impl Into<SettlementLeaf>,
) -> Result<Vec<StoreOp>, SettlementStoreError> {
    Ok(vec![StoreOp::Put(Box::new(StoreItem::new(path, leaf)?))])
}

pub fn fee_del_ops(path: SettlementPath) -> Vec<StoreOp> {
    vec![StoreOp::Delete(path)]
}

pub fn fee_envelope(mark: u8, support: FeeSupportCtx) -> FeeEnvelope {
    let payer = fee_bytes(mark, 40);
    let support_ref = Some(fee_bytes(mark, 41));
    let budget_units = support.required_units.saturating_add(1);
    FeeEnvelope {
        version: 1,
        payer_commitment: payer,
        sponsor_commitment: [0u8; 32],
        budget_units,
        budget_commitment: FeeEnvelope::budget_bind(budget_units, support_ref),
        domain_id: support.domain_id,
        expires_at: 80,
        nonce: fee_bytes(mark, 42),
        transition_id: support.transition_id,
        replay_key: fee_bytes(mark, 43),
        support_ref,
        failure_policy_id: fee_bytes(mark, 44),
    }
}

fn fee_bytes(mark: u8, slot: u8) -> [u8; 32] {
    let mut out = [mark.wrapping_add(slot); 32];
    out[0] = mark;
    out[1] = slot;
    if out == [0u8; 32] {
        out[0] = 1;
    }
    out
}

pub fn list_items(store: &SettlementStore) -> Result<Vec<StoreItem>, SettlementStoreError> {
    Ok(store
        .list_settlement(SettlementListReq::all(512))?
        .items()
        .to_vec())
}

pub fn terminal_set(items: &[StoreItem]) -> Vec<TerminalId> {
    items.iter().map(|item| item.path().terminal_id).collect()
}

pub struct HjmtEnvGuard {
    _lock: MutexGuard<'static, ()>,
    prev: Option<String>,
    prev_bits: Option<String>,
    prev_sched_cpu: Option<String>,
    prev_sched_queue: Option<String>,
}

impl HjmtEnvGuard {
    pub fn new() -> Self {
        Self::with_bits_opt(None)
    }

    pub fn with_bits(bits: &str) -> Self {
        Self::with_bits_opt(Some(bits))
    }

    fn with_bits_opt(bits: Option<&str>) -> Self {
        let lock = env_lock()
            .lock()
            .unwrap_or_else(|poison| poison.into_inner());
        let prev = env_value(BACKEND_ENV);
        let prev_bits = env_value(BUCKET_BITS_ENV);
        let prev_sched_cpu = env_value(SCHED_CPU_ENV);
        let prev_sched_queue = env_value(SCHED_QUEUE_ENV);
        std::env::set_var(BACKEND_ENV, "hjmt");
        std::env::set_var(SCHED_CPU_ENV, "1");
        std::env::set_var(SCHED_QUEUE_ENV, "1024");
        if let Some(bits) = bits {
            std::env::set_var(BUCKET_BITS_ENV, bits);
        } else {
            std::env::remove_var(BUCKET_BITS_ENV);
        }
        Self {
            _lock: lock,
            prev,
            prev_bits,
            prev_sched_cpu,
            prev_sched_queue,
        }
    }
}

impl Default for HjmtEnvGuard {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for HjmtEnvGuard {
    fn drop(&mut self) {
        if let Some(value) = self.prev.take() {
            std::env::set_var(BACKEND_ENV, value);
        } else {
            std::env::remove_var(BACKEND_ENV);
        }
        if let Some(value) = self.prev_bits.take() {
            std::env::set_var(BUCKET_BITS_ENV, value);
        } else {
            std::env::remove_var(BUCKET_BITS_ENV);
        }
        if let Some(value) = self.prev_sched_cpu.take() {
            std::env::set_var(SCHED_CPU_ENV, value);
        } else {
            std::env::remove_var(SCHED_CPU_ENV);
        }
        if let Some(value) = self.prev_sched_queue.take() {
            std::env::set_var(SCHED_QUEUE_ENV, value);
        } else {
            std::env::remove_var(SCHED_QUEUE_ENV);
        }
    }
}

pub fn redb_store() -> Result<(HjmtEnvGuard, TempDir, SettlementStore), Box<dyn std::error::Error>>
{
    redb_store_with_bits(None)
}

pub fn redb_store_with_bits(
    bits: Option<&str>,
) -> Result<(HjmtEnvGuard, TempDir, SettlementStore), Box<dyn std::error::Error>> {
    let guard = match bits {
        Some(bits) => HjmtEnvGuard::with_bits(bits),
        None => HjmtEnvGuard::new(),
    };
    let temp = tempfile::tempdir()?;
    let store = SettlementStore::load(temp.path())?;
    Ok((guard, temp, store))
}

pub fn load_fixture_items(fixture: &Fixture) -> Vec<StoreItem> {
    let mut items = Vec::new();
    for seed in &fixture.assets {
        items.push(asset_item(seed));
    }
    for seed in &fixture.rights {
        items.push(right_item(seed));
    }
    items
}

pub struct SchedEnv {
    prev_cpu: Option<String>,
    prev_queue: Option<String>,
}

impl SchedEnv {
    pub fn new(cpu: usize, queue: usize) -> Self {
        let prev_cpu = env_value(SCHED_CPU_ENV);
        let prev_queue = env_value(SCHED_QUEUE_ENV);
        std::env::set_var(SCHED_CPU_ENV, cpu.to_string());
        std::env::set_var(SCHED_QUEUE_ENV, queue.to_string());
        Self {
            prev_cpu,
            prev_queue,
        }
    }
}

impl Drop for SchedEnv {
    fn drop(&mut self) {
        if let Some(value) = self.prev_cpu.take() {
            std::env::set_var(SCHED_CPU_ENV, value);
        } else {
            std::env::remove_var(SCHED_CPU_ENV);
        }
        if let Some(value) = self.prev_queue.take() {
            std::env::set_var(SCHED_QUEUE_ENV, value);
        } else {
            std::env::remove_var(SCHED_QUEUE_ENV);
        }
    }
}

pub struct ProofBatchModeEnv {
    prev_mode: Option<String>,
}

impl ProofBatchModeEnv {
    pub fn serial() -> Self {
        Self::set("serial")
    }

    pub fn parallel() -> Self {
        Self::set("parallel")
    }

    fn set(mode: &str) -> Self {
        let prev_mode = env_value(PROOF_BATCH_MODE_ENV);
        std::env::set_var(PROOF_BATCH_MODE_ENV, mode);
        Self { prev_mode }
    }
}

fn env_value(key: &str) -> Option<String> {
    EnvConfig.get(key).ok().flatten()
}

impl Drop for ProofBatchModeEnv {
    fn drop(&mut self) {
        if let Some(value) = self.prev_mode.take() {
            std::env::set_var(PROOF_BATCH_MODE_ENV, value);
        } else {
            std::env::remove_var(PROOF_BATCH_MODE_ENV);
        }
    }
}

pub fn asset_seed(definition: u8, serial: u32, terminal: u8, value: u64) -> AssetSeed {
    AssetSeed {
        label: format!("asset_{definition}_{serial}_{terminal}"),
        definition_mark: definition,
        serial_id: serial,
        terminal_mark: terminal,
        value,
    }
}

pub fn right_seed(
    definition: u8,
    serial: u32,
    terminal: u8,
    class: FixtureRightClass,
) -> RightSeed {
    RightSeed {
        label: format!("right_{definition}_{serial}_{terminal}"),
        definition_mark: definition,
        serial_id: serial,
        terminal_mark: terminal,
        right_class: class,
    }
}

pub fn put_ops(items: &[StoreItem]) -> Vec<StoreOp> {
    items
        .iter()
        .cloned()
        .map(|item| StoreOp::Put(Box::new(item)))
        .collect()
}

pub fn del_ops(items: &[StoreItem]) -> Vec<StoreOp> {
    items
        .iter()
        .map(|item| StoreOp::Delete(item.path()))
        .collect()
}

pub fn many_defs(count: usize) -> Vec<StoreItem> {
    (0..count)
        .map(|idx| {
            asset_item(&asset_seed(
                u8::try_from((idx % 200) + 1).expect("u8"),
                1,
                u8::try_from((idx % 200) + 1).expect("u8"),
                10_000 + idx as u64,
            ))
        })
        .collect()
}

pub fn many_sers(definition: u8, count: usize) -> Vec<StoreItem> {
    (0..count)
        .map(|idx| {
            asset_item(&asset_seed(
                definition,
                u32::try_from(idx + 1).expect("u32"),
                u8::try_from((idx % 120) + 121).expect("u8"),
                20_000 + idx as u64,
            ))
        })
        .collect()
}

pub fn hot_assets(definition: u8, serial: u32, count: usize) -> Vec<StoreItem> {
    (0..count)
        .map(|idx| {
            asset_item(&asset_seed(
                definition,
                serial,
                u8::try_from((idx % 200) + 1).expect("u8"),
                30_000 + idx as u64,
            ))
        })
        .collect()
}

pub fn hot_rights(
    definition: u8,
    serial: u32,
    count: usize,
    class: FixtureRightClass,
) -> Vec<StoreItem> {
    (0..count)
        .map(|idx| {
            right_item(&right_seed(
                definition,
                serial,
                u8::try_from((idx % 200) + 1).expect("u8"),
                class,
            ))
        })
        .collect()
}

pub fn mixed_items() -> Vec<StoreItem> {
    let mut items = hot_assets(61, 7, 16);
    items.extend((0..8).map(|idx| {
        right_item(&right_seed(
            61,
            7,
            u8::try_from(idx + 101).expect("u8"),
            FixtureRightClass::ServiceEntitlement,
        ))
    }));
    items
}

pub fn seed_mem(items: &[StoreItem]) -> SettlementStore {
    let _guard = HjmtEnvGuard::with_bits("2");
    let mut store = SettlementStore::new();
    if !items.is_empty() {
        store
            .apply_settlement_ops(put_ops(items))
            .expect("seed settlement ops");
    }
    store
}

pub fn seed_redb(bits: &str, items: &[StoreItem]) -> (HjmtEnvGuard, TempDir, SettlementStore) {
    let (guard, temp, mut store) = redb_store_with_bits(Some(bits)).expect("redb store");
    if !items.is_empty() {
        store
            .apply_settlement_ops(put_ops(items))
            .expect("seed redb settlement ops");
    }
    (guard, temp, store)
}

pub fn create_right_fee(
    store: &mut SettlementStore,
    seed: &RightSeed,
    mark: u8,
) -> Result<SettlementStateRoot, SettlementStoreError> {
    let path = right_path(seed);
    let leaf = right_leaf(seed);
    let env = fee_envelope(mark, store.fee_support_ctx(&fee_put_ops(path, leaf)?)?);
    store.create_right_with_fee(path, leaf, right_ctx(&leaf, 15), env, fee_actor(mark, 15))
}

pub fn transfer_right_fee(
    store: &mut SettlementStore,
    seed: &RightSeed,
    mark: u8,
) -> Result<SettlementStateRoot, SettlementStoreError> {
    let path = right_path(seed);
    let prior = store
        .get_settlement_item(&path)?
        .and_then(|item| item.right_leaf().ok().copied())
        .ok_or(SettlementStoreError::PathMiss)?;
    let next = transferred_right_leaf(prior, seed.terminal_mark);
    let env = fee_envelope(mark, store.fee_support_ctx(&fee_put_ops(path, next)?)?);
    store.transfer_right_with_fee(path, next, right_ctx(&next, 15), env, fee_actor(mark, 15))
}

pub fn revoke_right_fee(
    store: &mut SettlementStore,
    seed: &RightSeed,
    mark: u8,
) -> Result<SettlementStateRoot, SettlementStoreError> {
    let path = right_path(seed);
    let current = store
        .get_settlement_item(&path)?
        .and_then(|item| item.right_leaf().ok().copied())
        .ok_or(SettlementStoreError::PathMiss)?;
    let env = fee_envelope(mark, store.fee_support_ctx(&fee_del_ops(path))?);
    store.revoke_right_with_fee(path, right_ctx(&current, 15), env, fee_actor(mark, 15))
}

pub fn consume_right_fee(
    store: &mut SettlementStore,
    seed: &RightSeed,
    mark: u8,
) -> Result<SettlementStateRoot, SettlementStoreError> {
    let path = right_path(seed);
    let current = store
        .get_settlement_item(&path)?
        .and_then(|item| item.right_leaf().ok().copied())
        .ok_or(SettlementStoreError::PathMiss)?;
    let env = fee_envelope(mark, store.fee_support_ctx(&fee_del_ops(path))?);
    store.consume_right_with_fee(path, right_ctx(&current, 15), env, fee_actor(mark, 15))
}

pub fn sum_proof_bytes(blobs: &[crate::settlement::ProofBlob]) -> usize {
    blobs
        .iter()
        .map(|blob| blob.encode().expect("proof bytes").len())
        .sum()
}

pub fn statm_resident() -> Option<u64> {
    let body = read_to_string("/proc/self/statm").ok()?;
    let pages = body.split_whitespace().nth(1)?.parse::<u64>().ok()?;
    Some(pages.saturating_mul(4096))
}

pub fn redb_bytes(root: &Path) -> u64 {
    read_file(root.join("settlement_state.redb"))
        .map(|bytes| bytes.len() as u64)
        .unwrap_or(0)
}

pub fn fixture_root() -> DefinitionId {
    DefinitionId::new([0x11; 32])
}

pub fn ser_id(value: u32) -> SerialId {
    SerialId::new(value)
}

pub fn marker_leaf(leaf: RightLeaf) -> SettlementLeaf {
    SettlementLeaf::Right(leaf)
}

#[cfg(all(feature = "test-params-fast", debug_assertions))]
fn relax_fixture_sched(store: &SettlementStore) {
    // Corpus builders probe many candidate terminals in sequence. Keep the
    // worker lane deterministic while raising the queue ceiling so proof-shape
    // tests do not fail on backpressure first.
    store.set_sched_limits_for_test(1, 1024);
}

#[cfg(any(not(feature = "test-params-fast"), not(debug_assertions)))]
fn relax_fixture_sched(_store: &SettlementStore) {}

fn split_threshold(store: &SettlementStore) -> usize {
    usize::try_from(store.bucket_policy().min_bucket_count()).expect("usize") + 1
}

pub fn split_ready_paths(
    store: &mut SettlementStore,
    definition_mark: u8,
    serial_id: u32,
) -> Vec<SettlementPath> {
    let needed = split_threshold(store);
    let paths = same_bucket_paths_with_count(store, definition_mark, serial_id, needed);
    assert!(
        store.split_proof(&paths[0]).is_ok(),
        "split-ready fixture must build a split proof"
    );
    paths
}

pub fn same_bucket_paths_with_count(
    store: &mut SettlementStore,
    definition_mark: u8,
    serial_id: u32,
    needed: usize,
) -> Vec<SettlementPath> {
    relax_fixture_sched(store);
    let policy = store.bucket_policy();
    let first = SettlementPath::new(
        DefinitionId::new(bytes(definition_mark)),
        SerialId::new(serial_id),
        TerminalId::new(bytes(1)),
    );
    let target_bucket = first.bucket_id(policy);
    let mut selected = vec![(1u8, first)];

    for seed in 2..=u8::MAX {
        let candidate = SettlementPath::new(
            DefinitionId::new(bytes(definition_mark)),
            SerialId::new(serial_id),
            TerminalId::new(bytes(seed)),
        );
        if candidate.bucket_id(policy) == target_bucket {
            selected.push((seed, candidate));
            if selected.len() == needed {
                break;
            }
        }
    }

    assert_eq!(selected.len(), needed, "missing same-bucket split fixture");

    for (seed, path) in &selected {
        put_split_item(store, *path, 2_100 + u64::from(*seed));
    }

    selected.into_iter().map(|(_, path)| path).collect()
}

fn put_split_item(store: &mut SettlementStore, path: SettlementPath, value: u64) {
    let _ = value;
    let mut leaf = TerminalLeaf::dummy_for_scan(path.serial_id.get());
    leaf.set_terminal_id(path.terminal_id());
    store
        .put_settlement_item(StoreItem::new(path, leaf).expect("split seed item"))
        .expect("put split seed item");
}

pub fn sibling_bucket_pair(
    store: &mut SettlementStore,
    definition_mark: u8,
    serial_id: u32,
) -> (SettlementPath, SettlementPath) {
    relax_fixture_sched(store);
    let mut first_by_bucket = BTreeMap::new();
    let bucket_bits = store.bucket_policy().bucket_bits();

    for mark in 1..=255 {
        let candidate_seed = AssetSeed {
            label: format!("merge_{mark}"),
            definition_mark,
            serial_id,
            terminal_mark: mark,
            value: u64::from(mark) + 3_000,
        };
        let candidate = asset_path(&candidate_seed);
        let _ = store
            .put_settlement_item(asset_item(&candidate_seed))
            .expect("seed merge candidate");
        let bucket = store
            .adaptive_bucket(&candidate)
            .expect("adaptive bucket")
            .bucket_id;
        let sibling = sibling_bucket_id(bucket, bucket_bits);
        if let Some(other) = first_by_bucket.get(&sibling).copied() {
            if store.merge_proof(&other, &candidate).is_ok() {
                return (other, candidate);
            }
        }
        first_by_bucket.entry(bucket).or_insert(candidate);
    }

    panic!("missing merge-trigger pair");
}

pub fn next_policy(store: &SettlementStore) -> crate::settlement::BucketPolicy {
    crate::settlement::BucketPolicy::new(
        store.bucket_policy().bucket_bits() + 1,
        store.bucket_policy().min_bucket_count(),
        store.bucket_policy().max_target_leaf_count(),
        store.bucket_policy().compatibility_generation() + 1,
    )
    .expect("next bucket policy")
}

fn sibling_bucket_id(
    bucket_id: crate::settlement::BucketId,
    bucket_bits: u8,
) -> crate::settlement::BucketId {
    let mut bytes = bucket_id.into_bytes();
    let bit_index = bucket_bits - 1;
    let byte_index = usize::from(bit_index / 8);
    let bit_mask = 1u8 << (7 - (bit_index % 8));
    bytes[byte_index] ^= bit_mask;
    crate::settlement::BucketId::new(bytes)
}

pub fn proof_family(blob: &crate::settlement::ProofBlob) -> HjmtProofFamily {
    blob.hjmt_proof_family().expect("hjmt proof family")
}

pub fn nonexistence_marker(path: SettlementPath, family: SettlementLeafFamily) -> SettlementLeaf {
    family.marker_leaf(path)
}

#[derive(Clone, Debug, Default)]
pub struct OracleState {
    defs: BTreeMap<DefinitionId, BTreeMap<SerialId, BTreeMap<TerminalId, SettlementLeaf>>>,
}

impl OracleState {
    pub fn put(&mut self, item: StoreItem) -> Result<SettlementStateRoot, SettlementStoreError> {
        item.leaf()
            .check_path(item.path())
            .map_err(|err| SettlementStoreError::Model(err.into()))?;
        self.defs
            .entry(item.path().definition_id)
            .or_default()
            .entry(item.path().serial_id)
            .or_default()
            .insert(item.path().terminal_id, item.leaf().clone());
        self.root()
    }

    pub fn delete(
        &mut self,
        path: SettlementPath,
    ) -> Result<SettlementStateRoot, SettlementStoreError> {
        let Some(serials) = self.defs.get_mut(&path.definition_id) else {
            return Err(SettlementStoreError::PathMiss);
        };
        let Some(terminals) = serials.get_mut(&path.serial_id) else {
            return Err(SettlementStoreError::PathMiss);
        };
        if terminals.remove(&path.terminal_id).is_none() {
            return Err(SettlementStoreError::PathMiss);
        }
        if terminals.is_empty() {
            serials.remove(&path.serial_id);
        }
        if serials.is_empty() {
            self.defs.remove(&path.definition_id);
        }
        self.root()
    }

    pub fn create_right(
        &mut self,
        path: SettlementPath,
        leaf: RightLeaf,
    ) -> Result<SettlementStateRoot, SettlementStoreError> {
        if self.get(path).is_some() {
            return Err(SettlementStoreError::Backend(
                "settlement path already exists".to_string(),
            ));
        }
        leaf.validate_action(RightAction::Create, RightActionCtx::default(), None)
            .map_err(|err| SettlementStoreError::Model(err.into()))?;
        self.put(StoreItem::new(path, SettlementLeaf::Right(leaf))?)
    }

    pub fn transfer_right(
        &mut self,
        path: SettlementPath,
        leaf: RightLeaf,
        now: u64,
    ) -> Result<SettlementStateRoot, SettlementStoreError> {
        let prior = self
            .get(path)
            .and_then(|item| item.right_leaf().ok().copied())
            .ok_or(SettlementStoreError::PathMiss)?;
        leaf.validate_action(RightAction::Transfer, right_ctx(&leaf, now), Some(&prior))
            .map_err(|err| SettlementStoreError::Model(err.into()))?;
        self.put(StoreItem::new(path, SettlementLeaf::Right(leaf))?)
    }

    pub fn revoke_right(
        &mut self,
        path: SettlementPath,
        now: u64,
    ) -> Result<SettlementStateRoot, SettlementStoreError> {
        let prior = self
            .get(path)
            .and_then(|item| item.right_leaf().ok().copied())
            .ok_or(SettlementStoreError::PathMiss)?;
        prior
            .validate_action(RightAction::Revoke, right_ctx(&prior, now), Some(&prior))
            .map_err(|err| SettlementStoreError::Model(err.into()))?;
        self.delete(path)
    }

    pub fn consume_right(
        &mut self,
        path: SettlementPath,
        now: u64,
    ) -> Result<SettlementStateRoot, SettlementStoreError> {
        let prior = self
            .get(path)
            .and_then(|item| item.right_leaf().ok().copied())
            .ok_or(SettlementStoreError::PathMiss)?;
        prior
            .validate_action(RightAction::Consume, right_ctx(&prior, now), Some(&prior))
            .map_err(|err| SettlementStoreError::Model(err.into()))?;
        self.delete(path)
    }

    pub fn expire_right(
        &mut self,
        path: SettlementPath,
        now: u64,
    ) -> Result<SettlementStateRoot, SettlementStoreError> {
        let prior = self
            .get(path)
            .and_then(|item| item.right_leaf().ok().copied())
            .ok_or(SettlementStoreError::PathMiss)?;
        prior
            .validate_action(
                RightAction::Expire,
                RightActionCtx {
                    now,
                    ..RightActionCtx::default()
                },
                Some(&prior),
            )
            .map_err(|err| SettlementStoreError::Model(err.into()))?;
        self.delete(path)
    }

    pub fn root(&self) -> Result<SettlementStateRoot, SettlementStoreError> {
        let mut def_parts = Vec::new();
        for (definition_id, serials) in &self.defs {
            let mut serial_parts = Vec::new();
            for (serial_id, terminals) in serials {
                let mut terminal_parts = Vec::new();
                for leaf in terminals.values() {
                    terminal_parts.push(leaf_hash(leaf)?);
                }
                let serial_leaf = SerialRootLeaf {
                    definition_id: *definition_id,
                    serial_id: *serial_id,
                    serial_root: hash_many::<SerialDom>(&terminal_parts),
                };
                serial_parts.push(serial_leaf.encode());
            }
            let def_leaf = DefinitionRootLeaf {
                definition_id: *definition_id,
                definition_root: hash_many::<DefDom>(&serial_parts),
            };
            def_parts.push(def_leaf.encode());
        }
        Ok(SettlementStateRoot::settlement_v1(hash_many::<StateDom>(
            &def_parts,
        )))
    }

    pub fn items(&self) -> Vec<StoreItem> {
        let mut items = Vec::new();
        for (definition_id, serials) in &self.defs {
            for (serial_id, terminals) in serials {
                for (terminal_id, leaf) in terminals {
                    items.push(
                        StoreItem::new(
                            SettlementPath::new(*definition_id, *serial_id, *terminal_id),
                            leaf.clone(),
                        )
                        .expect("oracle item"),
                    );
                }
            }
        }
        items
    }

    pub fn get(&self, path: SettlementPath) -> Option<StoreItem> {
        let leaf = self
            .defs
            .get(&path.definition_id)?
            .get(&path.serial_id)?
            .get(&path.terminal_id)?
            .clone();
        StoreItem::new(path, leaf).ok()
    }
}

pub fn assert_store_matches_oracle(store: &SettlementStore, oracle: &OracleState) {
    let store_root = store.settlement_root().expect("store root");
    let oracle_root = oracle.root().expect("oracle root");
    assert_eq!(store_root, oracle_root);
    assert_eq!(list_items(store).expect("store items"), oracle.items());
}

fn leaf_hash(leaf: &SettlementLeaf) -> Result<Vec<u8>, SettlementStoreError> {
    let payload = leaf.encode()?;
    Ok(poseidon2_hash(domain_tag::<AssetDom>().as_slice(), &[payload.as_slice()]).to_vec())
}

fn hash_many<M>(parts: &[Vec<u8>]) -> [u8; 32]
where
    M: DomainSeparation,
{
    let refs = parts.iter().map(Vec::as_slice).collect::<Vec<_>>();
    poseidon2_hash(domain_tag::<M>().as_slice(), &refs)
}

fn domain_tag<M>() -> Vec<u8>
where
    M: DomainSeparation,
{
    M::domain_separation_tag("").into_bytes()
}

fn env_lock() -> &'static Mutex<()> {
    static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    LOCK.get_or_init(|| Mutex::new(()))
}
