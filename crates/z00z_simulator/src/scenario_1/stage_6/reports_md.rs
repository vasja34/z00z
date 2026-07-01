use super::{
    build_tx_econ, write_file, ConfirmRow, PendingRow, SelectedInputRow, TxEcon, WalletDiffDump,
    WalletStateDump,
};

pub(crate) fn write_wallet_report_md(
    path: &std::path::Path,
    stage: u32,
    before: &WalletStateDump,
    after: &WalletStateDump,
    diff: &WalletDiffDump,
    selected: &[SelectedInputRow],
    pending_rows: &[PendingRow],
    confirm_rows: &[ConfirmRow],
) -> Result<(), String> {
    let econ = build_tx_econ(selected, confirm_rows);
    let mut md = String::new();

    push_md_summary(&mut md, stage, before, after, diff, selected, pending_rows);
    push_md_tx_econ(&mut md, &econ);
    push_selected_inputs(&mut md, selected);
    push_pending_rows(&mut md, pending_rows);
    push_confirm_rows(&mut md, confirm_rows);
    push_wallet_totals(&mut md, before, after);
    push_serial_dist(&mut md, after);
    push_diff_rows(&mut md, diff);

    write_file(path, md.as_bytes()).map_err(|e| e.to_string())
}

fn push_md_summary(
    md: &mut String,
    stage: u32,
    before: &WalletStateDump,
    after: &WalletStateDump,
    diff: &WalletDiffDump,
    selected: &[SelectedInputRow],
    pending_rows: &[PendingRow],
) {
    md.push_str(&format!("# Stage-{stage} Wallet State Report\n\n"));
    md.push_str("## Summary\n\n");
    md.push_str(&format!(
        "- before wallet states: {}\n- after wallet states: {}\n- diff rows: {}\n- selected inputs: {}\n- pending rows: {}\n\n",
        before.wallets.len(),
        after.wallets.len(),
        diff.rows.len(),
        selected.len(),
        pending_rows.len()
    ));
}

fn push_md_tx_econ(md: &mut String, econ: &TxEcon) {
    md.push_str("## Transaction Economics\n\n");
    md.push_str("| tx_digest | sender_actor | recipient_sum | sender_change_sum | fee_sum | selected_input_sum | output_sum | fee | equation_ok |\n");
    md.push_str("|---|---|---:|---:|---:|---:|---:|---:|---|\n");
    md.push_str(&format!(
        "| {} | {} | {} | {} | {} | {} | {} | {} | {} |\n\n",
        econ.tx_digest_hex,
        econ.sender_actor,
        econ.receiver_sum,
        econ.sender_change_sum,
        econ.fee_sum,
        econ.selected_input_sum,
        econ.output_sum,
        econ.fee,
        if econ.equation_ok { "yes" } else { "no" },
    ));
}

fn push_selected_inputs(md: &mut String, selected: &[SelectedInputRow]) {
    md.push_str("## Selected Inputs (Stage-4 Distinct Serial Source)\n\n");
    md.push_str("| actor | wallet_id | serial_id | asset_id | class | symbol | amount |\n");
    md.push_str("|---|---|---:|---|---|---|---:|\n");
    for row in selected {
        md.push_str(&format!(
            "| {} | {} | {} | {} | {} | {} | {} |\n",
            row.actor,
            row.wallet_id,
            row.serial_id,
            row.asset_id_hex,
            row.class,
            row.symbol,
            row.amount,
        ));
    }
    md.push('\n');
}

fn push_pending_rows(md: &mut String, pending_rows: &[PendingRow]) {
    md.push_str("## Pending Lifecycle Overlay (Pre-Consensus)\n\n");
    md.push_str("| actor | wallet_id | serial_id | asset_id | class | amount | output_role | lifecycle_status | tx_digest |\n");
    md.push_str("|---|---|---:|---|---|---:|---|---|---|\n");
    for row in pending_rows {
        md.push_str(&format!(
            "| {} | {} | {} | {} | {} | {} | {} | {} | {} |\n",
            row.actor,
            row.wallet_id,
            row.serial_id,
            row.asset_id_hex,
            row.class,
            row.amount,
            row.output_role.clone().unwrap_or_default(),
            row.lifecycle_status,
            row.tx_digest_hex,
        ));
    }
    md.push('\n');
}

fn push_confirm_rows(md: &mut String, confirm_rows: &[ConfirmRow]) {
    md.push_str("## Confirmed Lifecycle Overlay (Post-Consensus)\n\n");
    md.push_str("| actor | wallet_id | serial_id | asset_id | class | amount | output_role | lifecycle_status | tx_digest |\n");
    md.push_str("|---|---|---:|---|---|---:|---|---|---|\n");
    for row in confirm_rows {
        md.push_str(&format!(
            "| {} | {} | {} | {} | {} | {} | {} | {} | {} |\n",
            row.actor,
            row.wallet_id,
            row.serial_id,
            row.asset_id_hex,
            row.class,
            row.amount,
            row.output_role.clone().unwrap_or_default(),
            row.lifecycle_status,
            row.tx_digest_hex,
        ));
    }
}

fn push_wallet_totals(md: &mut String, before: &WalletStateDump, after: &WalletStateDump) {
    md.push_str("## Wallet Totals Before/After\n\n");
    md.push_str("| actor | wallet_id | wlt_exists | wlt_size_before | wlt_size_after | items_before | items_after | serials_before | serials_after |\n");
    md.push_str("|---|---|---:|---:|---:|---:|---:|---:|---:|\n");
    for before_wallet in &before.wallets {
        if let Some(after_wallet) = after
            .wallets
            .iter()
            .find(|row| row.wallet_id == before_wallet.wallet_id)
        {
            md.push_str(&format!(
                "| {} | {} | {} | {} | {} | {} | {} | {} | {} |\n",
                before_wallet.actor,
                before_wallet.wallet_id,
                if after_wallet.wlt_exists { "yes" } else { "no" },
                before_wallet.wlt_size_bytes,
                after_wallet.wlt_size_bytes,
                before_wallet.item_count,
                after_wallet.item_count,
                before_wallet.serial_dist.len(),
                after_wallet.serial_dist.len(),
            ));
        }
    }
    md.push('\n');
}

fn push_serial_dist(md: &mut String, after: &WalletStateDump) {
    md.push_str("## Serial Distribution After\n\n");
    md.push_str("| actor | serial_id | rows | total_amount |\n");
    md.push_str("|---|---:|---:|---:|\n");
    for wallet in &after.wallets {
        for row in &wallet.serial_dist {
            md.push_str(&format!(
                "| {} | {} | {} | {} |\n",
                wallet.actor, row.serial_id, row.row_count, row.total_amount
            ));
        }
    }
    md.push('\n');
}

fn push_diff_rows(md: &mut String, diff: &WalletDiffDump) {
    md.push_str("## Asset Status Delta\n\n");
    md.push_str("| actor | serial_id | asset_id | class | output_role | before | after | status | lifecycle_status | tx_digest |\n");
    md.push_str("|---|---:|---|---|---|---:|---:|---|---|---|\n");
    for row in &diff.rows {
        md.push_str(&format!(
            "| {} | {} | {} | {} | {} | {} | {} | {} | {} | {} |\n",
            row.actor,
            row.serial_id,
            row.asset_id_hex,
            row.class,
            row.output_role.clone().unwrap_or_default(),
            row.before_amount.unwrap_or(0),
            row.after_amount.unwrap_or(0),
            row.status,
            row.lifecycle_status,
            row.tx_digest_hex.clone().unwrap_or_default(),
        ));
    }
}
