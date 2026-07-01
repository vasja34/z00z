use super::finalize::{finish_claim_after_event, ClaimFinalizeArgs};
use super::{
    asset_wire_to_leaf, assign_class_split, assign_coin_sets, assign_uniform_all,
    build_claim_package, claim_cfg, count_assigned, create_dir_all, derive_output_nonce,
    format_system_time_local, has_restored_bins, hex_str, load_claim_state_file, load_json,
    load_stage1_bins, patch_claim_bundle_membership_in, path_exists, persist_claim_state_file,
    read_to_string, rehydrate_rows, remove_file, resolve_actor_idxs, save_json, stage3_rng_mode,
    to_claim_wire, verify_pending, verify_resume_wire, write_claim_bundle, ActorClaimSummary,
    ActorSnapItem, Asset, ClaimGenesisEvent, ClaimMode, ClaimRow, ClaimStateFile, ClaimStep,
    ClaimTxPackage, Codec, HashMap, HashSet, JsonCodec, LogRow, Path, SimContext, Stage1SnapRef,
    SystemTimeProvider, TimeProvider,
};

pub(in crate::scenario_1::stage_3) fn run_core_impl(
    ctx: &mut SimContext,
    stage_id: u32,
) -> Result<(), String> {
    let mut logs = Vec::new();
    let rng_mode = stage3_rng_mode(ctx);
    let cfg = claim_cfg(ctx);
    let mode = ClaimMode::from_active(cfg.active.as_deref());
    let consume_bins = cfg.consume_bins.unwrap_or(true);
    let claim_mode = "wallet_only_intermediate".to_string();
    let compatibility_version = 1u32;

    let run_id = format!(
        "{}|{}|consume={}",
        mode.mode_str(),
        rng_mode.kind_str(),
        consume_bins
    );

    let out_gen = ctx.outputs_dir.join("genesis");
    let out_claim = ctx.outputs_dir.join(&cfg.paths.claim_dir);
    let wallets_dir = ctx.outputs_dir.join(&cfg.paths.wallets_dir);

    create_dir_all(&out_gen).map_err(|e| e.to_string())?;
    create_dir_all(&out_claim).map_err(|e| e.to_string())?;
    create_dir_all(&ctx.outputs_dir).map_err(|e| e.to_string())?;
    create_dir_all(&wallets_dir).map_err(|e| e.to_string())?;

    let stale_report_path = out_claim.join("claim_wallet_import_report.json");
    if path_exists(&stale_report_path).map_err(|e| e.to_string())? {
        remove_file(&stale_report_path).map_err(|e| e.to_string())?;
    }

    let state_path = out_gen.join(&cfg.paths.claim_state_file);
    let prev_state = load_claim_state_file(&state_path)?;
    if let Some(prev) = prev_state.as_ref() {
        if prev.step == ClaimStep::BinsConsumed {
            let bins_restored = has_restored_bins(&ctx.outputs_dir);
            let allow_rerun = bins_restored || !ctx.config.simulation.abort_on_fail;
            if !allow_rerun {
                return Err(format!(
                    "claim already completed (step=BinsConsumed, run_id={}); restore genesis bins or clear claim_state.json to rerun",
                    prev.run_id
                ));
            }
            ctx.logger.warn(
                "stage3.resume_override: bins restored or abort_on_fail=false -> allowing re-run",
            );
        } else {
            ctx.logger.warn(&format!(
                "stage3.resume_detected: prev_step={:?} prev_run_id={}",
                prev.step, prev.run_id
            ));
            rehydrate_rows(&prev.claimed_rows)?;
        }
    }

    let started_at = SystemTimeProvider.compat_unix_timestamp();
    let base_state = ClaimStateFile {
        run_id: run_id.clone(),
        mode: mode.mode_str().to_string(),
        rng_kind: rng_mode.kind_str(),
        step: ClaimStep::Started,
        started_at_unix: started_at,
        claimed_rows: Vec::new(),
    };
    let is_resume = prev_state.is_some();
    let mut claim_state = prev_state.unwrap_or(base_state.clone());
    if claim_state.claimed_rows.is_empty() {
        claim_state.claimed_rows = Vec::new();
    }
    if claim_state.step == ClaimStep::Started {
        persist_claim_state_file(&state_path, &claim_state)?;
    }

    ctx.logger.info(&format!(
        "stage3.start: stage={stage_id} mode={} rng={} consume_bins={consume_bins}",
        mode.mode_str(),
        rng_mode.kind_str()
    ));
    ctx.logger.info(
        "stage3.rng_scope: rng_mode drives asset distribution only; claim-output formulas stay wallet-owned while SenderWallet cache remains per-output",
    );
    ctx.logger
        .info("stage3.mode_notice: NOT_CONSENSUS_PATH wallet_only_intermediate");
    let actor_idxs = resolve_actor_idxs(ctx)?;
    let mut assets = load_stage1_bins(&ctx.outputs_dir)?;

    let stage1_snap_path = ctx.outputs_dir.join("stage_1_snapshot.json");
    let stage1_snap: Stage1SnapRef =
        load_json(&stage1_snap_path).map_err(|e| format!("stage_1_snapshot load failed: {e}"))?;
    if assets.len() != stage1_snap.assets_count {
        return Err(format!(
            "S3-1 mismatch: loaded assets={} != stage_1_snapshot.assets_count={}",
            assets.len(),
            stage1_snap.assets_count
        ));
    }

    ctx.logger
        .info(&format!("stage3.bins_loaded: count={}", assets.len()));
    push_log(
        &mut logs,
        stage_id,
        "S3-1",
        "load_genesis_bins",
        "ok",
        &format!("count={}", assets.len()),
    )?;

    if assets.is_empty() {
        return Err("no genesis assets found in .bin files".to_string());
    }

    let mut assigned: Vec<Vec<Asset>> = vec![Vec::new(), Vec::new(), Vec::new()];
    let mut rng = rng_mode.make_rng();
    match mode {
        ClaimMode::ClassSplit => assign_class_split(&assets, &mut assigned),
        ClaimMode::CoinSets => assign_coin_sets(&assets, &mut assigned, &mut *rng),
        ClaimMode::UniformAll => assign_uniform_all(&mut assets, &mut assigned, &mut *rng),
    }

    let assigned_total = count_assigned(&assigned);
    if assigned_total != assets.len() {
        return Err(format!(
            "assignment count mismatch: input={} assigned={assigned_total}",
            assets.len()
        ));
    }
    push_log(
        &mut logs,
        stage_id,
        "S3-2",
        "distribute_assets",
        "ok",
        &format!("input={} assigned={assigned_total}", assets.len()),
    )?;

    let mut actor_snaps: Vec<ActorSnapItem> = Vec::new();
    let mut actor_event_items: Vec<ActorClaimSummary> = Vec::new();
    let mut per_actor_rows: Vec<Vec<ClaimRow>> = vec![Vec::new(), Vec::new(), Vec::new()];
    let mut per_actor_assets: Vec<Vec<Asset>> = vec![Vec::new(), Vec::new(), Vec::new()];
    let mut per_actor_claim_assets: Vec<Vec<Asset>> = vec![Vec::new(), Vec::new(), Vec::new()];
    let mut claim_pkg_artifacts: Vec<ClaimTxPackage> = Vec::new();
    let mut per_actor_deltas: Vec<HashMap<[u8; 32], u64>> =
        vec![HashMap::new(), HashMap::new(), HashMap::new()];

    for (slot, actor_assets) in assigned.into_iter().enumerate() {
        let actor_idx = *actor_idxs
            .get(slot)
            .ok_or_else(|| format!("actor slot {slot} missing"))?;

        let actor = ctx
            .actors
            .get(actor_idx)
            .ok_or_else(|| format!("actor index {actor_idx} missing"))?;
        let actor_name_lower = actor.name.to_lowercase();
        let actor_wallet_id = actor.wallet_id.clone();
        let recipient_owner = actor.card.owner_handle;

        let rows = &mut per_actor_rows[slot];
        rows.reserve(actor_assets.len());
        let mut total_amount = 0u64;
        let mut uniq_ids: HashSet<[u8; 32]> = HashSet::new();

        for (asset_pos, asset) in actor_assets.into_iter().enumerate() {
            let source_asset_id = asset.asset_id();
            let claim_id_bytes = derive_output_nonce(&asset.definition.id, asset.serial_id);
            let claim_wire = to_claim_wire(&asset, &actor.keys, &actor.card, &claim_id_bytes)
                .map_err(|e| {
                    format!(
                        "claim leaf build failed: actor={} asset_id={} err={e}",
                        actor_name_lower,
                        hex_str(&source_asset_id),
                    )
                })?;
            let claim_asset = claim_wire.clone().to_asset().map_err(|e| {
                format!(
                    "claim asset decode failed: actor={} asset_id={} err={e}",
                    actor_name_lower,
                    hex_str(&source_asset_id),
                )
            })?;
            let claim_asset_id = claim_asset.asset_id();
            total_amount = total_amount.checked_add(asset.amount).ok_or_else(|| {
                format!(
                    "actor {} amount overflow while summing claimed assets",
                    actor_name_lower
                )
            })?;
            uniq_ids.insert(claim_asset_id);
            per_actor_assets[slot].push(asset.clone());
            per_actor_claim_assets[slot].push(claim_asset.clone());
            let prev = per_actor_deltas[slot]
                .get(&claim_asset.definition.id)
                .copied()
                .unwrap_or(0);
            let next = prev.checked_add(claim_asset.amount).ok_or_else(|| {
                format!(
                    "actor {} balance delta overflow for definition {}",
                    actor_name_lower,
                    hex_str(&claim_asset.definition.id)
                )
            })?;
            per_actor_deltas[slot].insert(claim_asset.definition.id, next);

            rows.push(ClaimRow {
                asset_id: hex_str(&claim_asset_id),
                symbol: claim_asset.definition.symbol.clone(),
                class: claim_asset.definition.class.to_string(),
                serial_id: claim_asset.serial_id,
                amount: claim_asset.amount,
            });

            if asset.amount > 0 {
                let tx_nonce = ((slot as u64) << 32) | (asset_pos as u64);
                let asset_leaf = claim_wire;
                verify_resume_wire(&asset_leaf, &actor.keys).map_err(|e| {
                    format!(
                        "claim leaf verify failed: actor={} asset_id={} err={e}",
                        actor_name_lower,
                        hex_str(&source_asset_id),
                    )
                })?;
                let claim_leaf = asset_wire_to_leaf(&asset_leaf).map_err(|e| {
                    format!(
                        "claim leaf canonicalization failed: actor={} asset_id={} err={e}",
                        actor_name_lower,
                        hex_str(&source_asset_id),
                    )
                })?;
                let chain_type = ctx.config.chain.as_str();
                let chain_id = match chain_type {
                    "mainnet" => 1,
                    "testnet" => 2,
                    "devnet" => 3,
                    other => return Err(format!("unsupported scenario chain type: {other}")),
                };
                let chain_name = format!("z00z-{chain_type}-1");
                let pkg = build_claim_package(
                    chain_id,
                    chain_type,
                    &chain_name,
                    &actor_wallet_id,
                    &hex::encode(claim_leaf.asset_id),
                    asset.amount,
                    &claim_id_bytes,
                    &recipient_owner,
                    tx_nonce,
                    Some(asset_leaf),
                    Some(&actor.keys),
                )
                .map_err(|e| {
                    format!(
                        "claim package build failed: actor={} asset_id={} err={e}",
                        actor_name_lower,
                        hex_str(&source_asset_id),
                    )
                })?;
                let pkg_obj: ClaimTxPackage = JsonCodec
                    .deserialize(&pkg)
                    .map_err(|e| format!("claim package decode failed: {e}"))?;
                claim_pkg_artifacts.push(pkg_obj);
            }
        }

        let file = out_claim.join(format!("claim_rows_{actor_name_lower}.json"));
        save_json(&file, rows).map_err(|e| e.to_string())?;

        let actor_display = ctx
            .actors
            .get(actor_idx)
            .ok_or_else(|| format!("actor index {actor_idx} missing"))?
            .name
            .clone();

        ctx.logger.info(&format!(
            "stage3.actor_claim: actor={actor_display} assets={} total={total_amount}",
            rows.len()
        ));

        actor_snaps.push(ActorSnapItem {
            name: actor_display.clone(),
            assets_count: rows.len(),
            total_amount,
            unique_terminal_ids: uniq_ids.len(),
        });

        actor_event_items.push(ActorClaimSummary {
            name: actor_display,
            assets_count: rows.len(),
            total_amount,
            unique_terminal_ids: uniq_ids.len(),
        });
    }

    let claim_pkg_count = claim_pkg_artifacts.len();
    let claim_pkg_path = out_claim.join("tx_claim_pkg.json");
    patch_claim_bundle_membership_in(&mut claim_pkg_artifacts, &out_claim)
        .map_err(|e| format!("claim bundle membership patch failed: {e}"))?;
    write_claim_bundle(&out_claim, claim_pkg_artifacts)?;
    ctx.logger.info(&format!(
        "stage3.claim_pkg_written: file={} count={}",
        claim_pkg_path.display(),
        claim_pkg_count
    ));

    claim_state.step = ClaimStep::ArtifactsWritten;
    persist_claim_state_file(&state_path, &claim_state)?;
    ctx.logger.info(&format!(
        "stage3.artifacts_written: total_distributed={assigned_total}"
    ));

    if is_resume {
        verify_pending(ctx, &actor_idxs, &per_actor_claim_assets, &claim_state)?;
    }

    let claim_event = ClaimGenesisEvent {
        event: "claim_genesis_assets",
        scenario_id: ctx.config.scenario.id,
        stage: stage_id,
        claim_mode: claim_mode.clone(),
        compatibility_version,
        mode: mode.mode_str().to_string(),
        rng_kind: rng_mode.kind_str(),
        input_assets: assets.len(),
        distributed: assigned_total,
        actor_claims: actor_event_items,
        timestamp_unix: SystemTimeProvider.compat_unix_timestamp(),
    };
    claim_event.emit(&ctx.outputs_dir.join("events"))?;
    ctx.logger
        .info("stage3.event_emitted: events/claim_genesis.event.json");

    finish_claim_after_event(
        ctx,
        ClaimFinalizeArgs {
            stage_id,
            cfg: &cfg,
            state_path: &state_path,
            out_claim: &out_claim,
            wallets_dir: &wallets_dir,
            actor_idxs: &actor_idxs,
            assets: &assets,
            assigned_total,
            actor_snaps,
            per_actor_assets,
            per_actor_claim_assets,
            per_actor_deltas,
            claim_mode: &claim_mode,
            compatibility_version,
            mode_str: mode.mode_str(),
            rng_kind: &rng_mode.kind_str(),
            consume_bins,
        },
        &mut claim_state,
        &mut logs,
    )?;

    Ok(())
}

pub(super) fn push_log(
    logs: &mut Vec<LogRow>,
    stage: u32,
    step: &str,
    event: &str,
    status: &str,
    detail: &str,
) -> Result<(), String> {
    logs.push(LogRow {
        timestamp: format_system_time_local(SystemTimeProvider.now()),
        stage,
        step: step.to_string(),
        event: event.to_string(),
        status: status.to_string(),
        detail: detail.to_string(),
    });
    Ok(())
}

pub(super) fn flush_logs(path: &Path, logs: &[LogRow]) -> Result<(), String> {
    let mut out = String::new();
    if path_exists(path).map_err(|e| e.to_string())? {
        out = read_to_string(path).map_err(|e| e.to_string())?;
        if !out.is_empty() && !out.ends_with('\n') {
            out.push('\n');
        }
    }
    for row in logs {
        let line = JsonCodec.serialize(row).map_err(|e| e.to_string())?;
        let line = String::from_utf8(line).map_err(|e| e.to_string())?;
        out.push_str(&line);
        out.push('\n');
    }
    z00z_utils::io::write_file(path, out.as_bytes()).map_err(|e| e.to_string())
}
