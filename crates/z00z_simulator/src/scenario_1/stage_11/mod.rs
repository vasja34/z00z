use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

use z00z_wallets::tx::{TxOutputWire, TxPackage};

use crate::{DesignStage, SimContext, StageResult};

mod apply;
mod charlie;
mod finish;
pub mod jmt_wallet_scan;

// Stage 11 remains downstream of the stage_4 tx lane through the stage_6 bundle lane.
// Checkpoint acceptance here is package-coupled continuity across the stage4 proof, input refs,
// and stage6 bridge outputs; detached proof bytes remain insufficient by themselves and do not
// upgrade this path into standalone backend authority.
// Publish is not yet strong enough to be called fully trustless.
// Package-coupled checkpoint integrity exists here; authoritative publish-proof closure does not.

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct Stage11Checkpoint {
    pub(crate) stage: u32,
    pub(crate) prev_root_hex: String,
    pub(crate) new_root_hex: String,
    pub(crate) draft_id_hex: String,
    pub(crate) exec_input_id_hex: String,
    pub(crate) snapshot_id_hex: String,
    pub(crate) spent_delta: Vec<String>,
    pub(crate) created_delta: Vec<super::stage_9::bundle_lane_impl::MadeEnt>,
    pub(crate) fragment_ids: Vec<String>,
    pub(crate) charlie_detected_count: usize,
    pub(crate) charlie_detected_amount: u64,
    pub(crate) wallet_invariant_ok: bool,
    pub(crate) wallet_scan_file: String,
    pub(crate) status: String,
}

struct Stage11Load {
    bridge: super::stage_9::bundle_lane_impl::Stage9Bridge,
    frag_a: super::stage_9::bundle_lane_impl::FragTx,
    frag_b: super::stage_9::bundle_lane_impl::FragTx,
    pkg: TxPackage,
    snap_id: z00z_storage::snapshot::PrepSnapshotId,
    prep: z00z_storage::snapshot::PrepSnapshot,
    replay: Vec<z00z_storage::snapshot::PrepReplayEntry>,
    exec: z00z_storage::checkpoint::CheckpointExecInput,
    exec_id: z00z_storage::checkpoint::CheckpointExecInputId,
}

struct Stage11Apply {
    summary: Stage11Checkpoint,
    draft_id_hex: String,
}

pub(super) struct CharlieApply {
    pub(super) artifact: jmt_wallet_scan::JmtScanArtifact,
    pub(super) invariant_ok: bool,
}

#[derive(Clone)]
pub(super) struct RuntimeOut {
    pub(super) wire: TxOutputWire,
    pub(super) asset: z00z_core::Asset,
}

pub(super) struct Stage11Cfg {
    pub(super) out: PathBuf,
    pub(super) logs_dir: PathBuf,
    pub(super) tx_dir: PathBuf,
    pub(super) wallet_scan_file: String,
    pub(super) wallet_scan_path: PathBuf,
    pub(super) p6: crate::config::Stage6PathsCfg,
    pub(super) p7: crate::config::Stage7PathsCfg,
    pub(super) s4: crate::config::Stage4TxPrepareCfg,
}

pub fn run_apply(ctx: &mut SimContext, stage: &DesignStage) -> StageResult {
    let cfg = match load_cfg(ctx) {
        Ok(cfg) => cfg,
        Err(err) => {
            return StageResult::Fail(format!(
                "stage {} ({}) failed: {}",
                stage.stage, stage.name, err
            ));
        }
    };
    match run_stage11_with_cfg(ctx, stage, &cfg) {
        Ok(()) => StageResult::Ok,
        Err(err) => StageResult::Fail(format!(
            "stage {} ({}) failed: {}",
            stage.stage, stage.name, err
        )),
    }
}

pub fn run(ctx: &mut SimContext, stage: &DesignStage) -> StageResult {
    run_apply(ctx, stage)
}

fn run_stage11_with_cfg(
    ctx: &mut SimContext,
    stage: &DesignStage,
    cfg: &Stage11Cfg,
) -> Result<(), String> {
    let mut lines = start_stage11(stage.stage, &cfg.out, &cfg.logs_dir, &cfg.tx_dir)?;
    let apply = run_stage11_apply(ctx, stage.stage, cfg, &mut lines)?;
    finish_stage11(stage, cfg, &apply, &mut lines)?;
    Ok(())
}

fn load_cfg(ctx: &SimContext) -> Result<Stage11Cfg, String> {
    let p7 = ctx.config.stage7_paths();
    let p6 = ctx.config.stage6_paths();
    let out = ctx.outputs_dir.clone();
    let wallet_scan_file = ctx
        .config
        .runtime_observability_ref()
        .ok_or_else(|| "runtime_observability config missing".to_string())?
        .packet
        .wallet_scan_file
        .clone();
    Ok(Stage11Cfg {
        out: out.clone(),
        logs_dir: out.join(&p7.logs_dir),
        tx_dir: out.join(&p7.transactions_dir),
        wallet_scan_path: out.join(&wallet_scan_file),
        wallet_scan_file,
        p6,
        p7,
        s4: ctx
            .config
            .stage4_tx_prepare
            .as_ref()
            .ok_or_else(|| "stage4_tx_prepare config missing".to_string())?
            .clone(),
    })
}

fn start_stage11(
    stage_id: u32,
    out: &Path,
    logs_dir: &Path,
    tx_dir: &Path,
) -> Result<Vec<String>, String> {
    finish::prep_dirs(out, logs_dir, tx_dir)?;
    let mut lines = Vec::new();
    finish::log_step(
        &mut lines,
        stage_id,
        "S7-1",
        "prepare_dirs",
        "outputs/logs/transactions prepared",
    )?;
    Ok(lines)
}

fn run_stage11_apply(
    ctx: &mut SimContext,
    stage_id: u32,
    cfg: &Stage11Cfg,
    lines: &mut Vec<String>,
) -> Result<Stage11Apply, String> {
    let (load, bridge_path, exec_path) =
        apply::load_stage11_checked(&cfg.out, &cfg.tx_dir, &cfg.p6, &cfg.s4)?;
    finish::log_step(
        lines,
        stage_id,
        "S7-2",
        "load_bridge",
        &format!(
            "bridge={} exec_input={}",
            bridge_path.display(),
            exec_path.display()
        ),
    )?;

    let apply = apply::apply_stage11(ctx, stage_id, cfg, load)?;
    finish::log_step(
        lines,
        stage_id,
        "S7-3",
        "apply_storage",
        &format!("draft_id={}", apply.draft_id_hex),
    )?;
    Ok(apply)
}

fn finish_stage11(
    stage: &DesignStage,
    cfg: &Stage11Cfg,
    apply: &Stage11Apply,
    lines: &mut Vec<String>,
) -> Result<(), String> {
    let cp_path = finish::write_stage11(&cfg.tx_dir, &cfg.p7.checkpoint_file, &apply.summary)?;
    finish::log_step(
        lines,
        stage.stage,
        "S7-4",
        "write_checkpoint",
        &cp_path.to_string_lossy(),
    )?;
    finish::fill_steps(lines, stage, &["S7-1", "S7-2", "S7-3", "S7-4"])?;
    finish::flush_logs(&cfg.logs_dir.join(&cfg.p7.logger_file), lines)?;
    Ok(())
}
