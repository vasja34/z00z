use super::{
    pick_output_serials, split_output_amounts, AssetClass, AssetWire, BTreeSet, BobOutCfg,
    OutputBundle, ReceiverCard, Stage4TxPrepareCfg, Z00ZScalar,
};

use super::output_construction_balance::{
    build_bob_outs, build_change_out, build_fee_out, card_for_role, mk_out_with_blind, role_rank,
    sum_input_blindings,
};

#[cfg(test)]
pub(crate) fn make_output_with_blind(
    party: String,
    role: z00z_wallets::tx::TxOutRole,
    class: AssetClass,
    card: &ReceiverCard,
    value: u64,
    serial_id: u32,
    split_seed: Option<u64>,
    idx: usize,
    blinding: Z00ZScalar,
) -> Result<OutputBundle, String> {
    mk_out_with_blind(
        party, role, class, card, value, serial_id, split_seed, idx, blinding,
    )
}

pub(crate) fn send_target_cfg(input_amount: u64, cfg: &Stage4TxPrepareCfg) -> Result<u64, String> {
    let send_value = match cfg.transaction.mode.as_str() {
        "fraction" => split_fraction(input_amount, cfg.transaction.fraction)?,
        "amount" => split_amount(input_amount, cfg.transaction.amount)?,
        other => {
            return Err(format!(
                "stage4: transaction.mode must be 'fraction' or 'amount', got '{other}'"
            ));
        }
    };
    Ok(send_value)
}

pub(crate) fn split_amount_cfg(
    input_amount: u64,
    cfg: &Stage4TxPrepareCfg,
) -> Result<(u64, u64), String> {
    let send_value = send_target_cfg(input_amount, cfg)?;
    let change_value = input_amount.saturating_sub(send_value);
    Ok((send_value, change_value))
}

pub(crate) fn has_change_hint(input_amount: u64, cfg: &Stage4TxPrepareCfg) -> Result<bool, String> {
    Ok(send_target_cfg(input_amount, cfg)? < input_amount)
}

fn split_fraction(input_amount: u64, fraction: Option<f64>) -> Result<u64, String> {
    let fraction = fraction
        .ok_or_else(|| "stage4: transaction.fraction must be set when mode=fraction".to_string())?;
    if !(0.0..=1.0).contains(&fraction) || fraction == 0.0 {
        return Err("stage4: transaction.fraction must be in range (0, 1]".to_string());
    }
    Ok(((input_amount as f64) * fraction).floor() as u64)
}

fn split_amount(input_amount: u64, amount: Option<u64>) -> Result<u64, String> {
    let amount = amount
        .ok_or_else(|| "stage4: transaction.amount must be set when mode=amount".to_string())?;
    if amount == 0 {
        return Err("stage4: transaction.amount must be > 0".to_string());
    }
    if amount > input_amount {
        return Err(format!(
            "stage4: transaction.amount={} exceeds input_amount={}",
            amount, input_amount
        ));
    }
    Ok(amount)
}

pub(crate) fn check_zero_send(
    send_value: u64,
    amount_key: &str,
    amount_value: u64,
    mode: &str,
) -> Result<(), String> {
    if send_value == 0 {
        return Err(format!(
            "stage4: transaction produced zero send amount: {amount_key}={amount_value} mode={mode}"
        ));
    }
    Ok(())
}

pub(crate) fn build_outputs_cfg(
    selected: &[AssetWire],
    recv_sec: [u8; 32],
    cfg: &Stage4TxPrepareCfg,
    asset_class: AssetClass,
    sender_name: &str,
    recipient_name: &str,
    fee_name: &str,
    alice_card: &ReceiverCard,
    bob_card: &ReceiverCard,
    fee_card: &ReceiverCard,
    send_value: u64,
    change_value: u64,
    fee_value: u64,
    split_seed: Option<u64>,
) -> Result<Vec<OutputBundle>, String> {
    let serials = selected_serials(selected)?;
    let bob_amounts = split_output_amounts(
        send_value,
        BobOutCfg {
            count: cfg.transaction.outputs.bob_outputs_count,
        },
        split_seed,
    )
    .map_err(|e| format!("stage4: bob output split failed: {e}"))?;

    let bob_serials = pick_output_serials(
        &serials,
        bob_amounts.len(),
        split_seed.map(|v| v ^ 0xB0B0_B0B0_B0B0_B0B0),
    )
    .map_err(|e| format!("stage4: bob serial assignment failed: {e}"))?;

    let mut planned = build_bob_outs(
        asset_class,
        recipient_name,
        bob_card,
        &bob_amounts,
        &bob_serials,
        split_seed,
    )?;
    if change_value > 0 {
        planned.push(build_change_out(
            asset_class,
            sender_name,
            alice_card,
            change_value,
            serials[0],
            split_seed,
            bob_amounts.len(),
        )?);
    }

    if fee_value > 0 {
        planned.push(build_fee_out(
            fee_name,
            fee_card,
            fee_value,
            serials[0],
            split_seed,
            planned.len(),
        )?);
    }

    planned.sort_by(|(left, _), (right, _)| {
        role_rank(left.role)
            .cmp(&role_rank(right.role))
            .then_with(|| left.leaf.serial_id.cmp(&right.leaf.serial_id))
            .then_with(|| left.leaf.asset_id.cmp(&right.leaf.asset_id))
    });

    if planned.is_empty() {
        return Ok(Vec::new());
    }

    let in_blind_sum = sum_input_blindings(recv_sec, selected)?;
    let out_blinds = planned
        .iter()
        .take(planned.len().saturating_sub(1))
        .map(|(out, _)| {
            z00z_wallets::tx::decode_output_pack(out).and_then(|pack| {
                Z00ZScalar::try_from_bytes(pack.blinding).map_err(|e| e.to_string())
            })
        })
        .collect::<Result<Vec<_>, _>>()?;
    let balance_blind = out_blinds
        .into_iter()
        .fold(in_blind_sum, |acc, item| &acc - &item);

    let last_idx = planned.len() - 1;
    let (last_out, orig_idx) = &planned[last_idx];
    let balance_card = card_for_role(last_out.role, alice_card, bob_card, fee_card);
    planned[last_idx].0 = mk_out_with_blind(
        last_out.receiver.clone(),
        last_out.role,
        last_out.class,
        balance_card,
        last_out.value,
        last_out.leaf.serial_id,
        split_seed,
        *orig_idx,
        balance_blind,
    )?;

    Ok(planned.into_iter().map(|(out, _)| out).collect())
}

fn selected_serials(selected: &[AssetWire]) -> Result<Vec<u32>, String> {
    let serials: Vec<u32> = selected
        .iter()
        .map(|row| row.serial_id)
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect();
    if serials.is_empty() {
        return Err("stage4: selected serial pool is empty".to_string());
    }
    Ok(serials)
}
