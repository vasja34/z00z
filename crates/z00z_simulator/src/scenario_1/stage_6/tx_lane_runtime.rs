use super::{
    actor_runtime_password, build_logged_transport, build_wallet_diff, capture_wallet_states,
    create_dir_all, find_actor, flush_logs, list_sender_inputs_distinct_serials,
    load_claim_packages, load_stage4_verified_card, push_log, resolve_stage4_paths, save_json,
    stage4_cfg, stage4_flags_summary, to_hex, unique_serial_count, validate_stage4_cfg,
    write_wallet_report_md, write_wallet_report_xlsx, Arc, Codec, DesignStage, RpcTransport,
    SelectedInputRow, SimContext, Stage4Snap, PREP_FILE,
};

use super::tx_lane_runtime_flow::{
    persist_and_confirm_runtime, prepare_tx_package_artifacts, TxPrepArgs, TxRuntimeArgs,
};
use super::tx_lane_runtime_support::{
    apply_root_tamper, apply_test_tamper, apply_wit_tamper, fee_capture, resolve_fee_party,
};

pub(crate) use super::tx_lane_runtime_support::{
    validate_fee_sink, validate_tx_mode, Stage4ResolvedPaths,
};
use crate::scenario_1::stage_4::resolve_stage3_claim_pkg_file;

pub(super) async fn run_core(ctx: &mut SimContext, stage: &DesignStage) -> Result<(), String> {
    let cfg = stage4_cfg(ctx)?.clone();
    validate_stage4_cfg(ctx, &cfg)?;
    let paths = resolve_stage4_paths(ctx, &cfg);

    let out = paths.outputs_dir;
    let logs_dir = paths.logs_dir;
    let tx_dir = paths.transactions_dir;
    let wallets_dir = paths.wallets_dir;
    let tx_file = paths.tx_pkg_file;
    let prep_file = tx_file.parent().unwrap_or(tx_dir.as_path()).join(PREP_FILE);
    let snap_file = paths.snapshot_file;
    let log_file = paths.logger_file;
    let rpc_log_file = paths.rpc_logger_file;
    let before_state_file = paths
        .wallets_state_before_file
        .clone()
        .unwrap_or_else(|| tx_dir.join("wallets_state_before.json"));
    let after_state_file = paths
        .wallets_state_after_file
        .clone()
        .unwrap_or_else(|| tx_dir.join("wallets_state_after.json"));
    let diff_state_file = paths
        .wallets_state_diff_file
        .clone()
        .unwrap_or_else(|| tx_dir.join("wallets_state_diff.json"));
    let selected_inputs_file = tx_dir.join("wallets_selected_inputs.json");
    let pending_file = tx_dir.join("wallets_pending.json");
    let confirmed_file = tx_dir.join("wallets_confirmed.json");
    let md_report_file = paths
        .wallets_state_report_md_file
        .clone()
        .unwrap_or_else(|| tx_dir.join("wallets_state_report.md"));
    let xlsx_report_file = paths
        .wallets_state_report_xlsx_file
        .clone()
        .unwrap_or_else(|| tx_dir.join("wallets_state_report.xlsx"));

    let claim_pkg_file = resolve_stage3_claim_pkg_file(ctx);
    let claim_pkgs = load_claim_packages(&claim_pkg_file).map_err(|err| {
        format!(
            "stage4: claim package prerequisite failed at {}: {err}",
            claim_pkg_file.display()
        )
    })?;

    create_dir_all(&out).map_err(|e| e.to_string())?;
    create_dir_all(&logs_dir).map_err(|e| e.to_string())?;
    create_dir_all(&tx_dir).map_err(|e| e.to_string())?;

    let mut lines = Vec::new();
    push_log(
        &mut lines,
        stage.stage,
        "S4-1",
        "prepare_dirs",
        "ok",
        &format!(
            "{} flags={}",
            out.to_string_lossy(),
            stage4_flags_summary(&cfg)
        ),
    )?;

    // Inputs from wallet .wlt through configured RPC methods
    crate::scenario_1::stage_2::lock_existing_wallet_sessions(ctx).await?;
    let (wallet_svc, transport) = build_logged_transport(ctx, &wallets_dir, &rpc_log_file)?;
    crate::scenario_1::stage_2::reopen_wallet_sources(ctx, &wallet_svc, &wallets_dir).await?;
    ctx.wallet_service = Some(Arc::clone(&wallet_svc));
    let sender = find_actor(ctx, &cfg.sender_actor)?;
    let recipient = find_actor(ctx, &cfg.receiver_actor)?;
    let sender_password = actor_runtime_password(sender)
        .ok_or_else(|| format!("stage4: no password for actor {}", sender.name))?;
    let session = transport
        .call(
            &cfg.rpc.unlock_method,
            z00z_utils::codec::json!({
                "wallet_id": sender.wallet_id,
                "password": sender_password,
            }),
        )
        .await
        .map_err(|e| format!("stage4: unlock sender RPC failed: {e}"))?;

    let sender_recv_sec = *sender.receiver_secret.reveal();

    let sender_scan =
        list_sender_inputs_distinct_serials(&transport, &cfg, &sender.wallet_id, sender_recv_sec)
            .await;

    let sender_lock_res = transport
        .call(
            &cfg.rpc.lock_method,
            z00z_utils::codec::json!({"session": session}),
        )
        .await;

    let mut selected = match sender_scan {
        Ok(rows) => {
            sender_lock_res.map_err(|e| format!("stage4: lock sender RPC failed: {e}"))?;
            rows
        }
        Err(err) => {
            if let Err(lock_err) = sender_lock_res {
                return Err(format!(
                    "{err}; stage4: sender lock on failure RPC failed: {lock_err}"
                ));
            }
            return Err(err);
        }
    };

    let selected_dump: Vec<SelectedInputRow> = selected
        .iter()
        .map(|row| {
            let leaf = z00z_wallets::tx::asset_wire_to_leaf(row)?;
            Ok(SelectedInputRow {
                actor: sender.name.clone(),
                wallet_id: sender.wallet_id.clone(),
                asset_id_hex: to_hex(&leaf.asset_id),
                serial_id: row.serial_id,
                class: format!("{:?}", row.definition.class),
                symbol: row.definition.symbol.clone(),
                amount: row.amount,
            })
        })
        .collect::<Result<Vec<_>, String>>()?;
    save_json(&selected_inputs_file, &selected_dump).map_err(|e| e.to_string())?;

    push_log(
        &mut lines,
        stage.stage,
        "S4-2",
        "load_wallet_inputs",
        "ok",
        &format!(
            "selected_inputs={} distinct_serials={} with_stealth_fields={} with_secret={}",
            selected.len(),
            unique_serial_count(&selected),
            selected
                .iter()
                .filter(|row| row.r_pub.is_some()
                    && row.owner_tag.is_some()
                    && row.enc_pack.is_some()
                    && row.tag16.is_some())
                .count(),
            selected.iter().filter(|row| row.secret.is_some()).count()
        ),
    )?;
    ctx.logger.info(&format!(
        "stage4.inputs_diag: selected={} distinct={} with_stealth={} with_secret={}",
        selected.len(),
        unique_serial_count(&selected),
        selected
            .iter()
            .filter(|row| {
                row.r_pub.is_some()
                    && row.owner_tag.is_some()
                    && row.enc_pack.is_some()
                    && row.tag16.is_some()
            })
            .count(),
        selected.iter().filter(|row| row.secret.is_some()).count()
    ));

    let fee_party = resolve_fee_party(ctx, &transport, &cfg).await?;

    let before_dump = capture_wallet_states(
        &transport,
        &cfg,
        ctx,
        &wallets_dir,
        "before",
        fee_capture(&ctx.actors, &fee_party),
    )
    .await?;
    save_json(&before_state_file, &before_dump).map_err(|e| e.to_string())?;
    push_log(
        &mut lines,
        stage.stage,
        "S4-12",
        "write_wallet_state_before",
        "ok",
        &before_state_file.to_string_lossy(),
    )?;

    // Public ReceiverCard values from stage 2.
    let alice_card = load_stage4_verified_card(&paths.alice_keys_file)?;
    let bob_card = load_stage4_verified_card(&paths.bob_keys_file)?;
    let fee_card = fee_party.card.clone();
    push_log(
        &mut lines,
        stage.stage,
        "S4-3",
        "load_card_compact",
        "ok",
        "alice_keys.json + bob_keys.json via card_compact verify",
    )?;

    let prepared = prepare_tx_package_artifacts(
        ctx,
        stage,
        TxPrepArgs {
            cfg: &cfg,
            sender,
            recipient,
            alice_card: &alice_card,
            bob_card: &bob_card,
            fee_card: &fee_card,
            selected: &mut selected,
            sender_recv_sec,
            claim_pkgs: &claim_pkgs,
            out: &out,
            tx_dir: &tx_dir,
            tx_file: &tx_file,
            prep_file: &prep_file,
            lines: &mut lines,
        },
    )?;

    let pending = persist_and_confirm_runtime(
        stage,
        &transport,
        TxRuntimeArgs {
            cfg: &cfg,
            sender,
            recipient,
            fee_party: &fee_party,
            selected: &selected,
            outputs: &prepared.outputs,
            tx_outputs: &prepared.tx_outputs,
            pkg: &prepared.pkg,
            wallets_dir: &wallets_dir,
            pending_file: &pending_file,
            confirmed_file: &confirmed_file,
            lines: &mut lines,
        },
    )
    .await?;

    let after_dump = capture_wallet_states(
        &transport,
        &cfg,
        ctx,
        &wallets_dir,
        "after",
        fee_capture(&ctx.actors, &fee_party),
    )
    .await?;
    save_json(&after_state_file, &after_dump).map_err(|e| e.to_string())?;
    let diff_dump = build_wallet_diff(
        stage.stage,
        &before_dump,
        &after_dump,
        &pending.confirmed_rows,
    );
    save_json(&diff_state_file, &diff_dump).map_err(|e| e.to_string())?;
    write_wallet_report_md(
        &md_report_file,
        stage.stage,
        &before_dump,
        &after_dump,
        &diff_dump,
        &selected_dump,
        &pending.pending_rows,
        &pending.confirmed_rows,
    )?;
    write_wallet_report_xlsx(
        &xlsx_report_file,
        &before_dump,
        &after_dump,
        &diff_dump,
        &selected_dump,
        &pending.pending_rows,
        &pending.confirmed_rows,
    )?;
    push_log(
        &mut lines,
        stage.stage,
        "S4-12",
        "write_wallet_state_after",
        "ok",
        &format!(
            "{} {} {} {} {}",
            after_state_file.to_string_lossy(),
            diff_state_file.to_string_lossy(),
            md_report_file.to_string_lossy(),
            xlsx_report_file.to_string_lossy(),
            selected_inputs_file.to_string_lossy()
        ),
    )?;

    let snap = Stage4Snap {
        stage: stage.stage,
        tx_count: 1,
        output_count: prepared.pkg.tx.outputs.len() as u32,
        tx_digest_hex: prepared.pkg.tx_digest_hex.clone(),
        status: "ok".to_string(),
    };
    save_json(&snap_file, &snap).map_err(|e| e.to_string())?;
    push_log(
        &mut lines,
        stage.stage,
        "S4-11",
        "write_snapshot",
        "ok",
        &snap_file.to_string_lossy(),
    )?;

    push_log(
        &mut lines,
        stage.stage,
        "S4-13",
        "stage_complete",
        "ok",
        "all gates passed",
    )?;

    flush_logs(&log_file, &lines)?;
    Ok(())
}
