use super::{
    format_system_time_local, Codec, ConfirmRow, Deserialize, JsonCodec, Path, SystemTimeProvider,
    TimeProvider, TxStorage, WalletDiffDump, WalletDiffRow, WalletItemRow, WalletStateDump,
    WalletStateRow,
};
use std::collections::{BTreeMap, BTreeSet};
use z00z_utils::io::read_file;

// Report alignment only. This keeps serial consistency visible in diff rows
// without redefining the canonical state key, which remains asset_id.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct RowRef {
    asset_id_hex: String,
    serial_id: u32,
}

pub(crate) fn build_wallet_diff(
    stage: u32,
    before: &WalletStateDump,
    after: &WalletStateDump,
    confirm_rows: &[ConfirmRow],
) -> WalletDiffDump {
    let mut rows = Vec::<WalletDiffRow>::new();
    let empty_items: &[WalletItemRow] = &[];

    for wallet_id in merged_wallet_keys(before, after, confirm_rows) {
        let before_wallet = before.wallets.iter().find(|row| row.wallet_id == wallet_id);
        let after_wallet = after.wallets.iter().find(|row| row.wallet_id == wallet_id);
        let Some(actor) = find_actor(before_wallet, after_wallet, confirm_rows, &wallet_id) else {
            continue;
        };

        rows.extend(build_wallet_diff_rows(
            &actor,
            &wallet_id,
            before_wallet
                .map(|row| row.items.as_slice())
                .unwrap_or(empty_items),
            after_wallet
                .map(|row| row.items.as_slice())
                .unwrap_or(empty_items),
            confirm_rows,
        ));
    }

    rows.sort_by(|a, b| {
        a.actor
            .cmp(&b.actor)
            .then_with(|| a.serial_id.cmp(&b.serial_id))
            .then_with(|| a.asset_id_hex.cmp(&b.asset_id_hex))
    });

    WalletDiffDump {
        stage,
        generated_at: format_system_time_local(SystemTimeProvider.now()),
        rows,
    }
}

pub(crate) fn merge_wallet_state_dump(
    path: &Path,
    incoming: &WalletStateDump,
) -> Result<WalletStateDump, String> {
    let Ok(bytes) = read_file(path) else {
        return Ok(incoming.clone());
    };
    let mut merged: WalletStateDump = JsonCodec.deserialize(&bytes).map_err(|e| {
        format!(
            "stage7: wallet state merge decode failed at {}: {e}",
            path.display()
        )
    })?;

    for wallet in &incoming.wallets {
        if let Some(slot) = merged
            .wallets
            .iter_mut()
            .find(|item| item.wallet_id == wallet.wallet_id)
        {
            *slot = wallet.clone();
        } else {
            merged.wallets.push(wallet.clone());
        }
    }

    merged.wallets.sort_by(|a, b| {
        a.actor
            .cmp(&b.actor)
            .then_with(|| a.wallet_id.cmp(&b.wallet_id))
    });
    merged.stage = incoming.stage;
    merged.phase = incoming.phase.clone();
    merged.generated_at = incoming.generated_at.clone();
    Ok(merged)
}

pub(crate) fn merge_wallet_diff_dump(
    path: &Path,
    incoming: &WalletDiffDump,
) -> Result<WalletDiffDump, String> {
    let Ok(bytes) = read_file(path) else {
        return Ok(incoming.clone());
    };
    let existing: WalletDiffDump = JsonCodec.deserialize(&bytes).map_err(|e| {
        format!(
            "stage7: wallet diff merge decode failed at {}: {e}",
            path.display()
        )
    })?;
    let mut rows = BTreeMap::<(String, String, String, u32), WalletDiffRow>::new();

    for row in existing.rows {
        rows.insert(
            (
                row.actor.clone(),
                row.wallet_id.clone(),
                row.asset_id_hex.clone(),
                row.serial_id,
            ),
            row,
        );
    }
    for row in &incoming.rows {
        rows.insert(
            (
                row.actor.clone(),
                row.wallet_id.clone(),
                row.asset_id_hex.clone(),
                row.serial_id,
            ),
            row.clone(),
        );
    }

    Ok(WalletDiffDump {
        stage: incoming.stage,
        generated_at: incoming.generated_at.clone(),
        rows: rows.into_values().collect(),
    })
}

pub(crate) fn wallet_amount_total(dump: &WalletStateDump, wallet_id: &str) -> u64 {
    dump.wallets
        .iter()
        .find(|row| row.wallet_id == wallet_id)
        .map(|row| row.items.iter().map(|item| item.amount).sum())
        .unwrap_or(0)
}

fn build_wallet_diff_rows(
    actor: &str,
    wallet_id: &str,
    before_items: &[WalletItemRow],
    after_items: &[WalletItemRow],
    confirm_rows: &[ConfirmRow],
) -> Vec<WalletDiffRow> {
    let before_map = item_map(before_items);
    let after_map = item_map(after_items);
    let confirm_for_wallet = confirm_map(confirm_rows, wallet_id);
    let keys = merged_diff_keys(&before_map, &after_map, &confirm_for_wallet);

    keys.into_iter()
        .map(|key| {
            build_diff_row(
                actor,
                wallet_id,
                &before_map,
                &after_map,
                &confirm_for_wallet,
                key,
            )
        })
        .collect()
}

fn merged_wallet_keys(
    before: &WalletStateDump,
    after: &WalletStateDump,
    confirm_rows: &[ConfirmRow],
) -> Vec<String> {
    let mut keys = BTreeSet::<String>::new();
    keys.extend(before.wallets.iter().map(|row| row.wallet_id.clone()));
    keys.extend(after.wallets.iter().map(|row| row.wallet_id.clone()));
    keys.extend(confirm_rows.iter().map(|row| row.wallet_id.clone()));
    keys.into_iter().collect()
}

fn find_actor(
    before: Option<&WalletStateRow>,
    after: Option<&WalletStateRow>,
    confirm_rows: &[ConfirmRow],
    wallet_id: &str,
) -> Option<String> {
    before
        .map(|row| row.actor.clone())
        .or_else(|| after.map(|row| row.actor.clone()))
        .or_else(|| {
            confirm_rows
                .iter()
                .find(|row| row.wallet_id == wallet_id)
                .map(|row| row.actor.clone())
        })
}

fn item_map(items: &[WalletItemRow]) -> BTreeMap<RowRef, (&String, u64)> {
    items
        .iter()
        .map(|item| {
            (
                row_ref(item.asset_id_hex.clone(), item.serial_id),
                (&item.class, item.amount),
            )
        })
        .collect()
}

fn confirm_map<'a>(
    confirm_rows: &'a [ConfirmRow],
    wallet_id: &str,
) -> BTreeMap<RowRef, &'a ConfirmRow> {
    confirm_rows
        .iter()
        .filter(|row| row.wallet_id == wallet_id)
        .map(|row| (row_ref(row.asset_id_hex.clone(), row.serial_id), row))
        .collect()
}

fn merged_diff_keys(
    before_map: &BTreeMap<RowRef, (&String, u64)>,
    after_map: &BTreeMap<RowRef, (&String, u64)>,
    confirm_map: &BTreeMap<RowRef, &ConfirmRow>,
) -> Vec<RowRef> {
    let mut keys = BTreeSet::<RowRef>::new();
    keys.extend(before_map.keys().cloned());
    keys.extend(after_map.keys().cloned());
    keys.extend(confirm_map.keys().cloned());
    keys.into_iter().collect()
}

fn build_diff_row(
    actor: &str,
    wallet_id: &str,
    before_map: &BTreeMap<RowRef, (&String, u64)>,
    after_map: &BTreeMap<RowRef, (&String, u64)>,
    confirm_map: &BTreeMap<RowRef, &ConfirmRow>,
    key: RowRef,
) -> WalletDiffRow {
    let before = before_map.get(&key).copied();
    let after = after_map.get(&key).copied();
    let confirmed = confirm_map.get(&key).copied();

    WalletDiffRow {
        actor: actor.to_string(),
        wallet_id: wallet_id.to_string(),
        asset_id_hex: key.asset_id_hex,
        serial_id: key.serial_id,
        class: diff_class(before, after, confirmed),
        output_role: confirmed.and_then(|row| row.output_role.clone()),
        before_amount: before.map(|(_, amount)| amount),
        after_amount: after.map(|(_, amount)| amount),
        status: diff_status(
            before.map(|(_, amount)| amount),
            after.map(|(_, amount)| amount),
        ),
        lifecycle_status: confirmed
            .map(|row| row.lifecycle_status.clone())
            .unwrap_or_else(|| "none".to_string()),
        tx_digest_hex: confirmed.map(|row| row.tx_digest_hex.clone()),
    }
}

fn row_ref(asset_id_hex: String, serial_id: u32) -> RowRef {
    RowRef {
        asset_id_hex,
        serial_id,
    }
}

fn diff_class(
    before: Option<(&String, u64)>,
    after: Option<(&String, u64)>,
    confirmed: Option<&ConfirmRow>,
) -> String {
    before
        .map(|(class, _)| class.clone())
        .or_else(|| after.map(|(class, _)| class.clone()))
        .or_else(|| confirmed.map(|row| row.class.clone()))
        .unwrap_or_else(|| "Unknown".to_string())
}

fn diff_status(before: Option<u64>, after: Option<u64>) -> String {
    match (before, after) {
        (Some(_), None) => "spent",
        (None, Some(_)) => "new",
        (Some(before), Some(after)) if before != after => "changed",
        (Some(_), Some(_)) => "unchanged",
        (None, None) => "unknown",
    }
    .to_string()
}
