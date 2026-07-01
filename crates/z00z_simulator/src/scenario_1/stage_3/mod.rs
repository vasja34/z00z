//! Scenario 1 stage 3 implementation: distribute genesis assets to simulator actors.
//!
//! Features:
//! - Resume-safe `claim_state.json` artifact/checkpoint flow with four checkpoints:
//!   `Started → ArtifactsWritten → WalletsUpdated → BinsConsumed`.
//!   Stage 3 currently instantiates `SenderWallet` per output, so duplicate-`R`
//!   cache state is call-local rather than shared across the batch or resume
//!   boundary.
//! - Structured logging via [`SimContext::logger`].
//! - RPC-style [`ClaimGenesisEvent`] emitted to `events/claim_genesis.event.json`.
//! - RNG mode dispatch for asset distribution only:
//!   `mock_rng_seed: Some(s)` → `MockRngProvider`,
//!   `None` → `SystemRngProvider`.

use std::collections::{HashMap, HashSet};
use std::path::Path;
use std::sync::Arc;

use rand::RngCore;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use z00z_core::{assets::AssetWire, Asset};
use z00z_crypto::{create_range_proof, Z00ZCommitment, Z00ZScalar};
use z00z_networks_rpc::RpcTransport;
use z00z_utils::{
    codec::{Codec, JsonCodec},
    io::{
        create_dir_all, load_bincode, load_json, path_exists, read_file, read_to_string,
        remove_file, rename_file, save_bincode, save_json, write_file,
    },
    rng::RngCoreExt,
    time::{format_system_time_local, SystemTimeProvider, TimeProvider},
};
use z00z_wallets::claim::derive_nullifier;
use z00z_wallets::stealth::{build_card_output_serial_checked, BuildCheck, SenderWallet};
use z00z_wallets::{
    claim::registry as claim_registry,
    key::ReceiverKeys,
    receiver::{
        PinEntry, PinnedReceiverCards, ReceiverCard, ScanResult, StealthOutputScanner, TrustLevel,
    },
    tx::{asset_wire_to_leaf, derive_output_nonce, ClaimTxPackage},
};

use crate::{
    config::{Stage3ClaimCfg, Stage3PathsCfg},
    event::{ActorClaimSummary, ClaimGenesisEvent},
    rng_mode::RngMode,
    DesignStage, SimContext, StageResult,
};

mod audit;
mod bins;
mod claim_pkg;
mod distribution;
mod finalize;
mod post_claim;
pub(super) mod runtime;
mod snapshot;
mod state;
mod wallet_flow;
mod wallet_flow_restart;

pub(super) use self::audit::{parse_reason_code, write_audit_log, AuditLogRow};
pub(super) use self::bins::{
    empty_stage1_bins, has_restored_bins, load_stage1_bins, resolve_actor_idxs,
};
pub use self::claim_pkg::{
    build_claim_package, patch_claim_bundle_membership, patch_claim_bundle_membership_in,
    write_claim_bundle, write_claim_bundle_store, CLAIM_STORE_FILE,
};
pub use self::claim_pkg::{build_claim_package_fault, write_claim_bundle_fault};
pub(super) use self::distribution::{
    assign_class_split, assign_coin_sets, assign_uniform_all, count_assigned, want_half_abort,
    want_reject_first, want_replay_first, ClaimMode,
};
#[cfg(feature = "wallet_debug_tools")]
use self::post_claim::export_debug_dumps;
pub(super) use self::post_claim::run_post_claim_export;
pub(super) use self::snapshot::{apply_snap_fault, ClaimRow};
pub(super) use self::state::{
    claim_row_exists, load_claim_state_file, persist_claim_state_file, push_claim_row,
    rehydrate_rows,
};
pub(super) use self::wallet_flow::import_claim_to_wallets;

pub use self::snapshot::{
    reconcile_snapshot, write_snapshot, ActorPersistStat, ActorSnapItem, SnapshotError,
    Stage3Snapshot, WalletImportStat,
};
pub use self::state::{
    merge_state_files, rehydrate_rows_from_state, verify_resume_wire, ClaimStateFile,
    ClaimStateRow, ClaimStep,
};

const EMIT_BIND_MISMATCH: &str = "bind_mismatch";
const EMIT_PROOF_FAIL: &str = "proof_fail";
const EMIT_AUTH_FAIL: &str = "auth_fail";
const WRITE_SERIALIZE_FAIL: &str = "serialize_fail";
const WRITE_IO_FAIL: &str = "write_fail";
const WRITE_VERIFY_FAIL: &str = "verify_fail";

#[doc(hidden)]
pub fn to_claim_wire(
    asset: &Asset,
    keys: &ReceiverKeys,
    card: &ReceiverCard,
    tx_seed: &[u8; 32],
) -> Result<AssetWire, String> {
    let mut owned = z00z_core::genesis::asset_std::asset_from_dev_class(
        asset.definition.class,
        asset.serial_id,
        asset.amount,
    )
    .map_err(|e| format!("asset_from_dev_class failed: {e}"))?;
    let mut def = (*asset.definition).clone();
    def.id = asset.definition.id;
    owned.definition = Arc::new(def);
    owned.serial_id = asset.serial_id;
    let mut sender_wallet = SenderWallet::new([41u8; 32]);
    let mut pins = PinnedReceiverCards::from_pairs(vec![(
        card.owner_handle,
        PinEntry {
            view_pk: card.view_pk,
            identity_pk: card.identity_pk,
            directory_id: None,
            first_seen: 0,
            trust_level: TrustLevel::Pinned,
        },
    )]);
    let output = build_card_output_serial_checked(
        card,
        BuildCheck {
            pins: &mut pins,
            chain_id: 0,
        },
        &mut sender_wallet,
        tx_seed,
        0,
        asset.amount,
        &asset.definition.id,
        asset.serial_id,
    )
    .map_err(|e| format!("build_card_output_serial_checked failed: {e}"))?;

    let commitment = z00z_crypto::Commitment::from_bytes(&output.c_amount)
        .map_err(|e| format!("invalid commitment bytes: {e}"))?;
    owned.commitment = commitment.as_commitment().clone();
    owned.owner_pub = None;
    owned.owner_signature = None;
    owned.r_pub = Some(output.r_pub);
    owned.owner_tag = Some(output.owner_tag);
    owned.enc_pack = Some(output.enc_pack);
    owned.tag16 = output.tag16;
    owned.leaf_ad_id = Some(asset.definition.id);

    let scanner = StealthOutputScanner::from_keys(keys);
    let ScanResult::Mine { wallet_output } = scanner.scan_leaf(&owned) else {
        return Err("generated stealth output is not mine for actor".to_string());
    };
    let blinding_bytes = wallet_output
        .blinding
        .as_ref()
        .copied()
        .ok_or_else(|| "missing blinding from wallet output".to_string())?;
    let blinding = Z00ZScalar::try_from_bytes(blinding_bytes)
        .map_err(|e| format!("invalid blinding scalar: {e}"))?;
    owned.range_proof = Some(
        create_range_proof(owned.amount, &blinding, 64, 0)
            .map_err(|e| format!("create_range_proof failed: {e}"))?,
    );

    let mut wire = AssetWire::from_asset(&owned);
    wire.secret = None;
    Ok(wire)
}

fn verify_pending(
    ctx: &SimContext,
    actor_idxs: &[usize],
    per_actor_assets: &[Vec<Asset>],
    state: &ClaimStateFile,
) -> Result<(), String> {
    for (slot, assets) in per_actor_assets.iter().enumerate() {
        let actor_idx = *actor_idxs
            .get(slot)
            .ok_or_else(|| format!("actor slot {slot} missing"))?;
        let actor = ctx
            .actors
            .get(actor_idx)
            .ok_or_else(|| format!("actor index {actor_idx} missing"))?;
        for asset in assets {
            let asset_id = asset.asset_id();
            if claim_row_exists(&state.claimed_rows, &actor.wallet_id, asset_id) {
                continue;
            }
            let claim_id = derive_output_nonce(&asset.definition.id, asset.serial_id);
            let wire = to_claim_wire(asset, &actor.keys, &actor.card, &claim_id)
                .map_err(|e| format!("resume verify wire build failed: {e}"))?;
            verify_resume_wire(&wire, &actor.keys)?;
        }
    }
    Ok(())
}

fn claim_rows_from_assets(assets: &[Asset]) -> Vec<ClaimRow> {
    let mut rows = Vec::with_capacity(assets.len());
    for asset in assets {
        rows.push(ClaimRow {
            asset_id: hex_str(&asset.asset_id()),
            symbol: asset.definition.symbol.clone(),
            class: asset.definition.class.to_string(),
            serial_id: asset.serial_id,
            amount: asset.amount,
        });
    }
    rows
}

#[derive(Debug, Deserialize)]
struct Stage1SnapRef {
    assets_count: usize,
}

#[derive(Debug, Serialize)]
pub(crate) struct WalletImportReport {
    claim_mode: String,
    compatibility_version: u32,
    stats: Vec<WalletImportStat>,
    persist_stats: Vec<ActorPersistStat>,
    wallet_ids: Vec<String>,
    accepted_assets: Vec<Vec<Asset>>,
    decisions: Vec<AuditLogRow>,
    persisted_assets: Vec<Vec<Asset>>,
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

pub fn run_claim_prepare(ctx: &mut SimContext, stage: &DesignStage) -> StageResult {
    match run_core(ctx, stage.stage) {
        Ok(()) => StageResult::Ok,
        Err(err) => StageResult::Fail(format!(
            "stage {} ({}) failed: {}",
            stage.stage, stage.name, err
        )),
    }
}

pub fn run_claim_genesis(ctx: &mut SimContext, stage: &DesignStage) -> StageResult {
    run_claim_prepare(ctx, stage)
}

pub fn run(ctx: &mut SimContext, stage: &DesignStage) -> StageResult {
    run_claim_prepare(ctx, stage)
}

/// Separates balance drift from commitment drift so resume diagnostics stay precise.
#[derive(Debug, Error, PartialEq, Eq)]
pub enum ConservationError {
    #[error("amount conservation mismatch: input={input}, imported={imported}")]
    AmountMismatch { input: u128, imported: u128 },
    #[error("commitment conservation mismatch")]
    CommitmentMismatch,
}

/// Compare amount and commitment totals before the claim bundle is finalized.
pub fn verify_claim_conservation(
    input: &[Asset],
    imported: &[Asset],
) -> Result<(), ConservationError> {
    let input_sum = sum_amount(input);
    let imported_sum = sum_amount(imported);
    if input_sum != imported_sum {
        return Err(ConservationError::AmountMismatch {
            input: input_sum,
            imported: imported_sum,
        });
    }

    let input_commit = sum_commit(input);
    let imported_commit = sum_commit(imported);
    if input_commit != imported_commit {
        return Err(ConservationError::CommitmentMismatch);
    }

    Ok(())
}

fn sum_amount(items: &[Asset]) -> u128 {
    items.iter().map(|item| u128::from(item.amount)).sum()
}

fn sum_commit(items: &[Asset]) -> Option<Z00ZCommitment> {
    let mut it = items.iter();
    let first = it.next()?.commitment.clone();
    Some(it.fold(first, |acc, item| &acc + &item.commitment))
}

fn run_core(ctx: &mut SimContext, stage_id: u32) -> Result<(), String> {
    runtime::run_core_impl(ctx, stage_id)
}

// ---------------------------------------------------------------------------
// Config / RNG helpers
// ---------------------------------------------------------------------------

/// Derive the asset-distribution RNG mode for stage 3.
///
/// Priority: `stage3_claim.rng_seed` > `simulation.mock_rng_seed` > System.
/// Claim-output scalar selection stays on the wallet-owned sender seam.
fn stage3_rng_mode(ctx: &SimContext) -> RngMode {
    let stage_seed = ctx.config.stage3_claim.as_ref().and_then(|c| c.rng_seed);
    let sim_seed = ctx.config.simulation.mock_rng_seed;
    RngMode::from_seed(stage_seed.or(sim_seed))
}

fn claim_cfg(ctx: &SimContext) -> Stage3ClaimCfg {
    ctx.config.stage3_claim.clone().unwrap_or(Stage3ClaimCfg {
        active: Some("uniform_all".to_string()),
        rng_seed: None,
        consume_bins: Some(true),
        snapshot_fault: None,
        resume_fault: None,
        paths: Stage3PathsCfg::default(),
    })
}

// ---------------------------------------------------------------------------
// Hex encoding helpers
// ---------------------------------------------------------------------------

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
