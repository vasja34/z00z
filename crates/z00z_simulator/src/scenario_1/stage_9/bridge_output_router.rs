use z00z_core::assets::AssetPkgWire;
use z00z_crypto::expert::encoding::to_hex;
use z00z_wallets::{
    receiver::ReceiverCard,
    stealth::{bind_stealth_output_wire, build_card_stealth_leaf},
    tx::{derive_tx_output_nonce, TxOutRole, TxOutputWire},
};

use super::bundle_lane_impl::MadeEnt;
use super::fragment_construction::out_hash_hex;

pub(crate) fn build_bridge_out(
    output_idx: usize,
    output: &TxOutputWire,
    charlie: &ReceiverCard,
) -> Result<TxOutputWire, String> {
    let asset = output
        .asset_wire
        .clone()
        .to_asset()
        .map_err(|e| format!("stage6: bridge output asset decode failed: {e}"))?;
    let leaf = build_card_stealth_leaf(charlie, asset.amount, asset.serial_id)
        .map_err(|e| format!("stage6: charlie bridge output build failed: {e}"))?;
    let wire = output
        .asset_wire
        .clone()
        .to_wire()
        .map_err(|e| format!("stage6: bridge output wire decode failed: {e}"))?;
    let mut bound = bind_stealth_output_wire(wire, &leaf)
        .map_err(|e| format!("stage6: charlie bridge bind failed: {e}"))?;
    bound.nonce = derive_tx_output_nonce(&leaf, output_idx);
    Ok(TxOutputWire {
        role: TxOutRole::Recipient,
        asset_wire: AssetPkgWire::from_wire(&bound),
    })
}

pub(crate) fn build_made_rows(outputs: &[TxOutputWire]) -> Result<Vec<MadeEnt>, String> {
    outputs
        .iter()
        .enumerate()
        .map(|(idx, output)| {
            let asset = output
                .asset_wire
                .clone()
                .to_asset()
                .map_err(|e| format!("stage6: output asset decode failed: {e}"))?;
            let asset_hex = to_hex(&asset.asset_id());
            let leaf_hash_hex = out_hash_hex(idx as u32 + 1, output, &asset_hex)?;
            Ok(MadeEnt {
                asset_id_hex: asset_hex,
                leaf_hash_hex,
            })
        })
        .collect()
}
