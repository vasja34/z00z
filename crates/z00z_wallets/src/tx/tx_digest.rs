use blake3::Hasher;
use z00z_crypto::{frame_bytes, frame_str, frame_u32_le};
use z00z_utils::codec::{Codec, JsonCodec};

use super::spend_verification::canonicalize_regular_spend_tx;
use super::tx_wire::{canonicalize_tx_inputs, TxAuthWire, TxWire, REGULAR_TX_TYPE};

fn normalize_digest_tx(tx: &TxWire) -> Result<TxWire, String> {
    let mut normalized = canonicalize_tx_inputs(tx).map_err(str::to_string)?;
    for output in &mut normalized.outputs {
        output.asset_wire.range_proof = None;
        output.asset_wire.owner_signature = None;
    }
    if normalized.tx_type == REGULAR_TX_TYPE {
        normalized.auth = TxAuthWire::default();
        if let Some(spend) = normalized.proof.spend.as_mut() {
            spend.statement_hex.clear();
            spend.proof_hex.clear();
        }
        normalized = canonicalize_regular_spend_tx(&normalized).map_err(|err| err.to_string())?;
    }
    Ok(normalized)
}

/// Build canonical package digest over envelope context and tx payload.
pub fn build_tx_package_digest(
    kind: &str,
    package_type: &str,
    version: u8,
    chain_id: u32,
    chain_type: &str,
    chain_name: &str,
    tx: &TxWire,
) -> Result<String, String> {
    let digest_tx = normalize_digest_tx(tx)?;
    let tx_json = JsonCodec.serialize(&digest_tx).map_err(|e| e.to_string())?;
    let mut hasher = Hasher::new();
    hasher.update(b"z00z.tx.pkg.digest.v2.");
    hasher.update(&frame_str(kind));
    hasher.update(&frame_str(package_type));
    hasher.update(&frame_bytes(&[version]));
    hasher.update(&frame_u32_le(chain_id));
    hasher.update(&frame_str(chain_type));
    hasher.update(&frame_str(chain_name));
    hasher.update(&frame_bytes(&tx_json));
    Ok(hex::encode(*hasher.finalize().as_bytes()))
}
