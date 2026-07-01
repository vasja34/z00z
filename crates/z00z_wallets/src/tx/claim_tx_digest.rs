use z00z_utils::codec::{Codec, JsonCodec};

use super::{claim_tx_wire::ClaimScopeKey, claim_tx_wire::ClaimTxWire};

/// Build canonical claim transaction digest using domain-separated Blake3.
pub fn build_claim_tx_digest(
    kind: &str,
    package_type: &str,
    version: u32,
    chain_id: u32,
    chain_type: &str,
    chain_name: &str,
    tx: &ClaimTxWire,
) -> Result<String, String> {
    let codec = JsonCodec;
    let tx_json = codec.serialize(tx).map_err(|e| e.to_string())?;

    let mut h = blake3::Hasher::new();
    h.update(b"z00z.claim.digest.v1");
    h.update(kind.as_bytes());
    h.update(package_type.as_bytes());
    h.update(&version.to_le_bytes());
    h.update(&chain_id.to_le_bytes());
    h.update(chain_type.as_bytes());
    h.update(chain_name.as_bytes());
    h.update(&tx_json);
    Ok(hex::encode(*h.finalize().as_bytes()))
}

/// Compute deterministic claim scope hash with explicit length prefixes.
pub fn compute_claim_scope_hash(key: &ClaimScopeKey) -> [u8; 32] {
    let mut h = blake3::Hasher::new();
    h.update(b"z00z.claim.scope.v1");

    h.update(&key.chain_id.to_le_bytes());

    let tag = key.scenario_tag.as_bytes();
    h.update(&(tag.len() as u32).to_le_bytes());
    h.update(tag);

    h.update(&key.ruleset_version.to_le_bytes());
    *h.finalize().as_bytes()
}

/// Derive deterministic output nonce from claim id and output index.
pub fn derive_output_nonce(claim_id: &[u8; 32], output_index: u32) -> [u8; 32] {
    let mut h = blake3::Hasher::new();
    h.update(b"z00z.output.nonce.v1");
    h.update(claim_id);
    h.update(&output_index.to_le_bytes());
    *h.finalize().as_bytes()
}
