use std::time::Instant;

use criterion::{black_box, BatchSize, Criterion};
use rayon::prelude::*;
use z00z_storage::{
    fixture_support::{
        settlement_bench_output::{should_emit_side_outputs, write_meta, write_note, BenchMeta},
        settlement_corpus::{
            asset_item, asset_seed, del_ops, hot_assets, hot_rights, many_defs, many_sers, put_ops,
            seed_mem, seed_redb, FixtureRightClass, HjmtEnvGuard, SchedEnv,
        },
    },
    settlement::{
        CheckpointPublicationV1, PublicationModeTagV1, RootGenerationTagV1, SettlementStateRoot,
        SettlementStore, ShardRootLeafV1,
    },
};

const RUNTIME_FIXTURE_MANIFEST: &str = "config/hjmt_runtime/sim_5a7s/manifest.json";
const ROUTE_FIXTURE_MANIFEST: &str =
    "crates/z00z_storage/tests/fixtures/hjmt_upgrade/shard_route_table_v1/manifest.json";
const INITIAL_SHARD_SCALING_COUNTS: [usize; 5] = [1, 2, 4, 8, 16];
const INITIAL_SHARD_LOAD: usize = 16;
const GRID57_SHARDS: usize = 7;
const GRID57_SHARD_LOAD: usize = 24;

struct ShardScalingRow {
    lane: String,
    shard_count: usize,
    hot_shard_ratio: f64,
    global_cadence_us: u128,
    publication_latency_us: u128,
    shard_tps: f64,
    global_tps: f64,
    worker_local_tps: f64,
    durable_root_published_tps: f64,
    blocked_time_us: u128,
}

fn hot_bucket_assets(
    definition: u8,
    serial: u32,
    count: usize,
) -> Vec<z00z_storage::settlement::StoreItem> {
    let _guard = HjmtEnvGuard::with_bits("2");
    let mut store = SettlementStore::new();
    let first = asset_item(&asset_seed(definition, serial, 1, 40_000));
    let first_path = first.path();
    let _ = store
        .put_settlement_item(first.clone())
        .expect("seed first hot bucket item");
    let target_bucket = store
        .adaptive_bucket(&first_path)
        .expect("first adaptive bucket")
        .bucket_id;
    let mut items = vec![first];

    for seed in 2..=u8::MAX {
        let item = asset_item(&asset_seed(
            definition,
            serial,
            seed,
            40_000 + u64::from(seed),
        ));
        let path = item.path();
        let _ = store
            .put_settlement_item(item.clone())
            .expect("seed candidate hot bucket item");
        let bucket = store
            .adaptive_bucket(&path)
            .expect("candidate adaptive bucket")
            .bucket_id;
        if bucket == target_bucket {
            items.push(item);
            if items.len() == count {
                return items;
            }
        }
    }

    panic!("failed to build hot bucket assets");
}

fn bench_insert_many_defs(c: &mut Criterion) {
    let items = many_defs(96);
    let mut group = c.benchmark_group("insert_many_definitions");
    group.bench_function("generalized_assets", |b| {
        b.iter_batched(
            SettlementStore::new,
            |mut store| {
                black_box(
                    store
                        .apply_settlement_ops(put_ops(&items))
                        .expect("insert defs"),
                );
            },
            BatchSize::SmallInput,
        )
    });
    group.finish();
}

fn bench_insert_many_sers(c: &mut Criterion) {
    let items = many_sers(0x41, 96);
    let mut group = c.benchmark_group("insert_many_serials");
    group.bench_function("generalized_assets", |b| {
        b.iter_batched(
            SettlementStore::new,
            |mut store| {
                black_box(
                    store
                        .apply_settlement_ops(put_ops(&items))
                        .expect("insert sers"),
                );
            },
            BatchSize::SmallInput,
        )
    });
    group.finish();
}

fn bench_hot_definition(c: &mut Criterion) {
    let items = many_sers(0x41, 96);
    let mut group = c.benchmark_group("insert_hot_definition");
    group.bench_function("asset_batch", |b| {
        b.iter_batched(
            SettlementStore::new,
            |mut store| {
                black_box(
                    store
                        .apply_settlement_ops(put_ops(&items))
                        .expect("insert hot definition"),
                );
            },
            BatchSize::SmallInput,
        )
    });
    group.finish();
}

fn bench_hot_assets(c: &mut Criterion) {
    let items = hot_assets(0x51, 7, 128);
    let mut group = c.benchmark_group("insert_many_hot_serial");
    group.bench_function("asset_batch", |b| {
        b.iter_batched(
            SettlementStore::new,
            |mut store| {
                black_box(
                    store
                        .apply_settlement_ops(put_ops(&items))
                        .expect("insert hot serial assets"),
                );
            },
            BatchSize::SmallInput,
        )
    });
    let rights = hot_rights(0x51, 9, 64, FixtureRightClass::MachineCapability);
    group.bench_function("right_batch", |b| {
        b.iter_batched(
            SettlementStore::new,
            |mut store| {
                black_box(
                    store
                        .apply_settlement_ops(put_ops(&rights))
                        .expect("insert hot serial rights"),
                );
            },
            BatchSize::SmallInput,
        )
    });
    group.finish();
}

fn bench_hot_bucket(c: &mut Criterion) {
    let items = hot_bucket_assets(0x52, 8, 8);
    let mut group = c.benchmark_group("insert_hot_bucket");
    group.bench_function("asset_batch", |b| {
        b.iter_batched(
            SettlementStore::new,
            |mut store| {
                black_box(
                    store
                        .apply_settlement_ops(put_ops(&items))
                        .expect("insert hot bucket"),
                );
            },
            BatchSize::SmallInput,
        )
    });
    group.finish();
}

fn bench_delete_defs(c: &mut Criterion) {
    let items = many_defs(96);
    let ops = del_ops(&items);
    let mut group = c.benchmark_group("delete_many_definitions");
    group.bench_function("generalized_assets", |b| {
        b.iter_batched(
            || seed_mem(&items),
            |mut store| {
                black_box(
                    store
                        .apply_settlement_ops(ops.clone())
                        .expect("delete defs"),
                );
            },
            BatchSize::SmallInput,
        )
    });
    group.finish();
}

fn bench_delete_hot(c: &mut Criterion) {
    let items = hot_assets(0x62, 5, 128);
    let ops = del_ops(&items);
    let mut group = c.benchmark_group("delete_many_hot_serial");
    group.bench_function("generalized_assets", |b| {
        b.iter_batched(
            || seed_mem(&items),
            |mut store| {
                black_box(
                    store
                        .apply_settlement_ops(ops.clone())
                        .expect("delete hot serial"),
                );
            },
            BatchSize::SmallInput,
        )
    });
    group.finish();
}

fn bench_prove_many(c: &mut Criterion) {
    let items = hot_assets(0x73, 11, 64);
    let paths = items.iter().map(|item| item.path()).collect::<Vec<_>>();
    let mut group = c.benchmark_group("prove_many_assets");
    group.bench_function("shared_parent_batch", |b| {
        b.iter_batched(
            || seed_mem(&items),
            |store| {
                black_box(store.settlement_proof_blobs(&paths).expect("proof batch"));
            },
            BatchSize::SmallInput,
        )
    });
    group.finish();
}

fn shard_batches(
    shards: usize,
    items_per_shard: usize,
) -> Vec<Vec<z00z_storage::settlement::StoreItem>> {
    (0..shards)
        .map(|idx| {
            hot_assets(
                0x90u8.wrapping_add(u8::try_from(idx).expect("u8")),
                200 + u32::try_from(idx).expect("u32"),
                items_per_shard,
            )
        })
        .collect()
}

fn commit_shard_batches(
    batches: Vec<Vec<z00z_storage::settlement::StoreItem>>,
) -> Vec<z00z_storage::settlement::SettlementStateRoot> {
    batches
        .into_par_iter()
        .map(|items| {
            let mut store = SettlementStore::new();
            store
                .apply_settlement_ops(put_ops(&items))
                .expect("commit shard batch");
            store.settlement_root().expect("settlement root")
        })
        .collect()
}

fn commit_shard_batches_timed(
    batches: Vec<Vec<z00z_storage::settlement::StoreItem>>,
) -> (Vec<SettlementStateRoot>, Vec<u128>, u128) {
    let started = Instant::now();
    let results = batches
        .into_par_iter()
        .map(|items| {
            let worker_started = Instant::now();
            let mut store = SettlementStore::new();
            store
                .apply_settlement_ops(put_ops(&items))
                .expect("commit shard batch");
            (
                store.settlement_root().expect("settlement root"),
                worker_started.elapsed().as_micros(),
            )
        })
        .collect::<Vec<_>>();
    let elapsed_us = started.elapsed().as_micros();
    let (roots, worker_times) = results.into_iter().unzip();
    (roots, worker_times, elapsed_us)
}

fn scaling_lane_name(shard_count: usize) -> String {
    format!("shards_{shard_count}")
}

fn shard_root_leaf(
    shard_id: u32,
    shard_root: SettlementStateRoot,
    route_table_digest: [u8; 32],
    policy_set_digest: [u8; 32],
) -> ShardRootLeafV1 {
    ShardRootLeafV1::new(
        shard_id,
        *shard_root.as_bytes(),
        20 + u64::from(shard_id),
        7,
        route_table_digest,
        policy_set_digest,
        100 + u64::from(shard_id),
        200 + u64::from(shard_id),
        0,
    )
}

fn publication_from_roots(roots: &[SettlementStateRoot]) -> CheckpointPublicationV1 {
    let route_table_digest = [0x42; 32];
    let policy_set_digest = [0x51; 32];
    let shard_leaves = roots
        .iter()
        .enumerate()
        .map(|(idx, root)| {
            shard_root_leaf(
                u32::try_from(idx + 1).expect("u32"),
                *root,
                route_table_digest,
                policy_set_digest,
            )
        })
        .collect();
    CheckpointPublicationV1::new(
        RootGenerationTagV1::RootGeneration1,
        PublicationModeTagV1::Synchronous,
        100 + u64::try_from(roots.len()).expect("u64"),
        route_table_digest,
        SettlementStateRoot::settlement_v1([0x11; 32]),
        shard_leaves,
    )
}

fn ops_per_sec(ops: usize, elapsed_us: u128) -> f64 {
    if ops == 0 {
        return 0.0;
    }
    let elapsed = elapsed_us.max(1) as f64;
    ops as f64 / (elapsed / 1_000_000.0)
}

fn measure_scaling_row(lane: String, shard_count: usize, ops_per_shard: usize) -> ShardScalingRow {
    let batches = shard_batches(shard_count, ops_per_shard);
    let total_ops = batches.iter().map(Vec::len).sum::<usize>();
    let hot_shard_ops = batches.iter().map(Vec::len).max().unwrap_or(0);
    let hot_shard_ratio = if total_ops == 0 {
        0.0
    } else {
        hot_shard_ops as f64 / total_ops as f64
    };
    let (roots, worker_times, worker_local_elapsed_us) = commit_shard_batches_timed(batches);
    let publish_started = Instant::now();
    let publication = publication_from_roots(&roots);
    let _ = publication.public_root_v1().expect("public root");
    let publish_elapsed_us = publish_started.elapsed().as_micros();
    let durable_elapsed_us = worker_local_elapsed_us + publish_elapsed_us;
    let hottest_worker_us = worker_times.iter().copied().max().unwrap_or(0);
    let blocked_time_us = worker_local_elapsed_us
        .saturating_sub(hottest_worker_us)
        .saturating_add(publish_elapsed_us);

    ShardScalingRow {
        lane,
        shard_count,
        hot_shard_ratio,
        global_cadence_us: durable_elapsed_us,
        publication_latency_us: publish_elapsed_us,
        shard_tps: ops_per_sec(hot_shard_ops, hottest_worker_us),
        global_tps: ops_per_sec(total_ops, durable_elapsed_us),
        worker_local_tps: ops_per_sec(total_ops, worker_local_elapsed_us),
        durable_root_published_tps: ops_per_sec(total_ops, durable_elapsed_us),
        blocked_time_us,
    }
}

fn bench_shard_parallel_commit(c: &mut Criterion) {
    let mut group = c.benchmark_group("shard_parallel_commit");
    group.bench_function("sim_5a7s", |b| {
        b.iter_batched(
            || shard_batches(7, 24),
            |batches| {
                black_box(commit_shard_batches(batches));
            },
            BatchSize::SmallInput,
        )
    });
    group.finish();
}

fn bench_initial_shard_scaling(c: &mut Criterion) {
    let mut group = c.benchmark_group("initial_shard_scaling");
    for &shard_count in &INITIAL_SHARD_SCALING_COUNTS {
        let lane = scaling_lane_name(shard_count);
        group.bench_function(&lane, |b| {
            b.iter_batched(
                || shard_batches(shard_count, INITIAL_SHARD_LOAD),
                |batches| {
                    black_box(commit_shard_batches(batches));
                },
                BatchSize::SmallInput,
            )
        });
    }
    group.finish();
}

fn write_recovery_note() {
    let items = hot_assets(0x81, 17, 24);
    let (_guard, temp, store) = seed_redb("2", &items);
    let root_before = store.settlement_root().expect("root before reload");
    drop(store);
    let start = Instant::now();
    let reloaded = SettlementStore::load(temp.path()).expect("reload");
    let reload_us = start.elapsed().as_micros();
    let root_after = reloaded.settlement_root().expect("root after reload");

    let mut note = String::new();
    note.push_str("# Shard Recovery Note\n\n");
    note.push_str(
        "- command: `cargo bench -p z00z_storage --bench settlement_shard -- --sample-size 10`\n",
    );
    note.push_str(&format!("- reload_time_us: `{reload_us}`\n"));
    note.push_str(&format!("- root_before: `{:?}`\n", root_before));
    note.push_str(&format!("- root_after: `{:?}`\n", root_after));
    note.push_str(&format!("- root_equal: `{}`\n", root_before == root_after));
    note.push_str("- shard_parallel_commit_lane: `shard_parallel_commit/sim_5a7s`\n");
    note.push_str(
        "- initial_shard_scaling_lanes: `initial_shard_scaling/shards_1`, `initial_shard_scaling/shards_2`, `initial_shard_scaling/shards_4`, `initial_shard_scaling/shards_8`, `initial_shard_scaling/shards_16`\n",
    );
    note.push_str(&format!(
        "- runtime_fixture_manifest: `{RUNTIME_FIXTURE_MANIFEST}`\n"
    ));
    note.push_str(&format!(
        "- route_fixture_manifest: `{ROUTE_FIXTURE_MANIFEST}`\n"
    ));
    note.push_str(
        "- shard_scaling_columns: `hot_shard_ratio`, `global_cadence_us`, `publication_latency_us`, `shard_tps`, `global_tps`, `worker_local_tps`, `durable_root_published_tps`, `blocked_time_us`\n",
    );
    note.push_str(
        "- shard_tps_semantics: `hot-shard throughput from the largest shard batch divided by the slowest worker-local batch time.`\n",
    );
    note.push_str(
        "- global_tps_semantics: `durable-root-published global throughput from total ops divided by the full commit-plus-publication cadence.`\n",
    );
    note.push_str(
        "- publication_latency_semantics: `public-root publication wall-clock time after shard-local roots are already available.`\n",
    );
    note.push_str(
        "- blocked_time_semantics: `wall-clock coordination overhead above the slowest shard-local worker plus public-root publication time.`\n",
    );
    note.push_str("\n## Shard Scaling Matrix\n\n");
    note.push_str("| lane | shard_count | hot_shard_ratio | global_cadence_us | publication_latency_us | shard_tps | global_tps | worker_local_tps | durable_root_published_tps | blocked_time_us |\n");
    note.push_str("| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |\n");
    for &shard_count in &INITIAL_SHARD_SCALING_COUNTS {
        let row = measure_scaling_row(
            scaling_lane_name(shard_count),
            shard_count,
            INITIAL_SHARD_LOAD,
        );
        note.push_str(&format!(
            "| {} | {} | {:.3} | {} | {} | {:.2} | {:.2} | {:.2} | {:.2} | {} |\n",
            row.lane,
            row.shard_count,
            row.hot_shard_ratio,
            row.global_cadence_us,
            row.publication_latency_us,
            row.shard_tps,
            row.global_tps,
            row.worker_local_tps,
            row.durable_root_published_tps,
            row.blocked_time_us,
        ));
    }
    let sim_row = measure_scaling_row("sim_5a7s".to_string(), GRID57_SHARDS, GRID57_SHARD_LOAD);
    note.push_str(&format!(
        "| {} | {} | {:.3} | {} | {} | {:.2} | {:.2} | {:.2} | {:.2} | {} |\n",
        sim_row.lane,
        sim_row.shard_count,
        sim_row.hot_shard_ratio,
        sim_row.global_cadence_us,
        sim_row.publication_latency_us,
        sim_row.shard_tps,
        sim_row.global_tps,
        sim_row.worker_local_tps,
        sim_row.durable_root_published_tps,
        sim_row.blocked_time_us,
    ));
    write_note("settlement_shard_recovery.md", &note);
}

fn main() {
    let _sched = SchedEnv::new(4, 4096);
    if should_emit_side_outputs() {
        write_meta(BenchMeta::new(
            "settlement_shard",
            "cargo bench -p z00z_storage --bench settlement_shard",
        ));
        write_recovery_note();
    }
    let mut crit = Criterion::default().configure_from_args();
    bench_insert_many_defs(&mut crit);
    bench_insert_many_sers(&mut crit);
    bench_hot_definition(&mut crit);
    bench_hot_assets(&mut crit);
    bench_hot_bucket(&mut crit);
    bench_delete_defs(&mut crit);
    bench_delete_hot(&mut crit);
    bench_prove_many(&mut crit);
    bench_shard_parallel_commit(&mut crit);
    bench_initial_shard_scaling(&mut crit);
    crit.final_summary();
}
