//! Core tx output flow helpers extracted from simulator Stage-4.
//!
//! These helpers stay internal to the tx facade. Use `crate::stealth`
//! for the public sender-output construction surface.

use z00z_core::{
    assets::{AssetClass, AssetPackPlain},
    AssetWire,
};
use z00z_crypto::{
    compute_leaf_ad, compute_tag16, create_commitment, domains::TxOutputNonceDomain,
    hash_zk::hash_zk, range_ctx_hash, verify_range_proof, Z00ZCommitment, Z00ZScalar,
    AGGREGATION_FACTOR, MIN_VALUE_PROMISE, RANGE_PROOF_BITS,
};
#[cfg(test)]
use z00z_crypto::{domains::TxDigestDomain, frame_bytes};
use z00z_storage::settlement::TerminalLeaf;
#[cfg(test)]
use z00z_utils::codec::Codec;

use crate::stealth::zkpack::ZkPack;

#[cfg(test)]
use super::TxWire;
use super::{verify_tx_balance, TxOutRole};

/// Output bundle used by tx preparation flow.
#[derive(Debug, Clone)]
pub struct OutputBundle {
    /// Receiver label.
    pub receiver: String,
    /// Semantic transaction role.
    pub role: TxOutRole,
    /// Asset class.
    pub class: AssetClass,
    /// Plain value.
    pub value: u64,
    /// Built confidential settlement leaf.
    pub leaf: TerminalLeaf,
    /// Diffie-Hellman shared key.
    pub k_dh: [u8; 32],
    /// Output secret.
    pub s_out: [u8; 32],
}

fn out_leaf_ad(leaf: &TerminalLeaf) -> [u8; 32] {
    compute_leaf_ad(
        &leaf.asset_id,
        leaf.serial_id,
        &leaf.r_pub,
        &leaf.owner_tag,
        &leaf.c_amount,
    )
}

/// Derive the frozen range-proof context digest for one confidential output leaf.
pub fn output_range_ctx_hash(
    leaf: &TerminalLeaf,
    chain_id: u32,
    root_ver: u8,
    proof_ver: u8,
    policy_ver: u32,
) -> [u8; 32] {
    range_ctx_hash(
        &leaf.asset_id,
        chain_id,
        root_ver,
        proof_ver,
        policy_ver,
        &leaf.c_amount,
        &leaf.range_proof,
    )
}

/// Bind a confidential output leaf into the tx-output wire shape.
pub fn bind_output_wire(mut wire: AssetWire, leaf: &TerminalLeaf) -> Result<AssetWire, String> {
    wire.serial_id = leaf.serial_id;
    let commitment = z00z_crypto::Commitment::from_bytes(&leaf.c_amount)
        .map_err(|e| format!("tx output bridge: commitment parse failed: {e}"))?;
    wire.commitment = commitment.as_commitment().clone();
    wire.range_proof = Some(leaf.range_proof.clone());
    wire.owner_pub = None;
    wire.owner_signature = None;
    wire.r_pub = Some(leaf.r_pub);
    wire.owner_tag = Some(leaf.owner_tag);
    wire.enc_pack = Some(leaf.enc_pack.clone());
    wire.tag16 = Some(leaf.tag16);
    wire.leaf_ad_id = Some(leaf.asset_id);
    wire.secret = None;
    Ok(wire)
}

fn verify_out_tag(out: &OutputBundle, leaf_ad: &[u8; 32]) -> Result<(), String> {
    let leaf = &out.leaf;
    let expected_tag16 = compute_tag16(&out.k_dh, leaf_ad);
    if expected_tag16 != leaf.tag16 {
        return Err(format!(
            "stage4: tag16 mismatch receiver={} expected={} got={} serial_id={}",
            out.receiver, expected_tag16, leaf.tag16, leaf.serial_id
        ));
    }
    Ok(())
}

/// Decode output encrypted pack and verify local consistency.
pub fn decode_output_pack(out: &OutputBundle) -> Result<AssetPackPlain, String> {
    let leaf = &out.leaf;
    let leaf_ad = out_leaf_ad(leaf);
    verify_out_tag(out, &leaf_ad)?;

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
            "stage4: self-decrypt failed receiver={} serial_id={}",
            out.receiver, leaf.serial_id
        )
    })?;

    let pack = AssetPackPlain::from_bytes(&plaintext).ok_or_else(|| {
        format!(
            "stage4: plaintext decode failed receiver={} len={}",
            out.receiver,
            plaintext.len()
        )
    })?;

    if pack.value != out.value {
        return Err(format!(
            "stage4: value mismatch receiver={} expected={} got={}",
            out.receiver, out.value, pack.value
        ));
    }

    if pack.s_out != out.s_out {
        return Err(format!(
            "stage4: s_out mismatch receiver={} serial_id={}",
            out.receiver, leaf.serial_id
        ));
    }

    Ok(pack)
}

/// Verify self decrypt, commitment opening, and range proof.
pub fn verify_self_decrypt(out: &OutputBundle) -> Result<(), String> {
    let leaf = &out.leaf;
    let pack = decode_output_pack(out)?;

    let blinding = Z00ZScalar::try_from_bytes(pack.blinding).map_err(|e| e.to_string())?;
    let commitment = create_commitment(pack.value, &blinding).map_err(|e| e.to_string())?;
    let commitment_bytes: [u8; 32] = commitment
        .as_bytes()
        .try_into()
        .map_err(|_| "stage4: commitment bytes size mismatch".to_string())?;
    if commitment_bytes != leaf.c_amount {
        return Err(format!(
            "stage4: commitment mismatch receiver={} serial_id={}",
            out.receiver, leaf.serial_id
        ));
    }

    let parsed = z00z_crypto::Commitment::from_bytes(&leaf.c_amount).map_err(|e| e.to_string())?;
    verify_range_proof(
        &leaf.range_proof,
        parsed.as_commitment(),
        RANGE_PROOF_BITS,
        AGGREGATION_FACTOR,
        MIN_VALUE_PROMISE,
    )
    .map_err(|e| {
        format!(
            "stage4: range proof verify failed receiver={} err={e}",
            out.receiver
        )
    })?;

    Ok(())
}

/// Compute tx digest from canonical tx wire payload.
#[cfg(test)]
pub(crate) fn compute_tx_digest_from_wire(tx_wire: &TxWire) -> Result<[u8; 32], String> {
    let wire_bytes = z00z_utils::codec::JsonCodec
        .serialize(tx_wire)
        .map_err(|e| format!("stage4: tx wire serialize for digest failed: {e}"))?;
    let framed = frame_bytes(&wire_bytes);
    Ok(hash_zk::<TxDigestDomain>(
        "Z00Z/TXPKG_WIRE",
        &[framed.as_slice()],
    ))
}

/// Derive deterministic tx output nonce from leaf and output index.
pub fn derive_tx_output_nonce(leaf: &TerminalLeaf, idx: usize) -> [u8; 32] {
    let idx_bytes = (idx as u64).to_le_bytes();
    hash_zk::<TxOutputNonceDomain>(
        "",
        &[&leaf.r_pub, &leaf.asset_id, &leaf.c_amount, &idx_bytes],
    )
}

/// Verify plaintext amount balance with fee.
pub fn verify_plaintext_balance_with_fee(
    selected_inputs: &[AssetWire],
    outputs: &[OutputBundle],
    _fee: u64,
) -> Result<(), String> {
    let in_sum: u128 = selected_inputs.iter().map(|item| item.amount as u128).sum();
    let out_sum: u128 = outputs.iter().map(|out| out.value as u128).sum();
    if in_sum != out_sum {
        return Err(format!(
            "stage4: plaintext balance mismatch: in_sum={} out_sum={}",
            in_sum, out_sum
        ));
    }
    Ok(())
}

/// Derive the commitment delta between input and output sums.
///
/// With fee-as-output semantics, a valid transaction balances to the zero
/// commitment because the fee leaf is already part of `outputs`.
pub fn derive_balance_commitment(
    inputs: &[Z00ZCommitment],
    outputs: &[Z00ZCommitment],
) -> Result<Z00ZCommitment, String> {
    if inputs.is_empty() || outputs.is_empty() {
        return Err(
            "stage4: fee commitment derivation requires non-empty inputs and outputs".to_string(),
        );
    }

    let in_sum = inputs
        .iter()
        .skip(1)
        .fold(inputs[0].clone(), |acc, item| &acc + item);
    let out_sum = outputs
        .iter()
        .skip(1)
        .fold(outputs[0].clone(), |acc, item| &acc + item);
    Ok(&in_sum - &out_sum)
}

/// Verify commitment balance gate and return the resulting balance delta.
///
/// Under fee-as-output, the returned commitment is zero for a valid tx.
pub fn verify_commitment_balance_gate(
    in_commits: &[Z00ZCommitment],
    out_commits: &[Z00ZCommitment],
    _fee: u64,
) -> Result<Z00ZCommitment, String> {
    if in_commits.is_empty() || out_commits.is_empty() {
        return Err("stage4: balance gate requires non-empty inputs and outputs".to_string());
    }

    if !verify_tx_balance(in_commits, out_commits) {
        return Err("stage4: commitment balance proof failed: inputs != outputs".to_string());
    }

    derive_balance_commitment(in_commits, out_commits)
}

/// Verify fee commitment opening from input/output blinding sums.
pub fn verify_fee_commitment_opening(
    fee_commit: &Z00ZCommitment,
    fee: u64,
    in_blind_sum: &Z00ZScalar,
    out_blind_sum: &Z00ZScalar,
) -> Result<(), String> {
    let fee_blind = in_blind_sum - out_blind_sum;
    verify_fee_opening_eq(fee_commit, fee, &fee_blind)
}

/// Verify fee commitment equals declared opening and reconstructed commitment.
pub fn verify_fee_opening_eq(
    fee_commit: &Z00ZCommitment,
    fee: u64,
    fee_blind: &Z00ZScalar,
) -> Result<(), String> {
    let opening_ok = z00z_core::assets::verify_commitment_opening(fee_commit, fee, fee_blind)
        .map_err(|e| format!("stage4: fee opening verify error: {e}"))?;
    if !opening_ok {
        return Err(format!(
            "stage4: fee opening mismatch for declared fee={fee}"
        ));
    }

    let rebuilt = create_commitment(fee, fee_blind)
        .map_err(|e| format!("stage4: fee commitment rebuild failed: {e}"))?;
    if rebuilt.as_bytes() != fee_commit.as_bytes() {
        return Err("stage4: fee commitment mismatch against reconstructed opening".to_string());
    }
    Ok(())
}
