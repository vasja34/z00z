fn claim_range_hash(
    pkg: &ClaimTxPackage,
    leaf_pkg: &AssetPkgWire,
) -> Result<[u8; 32], ClaimTxError> {
    let wire = leaf_pkg
        .clone()
        .to_wire()
        .map_err(|e| ClaimTxError::LeafInvalid(format!("to_wire failed: {e}")))?;
    let leaf = asset_wire_to_leaf(&wire)
        .map_err(|e| ClaimTxError::LeafInvalid(format!("jmt leaf conversion failed: {e}")))?;

    Ok(output_range_ctx_hash(
        &leaf,
        pkg.chain_id,
        CLAIM_ROOT_VERSION,
        ClaimProofVer::V1.as_u8(),
        CLAIM_PKG,
    ))
}

fn decode_claim_source(value: &str) -> Result<ClaimSourceProof, ClaimTxError> {
    if value.is_empty() || !is_even_hex(value) {
        return Err(ClaimTxError::ProofBlobInvalidHex(
            "proof_hex invalid".to_string(),
        ));
    }
    let proof_bytes = hex::decode(value)
        .map_err(|_| ClaimTxError::ProofBlobInvalidHex("proof_hex decode failed".to_string()))?;
    ClaimSourceProof::from_bytes(&proof_bytes)
        .map_err(|e| ClaimTxError::ProofBlobDecode(e.to_string()))
}

/// Build the canonical root-bound claim contract statement from one verified claim package.
pub fn build_claim_stmt(pkg: &ClaimTxPackage) -> Result<ClaimStmt, String> {
    let leaves =
        ClaimTxVerifierImpl::verify_portable_leaf(&pkg.tx, true).map_err(|e| e.to_string())?;
    ClaimTxVerifierImpl::build_claim_stmt(pkg, &leaves).map_err(|e| e.to_string())
}
