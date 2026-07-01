#[cfg(feature = "wallet_debug_tools")]
use super::export_debug_dumps;
use super::runtime::{flush_logs, push_log};
use super::{
    apply_snap_fault, claim_rows_from_assets, create_dir_all, empty_stage1_bins, hex_str,
    import_claim_to_wallets, persist_claim_state_file, reconcile_snapshot, run_post_claim_export,
    save_json, verify_claim_conservation, write_audit_log, write_snapshot, ActorSnapItem, Asset,
    ClaimStateFile, ClaimStep, HashMap, LogRow, Path, SimContext, Stage3ClaimCfg, Stage3Snapshot,
};

pub(super) struct ClaimFinalizeArgs<'a> {
    pub(super) stage_id: u32,
    pub(super) cfg: &'a Stage3ClaimCfg,
    pub(super) state_path: &'a Path,
    pub(super) out_claim: &'a Path,
    pub(super) wallets_dir: &'a Path,
    pub(super) actor_idxs: &'a [usize],
    pub(super) assets: &'a [Asset],
    pub(super) assigned_total: usize,
    pub(super) actor_snaps: Vec<ActorSnapItem>,
    pub(super) per_actor_assets: Vec<Vec<Asset>>,
    pub(super) per_actor_claim_assets: Vec<Vec<Asset>>,
    pub(super) per_actor_deltas: Vec<HashMap<[u8; 32], u64>>,
    pub(super) claim_mode: &'a str,
    pub(super) compatibility_version: u32,
    pub(super) mode_str: &'a str,
    pub(super) rng_kind: &'a str,
    pub(super) consume_bins: bool,
}

pub(super) fn finish_claim_after_event(
    ctx: &mut SimContext,
    args: ClaimFinalizeArgs<'_>,
    claim_state: &mut ClaimStateFile,
    logs: &mut Vec<LogRow>,
) -> Result<(), String> {
    let ClaimFinalizeArgs {
        stage_id,
        cfg,
        state_path,
        out_claim,
        wallets_dir,
        actor_idxs,
        assets,
        assigned_total,
        actor_snaps,
        per_actor_assets,
        per_actor_claim_assets,
        per_actor_deltas,
        claim_mode,
        compatibility_version,
        mode_str,
        rng_kind,
        consume_bins,
    } = args;

    let import_report = import_claim_to_wallets(
        ctx,
        actor_idxs,
        &per_actor_claim_assets,
        wallets_dir,
        state_path,
        claim_state,
        cfg.resume_fault.as_deref(),
    )?;

    for row in &import_report.stats {
        ctx.logger.info(&format!(
            "stage3.actor_import: actor_id={} claimed_count={} skipped_count={} conflict_count={}",
            row.actor, row.inserted, row.already_exists, row.rejected
        ));
    }

    for (slot, assets) in import_report.accepted_assets.iter().enumerate() {
        let actor_idx = *actor_idxs
            .get(slot)
            .ok_or_else(|| format!("actor slot {slot} missing after import"))?;
        let actor = ctx
            .actors
            .get(actor_idx)
            .ok_or_else(|| format!("actor index {actor_idx} missing after import"))?;
        let actor_name = actor.name.to_ascii_lowercase();
        let file = out_claim.join(format!("claim_rows_{actor_name}.json"));
        let rows = claim_rows_from_assets(assets);
        save_json(&file, &rows).map_err(|e| e.to_string())?;
    }

    let imported: Vec<Asset> = per_actor_assets
        .iter()
        .flat_map(|rows| rows.iter().cloned())
        .collect();
    verify_claim_conservation(assets, &imported)
        .map_err(|e| format!("ConservationViolation: {e}"))?;

    let snap = Stage3Snapshot {
        stage: stage_id,
        claim_mode: claim_mode.to_string(),
        compatibility_version,
        mode: mode_str.to_string(),
        rng_kind: rng_kind.to_string(),
        consume_bins,
        input_assets_count: assets.len(),
        distributed_assets_count: assigned_total,
        actor_claims: actor_snaps,
        wallet_import_stats: import_report.stats.clone(),
        wallet_persist_stats: import_report.persist_stats.clone(),
    };
    let snap = apply_snap_fault(snap, cfg.snapshot_fault.as_deref())
        .map_err(|e| format!("SnapshotError: {e}"))?;
    reconcile_snapshot(&snap).map_err(|e| format!("SnapshotError: {e}"))?;
    write_snapshot(&ctx.outputs_dir.join(&cfg.paths.snapshot_file), &snap)
        .map_err(|e| format!("SnapshotError: {e}"))?;

    claim_state.step = ClaimStep::WalletsUpdated;
    persist_claim_state_file(state_path, claim_state)?;
    ctx.logger
        .info("stage3.wallets_updated: wallet.asset.import_asset completed");

    // The debug export lane stays opt-in and the JSON dump keeps wallet secrets
    // redacted while the default lane stays hardened; it must not be confused
    // with broader encrypted export or backup policy outside this artifact claim.
    #[cfg(feature = "wallet_debug_tools")]
    {
        export_debug_dumps(ctx, actor_idxs, &import_report.persisted_assets, out_claim)?;
        ctx.logger.info(&format!(
            "stage3.debug_dumps_written: outputs/{}/export_wallet_debug_*.json",
            cfg.paths.claim_dir
        ));
        push_log(
            logs,
            stage_id,
            "S3-3",
            "export_wallet_debug_toolss",
            "ok",
            "export_wallet_debug_{alice,bob,charlie}.json written",
        )?;
    }

    run_post_claim_export(ctx, actor_idxs, wallets_dir)?;
    push_log(
        logs,
        stage_id,
        "S3-3",
        "post_claim_export_import",
        "ok",
        "wallets_export_import post-claim export/import completed",
    )?;
    push_log(
        logs,
        stage_id,
        "S3-3",
        "persist_wallet_claim",
        "ok",
        "claim_rows json + stage_3_snapshot(with import stats)",
    )?;

    for (slot, deltas) in per_actor_deltas.into_iter().enumerate() {
        let actor_idx = *actor_idxs
            .get(slot)
            .ok_or_else(|| format!("actor slot {slot} missing"))?;
        let actor = ctx
            .actors
            .get_mut(actor_idx)
            .ok_or_else(|| format!("actor index {actor_idx} missing"))?;

        for (asset_id, amount) in deltas {
            let cur = actor.balance.get(&asset_id).copied().unwrap_or(0);
            let next = cur.checked_add(amount).ok_or_else(|| {
                format!(
                    "actor {} runtime balance overflow for asset {}",
                    actor.name,
                    hex_str(&asset_id)
                )
            })?;
            actor.balance.insert(asset_id, next);
        }
    }

    if consume_bins {
        empty_stage1_bins(&ctx.outputs_dir)?;
        claim_state.step = ClaimStep::BinsConsumed;
        persist_claim_state_file(state_path, claim_state)?;
        ctx.logger
            .info("stage3.bins_consumed: genesis .bin files cleared");
        push_log(
            logs,
            stage_id,
            "S3-4",
            "consume_bins",
            "ok",
            "genesis bins overwritten with empty vectors",
        )?;
    } else {
        push_log(
            logs,
            stage_id,
            "S3-4",
            "consume_bins",
            "ok",
            "skipped (consume_bins=false)",
        )?;
    }

    write_audit_log(&out_claim.join("audit_log.json"), &import_report.decisions)?;
    ctx.logger.info(&format!(
        "stage3.audit_written: file=claim/audit_log.json rows={}",
        import_report.decisions.len()
    ));

    let logs_dir = ctx.outputs_dir.join("logs");
    create_dir_all(&logs_dir).map_err(|e| e.to_string())?;
    flush_logs(&logs_dir.join("logger.json"), logs)?;

    ctx.logger.info(&format!(
        "stage3.done: stage={stage_id} distributed={assigned_total}"
    ));

    Ok(())
}
