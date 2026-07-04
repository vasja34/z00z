impl ClaimTxVerifierImpl {
    fn verify_owner_attest(
        tx: &ClaimTxWire,
        chain_id: u32,
        recipient_card: &ReceiverCard,
        leaves: &[&AssetPkgWire],
    ) -> Result<(), ClaimTxError> {
        for (idx, (out, leaf)) in tx.outputs.iter().zip(leaves.iter()).enumerate() {
            Self::verify_out_attest(
                out,
                Some(*leaf),
                true,
                chain_id,
                tx,
                idx as u32,
                Some(recipient_card),
            )?;
        }

        Ok(())
    }

    fn verify_nullifier(tx: &ClaimTxWire, chain_id: u32) -> Result<(), ClaimTxError> {
        let claim_id = decode_hex32(&tx.inputs[0].claim_id_hex, "claim_id_hex")?;
        let owner = decode_hex32(&tx.context.recipient_owner_hex, "recipient_owner_hex")?;
        // Recheck the wallet contract that chain_id is folded into the nullifier.
        let expected = derive_nullifier(&claim_id, &owner, chain_id).to_hex();
        if tx.context.nullifier_hex != expected {
            return Err(ClaimTxError::NullifierMismatch(
                "nullifier_hex must equal derive_nullifier(claim_id, recipient_owner_hex, chain_id)"
                    .to_string(),
            ));
        }

        Ok(())
    }

    fn hash_leaf_wire(leaf: &AssetPkgWire) -> Result<[u8; 32], ClaimTxError> {
        let wire = leaf
            .clone()
            .to_wire()
            .map_err(|e| ClaimTxError::LeafInvalid(format!("to_wire failed: {e}")))?;
        let leaf = asset_wire_to_leaf(&wire)
            .map_err(|e| ClaimTxError::LeafInvalid(format!("jmt leaf conversion failed: {e}")))?;
        let leaf_bytes = JsonCodec
            .serialize(&leaf)
            .map_err(|e| ClaimTxError::LeafInvalid(format!("leaf encode failed: {e}")))?;

        Ok(z00z_crypto::blake2b_hash(
            b"z00z.claim.output_leaf.v1",
            &[leaf_bytes.as_slice()],
        ))
    }

    fn hash_bind_msg(
        tx: &ClaimTxWire,
        chain_id: u32,
        leaves: &[&AssetPkgWire],
    ) -> Result<[u8; 32], ClaimTxError> {
        let claim = tx.inputs.first().ok_or_else(|| {
            ClaimTxError::StructureMalformed("claim tx inputs are empty".to_string())
        })?;
        let mut msgs = Vec::with_capacity(leaves.len());
        for (idx, leaf) in leaves.iter().enumerate() {
            let msg = build_owner_attest_msg(
                chain_id,
                &claim.claim_id_hex,
                &tx.context.recipient_wallet_id,
                &tx.context.recipient_owner_hex,
                &tx.context.claim_scope_hash_hex,
                idx as u32,
                leaf,
            )?;
            msgs.push(msg);
        }
        let refs = msgs.iter().map(Vec::as_slice).collect::<Vec<_>>();
        Ok(z00z_crypto::blake2b_hash(
            b"z00z.claim.owner_bind.v1",
            &refs,
        ))
    }

    fn build_claim_stmt(
        pkg: &ClaimTxPackage,
        leaves: &[&AssetPkgWire],
    ) -> Result<ClaimStmt, ClaimTxError> {
        let claim = pkg.tx.inputs.first().ok_or_else(|| {
            ClaimTxError::StructureMalformed("claim tx inputs are empty".to_string())
        })?;
        if leaves.len() != 1 {
            return Err(ClaimTxError::SourceLeafCount(
                "claim contract requires exactly one portable source leaf".to_string(),
            ));
        }
        let output_leaf_hashes = leaves
            .iter()
            .map(|leaf| Self::hash_leaf_wire(leaf))
            .collect::<Result<Vec<_>, _>>()?;
        let range_ctx_hash = claim_range_hash(pkg, leaves[0])?;
        let source_root = Self::derive_claim_source_root(&pkg.tx)?;

        Ok(ClaimStmt {
            chain_id: pkg.chain_id,
            root_version: CLAIM_ROOT_VERSION,
            proof_ver: ClaimProofVer::V1,
            tx_ver: pkg.version,
            range_ctx_hash,
            claim_id: decode_hex32(&claim.claim_id_hex, "claim_id_hex")?,
            claim_source_asset_id: decode_hex32(
                &claim.claim_source_asset_id_hex,
                "claim_source_asset_id_hex",
            )?,
            claim_source_commitment: decode_hex32(
                &claim.claim_source_commitment_hex,
                "claim_source_commitment_hex",
            )?,
            source_root,
            claim_scope_hash: decode_hex32(
                &pkg.tx.context.claim_scope_hash_hex,
                "claim_scope_hash_hex",
            )?,
            recipient_binding: decode_hex32(
                &pkg.tx.context.recipient_owner_hex,
                "recipient_owner_hex",
            )?,
            nullifier: decode_hex32(&pkg.tx.context.nullifier_hex, "nullifier_hex")?,
            owner_bind_digest: Self::hash_bind_msg(&pkg.tx, pkg.chain_id, leaves)?,
            output_leaf_hashes,
        })
    }

    fn derive_claim_source_root(tx: &ClaimTxWire) -> Result<[u8; 32], ClaimTxError> {
        // The claim statement source_root now comes from the carried
        // claim_source_proof so wallet verification stays bound to the same
        // storage-backed membership contract the producer emitted.
        let proof = decode_claim_source(&tx.proof.proof_hex)?;
        Self::verify_source_head(&proof)?;
        Ok(proof.source_root())
    }

    fn verify_claim_proof(
        tx: &ClaimTxWire,
        stmt: &ClaimStmt,
        leaf_pkg: &AssetPkgWire,
    ) -> Result<(), ClaimTxError> {
        let proof = decode_claim_source(&tx.proof.proof_hex)?;
        Self::verify_source_head(&proof)?;
        stmt.chk_source(&proof)
            .map_err(|e| ClaimTxError::SourceProofMismatch(e.to_string()))?;
        let proof_item = Self::decode_source_item(&proof)?;
        Self::verify_source_meta(stmt, &proof, &proof_item, leaf_pkg)?;
        Self::verify_source_blob(leaf_pkg, &proof)?;
        Ok(())
    }

    fn verify_source_head(proof: &ClaimSourceProof) -> Result<(), ClaimTxError> {
        if proof.source_root() == ZERO_ROOT {
            return Err(ClaimTxError::SourceRootZero(
                "claim_source_proof root is all zero".to_string(),
            ));
        }
        if proof.root_version() != CLAIM_ROOT_VERSION {
            return Err(ClaimTxError::SourceRootVersion(
                proof.root_version().to_string(),
            ));
        }
        // The live storage-backed source-proof contract remains V1 even though
        // the outer claim transport tag is CLAIM_SOURCE_PROOF_TAG.
        if proof.proof_ver() != ClaimProofVer::V1 {
            return Err(ClaimTxError::SourceProofVer(
                proof.proof_ver().as_u8().to_string(),
            ));
        }

        Ok(())
    }

    fn decode_source_item(proof: &ClaimSourceProof) -> Result<ProofItem, ClaimTxError> {
        proof_blob_item(proof.proof_blob()).map_err(|e| {
            ClaimTxError::SourceProofMismatch(format!("claim_source_proof blob decode failed: {e}"))
        })
    }

    fn verify_source_meta(
        stmt: &ClaimStmt,
        proof: &ClaimSourceProof,
        proof_item: &ProofItem,
        leaf_pkg: &AssetPkgWire,
    ) -> Result<(), ClaimTxError> {
        if stmt.source_root() != proof.source_root() {
            return Err(ClaimTxError::SourceProofMismatch(
                "claim statement source_root does not match claim_source_proof root".to_string(),
            ));
        }

        if proof.source_root() != proof_item.root().into_bytes() {
            return Err(ClaimTxError::SourceProofMismatch(
                "claim_source_proof root does not match proof blob root".to_string(),
            ));
        }

        let path = proof_item.path();
        if path.terminal_id().into_bytes() != stmt.claim_source_asset_id {
            return Err(ClaimTxError::SourceProofMismatch(
                "claim_source_proof path asset_id mismatch".to_string(),
            ));
        }

        if leaf_pkg.commitment.as_bytes() != stmt.claim_source_commitment {
            return Err(ClaimTxError::SourceProofMismatch(
                "claim statement source commitment does not match canonical source leaf"
                    .to_string(),
            ));
        }

        Ok(())
    }

    fn verify_source_blob(
        leaf_pkg: &AssetPkgWire,
        proof: &ClaimSourceProof,
    ) -> Result<(), ClaimTxError> {
        let wire = leaf_pkg
            .clone()
            .to_wire()
            .map_err(|e| ClaimTxError::LeafInvalid(format!("to_wire failed: {e}")))?;
        let leaf = asset_wire_to_leaf(&wire)
            .map_err(|e| ClaimTxError::LeafInvalid(format!("jmt leaf conversion failed: {e}")))?;
        let store_leaf = leaf;

        chk_blob_settlement_inclusion_carried(
            proof.proof_blob(),
            SettlementStateRoot::settlement_v1(proof.source_root()),
            &store_leaf,
        )
        .map_err(|e| {
            ClaimTxError::SourceProofMismatch(format!("claim_source_proof blob invalid: {e}"))
        })?;

        Ok(())
    }

    fn verify_claim_authority(tx: &ClaimTxWire, stmt: &ClaimStmt) -> Result<(), ClaimTxError> {
        let sig = decode_claim_auth(&tx.auth.claim_authority_sig_hex)?;
        sig.verify_with_pk(stmt, &claim_auth_pk())
            .map_err(|e| ClaimTxError::AuthoritySigInvalid(e.to_string()))
    }

    fn fail(err: ClaimTxError, report: ClaimTxVerifyReport) -> ClaimVerifyResult {
        ClaimVerifyResult {
            valid: false,
            reject_class: err_class(&err).to_string(),
            errors: vec![err.to_string()],
            report: Some(report),
        }
    }

    fn fail_raw(
        class: &str,
        msg: String,
        report: Option<ClaimTxVerifyReport>,
    ) -> ClaimVerifyResult {
        ClaimVerifyResult {
            valid: false,
            reject_class: class.to_string(),
            errors: vec![msg],
            report,
        }
    }

    fn verify_scope(pkg: &ClaimTxPackage) -> Result<(), ClaimTxError> {
        let scope = ClaimScopeKey {
            chain_id: pkg.chain_id,
            scenario_tag: "scenario_1_genesis_claim".to_string(),
            ruleset_version: 1,
        };
        let expected_scope = hex::encode(compute_claim_scope_hash(&scope));
        if pkg.tx.context.claim_scope_hash_hex != expected_scope {
            return Err(ClaimTxError::StructureMalformed(
                "claim_scope_hash_hex mismatch".to_string(),
            ));
        }
        Ok(())
    }

    fn verify_digest(pkg: &ClaimTxPackage) -> Result<(), ClaimTxError> {
        let expected = match build_claim_tx_digest(
            &pkg.kind,
            &pkg.package_type,
            pkg.version,
            pkg.chain_id,
            &pkg.chain_type,
            &pkg.chain_name,
            &pkg.tx,
        ) {
            Ok(value) => value,
            Err(err) => {
                return Err(ClaimTxError::StructureMalformed(format!(
                    "digest build failed: {err}"
                )))
            }
        };
        if pkg.tx_digest_hex != expected {
            return Err(ClaimTxError::DigestMismatch(
                "tx_digest_hex does not match payload".to_string(),
            ));
        }

        Ok(())
    }
}

// Threat T-2 anchor: raw verifier passes internal consistency only; store
// rebinding via claim_source_contract_for_item is the mandatory second step
// for final admission.
//
// Architectural boundary: ClaimTxVerifier::verify() checks internal
// consistency (structure, digest, auth). It does NOT perform store rebinding.
// claim_source_contract_for_item is required for final admission.
