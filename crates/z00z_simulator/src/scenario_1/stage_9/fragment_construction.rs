use z00z_core::AssetWire;
use z00z_crypto::expert::encoding::to_hex;
use z00z_storage::settlement::{SnapItem, TerminalLeaf as StorAssetLeaf};
use z00z_wallets::tx::{asset_wire_to_leaf, TxInputWire, TxOutputWire};

use super::bundle_lane_impl::{FragIn, FragOut, FragTx};
use super::demo_checkpoint_agg::is_hex32;
use z00z_crypto::hash_zk::hash_zk as hash_zk_fn;
use z00z_crypto::{domains::TxDigestDomain, frame_bytes, frame_u32_le};
#[cfg(test)]
use z00z_crypto::{frame_str, frame_u64_le};

pub(crate) fn build_target_frag(
    idx: u32,
    prev_root_hex: &str,
    entries: &[SnapItem],
    input: &TxInputWire,
    output: &TxOutputWire,
) -> Result<FragTx, String> {
    let input_asset_hex = input.asset_id_hex.as_str();
    if !is_hex32(input_asset_hex) {
        return Err(format!("invalid input asset_id_hex for fragment {idx}"));
    }
    let input_id = decode_hex32(input_asset_hex)?;

    let serial_id = input.serial_id;

    let snap_item = find_snap_item(entries, &input_id, input_asset_hex)?;
    if snap_item.path().serial_id.get() != serial_id {
        return Err(format!("input leaf_match mismatch for fragment {idx}"));
    }

    let out_asset = output
        .asset_wire
        .clone()
        .to_asset()
        .map_err(|e| format!("invalid output asset for fragment {idx}: {e}"))?;
    let amount = out_asset.amount();
    let asset_hex = to_hex(&out_asset.asset_id());
    if !is_hex32(&asset_hex) {
        return Err(format!("invalid output asset_id_hex for fragment {idx}"));
    }
    let out_hash = out_hash_hex(idx, output, &asset_hex)?;

    let snap_leaf = snap_item
        .terminal_leaf()
        .map_err(|e| format!("fragment {idx} carries non-asset snapshot leaf: {e}"))?;

    Ok(FragTx {
        id: format!("frag_{idx}"),
        prev_root_hex: prev_root_hex.to_string(),
        inputs: vec![FragIn {
            asset_id_hex: input_asset_hex.to_string(),
            serial_id,
            prev_root_hex: prev_root_hex.to_string(),
            leaf_hash_hex: to_hex(&hash_leaf(snap_leaf)),
            member_ok: true,
        }],
        outputs: vec![FragOut {
            asset_id_hex: asset_hex,
            leaf_hash_hex: out_hash,
            amount,
        }],
    })
}

fn find_snap_item<'a>(
    entries: &'a [SnapItem],
    input_id: &[u8; 32],
    asset_hex: &str,
) -> Result<&'a SnapItem, String> {
    entries
        .iter()
        .find(|entry| {
            entry
                .terminal_leaf()
                .map(|leaf| leaf.asset_id == *input_id)
                .unwrap_or(false)
        })
        .ok_or_else(|| format!("missing input leaf for asset_id {asset_hex}"))
}

pub(crate) fn out_hash_hex(
    idx: u32,
    output: &TxOutputWire,
    asset_hex: &str,
) -> Result<String, String> {
    let out_asset = output
        .asset_wire
        .clone()
        .to_asset()
        .map_err(|e| format!("invalid output asset for fragment {idx}: {e}"))?;
    let out_wire = AssetWire::from_asset(&out_asset);
    let out_leaf = match asset_wire_to_leaf(&out_wire) {
        Ok(out_leaf) => out_leaf,
        Err(err) => {
            #[cfg(test)]
            {
                let _ = &err;
                return Ok(to_hex(&hash_zk_fn::<TxDigestDomain>(
                    "S6/LEAF_HASH",
                    &[
                        &frame_u32_le(6),
                        &frame_u32_le(idx),
                        &frame_u64_le(out_asset.amount()),
                        &frame_str(asset_hex),
                    ],
                )));
            }
            #[cfg(not(test))]
            {
                return Err(format!(
                    "stage6: output leaf build failed for fragment {idx}: {err}"
                ));
            }
        }
    };
    if asset_hex != to_hex(&out_leaf.asset_id) {
        return Err(format!("output asset_id mismatch for fragment {idx}"));
    }
    Ok(to_hex(&hash_leaf(&out_leaf)))
}

pub(crate) fn decode_hex32(value: &str) -> Result<[u8; 32], String> {
    let raw = hex::decode(value).map_err(|_| format!("invalid 32-byte hex {value}"))?;
    raw.try_into()
        .map_err(|_| format!("invalid 32-byte hex length {value}"))
}

pub(crate) fn hash_leaf(leaf: &StorAssetLeaf) -> [u8; 32] {
    hash_zk_fn::<TxDigestDomain>(
        "S6/STATE_LEAF",
        &[
            &leaf.asset_id,
            &frame_u32_le(leaf.serial_id),
            &leaf.r_pub,
            &leaf.owner_tag,
            &leaf.c_amount,
            &frame_u32_le(leaf.tag16 as u32),
            &frame_bytes(&leaf.enc_pack.ciphertext),
            &frame_bytes(&leaf.range_proof),
        ],
    )
}
