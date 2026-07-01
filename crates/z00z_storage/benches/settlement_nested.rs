use std::time::Instant;

use criterion::{black_box, BatchSize, Criterion};
use z00z_storage::{
    fixture_support::{
        settlement_bench_output::{should_emit_side_outputs, write_meta, write_note, BenchMeta},
        settlement_corpus::{
            asset_item, asset_seed, del_ops, hot_assets, many_defs, many_sers, mixed_items,
            put_ops, seed_mem, seed_redb, SchedEnv,
        },
    },
    settlement::SettlementStore,
};

fn bench_hist_puts(c: &mut Criterion) {
    let mut items = many_defs(32);
    items.extend(many_sers(0x21, 32));
    let mut group = c.benchmark_group("nested_hist_puts");
    group.bench_function("generalized_mix", |b| {
        b.iter_batched(
            SettlementStore::new,
            |mut store| {
                black_box(
                    store
                        .apply_settlement_ops(put_ops(&items))
                        .expect("hist puts"),
                );
            },
            BatchSize::SmallInput,
        )
    });
    group.finish();
}

fn bench_root_mix(c: &mut Criterion) {
    let items = mixed_items();
    let next = asset_item(&asset_seed(0x44, 7, 0x88, 77_000));
    let mut group = c.benchmark_group("nested_root_mix");
    group.bench_function("mixed_asset_right", |b| {
        b.iter_batched(
            || seed_mem(&items),
            |mut store| {
                black_box(
                    store
                        .apply_settlement_ops(vec![z00z_storage::settlement::StoreOp::Put(
                            Box::new(next.clone()),
                        )])
                        .expect("root mix"),
                );
            },
            BatchSize::SmallInput,
        )
    });
    group.finish();
}

fn bench_hot_update(c: &mut Criterion) {
    let items = hot_assets(0x52, 9, 64);
    let updates = (0..8)
        .map(|idx| {
            asset_item(&asset_seed(
                0x52,
                9,
                u8::try_from(idx + 1).expect("u8"),
                88_000 + idx as u64,
            ))
        })
        .collect::<Vec<_>>();
    let mut group = c.benchmark_group("nested_hot_update");
    group.bench_function("asset_rewrites", |b| {
        b.iter_batched(
            || seed_mem(&items),
            |mut store| {
                black_box(
                    store
                        .apply_settlement_ops(put_ops(&updates))
                        .expect("hot update"),
                );
            },
            BatchSize::SmallInput,
        )
    });
    group.finish();
}

fn bench_delete_defs(c: &mut Criterion) {
    let items = many_defs(64);
    let mut group = c.benchmark_group("nested_delete_defs");
    group.bench_function("generalized_defs", |b| {
        b.iter_batched(
            || seed_mem(&items),
            |mut store| {
                black_box(
                    store
                        .apply_settlement_ops(del_ops(&items))
                        .expect("delete defs"),
                );
            },
            BatchSize::SmallInput,
        )
    });
    group.finish();
}

fn write_reload_note() {
    let items = mixed_items();
    let (_guard, temp, store) = seed_redb("2", &items);
    let root_before = store.settlement_root().expect("root before");
    drop(store);
    let start = Instant::now();
    let reloaded = SettlementStore::load(temp.path()).expect("reload");
    let reload_us = start.elapsed().as_micros();
    let root_after = reloaded.settlement_root().expect("root after");

    let mut note = String::new();
    note.push_str("# Nested Reload Note\n\n");
    note.push_str(
        "- command: `cargo bench -p z00z_storage --bench settlement_nested -- --sample-size 10`\n",
    );
    note.push_str(&format!("- reload_time_us: `{reload_us}`\n"));
    note.push_str(&format!("- root_before: `{:?}`\n", root_before));
    note.push_str(&format!("- root_after: `{:?}`\n", root_after));
    note.push_str(&format!("- root_equal: `{}`\n", root_before == root_after));
    write_note("settlement_nested_reload.md", &note);
}

fn main() {
    let _sched = SchedEnv::new(4, 4096);
    if should_emit_side_outputs() {
        write_meta(BenchMeta::new(
            "settlement_nested",
            "cargo bench -p z00z_storage --bench settlement_nested",
        ));
        write_reload_note();
    }
    let mut crit = Criterion::default().configure_from_args();
    bench_hist_puts(&mut crit);
    bench_root_mix(&mut crit);
    bench_hot_update(&mut crit);
    bench_delete_defs(&mut crit);
    crit.final_summary();
}
