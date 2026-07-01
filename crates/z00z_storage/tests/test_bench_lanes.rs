use z00z_core::assets::AssetLeaf;
use z00z_storage::settlement::{
    chk_blob_settlement_inclusion, DefinitionId, OccupancyClass, PolicyTransitionProof,
    ProofChkErr, RightClass, SerialId, SettlementLeaf, SettlementLeafFamily, SettlementListReq,
    SettlementLookup, SettlementPath, SettlementStore, StoreItem, StoreOp, TerminalId,
    TerminalLeaf,
};

const HJMT_BENCH: &str = include_str!("../benches/settlement_hjmt.rs");
const PROOFS_BENCH: &str = include_str!("../benches/settlement_proofs.rs");
const SHARD_BENCH: &str = include_str!("../benches/settlement_shard.rs");
const BENCH_DOC: &str = include_str!("../benches/settlement_benches.md");
const BENCH_HELPER: &str = include_str!("../scripts/run_storage_settlement_bench.py");
const BENCH_OUTPUT_POLICY: &str = include_str!("../src/fixture_support/settlement_bench_output.rs");
const STORAGE_CARGO_TOML: &str = include_str!("../Cargo.toml");
const FIXTURE_SUPPORT_MOD: &str = include_str!("../src/fixture_support/mod.rs");
const NESTED_WRAPPER: &str = include_str!("../scripts/run_storage_settlement_nested_bench.sh");
const SHARD_WRAPPER: &str = include_str!("../scripts/run_storage_settlement_shard_bench.sh");
const PHASE_SOURCE_DOC: &str =
    include_str!("../../../.planning/phases/000/062-Gaps-Closing-2/062-04-SUMMARY.md");

use std::{collections::BTreeSet, path::PathBuf, process::Command};

use z00z_storage::fixture_support::settlement_corpus::{
    asset_item, asset_path, load_fixture, load_fixture_items, next_policy, redb_store_with_bits,
    right_item, split_ready_paths, AssetSeed, FixtureRightClass, HjmtEnvGuard, RightSeed,
};

fn asset_seed(definition: u8, serial: u32, terminal: u8, value: u64) -> AssetSeed {
    AssetSeed {
        label: format!("asset_{definition}_{serial}_{terminal}"),
        definition_mark: definition,
        serial_id: serial,
        terminal_mark: terminal,
        value,
    }
}

fn right_seed(definition: u8, serial: u32, terminal: u8, class: FixtureRightClass) -> RightSeed {
    RightSeed {
        label: format!("right_{definition}_{serial}_{terminal}"),
        definition_mark: definition,
        serial_id: serial,
        terminal_mark: terminal,
        right_class: class,
    }
}

fn seed_store(bits: &str, items: Vec<StoreItem>) -> SettlementStore {
    let _guard = HjmtEnvGuard::with_bits(bits);
    let mut store = SettlementStore::new();
    if !items.is_empty() {
        store
            .apply_settlement_ops(
                items
                    .into_iter()
                    .map(|item| StoreOp::Put(Box::new(item)))
                    .collect(),
            )
            .expect("seed store");
    }
    store
}

fn bench_compare_path_seed(
    definition_id: DefinitionId,
    serial_id: SerialId,
    seed: u32,
) -> SettlementPath {
    let mut terminal = [0u8; 32];
    terminal[0] = (seed >> 8) as u8;
    terminal[1] = seed as u8;
    terminal[2] = definition_id.into_bytes()[0];
    terminal[3] = serial_id.get() as u8;
    terminal[4] = (seed >> 24) as u8;
    terminal[5] = (seed >> 16) as u8;
    SettlementPath::new(definition_id, serial_id, TerminalId::new(terminal))
}

fn bench_clustered_paths(
    store: &SettlementStore,
    definition_mark: u8,
    serial_id: u32,
    needed: usize,
    start_seed: u32,
) -> Vec<SettlementPath> {
    let definition_id = DefinitionId::new([definition_mark; 32]);
    let serial_id = SerialId::new(serial_id);
    let target_bucket = bench_compare_path_seed(definition_id, serial_id, start_seed)
        .bucket_id(store.bucket_policy());
    let mut paths = Vec::with_capacity(needed);
    let mut seen = BTreeSet::new();
    let mut seed = start_seed;
    loop {
        let path = bench_compare_path_seed(definition_id, serial_id, seed);
        if path.bucket_id(store.bucket_policy()) == target_bucket && seen.insert(path) {
            paths.push(path);
            if paths.len() == needed {
                break;
            }
        }
        assert!(
            seed < u32::MAX,
            "clustered bench fixture exhausted seed space"
        );
        seed += 1;
    }
    assert_eq!(
        paths.len(),
        needed,
        "clustered bench fixture exhausted seed space"
    );
    paths
}

fn bench_clustered_missing_paths(
    store: &SettlementStore,
    present_paths: &[SettlementPath],
    start_seed: u32,
) -> Vec<SettlementPath> {
    let mut seen = present_paths.iter().copied().collect::<BTreeSet<_>>();
    let mut missing = Vec::with_capacity(present_paths.len());
    let mut seed = start_seed;
    for (idx, base) in present_paths.iter().copied().enumerate() {
        let target_bucket = base.bucket_id(store.bucket_policy());
        let found = loop {
            let path = bench_compare_path_seed(
                base.definition_id,
                base.serial_id,
                seed.saturating_add(u32::try_from(idx).expect("u32")),
            );
            assert!(
                seed < u32::MAX,
                "clustered missing bench fixture exhausted seed space"
            );
            seed += 1;
            if path == base || seen.contains(&path) {
                continue;
            }
            if path.bucket_id(store.bucket_policy()) != target_bucket {
                continue;
            }
            break path;
        };
        seen.insert(found);
        missing.push(found);
    }
    missing
}

fn bench_same_bucket_companions(
    store: &SettlementStore,
    present_paths: &[SettlementPath],
    extra_per_path: usize,
    start_seed: u32,
) -> Vec<SettlementPath> {
    let mut seen = present_paths.iter().copied().collect::<BTreeSet<_>>();
    let mut siblings = Vec::with_capacity(present_paths.len() * extra_per_path);
    let mut seed = start_seed;
    for base in present_paths.iter().copied() {
        let target_bucket = base.bucket_id(store.bucket_policy());
        for _ in 0..extra_per_path {
            let found = loop {
                let path = bench_compare_path_seed(base.definition_id, base.serial_id, seed);
                assert!(
                    seed < u32::MAX,
                    "same-bucket sibling bench fixture exhausted seed space"
                );
                seed += 1;
                if path == base || seen.contains(&path) {
                    continue;
                }
                if path.bucket_id(store.bucket_policy()) != target_bucket {
                    continue;
                }
                break path;
            };
            seen.insert(found);
            siblings.push(found);
        }
    }
    siblings
}

fn bench_scattered_paths(
    definition_mark: u8,
    serial_base: u32,
    needed: usize,
    start_seed: u32,
) -> Vec<SettlementPath> {
    let mut paths = (0..needed)
        .map(|idx| {
            let mark = definition_mark.wrapping_add(u8::try_from(idx % 29).expect("u8"));
            bench_compare_path_seed(
                DefinitionId::new([mark; 32]),
                SerialId::new(serial_base + u32::try_from(idx).expect("u32")),
                start_seed + u32::try_from(idx).expect("u32"),
            )
        })
        .collect::<Vec<_>>();
    paths.sort_unstable();
    paths
}

fn bench_compare_asset_item(path: SettlementPath) -> StoreItem {
    let mut core = AssetLeaf::dummy_for_scan(path.serial_id.get());
    core.asset_id = path.terminal_id().into_bytes();
    let leaf = SettlementLeaf::Terminal(TerminalLeaf::from(core));
    StoreItem::new(path, leaf).expect("comparison item")
}

fn seed_bench_compare_paths(store: &mut SettlementStore, paths: &[SettlementPath]) {
    for path in paths {
        store
            .put_settlement_item(bench_compare_asset_item(*path))
            .expect("seed comparison path");
    }
}

fn bench_deletion_fixture(
    clustered: bool,
    path_count: usize,
) -> (SettlementStore, Vec<SettlementPath>) {
    let mut store = SettlementStore::new();
    let paths = if clustered {
        bench_clustered_paths(&store, 0xA1, 41, path_count, 1)
    } else {
        bench_scattered_paths(0xB1, 141, path_count, 10_000)
    };
    seed_bench_compare_paths(&mut store, &paths);
    let survivors =
        bench_same_bucket_companions(&store, &paths, if clustered { 1 } else { 2 }, 20_000);
    seed_bench_compare_paths(&mut store, &survivors);
    store
        .apply_settlement_ops(paths.iter().copied().map(StoreOp::Delete).collect())
        .expect("delete comparison paths");
    (store, paths)
}

fn bench_nonexistence_fixture(
    clustered: bool,
    path_count: usize,
) -> (SettlementStore, Vec<SettlementPath>) {
    let mut store = SettlementStore::new();
    let paths = if clustered {
        bench_clustered_paths(&store, 0xA1, 41, path_count, 1)
    } else {
        bench_scattered_paths(0xB1, 141, path_count, 10_000)
    };
    seed_bench_compare_paths(&mut store, &paths);
    if !clustered {
        let companions = bench_same_bucket_companions(&store, &paths, 2, 20_000);
        seed_bench_compare_paths(&mut store, &companions);
    }
    let missing = if clustered {
        bench_clustered_missing_paths(&store, &paths, 30_000)
    } else {
        bench_batchable_nonexistence_paths(&store, &paths, 30_000)
    };
    (store, missing)
}

fn bench_batchable_nonexistence_paths(
    store: &SettlementStore,
    present_paths: &[SettlementPath],
    start_seed: u32,
) -> Vec<SettlementPath> {
    // Keep the bench-fixture contract aligned with the live v1 batch surface.
    let policy = store.bucket_policy();
    let mut seen = present_paths.iter().copied().collect::<BTreeSet<_>>();
    let mut missing = Vec::with_capacity(present_paths.len());
    let mut seed = start_seed;
    for (idx, base) in present_paths.iter().copied().enumerate() {
        let target_bucket = base.bucket_id(policy);
        let found = loop {
            let path = bench_compare_path_seed(
                base.definition_id,
                base.serial_id,
                seed.saturating_add(u32::try_from(idx).expect("u32")),
            );
            assert!(
                seed < u32::MAX,
                "batchable missing bench fixture exhausted seed space"
            );
            seed += 1;
            if path == base || seen.contains(&path) {
                continue;
            }
            if path.bucket_id(policy) != target_bucket {
                continue;
            }
            if store
                .settlement_nonexistence_batch_v1(&[path], SettlementLeafFamily::Terminal)
                .is_err()
            {
                continue;
            }
            break path;
        };
        seen.insert(found);
        missing.push(found);
    }
    missing
}

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("workspace crates dir")
        .parent()
        .expect("workspace root")
        .to_path_buf()
}

fn run_bench_helper_dry_run(args: &[&str]) -> std::process::Output {
    Command::new("python3")
        .arg("crates/z00z_storage/scripts/run_storage_settlement_bench.py")
        .arg("--dry-run")
        .args(args)
        .current_dir(repo_root())
        .output()
        .expect("run bench helper")
}

fn assert_forbidden_tokens_absent(label: &str, surface: &str, forbidden: &[&str]) {
    for token in forbidden {
        assert!(
            !surface.contains(token),
            "{} still contains legacy family-level token {}",
            label,
            token
        );
    }
}

#[test]
fn test_helper_uses_output_root() {
    assert!(BENCH_HELPER.contains(r#""outputs" / "settlement""#));
    assert!(!BENCH_HELPER.contains(r#""outputs" / "assets""#));
    assert!(BENCH_HELPER.contains("Z00Z_SETTLEMENT_TIME_OUT"));
    assert!(BENCH_HELPER.contains("Z00Z_SETTLEMENT_TIME_RUN"));
}

#[test]
fn test_harness_tokens_stay_canonical() {
    let forbidden_family_tokens = [
        "assets_hjmt",
        "assets_proofs",
        "assets_nested",
        "assets_shard",
        "assets_benches",
        "run_storage_assets_bench.py",
        "run_storage_assets_nested_bench.sh",
        "run_storage_assets_shard_bench.sh",
        "outputs/assets",
        "Z00Z_STORAGE_ASSET_BENCH_KEEP",
        "Z00Z_ASSET_PROOF_NOTE_SCOPE",
        "Z00Z_ASSET_PROOF_NOTE_COMMAND",
        "Z00Z_ASSET_PROOF_NOTE_FILTER",
        "Z00Z_ASSET_BENCH_MODE",
        "Z00Z_ASSET_ROOT_MODE",
        "Z00Z_ASSET_BASELINE",
        "storage-assets-bench-output-v1",
        "assets_proofs_batch",
        "assets_proof_sizes",
        "assets_nested_reload",
        "assets_shard_recovery",
    ];
    assert_forbidden_tokens_absent(
        "storage Cargo.toml",
        STORAGE_CARGO_TOML,
        &forbidden_family_tokens[..4],
    );
    assert_forbidden_tokens_absent(
        "fixture support mod",
        FIXTURE_SUPPORT_MOD,
        &["assets_bench_output", "assets_bench_support"],
    );
    assert_forbidden_tokens_absent("settlement helper", BENCH_HELPER, &forbidden_family_tokens);
    assert_forbidden_tokens_absent(
        "settlement output policy",
        BENCH_OUTPUT_POLICY,
        &forbidden_family_tokens,
    );
    assert_forbidden_tokens_absent(
        "settlement bench doc",
        BENCH_DOC,
        &[
            "assets_hjmt",
            "assets_proofs",
            "assets_nested",
            "assets_shard",
            "assets_benches",
            "run_storage_assets_bench.py",
            "outputs/assets",
            "assets_proofs_batch",
            "assets_proof_sizes",
            "assets_nested_reload",
            "assets_shard_recovery",
        ],
    );
    assert_forbidden_tokens_absent(
        "nested wrapper",
        NESTED_WRAPPER,
        &[
            "run_storage_assets_bench.py",
            "run_storage_assets_nested_bench.sh",
            "assets_nested",
        ],
    );
    assert_forbidden_tokens_absent(
        "shard wrapper",
        SHARD_WRAPPER,
        &[
            "run_storage_assets_bench.py",
            "run_storage_assets_shard_bench.sh",
            "assets_shard",
        ],
    );

    assert!(STORAGE_CARGO_TOML.contains("name = \"settlement_hjmt\""));
    assert!(STORAGE_CARGO_TOML.contains("name = \"settlement_proofs\""));
    assert!(STORAGE_CARGO_TOML.contains("name = \"settlement_nested\""));
    assert!(STORAGE_CARGO_TOML.contains("name = \"settlement_shard\""));
    assert!(FIXTURE_SUPPORT_MOD.contains("pub mod settlement_bench_output;"));
    assert!(FIXTURE_SUPPORT_MOD.contains("pub mod settlement_corpus;"));
    assert!(!FIXTURE_SUPPORT_MOD.contains("pub mod settlement_bench_support;"));
    assert!(BENCH_HELPER.contains("Z00Z_STORAGE_SETTLEMENT_BENCH_KEEP"));
    assert!(BENCH_HELPER.contains("Z00Z_SETTLEMENT_PROOF_NOTE_SCOPE"));
    for needle in [
        "const BENCH_OUTPUT_HASH_SCHEMA: &str = \"storage-settlement-bench-output-v2\";",
        "hash_root_inputs(",
        "crates/z00z_storage/benches",
        "crates/z00z_storage/scripts",
        "crates/z00z_storage/src",
        "crates/z00z_simulator/src",
        "crates/z00z_wallets/src",
    ] {
        assert!(
            BENCH_OUTPUT_POLICY.contains(needle),
            "bench output policy contract must include {needle}"
        );
    }
    assert!(BENCH_DOC.contains("run_storage_settlement_bench.py"));
    assert!(NESTED_WRAPPER.contains("--bench settlement_nested"));
    assert!(SHARD_WRAPPER.contains("--bench settlement_shard"));
}

fn empty_bucket_fixture() -> (SettlementStore, z00z_storage::settlement::SettlementPath) {
    let _guard = HjmtEnvGuard::with_bits("2");
    let target = asset_seed(0xA2, 6, 1, 62_000);
    let target_item = asset_item(&target);
    let target_path = asset_path(&target);
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
        let candidate_path = asset_path(&candidate);
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
            return (store, target_path);
        }
    }

    panic!("missing empty bucket fixture")
}

#[test]
fn test_right_class_lane_terminals() {
    let machine_items = (1u8..=8u8).map(|terminal_mark| {
        right_item(&right_seed(
            0x71,
            7,
            terminal_mark,
            FixtureRightClass::MachineCapability,
        ))
    });
    let data_items = (101u8..=108u8).map(|terminal_mark| {
        right_item(&right_seed(
            0x71,
            8,
            terminal_mark,
            FixtureRightClass::DataAccess,
        ))
    });
    let store = seed_store("2", machine_items.chain(data_items).collect());

    let page = store
        .list_settlement(SettlementListReq::for_right_class(
            RightClass::DataAccess,
            16,
        ))
        .expect("right class list");
    assert_eq!(page.items().len(), 8);
}

#[test]
fn test_duplicate_batch_rejects() {
    let asset = asset_seed(0x91, 3, 8, 51_000);
    let item = asset_item(&asset);
    let mut store = SettlementStore::new();
    let err = store
        .apply_settlement_ops(vec![
            StoreOp::Put(Box::new(item.clone())),
            StoreOp::Put(Box::new(item)),
        ])
        .expect_err("duplicate path must reject");
    assert!(matches!(
        err,
        z00z_storage::settlement::SettlementStoreError::OpPathDup
    ));
}

#[test]
fn test_delete_prune_fixtures() {
    let (mut bucket_store, bucket_path) = empty_bucket_fixture();
    bucket_store
        .del_settlement_item(&bucket_path)
        .expect("empty bucket prune");
    assert!(bucket_store
        .lookup_settlement(SettlementLookup::Path(bucket_path))
        .expect("bucket lookup")
        .is_none());

    let serial_target = asset_seed(0xA3, 7, 1, 63_000);
    let serial_keep = asset_seed(0xA3, 8, 2, 63_100);
    let mut serial_store = seed_store(
        "2",
        vec![asset_item(&serial_target), asset_item(&serial_keep)],
    );
    serial_store
        .del_settlement_item(&asset_path(&serial_target))
        .expect("empty serial prune");
    assert!(serial_store
        .lookup_settlement(SettlementLookup::Path(asset_path(&serial_target)))
        .expect("serial lookup")
        .is_none());
    assert!(serial_store
        .lookup_settlement(SettlementLookup::Path(asset_path(&serial_keep)))
        .expect("serial keep lookup")
        .is_some());

    let definition_target = asset_seed(0xA4, 7, 1, 64_000);
    let definition_keep = asset_seed(0xA5, 7, 2, 64_100);
    let mut definition_store = seed_store(
        "2",
        vec![asset_item(&definition_target), asset_item(&definition_keep)],
    );
    definition_store
        .del_settlement_item(&asset_path(&definition_target))
        .expect("empty definition prune");
    assert!(definition_store
        .lookup_settlement(SettlementLookup::Path(asset_path(&definition_target)))
        .expect("definition lookup")
        .is_none());
    assert!(definition_store
        .lookup_settlement(SettlementLookup::Path(asset_path(&definition_keep)))
        .expect("definition keep lookup")
        .is_some());
}

#[test]
fn test_historical_proof_reject_lanes() {
    let fixture = load_fixture();
    let store = seed_store("2", load_fixture_items(&fixture));
    let path = asset_path(&fixture.assets[0]);
    let blob = store.settlement_proof_blob(&path).expect("inclusion proof");
    let item = blob.item();
    let mut malformed = blob.encode().expect("encode proof");
    malformed.truncate(malformed.len().saturating_sub(1));
    let err = chk_blob_settlement_inclusion(
        &malformed,
        item.settlement_root(),
        &item.path(),
        item.def_leaf(),
        item.ser_leaf(),
        item.terminal_leaf().expect("asset leaf").clone(),
    )
    .expect_err("malformed bytes reject");
    assert!(matches!(err, ProofChkErr::Codec(_)));

    let tampered = store
        .settlement_proof_blob(&path)
        .expect("inclusion proof")
        .with_root_bind(2, blob.root_bind());
    assert!(store.validate_settlement_proof_blob(&tampered).is_err());

    let (_guard, _temp, mut adaptive_store) =
        redb_store_with_bits(Some("1")).expect("adaptive test store");
    let split_path = split_ready_paths(&mut adaptive_store, 41, 9)[0];
    let next = next_policy(&adaptive_store);
    let proof = adaptive_store
        .policy_transition_proof(next)
        .expect("policy transition proof");
    adaptive_store
        .put_settlement_item(asset_item(&asset_seed(0x95, 1, 0x61, 105_000)))
        .expect("advance historical store");
    adaptive_store
        .validate_policy_transition_proof(&proof, next)
        .expect("historical transition verify");

    let stale_proof = PolicyTransitionProof {
        prior_policy_id: [0x11; 32],
        ..adaptive_store
            .policy_transition_proof(next)
            .expect("fresh policy transition proof")
    };
    assert!(adaptive_store
        .validate_policy_transition_proof(&stale_proof, next)
        .is_err());

    let occupancy = adaptive_store
        .bucket_occupancy_metric(&split_path)
        .expect("occupancy metric");
    assert_eq!(occupancy.class, OccupancyClass::SplitReady);
}

#[test]
fn test_mixed_batch_lane_shape() {
    let items = (0..16usize)
        .map(|index| {
            asset_item(&asset_seed(
                0x74,
                12,
                u8::try_from(index + 1).expect("u8 terminal"),
                104_000 + index as u64,
            ))
        })
        .collect::<Vec<_>>();
    let store = seed_store("2", items.clone());
    let paths = items
        .iter()
        .take(8)
        .map(StoreItem::path)
        .collect::<Vec<_>>();
    let missing = asset_path(&asset_seed(0x74, 12, 0xEE, 104_999));

    let inclusion = store
        .settlement_proof_blobs(&paths)
        .expect("mixed batch inclusion proofs");
    let absence = store
        .settlement_nonexistence_proof_blob(&missing, SettlementLeafFamily::Terminal)
        .expect("mixed batch absence proof");

    assert_eq!(inclusion.len(), 8);
    assert_eq!(
        absence.hjmt_proof_family(),
        Some(z00z_storage::settlement::HjmtProofFamily::NonExistence)
    );
}

#[test]
fn test_heavy_workload_lanes() {
    assert!(HJMT_BENCH.contains("bench_function(\"proof_heavy\""));
    assert!(HJMT_BENCH.contains("bench_function(\"policy_transition_heavy\""));
    assert!(HJMT_BENCH.contains("benchmark_group(\"cache_edge_support\")"));
    for lane in ["cap_minus_1", "cap", "cap_plus_1", "cap_times_2"] {
        assert!(
            HJMT_BENCH.contains(&format!("bench_function(\"{lane}\"")),
            "missing cache-edge lane {lane}"
        );
    }
    assert!(BENCH_DOC.contains("cache/proof_heavy"));
    assert!(BENCH_DOC.contains("cache/policy_transition_heavy"));
    assert!(BENCH_DOC.contains("cache_edge_support/cap_times_2"));
}

#[test]
fn test_root_publish_wired() {
    assert!(HJMT_BENCH.contains("benchmark_group(\"root_of_roots_publish\")"));
    for lane in ["shards_1", "shards_2", "shards_4", "shards_8", "shards_16"] {
        assert!(
            HJMT_BENCH.contains(&format!("root_of_roots_publish/{lane}"))
                || HJMT_BENCH.contains(&format!("bench_function(\"{lane}\""))
                || HJMT_BENCH.contains("bench_function(&lane"),
            "missing root-of-roots lane {lane}"
        );
    }
    assert!(HJMT_BENCH.contains("public_root_v1().expect(\"public root\")"));
    assert!(HJMT_BENCH.contains(
        "crates/z00z_storage/tests/fixtures/hjmt_upgrade/shard_root_leaf_v1/manifest.json"
    ));
    assert!(HJMT_BENCH.contains(
        "crates/z00z_storage/tests/fixtures/hjmt_upgrade/checkpoint_publication_v1/manifest.json"
    ));
}

#[test]
fn test_closeout_bench_wired() {
    assert!(SHARD_BENCH.contains("benchmark_group(\"shard_parallel_commit\")"));
    assert!(SHARD_BENCH.contains("bench_function(\"sim_5a7s\""));
    assert!(SHARD_BENCH.contains("benchmark_group(\"initial_shard_scaling\")"));
    for lane in ["shards_1", "shards_2", "shards_4", "shards_8", "shards_16"] {
        assert!(
            SHARD_BENCH.contains(&format!("bench_function(\"{lane}\""))
                || SHARD_BENCH.contains("bench_function(&lane"),
            "missing shard scaling lane {lane}"
        );
    }
    assert!(SHARD_BENCH.contains("settlement_shard_recovery.md"));
    assert!(SHARD_BENCH.contains("hot_shard_ratio"));
    assert!(SHARD_BENCH.contains("worker_local_tps"));
    assert!(SHARD_BENCH.contains("durable_root_published_tps"));
    assert!(SHARD_BENCH.contains("publication_latency_us"));
    assert!(SHARD_BENCH.contains("blocked_time_us"));
    assert!(SHARD_BENCH.contains("config/hjmt_runtime/sim_5a7s/manifest.json"));
    assert!(SHARD_BENCH.contains(
        "crates/z00z_storage/tests/fixtures/hjmt_upgrade/shard_route_table_v1/manifest.json"
    ));
    assert!(BENCH_DOC.contains("shard_parallel_commit/sim_5a7s"));
    assert!(BENCH_DOC.contains("initial_shard_scaling/shards_16"));
    assert!(BENCH_DOC.contains("runtime_fixture_manifest"));
    assert!(BENCH_DOC.contains("durable_root_published_tps"));
    assert!(BENCH_DOC.contains("publication_latency_us"));
}

#[test]
fn test_proof_bench_wired() {
    assert!(PROOFS_BENCH.contains("benchmark_group(\"hjmt_batch_proof_bytes\")"));
    assert!(PROOFS_BENCH.contains("benchmark_group(\"hjmt_batch_verify\")"));
    assert!(PROOFS_BENCH.contains("ProofNoteScope::BatchOnly"));
    assert!(PROOFS_BENCH.contains("ProofNoteScope::Skip"));
    assert!(PROOFS_BENCH.contains("let note_scope = proof_note_scope();"));
    assert!(PROOFS_BENCH.contains("write_proof_note(note_scope);"));
    assert!(PROOFS_BENCH.contains("crit.final_summary();"));
    assert!(PROOFS_BENCH.contains("note_runs_direct_matrix()"));
    assert!(PROOFS_BENCH.contains("collect_note_rows()"));
    assert!(PROOFS_BENCH.contains("if !direct_batch_note {"));
    assert!(PROOFS_BENCH.contains("record_batch_only_note_row("));
    assert!(PROOFS_BENCH.contains("take_batch_only_note_rows()"));
    assert!(PROOFS_BENCH.contains("compare_surface_raw_encoded_bytes("));
    assert!(PROOFS_BENCH.contains("store.clear_forest_cache();"));
    assert!(PROOFS_BENCH.contains("criterion_batch_proof_bytes_semantics"));
    assert!(PROOFS_BENCH.contains("serialized_bytes_semantics"));
    assert!(
        PROOFS_BENCH.contains("one cold sample captured during hjmt_batch_proof_bytes lane setup")
    );
    assert!(BENCH_OUTPUT_POLICY.contains("pub enum ProofNoteScope"));
    assert!(BENCH_OUTPUT_POLICY.contains("Z00Z_SETTLEMENT_PROOF_NOTE_SCOPE"));
    assert!(BENCH_OUTPUT_POLICY.contains("Z00Z_SETTLEMENT_PROOF_NOTE_COMMAND"));
    assert!(BENCH_OUTPUT_POLICY.contains("Z00Z_SETTLEMENT_PROOF_NOTE_FILTER"));
    assert!(BENCH_HELPER.contains("PROOF_NOTE_SCOPE_ENV = \"Z00Z_SETTLEMENT_PROOF_NOTE_SCOPE\""));
    assert!(
        BENCH_HELPER.contains("PROOF_NOTE_COMMAND_ENV = \"Z00Z_SETTLEMENT_PROOF_NOTE_COMMAND\"")
    );
    assert!(BENCH_HELPER.contains("PROOF_NOTE_FILTER_ENV = \"Z00Z_SETTLEMENT_PROOF_NOTE_FILTER\""));
    assert!(BENCH_HELPER.contains("SETTLEMENT_TIME_OUT_ENV = \"Z00Z_SETTLEMENT_TIME_OUT\""));
    assert!(BENCH_HELPER.contains("SETTLEMENT_TIME_RUN_ENV = \"Z00Z_SETTLEMENT_TIME_RUN\""));
    assert!(BENCH_HELPER.contains("def validate_quick_args(extra: list[str]) -> None:"));
    assert!(BENCH_HELPER.contains("def proof_note_scope(args: argparse.Namespace) -> str | None:"));
    assert!(BENCH_HELPER.contains("def parse_internal_stage_timings(path: Path)"));
    assert!(BENCH_HELPER.contains("\"batch_only\""));
    assert!(BENCH_HELPER.contains("\"skip\""));
    assert!(BENCH_HELPER.contains("## Internal Stage Timing Slices"));
    assert!(BENCH_HELPER.contains("timing_trace_file"));
    assert!(BENCH_HELPER.contains(
        "settlement_proofs_batch requires a filter that includes hjmt_batch_proof_bytes lanes"
    ));
    assert!(BENCH_HELPER.contains("--quick cannot be combined with --sample-size"));
    for token in [
        "BATCH_COMPARE_FULL_COUNTS: [usize; 6] = [2, 8, 32, 128, 1000, 1024]",
        "BATCH_COMPARE_EVIDENCE_COUNTS: [usize; 3] = [2, 8, 32]",
        "BatchCompareFamily::Inclusion",
        "BatchCompareFamily::Deletion",
        "BatchCompareFamily::NonExistence",
        "BatchCompareShape::Clustered",
        "BatchCompareShape::Scattered",
        "BatchCompareSurface::Single",
        "BatchCompareSurface::Vec",
        "BatchCompareSurface::Batch",
    ] {
        assert!(
            PROOFS_BENCH.contains(token),
            "missing logical proof bench token {token}"
        );
    }
    assert!(PROOFS_BENCH.contains("clear_forest_cache()"));
    assert!(PROOFS_BENCH.contains("check_contract_v1()"));
    assert!(PROOFS_BENCH.contains("\"reject/malformed_bytes/2\""));
    assert!(PROOFS_BENCH.contains("\"reject/mixed_family/2\""));
    assert!(BENCH_DOC.contains("hjmt_batch_proof_bytes"));
    assert!(BENCH_DOC.contains("hjmt_batch_verify"));
    assert!(BENCH_DOC.contains("hjmt_batch_verify/reject/malformed_bytes/2"));
    assert!(BENCH_DOC.contains("hjmt_batch_verify/reject/mixed_family/2"));
    assert!(BENCH_DOC.contains("proof_blob_single"));
    assert!(BENCH_DOC.contains("proof_blob_vec"));
    assert!(BENCH_DOC.contains("batch_proof_v1"));
    assert!(BENCH_DOC.contains("batch-only `settlement_proof_sizes.md` scope"));
    assert!(BENCH_DOC.contains("verify timing authority stays in `settlement_proofs_batch.md`"));
    assert!(BENCH_DOC.contains("direct batch-only note scope"));
    assert!(BENCH_DOC.contains("light live prove/bytes matrix for `count in {2,8,32}`"));
    assert!(BENCH_DOC.contains(
        "leaves `128/1000/1024` to the full `settlement_proofs.md` benchmark and stress lanes"
    ));
    assert!(BENCH_DOC.contains("does not rerun the full `hjmt_batch_proof_bytes` Criterion matrix"));
    assert!(BENCH_DOC.contains("does not recalculate unrelated proof-size bytes"));
    assert!(BENCH_DOC.contains("actual filtered bench command and lane selector"));
    assert!(BENCH_DOC.contains("synthetic seeded fixtures on an in-memory store"));
    assert!(BENCH_DOC.contains("raw encoded length with no compression step"));
    assert!(BENCH_DOC.contains("cold prove plus raw encode per iteration"));
    assert!(BENCH_DOC.contains("prove_time_us"));
    assert!(BENCH_DOC.contains("note-level `verify_time_us`"));
    assert!(BENCH_DOC.contains("proof_generate_cache_state"));
    assert!(BENCH_DOC.contains("proof_verify_cache_state"));
    assert!(BENCH_DOC.contains("proof_note_scope=skip"));
}

#[test]
fn test_helper_tracks_measurement_lanes() {
    for needle in [
        "\"hjmt_mapping_ab\"",
        "--shard-mapping",
        "criterion_closure_timing",
        "whole_command_resource",
        "scenario_stage_runtime",
        "user_facing_throughput",
        "def run_hjmt_mapping_ab(args: argparse.Namespace) -> int:",
        "def prepare_scenario_variant(base: str, mapping: str)",
        "def rewrite_shard_process_home(home: Path) -> None:",
        "runtime_config_variant",
        "report_label_only_storage_local_path",
        "failover_recovery_time_us",
    ] {
        assert!(
            BENCH_HELPER.contains(needle),
            "bench helper must retain mapping or lane token {needle}"
        );
    }
    for needle in [
        "## 🧭 Measurement Lane Taxonomy",
        "criterion_closure_timing",
        "whole_command_resource",
        "scenario_stage_runtime",
        "user_facing_throughput",
        "hjmt_mapping_ab.md",
    ] {
        assert!(
            BENCH_DOC.contains(needle),
            "bench doc must retain lane taxonomy token {needle}"
        );
    }
}

#[test]
fn test_phase27_stage13_only() {
    let outcome_block = PHASE_SOURCE_DOC
        .split("## Files Changed")
        .next()
        .expect("phase 062-04 summary must keep outcome block");
    assert!(outcome_block.contains("Phase 27 is `Closed by Stage13 evidence`."));
    assert!(outcome_block.contains("No standalone measurement sidecar file was introduced."));
    assert!(!outcome_block.contains("`Standalone sidecar required`"));
    assert!(BENCH_DOC.contains("Phase 27 selects `Closed by Stage13 evidence`"));
    assert!(BENCH_DOC.contains("no standalone measurement sidecar file"));
}

#[test]
fn test_doc_keeps_profile_contract() {
    for needle in [
        "SIM-SMALL` | fast deterministic correctness in the exact `16-64` operation range | `accepted`",
        "SIM-MEDIUM` | integration correctness in the exact `128-256` operation range | `accepted`",
        "SIM-CACHE-EDGE` | cache-relative validation at `cap - 1`, `cap`, `cap + 1`, and `2 * cap` | `accepted`",
        "SIM-BATCH-1000` | heavy benchmark and readiness evidence only",
        "| `commit_recovery_replay` | exact conformance replacement is `bucket_commit_equivalence/manifest.json` plus `crates/z00z_storage/tests/test_hjmt_batch_commit.rs`; the row is closed as deterministic bucket-commit equivalence and reload continuity, not as a second throughput harness | `accepted` |",
        "| `compat_equivalence_random_ops` | exact conformance replacement is `compat_equivalence_random_ops/manifest.json` plus `crates/z00z_storage/tests/test_hjmt_compat_equivalence.rs`; the row is closed as deterministic fixed-seed oracle parity and reload continuity, not as a duplicate benchmark harness | `accepted` |",
        "| current proof-size rows prove a compression win | `unsupported` |",
        "| cache-only throughput may stand in for durable-root-published TPS | `rejected` |",
        "| synthetic proof-size numbers may stand in for executed proof artifacts | `rejected` |",
        "| the legacy alternate archive-home alias is the live archive home | `rejected` |",
        "`hjmt_plan_ops`, `hjmt_child_commit`, `hjmt_parent_commit`, `hjmt_journal_sync`",
        "archives a sibling `*.timing.tsv` trace",
        "publication_latency_us",
        "Measurement Lane Taxonomy",
        "hjmt_mapping_ab.md",
    ] {
        assert!(
            BENCH_DOC.contains(needle),
            "bench doc must retain score-discipline anchor {needle}"
        );
    }
}

#[test]
fn test_proof_bench_scales() {
    let store = SettlementStore::new();
    let clustered = bench_clustered_paths(&store, 0xA1, 41, 1024, 1);
    let target_bucket = clustered[0].bucket_id(store.bucket_policy());
    assert_eq!(clustered.len(), 1024);
    assert!(clustered
        .iter()
        .all(|path| path.bucket_id(store.bucket_policy()) == target_bucket));

    let missing = bench_clustered_missing_paths(&store, &clustered, 30_000);
    assert_eq!(missing.len(), clustered.len());
    assert!(missing.iter().all(|path| !clustered.contains(path)));
    assert!(missing
        .iter()
        .all(|path| path.bucket_id(store.bucket_policy()) == target_bucket));
}

#[test]
fn test_deletion_matrix_stays_live() {
    let _guard = HjmtEnvGuard::with_bits("2");
    for &clustered in &[true, false] {
        for &count in &[2usize, 8, 32] {
            let (store, paths) = bench_deletion_fixture(clustered, count);
            let baseline = store
                .settlement_proof_blobs(&paths)
                .expect("deletion baseline proofs");
            assert_eq!(baseline.len(), count);
            store
                .settlement_deletion_batch_v1(&paths)
                .unwrap_or_else(|err| {
                    panic!(
                        "deletion batch compare fixture clustered={clustered} count={count}: {err:?}"
                    )
                });
        }
    }
}

#[test]
fn test_nonexistence_matrix_stays_live() {
    let _guard = HjmtEnvGuard::with_bits("2");
    for &clustered in &[true, false] {
        for &count in &[2usize, 8, 32] {
            let (store, paths) = bench_nonexistence_fixture(clustered, count);
            let baseline = paths
                .iter()
                .map(|path| {
                    store
                        .settlement_nonexistence_proof_blob(path, SettlementLeafFamily::Terminal)
                        .expect("nonexistence baseline proofs")
                })
                .collect::<Vec<_>>();
            assert_eq!(baseline.len(), count);
            store
                .settlement_nonexistence_batch_v1(&paths, SettlementLeafFamily::Terminal)
                .unwrap_or_else(|err| {
                    panic!(
                        "nonexistence batch compare fixture clustered={clustered} count={count}: {err:?}"
                    )
                });
        }
    }
}

#[test]
fn test_helper_rejects_sample_conflict() {
    let output = run_bench_helper_dry_run(&[
        "--bench",
        "settlement_proofs",
        "--",
        "hjmt_batch_",
        "--quick",
        "--sample-size",
        "10",
    ]);
    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("--quick cannot be combined with --sample-size"));
}

#[test]
fn test_helper_skips_note_probe() {
    let output = run_bench_helper_dry_run(&[
        "--bench",
        "settlement_proofs",
        "--",
        "hjmt_batch_verify/proof_blob_vec/clustered/inclusion/2",
        "--quick",
        "--noplot",
    ]);
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("proof_note_scope: skip"));
    assert!(stdout
        .contains("proof_note_filter: hjmt_batch_verify/proof_blob_vec/clustered/inclusion/2"));
}

#[test]
fn test_log_requires_note_source() {
    let output = run_bench_helper_dry_run(&[
        "--bench",
        "settlement_proofs",
        "--log-base",
        "settlement_proofs_batch",
        "--",
        "hjmt_batch_verify/proof_blob_vec/clustered/inclusion/2",
        "--quick",
        "--noplot",
    ]);
    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains(
        "settlement_proofs_batch requires a filter that includes hjmt_batch_proof_bytes lanes"
    ));
}

#[test]
fn test_helper_marks_batch_scope() {
    let output = run_bench_helper_dry_run(&[
        "--bench",
        "settlement_proofs",
        "--",
        "hjmt_batch_",
        "--quick",
        "--noplot",
    ]);
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("proof_note_scope: batch_only"));
    assert!(stdout.contains("proof_note_filter: hjmt_batch_"));
}

#[test]
fn test_helper_uses_release_fast() {
    let output = run_bench_helper_dry_run(&["--bench", "scenario_1"]);
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("cargo run --release -p z00z_simulator --bin scenario_1"));
    assert!(stdout.contains("--features test-params-fast"));
    assert!(stdout.contains("--features wallet_debug_tools"));
}
