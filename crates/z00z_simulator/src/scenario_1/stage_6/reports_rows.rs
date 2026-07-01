use super::{
    core_build_confirm, core_build_pending, core_validate_confirm, AssetWire, Codec, ConfirmRow,
    CoreConfirm, CorePending, OutputBundle, PendingRow, TxOutputWire,
};
use z00z_core::Asset;
use z00z_wallets::tx::TxOutRole;

pub(crate) fn build_pending_rows(
    sender: &crate::SimActor,
    receiver: &crate::SimActor,
    fee_actor: &str,
    fee_wallet: &str,
    selected: &[AssetWire],
    outputs: &[OutputBundle],
    tx_outputs: &[TxOutputWire],
    tx_digest_hex: &str,
) -> Result<Vec<PendingRow>, String> {
    let rows = core_build_pending(
        (&sender.name, &sender.wallet_id),
        (&receiver.name, &receiver.wallet_id),
        (fee_actor, fee_wallet),
        selected,
        outputs,
        tx_outputs,
        tx_digest_hex,
    )?;
    Ok(rows.into_iter().map(pending_from_core).collect())
}

pub(crate) fn build_confirm_rows(pending_rows: &[PendingRow]) -> Vec<ConfirmRow> {
    let core_pending: Vec<CorePending> = pending_rows.iter().map(core_pending_from_row).collect();
    core_build_confirm(&core_pending)
        .into_iter()
        .map(confirm_from_core)
        .collect()
}

pub(crate) fn build_pending_rows_for_assets(
    actor: &str,
    wallet_id: &str,
    assets: &[Asset],
    tx_digest_hex: &str,
    life_status: &str,
    role: Option<TxOutRole>,
) -> Vec<PendingRow> {
    assets
        .iter()
        .map(|asset| PendingRow {
            actor: actor.to_string(),
            wallet_id: wallet_id.to_string(),
            asset_id_hex: hex::encode(asset.asset_id()),
            serial_id: asset.serial_id,
            class: format!("{:?}", asset.definition.class),
            amount: asset.amount,
            lifecycle_status: life_status.to_string(),
            output_role: role.map(role_name),
            tx_digest_hex: tx_digest_hex.to_string(),
        })
        .collect()
}

pub(crate) fn validate_confirm_rows(
    pending_rows: &[PendingRow],
    confirm_rows: &[ConfirmRow],
) -> Result<(), String> {
    let core_pending: Vec<CorePending> = pending_rows.iter().map(core_pending_from_row).collect();
    let core_confirm: Vec<CoreConfirm> = confirm_rows.iter().map(core_confirm_from_row).collect();
    core_validate_confirm(&core_pending, &core_confirm)
}

fn pending_from_core(row: CorePending) -> PendingRow {
    PendingRow {
        actor: row.actor,
        wallet_id: row.wallet_id,
        asset_id_hex: row.asset_id_hex,
        serial_id: row.serial_id,
        class: row.class,
        amount: row.amount,
        lifecycle_status: row.life_status,
        output_role: row.out_role,
        tx_digest_hex: row.tx_digest_hex,
    }
}

fn confirm_from_core(row: CoreConfirm) -> ConfirmRow {
    ConfirmRow {
        actor: row.actor,
        wallet_id: row.wallet_id,
        asset_id_hex: row.asset_id_hex,
        serial_id: row.serial_id,
        class: row.class,
        amount: row.amount,
        lifecycle_status: row.life_status,
        output_role: row.out_role,
        tx_digest_hex: row.tx_digest_hex,
    }
}

fn core_pending_from_row(row: &PendingRow) -> CorePending {
    CorePending {
        actor: row.actor.clone(),
        wallet_id: row.wallet_id.clone(),
        asset_id_hex: row.asset_id_hex.clone(),
        serial_id: row.serial_id,
        class: row.class.clone(),
        amount: row.amount,
        life_status: row.lifecycle_status.clone(),
        out_role: row.output_role.clone(),
        tx_digest_hex: row.tx_digest_hex.clone(),
    }
}

fn core_confirm_from_row(row: &ConfirmRow) -> CoreConfirm {
    CoreConfirm {
        actor: row.actor.clone(),
        wallet_id: row.wallet_id.clone(),
        asset_id_hex: row.asset_id_hex.clone(),
        serial_id: row.serial_id,
        class: row.class.clone(),
        amount: row.amount,
        life_status: row.lifecycle_status.clone(),
        out_role: row.output_role.clone(),
        tx_digest_hex: row.tx_digest_hex.clone(),
    }
}

fn role_name(role: TxOutRole) -> String {
    match role {
        TxOutRole::Recipient => "recipient",
        TxOutRole::Change => "change",
        TxOutRole::Fee => "fee",
    }
    .to_string()
}
