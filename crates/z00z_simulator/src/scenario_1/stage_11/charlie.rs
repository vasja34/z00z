use std::{collections::BTreeMap, path::Path, sync::Arc};

use serde::de::DeserializeOwned;
use z00z_networks_rpc::RpcTransport;
use z00z_utils::{
    codec::{Codec, JsonCodec},
    io::{path_exists, read_file, save_json},
};
use z00z_wallets::{
    receiver::{ScanResult, StealthOutputScanner},
    tx::{TxOutRole, TxOutputWire},
};

use crate::scenario_1::stage_11::jmt_wallet_scan::{
    load_post_tx_candidate_set, verify_candidate, JmtScanArtifact, JmtScanRow,
};
use crate::scenario_1::stage_6::{
    build_confirm_rows, build_pending_rows_for_assets, build_wallet_diff, capture_wallet_actor,
    find_actor, merge_wallet_diff_dump, merge_wallet_state_dump, resolve_stage4_paths,
    validate_confirm_rows, wallet_amount_total, write_wallet_report_md, write_wallet_report_xlsx,
    SelectedInputRow,
};
use crate::SimContext;

use super::{CharlieApply, RuntimeOut, Stage11Cfg};

pub(super) fn refresh_charlie(
    ctx: &mut SimContext,
    cfg: &Stage11Cfg,
    bridge_outputs: &[TxOutputWire],
    tx_digest_hex: &str,
) -> Result<CharlieApply, String> {
    let stage4_paths = resolve_stage4_paths(ctx, &cfg.s4);
    let wallets_dir = stage4_paths.wallets_dir.clone();
    let rpc_log = stage4_paths.rpc_logger_file.clone();
    let before_state_file = stage4_paths
        .wallets_state_before_file
        .clone()
        .unwrap_or_else(|| {
            stage4_paths
                .transactions_dir
                .join("wallets_state_before.json")
        });
    let after_state_file = stage4_paths
        .wallets_state_after_file
        .clone()
        .unwrap_or_else(|| {
            stage4_paths
                .transactions_dir
                .join("wallets_state_after.json")
        });
    let diff_state_file = stage4_paths
        .wallets_state_diff_file
        .clone()
        .unwrap_or_else(|| {
            stage4_paths
                .transactions_dir
                .join("wallets_state_diff.json")
        });
    let selected_inputs_file = stage4_paths
        .transactions_dir
        .join("wallets_selected_inputs.json");
    let pending_file = stage4_paths.transactions_dir.join("wallets_pending.json");
    let confirmed_file = stage4_paths.transactions_dir.join("wallets_confirmed.json");
    let report_md_file = stage4_paths
        .wallets_state_report_md_file
        .clone()
        .unwrap_or_else(|| {
            stage4_paths
                .transactions_dir
                .join("wallets_state_report.md")
        });
    let report_xlsx_file = stage4_paths
        .wallets_state_report_xlsx_file
        .clone()
        .unwrap_or_else(|| {
            stage4_paths
                .transactions_dir
                .join("wallets_state_report.xlsx")
        });
    let rt = tokio::runtime::Runtime::new().map_err(|e| e.to_string())?;
    rt.block_on(async {
        crate::scenario_1::stage_2::lock_existing_wallet_sessions(ctx)
            .await
            .map_err(|e| format!("stage7: pre-refresh {e}"))
    })?;
    let (wallet_svc, transport) =
        crate::scenario_1::stage_2::build_logged_transport(ctx, &wallets_dir, &rpc_log)?;
    rt.block_on(async {
        crate::scenario_1::stage_2::reopen_wallet_sources(ctx, &wallet_svc, &wallets_dir)
            .await
            .map_err(|e| format!("stage7: {e}"))
    })?;
    ctx.wallet_service = Some(Arc::clone(&wallet_svc));
    let charlie = find_actor(ctx, "charlie")?;
    let before = rt.block_on(capture_wallet_actor(
        &transport,
        &cfg.s4,
        Some(&wallet_svc),
        charlie,
        &wallets_dir,
        "stage7_before",
    ))?;
    let (artifact, matched) = build_charlie_artifact(&cfg.out, charlie, bridge_outputs)?;
    if matched.is_empty() {
        let sample = artifact
            .rows
            .iter()
            .take(4)
            .map(|row| {
                format!(
                    "{}:{}:{}",
                    row.asset_id_hex, row.receive_status, row.owner_detected
                )
            })
            .collect::<Vec<_>>()
            .join(",");
        let output_ids = artifact
            .distinction
            .strip_prefix("output_ids=")
            .unwrap_or("unknown")
            .to_string();
        return Err(format!(
            "stage7: proof-validated committed scan found no Charlie-owned outputs; sample_rows={sample}; output_ids={output_ids}"
        ));
    }
    rt.block_on(import_actor_outputs(&transport, &cfg.s4, charlie, &matched))?;
    let after = rt.block_on(capture_wallet_actor(
        &transport,
        &cfg.s4,
        Some(&wallet_svc),
        charlie,
        &wallets_dir,
        "stage7_after",
    ))?;

    let pending_rows = build_pending_rows_for_assets(
        &charlie.name,
        &charlie.wallet_id,
        &matched
            .iter()
            .map(|item| item.asset.clone())
            .collect::<Vec<_>>(),
        tx_digest_hex,
        "pending_receive",
        Some(TxOutRole::Recipient),
    );
    let confirm_rows = build_confirm_rows(&pending_rows);
    validate_confirm_rows(&pending_rows, &confirm_rows)?;

    let report_pending_rows = pending_rows.clone();
    let report_confirm_rows = confirm_rows.clone();

    let diff = build_wallet_diff(7, &before, &after, &report_confirm_rows);
    let before_total = wallet_amount_total(&before, &charlie.wallet_id);
    let after_total = wallet_amount_total(&after, &charlie.wallet_id);
    let delta = after_total.saturating_sub(before_total);
    if delta != artifact.total_detected_amount {
        return Err(format!(
            "stage7: wallet invariant mismatch for charlie: before={} after={} delta={} detected={}",
            before_total,
            after_total,
            delta,
            artifact.total_detected_amount,
        ));
    }

    let merged_before = merge_wallet_state_dump(&before_state_file, &before)?;
    let merged_after = merge_wallet_state_dump(&after_state_file, &after)?;
    let merged_diff = merge_wallet_diff_dump(&diff_state_file, &diff)?;
    save_json(&before_state_file, &merged_before).map_err(|e| e.to_string())?;
    save_json(&after_state_file, &merged_after).map_err(|e| e.to_string())?;
    save_json(&diff_state_file, &merged_diff).map_err(|e| e.to_string())?;
    let merged_pending_rows = merge_saved_rows(&pending_file, &report_pending_rows)?;
    let merged_confirm_rows = merge_saved_rows(&confirmed_file, &report_confirm_rows)?;
    let selected_rows: Vec<SelectedInputRow> = load_saved_rows(&selected_inputs_file)?;
    save_json(&pending_file, &merged_pending_rows).map_err(|e| e.to_string())?;
    save_json(&confirmed_file, &merged_confirm_rows).map_err(|e| e.to_string())?;
    save_json(&cfg.wallet_scan_path, &artifact).map_err(|e| e.to_string())?;
    write_wallet_report_md(
        &report_md_file,
        7,
        &merged_before,
        &merged_after,
        &merged_diff,
        &selected_rows,
        &merged_pending_rows,
        &merged_confirm_rows,
    )?;
    write_wallet_report_xlsx(
        &report_xlsx_file,
        &merged_before,
        &merged_after,
        &merged_diff,
        &selected_rows,
        &merged_pending_rows,
        &merged_confirm_rows,
    )?;

    Ok(CharlieApply {
        artifact,
        invariant_ok: true,
    })
}

fn build_charlie_artifact(
    out: &Path,
    actor: &crate::SimActor,
    bridge_outputs: &[TxOutputWire],
) -> Result<(JmtScanArtifact, Vec<RuntimeOut>), String> {
    let mut out_map = BTreeMap::<String, RuntimeOut>::new();
    for output in bridge_outputs.iter().cloned() {
        let asset = output
            .asset_wire
            .clone()
            .to_asset()
            .map_err(|e| format!("stage7: output asset decode failed: {e}"))?;
        out_map.insert(
            hex::encode(asset.asset_id()),
            RuntimeOut {
                wire: output.clone(),
                asset,
            },
        );
    }

    let scanner = StealthOutputScanner::from_keys(&actor.keys);
    let loaded = load_post_tx_candidate_set(out)?;
    let store_root_hex = hex::encode(loaded.root.as_bytes());
    let skipped_non_asset_count = loaded.skipped_non_asset_count;
    let candidates = loaded.candidates;
    let mut rows = Vec::with_capacity(candidates.len());
    let mut matched = Vec::new();
    let mut total_detected_amount = 0u64;

    for candidate in &candidates {
        verify_candidate(candidate)?;
        let asset_id_hex = hex::encode(candidate.path.terminal_id().as_bytes());
        let matched_out = out_map.get(&asset_id_hex);
        let (report, owner_detected, amount) = if let Some(item) = matched_out {
            let scan = scanner.scan_leaf(&item.asset);
            let report = scan.recv_report();
            let owner_detected = matches!(scan, ScanResult::Mine { .. });
            let amount = owner_detected.then_some(item.asset.amount);
            if owner_detected {
                total_detected_amount = total_detected_amount.saturating_add(item.asset.amount);
                matched.push(item.clone());
            }
            (Some(report), owner_detected, amount)
        } else {
            (None, false, None)
        };

        rows.push(JmtScanRow {
            asset_id_hex,
            serial_id: candidate.path.serial_id.get(),
            proof_validated: true,
            receive_status: report
                .as_ref()
                .map(|item| item.status.rpc_code().to_string())
                .unwrap_or_else(|| "RUNTIME_ASSET_MISS".to_string()),
            receive_next: report
                .as_ref()
                .map(|item| format!("{:?}", item.next))
                .unwrap_or_else(|| "None".to_string()),
            receive_reject: report.and_then(|item| item.reject.map(|row| format!("{:?}", row))),
            owner_detected,
            amount,
            scan_path: "committed_post_tx_jmt".to_string(),
        });
    }

    let output_ids = out_map.keys().cloned().collect::<Vec<_>>().join(",");

    Ok((
        JmtScanArtifact {
            actor: actor.name.clone(),
            store_root_hex,
            scan_path: "jmt_scan".to_string(),
            proof_step: "proof_blob+chk_blob_settlement before runtime ownership detection"
                .to_string(),
            distinction: format!(
                "output_ids={output_ids}; This artifact proves committed-state JMT inclusion first, then replays runtime ownership detection; it is not equivalent to detached Stage 5 leaf scan."
            ),
            candidate_count: rows.len(),
            skipped_non_asset_count,
            proof_validated_count: rows.len(),
            detected_count: matched.len(),
            total_detected_amount,
            rows,
            status: "ok".to_string(),
        },
        matched,
    ))
}

fn merge_saved_rows<T>(path: &Path, new_rows: &[T]) -> Result<Vec<T>, String>
where
    T: Clone + DeserializeOwned,
{
    let mut rows = load_saved_rows(path)?;
    rows.extend_from_slice(new_rows);
    Ok(rows)
}

fn load_saved_rows<T>(path: &Path) -> Result<Vec<T>, String>
where
    T: DeserializeOwned,
{
    if !path_exists(path).map_err(|e| e.to_string())? {
        return Ok(Vec::new());
    }

    JsonCodec
        .deserialize(read_file(path).map_err(|e| e.to_string())?.as_slice())
        .map_err(|e| format!("stage7: decode {} failed: {e}", path.display()))
}

async fn import_actor_outputs(
    transport: &impl RpcTransport,
    cfg: &crate::config::Stage4TxPrepareCfg,
    actor: &crate::SimActor,
    outputs: &[RuntimeOut],
) -> Result<(), String> {
    let password = crate::scenario_1::stage_2::actor_runtime_password(actor)
        .ok_or_else(|| format!("stage7: no password for actor {}", actor.name))?;
    let session = transport
        .call(
            &cfg.rpc.unlock_method,
            z00z_utils::codec::json!({
                "wallet_id": actor.wallet_id,
                "password": password,
            }),
        )
        .await
        .map_err(|e| format!("stage7: unlock {} failed: {e}", actor.name))?;

    let run_res: Result<(), String> = async {
        for item in outputs {
            let asset_data = String::from_utf8(
                z00z_core::assets::encode_asset_pkg_json(&item.wire.asset_wire).map_err(|e| {
                    format!("stage7: serialize {} output dto failed: {e}", actor.name)
                })?,
            )
            .map_err(|e| format!("stage7: utf8 encode {} output dto failed: {e}", actor.name))?;
            let resp = transport
                .call(
                    &cfg.rpc.import_asset_method,
                    z00z_utils::codec::json!({
                        "session": session,
                        "asset_data": asset_data,
                    }),
                )
                .await
                .map_err(|e| format!("stage7: import {} asset RPC failed: {e}", actor.name))?;
            let success = resp
                .get("success")
                .and_then(|value| value.as_bool())
                .unwrap_or(true);
            let message = resp
                .get("message")
                .and_then(|value| value.as_str())
                .or_else(|| {
                    resp.get("status")
                        .and_then(|value| value.get("message"))
                        .and_then(|value| value.as_str())
                })
                .unwrap_or("");
            if !success && message != "asset_already_exists" {
                return Err(format!(
                    "stage7: import {} asset rejected: {}",
                    actor.name,
                    if message.is_empty() {
                        "unknown"
                    } else {
                        message
                    },
                ));
            }
        }
        Ok(())
    }
    .await;

    let lock_res = transport
        .call(
            &cfg.rpc.lock_method,
            z00z_utils::codec::json!({"session": session}),
        )
        .await;

    match run_res {
        Ok(()) => {
            lock_res.map_err(|e| format!("stage7: lock {} failed: {e}", actor.name))?;
            Ok(())
        }
        Err(err) => {
            if let Err(lock_err) = lock_res {
                return Err(format!(
                    "{err}; stage7: lock {} on failure failed: {lock_err}",
                    actor.name
                ));
            }
            Err(err)
        }
    }
}
