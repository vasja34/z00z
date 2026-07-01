use serde::Serialize;
use std::{
    path::{Path, PathBuf},
    sync::Arc,
};
use z00z_core::{assets::AssetPackPlain, Asset};
use z00z_storage::settlement::TerminalLeaf;
use z00z_utils::io::create_dir_all;
use z00z_wallets::{
    receiver::ReceiveReport, rpc::types::common::PersistWalletId, services::WalletService,
    tx::TxPackage,
};

use crate::SimContext;

use super::super::stage_2::actor_runtime_password;
use super::transfer_lane_runtime_support::log_ok;
use super::transfer_lane_support::{build_recv_ctx, load_stage4_pkg, log_recv};
#[derive(Serialize)]
pub(crate) struct LogRow {
    pub(super) timestamp: String,
    pub(super) stage: u32,
    pub(super) step: String,
    pub(super) event: String,
    pub(super) status: String,
    pub(super) detail: String,
}

#[derive(Serialize)]
pub(crate) struct Stage5Snap {
    pub(super) stage: u32,
    pub(super) transfer_count: u32,
    pub(super) input_tx_digest_hex: String,
    pub(super) recipient_output_index: usize,
    pub(super) canonical_status: String,
    pub(super) runtime_status: String,
    pub(super) rpc_status: String,
    pub(super) claimed_count_after_route: usize,
    pub(super) status: String,
}

#[derive(Serialize)]
pub(crate) struct Stage5TxFile {
    pub(super) stage: u32,
    pub(super) source_tx_digest_hex: String,
    pub(super) recipient_output_index: usize,
    pub(super) asset_id_hex: String,
    pub(super) serial_id: u32,
    pub(super) amount: u64,
    pub(super) r_pub: String,
    pub(super) owner_tag: String,
    pub(super) tag16: u16,
    pub(super) c_amount: String,
    pub(super) ciphertext_len: usize,
    pub(super) status: String,
}

pub(crate) struct RecvCtx {
    pub(crate) tx_pkg: TxPackage,
    pub(crate) out_idx: usize,
    pub(crate) asset: Asset,
    pub(crate) leaf: TerminalLeaf,
    pub(crate) pack: AssetPackPlain,
    pub(crate) canon: ReceiveReport,
    pub(crate) runtime: ReceiveReport,
}

pub(crate) type StageCtx<'a> = (
    crate::config::Stage5PathsCfg,
    &'a crate::config::Stage4TxPrepareCfg,
    Arc<WalletService>,
    &'a crate::SimActor,
    String,
    usize,
);

pub(crate) struct RpcCtx {
    pub(crate) bob_id: PersistWalletId,
    pub(crate) before_len: usize,
    pub(crate) rpc_status: String,
}

fn stage5_cfg(ctx: &SimContext) -> Result<&crate::config::Stage5TransferCfg, String> {
    ctx.config
        .stage5_transfer
        .as_ref()
        .ok_or_else(|| "stage5: stage5_transfer config missing".to_string())
}

fn stage4_cfg(ctx: &SimContext) -> Result<&crate::config::Stage4TxPrepareCfg, String> {
    ctx.config
        .stage4_tx_prepare
        .as_ref()
        .ok_or_else(|| "stage5: stage4_tx_prepare config missing".to_string())
}

fn wallet_svc(ctx: &SimContext) -> Result<Arc<WalletService>, String> {
    ctx.wallet_service
        .as_ref()
        .cloned()
        .ok_or_else(|| "stage5: wallet_service missing".to_string())
}

pub(crate) fn load_stage_ctx(ctx: &SimContext) -> Result<StageCtx<'_>, String> {
    let p = ctx.config.stage5_paths();
    let s5 = stage5_cfg(ctx)?;
    let s4 = stage4_cfg(ctx)?;
    let wallet_svc = wallet_svc(ctx)?;
    let receiver = pick_actor(ctx, &s4.receiver_actor)?;
    let receiver_pass = actor_runtime_password(receiver)
        .ok_or_else(|| format!("stage5: no password for actor {}", receiver.name))?;
    Ok((
        p,
        s4,
        wallet_svc,
        receiver,
        receiver_pass.to_string(),
        s5.recipient_output_index,
    ))
}

pub(crate) fn prep_dirs(
    out: &Path,
    cfg: &crate::config::Stage5PathsCfg,
    stage_id: u32,
    lines: &mut Vec<String>,
) -> Result<(PathBuf, PathBuf), String> {
    let logs_dir = out.join(&cfg.logs_dir);
    let tx_dir = out.join(&cfg.transactions_dir);
    create_dir_all(out).map_err(|e| e.to_string())?;
    create_dir_all(&logs_dir).map_err(|e| e.to_string())?;
    create_dir_all(&tx_dir).map_err(|e| e.to_string())?;
    log_ok(
        lines,
        stage_id,
        "S5-1",
        "prepare_dirs",
        &tx_dir.to_string_lossy(),
    )?;
    Ok((logs_dir, tx_dir))
}

pub(crate) fn load_recv(
    out: &Path,
    s4: &crate::config::Stage4TxPrepareCfg,
    out_idx: usize,
    receiver: &crate::SimActor,
    stage_id: u32,
    lines: &mut Vec<String>,
) -> Result<RecvCtx, String> {
    let tx_pkg = load_stage4_pkg(out, s4)?;
    let recv = build_recv_ctx(tx_pkg, out_idx, receiver)?;
    log_recv(lines, stage_id, &recv)?;
    Ok(recv)
}

fn pick_actor<'a>(ctx: &'a SimContext, name: &str) -> Result<&'a crate::SimActor, String> {
    ctx.actors
        .iter()
        .find(|actor| actor.name.eq_ignore_ascii_case(name))
        .ok_or_else(|| format!("stage5: actor not found in context: {name}"))
}
