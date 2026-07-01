use super::{ConfirmRow, SelectedInputRow};

use super::reports_capture as capture;
use super::reports_diff as diff;
use super::reports_md as render_md;
use super::reports_rows as rows;
use super::reports_xlsx as render_xlsx;

pub(crate) use self::capture::capture_wallet_actor;
pub(crate) use self::capture::capture_wallet_states;
pub(crate) use self::diff::{
    build_wallet_diff, merge_wallet_diff_dump, merge_wallet_state_dump, wallet_amount_total,
};
pub(crate) use self::render_md::write_wallet_report_md;
pub(crate) use self::render_xlsx::write_wallet_report_xlsx;
pub(crate) use self::rows::{
    build_confirm_rows, build_pending_rows, build_pending_rows_for_assets, validate_confirm_rows,
};

#[derive(Debug, Clone)]
pub(super) struct TxEcon {
    pub tx_digest_hex: String,
    pub sender_actor: String,
    pub selected_input_sum: u64,
    pub receiver_sum: u64,
    pub sender_change_sum: u64,
    pub fee_sum: u64,
    pub output_sum: u64,
    pub fee: u64,
    pub equation_ok: bool,
}

pub(super) fn build_tx_econ(selected: &[SelectedInputRow], confirm_rows: &[ConfirmRow]) -> TxEcon {
    let sender_actor = selected
        .first()
        .map(|row| row.actor.clone())
        .unwrap_or_default();
    let tx_digest_hex = confirm_rows
        .iter()
        .find(|row| !row.tx_digest_hex.is_empty())
        .map(|row| row.tx_digest_hex.clone())
        .unwrap_or_default();
    let selected_input_sum = selected.iter().map(|row| row.amount).sum::<u64>();
    let (receiver_sum, sender_change_sum, fee_sum) =
        sum_confirm_outputs(confirm_rows, &tx_digest_hex);
    let output_sum = receiver_sum
        .saturating_add(sender_change_sum)
        .saturating_add(fee_sum);
    let fee = fee_sum;

    TxEcon {
        tx_digest_hex,
        sender_actor,
        selected_input_sum,
        receiver_sum,
        sender_change_sum,
        fee_sum,
        output_sum,
        fee,
        equation_ok: selected_input_sum == output_sum,
    }
}

fn sum_confirm_outputs(confirm_rows: &[ConfirmRow], tx_digest_hex: &str) -> (u64, u64, u64) {
    let mut receiver_sum = 0u64;
    let mut sender_change_sum = 0u64;
    let mut fee_sum = 0u64;

    for row in confirm_rows
        .iter()
        .filter(|row| row.tx_digest_hex == tx_digest_hex)
    {
        match row.output_role.as_deref() {
            Some("Recipient") => {
                receiver_sum = receiver_sum.saturating_add(row.amount);
            }
            Some("Change") => {
                sender_change_sum = sender_change_sum.saturating_add(row.amount);
            }
            Some("Fee") => {
                fee_sum = fee_sum.saturating_add(row.amount);
            }
            _ => {}
        }
    }

    (receiver_sum, sender_change_sum, fee_sum)
}
