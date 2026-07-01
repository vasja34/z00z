//! Scenario 1 stage 1 implementation.

use serde::Serialize;
use std::{collections::HashSet, path::PathBuf, str::FromStr, sync::Arc};
use z00z_core::genesis::{
    export_genesis_settlement_artifacts, generate_genesis_lanes, load_genesis_context,
    validator::compute_genesis_state_hash, ChainType, GenesisGenerationPlan, GENESIS_POLICIES_FILE,
    GENESIS_RIGHTS_FILE, GENESIS_SETTLEMENT_MANIFEST_FILE, GENESIS_VOUCHERS_FILE,
};
use z00z_core::{
    assets::registry::AssetDefinitionRegistry, genesis::genesis_config::GenesisConfig,
};
use z00z_utils::{
    io::{create_dir_all, path_exists, save_bincode, save_json, write_file},
    logger::NoopLogger,
    metrics::NoopMetrics,
};

use crate::{DesignStage, SimContext, StageResult};
mod support;

use self::support::{
    flush_logs, hex_str, push_log, resolve_genesis_cfg_path, run_cli_checks, verify_assets_all,
};

#[derive(Debug, Serialize)]
struct Stage1Snap {
    stage: u32,
    chain: String,
    defs_count: usize,
    assets_count: usize,
    policy_count: usize,
    rights_count: usize,
    voucher_count: usize,
    state_hash: String,
    policies_artifact_file: String,
    rights_artifact_file: String,
    vouchers_artifact_file: String,
    settlement_manifest_file: String,
    out_dir: String,
}

#[derive(Debug, Serialize)]
struct LogRow {
    timestamp: String,
    stage: u32,
    step: String,
    event: String,
    status: String,
    detail: String,
}

/// Runs stage 1 (`genesis_init`).
pub fn run(ctx: &mut SimContext, stage: &DesignStage) -> StageResult {
    match run_core(ctx, stage.stage) {
        Ok(()) => StageResult::Ok,
        Err(err) => StageResult::Fail(format!(
            "stage {} ({}) failed: {}",
            stage.stage, stage.name, err
        )),
    }
}

pub fn run_core_with_config(cfg: &GenesisConfig) -> Result<(), String> {
    checked_net(cfg, "devnet").map(|_| ())
}

fn checked_net(cfg: &GenesisConfig, expected_chain: &str) -> Result<ChainType, String> {
    let net = ChainType::from_str(&cfg.chain.chain_type).map_err(|err| err.to_string())?;
    let expected = ChainType::from_str(expected_chain).map_err(|err| err.to_string())?;
    if net != expected {
        return Err(format!(
            "expected {} chain type but got '{}'",
            expected_chain, cfg.chain.chain_type
        ));
    }
    Ok(net)
}

fn run_core(ctx: &mut SimContext, stage_id: u32) -> Result<(), String> {
    let mut logs = Vec::new();
    let p = ctx.config.stage1_paths();
    let cfg_path =
        resolve_genesis_cfg_path(&ctx.config.stage1_genesis_config(), &p.fallback_genesis_dir);
    let plan = GenesisGenerationPlan::full_bootstrap();
    let genesis = load_genesis_context(&cfg_path, &plan).map_err(|e| e.to_string())?;
    push_log(&mut logs, stage_id, "S1-1", "load_config", "ok", &cfg_path)?;
    push_log(
        &mut logs,
        stage_id,
        "S1-1",
        "seed_validate",
        "ok",
        "load_genesis_context",
    )?;
    let net = checked_net(&genesis.config, &ctx.config.chain)?;

    let out = ctx.outputs_dir.clone();
    let out_gen = out.join(&p.genesis_dir);
    create_dir_all(&out).map_err(|e| e.to_string())?;
    create_dir_all(&out_gen).map_err(|e| e.to_string())?;
    push_log(
        &mut logs,
        stage_id,
        "S1-2",
        "prepare_output",
        "ok",
        &out.to_string_lossy(),
    )?;

    let outputs =
        generate_genesis_lanes(&genesis, &plan, Arc::new(NoopLogger), Arc::new(NoopMetrics))
            .map_err(|e| e.to_string())?;
    let defs = outputs
        .asset_definitions
        .clone()
        .ok_or_else(|| "full bootstrap must emit asset definitions".to_string())?;
    for def in &defs {
        if def.id.iter().all(|byte| *byte == 0) {
            return Err("asset id must be non-zero".to_string());
        }
        if def.serials == 0 {
            return Err(format!("serials must be > 0 for {}", def.symbol));
        }
    }
    ctx.registry = AssetDefinitionRegistry::from_definitions(&defs).map_err(|e| e.to_string())?;
    for def in &defs {
        push_log(
            &mut logs,
            stage_id,
            "S1-3",
            "register_definition",
            "ok",
            &def.symbol,
        )?;
    }

    if defs.len() != genesis.config.assets.len() {
        return Err("definitions count mismatch".to_string());
    }
    for def in &defs {
        let has_def = ctx.registry.contains(&def.id).map_err(|e| e.to_string())?;
        if !has_def {
            return Err(format!(
                "definition missing in ctx.registry: {}",
                def.symbol
            ));
        }
    }

    let policies = outputs.policies.clone().unwrap_or_default();
    let acc = outputs.combined_corpus();
    push_log(
        &mut logs,
        stage_id,
        "S1-4",
        "generate_assets",
        "ok",
        &format!(
            "asset_count={} rights_count={} policy_count={} voucher_count={}",
            acc.total_count(),
            acc.total_right_count(),
            policies.len(),
            acc.total_voucher_count(),
        ),
    )?;
    let all_assets = acc.flatten();

    if all_assets.len() != acc.total_count() {
        return Err("asset count mismatch after flatten".to_string());
    }
    if all_assets.is_empty() {
        return Err("generated assets are empty".to_string());
    }
    for asset in &all_assets {
        if asset.commitment.as_bytes().len() != 32 {
            return Err("commitment length must be 32".to_string());
        }
        if asset.serial_id >= asset.definition.serials {
            return Err("serial_id must be less than definition.serials".to_string());
        }
    }

    ctx.assets = all_assets.clone();
    ctx.genesis_rights = acc.rights.clone();
    if ctx.genesis_rights.is_empty() {
        return Err("generated rights are empty".to_string());
    }

    verify_assets_all(&ctx.assets).map_err(|e| e.to_string())?;
    push_log(
        &mut logs,
        stage_id,
        "S1-5",
        "verify_assets",
        "ok",
        "all proofs valid (single batch or chunked fallback)",
    )?;

    let state_hash = compute_genesis_state_hash(&acc);
    let state_hash_2 = compute_genesis_state_hash(&acc);
    if state_hash != state_hash_2 {
        return Err("state hash must be deterministic".to_string());
    }
    push_log(
        &mut logs,
        stage_id,
        "S1-6",
        "state_hash",
        "ok",
        &hex_str(&state_hash),
    )?;

    let mut saved_bins: HashSet<PathBuf> = HashSet::new();
    for def in &defs {
        let bin_file = out_gen.join(format!("genesis_{}.bin", def.symbol));
        if !saved_bins.insert(bin_file.clone()) {
            return Err(format!(
                "duplicate genesis bin path for symbol {}",
                def.symbol
            ));
        }

        let def_assets: Vec<_> = all_assets
            .iter()
            .filter(|item| item.definition.id == def.id)
            .cloned()
            .collect();
        if def_assets.is_empty() {
            return Err(format!("no generated assets for {}", def.symbol));
        }

        save_bincode(&bin_file, &def_assets).map_err(|e| e.to_string())?;
        if !path_exists(&bin_file).map_err(|e| e.to_string())? {
            return Err(format!("missing bin output: {}", bin_file.display()));
        }
        push_log(
            &mut logs,
            stage_id,
            "S1-7",
            "save_bin",
            "ok",
            &bin_file.to_string_lossy(),
        )?;
    }

    let hash_file = out_gen.join(&p.state_hash_file);
    write_file(&hash_file, hex_str(&state_hash).as_bytes()).map_err(|e| e.to_string())?;
    let (rights_file, manifest_file) = export_genesis_settlement_artifacts(
        &out_gen,
        &defs,
        &policies,
        &acc,
        net,
        z00z_core::genesis::GENESIS_ROOT_GENERATION,
        &state_hash,
        genesis.seed.as_bytes(),
    )
    .map_err(|e| e.to_string())?;
    push_log(
        &mut logs,
        stage_id,
        "S1-8",
        "export_settlement_artifacts",
        "ok",
        &format!(
            "rights={} manifest={}",
            rights_file.display(),
            manifest_file.display()
        ),
    )?;

    let stage_file = out.join(&p.snapshot_file);
    let stage_snap = Stage1Snap {
        stage: stage_id,
        chain: net.as_str().to_string(),
        defs_count: defs.len(),
        assets_count: ctx.assets.len(),
        policy_count: policies.len(),
        rights_count: ctx.genesis_rights.len(),
        voucher_count: acc.total_voucher_count(),
        state_hash: hex_str(&state_hash),
        policies_artifact_file: GENESIS_POLICIES_FILE.to_string(),
        rights_artifact_file: GENESIS_RIGHTS_FILE.to_string(),
        vouchers_artifact_file: GENESIS_VOUCHERS_FILE.to_string(),
        settlement_manifest_file: GENESIS_SETTLEMENT_MANIFEST_FILE.to_string(),
        out_dir: out.to_string_lossy().to_string(),
    };
    save_json(&stage_file, &stage_snap).map_err(|e| e.to_string())?;
    push_log(
        &mut logs,
        stage_id,
        "S1-8",
        "write_snapshot",
        "ok",
        &stage_file.to_string_lossy(),
    )?;

    run_cli_checks(&out_gen, &all_assets, &mut logs, stage_id)?;
    let logs_dir = out.join(&p.logs_dir);
    create_dir_all(&logs_dir).map_err(|e| e.to_string())?;
    flush_logs(&logs_dir.join(&p.logger_file), &logs)?;

    Ok(())
}
