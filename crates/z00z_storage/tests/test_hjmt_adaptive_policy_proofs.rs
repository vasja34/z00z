use std::{
    collections::BTreeMap,
    sync::{Mutex, OnceLock},
};

use z00z_core::assets::{AssetLeaf, AssetPackPlain};
use z00z_crypto::ZkPackEncrypted;
use z00z_storage::settlement::{
    AdaptiveProofErr, BucketEpoch, BucketId, BucketPolicy, DefinitionId, MergeProof,
    OccupancyScope, PolicyTransitionProof, SerialId, SettlementPath, SettlementStateRoot,
    SettlementStore, SplitProof, StoreItem, TerminalId, TerminalLeaf,
};

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

fn sibling_bucket_id(bucket_id: BucketId, bucket_bits: u8) -> BucketId {
    let mut bytes = bucket_id.into_bytes();
    let bit_index = bucket_bits - 1;
    let byte_index = usize::from(bit_index / 8);
    let bit_mask = 1u8 << (7 - (bit_index % 8));
    bytes[byte_index] ^= bit_mask;
    BucketId::new(bytes)
}

fn split_ready_paths(store: &mut SettlementStore) -> Vec<SettlementPath> {
    let needed = split_ready_count(store);
    let paths = same_bucket_group(store, needed);
    assert!(
        store.split_proof(&paths[0]).is_ok(),
        "split-ready fixture must build a split proof"
    );
    paths
}

fn same_bucket_group(store: &mut SettlementStore, needed: usize) -> Vec<SettlementPath> {
    let policy = store.bucket_policy();
    let first = path(41, 9, 1);
    let bucket_id = first.bucket_id(policy);
    let mut selected = vec![(1u8, first)];
    for seed in 2..=255 {
        let candidate = path(41, 9, seed);
        if candidate.bucket_id(policy) == bucket_id {
            selected.push((seed, candidate));
            if selected.len() == needed {
                break;
            }
        }
    }

    assert_eq!(selected.len(), needed, "failed to find same-bucket group");
    for (seed, candidate) in &selected {
        put(store, *candidate, 4_100 + u64::from(*seed));
    }
    selected.into_iter().map(|(_, path)| path).collect()
}

fn sibling_bucket_pair(store: &mut SettlementStore) -> (SettlementPath, SettlementPath) {
    let mut first_paths = BTreeMap::<BucketId, SettlementPath>::new();
    let bucket_bits = store.bucket_policy().bucket_bits();

    for seed in 1..=128 {
        let candidate = path(33, 11, seed);
        put(store, candidate, 3_300 + u64::from(seed));
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

fn next_same_bucket_path(
    store: &mut SettlementStore,
    target: SettlementPath,
    definition: u8,
    serial: u32,
    start_seed: u8,
    base_value: u64,
) -> SettlementPath {
    let policy = store.bucket_policy();
    let target_bucket = target.bucket_id(policy);
    let threshold = split_ready_count(store);
    let current = usize::try_from(
        store
            .bucket_occupancy_metric(&target)
            .expect("target occupancy metric")
            .exact_count,
    )
    .expect("usize");
    let needed = threshold.saturating_sub(current);
    assert!(needed > 0, "target is already split-ready");
    let mut selected = Vec::with_capacity(needed);

    for seed in start_seed..=255 {
        let candidate = path(definition, serial, seed);
        if candidate.bucket_id(policy) == target_bucket {
            selected.push((seed, candidate));
            if selected.len() == needed {
                break;
            }
        }
    }

    assert_eq!(
        selected.len(),
        needed,
        "failed to find extra same-bucket path"
    );
    for (seed, candidate) in &selected {
        put(store, *candidate, base_value + u64::from(*seed));
    }
    *selected
        .last()
        .map(|(_, path)| path)
        .expect("split trigger path")
}

fn trigger_split_path(
    store: &mut SettlementStore,
    target: SettlementPath,
    definition: u8,
    serial: u32,
    start_seed: u8,
    base_value: u64,
) -> SettlementPath {
    let candidate =
        next_same_bucket_path(store, target, definition, serial, start_seed, base_value);
    assert!(
        store.split_proof(&target).is_ok(),
        "failed to reach the split trigger"
    );
    candidate
}

#[test]
fn test_bucket_metadata_epoch_bound() {
    let _guard = env_lock().lock().expect("env lock");
    let _env = EnvGuard::with_bucket_bits("1");
    let mut store = SettlementStore::new();
    let live_path = path(14, 3, 1);

    put(&mut store, live_path, 1_401);
    let bucket = store.adaptive_bucket(&live_path).expect("adaptive bucket");

    assert_eq!(bucket.definition_id, live_path.definition_id);
    assert_eq!(bucket.serial_id, live_path.serial_id);
    assert_eq!(bucket.epoch, BucketEpoch::new(1));
    assert_eq!(
        bucket.bucket_policy_id,
        store.bucket_policy().bucket_policy_id()
    );
    assert_ne!(bucket.bucket_root, [0u8; 32]);
    assert_eq!(bucket.occupancy_evidence.version, 1);
    assert_eq!(bucket.occupancy_evidence.scope, OccupancyScope::Bucket);
    assert_ne!(bucket.occupancy_evidence.bind, [0u8; 32]);
}

#[test]
fn test_no_op_then_trigger() {
    let _guard = env_lock().lock().expect("env lock");
    let _env = EnvGuard::with_bucket_bits("1");
    let mut store = SettlementStore::new();
    let steady_count = below_split_count(&store);
    let paths = same_bucket_group(&mut store, steady_count);

    let err = store
        .split_proof(&paths[0])
        .expect_err("two members must stay below the split hysteresis trigger");
    assert!(matches!(err, AdaptiveProofErr::SplitIneligible));

    let trigger_path = trigger_split_path(&mut store, paths[0], 41, 9, 129, 4_100);
    let proof = store
        .split_proof(&trigger_path)
        .expect("three members must cross the split trigger");
    store
        .validate_split_proof(&proof)
        .expect("split proof after trigger");
}

#[test]
fn test_trigger_then_no_op() {
    let _guard = env_lock().lock().expect("env lock");
    let _env = EnvGuard::with_bucket_bits("2");
    let mut store = SettlementStore::new();
    let (left, right) = sibling_bucket_pair(&mut store);

    let proof = store.merge_proof(&left, &right).expect("merge proof");
    store
        .validate_merge_proof(&proof)
        .expect("merge proof validation");

    let extra = next_same_bucket_path(&mut store, left, 33, 11, 129, 3_500);
    let err = store
        .merge_proof(&left, &right)
        .expect_err("hysteresis must block merge after one sibling grows");
    assert!(matches!(err, AdaptiveProofErr::MergeIneligible));
    assert_ne!(left, extra);
}

#[test]
fn test_hist_epoch_rejects_tamper() {
    let _guard = env_lock().lock().expect("env lock");
    let _env = EnvGuard::with_bucket_bits("1");
    let mut store = SettlementStore::new();
    let split_paths = split_ready_paths(&mut store);
    let first = split_paths[0];
    let second = split_paths[1];

    let proof = store.split_proof(&first).expect("split proof");
    store
        .validate_split_proof(&proof)
        .expect("current split proof validation");

    put(&mut store, path(29, 5, 99), 9_999);
    store
        .validate_split_proof(&proof)
        .expect("historical split proof validation");

    let tampered_root = SplitProof {
        prior_root: SettlementStateRoot::settlement_v1([0x42; 32]),
        ..proof
    };
    let err = store
        .validate_split_proof(&tampered_root)
        .expect_err("wrong prior root must reject");
    assert!(matches!(err, AdaptiveProofErr::WrongOldRoot));

    let tampered_epoch = SplitProof {
        prior_epoch: BucketEpoch::new(proof.prior_epoch.get() + 1),
        ..proof
    };
    let err = store
        .validate_split_proof(&tampered_epoch)
        .expect_err("wrong epoch must reject");
    assert!(matches!(err, AdaptiveProofErr::WrongEpoch));

    let tampered_child = SplitProof {
        right_bucket_root: [0xA5; 32],
        ..proof
    };
    let err = store
        .validate_split_proof(&tampered_child)
        .expect_err("wrong child root must reject");
    assert!(matches!(err, AdaptiveProofErr::WrongNewRoot));

    assert_ne!(first, second);
}

#[test]
fn test_siblings_rejects_nonadjacent_pair() {
    let _guard = env_lock().lock().expect("env lock");
    let _env = EnvGuard::with_bucket_bits("2");
    let mut store = SettlementStore::new();
    let (left, right) = sibling_bucket_pair(&mut store);

    let proof = store.merge_proof(&left, &right).expect("merge proof");
    store
        .validate_merge_proof(&proof)
        .expect("merge proof validation");

    let nonadjacent = path(33, 11, 127);
    put(&mut store, nonadjacent, 4_127);
    let err = store
        .merge_proof(&left, &nonadjacent)
        .expect_err("non-sibling pair must reject");
    assert!(matches!(err, AdaptiveProofErr::MergeIneligible));

    let tampered_root = MergeProof {
        merged_bucket_root: [0x5A; 32],
        ..proof
    };
    let err = store
        .validate_merge_proof(&tampered_root)
        .expect_err("wrong merged root must reject");
    assert!(matches!(err, AdaptiveProofErr::WrongNewRoot));
}

#[test]
fn test_rejects_stale_drifted_policy() {
    let _guard = env_lock().lock().expect("env lock");
    let _env = EnvGuard::with_bucket_bits("1");
    let mut store = SettlementStore::new();
    let first = split_ready_paths(&mut store)[0];

    let next_policy = BucketPolicy::new(
        store.bucket_policy().bucket_bits(),
        store.bucket_policy().min_bucket_count(),
        store.bucket_policy().max_target_leaf_count(),
        store.bucket_policy().compatibility_generation() + 1,
    )
    .expect("next bucket policy");

    let proof = store
        .policy_transition_proof(next_policy)
        .expect("policy transition proof");
    store
        .validate_policy_transition_proof(&proof, next_policy)
        .expect("policy transition validation");

    let stale_policy = PolicyTransitionProof {
        prior_policy_id: [0x11; 32],
        ..proof
    };
    let err = store
        .validate_policy_transition_proof(&stale_policy, next_policy)
        .expect_err("stale prior policy must reject");
    assert!(matches!(err, AdaptiveProofErr::StalePolicyId));

    let drift_policy = BucketPolicy::new(
        next_policy.bucket_bits() + 1,
        next_policy.min_bucket_count(),
        next_policy.max_target_leaf_count(),
        next_policy.compatibility_generation() + 1,
    )
    .expect("drifted bucket policy");
    let err = store
        .validate_policy_transition_proof(&proof, drift_policy)
        .expect_err("next policy drift must reject");
    assert!(matches!(
        err,
        AdaptiveProofErr::NextPolicyDrift | AdaptiveProofErr::WrongNewRoot
    ));

    let stale_err = store
        .policy_transition_proof(store.bucket_policy())
        .expect_err("stale policy transition must reject");
    assert!(matches!(stale_err, AdaptiveProofErr::StalePolicyId));

    assert_ne!(first, path(21, 7, 64));
}
