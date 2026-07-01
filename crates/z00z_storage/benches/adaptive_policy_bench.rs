use criterion::{black_box, BatchSize, Criterion};
use z00z_core::assets::{AssetLeaf, AssetPackPlain};
use z00z_crypto::ZkPackEncrypted;
use z00z_storage::settlement::{
    BucketId, BucketPolicy, DefinitionId, SerialId, SettlementPath, SettlementStore, StoreItem,
    TerminalId, TerminalLeaf,
};
use z00z_utils::config::{ConfigSource, EnvConfig};

struct EnvGuard {
    bucket_bits: Option<String>,
    redb_root: Option<String>,
}

impl EnvGuard {
    fn with_bucket_bits(bits: &str) -> Self {
        let guard = Self {
            bucket_bits: EnvConfig.get("Z00Z_SETTLEMENT_BUCKET_BITS").ok().flatten(),
            redb_root: EnvConfig.get("Z00Z_STORAGE_REDB_ROOT").ok().flatten(),
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

fn same_bucket_group(
    store: &mut SettlementStore,
    needed: usize,
    definition: u8,
    serial: u32,
    base_value: u64,
) -> Vec<SettlementPath> {
    if needed == 0 {
        return Vec::new();
    }

    let policy = store.bucket_policy();
    let first = path(definition, serial, 1);
    let bucket_id = first.bucket_id(policy);
    let mut selected = vec![(1u8, first)];
    if selected.len() < needed {
        for seed in 2..=255 {
            let candidate = path(definition, serial, seed);
            if candidate.bucket_id(policy) == bucket_id {
                selected.push((seed, candidate));
                if selected.len() == needed {
                    break;
                }
            }
        }
    }

    assert_eq!(selected.len(), needed, "failed to find same-bucket group");
    for (seed, candidate) in &selected {
        put(store, *candidate, base_value + u64::from(*seed));
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

    for seed in 1..=255 {
        let candidate = path(83, 17, seed);
        put(store, candidate, 8_300 + u64::from(seed));
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
        "failed to find next same-bucket path"
    );
    for (seed, candidate) in &selected {
        put(store, *candidate, base_value + u64::from(*seed));
    }
    *selected
        .last()
        .map(|(_, path)| path)
        .expect("same-bucket trigger path")
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

fn split_store(trigger: bool) -> (EnvGuard, SettlementStore, SettlementPath) {
    let env = EnvGuard::with_bucket_bits("1");
    let mut store = SettlementStore::new();
    let steady_count = split_ready_count(&store).saturating_sub(1);
    let paths = same_bucket_group(&mut store, steady_count, 71, 13, 7_100);
    if trigger {
        let _ = trigger_split_path(&mut store, paths[0], 71, 13, 129, 7_100);
    }
    (env, store, paths[0])
}

fn merge_store(trigger: bool) -> (EnvGuard, SettlementStore, SettlementPath, SettlementPath) {
    let env = EnvGuard::with_bucket_bits("2");
    let mut store = SettlementStore::new();
    let (left, right) = sibling_bucket_pair(&mut store);
    if !trigger {
        let _ = next_same_bucket_path(&mut store, left, 83, 17, 129, 8_500);
    }
    (env, store, left, right)
}

fn fixed_store() -> (EnvGuard, SettlementStore, SettlementPath) {
    let env = EnvGuard::with_bucket_bits("1");
    let mut store = SettlementStore::new();
    let paths = same_bucket_group(&mut store, 1, 91, 23, 9_100);
    (env, store, paths[0])
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

fn bench_fixed_baseline(c: &mut Criterion) {
    let mut group = c.benchmark_group("fixed_baseline");
    group.bench_function("settlement_proof_blob", |b| {
        b.iter_batched(
            fixed_store,
            |(_env, store, path)| {
                let proof = store
                    .settlement_proof_blob(black_box(&path))
                    .expect("settlement proof blob");
                black_box(proof);
            },
            BatchSize::SmallInput,
        )
    });
    group.finish();
}

fn bench_split(c: &mut Criterion) {
    let mut group = c.benchmark_group("adaptive_split");
    group.bench_function("trigger", |b| {
        b.iter_batched(
            || split_store(true),
            |(_env, store, path)| {
                let proof = store.split_proof(black_box(&path)).expect("split proof");
                black_box(proof);
            },
            BatchSize::SmallInput,
        )
    });
    group.bench_function("no_op", |b| {
        b.iter_batched(
            || split_store(false),
            |(_env, store, path)| {
                let err = store
                    .split_proof(black_box(&path))
                    .expect_err("split hysteresis no-op");
                black_box(err);
            },
            BatchSize::SmallInput,
        )
    });
    group.finish();
}

fn bench_merge(c: &mut Criterion) {
    let mut group = c.benchmark_group("adaptive_merge");
    group.bench_function("trigger", |b| {
        b.iter_batched(
            || merge_store(true),
            |(_env, store, left, right)| {
                let proof = store
                    .merge_proof(black_box(&left), black_box(&right))
                    .expect("merge proof");
                black_box(proof);
            },
            BatchSize::SmallInput,
        )
    });
    group.bench_function("no_op", |b| {
        b.iter_batched(
            || merge_store(false),
            |(_env, store, left, right)| {
                let err = store
                    .merge_proof(black_box(&left), black_box(&right))
                    .expect_err("merge hysteresis no-op");
                black_box(err);
            },
            BatchSize::SmallInput,
        )
    });
    group.finish();
}

fn bench_transition(c: &mut Criterion) {
    let mut group = c.benchmark_group("policy_transition");
    group.bench_function("prove", |b| {
        b.iter_batched(
            || split_store(true),
            |(_env, store, _path)| {
                let next_policy = next_policy(&store);
                let proof = store
                    .policy_transition_proof(black_box(next_policy))
                    .expect("policy transition proof");
                black_box(proof);
            },
            BatchSize::SmallInput,
        )
    });
    group.bench_function("historical_prove", |b| {
        b.iter_batched(
            || split_store(true),
            |(_env, store, _path)| {
                let next_policy = next_policy(&store);
                let proof = store
                    .policy_transition_proof(black_box(next_policy))
                    .expect("historical transition proof");
                black_box(proof);
            },
            BatchSize::SmallInput,
        )
    });
    group.bench_function("verify", |b| {
        b.iter_batched(
            || {
                let (_env, store, _path) = split_store(true);
                let next_policy = next_policy(&store);
                let proof = store
                    .policy_transition_proof(next_policy)
                    .expect("policy transition proof");
                (_env, store, next_policy, proof)
            },
            |(_env, store, next_policy, proof)| {
                store
                    .validate_policy_transition_proof(black_box(&proof), black_box(next_policy))
                    .expect("policy transition validation");
            },
            BatchSize::SmallInput,
        )
    });
    group.bench_function("historical_verify", |b| {
        b.iter_batched(
            || {
                let (env, mut store, _path) = split_store(true);
                let next = next_policy(&store);
                let proof = store
                    .policy_transition_proof(next)
                    .expect("historical policy transition proof");
                put(&mut store, path(92, 29, 92), 9_200);
                (env, store, next, proof)
            },
            |(_env, store, next, proof)| {
                store
                    .validate_policy_transition_proof(black_box(&proof), black_box(next))
                    .expect("historical policy transition validation");
            },
            BatchSize::SmallInput,
        )
    });
    group.bench_function("stale_reject", |b| {
        b.iter_batched(
            || {
                let (env, store, _path) = split_store(true);
                let next = next_policy(&store);
                let proof = store
                    .policy_transition_proof(next)
                    .expect("policy transition proof");
                let stale = z00z_storage::settlement::PolicyTransitionProof {
                    prior_policy_id: [0x11; 32],
                    ..proof
                };
                (env, store, next, stale)
            },
            |(_env, store, next, stale)| {
                let err = store
                    .validate_policy_transition_proof(black_box(&stale), black_box(next))
                    .expect_err("stale transition rejection");
                black_box(err);
            },
            BatchSize::SmallInput,
        )
    });
    group.finish();
}

fn bench_occupancy(c: &mut Criterion) {
    let mut group = c.benchmark_group("occupancy_evidence");
    group.bench_function("split_metric", |b| {
        b.iter_batched(
            || split_store(true),
            |(_env, store, path)| {
                let metric = store
                    .bucket_occupancy_metric(black_box(&path))
                    .expect("bucket occupancy metric");
                black_box(metric);
            },
            BatchSize::SmallInput,
        )
    });
    group.finish();
}

fn main() {
    let mut crit = Criterion::default().configure_from_args();
    bench_fixed_baseline(&mut crit);
    bench_split(&mut crit);
    bench_merge(&mut crit);
    bench_transition(&mut crit);
    bench_occupancy(&mut crit);
    crit.final_summary();
}
