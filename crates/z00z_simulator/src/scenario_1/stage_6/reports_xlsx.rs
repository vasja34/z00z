use super::{
    build_tx_econ, ConfirmRow, PendingRow, SelectedInputRow, TxEcon, WalletDiffDump,
    WalletStateDump, Workbook, XlsxError,
};

enum CellValue<'a> {
    Str(&'a str),
    Num(f64),
}

pub(crate) fn write_wallet_report_xlsx(
    path: &std::path::Path,
    before: &WalletStateDump,
    after: &WalletStateDump,
    diff: &WalletDiffDump,
    selected: &[SelectedInputRow],
    pending_rows: &[PendingRow],
    confirm_rows: &[ConfirmRow],
) -> Result<(), String> {
    let econ = build_tx_econ(selected, confirm_rows);
    let mut book = Workbook::new();

    write_summary_sheet(&mut book, before, after)?;
    write_tx_econ_sheet(&mut book, &econ)?;
    write_confirm_sheet(&mut book, confirm_rows)?;
    write_serial_after_sheet(&mut book, after)?;
    write_selected_sheet(&mut book, selected)?;
    write_pending_sheet(&mut book, pending_rows)?;
    write_diff_sheet(&mut book, diff)?;

    book.save(path).map_err(map_xlsx_err)
}

fn write_summary_sheet(
    book: &mut Workbook,
    before: &WalletStateDump,
    after: &WalletStateDump,
) -> Result<(), String> {
    let sheet = book.add_worksheet();
    sheet.set_name("summary").map_err(map_xlsx_err)?;
    write_headers(
        sheet,
        &[
            "actor",
            "wallet_id",
            "wlt_exists",
            "wlt_size_before",
            "wlt_size_after",
            "items_before",
            "items_after",
            "serials_before",
            "serials_after",
        ],
    )?;

    let mut row = 1u32;
    for before_wallet in &before.wallets {
        if let Some(after_wallet) = after
            .wallets
            .iter()
            .find(|item| item.wallet_id == before_wallet.wallet_id)
        {
            write_row(
                sheet,
                row,
                &[
                    CellValue::Str(&before_wallet.actor),
                    CellValue::Str(&before_wallet.wallet_id),
                    CellValue::Str(if after_wallet.wlt_exists { "yes" } else { "no" }),
                    CellValue::Num(before_wallet.wlt_size_bytes as f64),
                    CellValue::Num(after_wallet.wlt_size_bytes as f64),
                    CellValue::Num(before_wallet.item_count as f64),
                    CellValue::Num(after_wallet.item_count as f64),
                    CellValue::Num(before_wallet.serial_dist.len() as f64),
                    CellValue::Num(after_wallet.serial_dist.len() as f64),
                ],
            )?;
            row = row.saturating_add(1);
        }
    }
    Ok(())
}

fn write_tx_econ_sheet(book: &mut Workbook, econ: &TxEcon) -> Result<(), String> {
    let sheet = book.add_worksheet();
    sheet.set_name("tx_economics").map_err(map_xlsx_err)?;
    write_headers(
        sheet,
        &[
            "tx_digest_hex",
            "sender_actor",
            "recipient_sum",
            "sender_change_sum",
            "fee_sum",
            "selected_input_sum",
            "output_sum",
            "fee",
            "equation_ok",
        ],
    )?;

    write_row(
        sheet,
        1,
        &[
            CellValue::Str(&econ.tx_digest_hex),
            CellValue::Str(&econ.sender_actor),
            CellValue::Num(econ.receiver_sum as f64),
            CellValue::Num(econ.sender_change_sum as f64),
            CellValue::Num(econ.fee_sum as f64),
            CellValue::Num(econ.selected_input_sum as f64),
            CellValue::Num(econ.output_sum as f64),
            CellValue::Num(econ.fee as f64),
            CellValue::Str(if econ.equation_ok { "yes" } else { "no" }),
        ],
    )?;
    Ok(())
}

fn write_confirm_sheet(book: &mut Workbook, confirm_rows: &[ConfirmRow]) -> Result<(), String> {
    let sheet = book.add_worksheet();
    sheet.set_name("confirmed").map_err(map_xlsx_err)?;
    write_headers(
        sheet,
        &[
            "actor",
            "wallet_id",
            "asset_id",
            "serial_id",
            "class",
            "amount",
            "output_role",
            "lifecycle_status",
            "tx_digest_hex",
        ],
    )?;

    let mut row = 1u32;
    for item in confirm_rows {
        write_row(
            sheet,
            row,
            &[
                CellValue::Str(&item.actor),
                CellValue::Str(&item.wallet_id),
                CellValue::Str(&item.asset_id_hex),
                CellValue::Num(item.serial_id as f64),
                CellValue::Str(&item.class),
                CellValue::Num(item.amount as f64),
                CellValue::Str(item.output_role.as_deref().unwrap_or("")),
                CellValue::Str(&item.lifecycle_status),
                CellValue::Str(&item.tx_digest_hex),
            ],
        )?;
        row = row.saturating_add(1);
    }
    Ok(())
}

fn write_serial_after_sheet(book: &mut Workbook, after: &WalletStateDump) -> Result<(), String> {
    let sheet = book.add_worksheet();
    sheet.set_name("serial_after").map_err(map_xlsx_err)?;
    write_headers(sheet, &["actor", "serial_id", "rows", "total_amount"])?;

    let mut row = 1u32;
    for wallet in &after.wallets {
        for dist in &wallet.serial_dist {
            write_row(
                sheet,
                row,
                &[
                    CellValue::Str(&wallet.actor),
                    CellValue::Num(dist.serial_id as f64),
                    CellValue::Num(dist.row_count as f64),
                    CellValue::Num(dist.total_amount as f64),
                ],
            )?;
            row = row.saturating_add(1);
        }
    }
    Ok(())
}

fn write_selected_sheet(book: &mut Workbook, selected: &[SelectedInputRow]) -> Result<(), String> {
    let sheet = book.add_worksheet();
    sheet.set_name("selected_inputs").map_err(map_xlsx_err)?;
    write_headers(
        sheet,
        &[
            "actor",
            "wallet_id",
            "asset_id",
            "serial_id",
            "class",
            "symbol",
            "amount",
        ],
    )?;

    let mut row = 1u32;
    for item in selected {
        write_row(
            sheet,
            row,
            &[
                CellValue::Str(&item.actor),
                CellValue::Str(&item.wallet_id),
                CellValue::Str(&item.asset_id_hex),
                CellValue::Num(item.serial_id as f64),
                CellValue::Str(&item.class),
                CellValue::Str(&item.symbol),
                CellValue::Num(item.amount as f64),
            ],
        )?;
        row = row.saturating_add(1);
    }
    Ok(())
}

fn write_pending_sheet(book: &mut Workbook, pending_rows: &[PendingRow]) -> Result<(), String> {
    let sheet = book.add_worksheet();
    sheet.set_name("pending").map_err(map_xlsx_err)?;
    write_headers(
        sheet,
        &[
            "actor",
            "wallet_id",
            "asset_id",
            "serial_id",
            "class",
            "amount",
            "output_role",
            "lifecycle_status",
            "tx_digest_hex",
        ],
    )?;

    let mut row = 1u32;
    for item in pending_rows {
        write_row(
            sheet,
            row,
            &[
                CellValue::Str(&item.actor),
                CellValue::Str(&item.wallet_id),
                CellValue::Str(&item.asset_id_hex),
                CellValue::Num(item.serial_id as f64),
                CellValue::Str(&item.class),
                CellValue::Num(item.amount as f64),
                CellValue::Str(item.output_role.as_deref().unwrap_or("")),
                CellValue::Str(&item.lifecycle_status),
                CellValue::Str(&item.tx_digest_hex),
            ],
        )?;
        row = row.saturating_add(1);
    }
    Ok(())
}

fn write_diff_sheet(book: &mut Workbook, diff: &WalletDiffDump) -> Result<(), String> {
    let sheet = book.add_worksheet();
    sheet.set_name("diff").map_err(map_xlsx_err)?;
    write_headers(
        sheet,
        &[
            "actor",
            "wallet_id",
            "asset_id",
            "serial_id",
            "class",
            "output_role",
            "before",
            "after",
            "status",
            "lifecycle_status",
            "tx_digest_hex",
        ],
    )?;

    let mut row = 1u32;
    for item in &diff.rows {
        write_row(
            sheet,
            row,
            &[
                CellValue::Str(&item.actor),
                CellValue::Str(&item.wallet_id),
                CellValue::Str(&item.asset_id_hex),
                CellValue::Num(item.serial_id as f64),
                CellValue::Str(&item.class),
                CellValue::Str(item.output_role.as_deref().unwrap_or("")),
                CellValue::Num(item.before_amount.unwrap_or(0) as f64),
                CellValue::Num(item.after_amount.unwrap_or(0) as f64),
                CellValue::Str(&item.status),
                CellValue::Str(&item.lifecycle_status),
                CellValue::Str(item.tx_digest_hex.as_deref().unwrap_or("")),
            ],
        )?;
        row = row.saturating_add(1);
    }
    Ok(())
}

fn write_row(
    sheet: &mut rust_xlsxwriter::Worksheet,
    row: u32,
    cells: &[CellValue<'_>],
) -> Result<(), String> {
    for (index, cell) in cells.iter().enumerate() {
        match cell {
            CellValue::Str(value) => sheet
                .write_string(row, index as u16, *value)
                .map_err(map_xlsx_err)?,
            CellValue::Num(value) => sheet
                .write_number(row, index as u16, *value)
                .map_err(map_xlsx_err)?,
        };
    }
    Ok(())
}

fn write_headers(sheet: &mut rust_xlsxwriter::Worksheet, headers: &[&str]) -> Result<(), String> {
    for (index, header) in headers.iter().enumerate() {
        sheet
            .write_string(0, index as u16, *header)
            .map_err(map_xlsx_err)?;
    }
    Ok(())
}

fn map_xlsx_err(err: XlsxError) -> String {
    format!("stage4: xlsx write failed: {err}")
}
