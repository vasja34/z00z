//! Tx lifecycle projection helpers extracted from simulator Stage-4.

use z00z_core::AssetWire;
use z00z_crypto::expert::encoding::to_hex;

use super::{asset_wire_to_leaf, OutputBundle, TxOutRole, TxOutputWire};

#[derive(Clone, Copy)]
struct PartyRef<'a> {
    name: &'a str,
    wallet: &'a str,
}

/// Pending lifecycle projection row.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PendingEnt {
    /// Actor name.
    pub actor: String,
    /// Wallet id.
    pub wallet_id: String,
    /// Asset id in hex.
    pub asset_id_hex: String,
    /// Asset serial id.
    pub serial_id: u32,
    /// Asset class text.
    pub class: String,
    /// Asset amount.
    pub amount: u64,
    /// Pending lifecycle status.
    pub life_status: String,
    /// Output role label when the row represents a created output.
    pub out_role: Option<String>,
    /// Tx digest in hex.
    pub tx_digest_hex: String,
}

/// Confirmed lifecycle projection row.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConfirmEnt {
    /// Actor name.
    pub actor: String,
    /// Wallet id.
    pub wallet_id: String,
    /// Asset id in hex.
    pub asset_id_hex: String,
    /// Asset serial id.
    pub serial_id: u32,
    /// Asset class text.
    pub class: String,
    /// Asset amount.
    pub amount: u64,
    /// Confirmed lifecycle status.
    pub life_status: String,
    /// Output role label when the row represents a created output.
    pub out_role: Option<String>,
    /// Tx digest in hex.
    pub tx_digest_hex: String,
}

fn pending_stat(role: TxOutRole) -> &'static str {
    match role {
        TxOutRole::Recipient => "pending_receive",
        TxOutRole::Change => "pending_change",
        TxOutRole::Fee => "pending_fee",
    }
}

fn confirm_stat(status: &str) -> &'static str {
    match status {
        "pending_spend" => "confirmed_spend",
        "pending_receive" => "confirmed_receive",
        "pending_change" => "confirmed_change",
        "pending_fee" => "confirmed_fee",
        _ => "confirmed_unknown",
    }
}

/// Build pending rows for sender spends and receiver receives.
pub fn build_pending_rows(
    sender: (&str, &str),
    receiver: (&str, &str),
    fee: (&str, &str),
    selected: &[AssetWire],
    _outputs: &[OutputBundle],
    tx_outputs: &[TxOutputWire],
    tx_digest_hex: &str,
) -> Result<Vec<PendingEnt>, String> {
    let sender = PartyRef {
        name: sender.0,
        wallet: sender.1,
    };
    let receiver = PartyRef {
        name: receiver.0,
        wallet: receiver.1,
    };
    let fee = PartyRef {
        name: fee.0,
        wallet: fee.1,
    };
    let mut out = Vec::<PendingEnt>::new();

    for row in selected {
        let leaf = asset_wire_to_leaf(row)?;
        out.push(PendingEnt {
            actor: sender.name.to_string(),
            wallet_id: sender.wallet.to_string(),
            asset_id_hex: to_hex(&leaf.asset_id),
            serial_id: row.serial_id,
            class: format!("{:?}", row.definition.class),
            amount: row.amount,
            life_status: "pending_spend".to_string(),
            out_role: None,
            tx_digest_hex: tx_digest_hex.to_string(),
        });
    }

    for wire in tx_outputs.iter() {
        let party = party_for_role(wire.role, sender, receiver, fee);
        let leaf = asset_wire_to_leaf(
            &wire
                .asset_wire
                .clone()
                .to_wire()
                .map_err(|e| format!("pending rows: output wire decode failed: {e}"))?,
        )?;

        out.push(PendingEnt {
            actor: party.name.to_string(),
            wallet_id: party.wallet.to_string(),
            asset_id_hex: to_hex(&leaf.asset_id),
            serial_id: wire.asset_wire.serial_id,
            class: format!("{:?}", wire.asset_wire.definition.class),
            amount: wire.asset_wire.amount,
            life_status: pending_stat(wire.role).to_string(),
            out_role: Some(format!("{:?}", wire.role)),
            tx_digest_hex: tx_digest_hex.to_string(),
        });
    }

    Ok(out)
}

/// Build confirmed rows from pending rows.
pub fn build_confirm_rows(pending_rows: &[PendingEnt]) -> Vec<ConfirmEnt> {
    pending_rows
        .iter()
        .map(|row| {
            let life_status = confirm_stat(row.life_status.as_str()).to_string();

            ConfirmEnt {
                actor: row.actor.clone(),
                wallet_id: row.wallet_id.clone(),
                asset_id_hex: row.asset_id_hex.clone(),
                serial_id: row.serial_id,
                class: row.class.clone(),
                amount: row.amount,
                life_status,
                out_role: row.out_role.clone(),
                tx_digest_hex: row.tx_digest_hex.clone(),
            }
        })
        .collect()
}

/// Validate confirm rows against pending rows with strict transition checks.
pub fn validate_confirm_rows(
    pending_rows: &[PendingEnt],
    confirm_rows: &[ConfirmEnt],
) -> Result<(), String> {
    if pending_rows.len() != confirm_rows.len() {
        return Err(format!(
            "stage4: confirm rows mismatch: pending={} confirmed={}",
            pending_rows.len(),
            confirm_rows.len()
        ));
    }

    for (pending, confirmed) in pending_rows.iter().zip(confirm_rows.iter()) {
        if !same_row_key(pending, confirmed) {
            return Err("stage4: confirmation integrity mismatch".to_string());
        }

        if !is_valid_transition(&pending.life_status, &confirmed.life_status) {
            return Err(format!(
                "stage4: invalid lifecycle transition {} -> {}",
                pending.life_status, confirmed.life_status
            ));
        }
    }

    Ok(())
}

fn party_for_role<'a>(
    role: TxOutRole,
    sender: PartyRef<'a>,
    receiver: PartyRef<'a>,
    fee: PartyRef<'a>,
) -> PartyRef<'a> {
    match role {
        TxOutRole::Recipient => receiver,
        TxOutRole::Change => sender,
        TxOutRole::Fee => fee,
    }
}

fn same_row_key(pending: &PendingEnt, confirmed: &ConfirmEnt) -> bool {
    pending.wallet_id == confirmed.wallet_id
        && pending.asset_id_hex == confirmed.asset_id_hex
        && pending.serial_id == confirmed.serial_id
        && pending.tx_digest_hex == confirmed.tx_digest_hex
        && pending.amount == confirmed.amount
        && pending.out_role == confirmed.out_role
}

fn is_valid_transition(pending: &str, confirmed: &str) -> bool {
    matches!(
        (pending, confirmed),
        ("pending_spend", "confirmed_spend")
            | ("pending_receive", "confirmed_receive")
            | ("pending_change", "confirmed_change")
            | ("pending_fee", "confirmed_fee")
    )
}

#[cfg(test)]
mod tests {
    use super::{
        build_confirm_rows, build_pending_rows, confirm_stat, pending_stat, ConfirmEnt, PendingEnt,
        TxOutRole, TxOutputWire,
    };
    use crate::stealth::bind_stealth_output_wire;
    use z00z_core::{genesis::asset_std::asset_from_dev_class, AssetPkgWire};

    use crate::{
        key::{ReceiverKeys, ReceiverSecret},
        receiver::ReceiverCard,
    };

    fn mk_card(seed: u8) -> ReceiverCard {
        let recv = ReceiverSecret::from_bytes([seed; 32]).expect("receiver secret");
        ReceiverKeys::from_receiver_secret(recv)
            .expect("receiver keys")
            .export_receiver_card()
            .expect("receiver card")
    }

    #[test]
    fn test_role_status_maps() {
        assert_eq!(pending_stat(TxOutRole::Recipient), "pending_receive");
        assert_eq!(pending_stat(TxOutRole::Change), "pending_change");
        assert_eq!(pending_stat(TxOutRole::Fee), "pending_fee");
        assert_eq!(confirm_stat("pending_receive"), "confirmed_receive");
        assert_eq!(confirm_stat("pending_change"), "confirmed_change");
        assert_eq!(confirm_stat("pending_fee"), "confirmed_fee");
    }

    #[test]
    fn test_confirm_rows_keep_change() {
        let pending = vec![PendingEnt {
            actor: "alice".to_string(),
            wallet_id: "alice_wallet".to_string(),
            asset_id_hex: "aa".to_string(),
            serial_id: 7,
            class: "Coin".to_string(),
            amount: 5,
            life_status: "pending_change".to_string(),
            out_role: Some("Change".to_string()),
            tx_digest_hex: "bb".to_string(),
        }];

        let confirm: Vec<ConfirmEnt> = build_confirm_rows(&pending);
        assert_eq!(confirm[0].life_status, "confirmed_change");
    }

    #[test]
    fn test_change_rows_ignore_receiver() {
        let out = crate::stealth::build_output_bundle(
            "alice".to_string(),
            TxOutRole::Change,
            z00z_core::assets::AssetClass::Coin,
            &mk_card(9),
            7,
            9,
        )
        .expect("output bundle");
        let asset = asset_from_dev_class(z00z_core::assets::AssetClass::Coin, 9, 7).expect("asset");
        let wire = bind_stealth_output_wire(z00z_core::AssetWire::from_asset(&asset), &out.leaf)
            .expect("bind output wire");
        let tx_outputs = vec![TxOutputWire {
            role: TxOutRole::Change,
            asset_wire: AssetPkgWire::from_wire(&wire),
        }];

        let rows = build_pending_rows(
            ("alice", "alice_wallet"),
            ("alice_observer", "bob_wallet"),
            ("fee", "fee_wallet"),
            &[],
            &[],
            &tx_outputs,
            "tx_digest",
        )
        .expect("pending rows");

        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].actor, "alice");
        assert_eq!(rows[0].wallet_id, "alice_wallet");
        assert_eq!(rows[0].life_status, "pending_change");
        assert_eq!(rows[0].out_role.as_deref(), Some("Change"));
    }
}
