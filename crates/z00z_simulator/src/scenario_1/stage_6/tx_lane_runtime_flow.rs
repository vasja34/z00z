use super::{
    apply_root_tamper, apply_test_tamper, apply_wit_tamper, build_canon_snapshot,
    build_confirm_rows, build_outputs_cfg, build_pending_rows, build_prep_file,
    build_public_spend_contract, build_tx_package_digest, calc_fee, canonical_input_asset_id,
    check_zero_send, core_plain_balance, core_verify_self_decrypt, has_change_hint,
    load_claim_post_store, parse_asset_class, persist_sender_state, prep_membership_witnesses,
    prepare_spend_public_inputs, push_log, run_bob_checks, run_fee_checks, save_json,
    send_target_cfg, split_amount_cfg, to_hex, to_tx_output_wires, validate_confirm_rows,
    verify_commitment_balance_gate, verify_fee_matches_formula,
    verify_spend_witness_gate_membership, verify_tx_package, verify_tx_public_spend_contract,
    AssetWire, ClaimTxPackage, Codec, DesignStage, JsonCodec, OutputBundle, PrepFsStore,
    PrepSnapshotStore, ReceiverCard, ReceiverKeys, ReceiverSecret, RpcTransport, ScenarioCfg,
    Serialize, SimContext, Stage4TxPrepareCfg, TxInputWire, TxOutputWire, TxPackage, TxRecord,
    TxStatus, TxStorage, TxStorageImpl, TxWire,
};

use std::path::{Path, PathBuf};

use crate::scenario_1::stage_4::export_pre_tx_view;
use crate::scenario_1::stage_6::{ConfirmRow, PendingRow};
use z00z_utils::time::{SystemTimeProvider, TimeProvider};
use z00z_wallets::domains::hashing::compute_wallet_file_id;

use super::prep_ref::write_prep_ref;
use super::tx_lane_impl::{FeeParty, TxInputRef, PREP_FILE};

pub(super) struct PreparedTxArtifacts {
    pub(super) outputs: Vec<OutputBundle>,
    pub(super) tx_outputs: Vec<TxOutputWire>,
    pub(super) pkg: TxPackage,
}

pub(super) struct PendingRuntimeRows {
    pub(super) pending_rows: Vec<PendingRow>,
    pub(super) confirmed_rows: Vec<ConfirmRow>,
}

pub(super) struct TxPrepArgs<'a> {
    pub(super) cfg: &'a Stage4TxPrepareCfg,
    pub(super) sender: &'a crate::SimActor,
    pub(super) recipient: &'a crate::SimActor,
    pub(super) alice_card: &'a ReceiverCard,
    pub(super) bob_card: &'a ReceiverCard,
    pub(super) fee_card: &'a ReceiverCard,
    pub(super) selected: &'a mut [AssetWire],
    pub(super) sender_recv_sec: [u8; 32],
    pub(super) claim_pkgs: &'a [ClaimTxPackage],
    pub(super) out: &'a Path,
    pub(super) tx_dir: &'a Path,
    pub(super) tx_file: &'a Path,
    pub(super) prep_file: &'a Path,
    pub(super) lines: &'a mut Vec<String>,
}

pub(super) struct TxRuntimeArgs<'a> {
    pub(super) cfg: &'a Stage4TxPrepareCfg,
    pub(super) sender: &'a crate::SimActor,
    pub(super) recipient: &'a crate::SimActor,
    pub(super) fee_party: &'a FeeParty,
    pub(super) selected: &'a [AssetWire],
    pub(super) outputs: &'a [OutputBundle],
    pub(super) tx_outputs: &'a [TxOutputWire],
    pub(super) pkg: &'a TxPackage,
    pub(super) wallets_dir: &'a Path,
    pub(super) pending_file: &'a Path,
    pub(super) confirmed_file: &'a Path,
    pub(super) lines: &'a mut Vec<String>,
}

fn wallet_history_jsonl_path(wallets_dir: &Path, wallet_id: &str) -> PathBuf {
    let wallet_file_id = compute_wallet_file_id(wallet_id);
    let wallet_stem = hex::encode(&wallet_file_id[..8]);
    wallets_dir.join(format!("wallet_{wallet_stem}_tx_history.jsonl"))
}

fn append_tx_history_for_wallet(
    wallets_dir: &Path,
    wallet_id: &str,
    pkg: &TxPackage,
) -> Result<(), String> {
    let history_path = wallet_history_jsonl_path(wallets_dir, wallet_id);
    let tx_bytes = JsonCodec
        .serialize(pkg)
        .map_err(|e| format!("stage4: tx package serialization failed: {e}"))?;
    let tx_hash = pkg.tx_digest_hex.clone();
    let timestamp_ms = SystemTimeProvider.compat_unix_timestamp_millis();
    let record = TxRecord {
        tx_hash: tx_hash.clone(),
        tx_bytes,
        imported: false,
        status: TxStatus::Pending,
        timestamp_ms,
        block_height: None,
        confirmation_evidence: None,
    };

    let mut store = TxStorageImpl::new(history_path, SystemTimeProvider);
    store
        .put(record)
        .map_err(|e| format!("stage4: tx-history put failed for wallet {wallet_id}: {e}"))?;
    store
        .update_status(&tx_hash, TxStatus::Confirmed)
        .map_err(|e| format!("stage4: tx-history confirm failed for wallet {wallet_id}: {e}"))
}

fn resolve_stage4_chain_metadata(cfg: &ScenarioCfg) -> Result<(String, u32, String), String> {
    let chain_type = cfg
        .stage2_wallet_create
        .as_ref()
        .map(|cfg| cfg.wallet_chain.as_str())
        .unwrap_or(cfg.chain.as_str());
    let chain_id = match chain_type {
        "mainnet" => 1,
        "testnet" => 2,
        "devnet" => 3,
        other => return Err(format!("stage4: unsupported wallet chain type: {other}")),
    };
    let chain_name = format!("z00z-{chain_type}-1");
    Ok((chain_type.to_string(), chain_id, chain_name))
}

pub(super) fn prepare_tx_package_artifacts(
    ctx: &SimContext,
    stage: &DesignStage,
    args: TxPrepArgs<'_>,
) -> Result<PreparedTxArtifacts, String> {
    let TxPrepArgs {
        cfg,
        sender,
        recipient,
        alice_card,
        bob_card,
        fee_card,
        selected,
        sender_recv_sec,
        claim_pkgs,
        out,
        tx_dir,
        tx_file,
        prep_file,
        lines,
    } = args;

    let input_amount: u64 = selected.iter().map(|row| row.amount).sum();
    let asset_class = parse_asset_class(&cfg.transaction.class)?;

    let inputs: Vec<TxInputRef> = selected
        .iter()
        .map(|item| {
            let asset_id = canonical_input_asset_id(sender_recv_sec, item)?.ok_or_else(|| {
                "stage4: selected input is missing canonical terminal asset id".to_string()
            })?;
            Ok(TxInputRef {
                asset_id,
                serial_id: item.serial_id,
                amount: item.amount,
            })
        })
        .collect::<Result<Vec<_>, String>>()?;
    let tx_inputs: Vec<TxInputWire> = inputs
        .iter()
        .map(|input| {
            let asset_hex = to_hex(&input.asset_id);
            Ok(TxInputWire {
                asset_id_hex: asset_hex,
                serial_id: input.serial_id,
            })
        })
        .collect::<Result<Vec<_>, String>>()?;

    let split_seed = if ctx.config.simulation.use_mock_rng {
        ctx.config.simulation.mock_rng_seed
    } else {
        None
    };
    let send_target = send_target_cfg(input_amount, cfg)?;
    check_zero_send(
        send_target,
        "input_amount",
        input_amount,
        &cfg.transaction.mode,
    )?;
    let shape_outputs = build_outputs_cfg(
        selected,
        sender_recv_sec,
        cfg,
        asset_class,
        &sender.name,
        &recipient.name,
        &cfg.transaction.fee_sink.wallet_id,
        alice_card,
        bob_card,
        fee_card,
        send_target,
        if has_change_hint(input_amount, cfg)? {
            input_amount.saturating_sub(send_target)
        } else {
            0
        },
        1,
        split_seed,
    )?;

    let fee = calc_fee(&tx_inputs, &to_tx_output_wires(&shape_outputs)?)?;
    let spendable_after_fee = input_amount.checked_sub(fee).ok_or_else(|| {
        format!(
            "stage4: insufficient input for fee: input_sum={} fee={}",
            input_amount, fee
        )
    })?;

    let (send_value, change_value) = split_amount_cfg(spendable_after_fee, cfg)?;
    check_zero_send(
        send_value,
        "spendable_after_fee",
        spendable_after_fee,
        &cfg.transaction.mode,
    )?;

    let mut outputs = build_outputs_cfg(
        selected,
        sender_recv_sec,
        cfg,
        asset_class,
        &sender.name,
        &recipient.name,
        &cfg.transaction.fee_sink.wallet_id,
        alice_card,
        bob_card,
        fee_card,
        send_value,
        change_value,
        fee,
        split_seed,
    )?;

    push_log(
        lines,
        stage.stage,
        "S4-4",
        "build_outputs",
        "ok",
        &format!(
            "outputs={} bob_outputs={} fee={} spendable_after_fee={}",
            outputs.len(),
            outputs
                .iter()
                .filter(|o| o.receiver.eq_ignore_ascii_case("bob"))
                .count(),
            fee,
            spendable_after_fee,
        ),
    )?;

    apply_test_tamper(&mut outputs, out)?;

    for out_item in &outputs {
        core_verify_self_decrypt(out_item)?;
    }
    push_log(
        lines,
        stage.stage,
        "S4-5",
        "self_decrypt",
        "ok",
        &format!("verified_outputs={}", outputs.len()),
    )?;

    let tx_outputs = to_tx_output_wires(&outputs)?;
    verify_fee_matches_formula(&tx_inputs, &tx_outputs, fee)?;
    core_plain_balance(selected, &outputs, fee)?;

    let claim_store = load_claim_post_store(out, claim_pkgs)?;
    let prep = build_prep_file(&claim_store, selected, &tx_inputs)?;
    let membership = prep_membership_witnesses(&prep)?;
    let prev_root = claim_store
        .settlement_root()
        .map(z00z_storage::settlement::CheckRoot::from)
        .map_err(|e| format!("stage4: claim store root load failed: {e}"))?;
    let (chain_type, chain_id, chain_name) = resolve_stage4_chain_metadata(&ctx.config)?;
    let receiver_secret = ReceiverSecret::from_bytes(sender_recv_sec)
        .map_err(|e| format!("stage4: sender receiver secret decode failed: {e}"))?;
    let receiver_keys = ReceiverKeys::from_receiver_secret(
        ReceiverSecret::from_bytes(sender_recv_sec)
            .map_err(|e| format!("stage4: sender receiver secret decode failed: {e}"))?,
    )
    .map_err(|e| format!("stage4: sender receiver keys derive failed: {e}"))?;
    let proof_inputs =
        prepare_spend_public_inputs(chain_id, sender_recv_sec, selected, &tx_inputs)?;
    let input_s_in = selected
        .iter()
        .map(|item| {
            z00z_wallets::tx::resolve_input_pack(sender_recv_sec, item).map(|pack| pack.s_out)
        })
        .collect::<Result<Vec<_>, String>>()?;

    let mut tx_wire = TxWire {
        tx_type: "regular_tx".to_string(),
        inputs: tx_inputs.clone(),
        outputs: tx_outputs.clone(),
        fee,
        nonce: 0,
        context: Default::default(),
        proof: Default::default(),
        auth: Default::default(),
    };
    let (spend_proof, spend_auth) = build_public_spend_contract(
        &receiver_keys,
        chain_id,
        1,
        &chain_type,
        &chain_name,
        &tx_wire,
        prev_root,
        proof_inputs,
        z00z_wallets::tx::SpendProofWitness {
            receiver_secret,
            input_s_in,
            membership: membership.clone(),
        },
    )
    .map_err(|e| format!("stage4: current-stack tx public spend proof build failed: {e}"))?;
    tx_wire.proof = spend_proof;
    tx_wire.auth = spend_auth;
    verify_tx_public_spend_contract(chain_id, 1, &chain_type, &chain_name, &tx_wire)
        .map_err(|e| format!("stage4: current-stack tx public spend verifier failed: {e}"))?;
    let tx_digest_hex = build_tx_package_digest(
        "TxPackage",
        "regular_tx",
        1,
        chain_id,
        &chain_type,
        &chain_name,
        &tx_wire,
    )?;

    verify_commitment_balance_gate(sender_recv_sec, selected, &outputs, fee)?;
    push_log(
        lines,
        stage.stage,
        "S4-9",
        "balance_gate",
        "ok",
        "plaintext + commitment fee-inclusive balance verified",
    )?;

    let mut prep = prep;
    apply_root_tamper(&mut prep, out)?;

    let (prep_snapshot, prep_snapshot_id) = build_canon_snapshot(&prep, &claim_store)?;
    let mut snap_store = PrepFsStore::new(tx_dir);
    let saved_snapshot_id = snap_store
        .save_snapshot(&prep_snapshot)
        .map_err(|e| format!("stage4: canonical snapshot save failed: {e}"))?;
    if saved_snapshot_id != prep_snapshot_id {
        return Err("stage4: canonical snapshot id drift".to_string());
    }
    export_pre_tx_view(out, saved_snapshot_id, &prep_snapshot)?;

    apply_wit_tamper(selected, out)?;
    verify_spend_witness_gate_membership(
        chain_id,
        sender_recv_sec,
        selected,
        &outputs,
        prev_root,
        membership,
    )?;
    push_log(
        lines,
        stage.stage,
        "S4-10",
        "spend_witness_gate",
        "ok",
        &format!(
            "spend witness gate passed canonical_snapshot_id={}",
            to_hex(prep_snapshot_id.as_bytes())
        ),
    )?;

    let pkg = TxPackage {
        kind: "TxPackage".to_string(),
        package_type: "regular_tx".to_string(),
        version: 1,
        chain_id,
        chain_type,
        chain_name,
        tx: tx_wire,
        tx_digest_hex,
        status: "prepared".to_string(),
    };

    let tx_bytes = JsonCodec.serialize(&pkg).map_err(|e| e.to_string())?;
    verify_tx_package(&tx_bytes)?;
    write_prep_ref(prep_file, saved_snapshot_id)?;
    save_json(tx_file, &pkg).map_err(|e| e.to_string())?;
    push_log(
        lines,
        stage.stage,
        "S4-6",
        "write_tx_pkg",
        "ok",
        &format!(
            "{} prep={} fee={}",
            tx_file.to_string_lossy(),
            prep_file.to_string_lossy(),
            fee
        ),
    )?;

    Ok(PreparedTxArtifacts {
        outputs,
        tx_outputs,
        pkg,
    })
}

pub(super) async fn persist_and_confirm_runtime(
    stage: &DesignStage,
    transport: &impl RpcTransport,
    args: TxRuntimeArgs<'_>,
) -> Result<PendingRuntimeRows, String> {
    let TxRuntimeArgs {
        cfg,
        sender,
        recipient,
        fee_party,
        selected,
        outputs,
        tx_outputs,
        pkg,
        wallets_dir,
        pending_file,
        confirmed_file,
        lines,
    } = args;

    let persist = persist_sender_state(
        transport, cfg, sender, recipient, selected, outputs, tx_outputs,
    )
    .await?;

    push_log(
        lines,
        stage.stage,
        "S4-7",
        "persist_sender_state",
        "ok",
        &format!(
            "spent_marked={} amount_changed={} change_imported={}",
            persist.spent_marked, persist.tracked_amount_changed, persist.change_imported
        ),
    )?;

    run_bob_checks(transport, cfg, recipient, outputs, tx_outputs).await?;
    if let Some(fee_pass) = fee_party.password.as_deref() {
        run_fee_checks(
            transport,
            cfg,
            &fee_party.actor,
            &fee_party.wallet_id,
            fee_pass,
            outputs,
            tx_outputs,
        )
        .await?;
    }

    let pending_rows = build_pending_rows(
        sender,
        recipient,
        &fee_party.actor,
        &fee_party.wallet_id,
        selected,
        outputs,
        tx_outputs,
        &pkg.tx_digest_hex,
    )?;
    save_json(pending_file, &pending_rows).map_err(|e| e.to_string())?;

    let confirmed_rows = build_confirm_rows(&pending_rows);
    validate_confirm_rows(&pending_rows, &confirmed_rows)?;
    save_json(confirmed_file, &confirmed_rows).map_err(|e| e.to_string())?;

    append_tx_history_for_wallet(wallets_dir, &sender.wallet_id, pkg)?;
    append_tx_history_for_wallet(wallets_dir, &recipient.wallet_id, pkg)?;

    push_log(
        lines,
        stage.stage,
        "S4-8",
        "bob_upload_decrypt_check",
        "ok",
        "bob import/upload and decrypt checks passed",
    )?;

    push_log(
        lines,
        stage.stage,
        "S4-C1",
        "confirm_pending",
        "ok",
        &format!(
            "pending={} confirmed={} artifacts=[{},{}]",
            pending_rows.len(),
            confirmed_rows.len(),
            pending_file.to_string_lossy(),
            confirmed_file.to_string_lossy()
        ),
    )?;

    push_log(
        lines,
        stage.stage,
        "S4-14",
        "persist_live_tx_history",
        "ok",
        "sender+recipient tx_history.jsonl updated",
    )?;

    Ok(PendingRuntimeRows {
        pending_rows,
        confirmed_rows,
    })
}

#[cfg(test)]
mod tests {
    use super::resolve_stage4_chain_metadata;
    use crate::ScenarioCfg;

    fn load_cfg() -> ScenarioCfg {
        ScenarioCfg::from_file("src/scenario_1/scenario_config.yaml").expect("load scenario cfg")
    }

    #[test]
    fn test_stage4_metadata_scenario_chain() {
        let cfg = load_cfg();

        let (chain_type, chain_id, chain_name) =
            resolve_stage4_chain_metadata(&cfg).expect("resolve stage4 chain metadata");

        assert_eq!(chain_type, "devnet");
        assert_eq!(chain_id, 3);
        assert_eq!(chain_name, "z00z-devnet-1");
    }

    #[test]
    fn test_stage4_metadata_wallet_chain() {
        let mut cfg = load_cfg();
        let stage2 = cfg
            .stage2_wallet_create
            .as_mut()
            .expect("stage2 wallet cfg");
        stage2.wallet_chain = "testnet".to_string();

        let (chain_type, chain_id, chain_name) =
            resolve_stage4_chain_metadata(&cfg).expect("resolve stage4 chain metadata");

        assert_eq!(chain_type, "testnet");
        assert_eq!(chain_id, 2);
        assert_eq!(chain_name, "z00z-testnet-1");
    }
}
