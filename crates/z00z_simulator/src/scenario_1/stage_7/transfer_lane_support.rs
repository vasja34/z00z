use std::{path::Path, sync::Arc};

use z00z_core::Asset;
use z00z_networks_rpc::RpcTransport;
use z00z_utils::{
    codec::{Codec, JsonCodec},
    io::read_to_string,
};
use z00z_wallets::{
    receiver::ReceiveStatus,
    rpc::types::{asset::RuntimeReceiveAssetResponse, common::PersistWalletId},
    services::WalletService,
    tx::TxPackage,
};

use super::transfer_lane_impl::{RecvCtx, RpcCtx};

use super::transfer_lane_runtime_support::{
    check_claim_set, check_role, check_scan, log_ok, parse_out, require_claim, resolve_input_path,
    run_receive_asset_rpc, select_bob_output,
};

pub(super) fn load_stage4_pkg(
    out: &Path,
    s4: &crate::config::Stage4TxPrepareCfg,
) -> Result<TxPackage, String> {
    let s4_tx_file = resolve_input_path(out, &s4.paths.tx_pkg_file)?;
    let body = read_to_string(&s4_tx_file).map_err(|e| e.to_string())?;
    crate::scenario_1::stage_6::verify_tx_package(body.as_bytes())
        .map_err(|e| format!("stage5: tx package verification failed: {e}"))?;
    JsonCodec
        .deserialize(body.as_bytes())
        .map_err(|e| format!("stage5: json parse {}: {e}", s4_tx_file.display()))
}

pub(super) fn log_recv(
    lines: &mut Vec<String>,
    stage_id: u32,
    recv: &RecvCtx,
) -> Result<(), String> {
    log_ok(
        lines,
        stage_id,
        "S5-2",
        "pick_output",
        &format!("recipient_output_index={}", recv.out_idx),
    )?;
    log_ok(
        lines,
        stage_id,
        "S5-3",
        "build_asset_leaf",
        &format!(
            "asset_id={} serial_id={} detected_amount={}",
            hex::encode(recv.asset.asset_id()),
            recv.asset.serial_id,
            recv.pack.value
        ),
    )?;
    log_ok(
        lines,
        stage_id,
        "S5-4",
        "canonical_scan",
        "selected recipient output detected via canonical leaf scan",
    )?;
    log_ok(
        lines,
        stage_id,
        "S5-4",
        "runtime_scan",
        "runtime scanner parity matches canonical report",
    )
}

pub(crate) async fn run_report_flow(
    wallet_svc: Arc<WalletService>,
    transport: &impl RpcTransport,
    s4: &crate::config::Stage4TxPrepareCfg,
    receiver: &crate::SimActor,
    receiver_pass: &str,
    recv: &RecvCtx,
    stage_id: u32,
    lines: &mut Vec<String>,
) -> Result<RpcCtx, String> {
    run_report(
        wallet_svc,
        transport,
        s4,
        receiver,
        receiver_pass,
        recv.asset.asset_id(),
        stage_id,
        lines,
    )
    .await
}

async fn run_report(
    wallet_svc: Arc<WalletService>,
    transport: &impl RpcTransport,
    s4: &crate::config::Stage4TxPrepareCfg,
    receiver: &crate::SimActor,
    receiver_pass: &str,
    asset_id: [u8; 32],
    stage_id: u32,
    lines: &mut Vec<String>,
) -> Result<RpcCtx, String> {
    let bob_id = PersistWalletId(receiver.wallet_id.clone());
    let claimed_before = list_claimed(&wallet_svc, &bob_id, "before receive").await?;
    require_claim(&claimed_before, asset_id, "before rpc receive")?;
    let recv =
        run_receive_asset_rpc(transport, s4, &receiver.wallet_id, receiver_pass, asset_id).await?;
    let claimed_mid = list_claimed(&wallet_svc, &bob_id, "after rpc receive").await?;
    require_claim(&claimed_mid, asset_id, "after rpc receive")?;
    check_claim_set(&claimed_before, &claimed_mid)?;
    check_recv_rpc(&recv, asset_id)?;
    log_ok(
        lines,
        stage_id,
        "S5-5",
        "rpc_report_only",
        &format!(
            "public receive surface remained report-only; claimed_before={} claimed_after_rpc={}",
            claimed_before.len(),
            claimed_mid.len()
        ),
    )?;
    Ok(RpcCtx {
        bob_id,
        before_len: claimed_before.len(),
        rpc_status: recv.status,
    })
}

async fn list_claimed(
    wallet_svc: &Arc<WalletService>,
    wallet_id: &PersistWalletId,
    phase: &str,
) -> Result<Vec<Asset>, String> {
    wallet_svc
        .list_claimed_assets(wallet_id)
        .await
        .map_err(|e| format!("stage5: list claimed {phase} failed: {e}"))
}

fn check_recv_rpc(recv: &RuntimeReceiveAssetResponse, asset_id: [u8; 32]) -> Result<(), String> {
    if recv.asset.asset_id != asset_id {
        return Err(format!(
            "stage5: receive_asset returned mismatched asset_id: expected={} got={}",
            hex::encode(asset_id),
            hex::encode(recv.asset.asset_id)
        ));
    }
    if recv.status != ReceiveStatus::Detected.rpc_code() {
        return Err(format!(
            "stage5: unexpected rpc receive status: {}",
            recv.status
        ));
    }
    Ok(())
}

pub(super) fn build_recv_ctx(
    tx_pkg: TxPackage,
    out_idx: usize,
    receiver: &crate::SimActor,
) -> Result<RecvCtx, String> {
    let selected = select_bob_output(&tx_pkg.tx.outputs, out_idx)?;
    check_role(selected)?;
    let (asset, leaf) = parse_out(selected)?;
    let (pack, canon, runtime) = check_scan(receiver, &asset, &leaf)?;
    Ok(RecvCtx {
        tx_pkg,
        out_idx,
        asset,
        leaf,
        pack,
        canon,
        runtime,
    })
}
