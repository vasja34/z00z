use z00z_utils::codec::{Codec, JsonCodec};

use super::{claim_tx_wire::ClaimScopeKey, claim_tx_wire::ClaimTxWire};

/// Build canonical claim transaction digest using domain-separated Blake2b.
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
    let version_bytes = version.to_le_bytes();
    let chain_id_bytes = chain_id.to_le_bytes();
    Ok(hex::encode(z00z_crypto::blake2b_hash(
        b"z00z.claim.digest.v1",
        &[
            kind.as_bytes(),
            package_type.as_bytes(),
            &version_bytes,
            &chain_id_bytes,
            chain_type.as_bytes(),
            chain_name.as_bytes(),
            tx_json.as_slice(),
        ],
    )))
}

/// Compute deterministic claim scope hash with explicit length prefixes.
pub fn compute_claim_scope_hash(key: &ClaimScopeKey) -> [u8; 32] {
    let chain_id_bytes = key.chain_id.to_le_bytes();
    let ruleset_version_bytes = key.ruleset_version.to_le_bytes();
    z00z_crypto::blake2b_hash(
        b"z00z.claim.scope.v1",
        &[
            &chain_id_bytes,
            key.scenario_tag.as_bytes(),
            &ruleset_version_bytes,
        ],
    )
}

/// Derive deterministic output nonce from claim id and output index.
pub fn derive_output_nonce(claim_id: &[u8; 32], output_index: u32) -> [u8; 32] {
    let output_index_bytes = output_index.to_le_bytes();
    z00z_crypto::blake2b_hash(
        b"z00z.output.nonce.v1",
        &[claim_id.as_slice(), &output_index_bytes],
    )
}
