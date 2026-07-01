use std::{
    collections::BTreeSet,
    path::{Path, PathBuf},
    time::Instant,
};

use z00z_storage::settlement::{
    check_hjmt_proof_family, chk_blob_settlement, BatchProofBlobV1, BatchProofFamilyTagV1,
    BucketPolicy, DefinitionId, FeeReplayKey, HjmtProofFamily, LeafFamilyTagV1, NodeDomainTagV1,
    NonExistenceOpeningV1, OpeningEntryV1, ProofBlob, RightLeaf, RootGeneration,
    RootGenerationTagV1, SerialId, SettlementLeafFamily, SettlementPath, SettlementStore,
    StoreItem, StoreOp, TerminalFamilyTagV1, TerminalId, HJMT_PROOF_ENVELOPE_VERSION,
};
use z00z_utils::{
    codec::json,
    codec::{BincodeCodec, Codec},
    io::{create_dir_all, path_exists, read_file, write_file},
};

use crate::{config::Stage13HjmtCfg, DesignStage, SimContext};

use super::{
    report::{
        self, Stage13CacheSchedulerReport, Stage13ExampleArtifact, Stage13ExamplesReport,
        Stage13ProofComparisonRow, Stage13ProofSizeEntry, Stage13ProofSizeReport,
        Stage13ReplayEntry, Stage13ReplayRootsReport, Stage13TamperCase, Stage13TamperReport,
        ATOMIC_VERDICT_ACCEPTED, ATOMIC_VERDICT_INDEPENDENT, PATH_SHAPE_CLUSTERED,
        PATH_SHAPE_SCATTERED, PATH_SHAPE_SINGLE, PROOF_SURFACE_BATCH, PROOF_SURFACE_SINGLE,
        PROOF_SURFACE_VEC, SHARD_CONTEXT_NONE, STAGE13_MODE, STAGE13_SCHEMA_VERSION,
        STAGE13_STATUS,
    },
    scan::{source_shape_note, FORBIDDEN_SOURCE_TERMS},
    storage::{
        asset_item_from_asset, ensure_expected_right_classes, fee_actor, fee_envelope,
        fixture_asset_item, leaf_family_name, missing_right_path_same_bucket, parse_path_hex,
        path_hex, pick_demo_right, proof_family_name, right_ctx, right_item_from_record,
        same_bucket_group, tampered_blob_present_path, terminal_hex, transfer_leaf, typed_error,
    },
    tamper,
};

const EXAMPLE_ASSET: &str = "E1_asset_inclusion";
const EXAMPLE_RIGHT: &str = "E2_right_inclusion";
const EXAMPLE_FEE: &str = "E3_fee_transition";
const EXAMPLE_DELETE: &str = "E4_right_deletion";
const EXAMPLE_ABSENT: &str = "E5_right_nonexistence";
const EXAMPLE_SPLIT: &str = "E6_adaptive_split";
const EXAMPLE_POLICY: &str = "E7_policy_transition";
const EXAMPLE_METRICS: &str = "E8_cache_scheduler";
const COMPARISON_COUNTS: [usize; 3] = [2, 8, 32];

#[cfg(feature = "test-params-fast")]
fn relax_stage13_sched(store: &SettlementStore) {
    // Stage 13 comparison fixtures probe many deterministic HJMT branches in one
    // evidence run. Keep the worker lane stable while lifting the queue ceiling
    // so evidence generation measures proof surfaces instead of scheduler limits.
    store.set_sched_limits_for_test(1, 1024);
}

#[cfg(not(feature = "test-params-fast"))]
fn relax_stage13_sched(_store: &SettlementStore) {}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ComparisonFamily {
    Inclusion,
    Deletion,
    NonExistence,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ComparisonShape {
    Clustered,
    Scattered,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ComparisonSurface {
    Single,
    Vec,
    Batch,
}

#[derive(Clone, Debug)]
struct ComparisonSeed {
    family: ComparisonFamily,
    shape: ComparisonShape,
    owner_example_id: &'static str,
    backend_mode: &'static str,
    paths: Vec<SettlementPath>,
}

#[derive(Debug, Clone)]
pub(crate) struct Stage13Paths {
    pub(crate) output_dir: PathBuf,
    pub(crate) store_dir: PathBuf,
    pub(crate) manifest_src: PathBuf,
    pub(crate) manifest_dst: PathBuf,
    pub(crate) examples_path: PathBuf,
    pub(crate) tamper_path: PathBuf,
    pub(crate) proof_size_path: PathBuf,
    pub(crate) cache_metrics_path: PathBuf,
    pub(crate) replay_roots_path: PathBuf,
    pub(crate) logger_path: PathBuf,
}

impl ComparisonFamily {
    const fn proof_family(self) -> HjmtProofFamily {
        match self {
            Self::Inclusion => HjmtProofFamily::Inclusion,
            Self::Deletion => HjmtProofFamily::Deletion,
            Self::NonExistence => HjmtProofFamily::NonExistence,
        }
    }

    const fn owner_example_id(self) -> &'static str {
        match self {
            Self::Inclusion => EXAMPLE_ASSET,
            Self::Deletion => EXAMPLE_DELETE,
            Self::NonExistence => EXAMPLE_ABSENT,
        }
    }

    const fn leaf_family(self) -> SettlementLeafFamily {
        match self {
            Self::Inclusion | Self::Deletion => SettlementLeafFamily::Terminal,
            Self::NonExistence => SettlementLeafFamily::Right,
        }
    }
}

impl ComparisonShape {
    const fn as_str(self) -> &'static str {
        match self {
            Self::Clustered => PATH_SHAPE_CLUSTERED,
            Self::Scattered => PATH_SHAPE_SCATTERED,
        }
    }
}

impl ComparisonSurface {
    const fn as_str(self) -> &'static str {
        match self {
            Self::Single => PROOF_SURFACE_SINGLE,
            Self::Vec => PROOF_SURFACE_VEC,
            Self::Batch => PROOF_SURFACE_BATCH,
        }
    }
}

fn terminal_from_seed(seed: u32, tag: u8) -> TerminalId {
    let mut bytes = [0u8; 32];
    bytes[..4].copy_from_slice(&seed.to_be_bytes());
    bytes[31] = tag.max(1);
    TerminalId::new(bytes)
}

fn same_bucket_paths(
    policy: BucketPolicy,
    definition_mark: u8,
    serial_id: u32,
    start_seed: u32,
    needed: usize,
    tag: u8,
) -> Result<Vec<SettlementPath>, String> {
    let definition_id = DefinitionId::new([definition_mark; 32]);
    let base = SettlementPath::new(
        definition_id,
        SerialId::new(serial_id),
        terminal_from_seed(start_seed.max(1), tag),
    );
    let target_bucket = policy.derive_bucket_id(base);
    let mut paths = Vec::with_capacity(needed);
    let mut seen = BTreeSet::new();
    for seed in start_seed.max(1)..=u16::MAX as u32 {
        let path = SettlementPath::new(
            definition_id,
            SerialId::new(serial_id),
            terminal_from_seed(seed, tag),
        );
        if policy.derive_bucket_id(path) != target_bucket || !seen.insert(path) {
            continue;
        }
        paths.push(path);
        if paths.len() == needed {
            paths.sort_unstable();
            return Ok(paths);
        }
    }
    Err(format!(
        "stage13 comparison fixture missing same-bucket paths for {} items",
        needed
    ))
}

fn scattered_paths(
    policy: BucketPolicy,
    definition_mark: u8,
    serial_id: u32,
    start_seed: u32,
    needed: usize,
    tag: u8,
) -> Result<Vec<SettlementPath>, String> {
    let definition_id = DefinitionId::new([definition_mark; 32]);
    let serial_id = SerialId::new(serial_id);
    let mut paths = Vec::with_capacity(needed);
    let mut buckets = BTreeSet::new();
    for seed in start_seed.max(1)..=u16::MAX as u32 {
        let path = SettlementPath::new(definition_id, serial_id, terminal_from_seed(seed, tag));
        if paths.contains(&path) {
            continue;
        }
        buckets.insert(policy.derive_bucket_id(path));
        paths.push(path);
        if paths.len() == needed && buckets.len() >= 2 {
            paths.sort_unstable();
            return Ok(paths);
        }
    }
    Err(format!(
        "stage13 comparison fixture missing scattered paths for {} items",
        needed
    ))
}

fn right_item_from_template(
    path: SettlementPath,
    template: RightLeaf,
    mark: u8,
) -> Result<StoreItem, String> {
    let mut leaf = template;
    leaf.terminal_id = path.terminal_id();
    leaf.holder_commitment = [mark; 32];
    leaf.beneficiary_commitment = [mark.wrapping_add(1); 32];
    leaf.payload_commitment = [mark.wrapping_add(2); 32];
    leaf.use_nonce = [mark.wrapping_add(3); 32];
    StoreItem::new(path, leaf).map_err(|e| format!("stage13 right fixture item failed: {e}"))
}

fn seed_asset_paths(store: &mut SettlementStore, paths: &[SettlementPath]) -> Result<(), String> {
    for path in paths {
        store
            .put_settlement_item(fixture_asset_item(*path)?)
            .map_err(|e| format!("stage13 comparison asset seed failed: {e}"))?;
    }
    Ok(())
}

fn seed_right_paths(
    store: &mut SettlementStore,
    paths: &[SettlementPath],
    template: RightLeaf,
    mark: u8,
) -> Result<(), String> {
    for (idx, path) in paths.iter().enumerate() {
        store
            .put_settlement_item(right_item_from_template(
                *path,
                template,
                mark.wrapping_add(u8::try_from(idx).unwrap_or(0)),
            )?)
            .map_err(|e| format!("stage13 comparison right seed failed: {e}"))?;
    }
    Ok(())
}

fn delete_paths(store: &mut SettlementStore, paths: &[SettlementPath]) -> Result<(), String> {
    store
        .apply_settlement_ops(paths.iter().copied().map(StoreOp::Delete).collect())
        .map_err(|e| format!("stage13 comparison delete failed: {e}"))?;
    Ok(())
}

fn derive_missing_paths(
    policy: BucketPolicy,
    present_paths: &[SettlementPath],
    require_same_bucket: bool,
    tag: u8,
) -> Result<Vec<SettlementPath>, String> {
    let occupied = present_paths.iter().copied().collect::<BTreeSet<_>>();
    let mut missing = Vec::with_capacity(present_paths.len());
    let mut seen = occupied.clone();
    for (idx, base) in present_paths.iter().copied().enumerate() {
        let target_bucket = policy.derive_bucket_id(base);
        let mut found = None;
        for seed in 1u32..=u16::MAX as u32 {
            let candidate = SettlementPath::new(
                base.definition_id,
                base.serial_id,
                terminal_from_seed(seed + u32::try_from(idx).unwrap_or(0), tag),
            );
            if candidate == base || seen.contains(&candidate) {
                continue;
            }
            let same_bucket = policy.derive_bucket_id(candidate) == target_bucket;
            if require_same_bucket && !same_bucket {
                continue;
            }
            if !require_same_bucket && same_bucket {
                continue;
            }
            found = Some(candidate);
            break;
        }
        let candidate = found.ok_or_else(|| {
            "stage13 comparison missing-path derivation ran out of deterministic candidates"
                .to_string()
        })?;
        seen.insert(candidate);
        missing.push(candidate);
    }
    missing.sort_unstable();
    Ok(missing)
}

fn same_bucket_companion_paths(
    policy: BucketPolicy,
    base_paths: &[SettlementPath],
    tag: u8,
    copies_per_base: usize,
) -> Result<Vec<SettlementPath>, String> {
    let mut occupied = base_paths.iter().copied().collect::<BTreeSet<_>>();
    let mut companions = Vec::with_capacity(base_paths.len().saturating_mul(copies_per_base));
    for (idx, base) in base_paths.iter().copied().enumerate() {
        let target_bucket = policy.derive_bucket_id(base);
        for copy in 0..copies_per_base {
            let mut found = None;
            for seed in 1u32..=u16::MAX as u32 {
                let candidate = SettlementPath::new(
                    base.definition_id,
                    base.serial_id,
                    terminal_from_seed(
                        seed + u32::try_from(idx).unwrap_or(0)
                            + u32::try_from(copy.saturating_mul(257)).unwrap_or(0),
                        tag,
                    ),
                );
                if occupied.contains(&candidate)
                    || policy.derive_bucket_id(candidate) != target_bucket
                {
                    continue;
                }
                found = Some(candidate);
                break;
            }
            let candidate = found.ok_or_else(|| {
                "stage13 comparison missing same-bucket companion path".to_string()
            })?;
            occupied.insert(candidate);
            companions.push(candidate);
        }
    }
    companions.sort_unstable();
    Ok(companions)
}

fn push_deletion_seed(
    store: &mut SettlementStore,
    seeds: &mut Vec<ComparisonSeed>,
    policy: BucketPolicy,
    shape: ComparisonShape,
    definition_mark: u8,
    serial_id: u32,
    start_seed: u32,
    count: usize,
    deleted_tag: u8,
    live_tag: u8,
    live_copies_per_base: usize,
) -> Result<(), String> {
    let deleted = match shape {
        ComparisonShape::Clustered => same_bucket_paths(
            policy,
            definition_mark,
            serial_id,
            start_seed,
            count,
            deleted_tag,
        )?,
        ComparisonShape::Scattered => scattered_paths(
            policy,
            definition_mark,
            serial_id,
            start_seed,
            count,
            deleted_tag,
        )?,
    };
    let live = same_bucket_companion_paths(policy, &deleted, live_tag, live_copies_per_base)?;
    seed_asset_paths(store, &deleted)?;
    seed_asset_paths(store, &live)?;
    delete_paths(store, &deleted)?;
    seeds.push(ComparisonSeed {
        family: ComparisonFamily::Deletion,
        shape,
        owner_example_id: ComparisonFamily::Deletion.owner_example_id(),
        backend_mode: "generalized",
        paths: deleted,
    });
    Ok(())
}

fn build_comparison_seeds(
    store: &mut SettlementStore,
    include_template: RightLeaf,
) -> Result<Vec<ComparisonSeed>, String> {
    let policy = store.bucket_policy();
    let mut seeds = Vec::new();

    let inclusion_clustered = same_bucket_paths(policy, 0x81, 181, 1, 32, 0x81)?;
    seed_asset_paths(store, &inclusion_clustered)?;
    seeds.push(ComparisonSeed {
        family: ComparisonFamily::Inclusion,
        shape: ComparisonShape::Clustered,
        owner_example_id: ComparisonFamily::Inclusion.owner_example_id(),
        backend_mode: "generalized",
        paths: inclusion_clustered,
    });

    let inclusion_scattered = scattered_paths(policy, 0x91, 281, 1_001, 32, 0x91)?;
    seed_asset_paths(store, &inclusion_scattered)?;
    seeds.push(ComparisonSeed {
        family: ComparisonFamily::Inclusion,
        shape: ComparisonShape::Scattered,
        owner_example_id: ComparisonFamily::Inclusion.owner_example_id(),
        backend_mode: "generalized",
        paths: inclusion_scattered,
    });

    for (count, serial_id, start_seed, deleted_tag, live_tag) in [
        (2usize, 381u32, 2_001u32, 0xA1u8, 0xA2u8),
        (8usize, 382u32, 2_101u32, 0xA3u8, 0xA4u8),
    ] {
        push_deletion_seed(
            store,
            &mut seeds,
            policy,
            ComparisonShape::Clustered,
            0xA1,
            serial_id,
            start_seed,
            count,
            deleted_tag,
            live_tag,
            1,
        )?;
    }

    for (count, serial_id, start_seed, deleted_tag, live_tag) in [
        (2usize, 481u32, 3_001u32, 0xB1u8, 0xB2u8),
        (8usize, 482u32, 3_101u32, 0xB3u8, 0xB4u8),
    ] {
        push_deletion_seed(
            store,
            &mut seeds,
            policy,
            ComparisonShape::Scattered,
            0xB1,
            serial_id,
            start_seed,
            count,
            deleted_tag,
            live_tag,
            2,
        )?;
    }

    let absence_clustered_present = same_bucket_paths(policy, 0xC1, 581, 4_001, 32, 0xC1)?;
    seed_right_paths(store, &absence_clustered_present, include_template, 0xC1)?;
    let absence_clustered = derive_missing_paths(policy, &absence_clustered_present, true, 0xD1)?;
    seeds.push(ComparisonSeed {
        family: ComparisonFamily::NonExistence,
        shape: ComparisonShape::Clustered,
        owner_example_id: ComparisonFamily::NonExistence.owner_example_id(),
        backend_mode: "generalized",
        paths: absence_clustered,
    });

    let absence_scattered_present = scattered_paths(policy, 0xD1, 681, 5_001, 32, 0xD1)?;
    seed_right_paths(store, &absence_scattered_present, include_template, 0xD1)?;
    let absence_scattered = derive_missing_paths(policy, &absence_scattered_present, false, 0xE1)?;
    seeds.push(ComparisonSeed {
        family: ComparisonFamily::NonExistence,
        shape: ComparisonShape::Scattered,
        owner_example_id: ComparisonFamily::NonExistence.owner_example_id(),
        backend_mode: "generalized",
        paths: absence_scattered,
    });

    Ok(seeds)
}

pub(crate) fn generate(
    ctx: &SimContext,
    stage: &DesignStage,
    cfg: &Stage13HjmtCfg,
    paths: &Stage13Paths,
) -> Result<(), String> {
    create_dir_all(&paths.output_dir).map_err(|e| e.to_string())?;
    create_dir_all(&paths.store_dir).map_err(|e| e.to_string())?;
    copy_manifest(&paths.manifest_src, &paths.manifest_dst)?;

    ensure_expected_right_classes(&ctx.genesis_rights, &cfg.expected_right_classes)?;

    let asset_item = asset_item_from_asset(
        ctx.assets
            .first()
            .ok_or_else(|| "stage13 requires at least one generated genesis asset".to_string())?,
    )?;
    let fee_record = pick_demo_right(&ctx.genesis_rights, &cfg.expected_right_classes)?;
    let delete_record = ctx
        .genesis_rights
        .iter()
        .find(|record| record.right_index != fee_record.right_index)
        .unwrap_or(fee_record);
    let include_record = ctx
        .genesis_rights
        .iter()
        .find(|record| {
            record.right_index != fee_record.right_index
                && record.right_index != delete_record.right_index
        })
        .unwrap_or(fee_record);

    let fee_path = right_item_from_record(fee_record)?.path();
    let delete_path = right_item_from_record(delete_record)?.path();
    let include_path = right_item_from_record(include_record)?.path();

    let mut store = SettlementStore::load(&paths.store_dir).map_err(|e| e.to_string())?;
    relax_stage13_sched(&store);
    store
        .put_settlement_item(asset_item.clone())
        .map_err(|e| format!("stage13 seed asset failed: {e}"))?;
    store
        .put_settlement_item(right_item_from_record(fee_record)?)
        .map_err(|e| format!("stage13 seed fee right failed: {e}"))?;
    store
        .put_settlement_item(right_item_from_record(delete_record)?)
        .map_err(|e| format!("stage13 seed delete right failed: {e}"))?;
    if include_path != fee_path && include_path != delete_path {
        store
            .put_settlement_item(right_item_from_record(include_record)?)
            .map_err(|e| format!("stage13 seed include right failed: {e}"))?;
    }

    let current_fee_item = store
        .get_settlement_item(&fee_path)
        .map_err(|e| format!("stage13 fee right load failed: {e}"))?
        .ok_or_else(|| "stage13 fee right missing after seed".to_string())?;
    let next_fee_leaf = transfer_leaf(
        current_fee_item
            .right_leaf()
            .map_err(|e| format!("stage13 fee right leaf decode failed: {e}"))?,
        0x44,
    );
    let fee_ops = vec![StoreOp::Put(Box::new(
        z00z_storage::settlement::StoreItem::new(fee_path, next_fee_leaf)
            .map_err(|e| format!("stage13 fee transfer item failed: {e}"))?,
    ))];
    let fee_support = store
        .fee_support_ctx(&fee_ops)
        .map_err(|e| format!("stage13 fee support ctx failed: {e}"))?;
    let fee_env = fee_envelope(0x55, fee_support);
    let fee_actor_ctx = fee_actor(0x55, 64);
    store
        .transfer_right_with_fee(
            fee_path,
            next_fee_leaf,
            right_ctx(&next_fee_leaf, 64),
            fee_env,
            fee_actor_ctx,
        )
        .map_err(|e| format!("stage13 fee-supported transition failed: {e}"))?;

    let delete_item = store
        .get_settlement_item(&delete_path)
        .map_err(|e| format!("stage13 delete right load failed: {e}"))?
        .ok_or_else(|| "stage13 delete right missing after seed".to_string())?;
    let delete_leaf = *delete_item
        .right_leaf()
        .map_err(|e| format!("stage13 delete right leaf decode failed: {e}"))?;
    let include_template = *right_item_from_record(include_record)?
        .right_leaf()
        .map_err(|e| format!("stage13 include right leaf decode failed: {e}"))?;
    let delete_support = store
        .fee_support_ctx(&[StoreOp::Delete(delete_path)])
        .map_err(|e| format!("stage13 delete fee support ctx failed: {e}"))?;
    let delete_env = fee_envelope(0x66, delete_support);
    let delete_actor_ctx = fee_actor(0x66, 64);
    store
        .consume_right_with_fee(
            delete_path,
            right_ctx(&delete_leaf, 64),
            delete_env,
            delete_actor_ctx,
        )
        .map_err(|e| format!("stage13 delete right failed: {e}"))?;

    let needed = usize::try_from(store.bucket_policy().min_bucket_count())
        .map_err(|e| format!("stage13 split threshold conversion failed: {e}"))?
        .saturating_add(1);
    let hot_paths = same_bucket_group(&mut store, needed)?;
    let comparison_seeds = build_comparison_seeds(&mut store, include_template)?;

    let final_root = store.settlement_root().map_err(|e| e.to_string())?;
    let final_root_hex = hex::encode(final_root.into_bytes());
    let root_generation = RootGeneration::SettlementV1.version();

    let artifact_names = artifact_names();
    let example1 = make_asset_inclusion(
        ctx,
        stage,
        &store,
        asset_item.path(),
        &artifact_names,
        final_root_hex.clone(),
        root_generation,
    )?;
    let example2 = make_right_inclusion(
        ctx,
        stage,
        &store,
        include_path,
        &artifact_names,
        final_root_hex.clone(),
        root_generation,
    )?;
    let example3 = make_fee_example(
        ctx,
        stage,
        &store,
        fee_path,
        fee_env,
        &artifact_names,
        final_root_hex.clone(),
        root_generation,
    )?;
    let example4 = make_deletion_example(
        ctx,
        stage,
        &store,
        delete_path,
        delete_item.leaf().clone(),
        &artifact_names,
        final_root_hex.clone(),
        root_generation,
    )?;
    let example5 = make_absence_example(
        ctx,
        stage,
        &store,
        include_path,
        &artifact_names,
        final_root_hex.clone(),
        root_generation,
    )?;
    let example6 = make_split_example(
        ctx,
        stage,
        &store,
        hot_paths[0],
        &artifact_names,
        final_root_hex.clone(),
        root_generation,
    )?;
    let next_policy = next_policy(store.bucket_policy())?;
    let example7 = make_policy_example(
        ctx,
        stage,
        &store,
        hot_paths[0],
        next_policy,
        &artifact_names,
        final_root_hex.clone(),
        root_generation,
    )?;

    // Warm the live cache through repeated proof work before reading metrics.
    let _ = store
        .settlement_proof_blobs(&hot_paths)
        .map_err(|e| format!("stage13 proof batch warmup failed: {e}"))?;
    let _ = store
        .settlement_proof_blobs(&hot_paths)
        .map_err(|e| format!("stage13 proof batch reuse failed: {e}"))?;
    let _ = store
        .settlement_nonexistence_proof_blob(
            &missing_right_path_same_bucket(store.bucket_policy(), include_path)?,
            SettlementLeafFamily::Right,
        )
        .map_err(|e| format!("stage13 nonexistence warmup failed: {e}"))?;
    let example8 = make_metrics_example(
        ctx,
        stage,
        &store,
        &artifact_names,
        final_root_hex.clone(),
        root_generation,
    )?;

    let examples = vec![
        example1.clone(),
        example2.clone(),
        example3.clone(),
        example4.clone(),
        example5.clone(),
        example6.clone(),
        example7.clone(),
        example8.clone(),
    ];
    let comparison_rows = build_comparison_rows(
        ctx,
        stage,
        &store,
        &comparison_seeds,
        final_root_hex.clone(),
        root_generation,
    )?;

    let examples_report = Stage13ExamplesReport {
        schema_version: STAGE13_SCHEMA_VERSION,
        scenario_id: ctx.config.scenario.id,
        stage: stage.stage,
        status: STAGE13_STATUS.to_string(),
        boundary_mode: STAGE13_MODE.to_string(),
        backend_modes: cfg.backend_modes.clone(),
        root_generation,
        artifact: report::report_artifact(
            "A1_examples_report",
            "mixed",
            "stage13_hjmt_examples_report",
        ),
        settlement_state_root_hex: final_root_hex.clone(),
        manifest_file: rel(&ctx.outputs_dir, &paths.manifest_dst),
        artifact_names: artifact_names.clone(),
        examples: examples.clone(),
        comparison_rows: comparison_rows.clone(),
    };
    report::write_json(&paths.examples_path, &examples_report)?;

    let proof_sizes = Stage13ProofSizeReport {
        schema_version: STAGE13_SCHEMA_VERSION,
        scenario_id: ctx.config.scenario.id,
        stage: stage.stage,
        status: STAGE13_STATUS.to_string(),
        root_generation,
        artifact: report::report_artifact(
            "A2_proof_size_report",
            "mixed",
            "stage13_hjmt_proof_size_report",
        ),
        entries: examples
            .iter()
            .filter_map(|example| {
                Some(Stage13ProofSizeEntry {
                    schema_version: STAGE13_SCHEMA_VERSION,
                    scenario_id: ctx.config.scenario.id,
                    stage: stage.stage,
                    example_id: example.example_id.clone(),
                    backend_mode: example.backend_mode.clone(),
                    api_surface: example.api_surface.clone(),
                    verifier_status: example.verifier_status.clone(),
                    root_generation,
                    typed_error: None,
                    proof_size_bytes: example.proof_size_bytes?,
                    verify_time_us: example.verify_time_us?,
                })
            })
            .collect(),
        comparison_rows: comparison_rows.clone(),
    };
    report::write_json(&paths.proof_size_path, &proof_sizes)?;

    let metrics_report =
        make_metrics_report(ctx, stage, &store, final_root_hex.clone(), root_generation)?;
    verify_metrics_report(&metrics_report)?;
    report::write_json(&paths.cache_metrics_path, &metrics_report)?;

    drop(store);
    let reloaded = SettlementStore::load(&paths.store_dir).map_err(|e| e.to_string())?;
    let replay_report = replay_report(
        ctx,
        stage,
        &reloaded,
        &examples,
        &metrics_report,
        root_generation,
        &final_root_hex,
    )?;
    report::write_json(&paths.replay_roots_path, &replay_report)?;

    let tamper_report = tamper_report(
        ctx,
        stage,
        &reloaded,
        &examples,
        &comparison_rows,
        &metrics_report,
        next_policy,
        include_path,
        root_generation,
    )?;
    report::write_json(&paths.tamper_path, &tamper_report)?;

    let log_rows = stage_log_rows(stage, paths, &final_root_hex, &artifact_names);
    write_file(&paths.logger_path, log_rows.join("\n").as_bytes()).map_err(|e| e.to_string())?;
    Ok(())
}

fn build_comparison_rows(
    ctx: &SimContext,
    stage: &DesignStage,
    store: &SettlementStore,
    seeds: &[ComparisonSeed],
    root_hex: String,
    root_generation: u8,
) -> Result<Vec<Stage13ProofComparisonRow>, String> {
    let mut rows = Vec::new();

    for family in [
        ComparisonFamily::Inclusion,
        ComparisonFamily::Deletion,
        ComparisonFamily::NonExistence,
    ] {
        let seed = seeds
            .iter()
            .find(|seed| {
                seed.family == family
                    && seed.shape == ComparisonShape::Clustered
                    && (family != ComparisonFamily::Deletion || seed.paths.len() == 2)
            })
            .ok_or_else(|| format!("stage13 missing clustered comparison seed for {family:?}"))?;
        rows.push(build_comparison_row(
            ctx,
            stage,
            store,
            family,
            ComparisonShape::Clustered,
            ComparisonSurface::Single,
            seed.owner_example_id,
            seed.backend_mode,
            &seed.paths[..1],
            root_hex.clone(),
            root_generation,
        )?);
    }

    for seed in seeds {
        if seed.family == ComparisonFamily::NonExistence && seed.shape == ComparisonShape::Scattered
        {
            continue;
        }
        for count in COMPARISON_COUNTS {
            if count == 32 && seed.family != ComparisonFamily::Inclusion {
                continue;
            }
            let paths = if seed.family == ComparisonFamily::Deletion {
                if seed.paths.len() != count {
                    continue;
                }
                seed.paths.as_slice()
            } else {
                &seed.paths[..count]
            };
            rows.push(build_comparison_row(
                ctx,
                stage,
                store,
                seed.family,
                seed.shape,
                ComparisonSurface::Vec,
                seed.owner_example_id,
                seed.backend_mode,
                paths,
                root_hex.clone(),
                root_generation,
            )?);
            rows.push(build_comparison_row(
                ctx,
                stage,
                store,
                seed.family,
                seed.shape,
                ComparisonSurface::Batch,
                seed.owner_example_id,
                seed.backend_mode,
                paths,
                root_hex.clone(),
                root_generation,
            )?);
        }
    }

    Ok(rows)
}

fn build_comparison_row(
    ctx: &SimContext,
    stage: &DesignStage,
    store: &SettlementStore,
    family: ComparisonFamily,
    shape: ComparisonShape,
    surface: ComparisonSurface,
    owner_example_id: &str,
    backend_mode: &str,
    paths: &[SettlementPath],
    root_hex: String,
    root_generation: u8,
) -> Result<Stage13ProofComparisonRow, String> {
    let row_id = comparison_row_id(surface, family, shape, paths.len());
    let (proof_size_bytes, verify_time_us, shard_context_mode) =
        measure_comparison_row(store, family, surface, paths)
            .map_err(|err| format!("stage13 comparison row {row_id} failed: {err}"))?;
    Ok(Stage13ProofComparisonRow {
        schema_version: STAGE13_SCHEMA_VERSION,
        scenario_id: ctx.config.scenario.id,
        stage: stage.stage,
        row_id,
        owner_example_id: owner_example_id.to_string(),
        backend_mode: backend_mode.to_string(),
        api_surface: comparison_api_surface(family, surface).to_string(),
        verifier_status: "verified".to_string(),
        typed_error: None,
        proof_surface: surface.as_str().to_string(),
        proof_family: proof_family_name(family.proof_family()).to_string(),
        leaf_family: leaf_family_name(family.leaf_family()).to_string(),
        path_count: u32::try_from(paths.len())
            .map_err(|_| "stage13 comparison path_count overflowed".to_string())?,
        path_shape: if matches!(surface, ComparisonSurface::Single) {
            PATH_SHAPE_SINGLE.to_string()
        } else {
            shape.as_str().to_string()
        },
        canonical_order: paths.windows(2).all(|pair| pair[0] <= pair[1]),
        atomic_verdict: if matches!(surface, ComparisonSurface::Batch) {
            ATOMIC_VERDICT_ACCEPTED.to_string()
        } else {
            ATOMIC_VERDICT_INDEPENDENT.to_string()
        },
        shard_context_mode: shard_context_mode.to_string(),
        root_generation,
        settlement_state_root_hex: root_hex,
        settlement_paths: paths.iter().map(|path| path_hex(*path)).collect(),
        proof_size_bytes,
        verify_time_us,
    })
}

fn comparison_row_id(
    surface: ComparisonSurface,
    family: ComparisonFamily,
    shape: ComparisonShape,
    path_count: usize,
) -> String {
    if matches!(surface, ComparisonSurface::Single) {
        format!(
            "{}_{}",
            surface.as_str(),
            proof_family_name(family.proof_family())
        )
    } else {
        format!(
            "{}_{}_{}_{}",
            surface.as_str(),
            shape.as_str(),
            proof_family_name(family.proof_family()),
            path_count
        )
    }
}

fn comparison_api_surface(family: ComparisonFamily, surface: ComparisonSurface) -> &'static str {
    match (family, surface) {
        (ComparisonFamily::Inclusion, ComparisonSurface::Single)
        | (ComparisonFamily::Deletion, ComparisonSurface::Single) => {
            "settlement_proof_blob + validate_settlement_proof_blob"
        }
        (ComparisonFamily::NonExistence, ComparisonSurface::Single) => {
            "settlement_nonexistence_proof_blob + validate_settlement_nonexistence_proof_blob"
        }
        (ComparisonFamily::Inclusion, ComparisonSurface::Vec)
        | (ComparisonFamily::Deletion, ComparisonSurface::Vec) => {
            "settlement_proof_blobs + per_path validation"
        }
        (ComparisonFamily::NonExistence, ComparisonSurface::Vec) => {
            "independent settlement_nonexistence_proof_blob + per_path validation"
        }
        (ComparisonFamily::Inclusion, ComparisonSurface::Batch) => {
            "settlement_inclusion_batch_v1 + BatchProofBlobV1::decode"
        }
        (ComparisonFamily::Deletion, ComparisonSurface::Batch) => {
            "settlement_deletion_batch_v1 + BatchProofBlobV1::decode"
        }
        (ComparisonFamily::NonExistence, ComparisonSurface::Batch) => {
            "settlement_nonexistence_batch_v1 + BatchProofBlobV1::decode"
        }
    }
}

fn measure_comparison_row(
    store: &SettlementStore,
    family: ComparisonFamily,
    surface: ComparisonSurface,
    paths: &[SettlementPath],
) -> Result<(usize, u64, &'static str), String> {
    match surface {
        ComparisonSurface::Single => {
            let (blob, verify_time_us) = timed(|| build_single_blob(store, family, paths[0]))?;
            Ok((
                blob.encode().map_err(|e| e.to_string())?.len(),
                verify_time_us,
                SHARD_CONTEXT_NONE,
            ))
        }
        ComparisonSurface::Vec => {
            let (blobs, verify_time_us) = timed(|| build_independent_blobs(store, family, paths))?;
            Ok((sum_blob_bytes(&blobs)?, verify_time_us, SHARD_CONTEXT_NONE))
        }
        ComparisonSurface::Batch => {
            let (batch, verify_time_us) = timed(|| build_batch_blob(store, family, paths))?;
            let bytes = batch.encode().map_err(|e| e.to_string())?;
            let shard_context_mode = if batch
                .path_table
                .iter()
                .any(|entry| entry.shard_id.is_some() || entry.routing_generation.is_some())
            {
                "present"
            } else {
                SHARD_CONTEXT_NONE
            };
            Ok((bytes.len(), verify_time_us, shard_context_mode))
        }
    }
}

fn sum_blob_bytes(blobs: &[ProofBlob]) -> Result<usize, String> {
    blobs.iter().try_fold(0usize, |acc, blob| {
        blob.encode()
            .map(|bytes| acc + bytes.len())
            .map_err(|e| e.to_string())
    })
}

fn build_single_blob(
    store: &SettlementStore,
    family: ComparisonFamily,
    path: SettlementPath,
) -> Result<ProofBlob, String> {
    match family {
        ComparisonFamily::Inclusion | ComparisonFamily::Deletion => {
            let blob = store
                .settlement_proof_blob(&path)
                .map_err(|e| e.to_string())?;
            store
                .validate_settlement_proof_blob(&blob)
                .map_err(|e| e.to_string())?;
            Ok(blob)
        }
        ComparisonFamily::NonExistence => {
            let blob = store
                .settlement_nonexistence_proof_blob(&path, SettlementLeafFamily::Right)
                .map_err(|e| e.to_string())?;
            store
                .validate_settlement_nonexistence_proof_blob(&blob, SettlementLeafFamily::Right)
                .map_err(|e| e.to_string())?;
            Ok(blob)
        }
    }
}

fn build_independent_blobs(
    store: &SettlementStore,
    family: ComparisonFamily,
    paths: &[SettlementPath],
) -> Result<Vec<ProofBlob>, String> {
    match family {
        ComparisonFamily::Inclusion | ComparisonFamily::Deletion => {
            let blobs = store
                .settlement_proof_blobs(paths)
                .map_err(|e| e.to_string())?;
            for blob in &blobs {
                store
                    .validate_settlement_proof_blob(blob)
                    .map_err(|e| e.to_string())?;
            }
            Ok(blobs)
        }
        ComparisonFamily::NonExistence => {
            let mut blobs = Vec::with_capacity(paths.len());
            for path in paths {
                let blob = store
                    .settlement_nonexistence_proof_blob(path, SettlementLeafFamily::Right)
                    .map_err(|e| e.to_string())?;
                store
                    .validate_settlement_nonexistence_proof_blob(&blob, SettlementLeafFamily::Right)
                    .map_err(|e| e.to_string())?;
                blobs.push(blob);
            }
            Ok(blobs)
        }
    }
}

fn build_batch_blob(
    store: &SettlementStore,
    family: ComparisonFamily,
    paths: &[SettlementPath],
) -> Result<BatchProofBlobV1, String> {
    let batch = match family {
        ComparisonFamily::Inclusion => store
            .settlement_inclusion_batch_v1(paths)
            .map_err(|e| e.to_string())?,
        ComparisonFamily::Deletion => store
            .settlement_deletion_batch_v1(paths)
            .map_err(|e| e.to_string())?,
        ComparisonFamily::NonExistence => store
            .settlement_nonexistence_batch_v1(paths, SettlementLeafFamily::Right)
            .map_err(|e| e.to_string())?,
    };
    let bytes = batch.encode().map_err(|e| e.to_string())?;
    let decoded = BatchProofBlobV1::decode(&bytes).map_err(|e| e.to_string())?;
    if decoded != batch {
        return Err("stage13 batch comparison decode drifted from encoded bytes".to_string());
    }
    Ok(batch)
}

fn make_asset_inclusion(
    ctx: &SimContext,
    stage: &DesignStage,
    store: &SettlementStore,
    path: z00z_storage::settlement::SettlementPath,
    artifact_names: &[String],
    root_hex: String,
    root_generation: u8,
) -> Result<Stage13ExampleArtifact, String> {
    let (blob, verify_time_us) = timed(|| {
        let blob = store
            .settlement_proof_blob(&path)
            .map_err(|e| e.to_string())?;
        store
            .validate_settlement_proof_blob(&blob)
            .map_err(|e| e.to_string())?;
        Ok(blob)
    })?;
    let family = blob
        .hjmt_proof_family()
        .ok_or_else(|| "stage13 asset inclusion missing proof family".to_string())?;
    check_hjmt_proof_family(family).map_err(|e| e.to_string())?;
    Ok(Stage13ExampleArtifact {
        schema_version: STAGE13_SCHEMA_VERSION,
        scenario_id: ctx.config.scenario.id,
        stage: stage.stage,
        example_id: EXAMPLE_ASSET.to_string(),
        backend_mode: "generalized".to_string(),
        api_surface: "put_settlement_item + settlement_proof_blob + validate_settlement_proof_blob"
            .to_string(),
        verifier_status: "verified".to_string(),
        root_generation,
        settlement_state_root_hex: root_hex,
        prior_state_root_hex: None,
        next_state_root_hex: None,
        proof_envelope_version: blob
            .hjmt_envelope_version()
            .or(Some(HJMT_PROOF_ENVELOPE_VERSION)),
        proof_family: proof_family_name(family).to_string(),
        leaf_family: leaf_family_name(SettlementLeafFamily::Terminal).to_string(),
        settlement_path: path_hex(path),
        terminal_id: terminal_hex(path),
        bucket_epoch: blob.hjmt_journal_checkpoint(),
        bucket_policy_id: blob
            .hjmt_bucket_policy()
            .map(|policy| hex::encode(policy.bucket_policy_id())),
        fee_envelope_id: None,
        fee_domain: None,
        transition_binding: None,
        payer_commitment: None,
        sponsor_commitment: None,
        expiry: None,
        replay_status: None,
        artifact_names: artifact_names.to_vec(),
        proof_size_bytes: Some(blob.encode().map_err(|e| e.to_string())?.len()),
        verify_time_us: Some(verify_time_us),
        typed_error: None,
        present_key_rejection: None,
        proof_is_ownership: Some(true),
    })
}

fn make_right_inclusion(
    ctx: &SimContext,
    stage: &DesignStage,
    store: &SettlementStore,
    path: z00z_storage::settlement::SettlementPath,
    artifact_names: &[String],
    root_hex: String,
    root_generation: u8,
) -> Result<Stage13ExampleArtifact, String> {
    let (blob, verify_time_us) = timed(|| {
        let blob = store
            .settlement_proof_blob(&path)
            .map_err(|e| e.to_string())?;
        store
            .validate_settlement_proof_blob(&blob)
            .map_err(|e| e.to_string())?;
        Ok(blob)
    })?;
    let family = blob
        .hjmt_proof_family()
        .ok_or_else(|| "stage13 right inclusion missing proof family".to_string())?;
    Ok(Stage13ExampleArtifact {
        schema_version: STAGE13_SCHEMA_VERSION,
        scenario_id: ctx.config.scenario.id,
        stage: stage.stage,
        example_id: EXAMPLE_RIGHT.to_string(),
        backend_mode: "generalized".to_string(),
        api_surface: "put_settlement_item + settlement_proof_blob + validate_settlement_proof_blob"
            .to_string(),
        verifier_status: "verified".to_string(),
        root_generation,
        settlement_state_root_hex: root_hex,
        prior_state_root_hex: None,
        next_state_root_hex: None,
        proof_envelope_version: blob
            .hjmt_envelope_version()
            .or(Some(HJMT_PROOF_ENVELOPE_VERSION)),
        proof_family: proof_family_name(family).to_string(),
        leaf_family: leaf_family_name(SettlementLeafFamily::Right).to_string(),
        settlement_path: path_hex(path),
        terminal_id: terminal_hex(path),
        bucket_epoch: blob.hjmt_journal_checkpoint(),
        bucket_policy_id: blob
            .hjmt_bucket_policy()
            .map(|policy| hex::encode(policy.bucket_policy_id())),
        fee_envelope_id: None,
        fee_domain: None,
        transition_binding: None,
        payer_commitment: None,
        sponsor_commitment: None,
        expiry: None,
        replay_status: None,
        artifact_names: artifact_names.to_vec(),
        proof_size_bytes: Some(blob.encode().map_err(|e| e.to_string())?.len()),
        verify_time_us: Some(verify_time_us),
        typed_error: None,
        present_key_rejection: None,
        proof_is_ownership: Some(true),
    })
}

fn make_fee_example(
    ctx: &SimContext,
    stage: &DesignStage,
    store: &SettlementStore,
    path: z00z_storage::settlement::SettlementPath,
    envelope: z00z_storage::settlement::FeeEnvelope,
    artifact_names: &[String],
    root_hex: String,
    root_generation: u8,
) -> Result<Stage13ExampleArtifact, String> {
    let item = store
        .get_settlement_item(&path)
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "stage13 fee example lost live right".to_string())?;
    let replay = store
        .fee_replay_rec(&envelope.replay_id())
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "stage13 fee replay row missing".to_string())?;
    let (blob, verify_time_us) = timed(|| {
        let blob = store
            .settlement_proof_blob(&path)
            .map_err(|e| e.to_string())?;
        store
            .validate_settlement_proof_blob(&blob)
            .map_err(|e| e.to_string())?;
        store
            .claim_source_contract_for_item(&item)
            .map_err(|e| e.to_string())?;
        Ok(blob)
    })?;
    let family = blob
        .hjmt_proof_family()
        .ok_or_else(|| "stage13 fee inclusion missing proof family".to_string())?;
    check_hjmt_proof_family(family).map_err(|e| e.to_string())?;
    Ok(Stage13ExampleArtifact {
        schema_version: STAGE13_SCHEMA_VERSION,
        scenario_id: ctx.config.scenario.id,
        stage: stage.stage,
        example_id: EXAMPLE_FEE.to_string(),
        backend_mode: "generalized".to_string(),
        api_surface: "transfer_right_with_fee + settlement_proof_blob + claim_source_contract_for_item + fee_replay_rec".to_string(),
        verifier_status: "verified".to_string(),
        root_generation,
        settlement_state_root_hex: root_hex,
        prior_state_root_hex: None,
        next_state_root_hex: None,
        proof_envelope_version: blob.hjmt_envelope_version().or(Some(HJMT_PROOF_ENVELOPE_VERSION)),
        proof_family: proof_family_name(family).to_string(),
        leaf_family: leaf_family_name(SettlementLeafFamily::Right).to_string(),
        settlement_path: path_hex(path),
        terminal_id: terminal_hex(path),
        bucket_epoch: blob.hjmt_journal_checkpoint(),
        bucket_policy_id: blob
            .hjmt_bucket_policy()
            .map(|policy| hex::encode(policy.bucket_policy_id())),
        fee_envelope_id: Some(hex::encode(envelope.replay_id().into_bytes())),
        fee_domain: Some(hex::encode(envelope.domain_id)),
        transition_binding: Some(hex::encode(envelope.transition_id)),
        payer_commitment: Some(hex::encode(replay.payer_commitment)),
        sponsor_commitment: Some(hex::encode(replay.sponsor_commitment)),
        expiry: Some(envelope.expires_at),
        replay_status: Some("accepted".to_string()),
        artifact_names: artifact_names.to_vec(),
        proof_size_bytes: Some(blob.encode().map_err(|e| e.to_string())?.len()),
        verify_time_us: Some(verify_time_us),
        typed_error: None,
        present_key_rejection: None,
        proof_is_ownership: Some(false),
    })
}

fn make_deletion_example(
    ctx: &SimContext,
    stage: &DesignStage,
    store: &SettlementStore,
    path: z00z_storage::settlement::SettlementPath,
    deleted_leaf: z00z_storage::settlement::SettlementLeaf,
    artifact_names: &[String],
    root_hex: String,
    root_generation: u8,
) -> Result<Stage13ExampleArtifact, String> {
    let (blob, verify_time_us) = timed(|| {
        let blob = store
            .settlement_proof_blob(&path)
            .map_err(|e| e.to_string())?;
        store
            .validate_settlement_proof_blob(&blob)
            .map_err(|e| e.to_string())?;
        chk_blob_settlement(
            &blob.encode().map_err(|e| e.to_string())?,
            store.settlement_root().map_err(|e| e.to_string())?,
            &path,
            blob.item().def_leaf(),
            blob.item().ser_leaf(),
            deleted_leaf.clone(),
        )
        .map_err(|e| e.to_string())?;
        Ok(blob)
    })?;
    Ok(Stage13ExampleArtifact {
        schema_version: STAGE13_SCHEMA_VERSION,
        scenario_id: ctx.config.scenario.id,
        stage: stage.stage,
        example_id: EXAMPLE_DELETE.to_string(),
        backend_mode: "generalized".to_string(),
        api_surface:
            "consume_right_with_fee + settlement_proof_blob + validate_settlement_proof_blob"
                .to_string(),
        verifier_status: "verified".to_string(),
        root_generation,
        settlement_state_root_hex: root_hex.clone(),
        prior_state_root_hex: None,
        next_state_root_hex: Some(root_hex),
        proof_envelope_version: blob
            .hjmt_envelope_version()
            .or(Some(HJMT_PROOF_ENVELOPE_VERSION)),
        proof_family: "deletion".to_string(),
        leaf_family: leaf_family_name(SettlementLeafFamily::Right).to_string(),
        settlement_path: path_hex(path),
        terminal_id: terminal_hex(path),
        bucket_epoch: blob.hjmt_journal_checkpoint(),
        bucket_policy_id: blob
            .hjmt_bucket_policy()
            .map(|policy| hex::encode(policy.bucket_policy_id())),
        fee_envelope_id: None,
        fee_domain: None,
        transition_binding: None,
        payer_commitment: None,
        sponsor_commitment: None,
        expiry: None,
        replay_status: None,
        artifact_names: artifact_names.to_vec(),
        proof_size_bytes: Some(blob.encode().map_err(|e| e.to_string())?.len()),
        verify_time_us: Some(verify_time_us),
        typed_error: None,
        present_key_rejection: None,
        proof_is_ownership: Some(true),
    })
}

fn make_absence_example(
    ctx: &SimContext,
    stage: &DesignStage,
    store: &SettlementStore,
    present_path: z00z_storage::settlement::SettlementPath,
    artifact_names: &[String],
    root_hex: String,
    root_generation: u8,
) -> Result<Stage13ExampleArtifact, String> {
    let missing_path = missing_right_path_same_bucket(store.bucket_policy(), present_path)?;
    let (blob, verify_time_us) = timed(|| {
        let blob = store
            .settlement_nonexistence_proof_blob(&missing_path, SettlementLeafFamily::Right)
            .map_err(|e| e.to_string())?;
        store
            .validate_settlement_nonexistence_proof_blob(&blob, SettlementLeafFamily::Right)
            .map_err(|e| e.to_string())?;
        Ok(blob)
    })?;
    let tampered = tampered_blob_present_path(&blob, present_path, SettlementLeafFamily::Right)?;
    let rejection = match store
        .validate_settlement_nonexistence_proof_blob(&tampered, SettlementLeafFamily::Right)
    {
        Ok(()) => {
            return Err(
                "stage13 present-key tamper unexpectedly passed non-existence validation"
                    .to_string(),
            )
        }
        Err(err) => err,
    };
    Ok(Stage13ExampleArtifact {
        schema_version: STAGE13_SCHEMA_VERSION,
        scenario_id: ctx.config.scenario.id,
        stage: stage.stage,
        example_id: EXAMPLE_ABSENT.to_string(),
        backend_mode: "generalized".to_string(),
        api_surface:
            "settlement_nonexistence_proof_blob + validate_settlement_nonexistence_proof_blob"
                .to_string(),
        verifier_status: "verified".to_string(),
        root_generation,
        settlement_state_root_hex: root_hex,
        prior_state_root_hex: None,
        next_state_root_hex: None,
        proof_envelope_version: blob
            .hjmt_envelope_version()
            .or(Some(HJMT_PROOF_ENVELOPE_VERSION)),
        proof_family: "nonexistence".to_string(),
        leaf_family: leaf_family_name(SettlementLeafFamily::Right).to_string(),
        settlement_path: path_hex(missing_path),
        terminal_id: terminal_hex(missing_path),
        bucket_epoch: blob.hjmt_journal_checkpoint(),
        bucket_policy_id: blob
            .hjmt_bucket_policy()
            .map(|policy| hex::encode(policy.bucket_policy_id())),
        fee_envelope_id: None,
        fee_domain: None,
        transition_binding: None,
        payer_commitment: None,
        sponsor_commitment: None,
        expiry: None,
        replay_status: None,
        artifact_names: artifact_names.to_vec(),
        proof_size_bytes: Some(blob.encode().map_err(|e| e.to_string())?.len()),
        verify_time_us: Some(verify_time_us),
        typed_error: None,
        present_key_rejection: Some(report::redact_error_class(&rejection)),
        proof_is_ownership: Some(false),
    })
}

fn make_split_example(
    ctx: &SimContext,
    stage: &DesignStage,
    store: &SettlementStore,
    path: z00z_storage::settlement::SettlementPath,
    artifact_names: &[String],
    root_hex: String,
    root_generation: u8,
) -> Result<Stage13ExampleArtifact, String> {
    let (proof, verify_time_us) = timed(|| {
        let proof = store.split_proof(&path).map_err(|e| e.to_string())?;
        store
            .validate_split_proof(&proof)
            .map_err(|e| e.to_string())?;
        Ok(proof)
    })?;
    Ok(Stage13ExampleArtifact {
        schema_version: STAGE13_SCHEMA_VERSION,
        scenario_id: ctx.config.scenario.id,
        stage: stage.stage,
        example_id: EXAMPLE_SPLIT.to_string(),
        backend_mode: "adaptive".to_string(),
        api_surface: "split_proof + validate_split_proof".to_string(),
        verifier_status: "verified".to_string(),
        root_generation,
        settlement_state_root_hex: root_hex,
        prior_state_root_hex: Some(hex::encode(proof.prior_root.into_bytes())),
        next_state_root_hex: Some(hex::encode(proof.next_root.into_bytes())),
        proof_envelope_version: None,
        proof_family: "split".to_string(),
        leaf_family: leaf_family_name(SettlementLeafFamily::Terminal).to_string(),
        settlement_path: path_hex(path),
        terminal_id: terminal_hex(path),
        bucket_epoch: Some(proof.prior_epoch.get()),
        bucket_policy_id: Some(hex::encode(proof.bucket_policy_id)),
        fee_envelope_id: None,
        fee_domain: None,
        transition_binding: None,
        payer_commitment: None,
        sponsor_commitment: None,
        expiry: None,
        replay_status: None,
        artifact_names: artifact_names.to_vec(),
        proof_size_bytes: Some(
            BincodeCodec
                .serialize(&proof)
                .map_err(|e| e.to_string())?
                .len(),
        ),
        verify_time_us: Some(verify_time_us),
        typed_error: None,
        present_key_rejection: None,
        proof_is_ownership: Some(false),
    })
}

fn make_policy_example(
    ctx: &SimContext,
    stage: &DesignStage,
    store: &SettlementStore,
    path: z00z_storage::settlement::SettlementPath,
    next_policy: BucketPolicy,
    artifact_names: &[String],
    root_hex: String,
    root_generation: u8,
) -> Result<Stage13ExampleArtifact, String> {
    let (proof, verify_time_us) = timed(|| {
        let proof = store
            .policy_transition_proof(next_policy)
            .map_err(|e| e.to_string())?;
        store
            .validate_policy_transition_proof(&proof, next_policy)
            .map_err(|e| e.to_string())?;
        Ok(proof)
    })?;
    Ok(Stage13ExampleArtifact {
        schema_version: STAGE13_SCHEMA_VERSION,
        scenario_id: ctx.config.scenario.id,
        stage: stage.stage,
        example_id: EXAMPLE_POLICY.to_string(),
        backend_mode: "adaptive".to_string(),
        api_surface: "policy_transition_proof + validate_policy_transition_proof".to_string(),
        verifier_status: "verified".to_string(),
        root_generation,
        settlement_state_root_hex: root_hex,
        prior_state_root_hex: Some(hex::encode(proof.prior_root.into_bytes())),
        next_state_root_hex: Some(hex::encode(proof.next_root.into_bytes())),
        proof_envelope_version: None,
        proof_family: "policy_transition".to_string(),
        leaf_family: leaf_family_name(SettlementLeafFamily::Terminal).to_string(),
        settlement_path: path_hex(path),
        terminal_id: terminal_hex(path),
        bucket_epoch: Some(proof.prior_epoch.get()),
        bucket_policy_id: Some(hex::encode(proof.next_policy_id)),
        fee_envelope_id: None,
        fee_domain: None,
        transition_binding: Some(hex::encode(proof.next_policy_id)),
        payer_commitment: None,
        sponsor_commitment: None,
        expiry: None,
        replay_status: None,
        artifact_names: artifact_names.to_vec(),
        proof_size_bytes: Some(
            BincodeCodec
                .serialize(&proof)
                .map_err(|e| e.to_string())?
                .len(),
        ),
        verify_time_us: Some(verify_time_us),
        typed_error: None,
        present_key_rejection: None,
        proof_is_ownership: Some(false),
    })
}

fn make_metrics_example(
    ctx: &SimContext,
    stage: &DesignStage,
    store: &SettlementStore,
    artifact_names: &[String],
    root_hex: String,
    root_generation: u8,
) -> Result<Stage13ExampleArtifact, String> {
    let metrics = make_metrics_report(ctx, stage, store, root_hex.clone(), root_generation)?;
    verify_metrics_report(&metrics)?;
    Ok(Stage13ExampleArtifact {
        schema_version: STAGE13_SCHEMA_VERSION,
        scenario_id: ctx.config.scenario.id,
        stage: stage.stage,
        example_id: EXAMPLE_METRICS.to_string(),
        backend_mode: "adaptive".to_string(),
        api_surface: "settlement_proof_blobs + forest_cache_metrics + forest_scheduler_metrics"
            .to_string(),
        verifier_status: "verified".to_string(),
        root_generation,
        settlement_state_root_hex: root_hex,
        prior_state_root_hex: None,
        next_state_root_hex: None,
        proof_envelope_version: None,
        proof_family: "metrics".to_string(),
        leaf_family: "none".to_string(),
        settlement_path: "none".to_string(),
        terminal_id: "none".to_string(),
        bucket_epoch: None,
        bucket_policy_id: None,
        fee_envelope_id: None,
        fee_domain: None,
        transition_binding: None,
        payer_commitment: None,
        sponsor_commitment: None,
        expiry: None,
        replay_status: None,
        artifact_names: artifact_names.to_vec(),
        proof_size_bytes: None,
        verify_time_us: None,
        typed_error: None,
        present_key_rejection: None,
        proof_is_ownership: Some(false),
    })
}

fn make_metrics_report(
    ctx: &SimContext,
    stage: &DesignStage,
    store: &SettlementStore,
    root_hex: String,
    root_generation: u8,
) -> Result<Stage13CacheSchedulerReport, String> {
    let cache = store.forest_cache_metrics();
    let scheduler = store.forest_scheduler_metrics();
    let cache_hits = cache.subtree_root.hits
        + cache.parent_leaf.hits
        + cache.terminal_leaf.hits
        + cache.bucket_derivation.hits
        + cache.proof_segment.hits
        + cache.nonexistence.hits
        + cache.policy_proof.hits
        + cache.journal_digest.hits
        + cache.path_index.hits;
    let cache_misses = cache.subtree_root.misses
        + cache.parent_leaf.misses
        + cache.terminal_leaf.misses
        + cache.proof_segment.misses
        + cache.bucket_derivation.misses
        + cache.nonexistence.misses
        + cache.policy_proof.misses
        + cache.journal_digest.misses
        + cache.path_index.misses;
    let root_denom = (cache.subtree_root.hits + cache.subtree_root.misses).max(1) as f64;
    let proof_denom = (cache.proof_segment.hits + cache.proof_segment.misses).max(1) as f64;
    let invalidation_count = cache.subtree_root.invalidations
        + cache.parent_leaf.invalidations
        + cache.terminal_leaf.invalidations
        + cache.bucket_derivation.invalidations
        + cache.proof_segment.invalidations
        + cache.nonexistence.invalidations
        + cache.policy_proof.invalidations
        + cache.journal_digest.invalidations
        + cache.path_index.invalidations;

    Ok(Stage13CacheSchedulerReport {
        schema_version: STAGE13_SCHEMA_VERSION,
        scenario_id: ctx.config.scenario.id,
        stage: stage.stage,
        example_id: EXAMPLE_METRICS.to_string(),
        backend_mode: "adaptive".to_string(),
        api_surface: "settlement_proof_blobs + forest_cache_metrics + forest_scheduler_metrics"
            .to_string(),
        verifier_status: "verified".to_string(),
        root_generation,
        typed_error: None,
        settlement_state_root_hex: root_hex,
        cache_hit_count: cache_hits,
        cache_miss_count: cache_misses,
        invalidation_count,
        root_reuse_ratio: cache.subtree_root.hits as f64 / root_denom,
        proof_segment_reuse_ratio: cache.proof_segment.hits as f64 / proof_denom,
        scheduler_queue_depth: scheduler.max_queued,
        scheduler_backpressure_count: scheduler.reject_count,
        deterministic_parent_ordering: scheduler.last_ordered,
        cache_metrics: cache,
        scheduler_metrics: scheduler,
    })
}

pub(crate) fn verify_metrics_report(report: &Stage13CacheSchedulerReport) -> Result<(), String> {
    report.validate_bounded()
}

fn replay_report(
    ctx: &SimContext,
    stage: &DesignStage,
    store: &SettlementStore,
    examples: &[Stage13ExampleArtifact],
    metrics: &Stage13CacheSchedulerReport,
    root_generation: u8,
    final_root_hex: &str,
) -> Result<Stage13ReplayRootsReport, String> {
    let mut entries = Vec::new();
    for example in examples {
        verify_example_artifact(store, example, metrics)
            .map_err(|e| format!("stage13 replay verify {} failed: {e}", example.example_id))?;
        entries.push(Stage13ReplayEntry {
            schema_version: STAGE13_SCHEMA_VERSION,
            scenario_id: ctx.config.scenario.id,
            stage: stage.stage,
            example_id: example.example_id.clone(),
            backend_mode: example.backend_mode.clone(),
            api_surface: example.api_surface.clone(),
            verifier_status: "verified".to_string(),
            root_generation,
            typed_error: None,
            settlement_state_root_hex: example.settlement_state_root_hex.clone(),
            reloaded_settlement_state_root_hex: final_root_hex.to_string(),
        });
    }
    Ok(Stage13ReplayRootsReport {
        schema_version: STAGE13_SCHEMA_VERSION,
        scenario_id: ctx.config.scenario.id,
        stage: stage.stage,
        status: STAGE13_STATUS.to_string(),
        root_generation,
        artifact: report::report_artifact(
            "A3_replay_roots_report",
            "mixed",
            "stage13_hjmt_replay_roots_report",
        ),
        store_dir: "hjmt/store".to_string(),
        replay_entries: entries,
    })
}

fn tamper_report(
    ctx: &SimContext,
    stage: &DesignStage,
    store: &SettlementStore,
    examples: &[Stage13ExampleArtifact],
    comparison_rows: &[Stage13ProofComparisonRow],
    metrics: &Stage13CacheSchedulerReport,
    next_policy: BucketPolicy,
    include_path: z00z_storage::settlement::SettlementPath,
    root_generation: u8,
) -> Result<Stage13TamperReport, String> {
    let mut cases = Vec::new();

    let mut root_generation_drift = examples[0].clone();
    root_generation_drift.root_generation = root_generation.saturating_add(1);
    cases.push(tamper::case(
        ctx.config.scenario.id,
        stage.stage,
        root_generation_drift.example_id.clone(),
        root_generation_drift.backend_mode.clone(),
        root_generation_drift.api_surface.clone(),
        root_generation,
        "wrong_root_generation",
        typed_error(&expect_reject(verify_example_artifact(
            store,
            &root_generation_drift,
            metrics,
        ))?),
    ));

    let mut root_byte_drift = examples[0].clone();
    root_byte_drift.settlement_state_root_hex = "00".repeat(32);
    cases.push(tamper::case(
        ctx.config.scenario.id,
        stage.stage,
        root_byte_drift.example_id.clone(),
        root_byte_drift.backend_mode.clone(),
        root_byte_drift.api_surface.clone(),
        root_generation,
        "wrong_root_bytes",
        typed_error(&expect_reject(verify_example_artifact(
            store,
            &root_byte_drift,
            metrics,
        ))?),
    ));

    let mut proof_family_drift = examples[0].clone();
    proof_family_drift.proof_family = "nonexistence".to_string();
    cases.push(tamper::case(
        ctx.config.scenario.id,
        stage.stage,
        proof_family_drift.example_id.clone(),
        proof_family_drift.backend_mode.clone(),
        proof_family_drift.api_surface.clone(),
        root_generation,
        "wrong_proof_family",
        typed_error(&expect_reject(verify_example_artifact(
            store,
            &proof_family_drift,
            metrics,
        ))?),
    ));

    let mut leaf_family_drift = examples[1].clone();
    leaf_family_drift.leaf_family = "asset".to_string();
    cases.push(tamper::case(
        ctx.config.scenario.id,
        stage.stage,
        leaf_family_drift.example_id.clone(),
        leaf_family_drift.backend_mode.clone(),
        leaf_family_drift.api_surface.clone(),
        root_generation,
        "wrong_leaf_family",
        typed_error(&expect_reject(verify_example_artifact(
            store,
            &leaf_family_drift,
            metrics,
        ))?),
    ));

    let mut path_drift = examples[1].clone();
    path_drift.settlement_path = path_hex(include_path);
    path_drift.terminal_id = "ff".repeat(32);
    cases.push(tamper::case(
        ctx.config.scenario.id,
        stage.stage,
        path_drift.example_id.clone(),
        path_drift.backend_mode.clone(),
        path_drift.api_surface.clone(),
        root_generation,
        "wrong_terminal_path",
        typed_error(&expect_reject(verify_example_artifact(
            store,
            &path_drift,
            metrics,
        ))?),
    ));

    let mut bucket_epoch_drift = examples[5].clone();
    bucket_epoch_drift.bucket_epoch = bucket_epoch_drift.bucket_epoch.map(|epoch| epoch + 1);
    cases.push(tamper::case(
        ctx.config.scenario.id,
        stage.stage,
        bucket_epoch_drift.example_id.clone(),
        bucket_epoch_drift.backend_mode.clone(),
        bucket_epoch_drift.api_surface.clone(),
        root_generation,
        "wrong_bucket_epoch",
        typed_error(&expect_reject(verify_example_artifact(
            store,
            &bucket_epoch_drift,
            metrics,
        ))?),
    ));

    let mut stale_policy = examples[6].clone();
    stale_policy.bucket_policy_id = Some(hex::encode(store.bucket_policy().bucket_policy_id()));
    cases.push(tamper::case(
        ctx.config.scenario.id,
        stage.stage,
        stale_policy.example_id.clone(),
        stale_policy.backend_mode.clone(),
        stale_policy.api_surface.clone(),
        root_generation,
        "stale_policy_transition_id",
        typed_error(&expect_reject(verify_example_artifact(
            store,
            &stale_policy,
            metrics,
        ))?),
    ));

    let absence = store
        .settlement_nonexistence_proof_blob(
            &parse_path_hex(&examples[4].settlement_path)?,
            SettlementLeafFamily::Right,
        )
        .map_err(|e| e.to_string())?;
    let default_commitment_drift = match store.validate_settlement_proof_blob(
        &absence
            .clone()
            .with_hjmt_default_commitment(Some([0u8; 32])),
    ) {
        Ok(()) => {
            return Err("stage13 tampered default commitment unexpectedly validated".to_string())
        }
        Err(err) => err,
    };
    cases.push(tamper::case(
        ctx.config.scenario.id,
        stage.stage,
        EXAMPLE_ABSENT,
        "generalized",
        "validate_settlement_proof_blob",
        root_generation,
        "tampered_default_commitment",
        typed_error(&default_commitment_drift),
    ));

    let mut fee_transition_drift = examples[2].clone();
    fee_transition_drift.transition_binding = Some("11".repeat(32));
    cases.push(tamper::case(
        ctx.config.scenario.id,
        stage.stage,
        fee_transition_drift.example_id.clone(),
        fee_transition_drift.backend_mode.clone(),
        fee_transition_drift.api_surface.clone(),
        root_generation,
        "wrong_fee_transition_binding",
        typed_error(&expect_reject(verify_example_artifact(
            store,
            &fee_transition_drift,
            metrics,
        ))?),
    ));

    let mut missing_cache = metrics.clone();
    missing_cache.cache_hit_count = 0;
    missing_cache.cache_miss_count = 0;
    cases.push(tamper::case(
        ctx.config.scenario.id,
        stage.stage,
        EXAMPLE_METRICS,
        "adaptive",
        missing_cache.api_surface.clone(),
        root_generation,
        "missing_cache_metrics",
        typed_error(&expect_reject(verify_metrics_report(&missing_cache))?),
    ));

    let mut missing_scheduler = metrics.clone();
    missing_scheduler.deterministic_parent_ordering = false;
    cases.push(tamper::case(
        ctx.config.scenario.id,
        stage.stage,
        EXAMPLE_METRICS,
        "adaptive",
        missing_scheduler.api_surface.clone(),
        root_generation,
        "missing_scheduler_determinism",
        typed_error(&expect_reject(verify_metrics_report(&missing_scheduler))?),
    ));

    let clustered_inclusion = comparison_rows
        .iter()
        .find(|row| {
            row.proof_surface == PROOF_SURFACE_BATCH
                && row.proof_family == "inclusion"
                && row.path_shape == PATH_SHAPE_CLUSTERED
                && row.path_count == 2
        })
        .ok_or_else(|| "stage13 missing clustered inclusion batch comparison row".to_string())?;
    let clustered_nonexistence = comparison_rows
        .iter()
        .find(|row| {
            row.proof_surface == PROOF_SURFACE_BATCH
                && row.proof_family == "nonexistence"
                && row.path_shape == PATH_SHAPE_CLUSTERED
                && row.path_count == 2
        })
        .ok_or_else(|| "stage13 missing clustered nonexistence batch comparison row".to_string())?;

    let mut batch_wrong_root_generation = batch_from_comparison_row(store, clustered_inclusion)?;
    batch_wrong_root_generation.header.root_generation = RootGenerationTagV1::RootGeneration0;
    cases.push(batch_tamper_case(
        ctx,
        stage,
        clustered_inclusion,
        root_generation,
        "batch_wrong_root_generation",
        typed_error(&expect_batch_reject(batch_wrong_root_generation)?),
    ));

    let mut batch_reordered_paths = batch_from_comparison_row(store, clustered_inclusion)?;
    batch_reordered_paths.path_table.swap(0, 1);
    cases.push(batch_tamper_case(
        ctx,
        stage,
        clustered_inclusion,
        root_generation,
        "batch_reordered_paths",
        typed_error(&expect_batch_reject(batch_reordered_paths)?),
    ));

    let mut batch_duplicate_path = batch_from_comparison_row(store, clustered_inclusion)?;
    batch_duplicate_path.path_table[1].path = batch_duplicate_path.path_table[0].path;
    cases.push(batch_tamper_case(
        ctx,
        stage,
        clustered_inclusion,
        root_generation,
        "batch_duplicate_path",
        typed_error(&expect_batch_reject(batch_duplicate_path)?),
    ));

    let mut batch_mixed_proof_family = batch_from_comparison_row(store, clustered_inclusion)?;
    batch_mixed_proof_family.header.proof_family = BatchProofFamilyTagV1::Deletion;
    cases.push(batch_tamper_case(
        ctx,
        stage,
        clustered_inclusion,
        root_generation,
        "batch_mixed_proof_family",
        typed_error(&expect_batch_reject(batch_mixed_proof_family)?),
    ));

    let mut batch_opening_kind_mismatch = batch_from_comparison_row(store, clustered_inclusion)?;
    let marker_leaf = SettlementLeafFamily::Terminal.marker_leaf(
        *comparison_paths(clustered_inclusion)?
            .first()
            .ok_or_else(|| "stage13 clustered inclusion row lost paths".to_string())?,
    );
    batch_opening_kind_mismatch.opening_table[0] = OpeningEntryV1::from_nonexistence(
        NonExistenceOpeningV1::new(&marker_leaf).map_err(|e| e.to_string())?,
    );
    cases.push(batch_tamper_case(
        ctx,
        stage,
        clustered_inclusion,
        root_generation,
        "batch_opening_kind_mismatch",
        typed_error(&expect_batch_reject(batch_opening_kind_mismatch)?),
    ));

    let mut batch_leaf_family_mismatch = batch_from_comparison_row(store, clustered_inclusion)?;
    batch_leaf_family_mismatch.path_table[0].leaf_family = LeafFamilyTagV1::Right;
    batch_leaf_family_mismatch.path_table[0].terminal_family = TerminalFamilyTagV1::Right;
    cases.push(batch_tamper_case(
        ctx,
        stage,
        clustered_inclusion,
        root_generation,
        "batch_leaf_family_mismatch",
        typed_error(&expect_batch_reject(batch_leaf_family_mismatch)?),
    ));

    let mut batch_witness_ref_out_of_range = batch_from_comparison_row(store, clustered_inclusion)?;
    batch_witness_ref_out_of_range.reference_table[0].witness_indexes = vec![9_999];
    cases.push(batch_tamper_case(
        ctx,
        stage,
        clustered_inclusion,
        root_generation,
        "batch_witness_ref_out_of_range",
        typed_error(&expect_batch_reject(batch_witness_ref_out_of_range)?),
    ));

    let mut batch_wrong_default_commitment =
        batch_from_comparison_row(store, clustered_nonexistence)?;
    let mut opening = batch_wrong_default_commitment.opening_table[0]
        .decode_nonexistence()
        .map_err(|e| e.to_string())?;
    opening.default_commitment = [0u8; 32];
    batch_wrong_default_commitment.opening_table[0] = OpeningEntryV1::from_nonexistence(opening);
    cases.push(batch_tamper_case(
        ctx,
        stage,
        clustered_nonexistence,
        root_generation,
        "batch_wrong_default_commitment",
        typed_error(&expect_batch_reject(batch_wrong_default_commitment)?),
    ));

    let mut batch_wrong_witness_domain = batch_from_comparison_row(store, clustered_inclusion)?;
    batch_wrong_witness_domain.witness_dag[0].node_domain = NodeDomainTagV1::Shard;
    cases.push(batch_tamper_case(
        ctx,
        stage,
        clustered_inclusion,
        root_generation,
        "batch_wrong_witness_domain",
        typed_error(&expect_batch_reject(batch_wrong_witness_domain)?),
    ));

    let mut batch_hash_material_count = batch_from_comparison_row(store, clustered_inclusion)?;
    batch_hash_material_count.witness_dag[0]
        .hash_material
        .push([0x44; 32]);
    cases.push(batch_tamper_case(
        ctx,
        stage,
        clustered_inclusion,
        root_generation,
        "batch_hash_material_count",
        typed_error(&expect_batch_reject(batch_hash_material_count)?),
    ));

    // Ensure the live transition verifier still accepts the intended next policy.
    let _ = store
        .policy_transition_proof(next_policy)
        .map_err(|e| format!("stage13 tamper baseline policy transition failed: {e}"))?;

    Ok(Stage13TamperReport {
        schema_version: STAGE13_SCHEMA_VERSION,
        scenario_id: ctx.config.scenario.id,
        stage: stage.stage,
        status: STAGE13_STATUS.to_string(),
        root_generation,
        artifact: report::report_artifact(
            "A4_tamper_report",
            "mixed",
            "stage13_hjmt_tamper_report",
        ),
        cases,
    })
}

fn comparison_paths(row: &Stage13ProofComparisonRow) -> Result<Vec<SettlementPath>, String> {
    row.settlement_paths
        .iter()
        .map(|raw| parse_path_hex(raw))
        .collect::<Result<Vec<_>, _>>()
}

fn batch_from_comparison_row(
    store: &SettlementStore,
    row: &Stage13ProofComparisonRow,
) -> Result<BatchProofBlobV1, String> {
    let family = parse_comparison_family(&row.proof_family)?;
    let paths = comparison_paths(row)?;
    build_batch_blob(store, family, &paths)
}

fn expect_batch_reject(
    batch: BatchProofBlobV1,
) -> Result<z00z_storage::settlement::ProofChkErr, String> {
    let bytes = batch.encode().map_err(|e| e.to_string())?;
    match BatchProofBlobV1::decode(&bytes) {
        Ok(_) => Err("stage13 batch tamper unexpectedly succeeded".to_string()),
        Err(err) => Ok(err),
    }
}

fn batch_tamper_case(
    ctx: &SimContext,
    stage: &DesignStage,
    row: &Stage13ProofComparisonRow,
    root_generation: u8,
    case_id: &str,
    error: report::RedactedError,
) -> Stage13TamperCase {
    Stage13TamperCase {
        schema_version: STAGE13_SCHEMA_VERSION,
        scenario_id: ctx.config.scenario.id,
        stage: stage.stage,
        example_id: row.owner_example_id.clone(),
        backend_mode: row.backend_mode.clone(),
        api_surface: row.api_surface.clone(),
        proof_surface: PROOF_SURFACE_BATCH.to_string(),
        verifier_status: "rejected".to_string(),
        root_generation,
        path_count: Some(row.path_count),
        path_shape: Some(row.path_shape.clone()),
        case_id: case_id.to_string(),
        typed_error: error,
    }
}

pub(crate) fn verify_example_artifact(
    store: &SettlementStore,
    artifact: &Stage13ExampleArtifact,
    metrics: &Stage13CacheSchedulerReport,
) -> Result<(), String> {
    if artifact.root_generation != RootGeneration::SettlementV1.version() {
        return Err("RootGenerationMix".to_string());
    }
    let live_root_hex = hex::encode(
        store
            .settlement_root()
            .map_err(|e| e.to_string())?
            .into_bytes(),
    );
    if artifact.settlement_state_root_hex != live_root_hex {
        return Err("RootBindMix".to_string());
    }
    if artifact.example_id != EXAMPLE_METRICS
        && (artifact.proof_size_bytes.unwrap_or(0) == 0
            || artifact.verify_time_us.unwrap_or(0) == 0)
    {
        return Err("ProofMeasureMix".to_string());
    }
    match artifact.example_id.as_str() {
        EXAMPLE_ASSET => verify_inclusion_family(store, artifact, SettlementLeafFamily::Terminal),
        EXAMPLE_RIGHT => verify_inclusion_family(store, artifact, SettlementLeafFamily::Right),
        EXAMPLE_FEE => verify_fee_family(store, artifact),
        EXAMPLE_DELETE => verify_deletion_family(store, artifact),
        EXAMPLE_ABSENT => verify_nonexistence_family(store, artifact),
        EXAMPLE_SPLIT => verify_split_family(store, artifact),
        EXAMPLE_POLICY => verify_policy_family(store, artifact),
        EXAMPLE_METRICS => verify_metrics_report(metrics),
        other => Err(format!("stage13 unknown example id: {other}")),
    }
}

pub(crate) fn verify_comparison_row(
    store: &SettlementStore,
    row: &Stage13ProofComparisonRow,
) -> Result<(), String> {
    if row.root_generation != RootGeneration::SettlementV1.version() {
        return Err("BatchRootGenerationMix".to_string());
    }
    if row.verifier_status != "verified" || row.typed_error.is_some() {
        return Err("BatchVerifierStatusMix".to_string());
    }
    if row.proof_size_bytes == 0 || row.verify_time_us == 0 {
        return Err("BatchMeasureMix".to_string());
    }
    let live_root_hex = hex::encode(
        store
            .settlement_root()
            .map_err(|e| e.to_string())?
            .into_bytes(),
    );
    if row.settlement_state_root_hex != live_root_hex {
        return Err("BatchRootBindMix".to_string());
    }
    let paths = row
        .settlement_paths
        .iter()
        .map(|raw| parse_path_hex(raw))
        .collect::<Result<Vec<_>, _>>()?;
    if row.path_count != u32::try_from(paths.len()).unwrap_or(u32::MAX) || paths.is_empty() {
        return Err("BatchPathCountMix".to_string());
    }
    if row.canonical_order && !paths.windows(2).all(|pair| pair[0] <= pair[1]) {
        return Err("BatchCanonicalOrderMix".to_string());
    }
    let family = parse_comparison_family(&row.proof_family)?;
    if row.leaf_family != leaf_family_name(family.leaf_family()) {
        return Err("BatchLeafFamilyMix".to_string());
    }
    let surface = parse_comparison_surface(&row.proof_surface)?;
    let actual_shape = comparison_shape_for_paths(store.bucket_policy(), &paths);
    let expected_shape = if matches!(surface, ComparisonSurface::Single) {
        PATH_SHAPE_SINGLE
    } else {
        actual_shape
    };
    if row.path_shape != expected_shape {
        return Err("BatchPathShapeMix".to_string());
    }
    let (actual_bytes, _verify_time_us, shard_context_mode) =
        measure_comparison_row(store, family, surface, &paths)?;
    if row.proof_size_bytes != actual_bytes {
        return Err("BatchProofBytesMix".to_string());
    }
    if row.shard_context_mode != shard_context_mode {
        return Err("BatchShardContextMix".to_string());
    }
    let expected_atomic = if matches!(surface, ComparisonSurface::Batch) {
        ATOMIC_VERDICT_ACCEPTED
    } else {
        ATOMIC_VERDICT_INDEPENDENT
    };
    if row.atomic_verdict != expected_atomic {
        return Err("BatchAtomicVerdictMix".to_string());
    }
    Ok(())
}

fn parse_comparison_family(raw: &str) -> Result<ComparisonFamily, String> {
    match raw {
        "inclusion" => Ok(ComparisonFamily::Inclusion),
        "deletion" => Ok(ComparisonFamily::Deletion),
        "nonexistence" => Ok(ComparisonFamily::NonExistence),
        other => Err(format!("stage13 unknown comparison proof_family: {other}")),
    }
}

fn parse_comparison_surface(raw: &str) -> Result<ComparisonSurface, String> {
    match raw {
        PROOF_SURFACE_SINGLE => Ok(ComparisonSurface::Single),
        PROOF_SURFACE_VEC => Ok(ComparisonSurface::Vec),
        PROOF_SURFACE_BATCH => Ok(ComparisonSurface::Batch),
        other => Err(format!("stage13 unknown comparison proof_surface: {other}")),
    }
}

fn comparison_shape_for_paths(policy: BucketPolicy, paths: &[SettlementPath]) -> &'static str {
    if paths.len() <= 1 {
        return PATH_SHAPE_SINGLE;
    }
    let first_bucket = policy.derive_bucket_id(paths[0]);
    if paths
        .iter()
        .all(|path| policy.derive_bucket_id(*path) == first_bucket)
    {
        PATH_SHAPE_CLUSTERED
    } else {
        PATH_SHAPE_SCATTERED
    }
}

fn verify_hjmt_blob_fields(
    artifact: &Stage13ExampleArtifact,
    blob: &z00z_storage::settlement::ProofBlob,
    path: z00z_storage::settlement::SettlementPath,
    proof_family: HjmtProofFamily,
    leaf_family: SettlementLeafFamily,
) -> Result<(), String> {
    if artifact.proof_family != proof_family_name(proof_family) {
        return Err("ProofFamilyMix".to_string());
    }
    if artifact.leaf_family != leaf_family_name(leaf_family) {
        return Err("LeafMix".to_string());
    }
    if artifact.terminal_id != terminal_hex(path) {
        return Err("PathTerminalMix".to_string());
    }
    if artifact.proof_envelope_version
        != blob
            .hjmt_envelope_version()
            .or(Some(HJMT_PROOF_ENVELOPE_VERSION))
    {
        return Err("ProofEnvelopeMix".to_string());
    }
    if artifact.bucket_epoch != blob.hjmt_journal_checkpoint() {
        return Err("WrongEpoch".to_string());
    }
    let live_policy = blob
        .hjmt_bucket_policy()
        .map(|policy| hex::encode(policy.bucket_policy_id()));
    if artifact.bucket_policy_id != live_policy {
        return Err("BucketPolicyMix".to_string());
    }
    Ok(())
}

fn verify_inclusion_family(
    store: &SettlementStore,
    artifact: &Stage13ExampleArtifact,
    family: SettlementLeafFamily,
) -> Result<(), String> {
    let path = parse_path_hex(&artifact.settlement_path)?;
    let blob = store
        .settlement_proof_blob(&path)
        .map_err(|e| e.to_string())?;
    store
        .validate_settlement_proof_blob(&blob)
        .map_err(|e| format!("{e:?}"))?;
    verify_hjmt_blob_fields(artifact, &blob, path, HjmtProofFamily::Inclusion, family)
}

fn verify_fee_family(
    store: &SettlementStore,
    artifact: &Stage13ExampleArtifact,
) -> Result<(), String> {
    verify_inclusion_family(store, artifact, SettlementLeafFamily::Right)?;
    if artifact.proof_is_ownership != Some(false) {
        return Err("OwnershipMix".to_string());
    }
    let replay_key_hex = artifact
        .fee_envelope_id
        .as_deref()
        .ok_or_else(|| "stage13 fee example missing fee_envelope_id".to_string())?;
    let replay_key = FeeReplayKey::new(
        hex::decode(replay_key_hex)
            .map_err(|e| format!("stage13 fee replay key decode failed: {e}"))?
            .try_into()
            .map_err(|_| "stage13 fee replay key length drifted".to_string())?,
    );
    let replay = store
        .fee_replay_rec(&replay_key)
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "stage13 fee replay row missing".to_string())?;
    if artifact.fee_domain.as_deref() != Some(hex::encode(replay.domain_id).as_str()) {
        return Err("FeeDomainMix".to_string());
    }
    if artifact.transition_binding.as_deref() != Some(hex::encode(replay.transition_id).as_str()) {
        return Err("TransitionMix".to_string());
    }
    if artifact.payer_commitment.as_deref() != Some(hex::encode(replay.payer_commitment).as_str()) {
        return Err("PayerMix".to_string());
    }
    if artifact.sponsor_commitment.as_deref()
        != Some(hex::encode(replay.sponsor_commitment).as_str())
    {
        return Err("SponsorMix".to_string());
    }
    if artifact.expiry != Some(replay.expires_at) {
        return Err("ExpiryMix".to_string());
    }
    if artifact.replay_status.as_deref() != Some("accepted") {
        return Err("ReplayStatusMix".to_string());
    }
    Ok(())
}

fn verify_deletion_family(
    store: &SettlementStore,
    artifact: &Stage13ExampleArtifact,
) -> Result<(), String> {
    let path = parse_path_hex(&artifact.settlement_path)?;
    let blob = store
        .settlement_proof_blob(&path)
        .map_err(|e| e.to_string())?;
    store
        .validate_settlement_proof_blob(&blob)
        .map_err(|e| format!("{e:?}"))?;
    verify_hjmt_blob_fields(
        artifact,
        &blob,
        path,
        HjmtProofFamily::Deletion,
        SettlementLeafFamily::Right,
    )
}

fn verify_nonexistence_family(
    store: &SettlementStore,
    artifact: &Stage13ExampleArtifact,
) -> Result<(), String> {
    let path = parse_path_hex(&artifact.settlement_path)?;
    let blob = store
        .settlement_nonexistence_proof_blob(&path, SettlementLeafFamily::Right)
        .map_err(|e| e.to_string())?;
    store
        .validate_settlement_nonexistence_proof_blob(&blob, SettlementLeafFamily::Right)
        .map_err(|e| format!("{e:?}"))?;
    verify_hjmt_blob_fields(
        artifact,
        &blob,
        path,
        HjmtProofFamily::NonExistence,
        SettlementLeafFamily::Right,
    )?;
    if artifact
        .present_key_rejection
        .as_deref()
        .is_none_or(|value| value.trim().is_empty())
    {
        return Err("PresentKeyRejectMix".to_string());
    }
    Ok(())
}

fn verify_split_family(
    store: &SettlementStore,
    artifact: &Stage13ExampleArtifact,
) -> Result<(), String> {
    let path = parse_path_hex(&artifact.settlement_path)?;
    let proof = store.split_proof(&path).map_err(|e| e.to_string())?;
    store
        .validate_split_proof(&proof)
        .map_err(|e| format!("{e:?}"))?;
    if artifact.proof_family != "split" {
        return Err("ProofFamilyMix".to_string());
    }
    if artifact.leaf_family != leaf_family_name(SettlementLeafFamily::Terminal) {
        return Err("LeafMix".to_string());
    }
    if artifact.terminal_id != terminal_hex(path) {
        return Err("PathTerminalMix".to_string());
    }
    if artifact.bucket_epoch != Some(proof.prior_epoch.get()) {
        return Err("WrongEpoch".to_string());
    }
    if artifact.prior_state_root_hex.as_deref()
        != Some(hex::encode(proof.prior_root.into_bytes()).as_str())
    {
        return Err("PriorRootMix".to_string());
    }
    if artifact.next_state_root_hex.as_deref()
        != Some(hex::encode(proof.next_root.into_bytes()).as_str())
    {
        return Err("NextRootMix".to_string());
    }
    if artifact.bucket_policy_id.as_deref() != Some(hex::encode(proof.bucket_policy_id).as_str()) {
        return Err("BucketPolicyMix".to_string());
    }
    Ok(())
}

fn verify_policy_family(
    store: &SettlementStore,
    artifact: &Stage13ExampleArtifact,
) -> Result<(), String> {
    let next_policy = next_policy(store.bucket_policy())?;
    let proof = store
        .policy_transition_proof(next_policy)
        .map_err(|e| e.to_string())?;
    store
        .validate_policy_transition_proof(&proof, next_policy)
        .map_err(|e| format!("{e:?}"))?;
    let path = parse_path_hex(&artifact.settlement_path)?;
    if artifact.proof_family != "policy_transition" {
        return Err("ProofFamilyMix".to_string());
    }
    if artifact.leaf_family != leaf_family_name(SettlementLeafFamily::Terminal) {
        return Err("LeafMix".to_string());
    }
    if artifact.terminal_id != terminal_hex(path) {
        return Err("PathTerminalMix".to_string());
    }
    if artifact.bucket_epoch != Some(proof.prior_epoch.get()) {
        return Err("WrongEpoch".to_string());
    }
    if artifact.prior_state_root_hex.as_deref()
        != Some(hex::encode(proof.prior_root.into_bytes()).as_str())
    {
        return Err("PriorRootMix".to_string());
    }
    if artifact.next_state_root_hex.as_deref()
        != Some(hex::encode(proof.next_root.into_bytes()).as_str())
    {
        return Err("NextRootMix".to_string());
    }
    if artifact.bucket_policy_id.as_deref() != Some(hex::encode(proof.next_policy_id).as_str()) {
        return Err("StalePolicyId".to_string());
    }
    if artifact.transition_binding.as_deref() != Some(hex::encode(proof.next_policy_id).as_str()) {
        return Err("TransitionMix".to_string());
    }
    Ok(())
}

fn copy_manifest(src: &Path, dst: &Path) -> Result<(), String> {
    if !path_exists(src).map_err(|e| e.to_string())? {
        return Err(format!(
            "stage13 genesis settlement manifest missing: {}",
            src.display()
        ));
    }
    if let Some(parent) = dst.parent() {
        create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    let bytes = read_file(src).map_err(|e| e.to_string())?;
    write_file(dst, &bytes).map_err(|e| e.to_string())?;
    Ok(())
}

fn rel(base: &Path, path: &Path) -> String {
    path.strip_prefix(base)
        .map(|value| value.to_string_lossy().to_string())
        .unwrap_or_else(|_| path.to_string_lossy().to_string())
}

fn artifact_names() -> Vec<String> {
    vec![
        "hjmt/hjmt_settlement_examples.json".to_string(),
        "hjmt/hjmt_tamper_report.json".to_string(),
        "hjmt/hjmt_proof_size_report.json".to_string(),
        "hjmt/hjmt_cache_scheduler_metrics.json".to_string(),
        "hjmt/hjmt_replay_roots.json".to_string(),
        "hjmt/genesis_settlement_manifest.json".to_string(),
    ]
}

fn timed<T, F>(f: F) -> Result<(T, u64), String>
where
    F: FnOnce() -> Result<T, String>,
{
    let started = Instant::now();
    let value = f()?;
    let micros = started.elapsed().as_micros();
    Ok((value, u64::try_from(micros).unwrap_or(u64::MAX)))
}

fn next_policy(current: BucketPolicy) -> Result<BucketPolicy, String> {
    BucketPolicy::new(
        current.bucket_bits(),
        current.min_bucket_count(),
        current.max_target_leaf_count(),
        current.compatibility_generation().saturating_add(1),
    )
    .map_err(|e| e.to_string())
}

fn expect_reject(result: Result<(), String>) -> Result<String, String> {
    match result {
        Ok(()) => Err("stage13 tamper case unexpectedly succeeded".to_string()),
        Err(err) => Ok(err),
    }
}

fn stage_log_rows(
    stage: &DesignStage,
    paths: &Stage13Paths,
    final_root_hex: &str,
    artifact_names: &[String],
) -> Vec<String> {
    let details = [
        format!(
            "hjmt output prepared at {} and manifest linked to {}",
            paths.output_dir.display(),
            paths.manifest_dst.display()
        ),
        "yaml-generated genesis asset and rights were seeded into the live settlement store".to_string(),
        "asset and right inclusion proofs plus fee-supported right transition were verified through live storage APIs".to_string(),
        "right deletion and non-existence proofs were verified and present-key rejection stayed fail-closed".to_string(),
        format!(
            "adaptive split and policy-transition proofs were produced from the live store without exposing physical layout authority; {}; forbidden source terms={}",
            source_shape_note(),
            FORBIDDEN_SOURCE_TERMS.join(", ")
        ),
        "cache and scheduler metrics were emitted after deterministic proof warmup".to_string(),
        format!("reload-debug reopened the durable store and re-verified every example against persisted root {final_root_hex}"),
        format!("wrote artifacts: {}", artifact_names.join(", ")),
    ];

    stage
        .steps
        .iter()
        .zip(details)
        .map(|(step, detail)| {
            json!({
                "timestamp": "2026-01-01T00:00:00Z",
                "stage": stage.stage,
                "step": step.id,
                "event": "live",
                "status": "ok",
                "detail": detail,
            })
            .to_string()
        })
        .collect()
}
