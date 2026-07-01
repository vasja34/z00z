impl ClaimTxVerifierImpl {
    /// Construct verifier instance.
    pub fn new() -> Self {
        Self
    }

    fn verify_structure(pkg: &ClaimTxPackage) -> Result<(), ClaimTxError> {
        if pkg.kind != TX_PACKAGE_KIND {
            return Err(ClaimTxError::KindMismatch(pkg.kind.clone()));
        }
        if pkg.package_type != CLAIM_TX_PACKAGE_TYPE {
            return Err(ClaimTxError::StructureMalformed(
                "package_type must be claim_tx".to_string(),
            ));
        }
        if pkg.version != CLAIM_PKG {
            return Err(ClaimTxError::VersionUnsupported(pkg.version.to_string()));
        }
        if pkg.chain_id == 0
            || pkg.chain_type.trim().is_empty()
            || pkg.chain_name.trim().is_empty()
        {
            return Err(ClaimTxError::StructureMalformed(
                "claim package chain metadata is incomplete".to_string(),
            ));
        }
        if !is_hex64(&pkg.tx_digest_hex) {
            return Err(ClaimTxError::StructureMalformed(
                "tx_digest_hex: not 64 lowercase hex chars".to_string(),
            ));
        }

        if pkg.status.is_empty() {
            return Err(ClaimTxError::StructureMalformed(
                "status is empty".to_string(),
            ));
        }

        Ok(())
    }

    fn check_hex64(value: &str, err: ClaimTxError) -> Result<(), ClaimTxError> {
        if is_hex64(value) {
            Ok(())
        } else {
            Err(err)
        }
    }

    fn verify_tx_fields(tx: &ClaimTxWire) -> Result<(), ClaimTxError> {
        Self::verify_tx_hexes(tx)?;
        Self::verify_tx_core(tx)?;

        Ok(())
    }

    fn verify_tx_hexes(tx: &ClaimTxWire) -> Result<(), ClaimTxError> {
        if tx.tx_type != CLAIM_TX_TYPE {
            return Err(ClaimTxError::StructureMalformed(
                "tx_type must be claim_tx".to_string(),
            ));
        }
        let claim_input = tx.inputs.first().ok_or_else(|| {
            ClaimTxError::StructureMalformed("claim tx inputs are empty".to_string())
        })?;
        Self::check_hex64(
            &claim_input.claim_id_hex,
            ClaimTxError::StructureMalformed("claim_id_hex invalid".to_string()),
        )?;
        Self::check_hex64(
            &tx.context.nullifier_hex,
            ClaimTxError::NullifierInvalidHex("nullifier_hex invalid".to_string()),
        )?;
        if tx.context.recipient_wallet_id.is_empty() {
            return Err(ClaimTxError::StructureMalformed(
                "recipient_wallet_id is empty".to_string(),
            ));
        }
        Self::check_hex64(
            &tx.context.recipient_owner_hex,
            ClaimTxError::StructureMalformed("recipient_owner_hex invalid".to_string()),
        )?;
        Self::check_hex64(
            &tx.context.claim_scope_hash_hex,
            ClaimTxError::StructureMalformed("claim_scope_hash_hex invalid".to_string()),
        )?;
        Self::check_hex64(
            &claim_input.claim_source_asset_id_hex,
            ClaimTxError::StructureMalformed("claim_source_asset_id_hex invalid".to_string()),
        )?;
        Self::check_hex64(
            &claim_input.claim_source_commitment_hex,
            ClaimTxError::StructureMalformed("claim_source_commitment_hex invalid".to_string()),
        )?;

        Ok(())
    }

    fn verify_tx_core(tx: &ClaimTxWire) -> Result<(), ClaimTxError> {
        // The outer wire tag must stay separate from the inner proof version.
        if tx.proof.proof_type != CLAIM_SOURCE_PROOF_TAG {
            return Err(ClaimTxError::ProofTypeInvalid(tx.proof.proof_type.clone()));
        }
        if tx.fee != 0 {
            return Err(ClaimTxError::FeeNonZero(tx.fee.to_string()));
        }

        Ok(())
    }

    fn verify_out_core(out: &ClaimOutputWire) -> Result<(), ClaimTxError> {
        if out.amount == 0 {
            return Err(ClaimTxError::OutputAmountZero("amount == 0".to_string()));
        }
        if out.asset_class.parse::<AssetClass>().is_err() {
            return Err(ClaimTxError::OutputAssetClassInvalid(
                out.asset_class.clone(),
            ));
        }

        Ok(())
    }

    fn verify_out_hexes(out: &ClaimOutputWire) -> Result<(), ClaimTxError> {
        Self::check_hex64(
            &out.nonce_hex,
            ClaimTxError::OutputNonceInvalidHex("nonce_hex invalid".to_string()),
        )?;
        if out.nonce_hex == "00".repeat(32) {
            return Err(ClaimTxError::OutputNonceIsZero(
                "nonce_hex is all zero".to_string(),
            ));
        }
        Self::check_hex64(
            &out.asset_id_hex,
            ClaimTxError::OutputAssetIdInvalidHex("asset_id_hex invalid".to_string()),
        )?;
        Self::check_hex64(
            &out.owner_binding_hex,
            ClaimTxError::OutputOwnerBindingInvalidHex("owner_binding_hex invalid".to_string()),
        )?;

        Ok(())
    }

    fn verify_out_uniq(
        out: &ClaimOutputWire,
        seen_nonces: &mut HashSet<String>,
        seen_asset_ids: &mut HashSet<String>,
    ) -> Result<(), ClaimTxError> {
        if !seen_nonces.insert(out.nonce_hex.clone()) {
            return Err(ClaimTxError::DuplicateNonce(out.nonce_hex.clone()));
        }
        if !seen_asset_ids.insert(out.asset_id_hex.clone()) {
            return Err(ClaimTxError::DuplicateAssetId(out.asset_id_hex.clone()));
        }

        Ok(())
    }

    fn verify_out_bind(
        out: &ClaimOutputWire,
        claim_id: &[u8; 32],
        output_index: u32,
        recipient_owner_hex: &str,
    ) -> Result<(), ClaimTxError> {
        if out.owner_binding_hex != recipient_owner_hex {
            return Err(ClaimTxError::OutputOwnerBindingMismatch(
                "owner_binding_hex must equal tx.context.recipient_owner_hex".to_string(),
            ));
        }

        let expect_nonce = hex::encode(derive_output_nonce(claim_id, output_index));
        if out.nonce_hex != expect_nonce {
            return Err(ClaimTxError::OutputNonceMismatch(
                "nonce_hex must equal derive_output_nonce(claim_id, output_index)".to_string(),
            ));
        }

        Ok(())
    }

    fn verify_out_leaf(
        out: &ClaimOutputWire,
        require_leaf: bool,
    ) -> Result<Option<&AssetPkgWire>, ClaimTxError> {
        let Some(asset_wire) = out.asset_wire.as_ref() else {
            if require_leaf {
                return Err(ClaimTxError::LeafRequired(
                    "claim package requires outputs[].asset_wire".to_string(),
                ));
            }
            return Ok(None);
        };

        let wire = asset_wire
            .clone()
            .to_wire()
            .map_err(|e| ClaimTxError::LeafInvalid(format!("to_wire failed: {e}")))?;

        let leaf = asset_wire_to_leaf(&wire)
            .map_err(|e| ClaimTxError::LeafInvalid(format!("jmt leaf conversion failed: {e}")))?;

        let expect_id = decode_hex32(&out.asset_id_hex, "asset_id_hex")?;
        if leaf.asset_id != expect_id {
            return Err(ClaimTxError::LeafMismatch(
                "asset_wire.asset_id != asset_id_hex".to_string(),
            ));
        }
        if asset_wire.amount != out.amount {
            return Err(ClaimTxError::LeafMismatch(
                "asset_wire.amount != output.amount".to_string(),
            ));
        }
        if asset_wire.definition.class.to_string().to_lowercase() != out.asset_class {
            return Err(ClaimTxError::LeafMismatch(
                "asset_wire.definition.class != asset_class".to_string(),
            ));
        }

        let asset = wire
            .to_asset()
            .map_err(|e| ClaimTxError::LeafInvalid(format!("to_asset failed: {e}")))?;
        asset
            .verify_complete()
            .map_err(|e| ClaimTxError::LeafInvalid(format!("verify_complete failed: {e}")))?;

        Ok(Some(asset_wire))
    }

    fn verify_recipient_card(
        tx: &ClaimTxWire,
        require_card: bool,
    ) -> Result<Option<ReceiverCard>, ClaimTxError> {
        let Some(card_hex) = tx.context.recipient_card_hex.as_ref() else {
            if require_card {
                return Err(ClaimTxError::RecipientCardRequired(
                    "claim package requires tx.context.recipient_card_hex".to_string(),
                ));
            }
            return Ok(None);
        };

        if card_hex.is_empty() || !is_even_hex(card_hex) {
            return Err(ClaimTxError::RecipientCardInvalid(
                "recipient_card_hex invalid".to_string(),
            ));
        }

        let bytes = hex::decode(card_hex).map_err(|_| {
            ClaimTxError::RecipientCardInvalid("recipient_card_hex decode failed".to_string())
        })?;
        let recipient_card = ReceiverCard::from_untrusted_bytes(&bytes).map_err(|e| {
            ClaimTxError::RecipientCardInvalid(format!("recipient card parse failed: {e}"))
        })?;
        recipient_card.verify().map_err(|e| {
            ClaimTxError::RecipientCardInvalid(format!("recipient card verify failed: {e}"))
        })?;

        if hex::encode(recipient_card.owner_handle) != tx.context.recipient_owner_hex {
            return Err(ClaimTxError::RecipientCardMismatch(
                "recipient card owner_handle != recipient_owner_hex".to_string(),
            ));
        }

        Ok(Some(recipient_card))
    }

    fn verify_out_attest(
        out: &ClaimOutputWire,
        leaf: Option<&AssetPkgWire>,
        require_leaf: bool,
        chain_id: u32,
        tx: &ClaimTxWire,
        output_index: u32,
        recipient_card: Option<&ReceiverCard>,
    ) -> Result<(), ClaimTxError> {
        let Some(leaf) = leaf else {
            return Ok(());
        };
        if !require_leaf {
            return Ok(());
        }

        let recipient_card = recipient_card.ok_or_else(|| {
            ClaimTxError::RecipientCardRequired(
                "portable leaf attestation requires verified recipient card".to_string(),
            )
        })?;
        let attest_hex = out.owner_attest_hex.as_ref().ok_or_else(|| {
            ClaimTxError::OwnerAttestRequired(
                "claim package requires outputs[].owner_attest_hex".to_string(),
            )
        })?;
        let sig = decode_sig_hex(attest_hex, "owner_attest_hex")?;
        let msg = build_owner_attest_msg(
            chain_id,
            &tx.inputs[0].claim_id_hex,
            &tx.context.recipient_wallet_id,
            &tx.context.recipient_owner_hex,
            &tx.context.claim_scope_hash_hex,
            output_index,
            leaf,
        )?;
        let identity_pk = Z00ZRistrettoPoint::try_from_bytes(recipient_card.identity_pk)
            .map_err(|e| {
                ClaimTxError::OwnerAttestInvalid(format!("recipient identity_pk invalid: {e}"))
            })?;

        verify_identity(&identity_pk, &msg, OWNER_ATTEST_CTX, &sig).map_err(|e| {
            ClaimTxError::OwnerAttestInvalid(format!("owner attestation verify failed: {e}"))
        })
    }

    fn verify_portable_leaf(
        tx: &ClaimTxWire,
        require_leaf: bool,
    ) -> Result<Vec<&AssetPkgWire>, ClaimTxError> {
        if tx.outputs.is_empty() {
            return Err(ClaimTxError::OutputsEmpty);
        }

        let claim_id = decode_hex32(&tx.inputs[0].claim_id_hex, "claim_id_hex")?;

        let mut seen_nonces: HashSet<String> = HashSet::new();
        let mut seen_asset_ids: HashSet<String> = HashSet::new();
        let mut leaves = Vec::with_capacity(tx.outputs.len());

        for (idx, out) in tx.outputs.iter().enumerate() {
            Self::verify_out_core(out)?;
            Self::verify_out_hexes(out)?;
            Self::verify_out_uniq(out, &mut seen_nonces, &mut seen_asset_ids)?;
            Self::verify_out_bind(out, &claim_id, idx as u32, &tx.context.recipient_owner_hex)?;
            let leaf = Self::verify_out_leaf(out, require_leaf)?;
            if let Some(leaf) = leaf {
                leaves.push(leaf);
            }
        }

        Ok(leaves)
    }
}

include!("claim_tx_verify_proof.rs");
