use criterion::{black_box, BatchSize, Criterion};
use stats_alloc::{Region, StatsAlloc, INSTRUMENTED_SYSTEM};
use std::{alloc::System, time::Instant};
use tempfile::TempDir;
use z00z_core::assets::AssetLeaf;
use z00z_storage::{
    fixture_support::{
        settlement_bench_output::{should_emit_side_outputs, write_meta, write_note, BenchMeta},
        settlement_corpus,
        settlement_corpus::{
            asset_item, asset_seed, consume_right_fee, create_right_fee, del_ops, fee_actor,
            fee_del_ops, fee_envelope, hot_assets, hot_rights, load_fixture, load_fixture_items,
            mixed_items, next_policy, put_ops, redb_bytes, revoke_right_fee, right_ctx, right_item,
            right_leaf, right_path, right_seed, seed_mem, seed_redb, sibling_bucket_pair,
            split_ready_paths, statm_resident, transferred_right_leaf, FixtureRightClass,
            HjmtEnvGuard, ProofBatchModeEnv, SchedEnv,
        },
    },
    settlement::{
        CheckpointPublicationV1, DefinitionId, PublicationModeTagV1, RootGenerationTagV1, SerialId,
        SettlementLeaf, SettlementLeafFamily, SettlementListReq, SettlementLookup, SettlementPath,
        SettlementStateRoot, SettlementStore, SettlementStoreError, ShardRootLeafV1, StoreItem,
        TerminalId, TerminalLeaf,
    },
};
use z00z_utils::{
    codec::{Codec, YamlCodec},
    io,
};

#[global_allocator]
static GLOBAL: &StatsAlloc<System> = &INSTRUMENTED_SYSTEM;

#[derive(serde::Deserialize)]
struct BenchStorageCfg {
    settings: BenchStorageSettings,
}

#[derive(serde::Deserialize)]
struct BenchStorageSettings {
    cache_capacity: usize,
}

const SHARD_ROOT_FIXTURE_MANIFEST: &str =
    "crates/z00z_storage/tests/fixtures/hjmt_upgrade/shard_root_leaf_v1/manifest.json";
const CHECKPOINT_FIXTURE_MANIFEST: &str =
    "crates/z00z_storage/tests/fixtures/hjmt_upgrade/checkpoint_publication_v1/manifest.json";

fn cache_invalidations(metrics: &z00z_storage::settlement::ForestCacheMetrics) -> u64 {
    metrics.subtree_root.invalidations
        + metrics.parent_leaf.invalidations
        + metrics.terminal_leaf.invalidations
        + metrics.bucket_derivation.invalidations
        + metrics.proof_segment.invalidations
        + metrics.nonexistence.invalidations
        + metrics.policy_proof.invalidations
        + metrics.journal_digest.invalidations
        + metrics.path_index.invalidations
}

fn cache_hits(metrics: &z00z_storage::settlement::ForestCacheMetrics) -> u64 {
    metrics.subtree_root.hits
        + metrics.parent_leaf.hits
        + metrics.terminal_leaf.hits
        + metrics.bucket_derivation.hits
        + metrics.proof_segment.hits
        + metrics.nonexistence.hits
        + metrics.policy_proof.hits
        + metrics.journal_digest.hits
        + metrics.path_index.hits
}

fn cache_misses(metrics: &z00z_storage::settlement::ForestCacheMetrics) -> u64 {
    metrics.subtree_root.misses
        + metrics.parent_leaf.misses
        + metrics.terminal_leaf.misses
        + metrics.bucket_derivation.misses
        + metrics.proof_segment.misses
        + metrics.nonexistence.misses
        + metrics.policy_proof.misses
        + metrics.journal_digest.misses
        + metrics.path_index.misses
}

fn reuse_ratio(hits: u64, misses: u64) -> f64 {
    let denom = (hits + misses).max(1) as f64;
    hits as f64 / denom
}

fn expect_support_required<T>(result: Result<T, SettlementStoreError>) {
    match result {
        Err(SettlementStoreError::Fee(z00z_storage::settlement::FeeErr::SupportRequired)) => {}
        Err(other) => panic!("unexpected error: {other:?}"),
        Ok(_) => panic!("expected fee support rejection"),
    }
}

fn base_store() -> SettlementStore {
    let fixture = load_fixture();
    seed_mem(&load_fixture_items(&fixture))
}

fn path_index_store() -> (SettlementStore, SettlementPath) {
    let fixture = load_fixture();
    let store = seed_mem(&load_fixture_items(&fixture));
    (store, settlement_corpus::asset_path(&fixture.assets[1]))
}

fn reload_lookup_store() -> (HjmtEnvGuard, TempDir, SettlementStore, SettlementPath) {
    let fixture = load_fixture();
    let items = load_fixture_items(&fixture);
    let path = settlement_corpus::asset_path(&fixture.assets[0]);
    let (guard, temp, store) = seed_redb("2", &items);
    drop(store);
    let reloaded = SettlementStore::load(temp.path()).expect("reload lookup store");
    (guard, temp, reloaded, path)
}

fn warm_reload_lookup_store() -> (HjmtEnvGuard, TempDir, SettlementStore, SettlementPath) {
    let (guard, temp, store, path) = reload_lookup_store();
    let _ = store
        .lookup_settlement(SettlementLookup::Path(path))
        .expect("warm reload lookup");
    (guard, temp, store, path)
}

fn empty_bucket_prune_store() -> (HjmtEnvGuard, SettlementStore, SettlementPath) {
    let guard = HjmtEnvGuard::with_bits("2");
    let target = asset_seed(0xA2, 6, 1, 62_000);
    let target_item = asset_item(&target);
    let target_path = settlement_corpus::asset_path(&target);

    let baseline_bucket = {
        let mut store = SettlementStore::new();
        store
            .put_settlement_item(target_item.clone())
            .expect("seed target bucket");
        store
            .adaptive_bucket(&target_path)
            .expect("target bucket")
            .bucket_id
    };

    for terminal in 2u8..=u8::MAX {
        let candidate = asset_seed(0xA2, 6, terminal, 62_000 + u64::from(terminal));
        let candidate_path = settlement_corpus::asset_path(&candidate);
        let mut store = SettlementStore::new();
        store
            .put_settlement_item(target_item.clone())
            .expect("seed target bucket");
        store
            .put_settlement_item(asset_item(&candidate))
            .expect("seed sibling bucket");
        let candidate_bucket = store
            .adaptive_bucket(&candidate_path)
            .expect("candidate bucket")
            .bucket_id;
        if candidate_bucket != baseline_bucket {
            return (guard, store, target_path);
        }
    }

    panic!("failed to find empty-bucket prune fixture")
}

fn empty_serial_prune_store() -> (HjmtEnvGuard, SettlementStore, SettlementPath) {
    let guard = HjmtEnvGuard::with_bits("2");
    let target = asset_seed(0xA3, 7, 1, 63_000);
    let keep = asset_seed(0xA3, 8, 2, 63_100);
    let mut store = SettlementStore::new();
    store
        .apply_settlement_ops(put_ops(&[asset_item(&target), asset_item(&keep)]))
        .expect("seed empty serial prune");
    (guard, store, settlement_corpus::asset_path(&target))
}

fn empty_definition_prune_store() -> (HjmtEnvGuard, SettlementStore, SettlementPath) {
    let guard = HjmtEnvGuard::with_bits("2");
    let target = asset_seed(0xA4, 7, 1, 64_000);
    let keep = asset_seed(0xA5, 7, 2, 64_100);
    let mut store = SettlementStore::new();
    store
        .apply_settlement_ops(put_ops(&[asset_item(&target), asset_item(&keep)]))
        .expect("seed empty definition prune");
    (guard, store, settlement_corpus::asset_path(&target))
}

fn root_lane_leaf(
    shard_id: u32,
    route_table_digest: [u8; 32],
    policy_set_digest: [u8; 32],
) -> ShardRootLeafV1 {
    let shard_mark = u8::try_from(shard_id).expect("u8");
    ShardRootLeafV1::new(
        shard_id,
        [0x80u8.wrapping_add(shard_mark); 32],
        20 + u64::from(shard_id),
        7,
        route_table_digest,
        policy_set_digest,
        100 + u64::from(shard_id),
        200 + u64::from(shard_id),
        0,
    )
}

fn root_of_roots_publication(shard_count: usize) -> CheckpointPublicationV1 {
    let route_table_digest = [0x42; 32];
    let policy_set_digest = [0x51; 32];
    let shard_leaves = (0..shard_count)
        .map(|idx| {
            root_lane_leaf(
                u32::try_from(idx + 1).expect("u32"),
                route_table_digest,
                policy_set_digest,
            )
        })
        .collect();
    CheckpointPublicationV1::new(
        RootGenerationTagV1::RootGeneration1,
        PublicationModeTagV1::Synchronous,
        100 + u64::try_from(shard_count).expect("u64"),
        route_table_digest,
        SettlementStateRoot::settlement_v1([0x11; 32]),
        shard_leaves,
    )
}

fn bench_search_read(c: &mut Criterion) {
    let fixture = load_fixture();
    let asset = fixture.assets[0].clone();
    let right = fixture.rights[0].clone();
    let path = settlement_corpus::asset_path(&asset);
    let missing = settlement_corpus::asset_path(&asset_seed(0x55, 99, 0xEE, 44_000));
    let right_path = right_path(&right);
    let mut group = c.benchmark_group("search_read");

    group.bench_function("full_path_cold", |b| {
        b.iter_batched(
            base_store,
            |store| {
                black_box(
                    store
                        .lookup_settlement(SettlementLookup::Path(path))
                        .expect("path lookup"),
                );
            },
            BatchSize::SmallInput,
        )
    });
    group.bench_function("full_path_warm", |b| {
        b.iter_batched(
            || {
                let store = base_store();
                let _ = store.settlement_proof_blob(&path).expect("warm proof");
                store
            },
            |store| {
                black_box(
                    store
                        .lookup_settlement(SettlementLookup::Path(path))
                        .expect("warm path lookup"),
                );
            },
            BatchSize::SmallInput,
        )
    });
    group.bench_function("terminal_lookup", |b| {
        b.iter_batched(
            base_store,
            |store| {
                black_box(
                    store
                        .lookup_settlement(SettlementLookup::Terminal(right_path.terminal_id))
                        .expect("terminal lookup"),
                );
            },
            BatchSize::SmallInput,
        )
    });
    group.bench_function("path_index_lookup", |b| {
        b.iter_batched(
            path_index_store,
            |(store, path)| {
                black_box(
                    store
                        .lookup_settlement(SettlementLookup::Terminal(path.terminal_id))
                        .expect("path index lookup"),
                );
            },
            BatchSize::SmallInput,
        )
    });
    group.bench_function("definition_list", |b| {
        b.iter_batched(
            base_store,
            |store| {
                black_box(
                    store
                        .list_settlement(SettlementListReq::for_def(path.definition_id, 16))
                        .expect("definition list"),
                );
            },
            BatchSize::SmallInput,
        )
    });
    group.bench_function("serial_list", |b| {
        b.iter_batched(
            base_store,
            |store| {
                black_box(
                    store
                        .list_settlement(SettlementListReq::for_ser(
                            path.definition_id,
                            path.serial_id,
                            16,
                        ))
                        .expect("serial list"),
                );
            },
            BatchSize::SmallInput,
        )
    });
    group.bench_function("right_class_list", |b| {
        b.iter_batched(
            || {
                let mut items = hot_rights(0x71, 7, 8, FixtureRightClass::MachineCapability);
                items.extend((101u8..=108u8).map(|terminal_mark| {
                    right_item(&right_seed(
                        0x71,
                        8,
                        terminal_mark,
                        FixtureRightClass::DataAccess,
                    ))
                }));
                seed_mem(&items)
            },
            |store| {
                black_box(
                    store
                        .list_settlement(SettlementListReq::for_right_class(
                            z00z_storage::settlement::RightClass::DataAccess,
                            16,
                        ))
                        .expect("right class list"),
                );
            },
            BatchSize::SmallInput,
        )
    });
    group.bench_function("paginated_list", |b| {
        b.iter_batched(
            base_store,
            |store| {
                let page = store
                    .list_settlement(SettlementListReq::all(4))
                    .expect("first page");
                black_box(
                    store
                        .list_settlement(SettlementListReq::all(4).with_after(page.next()))
                        .expect("next page"),
                );
            },
            BatchSize::SmallInput,
        )
    });
    group.bench_function("absent_nonexistence", |b| {
        b.iter_batched(
            base_store,
            |store| {
                let blob = store
                    .settlement_nonexistence_proof_blob(&missing, SettlementLeafFamily::Terminal)
                    .expect("nonexistence proof");
                store
                    .validate_settlement_nonexistence_proof_blob(
                        &blob,
                        SettlementLeafFamily::Terminal,
                    )
                    .expect("nonexistence verify");
                black_box(());
            },
            BatchSize::SmallInput,
        )
    });
    group.bench_function("post_reload_lookup", |b| {
        b.iter_batched(
            reload_lookup_store,
            |(_guard, _temp, store, path)| {
                black_box(
                    store
                        .lookup_settlement(SettlementLookup::Path(path))
                        .expect("post reload lookup"),
                );
            },
            BatchSize::SmallInput,
        )
    });
    group.finish();
}

fn bench_insert(c: &mut Criterion) {
    let asset = asset_seed(0x91, 3, 8, 51_000);
    let right = right_seed(0x91, 4, 9, FixtureRightClass::ValidatorMandate);
    let mixed = mixed_items();
    let fixture = load_fixture();
    let warm_path = settlement_corpus::asset_path(&fixture.assets[0]);
    let warm_asset = asset_seed(
        fixture.assets[0].definition_mark,
        fixture.assets[0].serial_id,
        0xF1,
        51_111,
    );
    let mut group = c.benchmark_group("insert");
    group.bench_function("single_asset", |b| {
        b.iter_batched(
            base_store,
            |mut store| {
                black_box(
                    store
                        .put_settlement_item(asset_item(&asset))
                        .expect("single asset put"),
                );
            },
            BatchSize::SmallInput,
        )
    });
    group.bench_function("single_right_fee", |b| {
        b.iter_batched(
            || {
                let _guard = HjmtEnvGuard::with_bits("2");
                SettlementStore::new()
            },
            |mut store| {
                black_box(create_right_fee(&mut store, &right, 77).expect("right fee"));
            },
            BatchSize::SmallInput,
        )
    });
    group.bench_function("create_right_support_required", |b| {
        b.iter_batched(
            || {
                let _guard = HjmtEnvGuard::with_bits("2");
                SettlementStore::new()
            },
            |mut store| {
                expect_support_required(store.create_right(
                    right_path(&right),
                    right_leaf(&right),
                    right_ctx(&right_leaf(&right), 15),
                ));
                black_box(());
            },
            BatchSize::SmallInput,
        )
    });
    group.bench_function("mixed_batch", |b| {
        b.iter_batched(
            base_store,
            |mut store| {
                black_box(
                    store
                        .apply_settlement_ops(put_ops(&mixed))
                        .expect("mixed batch"),
                );
            },
            BatchSize::SmallInput,
        )
    });
    group.bench_function("duplicate_reject", |b| {
        b.iter_batched(
            SettlementStore::new,
            |mut store| {
                let item = asset_item(&asset);
                black_box(
                    store
                        .apply_settlement_ops(vec![
                            z00z_storage::settlement::StoreOp::Put(Box::new(item.clone())),
                            z00z_storage::settlement::StoreOp::Put(Box::new(item)),
                        ])
                        .expect_err("duplicate reject"),
                );
            },
            BatchSize::SmallInput,
        )
    });
    group.bench_function("rejected_fee_transition", |b| {
        b.iter_batched(
            || {
                let _guard = HjmtEnvGuard::with_bits("2");
                let mut store = SettlementStore::new();
                let _ = create_right_fee(&mut store, &right, 85).expect("seed right");
                store
            },
            |mut store| {
                let path = right_path(&right);
                let prior = right_leaf(&right);
                let next = transferred_right_leaf(prior, right.terminal_mark);
                let env = fee_envelope(
                    86,
                    store
                        .fee_support_ctx(&fee_del_ops(path))
                        .expect("wrong fee support"),
                );
                black_box(
                    store
                        .transfer_right_with_fee(
                            path,
                            next,
                            right_ctx(&next, 15),
                            env,
                            fee_actor(86, 15),
                        )
                        .expect_err("fee transition reject"),
                );
            },
            BatchSize::SmallInput,
        )
    });
    group.finish();

    let mut cache_group = c.benchmark_group("insert_cache_state");
    cache_group.bench_function("cold_single_asset", |b| {
        b.iter_batched(
            base_store,
            |mut store| {
                black_box(
                    store
                        .put_settlement_item(asset_item(&warm_asset))
                        .expect("cold insert asset"),
                );
            },
            BatchSize::SmallInput,
        )
    });
    cache_group.bench_function("warm_single_asset", |b| {
        b.iter_batched(
            || {
                let store = base_store();
                let _ = store
                    .settlement_proof_blob(&warm_path)
                    .expect("warm insert seed");
                store
            },
            |mut store| {
                black_box(
                    store
                        .put_settlement_item(asset_item(&warm_asset))
                        .expect("warm insert asset"),
                );
            },
            BatchSize::SmallInput,
        )
    });
    cache_group.finish();
}

fn bench_delete(c: &mut Criterion) {
    let asset = asset_seed(0xA1, 3, 8, 61_000);
    let right = right_seed(0xA1, 4, 9, FixtureRightClass::MachineCapability);
    let mixed = mixed_items();
    let missing = settlement_corpus::asset_path(&asset_seed(0xA1, 99, 31, 91_000));
    let mut group = c.benchmark_group("delete_prune");
    group.bench_function("single_asset_delete", |b| {
        b.iter_batched(
            || seed_mem(&[asset_item(&asset)]),
            |mut store| {
                black_box(
                    store
                        .del_settlement_item(&settlement_corpus::asset_path(&asset))
                        .expect("asset delete"),
                );
            },
            BatchSize::SmallInput,
        )
    });
    group.bench_function("batch_delete", |b| {
        b.iter_batched(
            || seed_mem(&mixed),
            |mut store| {
                black_box(
                    store
                        .apply_settlement_ops(del_ops(&mixed))
                        .expect("batch delete"),
                );
            },
            BatchSize::SmallInput,
        )
    });
    group.bench_function("right_consume_fee", |b| {
        b.iter_batched(
            || {
                let _guard = HjmtEnvGuard::with_bits("2");
                let mut store = SettlementStore::new();
                let _ = create_right_fee(&mut store, &right, 81).expect("seed right");
                store
            },
            |mut store| {
                black_box(consume_right_fee(&mut store, &right, 82).expect("consume right"));
            },
            BatchSize::SmallInput,
        )
    });
    group.bench_function("consume_right_support_required", |b| {
        b.iter_batched(
            || {
                let _guard = HjmtEnvGuard::with_bits("2");
                let mut store = SettlementStore::new();
                let _ = create_right_fee(&mut store, &right, 81).expect("seed right");
                store
            },
            |mut store| {
                expect_support_required(
                    store.consume_right(right_path(&right), right_ctx(&right_leaf(&right), 15)),
                );
                black_box(());
            },
            BatchSize::SmallInput,
        )
    });
    group.bench_function("right_revoke_fee", |b| {
        b.iter_batched(
            || {
                let _guard = HjmtEnvGuard::with_bits("2");
                let mut store = SettlementStore::new();
                let _ = create_right_fee(&mut store, &right, 83).expect("seed right");
                store
            },
            |mut store| {
                black_box(revoke_right_fee(&mut store, &right, 84).expect("revoke right"));
            },
            BatchSize::SmallInput,
        )
    });
    group.bench_function("revoke_right_support_required", |b| {
        b.iter_batched(
            || {
                let _guard = HjmtEnvGuard::with_bits("2");
                let mut store = SettlementStore::new();
                let _ = create_right_fee(&mut store, &right, 83).expect("seed right");
                store
            },
            |mut store| {
                expect_support_required(
                    store.revoke_right(right_path(&right), right_ctx(&right_leaf(&right), 15)),
                );
                black_box(());
            },
            BatchSize::SmallInput,
        )
    });
    group.bench_function("right_expire", |b| {
        b.iter_batched(
            || {
                let _guard = HjmtEnvGuard::with_bits("2");
                let mut store = SettlementStore::new();
                let _ = create_right_fee(&mut store, &right, 85).expect("seed right");
                store
            },
            |mut store| {
                let leaf = right_leaf(&right);
                black_box(
                    store
                        .expire_right(right_path(&right), right_ctx(&leaf, 25))
                        .expect("expire right"),
                );
            },
            BatchSize::SmallInput,
        )
    });
    group.bench_function("empty_bucket_prune", |b| {
        b.iter_batched(
            empty_bucket_prune_store,
            |(_guard, mut store, path)| {
                black_box(
                    store
                        .del_settlement_item(&path)
                        .expect("empty bucket prune"),
                );
            },
            BatchSize::SmallInput,
        )
    });
    group.bench_function("empty_serial_prune", |b| {
        b.iter_batched(
            empty_serial_prune_store,
            |(_guard, mut store, path)| {
                black_box(
                    store
                        .del_settlement_item(&path)
                        .expect("empty serial prune"),
                );
            },
            BatchSize::SmallInput,
        )
    });
    group.bench_function("empty_definition_prune", |b| {
        b.iter_batched(
            empty_definition_prune_store,
            |(_guard, mut store, path)| {
                black_box(
                    store
                        .del_settlement_item(&path)
                        .expect("empty definition prune"),
                );
            },
            BatchSize::SmallInput,
        )
    });
    group.bench_function("missing_delete_reject", |b| {
        b.iter_batched(
            SettlementStore::new,
            |mut store| {
                black_box(
                    store
                        .del_settlement_item(&missing)
                        .expect_err("missing delete reject"),
                );
            },
            BatchSize::SmallInput,
        )
    });
    group.finish();

    let delete_asset = asset_seed(0xA2, 5, 10, 62_000);
    let delete_path = settlement_corpus::asset_path(&delete_asset);
    let mut cache_group = c.benchmark_group("delete_cache_state");
    cache_group.bench_function("cold_single_asset_delete", |b| {
        b.iter_batched(
            || seed_mem(&[asset_item(&delete_asset)]),
            |mut store| {
                black_box(
                    store
                        .del_settlement_item(&delete_path)
                        .expect("cold delete asset"),
                );
            },
            BatchSize::SmallInput,
        )
    });
    cache_group.bench_function("warm_single_asset_delete", |b| {
        b.iter_batched(
            || {
                let store = seed_mem(&[asset_item(&delete_asset)]);
                let _ = store
                    .settlement_proof_blob(&delete_path)
                    .expect("warm delete seed");
                store
            },
            |mut store| {
                black_box(
                    store
                        .del_settlement_item(&delete_path)
                        .expect("warm delete asset"),
                );
            },
            BatchSize::SmallInput,
        )
    });
    cache_group.finish();
}

fn bench_cache(c: &mut Criterion) {
    let asset = load_fixture().assets[0].clone();
    let path = settlement_corpus::asset_path(&asset);
    let dirty = asset_seed(0xB1, 8, 11, 71_000);
    let mut group = c.benchmark_group("cache");
    group.bench_function("cold_proof", |b| {
        b.iter_batched(
            base_store,
            |store| {
                black_box(store.settlement_proof_blob(&path).expect("cold proof"));
            },
            BatchSize::SmallInput,
        )
    });
    group.bench_function("warm_proof", |b| {
        b.iter_batched(
            || {
                let store = base_store();
                let _ = store.settlement_proof_blob(&path).expect("warm seed");
                store
            },
            |store| {
                black_box(store.settlement_proof_blob(&path).expect("warm proof"));
            },
            BatchSize::SmallInput,
        )
    });
    group.bench_function("dirty_small", |b| {
        b.iter_batched(
            || {
                let store = base_store();
                let _ = store.settlement_proof_blob(&path).expect("warm proof");
                store
            },
            |mut store| {
                let _ = store
                    .put_settlement_item(asset_item(&dirty))
                    .expect("dirty insert");
                black_box(store.settlement_proof_blob(&path).expect("dirty proof"));
            },
            BatchSize::SmallInput,
        )
    });
    group.bench_function("dirty_hot", |b| {
        b.iter_batched(
            || {
                let items = hot_assets(0xB2, 9, 20);
                let path = items[0].path();
                let seed = items[..16].to_vec();
                let dirty = items[16..].to_vec();
                let store = seed_mem(&seed);
                let _ = store.settlement_proof_blob(&path).expect("warm proof");
                (store, path, dirty)
            },
            |(mut store, path, dirty)| {
                let _ = store
                    .apply_settlement_ops(put_ops(&dirty))
                    .expect("dirty hot insert");
                black_box(store.settlement_proof_blob(&path).expect("dirty hot proof"));
            },
            BatchSize::SmallInput,
        )
    });
    group.bench_function("dirty_wide", |b| {
        b.iter_batched(
            || {
                let seed = hot_assets(0xB3, 11, 8);
                let path = seed[0].path();
                let dirty = (0..8)
                    .map(|idx| {
                        asset_item(&asset_seed(
                            0xC0 + u8::try_from(idx).expect("u8"),
                            1,
                            u8::try_from(idx + 101).expect("u8"),
                            81_000 + idx as u64,
                        ))
                    })
                    .collect::<Vec<_>>();
                let store = seed_mem(&seed);
                let _ = store.settlement_proof_blob(&path).expect("warm proof");
                (store, path, dirty)
            },
            |(mut store, path, dirty)| {
                let _ = store
                    .apply_settlement_ops(put_ops(&dirty))
                    .expect("dirty wide insert");
                black_box(
                    store
                        .settlement_proof_blob(&path)
                        .expect("dirty wide proof"),
                );
            },
            BatchSize::SmallInput,
        )
    });
    group.bench_function("proof_heavy", |b| {
        b.iter_batched(
            || {
                let items = hot_assets(0xB4, 13, 32);
                let paths = items
                    .iter()
                    .take(24)
                    .map(|item| item.path())
                    .collect::<Vec<_>>();
                let store = seed_mem(&items);
                let _ = store
                    .settlement_proof_blobs(&paths)
                    .expect("warm proof batch");
                (store, paths)
            },
            |(store, paths)| {
                black_box(
                    store
                        .settlement_proof_blobs(&paths)
                        .expect("proof heavy batch"),
                );
            },
            BatchSize::SmallInput,
        )
    });
    group.bench_function("policy_transition_heavy", |b| {
        b.iter_batched(
            || {
                let guard = HjmtEnvGuard::with_bits("1");
                let mut store = SettlementStore::new();
                let _ = split_ready_paths(&mut store, 41, 9);
                let extra = (0..24)
                    .map(|idx| {
                        asset_item(&asset_seed(
                            0xB5,
                            14,
                            u8::try_from(idx + 101).expect("u8"),
                            91_000 + idx as u64,
                        ))
                    })
                    .collect::<Vec<_>>();
                let _ = store
                    .apply_settlement_ops(put_ops(&extra))
                    .expect("policy transition seed");
                let next = next_policy(&store);
                let _ = store
                    .policy_transition_proof(next)
                    .expect("warm policy transition");
                (guard, store, next)
            },
            |(_guard, store, next)| {
                black_box(
                    store
                        .policy_transition_proof(next)
                        .expect("policy transition heavy"),
                );
            },
            BatchSize::SmallInput,
        )
    });
    group.finish();
}

fn bench_reload(c: &mut Criterion) {
    let mut group = c.benchmark_group("reload_cache_state");
    group.bench_function("cold_lookup", |b| {
        b.iter_batched(
            reload_lookup_store,
            |(_guard, _temp, store, path)| {
                black_box(
                    store
                        .lookup_settlement(SettlementLookup::Path(path))
                        .expect("cold reload lookup"),
                );
            },
            BatchSize::SmallInput,
        )
    });
    group.bench_function("warm_lookup", |b| {
        b.iter_batched(
            warm_reload_lookup_store,
            |(_guard, _temp, store, path)| {
                black_box(
                    store
                        .lookup_settlement(SettlementLookup::Path(path))
                        .expect("warm reload lookup"),
                );
            },
            BatchSize::SmallInput,
        )
    });
    group.finish();
}

fn bench_root_of_roots_publish(c: &mut Criterion) {
    let mut group = c.benchmark_group("root_of_roots_publish");
    for shard_count in [1usize, 2, 4, 8, 16] {
        let lane = format!("shards_{shard_count}");
        group.bench_function(&lane, |b| {
            b.iter_batched(
                || root_of_roots_publication(shard_count),
                |publication| {
                    black_box(publication.public_root_v1().expect("public root"));
                },
                BatchSize::SmallInput,
            )
        });
    }
    group.finish();
}

fn repo_storage_cfg_path() -> std::path::PathBuf {
    std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../config/hjmt_runtime/sim_5a7s/storage/storage-config.yaml")
}

fn sim_cache_capacity() -> usize {
    let cfg: BenchStorageCfg = YamlCodec
        .deserialize(&io::read_file(repo_storage_cfg_path()).expect("read SIM-5A7S storage config"))
        .expect("parse SIM-5A7S storage config");
    cfg.settings.cache_capacity
}

fn cache_edge_path(idx: usize) -> SettlementPath {
    let idx = u32::try_from(idx + 1).expect("u32");
    let mut terminal = [0u8; 32];
    terminal[..4].copy_from_slice(&idx.to_be_bytes());
    terminal[4] = 0xD7;
    SettlementPath::new(
        DefinitionId::new([0xD7; 32]),
        SerialId::new(idx),
        TerminalId::new(terminal),
    )
}

fn cache_edge_item(idx: usize) -> StoreItem {
    let path = cache_edge_path(idx);
    let mut core = AssetLeaf::dummy_for_scan(path.serial_id.get());
    core.asset_id = path.terminal_id().into_bytes();
    let leaf = SettlementLeaf::Terminal(TerminalLeaf::from(core));
    StoreItem::new(path, leaf).expect("cache-edge item")
}

fn cache_edge_queue(count: usize) -> usize {
    count.max(4096).saturating_mul(8)
}

fn cache_edge_seed(count: usize) -> (HjmtEnvGuard, SchedEnv, SettlementStore, SettlementPath) {
    let items = (0..count.max(1)).map(cache_edge_item).collect::<Vec<_>>();
    let path = items[0].path();
    let guard = HjmtEnvGuard::with_bits("2");
    let sched = SchedEnv::new(4, cache_edge_queue(count));
    let mut store = SettlementStore::new();
    store
        .apply_settlement_ops(put_ops(&items))
        .expect("seed cache-edge fixture");
    (guard, sched, store, path)
}

fn bench_cache_edge_support(c: &mut Criterion) {
    let cap = sim_cache_capacity();
    let mut group = c.benchmark_group("cache_edge_support");
    group.bench_function("cap_minus_1", |b| {
        let count = cap.saturating_sub(1).max(1);
        let (_guard, _sched, store, path) = cache_edge_seed(count);
        b.iter(|| {
            black_box(
                store
                    .settlement_proof_blob(&path)
                    .expect("cache-edge cap_minus_1"),
            );
        })
    });
    group.bench_function("cap", |b| {
        let (_guard, _sched, store, path) = cache_edge_seed(cap);
        b.iter(|| {
            black_box(store.settlement_proof_blob(&path).expect("cache-edge cap"));
        })
    });
    group.bench_function("cap_plus_1", |b| {
        let count = cap + 1;
        let (_guard, _sched, store, path) = cache_edge_seed(count);
        b.iter(|| {
            black_box(
                store
                    .settlement_proof_blob(&path)
                    .expect("cache-edge cap_plus_1"),
            );
        })
    });
    group.bench_function("cap_times_2", |b| {
        let count = cap * 2;
        let (_guard, _sched, store, path) = cache_edge_seed(count);
        b.iter(|| {
            black_box(
                store
                    .settlement_proof_blob(&path)
                    .expect("cache-edge cap_times_2"),
            );
        })
    });
    group.finish();
}

fn bench_sched(c: &mut Criterion) {
    let items = hot_assets(0xC1, 12, 48);
    let paths = items.iter().map(|item| item.path()).collect::<Vec<_>>();
    let mut group = c.benchmark_group("scheduler");
    group.bench_function("serial_batch", |b| {
        b.iter_batched(
            || {
                let _sched = SchedEnv::new(1, 64);
                let _mode = ProofBatchModeEnv::serial();
                let store = seed_mem(&items);
                (_sched, _mode, store)
            },
            |(_sched, _mode, store)| {
                black_box(store.settlement_proof_blobs(&paths).expect("serial batch"));
            },
            BatchSize::SmallInput,
        )
    });
    group.bench_function("parallel_batch", |b| {
        b.iter_batched(
            || {
                let _sched = SchedEnv::new(4, 64);
                let _mode = ProofBatchModeEnv::parallel();
                let store = seed_mem(&items);
                (_sched, _mode, store)
            },
            |(_sched, _mode, store)| {
                black_box(
                    store
                        .settlement_proof_blobs(&paths)
                        .expect("parallel batch"),
                );
            },
            BatchSize::SmallInput,
        )
    });
    group.bench_function("backpressure_reject", |b| {
        b.iter_batched(
            || {
                let _guard = HjmtEnvGuard::with_bits("2");
                let _sched = SchedEnv::new(2, 1);
                let store = SettlementStore::new();
                (_sched, store)
            },
            |(_sched, mut store)| {
                let err = store
                    .apply_settlement_ops(put_ops(&items))
                    .expect_err("backpressure reject");
                match err {
                    SettlementStoreError::SchedBackpressure {
                        stage,
                        queued,
                        limit,
                    } => {
                        assert_eq!(stage, "hjmt_plan_ops");
                        assert!(queued > limit);
                        black_box((queued, limit));
                    }
                    other => panic!("unexpected error: {other:?}"),
                }
            },
            BatchSize::SmallInput,
        )
    });
    group.finish();
}

fn write_diag_note() {
    let sim_cache_capacity = sim_cache_capacity();
    let fixture = load_fixture();
    let sample_path = settlement_corpus::asset_path(&fixture.assets[0]);
    let path_index_path = settlement_corpus::asset_path(&fixture.assets[1]);
    let right = right_seed(0xE1, 7, 9, FixtureRightClass::ServiceEntitlement);
    let (
        cold_ms,
        warm_ms,
        path_index_lookup_us,
        post_reload_lookup_us,
        cold_bytes,
        warm_bytes,
        cold_proof_alloc_bytes,
        cache_hits,
        cache_misses,
        invalidations,
        path_index_hits,
        path_index_misses,
        root_reuse_ratio,
        proof_reuse_ratio,
        sched_depth,
        sched_rejects,
        mem_before,
        redb_file_bytes,
        blocking_thread,
        blocking_wait_us,
        reload_root,
    ) = {
        let (_guard, temp, mut store) = seed_redb("2", &load_fixture_items(&fixture));
        let mem_before = statm_resident().unwrap_or(0);
        let start = Instant::now();
        let alloc_region = Region::new(GLOBAL);
        let cold = store
            .settlement_proof_blob(&sample_path)
            .expect("cold proof");
        let cold_proof_alloc_bytes = alloc_region.change().bytes_allocated;
        let cold_ms = start.elapsed().as_micros();
        let warm_start = Instant::now();
        let warm = store
            .settlement_proof_blob(&sample_path)
            .expect("warm proof");
        let warm_ms = warm_start.elapsed().as_micros();
        let path_index_start = Instant::now();
        let _ = store
            .lookup_settlement(SettlementLookup::Terminal(path_index_path.terminal_id))
            .expect("path index lookup");
        let path_index_lookup_us = path_index_start.elapsed().as_micros();
        let _ = create_right_fee(&mut store, &right, 91).expect("create right");
        let _ = revoke_right_fee(&mut store, &right, 92).expect("revoke right");
        let cache = store.forest_cache_metrics();
        let sched = store.forest_scheduler_metrics();
        let cold_bytes = cold.encode().expect("cold bytes").len();
        let warm_bytes = warm.encode().expect("warm bytes").len();
        let cache_hits = cache_hits(&cache);
        let cache_misses = cache_misses(&cache);
        let invalidations = cache_invalidations(&cache);
        let path_index_hits = cache.path_index.hits;
        let path_index_misses = cache.path_index.misses;
        let root_reuse_ratio = reuse_ratio(cache.subtree_root.hits, cache.subtree_root.misses);
        let proof_reuse_ratio = reuse_ratio(cache.proof_segment.hits, cache.proof_segment.misses);
        let sched_depth = sched.max_queued;
        let sched_rejects = sched.reject_count;
        let blocking_thread = sched
            .last_blocking_thread
            .unwrap_or_else(|| "none".to_string());
        let blocking_wait_us = sched.last_blocking_wait_us;
        drop(store);
        let reloaded = SettlementStore::load(temp.path()).expect("reload");
        let reload_root = reloaded.settlement_root().expect("reload root");
        let lookup_start = Instant::now();
        let _ = reloaded
            .lookup_settlement(SettlementLookup::Path(sample_path))
            .expect("lookup after reload");
        let post_reload_lookup_us = lookup_start.elapsed().as_micros();
        let redb_file_bytes = redb_bytes(temp.path());
        (
            cold_ms,
            warm_ms,
            path_index_lookup_us,
            post_reload_lookup_us,
            cold_bytes,
            warm_bytes,
            cold_proof_alloc_bytes,
            cache_hits,
            cache_misses,
            invalidations,
            path_index_hits,
            path_index_misses,
            root_reuse_ratio,
            proof_reuse_ratio,
            sched_depth,
            sched_rejects,
            mem_before,
            redb_file_bytes,
            blocking_thread,
            blocking_wait_us,
            reload_root,
        )
    };
    let split_us = {
        let _guard = HjmtEnvGuard::with_bits("1");
        let mut store = SettlementStore::new();
        let left = settlement_corpus::split_ready_paths(&mut store, 41, 9)[0];
        let start = Instant::now();
        let _ = store.split_proof(&left).expect("split proof");
        start.elapsed().as_micros()
    };
    let merge_us = {
        let _guard = HjmtEnvGuard::with_bits("2");
        let mut store = SettlementStore::new();
        let (left, right) = sibling_bucket_pair(&mut store, 33, 11);
        let start = Instant::now();
        let _ = store.merge_proof(&left, &right).expect("merge proof");
        start.elapsed().as_micros()
    };
    let policy_transition_us = {
        let _guard = HjmtEnvGuard::with_bits("1");
        let mut store = SettlementStore::new();
        let left = settlement_corpus::split_ready_paths(&mut store, 41, 9)[0];
        let _ = left;
        let start = Instant::now();
        let _ = store
            .policy_transition_proof(next_policy(&store))
            .expect("policy transition");
        start.elapsed().as_micros()
    };
    let backpressure_ops_us = {
        let _guard = HjmtEnvGuard::with_bits("2");
        let _sched = SchedEnv::new(2, 1);
        let mut store = SettlementStore::new();
        let items = hot_assets(0xD1, 17, 48);
        let start = Instant::now();
        let err = store
            .apply_settlement_ops(put_ops(&items))
            .expect_err("backpressure evidence");
        match err {
            SettlementStoreError::SchedBackpressure { .. } => start.elapsed().as_micros(),
            other => panic!("unexpected error: {other:?}"),
        }
    };
    let mut note = String::new();
    note.push_str("# HJMT Bench Diagnostics\n\n");
    note.push_str(&format!(
        "- cold_proof_us: `{cold_ms}`\n- warm_proof_us: `{warm_ms}`\n- path_index_lookup_us: `{path_index_lookup_us}`\n- post_reload_lookup_us: `{post_reload_lookup_us}`\n"
    ));
    note.push_str(&format!(
        "- cold_proof_bytes: `{cold_bytes}`\n- warm_proof_bytes: `{warm_bytes}`\n- cold_proof_alloc_bytes: `{cold_proof_alloc_bytes}`\n",
    ));
    note.push_str(&format!(
        "- cache_hits: `{cache_hits}`\n- cache_misses: `{cache_misses}`\n- cache_invalidations: `{invalidations}`\n- path_index_hits: `{path_index_hits}`\n- path_index_misses: `{path_index_misses}`\n- root_reuse_ratio: `{root_reuse_ratio:.3}`\n- proof_reuse_ratio: `{proof_reuse_ratio:.3}`\n- scheduler_queue_depth: `{sched_depth}`\n- scheduler_rejects: `{sched_rejects}`\n",
    ));
    note.push_str(&format!(
        "- resident_bytes: `{mem_before}`\n- redb_file_bytes: `{redb_file_bytes}`\n- last_blocking_thread: `{blocking_thread}`\n- last_blocking_wait_us: `{blocking_wait_us}`\n- reload_root: `{:?}`\n",
        reload_root
    ));
    note.push_str(&format!(
        "- split_trigger_us: `{split_us}`\n- merge_trigger_us: `{merge_us}`\n- policy_transition_us: `{policy_transition_us}`\n- backpressure_ops_us: `{backpressure_ops_us}`\n",
    ));
    note.push_str(&format!(
        "- sim_cache_capacity: `{sim_cache_capacity}`\n- cache_edge_support: `cache_edge_support/cap_minus_1`, `cache_edge_support/cap`, `cache_edge_support/cap_plus_1`, `cache_edge_support/cap_times_2`\n- root_of_roots_publish: `root_of_roots_publish/shards_1`, `root_of_roots_publish/shards_2`, `root_of_roots_publish/shards_4`, `root_of_roots_publish/shards_8`, `root_of_roots_publish/shards_16`\n- shard_root_fixture_manifest: `{SHARD_ROOT_FIXTURE_MANIFEST}`\n- checkpoint_publication_fixture_manifest: `{CHECKPOINT_FIXTURE_MANIFEST}`\n",
    ));
    write_note("settlement_hjmt_diag.md", &note);
}

fn main() {
    let _sched = SchedEnv::new(4, 4096);
    if should_emit_side_outputs() {
        write_meta(BenchMeta::new(
            "settlement_hjmt",
            "cargo bench -p z00z_storage --bench settlement_hjmt",
        ));
        write_diag_note();
    }
    let mut crit = Criterion::default().configure_from_args();
    bench_search_read(&mut crit);
    bench_insert(&mut crit);
    bench_delete(&mut crit);
    bench_cache(&mut crit);
    bench_reload(&mut crit);
    bench_root_of_roots_publish(&mut crit);
    bench_cache_edge_support(&mut crit);
    bench_sched(&mut crit);
    crit.final_summary();
}
