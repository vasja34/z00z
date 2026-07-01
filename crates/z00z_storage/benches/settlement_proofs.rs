use jmt::ValueHash;
use sha2::Sha256;
use std::{
    collections::{BTreeMap, BTreeSet},
    sync::{Mutex, OnceLock},
    time::Instant,
};

use criterion::{black_box, BatchSize, Criterion};
use z00z_core::assets::AssetLeaf;
use z00z_storage::{
    fixture_support::{
        settlement_bench_output::{
            proof_note_command, proof_note_filter, proof_note_scope, should_emit_side_outputs,
            write_meta, write_note, BenchMeta, ProofNoteScope,
        },
        settlement_corpus,
        settlement_corpus::{
            asset_item, asset_seed, consume_right_fee, create_right_fee, fee_envelope, fee_put_ops,
            hot_assets, load_fixture, next_policy, proof_family, right_leaf, right_seed, seed_mem,
            sibling_bucket_pair, sum_proof_bytes, FixtureRightClass, HjmtEnvGuard, SchedEnv,
        },
    },
    settlement::{
        chk_blob_settlement_inclusion, proof_blob_rebind_root, BatchProofBlobV1,
        BatchProofFamilyTagV1, BucketEpoch, DefinitionId, ProofBlob, ProofItem, SerialId,
        SettlementLeaf, SettlementLeafFamily, SettlementPath, SettlementStore,
        SettlementTreeBackend, StoreItem, StoreOp, TerminalId, TerminalLeaf,
    },
};
use z00z_utils::codec::{BincodeCodec, Codec};

fn inclusion_store() -> (SettlementStore, z00z_storage::settlement::SettlementPath) {
    let fixture = load_fixture();
    let items = settlement_corpus::load_fixture_items(&fixture);
    let store = seed_mem(&items);
    (store, settlement_corpus::asset_path(&fixture.assets[0]))
}

fn right_store() -> (SettlementStore, z00z_storage::settlement::SettlementPath) {
    let fixture = load_fixture();
    let items = settlement_corpus::load_fixture_items(&fixture);
    let store = seed_mem(&items);
    (store, settlement_corpus::right_path(&fixture.rights[0]))
}

fn deletion_store() -> (SettlementStore, z00z_storage::settlement::SettlementPath) {
    let right = right_seed(0x31, 7, 9, FixtureRightClass::OneTimeUse);
    let _guard = HjmtEnvGuard::with_bits("2");
    let mut store = SettlementStore::new();
    let path = settlement_corpus::right_path(&right);
    let _ = create_right_fee(&mut store, &right, 71).expect("seed right");
    let _ = consume_right_fee(&mut store, &right, 72).expect("consume right");
    (store, path)
}

fn split_store() -> (
    HjmtEnvGuard,
    SettlementStore,
    z00z_storage::settlement::SettlementPath,
) {
    let guard = HjmtEnvGuard::with_bits("1");
    let mut store = SettlementStore::new();
    let left = settlement_corpus::split_ready_paths(&mut store, 41, 9)[0];
    (guard, store, left)
}

fn merge_store() -> (
    HjmtEnvGuard,
    SettlementStore,
    z00z_storage::settlement::SettlementPath,
    z00z_storage::settlement::SettlementPath,
) {
    let guard = HjmtEnvGuard::with_bits("2");
    let mut store = SettlementStore::new();
    let (left, right) = sibling_bucket_pair(&mut store, 33, 11);
    (guard, store, left, right)
}

fn shared_parent_store() -> (SettlementStore, Vec<SettlementPath>) {
    let items = hot_assets(0x73, 11, 16);
    let paths = items
        .iter()
        .take(8)
        .map(|item| item.path())
        .collect::<Vec<_>>();
    (seed_mem(&items), paths)
}

fn mixed_batch_store() -> (SettlementStore, Vec<SettlementPath>, SettlementPath) {
    let items = hot_assets(0x74, 12, 16);
    let paths = items
        .iter()
        .take(8)
        .map(|item| item.path())
        .collect::<Vec<_>>();
    let missing = settlement_corpus::asset_path(&asset_seed(0x74, 12, 0xEE, 104_000));
    (seed_mem(&items), paths, missing)
}

fn proof_blob_via_backend<T: SettlementTreeBackend>(store: &T, path: &SettlementPath) -> ProofBlob {
    store
        .settlement_proof_blob(path)
        .expect("semantic proof surface")
}

fn warm_inclusion_store() -> (SettlementStore, z00z_storage::settlement::SettlementPath) {
    let (store, path) = inclusion_store();
    let _ = proof_blob_via_backend(&store, &path);
    (store, path)
}

fn warm_split_store() -> (
    HjmtEnvGuard,
    SettlementStore,
    z00z_storage::settlement::SettlementPath,
) {
    let (guard, store, path) = split_store();
    let _ = store.split_proof(&path).expect("warm split proof");
    (guard, store, path)
}

fn warm_merge_store() -> (
    HjmtEnvGuard,
    SettlementStore,
    z00z_storage::settlement::SettlementPath,
    z00z_storage::settlement::SettlementPath,
) {
    let (guard, store, left, right) = merge_store();
    let _ = store.merge_proof(&left, &right).expect("warm merge proof");
    (guard, store, left, right)
}

fn warm_policy_store() -> (
    HjmtEnvGuard,
    SettlementStore,
    z00z_storage::settlement::BucketPolicy,
) {
    let (guard, store, _path) = split_store();
    let next = next_policy(&store);
    let _ = store
        .policy_transition_proof(next)
        .expect("warm policy proof");
    (guard, store, next)
}

const BATCH_COMPARE_FULL_COUNTS: [usize; 6] = [2, 8, 32, 128, 1000, 1024];
const BATCH_COMPARE_EVIDENCE_COUNTS: [usize; 3] = [2, 8, 32];

#[derive(Clone, Copy)]
enum BatchCompareFamily {
    Inclusion,
    Deletion,
    NonExistence,
}

#[derive(Clone, Copy)]
enum BatchCompareShape {
    Clustered,
    Scattered,
}

#[derive(Clone, Copy)]
enum BatchCompareSurface {
    Single,
    Vec,
    Batch,
}

enum CompareVerifyFixture {
    Single(Box<ProofBlob>),
    Vec(Vec<ProofBlob>),
    Batch(Box<BatchProofBlobV1>),
}

#[derive(Clone)]
struct BatchOnlyNoteRow {
    proof_surface: String,
    path_count: usize,
    path_shape: String,
    proof_family: String,
    serialized_bytes: usize,
    prove_time_us: u128,
}

static BATCH_ONLY_NOTE_ROWS: OnceLock<Mutex<BTreeMap<String, BatchOnlyNoteRow>>> = OnceLock::new();

impl BatchCompareFamily {
    const fn as_str(self) -> &'static str {
        match self {
            Self::Inclusion => "inclusion",
            Self::Deletion => "deletion",
            Self::NonExistence => "nonexistence",
        }
    }
}

impl BatchCompareShape {
    const fn as_str(self) -> &'static str {
        match self {
            Self::Clustered => "clustered",
            Self::Scattered => "scattered",
        }
    }
}

impl BatchCompareSurface {
    const fn as_str(self) -> &'static str {
        match self {
            Self::Single => "proof_blob_single",
            Self::Vec => "proof_blob_vec",
            Self::Batch => "batch_proof_v1",
        }
    }
}

fn batch_only_note_rows() -> &'static Mutex<BTreeMap<String, BatchOnlyNoteRow>> {
    BATCH_ONLY_NOTE_ROWS.get_or_init(|| Mutex::new(BTreeMap::new()))
}

fn batch_only_note_key(
    surface: BatchCompareSurface,
    shape: Option<BatchCompareShape>,
    family: BatchCompareFamily,
    count: usize,
) -> String {
    match shape {
        Some(shape) => format!(
            "{}/{}/{}/{}",
            surface.as_str(),
            shape.as_str(),
            family.as_str(),
            count
        ),
        None => format!("{}/{}", surface.as_str(), family.as_str()),
    }
}

fn batch_only_note_row(
    surface: BatchCompareSurface,
    shape: Option<BatchCompareShape>,
    family: BatchCompareFamily,
    count: usize,
    serialized_bytes: usize,
    prove_time_us: u128,
) -> BatchOnlyNoteRow {
    let path_shape = shape.map_or_else(|| "single".to_string(), |value| value.as_str().to_string());
    BatchOnlyNoteRow {
        proof_surface: surface.as_str().to_string(),
        path_count: count,
        path_shape,
        proof_family: family.as_str().to_string(),
        serialized_bytes,
        prove_time_us,
    }
}

fn record_batch_only_note_row(
    surface: BatchCompareSurface,
    shape: Option<BatchCompareShape>,
    family: BatchCompareFamily,
    count: usize,
    serialized_bytes: usize,
    prove_time_us: u128,
) {
    if !matches!(proof_note_scope(), ProofNoteScope::BatchOnly) {
        return;
    }
    let key = batch_only_note_key(surface, shape, family, count);
    let row = batch_only_note_row(
        surface,
        shape,
        family,
        count,
        serialized_bytes,
        prove_time_us,
    );
    batch_only_note_rows()
        .lock()
        .expect("batch-only note rows lock")
        .insert(key, row);
}

fn take_batch_only_note_rows() -> Vec<BatchOnlyNoteRow> {
    let mut rows = batch_only_note_rows()
        .lock()
        .expect("batch-only note rows lock");
    let collected = rows.values().cloned().collect::<Vec<_>>();
    rows.clear();
    collected
}

fn note_runs_direct_matrix() -> bool {
    matches!(proof_note_scope(), ProofNoteScope::BatchOnly)
        && matches!(proof_note_filter().as_deref(), Some("hjmt_batch_"))
}

fn active_batch_compare_counts() -> &'static [usize] {
    if note_runs_direct_matrix() {
        &BATCH_COMPARE_EVIDENCE_COUNTS
    } else {
        &BATCH_COMPARE_FULL_COUNTS
    }
}

fn expected_batch_only_note_rows() -> usize {
    3 + (3 * 2 * active_batch_compare_counts().len() * 2)
}

fn collect_note_rows() -> Vec<BatchOnlyNoteRow> {
    let mut rows = Vec::with_capacity(expected_batch_only_note_rows());
    for family in [
        BatchCompareFamily::Inclusion,
        BatchCompareFamily::Deletion,
        BatchCompareFamily::NonExistence,
    ] {
        let (store, paths) = compare_fixture(family, BatchCompareShape::Clustered, 1);
        let (serialized_bytes, prove_time_us, _) = measure_compare_note_surface(
            &store,
            &paths,
            family,
            BatchCompareSurface::Single,
            false,
        );
        rows.push(batch_only_note_row(
            BatchCompareSurface::Single,
            None,
            family,
            1,
            serialized_bytes,
            prove_time_us,
        ));
    }

    for family in [
        BatchCompareFamily::Inclusion,
        BatchCompareFamily::Deletion,
        BatchCompareFamily::NonExistence,
    ] {
        for shape in [BatchCompareShape::Clustered, BatchCompareShape::Scattered] {
            for &count in active_batch_compare_counts() {
                let (store, paths) = compare_fixture(family, shape, count);
                for surface in [BatchCompareSurface::Vec, BatchCompareSurface::Batch] {
                    let (serialized_bytes, prove_time_us, _) =
                        measure_compare_note_surface(&store, &paths, family, surface, false);
                    rows.push(batch_only_note_row(
                        surface,
                        Some(shape),
                        family,
                        count,
                        serialized_bytes,
                        prove_time_us,
                    ));
                }
            }
        }
    }
    rows
}

fn compare_path_seed(
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

fn compare_asset_item(path: SettlementPath) -> StoreItem {
    let mut core = AssetLeaf::dummy_for_scan(path.serial_id.get());
    core.asset_id = path.terminal_id().into_bytes();
    let leaf = SettlementLeaf::Terminal(TerminalLeaf::from(core));
    StoreItem::new(path, leaf).expect("comparison item")
}

fn compare_same_bucket_paths(
    policy: z00z_storage::settlement::BucketPolicy,
    definition_mark: u8,
    serial_id: u32,
    needed: usize,
    start_seed: u32,
) -> Vec<SettlementPath> {
    let definition_id = DefinitionId::new([definition_mark; 32]);
    let serial_id = SerialId::new(serial_id);
    let base = compare_path_seed(definition_id, serial_id, start_seed);
    let target_bucket = base.bucket_id(policy);
    let mut paths = Vec::with_capacity(needed);
    let mut seen = BTreeSet::new();
    let mut seed = start_seed;
    loop {
        let path = compare_path_seed(definition_id, serial_id, seed);
        if path.bucket_id(policy) != target_bucket || !seen.insert(path) {
            assert!(seed < u32::MAX, "missing clustered comparison paths");
            seed += 1;
            continue;
        }
        paths.push(path);
        if paths.len() == needed {
            break;
        }
        assert!(seed < u32::MAX, "missing clustered comparison paths");
        seed += 1;
    }
    assert_eq!(paths.len(), needed, "missing clustered comparison paths");
    paths.sort_unstable();
    paths
}

fn compare_scattered_paths(
    definition_mark: u8,
    serial_base: u32,
    needed: usize,
    start_seed: u32,
) -> Vec<SettlementPath> {
    let mut paths = (0..needed)
        .map(|idx| {
            let mark = definition_mark.wrapping_add(u8::try_from(idx % 29).expect("u8"));
            let definition_id = DefinitionId::new([mark; 32]);
            let serial_id = SerialId::new(serial_base + u32::try_from(idx).expect("u32"));
            compare_path_seed(
                definition_id,
                serial_id,
                start_seed + u32::try_from(idx).expect("u32"),
            )
        })
        .collect::<Vec<_>>();
    paths.sort_unstable();
    paths
}

fn compare_seed_paths(store: &mut SettlementStore, paths: &[SettlementPath]) {
    for path in paths {
        store
            .put_settlement_item(compare_asset_item(*path))
            .expect("seed comparison path");
    }
}

fn compare_missing_paths(
    policy: z00z_storage::settlement::BucketPolicy,
    present_paths: &[SettlementPath],
    require_same_bucket: bool,
    start_seed: u32,
) -> Vec<SettlementPath> {
    let mut seen = present_paths.iter().copied().collect::<BTreeSet<_>>();
    let mut paths = Vec::with_capacity(present_paths.len());
    let mut seed = start_seed;
    for (idx, base) in present_paths.iter().copied().enumerate() {
        let target_bucket = base.bucket_id(policy);
        let mut selected = None;
        loop {
            let path = compare_path_seed(
                base.definition_id,
                base.serial_id,
                seed.saturating_add(u32::try_from(idx).expect("u32")),
            );
            let exhausted = seed == u32::MAX;
            if !exhausted {
                seed += 1;
            }
            if path == base || seen.contains(&path) {
                if exhausted {
                    break;
                }
                continue;
            }
            let same_bucket = path.bucket_id(policy) == target_bucket;
            if require_same_bucket != same_bucket {
                if exhausted {
                    break;
                }
                continue;
            }
            selected = Some(path);
            break;
        }
        let selected = selected.expect("missing comparison nonexistence path");
        seen.insert(selected);
        paths.push(selected);
    }
    paths.sort_unstable();
    paths
}

fn compare_same_bucket_companions(
    policy: z00z_storage::settlement::BucketPolicy,
    base_paths: &[SettlementPath],
    extra_per_path: usize,
    start_seed: u32,
) -> Vec<SettlementPath> {
    let mut seen = base_paths.iter().copied().collect::<BTreeSet<_>>();
    let mut paths = Vec::with_capacity(base_paths.len() * extra_per_path);
    let mut seed = start_seed;
    for base in base_paths {
        let target_bucket = base.bucket_id(policy);
        for _ in 0..extra_per_path {
            let selected = loop {
                let candidate = compare_path_seed(base.definition_id, base.serial_id, seed);
                let exhausted = seed == u32::MAX;
                if !exhausted {
                    seed += 1;
                }
                if candidate == *base || seen.contains(&candidate) {
                    if exhausted {
                        break None;
                    }
                    continue;
                }
                if candidate.bucket_id(policy) != target_bucket {
                    if exhausted {
                        break None;
                    }
                    continue;
                }
                break Some(candidate);
            };
            let selected = selected.expect("missing same-bucket companion path");
            seen.insert(selected);
            paths.push(selected);
        }
    }
    paths.sort_unstable();
    paths
}

fn compare_batchable_nonexistence_paths(
    store: &SettlementStore,
    present_paths: &[SettlementPath],
    start_seed: u32,
) -> Vec<SettlementPath> {
    // The current batch v1 envelope can only consume scattered nonexistence paths
    // that already lower to a valid single-path batch surface.
    let policy = store.bucket_policy();
    let mut seen = present_paths.iter().copied().collect::<BTreeSet<_>>();
    let mut paths = Vec::with_capacity(present_paths.len());
    let mut seed = start_seed;
    for (idx, base) in present_paths.iter().copied().enumerate() {
        let target_bucket = base.bucket_id(policy);
        let selected = loop {
            let candidate = compare_path_seed(
                base.definition_id,
                base.serial_id,
                seed.saturating_add(u32::try_from(idx).expect("u32")),
            );
            assert!(
                seed < u32::MAX,
                "missing batchable comparison nonexistence path"
            );
            seed += 1;
            if candidate == base || seen.contains(&candidate) {
                continue;
            }
            if candidate.bucket_id(policy) != target_bucket {
                continue;
            }
            if store
                .settlement_nonexistence_batch_v1(&[candidate], SettlementLeafFamily::Terminal)
                .is_err()
            {
                continue;
            }
            break candidate;
        };
        seen.insert(selected);
        paths.push(selected);
    }
    paths.sort_unstable();
    paths
}

fn compare_fixture(
    family: BatchCompareFamily,
    shape: BatchCompareShape,
    path_count: usize,
) -> (SettlementStore, Vec<SettlementPath>) {
    let mut store = SettlementStore::new();
    let paths = match (family, shape) {
        (_, BatchCompareShape::Clustered) => {
            compare_same_bucket_paths(store.bucket_policy(), 0xA1, 41, path_count, 1)
        }
        (_, BatchCompareShape::Scattered) => compare_scattered_paths(0xB1, 141, path_count, 10_000),
    };
    compare_seed_paths(&mut store, &paths);
    match family {
        BatchCompareFamily::Inclusion => (store, paths),
        BatchCompareFamily::Deletion => {
            let survivors = compare_same_bucket_companions(
                store.bucket_policy(),
                &paths,
                match shape {
                    BatchCompareShape::Clustered => 1,
                    BatchCompareShape::Scattered => 2,
                },
                20_000,
            );
            compare_seed_paths(&mut store, &survivors);
            store
                .apply_settlement_ops(paths.iter().copied().map(StoreOp::Delete).collect())
                .expect("delete comparison paths");
            (store, paths)
        }
        BatchCompareFamily::NonExistence => {
            if matches!(shape, BatchCompareShape::Scattered) {
                let companions =
                    compare_same_bucket_companions(store.bucket_policy(), &paths, 2, 20_000);
                compare_seed_paths(&mut store, &companions);
            }
            let missing = if matches!(shape, BatchCompareShape::Scattered) {
                compare_batchable_nonexistence_paths(&store, &paths, 30_000)
            } else {
                compare_missing_paths(store.bucket_policy(), &paths, true, 30_000)
            };
            (store, missing)
        }
    }
}

fn build_compare_single(
    store: &SettlementStore,
    family: BatchCompareFamily,
    path: SettlementPath,
) -> ProofBlob {
    match family {
        BatchCompareFamily::Inclusion | BatchCompareFamily::Deletion => {
            let blob = store.settlement_proof_blob(&path).expect("single proof");
            store
                .validate_settlement_proof_blob(&blob)
                .expect("single verify");
            blob
        }
        BatchCompareFamily::NonExistence => {
            let blob = store
                .settlement_nonexistence_proof_blob(&path, SettlementLeafFamily::Terminal)
                .expect("single absence");
            store
                .validate_settlement_nonexistence_proof_blob(&blob, SettlementLeafFamily::Terminal)
                .expect("single absence verify");
            blob
        }
    }
}

fn build_compare_vec(
    store: &SettlementStore,
    family: BatchCompareFamily,
    paths: &[SettlementPath],
) -> Vec<ProofBlob> {
    match family {
        BatchCompareFamily::Inclusion | BatchCompareFamily::Deletion => {
            let blobs = store
                .settlement_proof_blobs(paths)
                .expect("independent proof vec");
            for blob in &blobs {
                store
                    .validate_settlement_proof_blob(blob)
                    .expect("independent verify");
            }
            blobs
        }
        BatchCompareFamily::NonExistence => paths
            .iter()
            .map(|path| {
                let blob = store
                    .settlement_nonexistence_proof_blob(path, SettlementLeafFamily::Terminal)
                    .expect("independent absence");
                store
                    .validate_settlement_nonexistence_proof_blob(
                        &blob,
                        SettlementLeafFamily::Terminal,
                    )
                    .expect("independent absence verify");
                blob
            })
            .collect(),
    }
}

fn build_compare_batch(
    store: &SettlementStore,
    family: BatchCompareFamily,
    paths: &[SettlementPath],
) -> BatchProofBlobV1 {
    let batch = match family {
        BatchCompareFamily::Inclusion => store
            .settlement_inclusion_batch_v1(paths)
            .expect("inclusion batch"),
        BatchCompareFamily::Deletion => store
            .settlement_deletion_batch_v1(paths)
            .expect("deletion batch"),
        BatchCompareFamily::NonExistence => store
            .settlement_nonexistence_batch_v1(paths, SettlementLeafFamily::Terminal)
            .expect("nonexistence batch"),
    };
    let bytes = batch.encode().expect("encode batch");
    let decoded = BatchProofBlobV1::decode(&bytes).expect("verify batch");
    assert_eq!(decoded, batch, "batch decode drift");
    batch
}

fn build_malformed_batch_bytes(
    store: &SettlementStore,
    family: BatchCompareFamily,
    paths: &[SettlementPath],
) -> Vec<u8> {
    let mut bytes = build_compare_batch(store, family, paths)
        .encode()
        .expect("encode malformed batch seed");
    bytes.truncate(bytes.len().saturating_sub(1));
    bytes
}

fn build_mixed_family_batch_bytes(store: &SettlementStore, paths: &[SettlementPath]) -> Vec<u8> {
    let mut batch = build_compare_batch(store, BatchCompareFamily::Inclusion, paths);
    batch.header.proof_family = BatchProofFamilyTagV1::Deletion;
    batch.encode().expect("encode mixed family batch")
}

fn build_compare_verify_fixture(
    store: &SettlementStore,
    family: BatchCompareFamily,
    surface: BatchCompareSurface,
    paths: &[SettlementPath],
) -> CompareVerifyFixture {
    match surface {
        BatchCompareSurface::Single => {
            CompareVerifyFixture::Single(Box::new(build_compare_single(store, family, paths[0])))
        }
        BatchCompareSurface::Vec => {
            CompareVerifyFixture::Vec(build_compare_vec(store, family, paths))
        }
        BatchCompareSurface::Batch => {
            CompareVerifyFixture::Batch(Box::new(build_compare_batch(store, family, paths)))
        }
    }
}

fn compare_fixture_verify(
    store: &SettlementStore,
    family: BatchCompareFamily,
    fixture: &CompareVerifyFixture,
) {
    match fixture {
        CompareVerifyFixture::Single(blob) => match family {
            BatchCompareFamily::Inclusion | BatchCompareFamily::Deletion => {
                store
                    .validate_settlement_proof_blob(blob)
                    .expect("single verify");
                black_box(());
            }
            BatchCompareFamily::NonExistence => {
                store
                    .validate_settlement_nonexistence_proof_blob(
                        blob,
                        SettlementLeafFamily::Terminal,
                    )
                    .expect("single absence verify");
                black_box(());
            }
        },
        CompareVerifyFixture::Vec(blobs) => match family {
            BatchCompareFamily::Inclusion | BatchCompareFamily::Deletion => {
                for blob in blobs {
                    store
                        .validate_settlement_proof_blob(blob)
                        .expect("independent verify");
                }
                black_box(());
            }
            BatchCompareFamily::NonExistence => {
                for blob in blobs {
                    store
                        .validate_settlement_nonexistence_proof_blob(
                            blob,
                            SettlementLeafFamily::Terminal,
                        )
                        .expect("independent absence verify");
                }
                black_box(());
            }
        },
        CompareVerifyFixture::Batch(batch) => {
            batch.check_contract_v1().expect("batch verify");
            black_box(());
        }
    }
}

fn bench_generate(c: &mut Criterion) {
    let mut group = c.benchmark_group("proof_generate");
    group.bench_function("inclusion_asset", |b| {
        b.iter_batched(
            inclusion_store,
            |(store, path)| {
                black_box(proof_blob_via_backend(&store, &path));
            },
            BatchSize::SmallInput,
        )
    });
    group.bench_function("inclusion_right", |b| {
        b.iter_batched(
            right_store,
            |(store, path)| {
                black_box(proof_blob_via_backend(&store, &path));
            },
            BatchSize::SmallInput,
        )
    });
    group.bench_function("deletion_right", |b| {
        b.iter_batched(
            deletion_store,
            |(store, path)| {
                black_box(proof_blob_via_backend(&store, &path));
            },
            BatchSize::SmallInput,
        )
    });
    group.bench_function("nonexistence_asset", |b| {
        b.iter_batched(
            || {
                let (store, _) = inclusion_store();
                let path = settlement_corpus::asset_path(&asset_seed(0x91, 77, 88, 101_000));
                (store, path)
            },
            |(store, path)| {
                black_box(
                    store
                        .settlement_nonexistence_proof_blob(&path, SettlementLeafFamily::Terminal)
                        .expect("nonexistence asset"),
                );
            },
            BatchSize::SmallInput,
        )
    });
    group.bench_function("split", |b| {
        b.iter_batched(
            split_store,
            |(_guard, store, path)| {
                black_box(store.split_proof(&path).expect("split proof"));
            },
            BatchSize::SmallInput,
        )
    });
    group.bench_function("merge", |b| {
        b.iter_batched(
            merge_store,
            |(_guard, store, left, right)| {
                black_box(store.merge_proof(&left, &right).expect("merge proof"));
            },
            BatchSize::SmallInput,
        )
    });
    group.bench_function("policy_transition", |b| {
        b.iter_batched(
            || {
                let (guard, store, _path) = split_store();
                let next = next_policy(&store);
                (guard, store, next)
            },
            |(_guard, store, next)| {
                black_box(
                    store
                        .policy_transition_proof(next)
                        .expect("policy transition proof"),
                );
            },
            BatchSize::SmallInput,
        )
    });
    group.bench_function("shared_parent_batch", |b| {
        b.iter_batched(
            shared_parent_store,
            |(store, paths)| {
                black_box(
                    store
                        .settlement_proof_blobs(&paths)
                        .expect("shared parent batch"),
                );
            },
            BatchSize::SmallInput,
        )
    });
    group.bench_function("mixed_inclusion_nonexistence_batch", |b| {
        b.iter_batched(
            mixed_batch_store,
            |(store, paths, missing)| {
                let inclusion = store
                    .settlement_proof_blobs(&paths)
                    .expect("mixed inclusion batch");
                let absence = store
                    .settlement_nonexistence_proof_blob(&missing, SettlementLeafFamily::Terminal)
                    .expect("mixed absence proof");
                black_box((inclusion, absence));
            },
            BatchSize::SmallInput,
        )
    });
    group.finish();

    let mut cache_group = c.benchmark_group("proof_generate_cache_state");
    cache_group.bench_function("cold_inclusion_asset", |b| {
        b.iter_batched(
            inclusion_store,
            |(store, path)| {
                black_box(proof_blob_via_backend(&store, &path));
            },
            BatchSize::SmallInput,
        )
    });
    cache_group.bench_function("warm_inclusion_asset", |b| {
        b.iter_batched(
            warm_inclusion_store,
            |(store, path)| {
                black_box(
                    store
                        .settlement_proof_blob(&path)
                        .expect("warm inclusion asset"),
                );
            },
            BatchSize::SmallInput,
        )
    });
    cache_group.bench_function("cold_split", |b| {
        b.iter_batched(
            split_store,
            |(_guard, store, path)| {
                black_box(store.split_proof(&path).expect("cold split proof"));
            },
            BatchSize::SmallInput,
        )
    });
    cache_group.bench_function("warm_split", |b| {
        b.iter_batched(
            warm_split_store,
            |(_guard, store, path)| {
                black_box(store.split_proof(&path).expect("warm split proof"));
            },
            BatchSize::SmallInput,
        )
    });
    cache_group.bench_function("cold_merge", |b| {
        b.iter_batched(
            merge_store,
            |(_guard, store, left, right)| {
                black_box(store.merge_proof(&left, &right).expect("cold merge proof"));
            },
            BatchSize::SmallInput,
        )
    });
    cache_group.bench_function("warm_merge", |b| {
        b.iter_batched(
            warm_merge_store,
            |(_guard, store, left, right)| {
                black_box(store.merge_proof(&left, &right).expect("warm merge proof"));
            },
            BatchSize::SmallInput,
        )
    });
    cache_group.bench_function("cold_policy_transition", |b| {
        b.iter_batched(
            || {
                let (guard, store, _path) = split_store();
                let next = next_policy(&store);
                (guard, store, next)
            },
            |(_guard, store, next)| {
                black_box(
                    store
                        .policy_transition_proof(next)
                        .expect("cold policy transition proof"),
                );
            },
            BatchSize::SmallInput,
        )
    });
    cache_group.bench_function("warm_policy_transition", |b| {
        b.iter_batched(
            warm_policy_store,
            |(_guard, store, next)| {
                black_box(
                    store
                        .policy_transition_proof(next)
                        .expect("warm policy transition proof"),
                );
            },
            BatchSize::SmallInput,
        )
    });
    cache_group.finish();
}

fn bench_verify(c: &mut Criterion) {
    let mut group = c.benchmark_group("proof_verify");
    group.bench_function("inclusion_asset", |b| {
        b.iter_batched(
            || {
                let (store, path) = inclusion_store();
                let blob = store.settlement_proof_blob(&path).expect("inclusion asset");
                (store, blob)
            },
            |(store, blob)| {
                store
                    .validate_settlement_proof_blob(&blob)
                    .expect("verify inclusion");
                black_box(());
            },
            BatchSize::SmallInput,
        )
    });
    group.bench_function("deletion_right", |b| {
        b.iter_batched(
            || {
                let (store, path) = deletion_store();
                let blob = store.settlement_proof_blob(&path).expect("deletion proof");
                (store, blob)
            },
            |(store, blob)| {
                store
                    .validate_settlement_proof_blob(&blob)
                    .expect("verify deletion");
                black_box(());
            },
            BatchSize::SmallInput,
        )
    });
    group.bench_function("nonexistence_asset", |b| {
        b.iter_batched(
            || {
                let (store, _) = inclusion_store();
                let path = settlement_corpus::asset_path(&asset_seed(0x92, 78, 89, 102_000));
                let blob = store
                    .settlement_nonexistence_proof_blob(&path, SettlementLeafFamily::Terminal)
                    .expect("nonexistence proof");
                (store, blob)
            },
            |(store, blob)| {
                store
                    .validate_settlement_nonexistence_proof_blob(
                        &blob,
                        SettlementLeafFamily::Terminal,
                    )
                    .expect("verify nonexistence");
                black_box(());
            },
            BatchSize::SmallInput,
        )
    });
    group.bench_function("tampered_reject", |b| {
        b.iter_batched(
            || {
                let (store, path) = inclusion_store();
                let blob = store.settlement_proof_blob(&path).expect("inclusion asset");
                let bytes = blob.encode().expect("encode");
                let rebound = proof_blob_rebind_root(
                    &bytes,
                    z00z_storage::settlement::SettlementStateRoot::settlement_v1([0x55; 32]),
                )
                .expect("rebind root");
                let tampered = ProofBlob::decode(&rebound).expect("decode tampered");
                (store, tampered)
            },
            |(store, tampered)| {
                black_box(
                    store
                        .validate_settlement_proof_blob(&tampered)
                        .expect_err("tampered reject"),
                );
            },
            BatchSize::SmallInput,
        )
    });
    group.bench_function("split", |b| {
        b.iter_batched(
            || {
                let (guard, store, path) = split_store();
                let proof = store.split_proof(&path).expect("split proof");
                (guard, store, proof)
            },
            |(_guard, store, proof)| {
                store.validate_split_proof(&proof).expect("verify split");
                black_box(());
            },
            BatchSize::SmallInput,
        )
    });
    group.bench_function("merge", |b| {
        b.iter_batched(
            || {
                let (guard, store, left, right) = merge_store();
                let proof = store.merge_proof(&left, &right).expect("merge proof");
                (guard, store, proof)
            },
            |(_guard, store, proof)| {
                store.validate_merge_proof(&proof).expect("verify merge");
                black_box(());
            },
            BatchSize::SmallInput,
        )
    });
    group.bench_function("policy_transition", |b| {
        b.iter_batched(
            || {
                let (guard, store, _path) = split_store();
                let next = next_policy(&store);
                let proof = store
                    .policy_transition_proof(next)
                    .expect("policy transition proof");
                (guard, store, proof, next)
            },
            |(_guard, store, proof, next)| {
                store
                    .validate_policy_transition_proof(&proof, next)
                    .expect("verify transition");
                black_box(());
            },
            BatchSize::SmallInput,
        )
    });
    group.bench_function("nonexistence_default_commitment_reject", |b| {
        b.iter_batched(
            || {
                let (store, _) = inclusion_store();
                let path = settlement_corpus::asset_path(&asset_seed(0x92, 78, 89, 102_000));
                let blob = store
                    .settlement_nonexistence_proof_blob(&path, SettlementLeafFamily::Terminal)
                    .expect("nonexistence proof");
                let tampered = blob.with_hjmt_default_commitment(Some([0u8; 32]));
                (store, tampered)
            },
            |(store, tampered)| {
                black_box(
                    store
                        .validate_settlement_proof_blob(&tampered)
                        .expect_err("default commitment reject"),
                );
            },
            BatchSize::SmallInput,
        )
    });
    group.bench_function("present_key_absence_reject", |b| {
        b.iter_batched(
            || {
                let fixture = load_fixture();
                let present = fixture.assets[0].clone();
                let items = settlement_corpus::load_fixture_items(&fixture);
                let store = seed_mem(&items);
                let present_path = settlement_corpus::asset_path(&present);
                let missing_path = settlement_corpus::asset_path(&asset_seed(
                    present.definition_mark,
                    present.serial_id,
                    0xEE,
                    106_000,
                ));
                let blob = store
                    .settlement_nonexistence_proof_blob(
                        &missing_path,
                        SettlementLeafFamily::Terminal,
                    )
                    .expect("absence proof");
                let marker_leaf = SettlementLeafFamily::Terminal.marker_leaf(present_path);
                let item = ProofItem::new_settlement(
                    blob.item().settlement_root(),
                    present_path,
                    blob.item().def_leaf(),
                    blob.item().ser_leaf(),
                    marker_leaf.clone(),
                )
                .expect("present-path marker item");
                let marker_hash =
                    ValueHash::with::<Sha256>(&marker_leaf.encode().expect("marker leaf bytes")).0;
                let tampered = blob.rebind(item).with_terminal_leaf_hash(marker_hash);
                (store, tampered)
            },
            |(store, tampered)| {
                black_box(
                    store
                        .validate_settlement_nonexistence_proof_blob(
                            &tampered,
                            SettlementLeafFamily::Terminal,
                        )
                        .expect_err("present key absence reject"),
                );
            },
            BatchSize::SmallInput,
        )
    });
    group.bench_function("split_wrong_epoch_reject", |b| {
        b.iter_batched(
            || {
                let (guard, store, path) = split_store();
                let proof = store.split_proof(&path).expect("split proof");
                let tampered = z00z_storage::settlement::SplitProof {
                    prior_epoch: BucketEpoch::new(proof.prior_epoch.get() + 1),
                    ..proof
                };
                (guard, store, tampered)
            },
            |(_guard, store, tampered)| {
                black_box(
                    store
                        .validate_split_proof(&tampered)
                        .expect_err("wrong epoch reject"),
                );
            },
            BatchSize::SmallInput,
        )
    });
    group.bench_function("policy_transition_stale_reject", |b| {
        b.iter_batched(
            || {
                let (guard, store, _path) = split_store();
                let next = next_policy(&store);
                let proof = store
                    .policy_transition_proof(next)
                    .expect("policy transition proof");
                let stale = z00z_storage::settlement::PolicyTransitionProof {
                    prior_policy_id: [0x11; 32],
                    ..proof
                };
                (guard, store, stale, next)
            },
            |(_guard, store, stale, next)| {
                black_box(
                    store
                        .validate_policy_transition_proof(&stale, next)
                        .expect_err("stale policy reject"),
                );
            },
            BatchSize::SmallInput,
        )
    });
    group.bench_function("historical_policy_transition", |b| {
        b.iter_batched(
            || {
                let (guard, mut store, _path) = split_store();
                let next = next_policy(&store);
                let proof = store
                    .policy_transition_proof(next)
                    .expect("policy transition proof");
                let _ = store
                    .put_settlement_item(asset_item(&asset_seed(0x95, 1, 0x61, 105_000)))
                    .expect("advance historical store");
                (guard, store, proof, next)
            },
            |(_guard, store, proof, next)| {
                store
                    .validate_policy_transition_proof(&proof, next)
                    .expect("historical transition verify");
                black_box(());
            },
            BatchSize::SmallInput,
        )
    });
    group.bench_function("malformed_bytes_reject", |b| {
        b.iter_batched(
            || {
                let (store, path) = inclusion_store();
                let blob = store.settlement_proof_blob(&path).expect("inclusion asset");
                let item = blob.item();
                let mut bytes = blob.encode().expect("encode proof blob");
                bytes.truncate(bytes.len().saturating_sub(1));
                (
                    bytes,
                    item.settlement_root(),
                    item.path(),
                    item.def_leaf(),
                    item.ser_leaf(),
                    item.terminal_leaf().expect("asset leaf").clone(),
                )
            },
            |(bytes, root, path, def_leaf, ser_leaf, leaf)| {
                black_box(
                    chk_blob_settlement_inclusion(&bytes, root, &path, def_leaf, ser_leaf, leaf)
                        .expect_err("malformed bytes reject"),
                );
            },
            BatchSize::SmallInput,
        )
    });
    group.bench_function("root_bind_version_reject", |b| {
        b.iter_batched(
            || {
                let (store, path) = inclusion_store();
                let blob = store.settlement_proof_blob(&path).expect("inclusion asset");
                let root_bind = blob.root_bind();
                let tampered = blob.with_root_bind(2, root_bind);
                (store, tampered)
            },
            |(store, tampered)| {
                black_box(
                    store
                        .validate_settlement_proof_blob(&tampered)
                        .expect_err("root bind version reject"),
                );
            },
            BatchSize::SmallInput,
        )
    });
    group.finish();

    let mut cache_group = c.benchmark_group("proof_verify_cache_state");
    cache_group.bench_function("cold_inclusion_asset", |b| {
        b.iter_batched(
            || {
                let (store, path) = inclusion_store();
                let blob = store
                    .settlement_proof_blob(&path)
                    .expect("cold inclusion proof");
                (store, blob)
            },
            |(store, blob)| {
                store
                    .validate_settlement_proof_blob(&blob)
                    .expect("cold verify inclusion");
                black_box(());
            },
            BatchSize::SmallInput,
        )
    });
    cache_group.bench_function("warm_inclusion_asset", |b| {
        b.iter_batched(
            || {
                let (store, path) = inclusion_store();
                let blob = store
                    .settlement_proof_blob(&path)
                    .expect("warm inclusion proof");
                store
                    .validate_settlement_proof_blob(&blob)
                    .expect("warm verify seed");
                (store, blob)
            },
            |(store, blob)| {
                store
                    .validate_settlement_proof_blob(&blob)
                    .expect("warm verify inclusion");
                black_box(());
            },
            BatchSize::SmallInput,
        )
    });
    cache_group.bench_function("cold_split", |b| {
        b.iter_batched(
            || {
                let (guard, store, path) = split_store();
                let proof = store.split_proof(&path).expect("cold split proof");
                (guard, store, proof)
            },
            |(_guard, store, proof)| {
                store
                    .validate_split_proof(&proof)
                    .expect("cold verify split");
                black_box(());
            },
            BatchSize::SmallInput,
        )
    });
    cache_group.bench_function("warm_split", |b| {
        b.iter_batched(
            || {
                let (guard, store, path) = split_store();
                let proof = store.split_proof(&path).expect("warm split proof");
                store.validate_split_proof(&proof).expect("warm split seed");
                (guard, store, proof)
            },
            |(_guard, store, proof)| {
                store
                    .validate_split_proof(&proof)
                    .expect("warm verify split");
                black_box(());
            },
            BatchSize::SmallInput,
        )
    });
    cache_group.bench_function("cold_merge", |b| {
        b.iter_batched(
            || {
                let (guard, store, left, right) = merge_store();
                let proof = store.merge_proof(&left, &right).expect("cold merge proof");
                (guard, store, proof)
            },
            |(_guard, store, proof)| {
                store
                    .validate_merge_proof(&proof)
                    .expect("cold verify merge");
                black_box(());
            },
            BatchSize::SmallInput,
        )
    });
    cache_group.bench_function("warm_merge", |b| {
        b.iter_batched(
            || {
                let (guard, store, left, right) = merge_store();
                let proof = store.merge_proof(&left, &right).expect("warm merge proof");
                store.validate_merge_proof(&proof).expect("warm merge seed");
                (guard, store, proof)
            },
            |(_guard, store, proof)| {
                store
                    .validate_merge_proof(&proof)
                    .expect("warm verify merge");
                black_box(());
            },
            BatchSize::SmallInput,
        )
    });
    cache_group.bench_function("cold_policy_transition", |b| {
        b.iter_batched(
            || {
                let (guard, store, _path) = split_store();
                let next = next_policy(&store);
                let proof = store
                    .policy_transition_proof(next)
                    .expect("cold policy proof");
                (guard, store, proof, next)
            },
            |(_guard, store, proof, next)| {
                store
                    .validate_policy_transition_proof(&proof, next)
                    .expect("cold verify transition");
                black_box(());
            },
            BatchSize::SmallInput,
        )
    });
    cache_group.bench_function("warm_policy_transition", |b| {
        b.iter_batched(
            || {
                let (guard, store, _path) = split_store();
                let next = next_policy(&store);
                let proof = store
                    .policy_transition_proof(next)
                    .expect("warm policy proof");
                store
                    .validate_policy_transition_proof(&proof, next)
                    .expect("warm transition seed");
                (guard, store, proof, next)
            },
            |(_guard, store, proof, next)| {
                store
                    .validate_policy_transition_proof(&proof, next)
                    .expect("warm verify transition");
                black_box(());
            },
            BatchSize::SmallInput,
        )
    });
    cache_group.finish();
}

fn bench_batch_proof_bytes(c: &mut Criterion) {
    let mut group = c.benchmark_group("hjmt_batch_proof_bytes");
    for family in [
        BatchCompareFamily::Inclusion,
        BatchCompareFamily::Deletion,
        BatchCompareFamily::NonExistence,
    ] {
        let lane = format!(
            "{}/{}",
            BatchCompareSurface::Single.as_str(),
            family.as_str()
        );
        group.bench_function(&lane, |b| {
            let (store, paths) = compare_fixture(family, BatchCompareShape::Clustered, 1);
            let (bytes, prove_time_us, _) = measure_compare_note_surface(
                &store,
                &paths,
                family,
                BatchCompareSurface::Single,
                false,
            );
            record_batch_only_note_row(
                BatchCompareSurface::Single,
                None,
                family,
                1,
                bytes,
                prove_time_us,
            );
            b.iter_batched(
                || {
                    store.clear_forest_cache();
                },
                |_| {
                    black_box(compare_surface_raw_encoded_bytes(
                        &store,
                        &paths,
                        family,
                        BatchCompareSurface::Single,
                    ));
                },
                BatchSize::SmallInput,
            )
        });
    }

    for family in [
        BatchCompareFamily::Inclusion,
        BatchCompareFamily::Deletion,
        BatchCompareFamily::NonExistence,
    ] {
        for shape in [BatchCompareShape::Clustered, BatchCompareShape::Scattered] {
            for &count in active_batch_compare_counts() {
                for surface in [BatchCompareSurface::Vec, BatchCompareSurface::Batch] {
                    let lane = format!(
                        "{}/{}/{}/{}",
                        surface.as_str(),
                        shape.as_str(),
                        family.as_str(),
                        count
                    );
                    group.bench_function(&lane, |b| {
                        let (store, paths) = compare_fixture(family, shape, count);
                        let (bytes, prove_time_us, _) =
                            measure_compare_note_surface(&store, &paths, family, surface, false);
                        record_batch_only_note_row(
                            surface,
                            Some(shape),
                            family,
                            count,
                            bytes,
                            prove_time_us,
                        );
                        b.iter_batched(
                            || {
                                store.clear_forest_cache();
                            },
                            |_| {
                                black_box(compare_surface_raw_encoded_bytes(
                                    &store, &paths, family, surface,
                                ));
                            },
                            BatchSize::SmallInput,
                        )
                    });
                }
            }
        }
    }
    group.finish();
}

fn bench_batch_verify(c: &mut Criterion) {
    let mut group = c.benchmark_group("hjmt_batch_verify");
    for family in [
        BatchCompareFamily::Inclusion,
        BatchCompareFamily::Deletion,
        BatchCompareFamily::NonExistence,
    ] {
        let lane = format!(
            "{}/{}",
            BatchCompareSurface::Single.as_str(),
            family.as_str()
        );
        group.bench_function(&lane, |b| {
            let (store, paths) = compare_fixture(family, BatchCompareShape::Clustered, 1);
            let fixture =
                build_compare_verify_fixture(&store, family, BatchCompareSurface::Single, &paths);
            b.iter_batched(
                || {
                    store.clear_forest_cache();
                },
                |_| {
                    compare_fixture_verify(&store, family, &fixture);
                },
                BatchSize::SmallInput,
            )
        });
    }

    for family in [
        BatchCompareFamily::Inclusion,
        BatchCompareFamily::Deletion,
        BatchCompareFamily::NonExistence,
    ] {
        for shape in [BatchCompareShape::Clustered, BatchCompareShape::Scattered] {
            for &count in active_batch_compare_counts() {
                for surface in [BatchCompareSurface::Vec, BatchCompareSurface::Batch] {
                    let lane = format!(
                        "{}/{}/{}/{}",
                        surface.as_str(),
                        shape.as_str(),
                        family.as_str(),
                        count
                    );
                    group.bench_function(&lane, |b| {
                        let (store, paths) = compare_fixture(family, shape, count);
                        let fixture = build_compare_verify_fixture(&store, family, surface, &paths);
                        b.iter_batched(
                            || {
                                store.clear_forest_cache();
                            },
                            |_| {
                                compare_fixture_verify(&store, family, &fixture);
                            },
                            BatchSize::SmallInput,
                        )
                    });
                }
            }
        }
    }

    group.bench_function("reject/malformed_bytes/2", |b| {
        let (store, clustered_paths) = compare_fixture(
            BatchCompareFamily::Inclusion,
            BatchCompareShape::Clustered,
            2,
        );
        let malformed =
            build_malformed_batch_bytes(&store, BatchCompareFamily::Inclusion, &clustered_paths);
        b.iter(|| {
            black_box(BatchProofBlobV1::decode(&malformed).expect_err("malformed bytes reject"));
        })
    });

    group.bench_function("reject/mixed_family/2", |b| {
        let (store, clustered_paths) = compare_fixture(
            BatchCompareFamily::Inclusion,
            BatchCompareShape::Clustered,
            2,
        );
        let mixed_family = build_mixed_family_batch_bytes(&store, &clustered_paths);
        b.iter(|| {
            black_box(BatchProofBlobV1::decode(&mixed_family).expect_err("mixed family reject"));
        })
    });
    group.finish();
}

fn compare_surface_raw_encoded_bytes(
    store: &SettlementStore,
    paths: &[SettlementPath],
    family: BatchCompareFamily,
    surface: BatchCompareSurface,
) -> usize {
    match surface {
        BatchCompareSurface::Single => match family {
            BatchCompareFamily::Inclusion | BatchCompareFamily::Deletion => store
                .settlement_proof_blob(&paths[0])
                .expect("single proof")
                .encode()
                .expect("single raw bytes")
                .len(),
            BatchCompareFamily::NonExistence => store
                .settlement_nonexistence_proof_blob(&paths[0], SettlementLeafFamily::Terminal)
                .expect("single absence")
                .encode()
                .expect("single absence raw bytes")
                .len(),
        },
        BatchCompareSurface::Vec => match family {
            BatchCompareFamily::Inclusion | BatchCompareFamily::Deletion => {
                let blobs = store
                    .settlement_proof_blobs(paths)
                    .expect("independent proof vec");
                sum_proof_bytes(&blobs)
            }
            BatchCompareFamily::NonExistence => {
                let blobs = paths
                    .iter()
                    .map(|path| {
                        store
                            .settlement_nonexistence_proof_blob(
                                path,
                                SettlementLeafFamily::Terminal,
                            )
                            .expect("independent absence")
                    })
                    .collect::<Vec<_>>();
                sum_proof_bytes(&blobs)
            }
        },
        BatchCompareSurface::Batch => match family {
            BatchCompareFamily::Inclusion => store
                .settlement_inclusion_batch_v1(paths)
                .expect("inclusion batch"),
            BatchCompareFamily::Deletion => store
                .settlement_deletion_batch_v1(paths)
                .expect("deletion batch"),
            BatchCompareFamily::NonExistence => store
                .settlement_nonexistence_batch_v1(paths, SettlementLeafFamily::Terminal)
                .expect("nonexistence batch"),
        }
        .encode()
        .expect("batch raw bytes")
        .len(),
    }
}

fn measure_compare_note_surface(
    store: &SettlementStore,
    paths: &[SettlementPath],
    family: BatchCompareFamily,
    surface: BatchCompareSurface,
    include_verify: bool,
) -> (usize, u128, Option<u128>) {
    match surface {
        BatchCompareSurface::Single => match family {
            BatchCompareFamily::Inclusion | BatchCompareFamily::Deletion => {
                let started = Instant::now();
                let blob = store
                    .settlement_proof_blob(&paths[0])
                    .expect("single proof");
                let prove_time_us = started.elapsed().as_micros();
                let bytes = blob.encode().expect("single bytes").len();
                let verify_time_us = if include_verify {
                    let verify_started = Instant::now();
                    store
                        .validate_settlement_proof_blob(&blob)
                        .expect("single verify");
                    Some(verify_started.elapsed().as_micros())
                } else {
                    None
                };
                (bytes, prove_time_us, verify_time_us)
            }
            BatchCompareFamily::NonExistence => {
                let started = Instant::now();
                let blob = store
                    .settlement_nonexistence_proof_blob(&paths[0], SettlementLeafFamily::Terminal)
                    .expect("single absence");
                let prove_time_us = started.elapsed().as_micros();
                let bytes = blob.encode().expect("single absence bytes").len();
                let verify_time_us = if include_verify {
                    let verify_started = Instant::now();
                    store
                        .validate_settlement_nonexistence_proof_blob(
                            &blob,
                            SettlementLeafFamily::Terminal,
                        )
                        .expect("single absence verify");
                    Some(verify_started.elapsed().as_micros())
                } else {
                    None
                };
                (bytes, prove_time_us, verify_time_us)
            }
        },
        BatchCompareSurface::Vec => match family {
            BatchCompareFamily::Inclusion | BatchCompareFamily::Deletion => {
                let started = Instant::now();
                let blobs = store
                    .settlement_proof_blobs(paths)
                    .expect("independent proof vec");
                let prove_time_us = started.elapsed().as_micros();
                let bytes = sum_proof_bytes(&blobs);
                (bytes, prove_time_us, None)
            }
            BatchCompareFamily::NonExistence => {
                let started = Instant::now();
                let blobs = paths
                    .iter()
                    .map(|path| {
                        store
                            .settlement_nonexistence_proof_blob(
                                path,
                                SettlementLeafFamily::Terminal,
                            )
                            .expect("independent absence")
                    })
                    .collect::<Vec<_>>();
                let prove_time_us = started.elapsed().as_micros();
                let bytes = sum_proof_bytes(&blobs);
                (bytes, prove_time_us, None)
            }
        },
        BatchCompareSurface::Batch => {
            let started = Instant::now();
            let batch = match family {
                BatchCompareFamily::Inclusion => store
                    .settlement_inclusion_batch_v1(paths)
                    .expect("inclusion batch"),
                BatchCompareFamily::Deletion => store
                    .settlement_deletion_batch_v1(paths)
                    .expect("deletion batch"),
                BatchCompareFamily::NonExistence => store
                    .settlement_nonexistence_batch_v1(paths, SettlementLeafFamily::Terminal)
                    .expect("nonexistence batch"),
            };
            let prove_time_us = started.elapsed().as_micros();
            let bytes = batch.encode().expect("batch bytes").len();
            (bytes, prove_time_us, None)
        }
    }
}

fn format_note_micros(value: Option<u128>) -> String {
    value
        .map(|micros| micros.to_string())
        .unwrap_or_else(|| "n/a".to_string())
}

fn default_note_command(scope: ProofNoteScope) -> &'static str {
    match scope {
        ProofNoteScope::Full => {
            "cargo bench -p z00z_storage --bench settlement_proofs -- --sample-size 10"
        }
        ProofNoteScope::BatchOnly => {
            "cargo bench -p z00z_storage --bench settlement_proofs -- hjmt_batch_ --quick --noplot --warm-up-time 0.01 --measurement-time 0.02"
        }
        ProofNoteScope::Skip => {
            "filtered settlement_proofs runs with proof_note_scope=skip do not emit note authority"
        }
    }
}

fn batch_filtered_lanes() -> String {
    match proof_note_filter().as_deref() {
        Some("hjmt_batch_") if note_runs_direct_matrix() => "`hjmt_batch_` selector with direct batch note rows and light live counts `{2,8,32}`; `128/1000/1024` stay in the full `settlement_proofs` benchmark and stress lanes while `hjmt_batch_verify` remains the runtime authority".to_string(),
        Some("hjmt_batch_") => "`hjmt_batch_proof_bytes`, `hjmt_batch_verify`".to_string(),
        Some(filter) => format!("selector `{filter}`"),
        None => "`hjmt_batch_proof_bytes`, `hjmt_batch_verify`".to_string(),
    }
}

fn write_batch_note_semantics(note: &mut String) {
    note.push_str(
        "- fixture_scope: `synthetic seeded path sets on an in-memory store; this matrix is not a live update workload.`\n",
    );
    note.push_str("- cache_semantics: `cold-only comparison; warm and reuse claims belong to proof_generate_cache_state and proof_verify_cache_state.`\n");
    note.push_str(
        "- serialized_bytes_semantics: `raw encode length; no compression step is applied in this path.`\n",
    );
    note.push_str(
        "- note_time_semantics: `prove_time_us and any note-level verify_time_us column are one-shot wall-clock samples from the note path; they are not Criterion statistics.`\n",
    );
    note.push_str(
        "- criterion_batch_proof_bytes_semantics: `Criterion timing for hjmt_batch_proof_bytes measures cold prove plus raw encode per iteration on the same synthetic fixture.`\n",
    );
}

fn write_proof_note_header(note: &mut String, scope: ProofNoteScope) {
    let command = proof_note_command().unwrap_or_else(|| default_note_command(scope).to_string());
    note.push_str("# Proof Size Summary\n\n");
    note.push_str(&format!("- command: `{command}`\n"));
    note.push_str(&format!("- note_scope: `{}`\n", scope.as_str()));
    note.push_str("- batch_verify_authority: `settlement_proofs_batch.md`\n");
    if let Some(filter) = proof_note_filter() {
        note.push_str(&format!("- lane_selector: `{filter}`\n"));
    }
}

fn append_batch_comparison_matrix(note: &mut String, scope: ProofNoteScope) {
    let include_verify = matches!(scope, ProofNoteScope::Full);
    note.push_str("\n## Batch Comparison Matrix\n\n");
    if matches!(scope, ProofNoteScope::BatchOnly) {
        write_batch_note_semantics(note);
        note.push_str(
            "- scope_rule: `batch-only scope records serialization and prove time only; live verify timing stays in hjmt_batch_verify.`\n",
        );
        note.push_str(&format!("- filtered_lanes: {}\n\n", batch_filtered_lanes()));
    }
    note.push_str("| proof_surface | path_count | path_shape | proof_family | cache_mode | persistence_mode | serialized_bytes | bytes_per_path | prove_time_us | verify_time_us | peak_memory_bytes |\n");
    note.push_str("| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |\n");
    for family in [
        BatchCompareFamily::Inclusion,
        BatchCompareFamily::Deletion,
        BatchCompareFamily::NonExistence,
    ] {
        let (store, paths) = compare_fixture(family, BatchCompareShape::Clustered, 1);
        let (serialized_bytes, prove_time_us, verify_time_us) = measure_compare_note_surface(
            &store,
            &paths,
            family,
            BatchCompareSurface::Single,
            include_verify,
        );
        note.push_str(&format!(
            "| {} | 1 | single | {} | cold | mem | {} | {:.2} | {} | {} | n/a |\n",
            BatchCompareSurface::Single.as_str(),
            family.as_str(),
            serialized_bytes,
            serialized_bytes as f64,
            prove_time_us,
            format_note_micros(verify_time_us),
        ));
    }

    for family in [
        BatchCompareFamily::Inclusion,
        BatchCompareFamily::Deletion,
        BatchCompareFamily::NonExistence,
    ] {
        for shape in [BatchCompareShape::Clustered, BatchCompareShape::Scattered] {
            for count in BATCH_COMPARE_FULL_COUNTS {
                let (store, paths) = compare_fixture(family, shape, count);
                for surface in [BatchCompareSurface::Vec, BatchCompareSurface::Batch] {
                    let (serialized_bytes, prove_time_us, verify_time_us) =
                        measure_compare_note_surface(&store, &paths, family, surface, false);
                    note.push_str(&format!(
                        "| {} | {} | {} | {} | cold | mem | {} | {:.2} | {} | {} | n/a |\n",
                        surface.as_str(),
                        count,
                        shape.as_str(),
                        family.as_str(),
                        serialized_bytes,
                        serialized_bytes as f64 / count as f64,
                        prove_time_us,
                        format_note_micros(verify_time_us),
                    ));
                }
            }
        }
    }
}

fn write_full_proof_note() {
    let mut note = String::new();
    write_proof_note_header(&mut note, ProofNoteScope::Full);

    let (store, asset_path) = inclusion_store();
    let asset = store
        .settlement_proof_blob(&asset_path)
        .expect("asset proof");
    let (store, right_path) = right_store();
    let right = store
        .settlement_proof_blob(&right_path)
        .expect("right proof");
    let (store, deleted_path) = deletion_store();
    let deletion = store
        .settlement_proof_blob(&deleted_path)
        .expect("deletion proof");
    let absent_path = settlement_corpus::asset_path(&asset_seed(0x93, 79, 90, 103_000));
    let absence = store
        .settlement_nonexistence_proof_blob(&absent_path, SettlementLeafFamily::Terminal)
        .expect("absence proof");
    let split = {
        let (_guard, split_store_ref, split_path) = split_store();
        let start = Instant::now();
        let split = split_store_ref
            .split_proof(&split_path)
            .expect("split proof");
        let elapsed = start.elapsed().as_micros();
        note.push_str(&format!("- split_time_us: `{elapsed}`\n"));
        split
    };
    let merge = {
        let (_guard, merge_store, left, right_path) = merge_store();
        let start = Instant::now();
        let merge = merge_store
            .merge_proof(&left, &right_path)
            .expect("merge proof");
        let elapsed = start.elapsed().as_micros();
        note.push_str(&format!("- merge_time_us: `{elapsed}`\n"));
        merge
    };
    let transition = {
        let (_guard, trans_store, _path) = split_store();
        let start = Instant::now();
        let transition = trans_store
            .policy_transition_proof(next_policy(&trans_store))
            .expect("transition proof");
        let elapsed = start.elapsed().as_micros();
        note.push_str(&format!("- policy_transition_time_us: `{elapsed}`\n"));
        transition
    };
    let fee_support_binding_bytes = {
        let _guard = HjmtEnvGuard::with_bits("2");
        let store = SettlementStore::new();
        let fee_right = right_seed(0x96, 5, 13, FixtureRightClass::ValidatorMandate);
        let path = settlement_corpus::right_path(&fee_right);
        let leaf = right_leaf(&fee_right);
        let ops = fee_put_ops(path, leaf).expect("fee put ops");
        let support = store.fee_support_ctx(&ops).expect("fee support ctx");
        let envelope = fee_envelope(97, support);
        envelope.support_ref.map_or(0, |_| 32)
    };
    let shared_parent_batch = {
        let (store, paths) = shared_parent_store();
        store
            .settlement_proof_blobs(&paths)
            .expect("shared parent batch proofs")
    };
    let mixed_batch = {
        let (store, paths, missing) = mixed_batch_store();
        let inclusion = store
            .settlement_proof_blobs(&paths)
            .expect("mixed batch inclusion proofs");
        let absence = store
            .settlement_nonexistence_proof_blob(&missing, SettlementLeafFamily::Terminal)
            .expect("mixed batch absence proof");
        (inclusion, absence)
    };

    let blobs = vec![
        asset.clone(),
        right.clone(),
        deletion.clone(),
        absence.clone(),
    ];
    note.push_str(&format!(
        "- inclusion_asset: `{}` bytes ({:?})\n",
        asset.encode().expect("asset bytes").len(),
        proof_family(&asset),
    ));
    note.push_str(&format!(
        "- inclusion_right: `{}` bytes ({:?})\n",
        right.encode().expect("right bytes").len(),
        proof_family(&right),
    ));
    note.push_str(&format!(
        "- deletion_right: `{}` bytes ({:?})\n",
        deletion.encode().expect("deletion bytes").len(),
        proof_family(&deletion),
    ));
    note.push_str(&format!(
        "- nonexistence_asset: `{}` bytes ({:?})\n",
        absence.encode().expect("absence bytes").len(),
        proof_family(&absence),
    ));
    note.push_str(&format!(
        "- split: `{}` bytes\n- merge: `{}` bytes\n- policy_transition: `{}` bytes\n",
        BincodeCodec.serialize(&split).expect("split bytes").len(),
        BincodeCodec.serialize(&merge).expect("merge bytes").len(),
        BincodeCodec
            .serialize(&transition)
            .expect("transition bytes")
            .len(),
    ));
    note.push_str(&format!(
        "- root_bind_bytes: `{}`\n- definition_proof_bytes: `{}`\n- serial_proof_bytes: `{}`\n- bucket_proof_bytes: `{}`\n- terminal_proof_bytes: `{}`\n- default_commitment_bytes: `{}`\n- default_child_commitment_bytes: `{}`\n",
        asset.root_bind().len(),
        asset.definition_proof().len(),
        asset.serial_proof().len(),
        asset.hjmt_bucket_proof().map_or(0, |bytes| bytes.len()),
        asset.terminal_proof().len(),
        asset.hjmt_default_commitment().map_or(0, |bytes| bytes.len()),
        asset.hjmt_default_child_commitment().map_or(0, |bytes| bytes.len()),
    ));
    note.push_str(&format!(
        "- definition_leaf_bytes: `{}`\n- serial_leaf_bytes: `{}`\n- bucket_leaf_bytes: `{}`\n- policy_transition_occupancy_bytes: `{}`\n- fee_support_binding_bytes: `{}`\n",
        asset.item().def_leaf().encode().len(),
        asset.item().ser_leaf().encode().len(),
        asset
            .hjmt_bucket_root_leaf()
            .map_or(0, |leaf| leaf.encode().len()),
        BincodeCodec
            .serialize(&transition.occupancy_evidence)
            .expect("occupancy bytes")
            .len(),
        fee_support_binding_bytes,
    ));
    note.push_str(&format!(
        "- shared_parent_batch_count: `{}`\n- shared_parent_batch_bytes: `{}`\n",
        shared_parent_batch.len(),
        sum_proof_bytes(&shared_parent_batch),
    ));
    note.push_str(&format!(
        "- mixed_batch_inclusion_count: `{}`\n- mixed_batch_inclusion_bytes: `{}`\n- mixed_batch_nonexistence_bytes: `{}`\n",
        mixed_batch.0.len(),
        sum_proof_bytes(&mixed_batch.0),
        mixed_batch.1.encode().expect("mixed absence bytes").len(),
    ));
    note.push_str(&format!(
        "- proof_bytes_total: `{}`\n",
        sum_proof_bytes(&blobs)
    ));
    write_batch_note_semantics(&mut note);
    append_batch_comparison_matrix(&mut note, ProofNoteScope::Full);
    write_note("settlement_proof_sizes.md", &note);
}

fn write_batch_only_proof_note() {
    let direct_matrix = note_runs_direct_matrix();
    let rows = if direct_matrix {
        collect_note_rows()
    } else {
        take_batch_only_note_rows()
    };
    let expected_rows = expected_batch_only_note_rows();
    let mut note = String::new();
    write_proof_note_header(&mut note, ProofNoteScope::BatchOnly);
    note.push_str(&format!("- matrix_row_count: `{}`\n", rows.len()));
    note.push_str(&format!(
        "- matrix_coverage: `{}`\n",
        if rows.len() == expected_rows {
            "complete"
        } else {
            "partial"
        }
    ));
    if direct_matrix {
        note.push_str(
            "- note_source: `one cold sample captured directly from batch-only note scope on the same canonical settlement_proofs.rs home`\n",
        );
        note.push_str("- live_scope_counts: `{2,8,32}`\n");
        note.push_str(
            "- benchmark_scope_rule: `larger counts 128, 1000, and 1024 stay in the full settlement_proofs benchmark and stress lanes only.`\n",
        );
        note.push_str(
            "- criterion_scope_rule: `the hjmt_batch_ selector keeps one canonical report path, but only the light live prove/bytes rows refresh here so repeated-loop runtime authority stays with hjmt_batch_verify.`\n",
        );
    } else {
        note.push_str(
            "- note_source: `one cold sample captured during hjmt_batch_proof_bytes lane setup from the same bench run`\n",
        );
    }
    if rows.len() != expected_rows {
        note.push_str("- matrix_scope: `partial filtered selector; omitted rows were not executed in this run`\n");
    }
    note.push_str("\n## Batch Comparison Matrix\n\n");
    write_batch_note_semantics(&mut note);
    note.push_str(
        "- scope_rule: `batch-only scope records serialization and prove time only; live verify timing stays in hjmt_batch_verify.`\n",
    );
    note.push_str(&format!("- filtered_lanes: {}\n\n", batch_filtered_lanes()));
    note.push_str("| proof_surface | path_count | path_shape | proof_family | cache_mode | persistence_mode | serialized_bytes | bytes_per_path | prove_time_us | verify_time_us | peak_memory_bytes |\n");
    note.push_str("| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |\n");
    for row in rows {
        note.push_str(&format!(
            "| {} | {} | {} | {} | cold | mem | {} | {:.2} | {} | n/a | n/a |\n",
            row.proof_surface,
            row.path_count,
            row.path_shape,
            row.proof_family,
            row.serialized_bytes,
            row.serialized_bytes as f64 / row.path_count as f64,
            row.prove_time_us,
        ));
    }
    write_note("settlement_proof_sizes.md", &note);
}

fn write_proof_note(scope: ProofNoteScope) {
    match scope {
        ProofNoteScope::Full => write_full_proof_note(),
        ProofNoteScope::BatchOnly => write_batch_only_proof_note(),
        ProofNoteScope::Skip => {}
    }
}

fn main() {
    let _sched = SchedEnv::new(4, 4096);
    let emit_side_outputs = should_emit_side_outputs();
    let note_scope = proof_note_scope();
    let direct_batch_note = note_runs_direct_matrix();
    if emit_side_outputs {
        write_meta(BenchMeta::new(
            "settlement_proofs",
            "cargo bench -p z00z_storage --bench settlement_proofs",
        ));
        if matches!(note_scope, ProofNoteScope::Full) || direct_batch_note {
            write_proof_note(note_scope);
        }
    }
    let mut crit = Criterion::default().configure_from_args();
    bench_generate(&mut crit);
    bench_verify(&mut crit);
    if !direct_batch_note {
        bench_batch_proof_bytes(&mut crit);
    }
    bench_batch_verify(&mut crit);
    crit.final_summary();
    if emit_side_outputs && matches!(note_scope, ProofNoteScope::BatchOnly) && !direct_batch_note {
        write_proof_note(note_scope);
    }
}
