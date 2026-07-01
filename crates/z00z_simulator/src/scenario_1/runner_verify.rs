use std::{
    collections::{BTreeSet, HashSet},
    path::{Path, PathBuf},
    sync::Arc,
    thread::sleep,
    time::Duration,
};

use z00z_core::genesis::{
    compute_genesis_manifest_hash, compute_genesis_policies_digest, compute_genesis_rights_digest,
    compute_genesis_seed_hash, compute_genesis_vouchers_digest, create_asset_definition,
    ensure_terminal_collision_free, generate_genesis_policies, generate_genesis_settlement_corpus,
    genesis_config::load_genesis_config, validator::compute_genesis_state_hash,
    GenesisPolicyRecord, GenesisRightRecord, GenesisSeed, GenesisSettlementCorpus,
    GenesisSettlementManifest, GenesisVoucherRecord, GENESIS_POLICIES_FILE,
    GENESIS_POLICIES_REPLAY_DIGEST_LABEL, GENESIS_POLICIES_ROUNDTRIP_DIGEST_LABEL,
    GENESIS_RIGHTS_FILE, GENESIS_RIGHTS_REPLAY_DIGEST_LABEL, GENESIS_RIGHTS_ROUNDTRIP_DIGEST_LABEL,
    GENESIS_ROOT_GENERATION, GENESIS_SETTLEMENT_MANIFEST_FILE, GENESIS_VOUCHERS_FILE,
    GENESIS_VOUCHERS_REPLAY_DIGEST_LABEL, GENESIS_VOUCHERS_ROUNDTRIP_DIGEST_LABEL,
};
use z00z_storage::settlement::SettlementStore;
use z00z_utils::{
    codec::{Codec, JsonCodec, Value},
    io,
    logger::{Logger, NoopLogger},
    metrics::NoopMetrics,
};

use crate::scenario_1::stage_13::{
    hjmt_examples,
    report::{
        redaction_violation, RedactedError, Stage13ArtifactMeta, Stage13CacheSchedulerReport,
        Stage13ExamplesReport, Stage13ProofComparisonRow, Stage13ProofSizeReport,
        Stage13ReplayRootsReport, Stage13TamperReport, ATOMIC_VERDICT_ACCEPTED,
        PATH_SHAPE_CLUSTERED, PATH_SHAPE_SCATTERED, PROOF_SURFACE_BATCH, PROOF_SURFACE_SINGLE,
        PROOF_SURFACE_VEC, SHARD_CONTEXT_NONE,
    },
};

use crate::{DesignStage, SimContext, StageResult};

const STAGE13_STATUS: &str = "hjmt_examples_complete";
const STAGE13_MODE: &str = "generalized_settlement";
const STAGE13_LOG_FILE: &str = "hjmt/stage13_hjmt_examples.log";
const STAGE13_REQUIRED_ARTIFACTS: &[&str] = &[
    "hjmt/hjmt_settlement_examples.json",
    "hjmt/hjmt_tamper_report.json",
    "hjmt/hjmt_proof_size_report.json",
    "hjmt/hjmt_cache_scheduler_metrics.json",
    "hjmt/hjmt_replay_roots.json",
    "hjmt/genesis_settlement_manifest.json",
];
const STAGE13_EXPECTED_EXAMPLES: &[&str] = &[
    "E1_asset_inclusion",
    "E2_right_inclusion",
    "E3_fee_transition",
    "E4_right_deletion",
    "E5_right_nonexistence",
    "E6_adaptive_split",
    "E7_policy_transition",
    "E8_cache_scheduler",
];
const STAGE13_EXPECTED_TAMPER_CASES: &[&str] = &[
    "wrong_root_generation",
    "wrong_root_bytes",
    "wrong_proof_family",
    "wrong_leaf_family",
    "wrong_terminal_path",
    "wrong_bucket_epoch",
    "stale_policy_transition_id",
    "tampered_default_commitment",
    "wrong_fee_transition_binding",
    "missing_cache_metrics",
    "missing_scheduler_determinism",
    "batch_wrong_root_generation",
    "batch_reordered_paths",
    "batch_duplicate_path",
    "batch_mixed_proof_family",
    "batch_opening_kind_mismatch",
    "batch_leaf_family_mismatch",
    "batch_witness_ref_out_of_range",
    "batch_wrong_default_commitment",
    "batch_wrong_witness_domain",
    "batch_hash_material_count",
];
const STAGE13_REQUIRED_EXAMPLE_FIELDS: &[&str] = &[
    "root_generation",
    "proof_envelope_version",
    "proof_family",
    "leaf_family",
    "settlement_path",
    "terminal_id",
    "bucket_epoch",
    "verifier_status",
];
const STAGE13_REQUIRED_COMPARISON_FIELDS: &[&str] = &[
    "row_id",
    "proof_surface",
    "proof_family",
    "leaf_family",
    "path_count",
    "path_shape",
    "canonical_order",
    "atomic_verdict",
    "shard_context_mode",
    "root_generation",
    "settlement_state_root_hex",
    "settlement_paths",
    "proof_size_bytes",
    "verify_time_us",
    "verifier_status",
];
const POST_CONDITION_RETRIES: u32 = 200;
const POST_CONDITION_WAIT_MS: u64 = 50;

fn verify_stage13_artifact_meta(label: &str, artifact: &Stage13ArtifactMeta) -> Result<(), String> {
    if artifact.example_id.trim().is_empty() {
        return Err(format!("{label} example_id must not be empty"));
    }
    if artifact.backend_mode.trim().is_empty() {
        return Err(format!("{label} backend_mode must not be empty"));
    }
    if artifact.api_surface.trim().is_empty() {
        return Err(format!("{label} api_surface must not be empty"));
    }
    if artifact.verifier_status != "verified" {
        return Err(format!("{label} verifier_status must stay verified"));
    }
    if let Some(err) = artifact.typed_error.as_ref() {
        verify_redacted_error(label, err)?;
    }
    Ok(())
}

fn verify_redacted_error(label: &str, err: &RedactedError) -> Result<(), String> {
    if err.class.trim().is_empty() || err.message.trim().is_empty() {
        return Err(format!("{label} typed_error must stay populated"));
    }
    if let Some(kind) = redaction_violation(&err.class) {
        return Err(format!(
            "{label} typed_error.class violated redaction: {kind}"
        ));
    }
    if let Some(kind) = redaction_violation(&err.message) {
        return Err(format!(
            "{label} typed_error.message violated redaction: {kind}"
        ));
    }
    Ok(())
}

fn verify_stage13_examples_schema(raw: &Value) -> Result<(), String> {
    let examples = raw
        .get("examples")
        .and_then(|value| value.as_array())
        .ok_or_else(|| "stage13 examples report missing examples[]".to_string())?;
    for (idx, example) in examples.iter().enumerate() {
        let obj = example
            .as_object()
            .ok_or_else(|| format!("stage13 examples report example[{idx}] must be an object"))?;
        for field in STAGE13_REQUIRED_EXAMPLE_FIELDS {
            if !obj.contains_key(*field) {
                return Err(format!(
                    "stage13 examples report example[{idx}] missing field {field}"
                ));
            }
        }
    }
    let comparison_rows = raw
        .get("comparison_rows")
        .and_then(|value| value.as_array())
        .ok_or_else(|| "stage13 examples report missing comparison_rows[]".to_string())?;
    for (idx, row) in comparison_rows.iter().enumerate() {
        let obj = row.as_object().ok_or_else(|| {
            format!("stage13 examples report comparison_rows[{idx}] must be an object")
        })?;
        for field in STAGE13_REQUIRED_COMPARISON_FIELDS {
            if !obj.contains_key(*field) {
                return Err(format!(
                    "stage13 examples report comparison_rows[{idx}] missing field {field}"
                ));
            }
        }
    }
    Ok(())
}

fn verify_stage13_comparison_row(
    row: &Stage13ProofComparisonRow,
    store: &SettlementStore,
) -> Result<(), String> {
    if row.row_id.trim().is_empty() {
        return Err("stage13 comparison row_id must stay populated".to_string());
    }
    if row.backend_mode.trim().is_empty() || row.api_surface.trim().is_empty() {
        return Err(format!(
            "stage13 comparison row {} binding fields must stay populated",
            row.row_id
        ));
    }
    hjmt_examples::verify_comparison_row(store, row).map_err(|err| {
        format!(
            "stage13 comparison row {} failed live verification: {err}",
            row.row_id
        )
    })
}

pub(super) fn check_stage(ctx: &SimContext, stage: &DesignStage) -> StageResult {
    if let Err(err) = validate_step_coverage(ctx, stage) {
        return StageResult::Fail(format!(
            "stage {} ({}) missing step coverage: {}",
            stage.stage, stage.name, err
        ));
    }

    if stage.stage == 3 {
        if let Err(err) = retry_post_conditions(|| verify_claim_outputs(ctx)) {
            return StageResult::Fail(format!(
                "stage {} ({}) missing post-conditions: {}",
                stage.stage, stage.name, err
            ));
        }
    }

    if stage.stage == 1 {
        if let Err(err) = retry_post_conditions(|| verify_stage1_outputs(ctx)) {
            return StageResult::Fail(format!(
                "stage {} ({}) missing post-conditions: {}",
                stage.stage, stage.name, err
            ));
        }
    }

    if stage.stage == 4 {
        if let Err(err) = retry_post_conditions(|| verify_claim_publish_outputs(ctx)) {
            return StageResult::Fail(format!(
                "stage {} ({}) missing post-conditions: {}",
                stage.stage, stage.name, err
            ));
        }
    }

    if stage.stage == 11 {
        if let Err(err) = retry_post_conditions(|| verify_stage11_right_rejection(ctx)) {
            return StageResult::Fail(format!(
                "stage {} ({}) missing post-conditions: {}",
                stage.stage, stage.name, err
            ));
        }
    }

    if stage.stage == 13 {
        if let Err(err) = retry_post_conditions(|| verify_stage13_contract(ctx, stage)) {
            return StageResult::Fail(format!(
                "stage {} ({}) missing post-conditions: {}",
                stage.stage, stage.name, err
            ));
        }

        return StageResult::Ok;
    }

    StageResult::Ok
}

fn retry_post_conditions<T>(mut check: impl FnMut() -> Result<T, String>) -> Result<T, String> {
    let mut last_err = String::new();
    for attempt in 0..=POST_CONDITION_RETRIES {
        match check() {
            Ok(value) => return Ok(value),
            Err(err) => {
                last_err = err;
                if attempt < POST_CONDITION_RETRIES {
                    sleep(Duration::from_millis(POST_CONDITION_WAIT_MS));
                }
            }
        }
    }
    Err(last_err)
}

pub(super) fn log_stage(logger: &impl Logger, stage: &DesignStage, result: &StageResult) {
    logger.info(&format!(
        "stage.done: id={}, name={}, result={}",
        stage.stage,
        stage.name,
        result_tag(result)
    ));
    match result {
        StageResult::Warn(msg) => logger.warn(&format!(
            "stage.warn: id={}, name={}, msg={}",
            stage.stage, stage.name, msg
        )),
        StageResult::Fail(msg) => logger.error(&format!(
            "stage.fail: id={}, name={}, msg={}",
            stage.stage, stage.name, msg
        )),
        StageResult::Ok => {}
    }
}

fn validate_step_coverage(ctx: &SimContext, stage: &DesignStage) -> Result<(), String> {
    if stage.steps.is_empty() {
        return Ok(());
    }

    let log_path = stage_log_file(ctx, stage)?;
    if !io::path_exists(&log_path).map_err(|e| e.to_string())? {
        return Err(format!("log file missing: {}", log_path.display()));
    }

    let text = io::read_to_string(&log_path).map_err(|e| e.to_string())?;
    let seen_steps = seen_steps(&text, stage.stage)?;

    let missing: Vec<&str> = stage
        .steps
        .iter()
        .map(|s| s.id.as_str())
        .filter(|id| !seen_steps.contains(*id))
        .collect();

    if !missing.is_empty() {
        return Err(format!("missing steps: {}", missing.join(", ")));
    }

    Ok(())
}

fn seen_steps(text: &str, stage_id: u32) -> Result<HashSet<String>, String> {
    let mut seen = HashSet::<String>::new();
    for line in text.lines().map(str::trim).filter(|line| !line.is_empty()) {
        if let Some(step) = log_step(line, stage_id)? {
            seen.insert(step);
        }
    }
    Ok(seen)
}

fn log_step(line: &str, stage_id: u32) -> Result<Option<String>, String> {
    let row: Value = JsonCodec
        .deserialize(line.as_bytes())
        .map_err(|e| format!("invalid log json: {e}"))?;
    let row_stage = row
        .get("stage")
        .and_then(|v| v.as_u64())
        .ok_or_else(|| "log row missing numeric 'stage'".to_string())?;
    if row_stage != stage_id as u64 {
        return Ok(None);
    }
    Ok(row
        .get("step")
        .and_then(|v| v.as_str())
        .map(|step| step.to_string()))
}

fn stage_log_file(ctx: &SimContext, stage: &DesignStage) -> Result<PathBuf, String> {
    match stage.stage {
        1 => {
            let p = ctx.config.stage1_paths();
            Ok(ctx.outputs_dir.join(&p.logs_dir).join(&p.logger_file))
        }
        2 => {
            let p = ctx.config.stage2_paths();
            Ok(ctx.outputs_dir.join(&p.logs_dir).join(&p.logger_file))
        }
        3 => {
            let p = ctx.config.stage3_paths();
            Ok(ctx.outputs_dir.join(&p.logs_dir).join(&p.logger_file))
        }
        4 => {
            let p = ctx.config.stage4_claim_paths();
            Ok(ctx.outputs_dir.join(&p.logs_dir).join(&p.logger_file))
        }
        5 | 6 => tx_lane_log_file(ctx),
        7 | 8 => {
            let p = ctx.config.stage5_paths();
            Ok(ctx.outputs_dir.join(&p.logs_dir).join(&p.logger_file))
        }
        9 | 10 => {
            let p = ctx.config.stage6_paths();
            Ok(ctx.outputs_dir.join(&p.logs_dir).join(&p.logger_file))
        }
        11 => {
            let p = ctx.config.stage7_paths();
            Ok(ctx.outputs_dir.join(&p.logs_dir).join(&p.logger_file))
        }
        12 => {
            let p = ctx.config.stage8_paths();
            Ok(ctx.outputs_dir.join(&p.logs_dir).join(&p.logger_file))
        }
        13 => {
            let cfg = ctx
                .config
                .stage13_hjmt_settlement_examples
                .as_ref()
                .ok_or_else(|| "stage13_hjmt_settlement_examples config missing".to_string())?;
            let _ = cfg;
            Ok(ctx.outputs_dir.join(STAGE13_LOG_FILE))
        }
        other => Err(format!("unsupported stage for coverage: {other}")),
    }
}

fn tx_lane_log_file(ctx: &SimContext) -> Result<PathBuf, String> {
    let cfg = ctx
        .config
        .stage4_tx_prepare
        .as_ref()
        .ok_or_else(|| "stage4_tx_prepare config missing".to_string())?;
    Ok(crate::scenario_1::stage_6::resolve_stage4_paths(ctx, cfg).logger_file)
}

fn verify_claim_outputs(ctx: &SimContext) -> Result<(), String> {
    let p = ctx.config.stage3_paths();
    let outputs_dir = &ctx.outputs_dir;
    let claim = outputs_dir.join(&p.claim_dir);

    verify_claims(&claim)?;

    let snapshot = outputs_dir.join(&p.snapshot_file);
    verify_snapshot(&snapshot)?;

    #[cfg(feature = "wallet_debug_tools")]
    verify_debug_wallets(&claim, outputs_dir)?;

    Ok(())
}

fn verify_stage1_outputs(ctx: &SimContext) -> Result<(), String> {
    let p = ctx.config.stage1_paths();
    let outputs_dir = &ctx.outputs_dir;
    let genesis_dir = outputs_dir.join(&p.genesis_dir);
    let policies_path = genesis_dir.join(GENESIS_POLICIES_FILE);
    let rights_path = genesis_dir.join(GENESIS_RIGHTS_FILE);
    let manifest_path = genesis_dir.join(GENESIS_SETTLEMENT_MANIFEST_FILE);
    let vouchers_path = genesis_dir.join(GENESIS_VOUCHERS_FILE);
    let snapshot_path = outputs_dir.join(&p.snapshot_file);

    for path in [
        &policies_path,
        &rights_path,
        &manifest_path,
        &snapshot_path,
        &vouchers_path,
    ] {
        if !io::path_exists(path).map_err(|e| e.to_string())? {
            return Err(format!("missing {}", path.display()));
        }
    }

    let snapshot: Value = JsonCodec
        .deserialize(
            io::read_to_string(&snapshot_path)
                .map_err(|e| e.to_string())?
                .as_bytes(),
        )
        .map_err(|e| format!("invalid {}: {e}", p.snapshot_file))?;
    let (expected_policies, expected_corpus, expected_generation_seed_hash) =
        load_expected_stage1_packet(ctx)?;
    let policy_count = snapshot
        .get("policy_count")
        .and_then(|value| value.as_u64())
        .ok_or_else(|| "stage_1_snapshot missing policy_count".to_string())?;
    if policy_count != expected_policies.len() as u64 {
        return Err(format!(
            "stage_1_snapshot policy_count drifted: expected {}, got {}",
            expected_policies.len(),
            policy_count
        ));
    }
    let rights_count = snapshot
        .get("rights_count")
        .and_then(|value| value.as_u64())
        .ok_or_else(|| "stage_1_snapshot missing rights_count".to_string())?;
    if rights_count != ctx.genesis_rights.len() as u64 {
        return Err(format!(
            "stage_1_snapshot rights_count drifted: expected {}, got {}",
            ctx.genesis_rights.len(),
            rights_count
        ));
    }
    let voucher_count = snapshot
        .get("voucher_count")
        .and_then(|value| value.as_u64())
        .ok_or_else(|| "stage_1_snapshot missing voucher_count".to_string())?;
    if voucher_count != expected_corpus.vouchers.len() as u64 {
        return Err(format!(
            "stage_1_snapshot voucher_count drifted: expected {}, got {}",
            expected_corpus.vouchers.len(),
            voucher_count
        ));
    }
    if snapshot
        .get("policies_artifact_file")
        .and_then(|value| value.as_str())
        != Some(GENESIS_POLICIES_FILE)
    {
        return Err(
            "stage_1_snapshot policies_artifact_file drifted from canonical name".to_string(),
        );
    }
    if snapshot
        .get("rights_artifact_file")
        .and_then(|value| value.as_str())
        != Some(GENESIS_RIGHTS_FILE)
    {
        return Err(
            "stage_1_snapshot rights_artifact_file drifted from canonical name".to_string(),
        );
    }
    if snapshot
        .get("vouchers_artifact_file")
        .and_then(|value| value.as_str())
        != Some(GENESIS_VOUCHERS_FILE)
    {
        return Err(
            "stage_1_snapshot vouchers_artifact_file drifted from canonical name".to_string(),
        );
    }
    if snapshot
        .get("settlement_manifest_file")
        .and_then(|value| value.as_str())
        != Some(GENESIS_SETTLEMENT_MANIFEST_FILE)
    {
        return Err(
            "stage_1_snapshot settlement_manifest_file drifted from canonical name".to_string(),
        );
    }

    let rights_artifact: Vec<GenesisRightRecord> = JsonCodec
        .deserialize(
            io::read_to_string(&rights_path)
                .map_err(|e| e.to_string())?
                .as_bytes(),
        )
        .map_err(|e| format!("invalid {}: {e}", rights_path.display()))?;
    let policies_artifact: Vec<GenesisPolicyRecord> = JsonCodec
        .deserialize(
            io::read_to_string(&policies_path)
                .map_err(|e| e.to_string())?
                .as_bytes(),
        )
        .map_err(|e| format!("invalid {}: {e}", policies_path.display()))?;
    let vouchers_artifact: Vec<GenesisVoucherRecord> = JsonCodec
        .deserialize(
            io::read_to_string(&vouchers_path)
                .map_err(|e| e.to_string())?
                .as_bytes(),
        )
        .map_err(|e| format!("invalid {}: {e}", vouchers_path.display()))?;
    let manifest: GenesisSettlementManifest = JsonCodec
        .deserialize(
            io::read_to_string(&manifest_path)
                .map_err(|e| e.to_string())?
                .as_bytes(),
        )
        .map_err(|e| format!("invalid {}: {e}", manifest_path.display()))?;
    if policies_artifact != expected_policies {
        return Err("genesis_policies artifact drifted from configured genesis packet".to_string());
    }
    if rights_artifact != ctx.genesis_rights {
        return Err("genesis_rights artifact drifted from ctx.genesis_rights".to_string());
    }
    if rights_artifact != expected_corpus.rights {
        return Err("genesis_rights artifact drifted from configured genesis packet".to_string());
    }
    if vouchers_artifact != expected_corpus.vouchers {
        return Err("genesis_vouchers artifact drifted from configured genesis packet".to_string());
    }
    let expected_asset_ids: Vec<_> = expected_corpus
        .flatten()
        .into_iter()
        .map(|asset| asset.asset_id())
        .collect();
    let actual_asset_ids: Vec<_> = ctx.assets.iter().map(|asset| asset.asset_id()).collect();
    if actual_asset_ids != expected_asset_ids {
        return Err("ctx.assets drifted from configured genesis packet".to_string());
    }
    if ctx.genesis_rights != expected_corpus.rights {
        return Err("ctx.genesis_rights drifted from configured genesis packet".to_string());
    }
    verify_stage1_manifest_contract(
        &manifest,
        snapshot
            .get("state_hash")
            .and_then(|value| value.as_str())
            .unwrap_or_default(),
        ctx.chain_type,
        &expected_corpus,
        &policies_artifact,
        &rights_artifact,
        &vouchers_artifact,
        &expected_generation_seed_hash,
    )?;

    Ok(())
}

fn verify_stage1_manifest_contract(
    manifest: &GenesisSettlementManifest,
    snapshot_state_hash: &str,
    chain_type: z00z_core::ChainType,
    corpus: &GenesisSettlementCorpus,
    policies_artifact: &[GenesisPolicyRecord],
    rights_artifact: &[GenesisRightRecord],
    vouchers_artifact: &[GenesisVoucherRecord],
    expected_generation_seed_hash: &str,
) -> Result<(), String> {
    let expected_state_hash = hex::encode(compute_genesis_state_hash(corpus));
    let collision_report = ensure_terminal_collision_free(corpus).map_err(|e| e.to_string())?;
    let asset_definition_count = corpus
        .flatten()
        .into_iter()
        .map(|asset| asset.definition.id)
        .collect::<HashSet<_>>()
        .len();
    let right_template_count = corpus
        .rights
        .iter()
        .map(|record| record.right_id.as_str())
        .collect::<HashSet<_>>()
        .len();

    if rights_artifact != corpus.rights {
        return Err("genesis_rights artifact drifted from reconstructed corpus".to_string());
    }
    if manifest.version != 2 {
        return Err(
            "genesis_settlement_manifest version drifted from canonical version".to_string(),
        );
    }
    if manifest.policy_count != policies_artifact.len() {
        return Err(
            "genesis_settlement_manifest policy_count drifted from policy artifact".to_string(),
        );
    }
    if manifest.right_count != corpus.rights.len() {
        return Err(
            "genesis_settlement_manifest right_count drifted from reconstructed corpus".to_string(),
        );
    }
    if manifest.voucher_count != vouchers_artifact.len() {
        return Err(
            "genesis_settlement_manifest voucher_count drifted from voucher artifact".to_string(),
        );
    }
    if manifest.policies_artifact != GENESIS_POLICIES_FILE {
        return Err(
            "genesis_settlement_manifest policies_artifact drifted from canonical name".to_string(),
        );
    }
    if manifest.rights_artifact != GENESIS_RIGHTS_FILE {
        return Err(
            "genesis_settlement_manifest rights_artifact drifted from canonical name".to_string(),
        );
    }
    if manifest.vouchers_artifact != GENESIS_VOUCHERS_FILE {
        return Err(
            "genesis_settlement_manifest vouchers_artifact drifted from canonical name".to_string(),
        );
    }
    if manifest.network != chain_type.as_str() {
        return Err(
            "genesis_settlement_manifest network drifted from scenario chain type".to_string(),
        );
    }
    if manifest.root_generation != GENESIS_ROOT_GENERATION {
        return Err(
            "genesis_settlement_manifest root_generation drifted from canonical generation"
                .to_string(),
        );
    }
    if manifest.generation_seed_hash != expected_generation_seed_hash {
        return Err(
            "genesis_settlement_manifest generation_seed_hash drifted from configured genesis seed"
                .to_string(),
        );
    }
    if manifest.asset_count != corpus.total_count() {
        return Err(
            "genesis_settlement_manifest asset_count drifted from reconstructed corpus".to_string(),
        );
    }
    if manifest.asset_definition_count != asset_definition_count {
        return Err(
            "genesis_settlement_manifest asset_definition_count drifted from reconstructed corpus"
                .to_string(),
        );
    }
    if manifest.right_template_count != right_template_count {
        return Err(
            "genesis_settlement_manifest right_template_count drifted from reconstructed corpus"
                .to_string(),
        );
    }
    if snapshot_state_hash != expected_state_hash {
        return Err("stage_1_snapshot state_hash drifted from reconstructed corpus".to_string());
    }
    if manifest.state_hash != expected_state_hash {
        return Err(
            "genesis_settlement_manifest state_hash drifted from reconstructed corpus".to_string(),
        );
    }
    if manifest.corpus_digest != expected_state_hash {
        return Err(
            "genesis_settlement_manifest corpus_digest drifted from reconstructed corpus"
                .to_string(),
        );
    }
    if manifest.leaf_count != corpus.total_leaf_count() {
        return Err(
            "genesis_settlement_manifest leaf_count drifted from reconstructed corpus".to_string(),
        );
    }
    if manifest.terminal_collision_checks != collision_report {
        return Err(
            "genesis_settlement_manifest terminal_collision_checks drifted from reconstructed corpus"
                .to_string(),
        );
    }
    let policies_replay_digest = hex::encode(compute_genesis_policies_digest(
        policies_artifact,
        GENESIS_POLICIES_REPLAY_DIGEST_LABEL,
    ));
    if manifest.policies_replay_digest != policies_replay_digest {
        return Err(
            "genesis_settlement_manifest policies_replay_digest drifted from policy artifact"
                .to_string(),
        );
    }
    let policies_roundtrip_digest = hex::encode(compute_genesis_policies_digest(
        policies_artifact,
        GENESIS_POLICIES_ROUNDTRIP_DIGEST_LABEL,
    ));
    if manifest.policies_output_roundtrip_digest != policies_roundtrip_digest {
        return Err(
            "genesis_settlement_manifest policies_output_roundtrip_digest drifted from policy artifact"
                .to_string(),
        );
    }
    let replay_digest = hex::encode(compute_genesis_rights_digest(
        &corpus.rights,
        GENESIS_RIGHTS_REPLAY_DIGEST_LABEL,
    ));
    if manifest.deterministic_replay_digest != replay_digest {
        return Err(
            "genesis_settlement_manifest deterministic_replay_digest drifted from rights corpus"
                .to_string(),
        );
    }
    let roundtrip_digest = hex::encode(compute_genesis_rights_digest(
        rights_artifact,
        GENESIS_RIGHTS_ROUNDTRIP_DIGEST_LABEL,
    ));
    if manifest.rights_output_roundtrip_digest != roundtrip_digest {
        return Err(
            "genesis_settlement_manifest rights_output_roundtrip_digest drifted from rights artifact"
                .to_string(),
        );
    }
    let vouchers_replay_digest = hex::encode(compute_genesis_vouchers_digest(
        vouchers_artifact,
        GENESIS_VOUCHERS_REPLAY_DIGEST_LABEL,
    ));
    if manifest.vouchers_replay_digest != vouchers_replay_digest {
        return Err(
            "genesis_settlement_manifest vouchers_replay_digest drifted from voucher artifact"
                .to_string(),
        );
    }
    let vouchers_roundtrip_digest = hex::encode(compute_genesis_vouchers_digest(
        vouchers_artifact,
        GENESIS_VOUCHERS_ROUNDTRIP_DIGEST_LABEL,
    ));
    if manifest.vouchers_output_roundtrip_digest != vouchers_roundtrip_digest {
        return Err(
            "genesis_settlement_manifest vouchers_output_roundtrip_digest drifted from voucher artifact"
                .to_string(),
        );
    }
    let manifest_hash = hex::encode(compute_genesis_manifest_hash(manifest));
    if manifest.manifest_hash != manifest_hash {
        return Err("genesis_settlement_manifest manifest_hash failed self-check".to_string());
    }

    Ok(())
}

fn load_expected_stage1_packet(
    ctx: &SimContext,
) -> Result<(Vec<GenesisPolicyRecord>, GenesisSettlementCorpus, String), String> {
    let p = ctx.config.stage1_paths();
    let configured = ctx.config.stage1_genesis_config();
    let direct = PathBuf::from(&configured);
    let cfg_path = if configured.is_empty() {
        PathBuf::from(&p.fallback_genesis_dir).join(ctx.config.stage1_genesis_config())
    } else if direct.exists() {
        direct
    } else {
        PathBuf::from(&p.fallback_genesis_dir).join(configured)
    };
    let cfg_path = cfg_path.to_string_lossy().to_string();
    let cfg = load_genesis_config(&cfg_path).map_err(|e| e.to_string())?;
    if cfg.chain.chain_type != ctx.chain_type.as_str() {
        return Err(format!(
            "stage1 genesis config chain_type drifted from scenario chain: expected {}, got {}",
            ctx.chain_type.as_str(),
            cfg.chain.chain_type
        ));
    }
    let seed = GenesisSeed::from_config(&cfg).map_err(|e| e.to_string())?;
    let definitions = cfg
        .assets
        .iter()
        .map(|asset| create_asset_definition(asset, seed.as_bytes(), ctx.chain_type))
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;
    let policies =
        generate_genesis_policies(&cfg.assets, &cfg.policies).map_err(|e| e.to_string())?;
    let corpus = generate_genesis_settlement_corpus(
        &definitions,
        &cfg.rights,
        &cfg.vouchers,
        &policies,
        seed.as_bytes(),
        cfg.chain.id,
        ctx.chain_type,
        Arc::new(NoopLogger),
        Arc::new(NoopMetrics),
    )
    .map_err(|e| e.to_string())?;
    Ok((
        policies,
        corpus,
        hex::encode(compute_genesis_seed_hash(seed.as_bytes())),
    ))
}

#[cfg(test)]
mod manifest_tests {
    use super::*;
    use z00z_core::genesis::{
        TerminalCollisionReport, GENESIS_POLICIES_FILE, GENESIS_RIGHTS_FILE, GENESIS_VOUCHERS_FILE,
    };

    fn sample_manifest(
        corpus: &GenesisSettlementCorpus,
        generation_seed_hash: &str,
    ) -> GenesisSettlementManifest {
        let state_hash = hex::encode(compute_genesis_state_hash(corpus));
        let policies = Vec::<GenesisPolicyRecord>::new();
        let vouchers = Vec::<GenesisVoucherRecord>::new();
        let mut manifest = GenesisSettlementManifest {
            version: 2,
            network: z00z_core::ChainType::Devnet.as_str().to_string(),
            asset_definition_count: 0,
            asset_count: 0,
            policy_count: policies.len(),
            right_template_count: 0,
            right_count: 0,
            voucher_count: vouchers.len(),
            leaf_count: 0,
            root_generation: GENESIS_ROOT_GENERATION,
            generation_seed_hash: generation_seed_hash.to_string(),
            corpus_digest: state_hash.clone(),
            state_hash,
            policies_replay_digest: hex::encode(compute_genesis_policies_digest(
                &policies,
                GENESIS_POLICIES_REPLAY_DIGEST_LABEL,
            )),
            policies_output_roundtrip_digest: hex::encode(compute_genesis_policies_digest(
                &policies,
                GENESIS_POLICIES_ROUNDTRIP_DIGEST_LABEL,
            )),
            deterministic_replay_digest: hex::encode(compute_genesis_rights_digest(
                &corpus.rights,
                GENESIS_RIGHTS_REPLAY_DIGEST_LABEL,
            )),
            rights_output_roundtrip_digest: hex::encode(compute_genesis_rights_digest(
                &corpus.rights,
                GENESIS_RIGHTS_ROUNDTRIP_DIGEST_LABEL,
            )),
            vouchers_replay_digest: hex::encode(compute_genesis_vouchers_digest(
                &vouchers,
                GENESIS_VOUCHERS_REPLAY_DIGEST_LABEL,
            )),
            vouchers_output_roundtrip_digest: hex::encode(compute_genesis_vouchers_digest(
                &vouchers,
                GENESIS_VOUCHERS_ROUNDTRIP_DIGEST_LABEL,
            )),
            terminal_collision_checks: TerminalCollisionReport::default(),
            policies_artifact: GENESIS_POLICIES_FILE.to_string(),
            rights_artifact: GENESIS_RIGHTS_FILE.to_string(),
            vouchers_artifact: GENESIS_VOUCHERS_FILE.to_string(),
            manifest_hash: String::new(),
        };
        manifest.manifest_hash = hex::encode(compute_genesis_manifest_hash(&manifest));
        manifest
    }

    #[test]
    fn stage1_manifest_rejects_state_tamper() {
        let corpus = GenesisSettlementCorpus::new();
        let expected_generation_seed_hash = hex::encode(compute_genesis_seed_hash(&[0x11; 32]));
        let mut manifest = sample_manifest(&corpus, &expected_generation_seed_hash);
        manifest.state_hash = hex::encode([0x55; 32]);
        manifest.manifest_hash = hex::encode(compute_genesis_manifest_hash(&manifest));

        let err = verify_stage1_manifest_contract(
            &manifest,
            &manifest.state_hash,
            z00z_core::ChainType::Devnet,
            &corpus,
            &[],
            &corpus.rights,
            &[],
            &expected_generation_seed_hash,
        )
        .expect_err("tampered state_hash must fail");

        assert!(err.contains("state_hash drifted from reconstructed corpus"));
    }

    #[test]
    fn stage1_manifest_rejects_bad_rootgen() {
        let corpus = GenesisSettlementCorpus::new();
        let expected_generation_seed_hash = hex::encode(compute_genesis_seed_hash(&[0x11; 32]));
        let mut manifest = sample_manifest(&corpus, &expected_generation_seed_hash);
        manifest.root_generation += 1;
        manifest.manifest_hash = hex::encode(compute_genesis_manifest_hash(&manifest));

        let err = verify_stage1_manifest_contract(
            &manifest,
            &manifest.state_hash,
            z00z_core::ChainType::Devnet,
            &corpus,
            &[],
            &corpus.rights,
            &[],
            &expected_generation_seed_hash,
        )
        .expect_err("wrong root_generation must fail");

        assert!(err.contains("root_generation drifted from canonical generation"));
    }

    #[test]
    fn stage1_manifest_rejects_bad_seedhash() {
        let corpus = GenesisSettlementCorpus::new();
        let expected_generation_seed_hash = hex::encode(compute_genesis_seed_hash(&[0x11; 32]));
        let mut manifest = sample_manifest(&corpus, &expected_generation_seed_hash);
        manifest.generation_seed_hash = hex::encode([0x55; 32]);
        manifest.manifest_hash = hex::encode(compute_genesis_manifest_hash(&manifest));

        let err = verify_stage1_manifest_contract(
            &manifest,
            &manifest.state_hash,
            z00z_core::ChainType::Devnet,
            &corpus,
            &[],
            &corpus.rights,
            &[],
            &expected_generation_seed_hash,
        )
        .expect_err("wrong generation_seed_hash must fail");

        assert!(err.contains("generation_seed_hash drifted from configured genesis seed"));
    }

    #[test]
    fn stage1_manifest_rejects_bad_class() {
        let expected_generation_seed_hash = hex::encode(compute_genesis_seed_hash(&[0x11; 32]));
        let mut corpus = GenesisSettlementCorpus::new();
        corpus.rights.push(GenesisRightRecord {
            right_id: "service_entitlement".to_string(),
            right_index: 0,
            definition_id: [0x21; 32],
            serial_id: 0,
            domain_name: "rights.test".to_string(),
            metadata_purpose: "create, transfer, revoke".to_string(),
            leaf: z00z_core::genesis::GenesisRightLeaf {
                version: 1,
                terminal_id: [1u8; 32],
                right_class: z00z_core::rights::RightClassConfig::ServiceEntitlement,
                issuer_scope: [2u8; 32],
                provider_scope: [3u8; 32],
                holder_commitment: [4u8; 32],
                control_commitment: [5u8; 32],
                beneficiary_commitment: [6u8; 32],
                payload_commitment: [7u8; 32],
                valid_from: 0,
                valid_until: 10,
                challenge_from: 0,
                challenge_until: 0,
                use_nonce: [8u8; 32],
                revocation_policy_id: [9u8; 32],
                transition_policy_id: [10u8; 32],
                challenge_policy_id: [11u8; 32],
                disclosure_policy_id: [12u8; 32],
                retention_policy_id: [13u8; 32],
            },
        });
        let manifest = sample_manifest(&corpus, &expected_generation_seed_hash);
        let mut rights_artifact = corpus.rights.clone();
        rights_artifact[0].leaf.right_class = z00z_core::rights::RightClassConfig::DataAccess;

        let err = verify_stage1_manifest_contract(
            &manifest,
            &manifest.state_hash,
            z00z_core::ChainType::Devnet,
            &corpus,
            &[],
            &rights_artifact,
            &[],
            &expected_generation_seed_hash,
        )
        .expect_err("wrong right class must fail");

        assert!(err.contains("artifact drifted from reconstructed corpus"));
    }

    #[test]
    fn stage1_manifest_rejects_bad_control() {
        let expected_generation_seed_hash = hex::encode(compute_genesis_seed_hash(&[0x11; 32]));
        let mut corpus = GenesisSettlementCorpus::new();
        corpus.rights.push(GenesisRightRecord {
            right_id: "service_entitlement".to_string(),
            right_index: 0,
            definition_id: [0x21; 32],
            serial_id: 0,
            domain_name: "rights.test".to_string(),
            metadata_purpose: "create, transfer, revoke".to_string(),
            leaf: z00z_core::genesis::GenesisRightLeaf {
                version: 1,
                terminal_id: [1u8; 32],
                right_class: z00z_core::rights::RightClassConfig::ServiceEntitlement,
                issuer_scope: [2u8; 32],
                provider_scope: [3u8; 32],
                holder_commitment: [4u8; 32],
                control_commitment: [5u8; 32],
                beneficiary_commitment: [6u8; 32],
                payload_commitment: [7u8; 32],
                valid_from: 0,
                valid_until: 10,
                challenge_from: 0,
                challenge_until: 0,
                use_nonce: [8u8; 32],
                revocation_policy_id: [9u8; 32],
                transition_policy_id: [10u8; 32],
                challenge_policy_id: [11u8; 32],
                disclosure_policy_id: [12u8; 32],
                retention_policy_id: [13u8; 32],
            },
        });
        let manifest = sample_manifest(&corpus, &expected_generation_seed_hash);
        let mut rights_artifact = corpus.rights.clone();
        rights_artifact[0].leaf.control_commitment = [0xAA; 32];

        let err = verify_stage1_manifest_contract(
            &manifest,
            &manifest.state_hash,
            z00z_core::ChainType::Devnet,
            &corpus,
            &[],
            &rights_artifact,
            &[],
            &expected_generation_seed_hash,
        )
        .expect_err("wrong control binding must fail");

        assert!(err.contains("artifact drifted from reconstructed corpus"));
    }

    #[test]
    fn stage1_manifest_rejects_bad_holder() {
        let expected_generation_seed_hash = hex::encode(compute_genesis_seed_hash(&[0x11; 32]));
        let mut corpus = GenesisSettlementCorpus::new();
        corpus.rights.push(GenesisRightRecord {
            right_id: "service_entitlement".to_string(),
            right_index: 0,
            definition_id: [0x21; 32],
            serial_id: 0,
            domain_name: "rights.test".to_string(),
            metadata_purpose: "create, transfer, revoke".to_string(),
            leaf: z00z_core::genesis::GenesisRightLeaf {
                version: 1,
                terminal_id: [1u8; 32],
                right_class: z00z_core::rights::RightClassConfig::ServiceEntitlement,
                issuer_scope: [2u8; 32],
                provider_scope: [3u8; 32],
                holder_commitment: [4u8; 32],
                control_commitment: [5u8; 32],
                beneficiary_commitment: [6u8; 32],
                payload_commitment: [7u8; 32],
                valid_from: 0,
                valid_until: 10,
                challenge_from: 0,
                challenge_until: 0,
                use_nonce: [8u8; 32],
                revocation_policy_id: [9u8; 32],
                transition_policy_id: [10u8; 32],
                challenge_policy_id: [11u8; 32],
                disclosure_policy_id: [12u8; 32],
                retention_policy_id: [13u8; 32],
            },
        });
        let manifest = sample_manifest(&corpus, &expected_generation_seed_hash);
        let mut rights_artifact = corpus.rights.clone();
        rights_artifact[0].leaf.holder_commitment = [0xAA; 32];

        let err = verify_stage1_manifest_contract(
            &manifest,
            &manifest.state_hash,
            z00z_core::ChainType::Devnet,
            &corpus,
            &[],
            &rights_artifact,
            &[],
            &expected_generation_seed_hash,
        )
        .expect_err("wrong holder binding must fail");

        assert!(err.contains("artifact drifted from reconstructed corpus"));
    }
}

fn verify_claim_publish_outputs(ctx: &SimContext) -> Result<(), String> {
    let p = ctx.config.stage4_claim_paths();
    let outputs_dir = &ctx.outputs_dir;
    let publish = outputs_dir.join(&p.claim_dir);
    let audit_log = publish.join("audit_log.json");

    if !io::path_exists(&audit_log).map_err(|e| e.to_string())? {
        return Err(format!("missing {}", audit_log.display()));
    }

    let snapshot = outputs_dir.join(&p.snapshot_file);
    if !io::path_exists(&snapshot).map_err(|e| e.to_string())? {
        return Err(format!("missing {}", snapshot.display()));
    }

    verify_claim_publish_snapshot(ctx, &snapshot)
}

fn verify_claim_publish_snapshot(ctx: &SimContext, snapshot: &Path) -> Result<(), String> {
    let snap: Value = JsonCodec
        .deserialize(
            io::read_to_string(snapshot)
                .map_err(|e| e.to_string())?
                .as_bytes(),
        )
        .map_err(|e| format!("invalid stage_4_snapshot.json: {e}"))?;

    if snap.get("source_stage").and_then(|v| v.as_u64()) != Some(3) {
        return Err("stage_4_snapshot must reference stage 3 source".to_string());
    }

    let wallet_stats = snap
        .get("wallet_import_stats")
        .and_then(|v| v.as_u64())
        .ok_or_else(|| {
            "stage_4_snapshot wallet_import_stats must be a positive integer".to_string()
        })?;

    if wallet_stats == 0 {
        return Err("stage_4_snapshot wallet_import_stats must be > 0".to_string());
    }

    if snap
        .get("genesis_rights_included")
        .and_then(|value| value.as_bool())
        != Some(true)
    {
        return Err("stage_4_snapshot must declare genesis_rights_included=true".to_string());
    }
    if snap
        .get("genesis_rights_count")
        .and_then(|value| value.as_u64())
        != Some(ctx.genesis_rights.len() as u64)
    {
        return Err(
            "stage_4_snapshot genesis_rights_count drifted from ctx.genesis_rights".to_string(),
        );
    }
    if snap
        .get("rights_artifact_file")
        .and_then(|value| value.as_str())
        != Some(GENESIS_RIGHTS_FILE)
    {
        return Err(
            "stage_4_snapshot rights_artifact_file drifted from canonical name".to_string(),
        );
    }
    if snap
        .get("settlement_manifest_file")
        .and_then(|value| value.as_str())
        != Some(GENESIS_SETTLEMENT_MANIFEST_FILE)
    {
        return Err(
            "stage_4_snapshot settlement_manifest_file drifted from canonical name".to_string(),
        );
    }

    Ok(())
}

fn verify_stage11_right_rejection(ctx: &SimContext) -> Result<(), String> {
    let scan_path = ctx
        .config
        .runtime_observability_ref()
        .ok_or_else(|| "runtime_observability config missing".to_string())?
        .packet
        .wallet_scan_file
        .clone();
    let scan_path = ctx.outputs_dir.join(scan_path);
    if !io::path_exists(&scan_path).map_err(|e| e.to_string())? {
        return Err(format!("missing {}", scan_path.display()));
    }

    let scan: Value = JsonCodec
        .deserialize(
            io::read_to_string(&scan_path)
                .map_err(|e| e.to_string())?
                .as_bytes(),
        )
        .map_err(|e| format!("invalid {}: {e}", scan_path.display()))?;
    let skipped = scan
        .get("skipped_non_asset_count")
        .and_then(|value| value.as_u64())
        .ok_or_else(|| "wallet_scan missing skipped_non_asset_count".to_string())?;
    if skipped < ctx.genesis_rights.len() as u64 {
        return Err(format!(
            "wallet_scan skipped_non_asset_count must be at least {}, got {}",
            ctx.genesis_rights.len(),
            skipped
        ));
    }
    let proof_step = scan
        .get("proof_step")
        .and_then(|value| value.as_str())
        .ok_or_else(|| "wallet_scan missing proof_step".to_string())?;
    if !proof_step.contains("before runtime ownership detection") {
        return Err("wallet_scan proof_step lost the pre-ownership rejection boundary".to_string());
    }

    Ok(())
}

fn verify_stage13_contract(ctx: &SimContext, stage: &DesignStage) -> Result<(), String> {
    let cfg = ctx
        .config
        .stage13_hjmt_settlement_examples
        .as_ref()
        .ok_or_else(|| "stage13_hjmt_settlement_examples config missing".to_string())?;

    let examples_path = ctx.outputs_dir.join(&cfg.examples_file);
    let tamper_path = ctx.outputs_dir.join(&cfg.tamper_report_file);
    let proof_size_path = ctx.outputs_dir.join(&cfg.proof_size_report_file);
    let metrics_path = ctx.outputs_dir.join(&cfg.cache_scheduler_metrics_file);
    let replay_path = ctx.outputs_dir.join(&cfg.replay_roots_file);
    let manifest_path = ctx
        .outputs_dir
        .join(&cfg.output_dir)
        .join("genesis_settlement_manifest.json");
    let store_root = ctx.outputs_dir.join(&cfg.output_dir).join("store");
    let store_file = store_root.join("settlement_state.redb");

    for path in [
        &examples_path,
        &tamper_path,
        &proof_size_path,
        &metrics_path,
        &replay_path,
        &manifest_path,
        &store_file,
    ] {
        if !io::path_exists(path).map_err(|e| e.to_string())? {
            return Err(format!("missing {}", path.display()));
        }
    }

    let examples_text = io::read_to_string(&examples_path)
        .map_err(|e| format!("read {} failed: {e}", examples_path.display()))?;
    let examples_raw: Value = JsonCodec
        .deserialize(examples_text.as_bytes())
        .map_err(|e| format!("invalid stage13 examples report: {e}"))?;
    verify_stage13_examples_schema(&examples_raw)?;
    let examples: Stage13ExamplesReport = JsonCodec
        .serialize(&examples_raw)
        .and_then(|bytes| JsonCodec.deserialize(&bytes))
        .map_err(|e| format!("invalid stage13 examples report: {e}"))?;
    let metrics: Stage13CacheSchedulerReport = JsonCodec
        .deserialize(
            io::read_to_string(&metrics_path)
                .map_err(|e| format!("read {} failed: {e}", metrics_path.display()))?
                .as_bytes(),
        )
        .map_err(|e| format!("invalid stage13 metrics report: {e}"))?;
    let replay: Stage13ReplayRootsReport = JsonCodec
        .deserialize(
            io::read_to_string(&replay_path)
                .map_err(|e| format!("read {} failed: {e}", replay_path.display()))?
                .as_bytes(),
        )
        .map_err(|e| format!("invalid stage13 replay report: {e}"))?;
    let proof_sizes: Stage13ProofSizeReport = JsonCodec
        .deserialize(
            io::read_to_string(&proof_size_path)
                .map_err(|e| format!("read {} failed: {e}", proof_size_path.display()))?
                .as_bytes(),
        )
        .map_err(|e| format!("invalid stage13 proof-size report: {e}"))?;
    let tamper: Stage13TamperReport = JsonCodec
        .deserialize(
            io::read_to_string(&tamper_path)
                .map_err(|e| format!("read {} failed: {e}", tamper_path.display()))?
                .as_bytes(),
        )
        .map_err(|e| format!("invalid stage13 tamper report: {e}"))?;

    if examples.status != STAGE13_STATUS {
        return Err(format!(
            "stage13 examples report must stay at {STAGE13_STATUS} status"
        ));
    }
    if examples.stage != stage.stage {
        return Err("stage13 examples report stage drifted from design".to_string());
    }
    if examples.boundary_mode != STAGE13_MODE {
        return Err(format!(
            "stage13 examples report must declare {STAGE13_MODE} boundary mode"
        ));
    }
    if examples.scenario_id != ctx.config.scenario.id {
        return Err("stage13 examples report scenario_id drifted from config".to_string());
    }
    if examples.backend_modes != cfg.backend_modes {
        return Err("stage13 backend_modes drifted from config".to_string());
    }
    verify_stage13_artifact_meta("stage13 examples report", &examples.artifact)?;
    if examples.manifest_file != "hjmt/genesis_settlement_manifest.json" {
        return Err("stage13 manifest linkage drifted from deterministic output path".to_string());
    }
    if examples.artifact_names
        != STAGE13_REQUIRED_ARTIFACTS
            .iter()
            .map(|value| (*value).to_string())
            .collect::<Vec<_>>()
    {
        return Err("stage13 artifact_names drifted from required set".to_string());
    }

    let actual_example_ids = examples
        .examples
        .iter()
        .map(|example| example.example_id.clone())
        .collect::<BTreeSet<_>>();
    let expected_example_ids = STAGE13_EXPECTED_EXAMPLES
        .iter()
        .map(|value| (*value).to_string())
        .collect::<BTreeSet<_>>();
    if actual_example_ids != expected_example_ids {
        return Err("stage13 example ids drifted from required set".to_string());
    }

    let store = SettlementStore::load(&store_root)
        .map_err(|e| format!("load {} failed: {e}", store_root.display()))?;
    let live_state_root_hex = crate::scenario_1::stage_4::describe_store_roots(&store)
        .map_err(|e| format!("describe roots for {} failed: {e}", store_root.display()))?
        .state_root_hex;
    if live_state_root_hex != examples.settlement_state_root_hex {
        return Err("stage13 examples report settlement root drifted from live store".to_string());
    }

    for example in &examples.examples {
        if example.schema_version != examples.schema_version {
            return Err(format!(
                "stage13 example {} schema_version drifted from report",
                example.example_id
            ));
        }
        if example.scenario_id != ctx.config.scenario.id || example.stage != stage.stage {
            return Err(format!(
                "stage13 example {} scenario_id/stage drifted",
                example.example_id
            ));
        }
        if example.verifier_status != "verified" {
            return Err(format!(
                "stage13 example {} must stay verified",
                example.example_id
            ));
        }
        if example.typed_error.is_some() {
            return Err(format!(
                "stage13 example {} typed_error must stay empty on verified artifacts",
                example.example_id
            ));
        }
        if example.root_generation != examples.root_generation {
            return Err(format!(
                "stage13 example {} root_generation drifted",
                example.example_id
            ));
        }
        if example.settlement_state_root_hex != examples.settlement_state_root_hex {
            return Err(format!(
                "stage13 example {} settlement_state_root_hex drifted",
                example.example_id
            ));
        }
        if !cfg.backend_modes.contains(&example.backend_mode) {
            return Err(format!(
                "stage13 example {} backend_mode drifted from config",
                example.example_id
            ));
        }
        if example.artifact_names != examples.artifact_names {
            return Err(format!(
                "stage13 example {} artifact_names drifted from report",
                example.example_id
            ));
        }
        if example.api_surface.trim().is_empty() {
            return Err(format!(
                "stage13 example {} api_surface must not be empty",
                example.example_id
            ));
        }
        if let Some(reject) = example.present_key_rejection.as_deref() {
            if let Some(kind) = redaction_violation(reject) {
                return Err(format!(
                    "stage13 example {} present_key_rejection violated redaction: {kind}",
                    example.example_id
                ));
            }
        }
        hjmt_examples::verify_example_artifact(&store, example, &metrics).map_err(|err| {
            format!(
                "stage13 reload verification failed for {}: {err}",
                example.example_id
            )
        })?;
    }
    if examples.comparison_rows.is_empty() {
        return Err("stage13 comparison_rows must not be empty".to_string());
    }
    let actual_surfaces = examples
        .comparison_rows
        .iter()
        .map(|row| row.proof_surface.clone())
        .collect::<BTreeSet<_>>();
    let expected_surfaces = [PROOF_SURFACE_SINGLE, PROOF_SURFACE_VEC, PROOF_SURFACE_BATCH]
        .into_iter()
        .map(str::to_string)
        .collect::<BTreeSet<_>>();
    if actual_surfaces != expected_surfaces {
        return Err("stage13 comparison proof_surface set drifted".to_string());
    }
    let batch_rows = examples
        .comparison_rows
        .iter()
        .filter(|row| row.proof_surface == PROOF_SURFACE_BATCH)
        .collect::<Vec<_>>();
    if batch_rows.is_empty() {
        return Err("stage13 comparison rows lost batch evidence".to_string());
    }
    let batch_shapes = batch_rows
        .iter()
        .map(|row| row.path_shape.clone())
        .collect::<BTreeSet<_>>();
    let expected_batch_shapes = [PATH_SHAPE_CLUSTERED, PATH_SHAPE_SCATTERED]
        .into_iter()
        .map(str::to_string)
        .collect::<BTreeSet<_>>();
    if batch_shapes != expected_batch_shapes {
        return Err("stage13 batch path_shape coverage drifted".to_string());
    }
    let batch_counts = batch_rows
        .iter()
        .map(|row| row.path_count)
        .collect::<BTreeSet<_>>();
    let expected_batch_counts = [2u32, 8u32, 32u32].into_iter().collect::<BTreeSet<_>>();
    if !expected_batch_counts.is_subset(&batch_counts) {
        return Err("stage13 batch path_count coverage drifted".to_string());
    }
    let batch_families = batch_rows
        .iter()
        .map(|row| row.proof_family.clone())
        .collect::<BTreeSet<_>>();
    let expected_batch_families = ["inclusion", "deletion", "nonexistence"]
        .into_iter()
        .map(str::to_string)
        .collect::<BTreeSet<_>>();
    if !expected_batch_families.is_subset(&batch_families) {
        return Err("stage13 batch proof_family coverage drifted".to_string());
    }
    let single_families = examples
        .comparison_rows
        .iter()
        .filter(|row| row.proof_surface == PROOF_SURFACE_SINGLE)
        .map(|row| row.proof_family.clone())
        .collect::<BTreeSet<_>>();
    if single_families != expected_batch_families {
        return Err("stage13 single-proof reference coverage drifted".to_string());
    }
    for row in &examples.comparison_rows {
        if row.schema_version != examples.schema_version
            || row.scenario_id != ctx.config.scenario.id
            || row.stage != stage.stage
        {
            return Err(format!(
                "stage13 comparison row {} stage/scenario drifted",
                row.row_id
            ));
        }
        if !expected_example_ids.contains(&row.owner_example_id) {
            return Err(format!(
                "stage13 comparison row {} owner_example_id drifted",
                row.row_id
            ));
        }
        if row.root_generation != examples.root_generation
            || row.settlement_state_root_hex != examples.settlement_state_root_hex
        {
            return Err(format!(
                "stage13 comparison row {} root binding drifted",
                row.row_id
            ));
        }
        if row.proof_surface == PROOF_SURFACE_BATCH {
            if row.atomic_verdict != ATOMIC_VERDICT_ACCEPTED {
                return Err(format!(
                    "stage13 comparison row {} atomic verdict drifted",
                    row.row_id
                ));
            }
            if row.shard_context_mode != SHARD_CONTEXT_NONE {
                return Err(format!(
                    "stage13 comparison row {} shard_context_mode drifted",
                    row.row_id
                ));
            }
            if row.path_count <= 1 {
                return Err(format!(
                    "stage13 comparison row {} lost multi-path batch evidence",
                    row.row_id
                ));
            }
        }
        verify_stage13_comparison_row(row, &store)?;
    }

    if proof_sizes.status != STAGE13_STATUS {
        return Err("stage13 proof-size report status drifted".to_string());
    }
    if proof_sizes.root_generation != examples.root_generation {
        return Err("stage13 proof-size report root_generation drifted".to_string());
    }
    verify_stage13_artifact_meta("stage13 proof-size report", &proof_sizes.artifact)?;
    if proof_sizes.entries.is_empty() {
        return Err("stage13 proof-size report must not be empty".to_string());
    }
    let expected_proof_ids = examples
        .examples
        .iter()
        .filter(|example| example.proof_size_bytes.is_some() && example.verify_time_us.is_some())
        .map(|example| example.example_id.clone())
        .collect::<BTreeSet<_>>();
    let actual_proof_ids = proof_sizes
        .entries
        .iter()
        .map(|entry| entry.example_id.clone())
        .collect::<BTreeSet<_>>();
    if proof_sizes.entries.len() != expected_proof_ids.len()
        || actual_proof_ids != expected_proof_ids
    {
        return Err("stage13 proof-size coverage drifted from examples".to_string());
    }
    for entry in &proof_sizes.entries {
        let example = examples
            .examples
            .iter()
            .find(|example| example.example_id == entry.example_id)
            .ok_or_else(|| {
                format!(
                    "stage13 proof-size entry {} missing example",
                    entry.example_id
                )
            })?;
        if entry.stage != stage.stage || entry.scenario_id != ctx.config.scenario.id {
            return Err("stage13 proof-size entry stage/scenario drifted".to_string());
        }
        if entry.typed_error.is_some() {
            return Err(format!(
                "stage13 proof-size entry {} typed_error must stay empty",
                entry.example_id
            ));
        }
        if entry.backend_mode != example.backend_mode || entry.api_surface != example.api_surface {
            return Err(format!(
                "stage13 proof-size entry {} api binding drifted",
                entry.example_id
            ));
        }
        if entry.root_generation != example.root_generation {
            return Err(format!(
                "stage13 proof-size entry {} root_generation drifted",
                entry.example_id
            ));
        }
        if entry.verifier_status != "verified" {
            return Err("stage13 proof-size entry must stay verified".to_string());
        }
        if entry.proof_size_bytes == 0 || entry.verify_time_us == 0 {
            return Err("stage13 proof-size entry must stay non-zero".to_string());
        }
        if example.proof_size_bytes != Some(entry.proof_size_bytes)
            || example.verify_time_us != Some(entry.verify_time_us)
        {
            return Err(format!(
                "stage13 proof-size entry {} drifted from example metrics",
                entry.example_id
            ));
        }
    }
    let example_comparison_ids = examples
        .comparison_rows
        .iter()
        .map(|row| row.row_id.clone())
        .collect::<BTreeSet<_>>();
    let proof_comparison_ids = proof_sizes
        .comparison_rows
        .iter()
        .map(|row| row.row_id.clone())
        .collect::<BTreeSet<_>>();
    if proof_comparison_ids != example_comparison_ids
        || proof_sizes.comparison_rows.len() != examples.comparison_rows.len()
    {
        return Err("stage13 proof-size comparison coverage drifted".to_string());
    }
    for row in &proof_sizes.comparison_rows {
        let example_row = examples
            .comparison_rows
            .iter()
            .find(|candidate| candidate.row_id == row.row_id)
            .ok_or_else(|| {
                format!(
                    "stage13 proof-size comparison row {} missing example row",
                    row.row_id
                )
            })?;
        if row != example_row {
            return Err(format!(
                "stage13 proof-size comparison row {} drifted from examples report",
                row.row_id
            ));
        }
    }

    if metrics.verifier_status != "verified" {
        return Err("stage13 cache/scheduler metrics must stay verified".to_string());
    }
    if metrics.typed_error.is_some() {
        return Err("stage13 cache/scheduler metrics typed_error must stay empty".to_string());
    }
    if metrics.stage != stage.stage || metrics.scenario_id != ctx.config.scenario.id {
        return Err("stage13 metrics stage/scenario drifted".to_string());
    }
    if metrics.example_id != "E8_cache_scheduler" {
        return Err("stage13 metrics example_id drifted".to_string());
    }
    if !metrics.deterministic_parent_ordering {
        return Err("stage13 metrics lost deterministic parent ordering".to_string());
    }
    if metrics.root_generation != examples.root_generation
        || metrics.settlement_state_root_hex != examples.settlement_state_root_hex
    {
        return Err("stage13 metrics root binding drifted".to_string());
    }
    metrics.validate_bounded()?;

    if replay.status != STAGE13_STATUS {
        return Err("stage13 replay report status drifted".to_string());
    }
    if replay.root_generation != examples.root_generation {
        return Err("stage13 replay report root_generation drifted".to_string());
    }
    verify_stage13_artifact_meta("stage13 replay report", &replay.artifact)?;
    if replay.store_dir != format!("{}/store", cfg.output_dir) {
        return Err("stage13 replay store_dir drifted from deterministic output path".to_string());
    }
    if replay.replay_entries.len() != examples.examples.len() {
        return Err("stage13 replay entry count drifted from examples".to_string());
    }
    let actual_replay_ids = replay
        .replay_entries
        .iter()
        .map(|entry| entry.example_id.clone())
        .collect::<BTreeSet<_>>();
    if actual_replay_ids != expected_example_ids {
        return Err("stage13 replay entry coverage drifted from examples".to_string());
    }
    for entry in &replay.replay_entries {
        let example = examples
            .examples
            .iter()
            .find(|example| example.example_id == entry.example_id)
            .ok_or_else(|| format!("stage13 replay entry {} missing example", entry.example_id))?;
        if entry.stage != stage.stage || entry.scenario_id != ctx.config.scenario.id {
            return Err(format!(
                "stage13 replay entry {} stage/scenario drifted",
                entry.example_id
            ));
        }
        if entry.typed_error.is_some() {
            return Err(format!(
                "stage13 replay entry {} typed_error must stay empty",
                entry.example_id
            ));
        }
        if entry.verifier_status != "verified" {
            return Err(format!(
                "stage13 replay entry {} must stay verified",
                entry.example_id
            ));
        }
        if entry.root_generation != example.root_generation
            || entry.backend_mode != example.backend_mode
            || entry.api_surface != example.api_surface
        {
            return Err(format!(
                "stage13 replay entry {} binding drifted",
                entry.example_id
            ));
        }
        if entry.reloaded_settlement_state_root_hex != examples.settlement_state_root_hex
            || entry.settlement_state_root_hex != examples.settlement_state_root_hex
        {
            return Err(format!(
                "stage13 replay entry {} root binding drifted",
                entry.example_id
            ));
        }
    }

    if tamper.status != STAGE13_STATUS {
        return Err("stage13 tamper report status drifted".to_string());
    }
    if tamper.root_generation != examples.root_generation {
        return Err("stage13 tamper report root_generation drifted".to_string());
    }
    verify_stage13_artifact_meta("stage13 tamper report", &tamper.artifact)?;
    let actual_cases = tamper
        .cases
        .iter()
        .map(|case| case.case_id.clone())
        .collect::<BTreeSet<_>>();
    let expected_cases = STAGE13_EXPECTED_TAMPER_CASES
        .iter()
        .map(|value| (*value).to_string())
        .collect::<BTreeSet<_>>();
    if actual_cases != expected_cases {
        return Err("stage13 tamper cases drifted from required set".to_string());
    }
    for case in &tamper.cases {
        if case.verifier_status != "rejected" {
            return Err(format!(
                "stage13 tamper case {} must stay rejected",
                case.case_id
            ));
        }
        if case.root_generation != examples.root_generation {
            return Err(format!(
                "stage13 tamper case {} root_generation drifted",
                case.case_id
            ));
        }
        if case.api_surface.trim().is_empty() {
            return Err(format!(
                "stage13 tamper case {} api_surface must stay populated",
                case.case_id
            ));
        }
        if case.example_id.trim().is_empty() || case.backend_mode.trim().is_empty() {
            return Err(format!(
                "stage13 tamper case {} binding fields must stay populated",
                case.case_id
            ));
        }
        if case.proof_surface.trim().is_empty() {
            return Err(format!(
                "stage13 tamper case {} proof_surface must stay populated",
                case.case_id
            ));
        }
        if case.case_id.starts_with("batch_") {
            if case.proof_surface != PROOF_SURFACE_BATCH {
                return Err(format!(
                    "stage13 tamper case {} proof_surface drifted",
                    case.case_id
                ));
            }
            if case.path_count.unwrap_or(0) <= 1 {
                return Err(format!(
                    "stage13 tamper case {} path_count drifted",
                    case.case_id
                ));
            }
            if !matches!(
                case.path_shape.as_deref(),
                Some(PATH_SHAPE_CLUSTERED) | Some(PATH_SHAPE_SCATTERED)
            ) {
                return Err(format!(
                    "stage13 tamper case {} path_shape drifted",
                    case.case_id
                ));
            }
        }
        verify_redacted_error(
            &format!("stage13 tamper case {}", case.case_id),
            &case.typed_error,
        )?;
    }

    let log_path = stage_log_file(ctx, stage)?;
    let log_rows = stage_log_rows(&log_path, stage.stage)
        .map_err(|e| format!("read {} failed: {e}", log_path.display()))?;
    let actual_steps: Vec<&str> = log_rows.iter().map(|row| row.step.as_str()).collect();
    let expected_steps: Vec<&str> = stage.steps.iter().map(|step| step.id.as_str()).collect();
    if actual_steps != expected_steps {
        return Err(format!(
            "stage13 log step order drifted: expected {}, got {}",
            expected_steps.join(", "),
            actual_steps.join(", ")
        ));
    }
    if log_rows.iter().any(|row| row.status != "ok") {
        return Err("stage13 log rows must stay on ok status".to_string());
    }
    for (step, needle) in [
        ("S13-1", "hjmt output prepared"),
        (
            "S13-2",
            "yaml-generated genesis asset and rights were seeded",
        ),
        ("S13-3", "fee-supported right transition"),
        ("S13-4", "present-key rejection stayed fail-closed"),
        ("S13-5", "policy-transition proofs"),
        ("S13-6", "cache and scheduler metrics"),
        ("S13-7", "reload-debug reopened"),
        ("S13-8", "wrote artifacts:"),
    ] {
        let row = log_rows
            .iter()
            .find(|row| row.step == step)
            .ok_or_else(|| format!("stage13 log row missing {step}"))?;
        if !row.detail.contains(needle) {
            return Err(format!("stage13 log detail drifted for {step}"));
        }
    }

    Ok(())
}

struct StageLogRow {
    step: String,
    status: String,
    detail: String,
}

fn stage_log_rows(path: &Path, stage_id: u32) -> Result<Vec<StageLogRow>, String> {
    let text = io::read_to_string(path).map_err(|e| e.to_string())?;
    let mut rows = Vec::new();
    for line in text.lines().map(str::trim).filter(|line| !line.is_empty()) {
        let row: Value = JsonCodec
            .deserialize(line.as_bytes())
            .map_err(|e| format!("invalid log json: {e}"))?;
        if row.get("stage").and_then(|value| value.as_u64()) != Some(stage_id as u64) {
            continue;
        }
        let step = row
            .get("step")
            .and_then(|value| value.as_str())
            .ok_or_else(|| "stage13 log row missing step".to_string())?;
        let status = row
            .get("status")
            .and_then(|value| value.as_str())
            .ok_or_else(|| "stage13 log row missing status".to_string())?;
        let detail = row
            .get("detail")
            .and_then(|value| value.as_str())
            .ok_or_else(|| "stage13 log row missing detail".to_string())?;
        rows.push(StageLogRow {
            step: step.to_string(),
            status: status.to_string(),
            detail: detail.to_string(),
        });
    }
    Ok(rows)
}

fn verify_claims(claim: &Path) -> Result<(), String> {
    for name in ["alice", "bob", "charlie"] {
        let file = claim.join(format!("claim_rows_{name}.json"));
        if !io::path_exists(&file).map_err(|e| e.to_string())? {
            return Err(format!("missing {}", file.display()));
        }
    }
    Ok(())
}

fn verify_snapshot(snapshot: &Path) -> Result<(), String> {
    if !io::path_exists(snapshot).map_err(|e| e.to_string())? {
        return Err(format!("missing {}", snapshot.display()));
    }

    let snap: Value = JsonCodec
        .deserialize(
            io::read_to_string(snapshot)
                .map_err(|e| e.to_string())?
                .as_bytes(),
        )
        .map_err(|e| format!("invalid stage_3_snapshot.json: {e}"))?;

    let stats = snap
        .get("wallet_import_stats")
        .and_then(|v| v.as_array())
        .ok_or_else(|| "stage_3_snapshot missing wallet_import_stats[]".to_string())?;

    if stats.len() != 3 {
        return Err(format!(
            "stage_3_snapshot.wallet_import_stats len must be 3, got {}",
            stats.len()
        ));
    }

    Ok(())
}

#[cfg(feature = "wallet_debug_tools")]
fn verify_debug_wallets(claim: &Path, outputs_dir: &Path) -> Result<(), String> {
    for name in ["alice", "bob", "charlie"] {
        let file = claim.join(format!("export_wallet_debug_{name}.json"));
        verify_redacted_debug_dump(&file, "claim lane")?;
    }

    let post_claim = outputs_dir
        .join("wallets_export_import")
        .join("export_wallet_debug_post_claim.json");
    verify_redacted_debug_dump(&post_claim, "post-claim export lane")?;

    Ok(())
}

#[cfg(feature = "wallet_debug_tools")]
fn verify_redacted_debug_dump(file: &Path, lane: &str) -> Result<(), String> {
    if !io::path_exists(file).map_err(|e| e.to_string())? {
        return Err(format!("missing {}", file.display()));
    }

    let text = io::read_to_string(file).map_err(|e| e.to_string())?;
    if text.contains("\"seed_phrase\"") || text.contains("\"plaintext_b64\"") {
        return Err(format!(
            "debug dump must redact wallet secrets on {lane}: {}",
            file.display()
        ));
    }

    let root: Value = JsonCodec
        .deserialize(text.as_bytes())
        .map_err(|e| format!("invalid {}: {e}", file.display()))?;
    let secrets = root
        .get("secrets")
        .and_then(|value| value.as_array())
        .ok_or_else(|| format!("debug dump missing secrets[]: {}", file.display()))?;
    if !secrets.is_empty() {
        return Err(format!(
            "debug dump secrets[] must be redacted on {lane}: {}",
            file.display()
        ));
    }

    if root
        .get("secrets_redacted")
        .and_then(|value| value.as_bool())
        != Some(true)
    {
        return Err(format!(
            "debug dump must set secrets_redacted=true on {lane}: {}",
            file.display()
        ));
    }

    Ok(())
}

fn result_tag(result: &StageResult) -> &'static str {
    match result {
        StageResult::Ok => "ok",
        StageResult::Warn(_) => "warn",
        StageResult::Fail(_) => "fail",
    }
}

#[cfg(test)]
mod tests {
    use std::{path::PathBuf, sync::Arc, sync::OnceLock};

    use tempfile::TempDir;
    use z00z_core::{AssetDefinitionRegistry, ChainType};
    use z00z_utils::{
        codec::JsonCodec, io::write_file, logger::NoopLogger, metrics::NoopMetrics,
        time::SystemTimeProvider,
    };

    use super::*;
    use crate::{DesignDoc, ScenarioCfg};

    fn build_stage13_ctx(outputs_dir: PathBuf) -> (SimContext, DesignStage) {
        let genesis_root = z00z_core::config_paths::core_config_dir();

        let mut cfg = ScenarioCfg::from_file("src/scenario_1/scenario_config.yaml")
            .expect("load scenario config");
        cfg.outputs.dir = outputs_dir.to_string_lossy().to_string();
        let stage1_cfg = cfg.stage1_genesis.as_mut().expect("stage1 genesis cfg");
        stage1_cfg.genesis_config = z00z_core::config_paths::devnet_genesis_path()
            .to_string_lossy()
            .to_string();
        stage1_cfg.paths.fallback_genesis_dir = genesis_root.to_string_lossy().to_string();

        let stage = DesignDoc::from_file("src/scenario_1/scenario_design.yaml")
            .expect("load design")
            .stages
            .into_iter()
            .find(|stage| stage.stage == 13)
            .expect("stage 13 present");

        let ctx = SimContext {
            config: cfg,
            chain_type: "devnet".parse::<ChainType>().expect("chain type"),
            registry: AssetDefinitionRegistry::new(
                Arc::new(NoopLogger),
                Arc::new(NoopMetrics),
                Arc::new(SystemTimeProvider),
            ),
            assets: Vec::new(),
            genesis_rights: Vec::new(),
            actors: Vec::new(),
            leaves: Vec::new(),
            block_height: 0,
            outputs_dir: outputs_dir.clone(),
            logger: Arc::new(NoopLogger),
            wallet_service: None,
        };

        (ctx, stage)
    }

    fn shared_stage13_contract_out() -> &'static PathBuf {
        static OUT: OnceLock<PathBuf> = OnceLock::new();
        OUT.get_or_init(|| {
            let root = crate::scenario_1::support::fixture_cache::ensure_case(
                "runner_verify_stage13_contract_v2",
                |base| {
                    let outputs_dir = base.join("outputs");
                    let (mut ctx, stage13) = build_stage13_ctx(outputs_dir.clone());
                    let stage1 = DesignDoc::from_file("src/scenario_1/scenario_design.yaml")
                        .expect("load design")
                        .stages
                        .into_iter()
                        .find(|stage| stage.stage == 1)
                        .expect("stage 1 present");

                    let stage1_res = crate::scenario_1::stage_1::run(&mut ctx, &stage1);
                    assert!(
                        matches!(stage1_res, StageResult::Ok),
                        "stage1 failed in shared stage13 fixture: {stage1_res:?}"
                    );
                    let stage13_res =
                        crate::scenario_1::stage_13::run_hjmt_examples(&mut ctx, &stage13);
                    assert!(
                        matches!(stage13_res, StageResult::Ok),
                        "stage13 failed in shared stage13 fixture: {stage13_res:?}"
                    );
                },
            );
            root.join("outputs")
        })
    }

    fn stage13_fixture() -> (TempDir, SimContext, DesignStage) {
        let temp = TempDir::new().expect("temp dir");
        let outputs_dir = temp.path().join("outputs");
        crate::scenario_1::support::fixture_cache::copy_tree(
            shared_stage13_contract_out(),
            &outputs_dir,
        );
        let (ctx, stage) = build_stage13_ctx(outputs_dir);
        (temp, ctx, stage)
    }

    fn load_json(path: &Path) -> Value {
        JsonCodec
            .deserialize(io::read_to_string(path).expect("read json").as_bytes())
            .expect("parse json")
    }

    #[test]
    fn s13_verify_live_outputs_ok() {
        let (_tmp, ctx, stage) = stage13_fixture();
        let verify = verify_stage13_contract(&ctx, &stage);
        assert!(verify.is_ok(), "stage13 contract verify failed: {verify:?}");
    }

    #[test]
    fn s13_root_drift_rejects() {
        let (_tmp, ctx, stage) = stage13_fixture();
        let report_path = ctx.outputs_dir.join(
            &ctx.config
                .stage13_hjmt_settlement_examples
                .as_ref()
                .expect("stage13 cfg")
                .examples_file,
        );
        let mut report = load_json(&report_path);
        report["examples"][0]["settlement_state_root_hex"] = Value::from("00".repeat(32));
        write_file(
            &report_path,
            JsonCodec
                .serialize_pretty(&report)
                .expect("report bytes")
                .as_slice(),
        )
        .expect("rewrite report");

        let err = verify_stage13_contract(&ctx, &stage).unwrap_err();
        assert!(
            err.contains("settlement_state_root_hex drifted"),
            "stage13 root drift expected example root drift failure, got: {err}"
        );
    }

    #[test]
    fn s13_schema_rejects_req_fields() {
        let required = [
            "root_generation",
            "proof_envelope_version",
            "proof_family",
            "leaf_family",
            "settlement_path",
            "terminal_id",
            "bucket_epoch",
            "verifier_status",
        ];
        for field in required {
            let (_tmp, ctx, stage) = stage13_fixture();
            let report_path = ctx.outputs_dir.join(
                &ctx.config
                    .stage13_hjmt_settlement_examples
                    .as_ref()
                    .expect("stage13 cfg")
                    .examples_file,
            );
            let mut report = load_json(&report_path);
            report["examples"].as_array_mut().expect("examples array")[0]
                .as_object_mut()
                .expect("example object")
                .remove(field);
            write_file(
                &report_path,
                JsonCodec
                    .serialize_pretty(&report)
                    .expect("report bytes")
                    .as_slice(),
            )
            .expect("rewrite report");

            let err = verify_stage13_contract(&ctx, &stage).unwrap_err();
            assert!(
                err.contains("stage13 examples report") && err.contains(field),
                "missing {field} expected schema failure, got: {err}"
            );
        }
    }

    #[test]
    fn s13_schema_rejects_missing_fields() {
        for field in ["proof_surface", "path_count", "atomic_verdict"] {
            let (_tmp, ctx, stage) = stage13_fixture();
            let report_path = ctx.outputs_dir.join(
                &ctx.config
                    .stage13_hjmt_settlement_examples
                    .as_ref()
                    .expect("stage13 cfg")
                    .examples_file,
            );
            let mut report = load_json(&report_path);
            report["comparison_rows"]
                .as_array_mut()
                .expect("comparison rows")[0]
                .as_object_mut()
                .expect("comparison object")
                .remove(field);
            write_file(
                &report_path,
                JsonCodec
                    .serialize_pretty(&report)
                    .expect("report bytes")
                    .as_slice(),
            )
            .expect("rewrite report");

            let err = verify_stage13_contract(&ctx, &stage).unwrap_err();
            assert!(
                err.contains("comparison_rows") && err.contains(field),
                "missing {field} expected comparison schema failure, got: {err}"
            );
        }
    }

    #[test]
    fn s13_tamper_case_drift_rejects() {
        let (_tmp, ctx, stage) = stage13_fixture();
        let report_path = ctx.outputs_dir.join(
            &ctx.config
                .stage13_hjmt_settlement_examples
                .as_ref()
                .expect("stage13 cfg")
                .tamper_report_file,
        );
        let mut report = load_json(&report_path);
        report["cases"].as_array_mut().expect("cases").pop();
        write_file(
            &report_path,
            JsonCodec
                .serialize_pretty(&report)
                .expect("report bytes")
                .as_slice(),
        )
        .expect("rewrite report");

        assert!(verify_stage13_contract(&ctx, &stage)
            .unwrap_err()
            .contains("tamper cases drifted"));
    }

    #[test]
    fn s13_comparison_surface_drift_rejects() {
        let (_tmp, ctx, stage) = stage13_fixture();
        let report_path = ctx.outputs_dir.join(
            &ctx.config
                .stage13_hjmt_settlement_examples
                .as_ref()
                .expect("stage13 cfg")
                .examples_file,
        );
        let mut report = load_json(&report_path);
        report["comparison_rows"][0]["proof_surface"] = Value::from("unknown_surface");
        write_file(
            &report_path,
            JsonCodec
                .serialize_pretty(&report)
                .expect("report bytes")
                .as_slice(),
        )
        .expect("rewrite report");

        let err = verify_stage13_contract(&ctx, &stage).unwrap_err();
        assert!(
            err.contains("proof_surface set drifted")
                || err.contains("unknown comparison proof_surface"),
            "stage13 comparison proof_surface drift expected failure, got: {err}"
        );
    }

    #[test]
    fn s13_verdict_drift_rejects() {
        let (_tmp, ctx, stage) = stage13_fixture();
        let report_path = ctx.outputs_dir.join(
            &ctx.config
                .stage13_hjmt_settlement_examples
                .as_ref()
                .expect("stage13 cfg")
                .examples_file,
        );
        let mut report = load_json(&report_path);
        let batch_row = report["comparison_rows"]
            .as_array_mut()
            .expect("comparison rows")
            .iter_mut()
            .find(|row| row["proof_surface"].as_str() == Some("batch_proof_v1"))
            .expect("batch comparison row");
        batch_row["atomic_verdict"] = Value::from("partial_accept");
        write_file(
            &report_path,
            JsonCodec
                .serialize_pretty(&report)
                .expect("report bytes")
                .as_slice(),
        )
        .expect("rewrite report");

        let err = verify_stage13_contract(&ctx, &stage).unwrap_err();
        assert!(
            err.contains("atomic verdict drifted") || err.contains("BatchAtomicVerdictMix"),
            "stage13 batch atomic verdict drift expected failure, got: {err}"
        );
    }

    #[test]
    fn s13_tamper_redaction_drift_rejects() {
        let (_tmp, ctx, stage) = stage13_fixture();
        let report_path = ctx.outputs_dir.join(
            &ctx.config
                .stage13_hjmt_settlement_examples
                .as_ref()
                .expect("stage13 cfg")
                .tamper_report_file,
        );
        let mut report = load_json(&report_path);
        report["cases"][0]["typed_error"]["message"] =
            Value::from("owner_sk deadbeefdeadbeefdeadbeefdeadbeef");
        write_file(
            &report_path,
            JsonCodec
                .serialize_pretty(&report)
                .expect("report bytes")
                .as_slice(),
        )
        .expect("rewrite report");

        let err = verify_stage13_contract(&ctx, &stage).unwrap_err();
        assert!(
            err.contains("typed_error.message violated redaction"),
            "stage13 tamper redaction drift expected redaction failure, got: {err}"
        );
    }

    #[test]
    fn s13_metrics_missing_rejects() {
        let (_tmp, ctx, stage) = stage13_fixture();
        let metrics_path = ctx.outputs_dir.join(
            &ctx.config
                .stage13_hjmt_settlement_examples
                .as_ref()
                .expect("stage13 cfg")
                .cache_scheduler_metrics_file,
        );
        io::remove_file(&metrics_path).expect("remove metrics report");

        assert!(verify_stage13_contract(&ctx, &stage)
            .unwrap_err()
            .contains("missing"));
    }

    #[test]
    fn s13_fee_ownership_drift_rejects() {
        let (_tmp, ctx, stage) = stage13_fixture();
        let report_path = ctx.outputs_dir.join(
            &ctx.config
                .stage13_hjmt_settlement_examples
                .as_ref()
                .expect("stage13 cfg")
                .examples_file,
        );
        let mut report = load_json(&report_path);
        report["examples"][2]["proof_is_ownership"] = Value::from(true);
        write_file(
            &report_path,
            JsonCodec
                .serialize_pretty(&report)
                .expect("report bytes")
                .as_slice(),
        )
        .expect("rewrite report");

        let err = verify_stage13_contract(&ctx, &stage).unwrap_err();
        assert!(
            err.contains("OwnershipMix"),
            "stage13 fee ownership drift expected OwnershipMix, got: {err}"
        );
    }

    #[test]
    fn s13_fee_status_drift_rejects() {
        let (_tmp, ctx, stage) = stage13_fixture();
        let report_path = ctx.outputs_dir.join(
            &ctx.config
                .stage13_hjmt_settlement_examples
                .as_ref()
                .expect("stage13 cfg")
                .examples_file,
        );
        let mut report = load_json(&report_path);
        report["examples"][2]["replay_status"] = Value::from("rejected");
        write_file(
            &report_path,
            JsonCodec
                .serialize_pretty(&report)
                .expect("report bytes")
                .as_slice(),
        )
        .expect("rewrite report");

        let err = verify_stage13_contract(&ctx, &stage).unwrap_err();
        assert!(
            err.contains("ReplayStatusMix"),
            "stage13 fee replay_status drift expected ReplayStatusMix, got: {err}"
        );
    }

    #[test]
    fn s13_absence_drift_rejects() {
        let (_tmp, ctx, stage) = stage13_fixture();
        let report_path = ctx.outputs_dir.join(
            &ctx.config
                .stage13_hjmt_settlement_examples
                .as_ref()
                .expect("stage13 cfg")
                .examples_file,
        );
        let mut report = load_json(&report_path);
        report["examples"][4]["present_key_rejection"] = Value::Null;
        write_file(
            &report_path,
            JsonCodec
                .serialize_pretty(&report)
                .expect("report bytes")
                .as_slice(),
        )
        .expect("rewrite report");

        let err = verify_stage13_contract(&ctx, &stage).unwrap_err();
        assert!(
            err.contains("PresentKeyRejectMix"),
            "stage13 absence drift expected PresentKeyRejectMix, got: {err}"
        );
    }

    #[test]
    fn s13_policy_root_drift_rejects() {
        let (_tmp, ctx, stage) = stage13_fixture();
        let report_path = ctx.outputs_dir.join(
            &ctx.config
                .stage13_hjmt_settlement_examples
                .as_ref()
                .expect("stage13 cfg")
                .examples_file,
        );
        let mut report = load_json(&report_path);
        report["examples"][6]["next_state_root_hex"] = Value::from("00".repeat(32));
        write_file(
            &report_path,
            JsonCodec
                .serialize_pretty(&report)
                .expect("report bytes")
                .as_slice(),
        )
        .expect("rewrite report");

        let err = verify_stage13_contract(&ctx, &stage).unwrap_err();
        assert!(
            err.contains("NextRootMix"),
            "stage13 policy root drift expected NextRootMix, got: {err}"
        );
    }

    #[test]
    fn s13_proof_coverage_drift_rejects() {
        let (_tmp, ctx, stage) = stage13_fixture();
        let proof_path = ctx.outputs_dir.join(
            &ctx.config
                .stage13_hjmt_settlement_examples
                .as_ref()
                .expect("stage13 cfg")
                .proof_size_report_file,
        );
        let mut report = load_json(&proof_path);
        report["entries"][0]["example_id"] = Value::from("E7_policy_transition");
        write_file(
            &proof_path,
            JsonCodec
                .serialize_pretty(&report)
                .expect("proof bytes")
                .as_slice(),
        )
        .expect("rewrite proof report");

        let err = verify_stage13_contract(&ctx, &stage).unwrap_err();
        assert!(
            err.contains("proof-size coverage drifted"),
            "stage13 proof-size drift expected coverage error, got: {err}"
        );
    }

    #[test]
    fn s13_proof_comparison_drift_rejects() {
        let (_tmp, ctx, stage) = stage13_fixture();
        let proof_path = ctx.outputs_dir.join(
            &ctx.config
                .stage13_hjmt_settlement_examples
                .as_ref()
                .expect("stage13 cfg")
                .proof_size_report_file,
        );
        let mut report = load_json(&proof_path);
        report["comparison_rows"][0]["row_id"] = Value::from("drifted_row_id");
        write_file(
            &proof_path,
            JsonCodec
                .serialize_pretty(&report)
                .expect("proof bytes")
                .as_slice(),
        )
        .expect("rewrite proof report");

        let err = verify_stage13_contract(&ctx, &stage).unwrap_err();
        assert!(
            err.contains("proof-size comparison coverage drifted")
                || err.contains("proof-size comparison row"),
            "stage13 proof-size comparison drift expected failure, got: {err}"
        );
    }

    #[test]
    fn s13_replay_coverage_drift_rejects() {
        let (_tmp, ctx, stage) = stage13_fixture();
        let replay_path = ctx.outputs_dir.join(
            &ctx.config
                .stage13_hjmt_settlement_examples
                .as_ref()
                .expect("stage13 cfg")
                .replay_roots_file,
        );
        let mut report = load_json(&replay_path);
        report["replay_entries"][0]["example_id"] = Value::from("E8_cache_scheduler");
        write_file(
            &replay_path,
            JsonCodec
                .serialize_pretty(&report)
                .expect("replay bytes")
                .as_slice(),
        )
        .expect("rewrite replay report");

        let err = verify_stage13_contract(&ctx, &stage).unwrap_err();
        assert!(
            err.contains("replay entry coverage drifted"),
            "stage13 replay drift expected coverage error, got: {err}"
        );
    }

    #[test]
    fn s13_tamper_drift_rejects() {
        let (_tmp, ctx, stage) = stage13_fixture();
        let report_path = ctx.outputs_dir.join(
            &ctx.config
                .stage13_hjmt_settlement_examples
                .as_ref()
                .expect("stage13 cfg")
                .tamper_report_file,
        );
        let mut report = load_json(&report_path);
        let batch_case = report["cases"]
            .as_array_mut()
            .expect("cases")
            .iter_mut()
            .find(|case| {
                case["case_id"]
                    .as_str()
                    .is_some_and(|value| value.starts_with("batch_"))
            })
            .expect("batch tamper case");
        batch_case["proof_surface"] = Value::from("proof_blob_vec");
        write_file(
            &report_path,
            JsonCodec
                .serialize_pretty(&report)
                .expect("report bytes")
                .as_slice(),
        )
        .expect("rewrite report");

        let err = verify_stage13_contract(&ctx, &stage).unwrap_err();
        assert!(
            err.contains("proof_surface drifted"),
            "stage13 batch tamper proof_surface drift expected failure, got: {err}"
        );
    }

    #[test]
    fn s13_log_order_rejects() {
        let (_tmp, ctx, stage) = stage13_fixture();
        let log_path = ctx.outputs_dir.join(STAGE13_LOG_FILE);
        let mut rows: Vec<Value> = io::read_to_string(&log_path)
            .expect("read log")
            .lines()
            .map(|line| JsonCodec.deserialize(line.as_bytes()).expect("parse row"))
            .collect();
        rows.swap(0, 1);
        let body = rows
            .iter()
            .map(|row| row.to_string())
            .collect::<Vec<_>>()
            .join("\n");
        write_file(&log_path, body.as_bytes()).expect("rewrite log");

        let err = verify_stage13_contract(&ctx, &stage).unwrap_err();
        assert!(
            err.contains("log step order drifted"),
            "unexpected error: {err}"
        );
    }
}
