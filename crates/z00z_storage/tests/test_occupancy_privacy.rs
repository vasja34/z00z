use std::sync::{Mutex, OnceLock};

use z00z_core::assets::{AssetLeaf, AssetPackPlain};
use z00z_crypto::ZkPackEncrypted;
use z00z_storage::settlement::{
    DefinitionId, OccupancyClass, PolicyTransitionProof, SerialId, SettlementPath, SettlementStore,
    SplitProof, StoreItem, TerminalId, TerminalLeaf,
};
use z00z_utils::codec::{Codec, JsonCodec};

fn env_lock() -> &'static Mutex<()> {
    static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    LOCK.get_or_init(|| Mutex::new(()))
}

struct EnvGuard {
    bucket_bits: Option<String>,
    redb_root: Option<String>,
}

impl EnvGuard {
    fn with_bucket_bits(bits: &str) -> Self {
        let guard = Self {
            bucket_bits: std::env::var("Z00Z_SETTLEMENT_BUCKET_BITS").ok(),
            redb_root: std::env::var("Z00Z_STORAGE_REDB_ROOT").ok(),
        };
        std::env::set_var("Z00Z_SETTLEMENT_BUCKET_BITS", bits);
        std::env::remove_var("Z00Z_STORAGE_REDB_ROOT");
        guard
    }
}

impl Drop for EnvGuard {
    fn drop(&mut self) {
        if let Some(bits) = &self.bucket_bits {
            std::env::set_var("Z00Z_SETTLEMENT_BUCKET_BITS", bits);
        } else {
            std::env::remove_var("Z00Z_SETTLEMENT_BUCKET_BITS");
        }
        if let Some(root) = &self.redb_root {
            std::env::set_var("Z00Z_STORAGE_REDB_ROOT", root);
        } else {
            std::env::remove_var("Z00Z_STORAGE_REDB_ROOT");
        }
    }
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

fn put(store: &mut SettlementStore, path: SettlementPath, value: u64) {
    store
        .put_settlement_item(item(path, value))
        .expect("put settlement item");
}

fn split_ready_count(store: &SettlementStore) -> usize {
    usize::try_from(store.bucket_policy().min_bucket_count()).expect("usize") + 1
}

fn below_split_count(store: &SettlementStore) -> usize {
    split_ready_count(store).saturating_sub(1)
}

fn same_bucket_group(store: &mut SettlementStore, need: usize) -> Vec<SettlementPath> {
    let policy = store.bucket_policy();
    let first = path(41, 9, 1);
    let bucket_id = first.bucket_id(policy);
    let mut out = vec![(1u8, first)];
    for seed in 2..=255 {
        let next = path(41, 9, seed);
        if next.bucket_id(policy) == bucket_id {
            out.push((seed, next));
            if out.len() == need {
                break;
            }
        }
    }

    assert_eq!(out.len(), need, "failed same bucket group");
    for (seed, next) in &out {
        put(store, *next, 4_100 + u64::from(*seed));
    }
    out.into_iter().map(|(_, path)| path).collect()
}

fn trigger_split(store: &mut SettlementStore, target: SettlementPath) -> SettlementPath {
    let policy = store.bucket_policy();
    let target_bucket = target.bucket_id(policy);
    let current = usize::try_from(
        store
            .bucket_occupancy_metric(&target)
            .expect("target occupancy metric")
            .exact_count,
    )
    .expect("usize");
    let needed = split_ready_count(store).saturating_sub(current);
    assert!(needed > 0, "target is already split-ready");
    let mut selected = Vec::with_capacity(needed);
    for seed in 129..=255 {
        let next = path(41, 9, seed);
        if next.bucket_id(policy) == target_bucket {
            selected.push((seed, next));
            if selected.len() == needed {
                break;
            }
        }
    }

    assert_eq!(selected.len(), needed, "failed split trigger");
    for (seed, next) in &selected {
        put(store, *next, 4_100 + u64::from(*seed));
    }
    assert!(store.split_proof(&target).is_ok(), "failed split trigger");
    *selected
        .last()
        .map(|(_, path)| path)
        .expect("split trigger path")
}

fn sibling_bucket_id(
    bucket_id: z00z_storage::settlement::BucketId,
    bucket_bits: u8,
) -> z00z_storage::settlement::BucketId {
    let mut bytes = bucket_id.into_bytes();
    let bit_index = bucket_bits - 1;
    let byte_index = usize::from(bit_index / 8);
    let bit_mask = 1u8 << (7 - (bit_index % 8));
    bytes[byte_index] ^= bit_mask;
    z00z_storage::settlement::BucketId::new(bytes)
}

fn sibling_pair(store: &mut SettlementStore) -> (SettlementPath, SettlementPath) {
    let mut firsts = std::collections::BTreeMap::new();
    let bits = store.bucket_policy().bucket_bits();

    for seed in 1..=128 {
        let next = path(33, 11, seed);
        put(store, next, 3_300 + u64::from(seed));
        let bucket = next.bucket_id(store.bucket_policy());
        let sibling = sibling_bucket_id(bucket, bits);
        if let Some(other) = firsts.get(&sibling).copied() {
            if store.merge_proof(&other, &next).is_ok() {
                return (other, next);
            }
        }
        firsts.entry(bucket).or_insert(next);
    }

    panic!("failed sibling pair")
}

fn add_noise(store: &mut SettlementStore, left: &[u8], right: &[u8]) {
    for seed in left {
        put(store, path(71, 1, *seed), 7_100 + u64::from(*seed));
    }
    for seed in right {
        put(store, path(72, 1, *seed), 7_200 + u64::from(*seed));
    }
}

fn split_store(left: &[u8], right: &[u8]) -> (SettlementStore, SettlementPath) {
    let mut store = SettlementStore::new();
    let steady_count = below_split_count(&store);
    let group = same_bucket_group(&mut store, steady_count);
    let target = trigger_split(&mut store, group[0]);
    add_noise(&mut store, left, right);
    (store, target)
}

fn merge_store(left: &[u8], right: &[u8]) -> (SettlementStore, SettlementPath, SettlementPath) {
    let mut store = SettlementStore::new();
    let (first, second) = sibling_pair(&mut store);
    add_noise(&mut store, left, right);
    (store, first, second)
}

fn next_policy(store: &SettlementStore) -> z00z_storage::settlement::BucketPolicy {
    z00z_storage::settlement::BucketPolicy::new(
        store.bucket_policy().bucket_bits(),
        store.bucket_policy().min_bucket_count(),
        store.bucket_policy().max_target_leaf_count(),
        store.bucket_policy().compatibility_generation() + 1,
    )
    .expect("next policy")
}

fn json<T: serde::Serialize>(value: &T) -> String {
    String::from_utf8(JsonCodec.serialize(value).expect("json bytes")).expect("utf8")
}

fn assert_hidden(json: &str) {
    for needle in [
        "leaf_count",
        "exact_count",
        "raw_delta",
        "timing",
        "timestamp",
    ] {
        assert!(
            !json.contains(needle),
            "proof-visible payload must not contain {needle}"
        );
    }
}

#[test]
fn test_metric_stays_private() {
    let _guard = env_lock()
        .lock()
        .unwrap_or_else(|poison| poison.into_inner());
    let _env = EnvGuard::with_bucket_bits("1");
    let (store, target) = split_store(&[], &[]);
    let metric = store.bucket_occupancy_metric(&target).expect("metric");
    let bucket = store.adaptive_bucket(&target).expect("bucket");
    let split = store.split_proof(&target).expect("split proof");
    let transition: PolicyTransitionProof = store
        .policy_transition_proof(next_policy(&store))
        .expect("transition proof");

    assert!(metric.exact_count >= 3);
    assert!(json(&metric).contains("exact_count"));
    assert_hidden(&json(&bucket));
    assert_hidden(&json(&split));
    assert_hidden(&json(&transition));
}

#[test]
fn test_split_noise_blind() {
    let _guard = env_lock()
        .lock()
        .unwrap_or_else(|poison| poison.into_inner());
    let _env = EnvGuard::with_bucket_bits("1");
    let (store_a, target_a) = split_store(&[201], &[211, 212, 213]);
    let (store_b, target_b) = split_store(&[201, 202, 203], &[211]);
    let proof_a: SplitProof = store_a.split_proof(&target_a).expect("split proof a");
    let proof_b: SplitProof = store_b.split_proof(&target_b).expect("split proof b");

    assert_eq!(proof_a.occupancy_evidence, proof_b.occupancy_evidence);
}

#[test]
fn test_merge_noise_blind() {
    let _guard = env_lock()
        .lock()
        .unwrap_or_else(|poison| poison.into_inner());
    let _env = EnvGuard::with_bucket_bits("2");
    let (store_a, left_a, right_a) = merge_store(&[201], &[211, 212, 213]);
    let (store_b, left_b, right_b) = merge_store(&[201, 202, 203], &[211]);
    let proof_a = store_a
        .merge_proof(&left_a, &right_a)
        .expect("merge proof a");
    let proof_b = store_b
        .merge_proof(&left_b, &right_b)
        .expect("merge proof b");

    assert_eq!(proof_a.left_evidence, proof_b.left_evidence);
    assert_eq!(proof_a.right_evidence, proof_b.right_evidence);
    assert_eq!(proof_a.pair_evidence, proof_b.pair_evidence);
}

#[test]
fn test_split_class_bound() {
    let _guard = env_lock()
        .lock()
        .unwrap_or_else(|poison| poison.into_inner());
    let _env = EnvGuard::with_bucket_bits("1");
    let (mut store, target) = split_store(&[], &[]);
    let metric_a = store.bucket_occupancy_metric(&target).expect("metric a");
    let proof_a = store.split_proof(&target).expect("proof a");

    add_noise(&mut store, &[231], &[241, 242, 243]);

    let metric_b = store.bucket_occupancy_metric(&target).expect("metric b");
    let proof_b = store.split_proof(&target).expect("proof b");

    assert_eq!(metric_a.class, OccupancyClass::SplitReady);
    assert_eq!(metric_b.class, OccupancyClass::SplitReady);
    assert_eq!(metric_a.exact_count, metric_b.exact_count);
    assert_eq!(
        proof_a.occupancy_evidence.class,
        proof_b.occupancy_evidence.class
    );
    assert_hidden(&json(&proof_a));
    assert_hidden(&json(&proof_b));
}
