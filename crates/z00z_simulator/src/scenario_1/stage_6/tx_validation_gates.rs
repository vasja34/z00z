use std::path::Path;

use serde::Deserialize;
use z00z_core::{
    assets::{AssetClass, AssetPackPlain, AssetPkgWire},
    AssetWire,
};
use z00z_storage::settlement::CheckRoot;
use z00z_utils::{
    codec::{Codec, JsonCodec},
    io::read_to_string,
};
use z00z_wallets::{
    receiver::{decode_card_compact, ReceiverCard},
    stealth::bind_stealth_output_wire,
    stealth::{
        kdf::{compute_leaf_ad, compute_tag16},
        zkpack::ZkPack,
    },
    tx::{OutputBundle, TxAssemblerImpl, TxInputWire, TxOutputWire},
};

use crate::{
    config::Stage4TxPrepareCfg,
    scenario_1::stage_6::tx_lane_impl::{validate_fee_sink, validate_tx_mode, KeysFile},
    SimContext,
};

use super::{distinct_serial_target, find_actor};

fn load_json<T: for<'de> Deserialize<'de>>(path: &Path) -> Result<T, String> {
    let body = read_to_string(path).map_err(|e| e.to_string())?;
    JsonCodec
        .deserialize(body.as_bytes())
        .map_err(|e| format!("json parse {}: {e}", path.display()))
}

pub(crate) fn load_stage4_verified_card(path: &Path) -> Result<ReceiverCard, String> {
    let keys: KeysFile = load_json(path)?;
    let compact = keys
        .card_compact
        .as_deref()
        .ok_or_else(|| format!("stage4: missing card_compact in {}", path.display()))?;
    let card = decode_card_compact(compact)
        .map_err(|e| format!("stage4: invalid receiver card compact: {e}"))?;
    card.verify()
        .map_err(|e| format!("stage4: receiver card verify failed: {e}"))?;
    Ok(card)
}

pub(crate) fn parse_asset_class(class_name: &str) -> Result<AssetClass, String> {
    match class_name.to_ascii_lowercase().as_str() {
        "coin" => Ok(AssetClass::Coin),
        "token" => Ok(AssetClass::Token),
        "nft" => Ok(AssetClass::Nft),
        "void" => Ok(AssetClass::Void),
        _ => Err(format!("stage4: unsupported asset class: {class_name}")),
    }
}

pub(crate) fn decode_output_pack(out: &OutputBundle) -> Result<AssetPackPlain, String> {
    let leaf = &out.leaf;
    let leaf_ad = compute_leaf_ad(
        &leaf.asset_id,
        leaf.serial_id,
        &leaf.r_pub,
        &leaf.owner_tag,
        &leaf.c_amount,
    );

    let expected_tag16 = compute_tag16(&out.k_dh, &leaf_ad);
    if expected_tag16 != leaf.tag16 {
        return Err(format!(
            "stage4: tag16 mismatch party={} expected={} got={} serial_id= {}",
            out.receiver, expected_tag16, leaf.tag16, leaf.serial_id
        ));
    }

    let plaintext = ZkPack::decrypt(
        &out.k_dh,
        &leaf_ad,
        &leaf.r_pub,
        &leaf.asset_id,
        leaf.serial_id,
        &leaf.enc_pack,
    )
    .ok_or_else(|| {
        format!(
            "stage4: self-decrypt failed party={} serial_id={}",
            out.receiver, leaf.serial_id
        )
    })?;

    let pack = AssetPackPlain::from_bytes(&plaintext).ok_or_else(|| {
        format!(
            "stage4: plaintext decode failed party={} len={}",
            out.receiver,
            plaintext.len()
        )
    })?;

    if pack.value != out.value {
        return Err(format!(
            "stage4: value mismatch party={} expected={} got={}",
            out.receiver, out.value, pack.value
        ));
    }

    if pack.s_out != out.s_out {
        return Err(format!(
            "stage4: s_out mismatch party={} serial_id={}",
            out.receiver, leaf.serial_id
        ));
    }

    Ok(pack)
}

pub(crate) fn to_tx_output_wires(outputs: &[OutputBundle]) -> Result<Vec<TxOutputWire>, String> {
    outputs
        .iter()
        .enumerate()
        .map(|(idx, out)| {
            let wire_serial = out.leaf.serial_id;
            let mut asset = z00z_core::genesis::asset_std::asset_from_dev_class(
                out.class,
                wire_serial,
                out.value,
            )
            .map_err(|e| format!("stage4: failed to build output asset: {e}"))?;
            asset.nonce = z00z_wallets::tx::derive_tx_output_nonce(&out.leaf, idx);
            asset.r_pub = Some(out.leaf.r_pub);
            asset.owner_tag = Some(out.leaf.owner_tag);
            asset.enc_pack = Some(out.leaf.enc_pack.clone());
            asset.tag16 = Some(out.leaf.tag16);
            let commitment = z00z_crypto::Commitment::from_bytes(&out.leaf.c_amount)
                .map_err(|e| format!("stage4: leaf c_amount parse failed: {e}"))?;
            asset.commitment = commitment.as_commitment().clone();
            asset.range_proof = Some(out.leaf.range_proof.clone());
            asset.owner_signature = None;

            let wire = bind_stealth_output_wire(AssetWire::from_asset(&asset), &out.leaf)?;
            Ok(TxOutputWire {
                role: out.role,
                asset_wire: AssetPkgWire::from_wire(&wire),
            })
        })
        .collect()
}

pub(crate) fn calc_fee(inputs: &[TxInputWire], outputs: &[TxOutputWire]) -> Result<u64, String> {
    let out_wires: Vec<AssetWire> = outputs
        .iter()
        .map(|out| out.asset_wire.clone().to_wire())
        .collect::<Result<_, _>>()
        .map_err(|e| format!("stage4: output asset_wire conversion failed: {e}"))?;
    TxAssemblerImpl::new()
        .calculate_fee_for_wires(inputs.len(), &out_wires)
        .map_err(|e| format!("stage4: fee calc failed: {e}"))
}

pub(crate) fn verify_fee_matches_formula(
    inputs: &[TxInputWire],
    outputs: &[TxOutputWire],
    declared_fee: u64,
) -> Result<(), String> {
    let expected = calc_fee(inputs, outputs)?;
    if expected != declared_fee {
        return Err(format!(
            "stage4: fee mismatch: declared={} expected={}",
            declared_fee, expected
        ));
    }
    Ok(())
}

pub(crate) fn verify_commitment_balance_gate(
    _recv_sec: [u8; 32],
    selected_inputs: &[AssetWire],
    outputs: &[OutputBundle],
    fee: u64,
) -> Result<(), String> {
    if selected_inputs.is_empty() || outputs.is_empty() {
        return Err("stage4: balance gate requires non-empty inputs and outputs".to_string());
    }
    let in_commits: Vec<z00z_crypto::Z00ZCommitment> = selected_inputs
        .iter()
        .map(|item| item.commitment.clone())
        .collect();
    let out_commits: Vec<z00z_crypto::Z00ZCommitment> = outputs
        .iter()
        .map(|out| {
            z00z_crypto::Commitment::from_bytes(&out.leaf.c_amount)
                .map(|commitment| commitment.as_commitment().clone())
                .map_err(|e| format!("stage4: output commitment parse failed: {e}"))
        })
        .collect::<Result<Vec<_>, _>>()?;

    let _ = fee;
    z00z_wallets::tx::verify_commitment_balance_gate(&in_commits, &out_commits, 0)?;
    Ok(())
}

#[cfg(test)]
pub(crate) fn verify_spend_witness_gate(
    chain_id: u32,
    recv_sec: [u8; 32],
    selected_inputs: &[AssetWire],
    outputs: &[OutputBundle],
    prev_root: CheckRoot,
) -> Result<(), String> {
    z00z_wallets::tx::verify_spend_witness_gate(
        chain_id,
        recv_sec,
        selected_inputs,
        outputs,
        prev_root,
    )
}

pub(crate) fn verify_spend_witness_gate_membership(
    chain_id: u32,
    recv_sec: [u8; 32],
    selected_inputs: &[AssetWire],
    outputs: &[OutputBundle],
    prev_root: CheckRoot,
    membership: Vec<z00z_wallets::tx::SpendMembershipWitness>,
) -> Result<(), String> {
    z00z_wallets::tx::verify_spend_witness_gate_membership(
        chain_id,
        recv_sec,
        selected_inputs,
        outputs,
        prev_root,
        membership,
    )
}

pub(crate) fn verify_tx_package(tx_bytes: &[u8]) -> Result<(), String> {
    let result = z00z_wallets::tx::verify_full_tx_package(tx_bytes)
        .map_err(|e| format!("stage4: tx verifier failed: {e}"))?;
    if !result.valid {
        return Err(format!(
            "stage4: tx package invalid: {}",
            result.errors.join("; ")
        ));
    }

    Ok(())
}

pub(crate) fn stage4_cfg(ctx: &SimContext) -> Result<&Stage4TxPrepareCfg, String> {
    ctx.config
        .stage4_tx_prepare
        .as_ref()
        .ok_or_else(|| "stage4: missing stage4_tx_prepare config".to_string())
}

pub(crate) fn validate_stage4_cfg(
    ctx: &SimContext,
    cfg: &Stage4TxPrepareCfg,
) -> Result<(), String> {
    if !cfg.enabled {
        return Err("stage4: stage4_tx_prepare.enabled=false".to_string());
    }
    if cfg.rpc.transport != "logged_local" {
        return Err("stage4: only rpc.transport=logged_local is supported".to_string());
    }
    if cfg.transaction.symbol.trim().is_empty() {
        return Err("stage4: transaction.symbol must be set".to_string());
    }
    if parse_asset_class(&cfg.transaction.class).is_err() {
        return Err("stage4: transaction.class must be one of Coin|Token|Nft|Void".to_string());
    }
    validate_tx_mode(cfg)?;
    if cfg.transaction.outputs.bob_outputs_count == 0 {
        return Err("stage4: transaction.outputs.bob_outputs_count must be > 0".to_string());
    }
    validate_bob_count(cfg)?;
    if cfg.rpc.import_asset_method.is_empty() {
        return Err("stage4: rpc.import_asset_method must be set".to_string());
    }
    validate_fee_sink(ctx, &cfg.transaction.fee_sink)?;
    if cfg.rpc.build_transaction_method.is_empty() {
        return Err("stage4: rpc.build_transaction_method must be set".to_string());
    }
    let _ = find_actor(ctx, &cfg.receiver_actor)?;
    let _ = find_actor(ctx, &cfg.sender_actor)?;
    Ok(())
}

fn validate_bob_count(cfg: &Stage4TxPrepareCfg) -> Result<(), String> {
    let distinct_target = distinct_serial_target(&cfg.transaction.input_assets_selection)?;
    let bob_count = cfg.transaction.outputs.bob_outputs_count as usize;
    if bob_count < distinct_target {
        return Err(format!(
            "stage4: transaction.outputs.bob_outputs_count={} must be >= distinct serial target {}",
            bob_count, distinct_target
        ));
    }
    Ok(())
}

pub(crate) fn stage4_flags_summary(cfg: &Stage4TxPrepareCfg) -> String {
    format!(
        "tx(class={},symbol={},mode={},bob_out={},fee_sink={}) distinct(min={},target={},max={})",
        cfg.transaction.class,
        cfg.transaction.symbol,
        cfg.transaction.mode,
        cfg.transaction.outputs.bob_outputs_count,
        cfg.transaction.fee_sink.wallet_id,
        cfg.transaction
            .input_assets_selection
            .distinct_serial_ids_min,
        cfg.transaction
            .input_assets_selection
            .distinct_serial_ids_target,
        cfg.transaction
            .input_assets_selection
            .distinct_serial_ids_max,
    )
}
