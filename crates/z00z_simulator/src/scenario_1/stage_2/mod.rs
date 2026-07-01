//! Scenario 1 stage 2 implementation: wallet_create.
//!
//! Creates three wallets (Alice, Bob, Charlie) via the RPC layer,
//! derives their stealth receiver keys from a deterministic mock RNG,
//! verifies the .wlt files exist on disk, and persists a JSON snapshot.

use std::{
    collections::HashMap,
    path::{Path, PathBuf},
    sync::{
        atomic::{AtomicU64, Ordering},
        Arc, Mutex, OnceLock,
    },
};

use serde::Serialize;
use z00z_crypto::expert::encoding::{from_hex, SafePassword};
use z00z_crypto::{aead, Hidden};
use z00z_networks_rpc::{LocalRpcTransport, RpcDispatcher, RpcTransport};
use z00z_utils::codec::json;

use z00z_utils::{
    codec::{Codec, JsonCodec},
    io::{create_dir_all, path_exists, read_file, save_json},
    rng::{MockRngProvider, RngCoreExt, SystemRngProvider},
    time::{format_system_time_local, MockTimeProvider, SystemTimeProvider, TimeProvider},
};
use z00z_wallets::{
    domains::hashing::compute_wallet_file_id,
    key::seed::{MnemonicLanguage, SeedPhrase24},
    key::Bip44Path,
    receiver::encode_card_compact,
    rpc::{
        logging::{build_rpc_file_logger, LoggedRpcTransport, RpcLoggingConfig},
        methods::{
            AppRpcImpl, AssetRpcImpl, BackupRpcImpl, ChainRpcImpl, ChainScanRpcImpl, KeyRpcImpl,
            NetworkRpcImpl, StorageRpcImpl, TxRpcImpl, WalletRpcImpl,
        },
        register_all_wallet_rpc_methods,
        types::{common::PersistWalletId, wallet::WalletSource},
    },
    wallet::{
        ChainId, WalletId, WalletKernel, WalletRecord, WalletSystemMetadata, WalletUserFields,
    },
    AppService, WalletService,
};

use crate::{config::Stage2ActorCfg, DesignStage, SimActor, SimContext, StageResult};

mod actors;
mod artifacts;
mod checks;
mod flow;
mod transport;

#[cfg(feature = "wallet_debug_tools")]
pub(crate) use self::artifacts::{norm_seed, read_seed_md};
pub(crate) use self::{
    actors::{actor_runtime_password, cfg_actors, cfg_net, set_actor_passwords, ActorSpec},
    artifacts::{
        debug_write_wallet_secrets_md, decode_export_salt, decrypt_seed_phrase,
        extract_receiver_ids, flush_logs, push_log, validate_rpc_log_privacy, write_wlt_map_txt,
        ActorRun, ActorSnap, Stage2Snap,
    },
    checks::{run_export_roundtrip, run_lock_check, run_restart_check},
    flow::{create_actor_runtime, enrich_actor_runtime},
    transport::{
        build_logged_transport, build_logged_transport_with_wallet, lock_existing_wallet_sessions,
        reopen_wallet_sources,
    },
};

pub fn set_actor_passwords_for_test(actors: &[Stage2ActorCfg]) {
    let specs: Vec<_> = actors
        .iter()
        .map(|row| ActorSpec {
            name: row.name.to_ascii_lowercase(),
            password: row.password.clone(),
            rng_seed: row.mock_rng_seed,
        })
        .collect();
    set_actor_passwords(&specs);
}

// ── Entry point ───────────────────────────────────────────────────────────────

/// Runs stage 2 (`wallet_create`).
pub fn run(ctx: &mut SimContext, stage: &DesignStage) -> StageResult {
    let rt = match tokio::runtime::Runtime::new() {
        Ok(r) => r,
        Err(e) => return StageResult::Fail(format!("tokio runtime: {e}")),
    };
    match rt.block_on(run_core(ctx, stage.stage)) {
        Ok(()) => StageResult::Ok,
        Err(e) => {
            ctx.logger.error(&format!(
                "[STAGE2-ERROR] stage {} ({}) failed: {}",
                stage.stage, stage.name, e
            ));
            StageResult::Fail(format!(
                "stage {} ({}) failed: {}",
                stage.stage, stage.name, e
            ))
        }
    }
}

// ── Core logic ────────────────────────────────────────────────────────────────

async fn run_core(ctx: &mut SimContext, stage_id: u32) -> Result<(), String> {
    let mut logs = Vec::new();
    let actors = cfg_actors(ctx);
    set_actor_passwords(&actors);
    let (wallet_net, wallet_chain) = cfg_net(ctx);

    // S2-1: Output dirs + env vars + RPC stack.
    let out = ctx.outputs_dir.clone();
    let p = ctx.config.stage2_paths();
    let wallets_dir = out.join(&p.wallets_dir);
    let keys_dir = out.join(&p.keys_dir);
    create_dir_all(&out).map_err(|e| e.to_string())?;
    create_dir_all(&wallets_dir).map_err(|e| e.to_string())?;
    create_dir_all(&keys_dir).map_err(|e| e.to_string())?;
    push_log(
        &mut logs,
        stage_id,
        "S2-1",
        "prepare_dirs",
        "ok",
        &out.to_string_lossy(),
    )?;

    // Required for AppService::create_wallet to know which network/chain to use.
    std::env::set_var("Z00Z_WALLET_NETWORK", &wallet_net);
    std::env::set_var("Z00Z_WALLET_CHAIN", &wallet_chain);
    push_log(
        &mut logs,
        stage_id,
        "S2-1",
        "env_vars",
        "ok",
        &format!(
            "Z00Z_WALLET_NETWORK={}  Z00Z_WALLET_CHAIN={}",
            wallet_net, wallet_chain
        ),
    )?;

    let log_dir = out.join(&p.logs_dir);
    create_dir_all(&log_dir).map_err(|e| e.to_string())?;
    let rpc_log_path = log_dir.join(&p.rpc_logger_file);
    let (wallet_svc, transport) = build_logged_transport(ctx, &wallets_dir, &rpc_log_path)?;
    ctx.wallet_service = Some(Arc::clone(&wallet_svc));

    push_log(
        &mut logs,
        stage_id,
        "S2-1",
        "rpc_stack",
        "ok",
        "LocalRpcTransport + LoggedRpcTransport ready",
    )?;

    // S2-2..S2-4: Create wallets via RPC, derive keys, build actors.
    let mut actor_snaps = Vec::new();
    let mut actor_runs = Vec::new();

    for spec in &actors {
        let (sim_actor, actor_snap, actor_run) = create_actor_runtime(
            ctx,
            &transport,
            &wallet_svc,
            spec,
            stage_id,
            &mut logs,
            &wallets_dir,
            &keys_dir,
            &wallet_net,
            &wallet_chain,
        )
        .await?;
        actor_snaps.push(actor_snap);
        ctx.actors.push(sim_actor);
        actor_runs.push(actor_run);
    }

    let wlt_map_file = wallets_dir.join("wlt_map.md");
    write_wlt_map_txt(&wlt_map_file, &actor_snaps)?;
    let alice_wlt_path = actor_snaps
        .iter()
        .find(|a| a.name == "alice")
        .map(|a| PathBuf::from(&a.wlt_path))
        .ok_or_else(|| "alice wlt path not found".to_string())?;
    push_log(
        &mut logs,
        stage_id,
        "S2-9",
        "write_wlt_map",
        "ok",
        &wlt_map_file.to_string_lossy(),
    )?;

    // S2-5: List wallets via RPC and verify the expected count.
    let list_resp = transport
        .call("app.wallet.list_wallets", json!({}))
        .await
        .map_err(|e| format!("list_wallets RPC: {e}"))?;

    let wallet_list = list_resp
        .as_array()
        .ok_or_else(|| "list_wallets: expected JSON array".to_string())?;

    if wallet_list.len() != actors.len() {
        return Err(format!(
            "expected {} wallets, got {}",
            actors.len(),
            wallet_list.len()
        ));
    }
    // Post-condition: verify all expected actor names are present.
    for item in &actors {
        if !wallet_list.iter().any(|w| {
            w["name"]
                .as_str()
                .map(|v| v.eq_ignore_ascii_case(&item.name))
                .unwrap_or(false)
        }) {
            return Err(format!(
                "list_wallets response missing expected name: {}",
                item.name
            ));
        }
    }
    push_log(
        &mut logs,
        stage_id,
        "S2-5",
        "list_wallets",
        "ok",
        &format!("count={}", wallet_list.len()),
    )?;

    for (idx, actor) in actor_runs.iter_mut().enumerate() {
        let sim_actor = ctx
            .actors
            .get_mut(idx)
            .ok_or_else(|| format!("missing sim actor at index {idx}"))?;
        enrich_actor_runtime(
            &transport,
            stage_id,
            &mut logs,
            &wallets_dir,
            actor,
            sim_actor,
            idx,
        )
        .await?;
    }

    let secrets_table = ctx.config.stage2_secret_artifact_path(&wallets_dir);

    if let Some(secrets_table) = secrets_table.as_ref() {
        debug_write_wallet_secrets_md(secrets_table, &actor_runs)?;
        push_log(
            &mut logs,
            stage_id,
            "S2-14",
            "debug_write_wallet_secrets",
            "ok",
            &secrets_table.to_string_lossy(),
        )?;
    } else {
        push_log(
            &mut logs,
            stage_id,
            "S2-14",
            "debug_write_wallet_secrets",
            "ok",
            "wallet_debug_tools disabled; default lane emitted no plaintext wallet secret artifact",
        )?;
        // This default-lane closure is intentionally narrow: encrypted export
        // and backup surfaces are separate from the plaintext debug-artifact lane.
    }

    let alice_run = actor_runs
        .iter()
        .find(|a| a.name == "alice")
        .ok_or_else(|| "alice runtime not found".to_string())?;
    let alice_wlt_bytes = read_file(&alice_wlt_path).map_err(|e| {
        format!(
            "read alice restart source {} failed: {e}",
            alice_wlt_path.display()
        )
    })?;

    run_export_roundtrip(
        ctx,
        &transport,
        stage_id,
        &mut logs,
        &out,
        &rpc_log_path,
        &wallet_net,
        &wallet_chain,
        &actor_runs,
        secrets_table.as_deref(),
    )
    .await?;

    run_restart_check(
        &wallet_svc,
        stage_id,
        &mut logs,
        alice_run,
        &alice_wlt_path,
        &wallets_dir,
        &alice_wlt_bytes,
    )
    .await?;

    run_lock_check(&transport, stage_id, &mut logs, alice_run).await?;

    // S2-15: Stage 3 skeleton - log readiness for stealth send/scan.
    push_log(
        &mut logs,
        stage_id,
        "S2-15",
        "stage3_skeleton",
        "ok",
        "ctx.actors ready for stealth send/scan",
    )?;

    validate_rpc_log_privacy(&rpc_log_path, &actor_runs)?;
    push_log(
        &mut logs,
        stage_id,
        "S2-19",
        "rpc_log_privacy_risk",
        "ok",
        "rpc logger redaction+risk checks passed",
    )?;

    // S2-11: Save stage snapshot — written LAST after all steps complete.
    let snap = Stage2Snap {
        stage: stage_id,
        chain_id: wallet_chain,
        wallet_count: ctx.actors.len(),
        out_dir: out.to_string_lossy().to_string(),
        actors_ready_for_stage3: true,
    };
    let snap_file = out.join(&p.snapshot_file);
    save_json(&snap_file, &snap).map_err(|e| e.to_string())?;
    push_log(
        &mut logs,
        stage_id,
        "S2-11",
        "write_snapshot",
        "ok",
        &snap_file.to_string_lossy(),
    )?;

    flush_logs(&log_dir.join(&p.logger_file), &logs)?;

    Ok(())
}

pub(crate) fn deterministic_seed_phrase_24(seed: u64) -> Result<String, String> {
    let mut rng = MockRngProvider::with_u64_seed(seed).rng();
    let mut entropy_bytes = [0u8; 32];
    rng.fill_bytes_ext(&mut entropy_bytes);

    let phrase = SeedPhrase24::from_bip39_entropy_bytes(&entropy_bytes, MnemonicLanguage::English)
        .map_err(|e| format!("seed phrase from entropy: {e}"))?;
    Ok(phrase.with_phrase(|s| s.to_string()))
}

fn hex_str(bytes: &[u8]) -> String {
    let mut out = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        out.push(nib_hex(byte >> 4));
        out.push(nib_hex(byte & 0x0f));
    }
    out
}

#[inline(always)]
fn nib_hex(nib: u8) -> char {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    HEX[nib as usize] as char
}
