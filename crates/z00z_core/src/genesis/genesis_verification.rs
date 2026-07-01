use super::{
    Asset, ChainType, DomainHasher, GenesisAssetAccumulator, GenesisConfig, GenesisError,
    GenesisStateHashDomain, VoucherBackingReferenceV1,
};

/// Verify all range proofs in genesis assets using batch verification.
pub fn verify_genesis_assets(assets: &[Asset]) -> Result<(), GenesisError> {
    validate_genesis_commitments_batch(assets).map_err(|e| GenesisError::ProofVerificationFailed {
        asset_id: [0u8; 32],
        serial_id: 0,
        error: format!("Batch verification failed: {}", e),
    })
}

/// Batch range proof validation using Bulletproofs+ batch verification.
pub fn validate_genesis_commitments_batch(assets: &[Asset]) -> Result<(), GenesisError> {
    for (idx, asset) in assets.iter().enumerate() {
        if asset.range_proof.is_none() {
            return Err(GenesisError::ProofVerificationFailed {
                asset_id: asset.asset_id(),
                serial_id: asset.serial_id,
                error: format!("Asset index {} missing range proof", idx),
            });
        }
    }

    if assets.is_empty() {
        return Ok(());
    }

    let commitments: Vec<_> = assets.iter().map(|a| &a.commitment).collect();
    let proofs: Vec<_> = assets
        .iter()
        .filter_map(|a| a.range_proof.as_ref())
        .collect();
    let minimum_value_promises = vec![z00z_crypto::MIN_VALUE_PROMISE; proofs.len()];
    z00z_crypto::batch_verify_range_proofs(
        &proofs,
        &commitments,
        z00z_crypto::RANGE_PROOF_BITS,
        z00z_crypto::AGGREGATION_FACTOR,
        &minimum_value_promises,
    )
    .map_err(|e| GenesisError::ProofVerificationFailed {
        asset_id: [0u8; 32],
        serial_id: 0,
        error: format!("Batch verification failed: {}", e),
    })
}

/// Accumulator for validation results.
#[derive(Debug, Default, serde::Serialize, serde::Deserialize)]
pub struct ValidationReport {
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
    pub total_validated: usize,
}

impl ValidationReport {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_error(&mut self, error: String) {
        self.errors.push(error);
    }

    pub fn add_warning(&mut self, warning: String) {
        self.warnings.push(warning);
    }

    pub fn merge(&mut self, other: ValidationReport) {
        self.errors.extend(other.errors);
        self.warnings.extend(other.warnings);
        self.total_validated += other.total_validated;
    }

    pub fn is_valid(&self) -> bool {
        self.errors.is_empty()
    }
}

/// Detect chain type from genesis configuration.
pub fn detect_chain_type(config: &GenesisConfig) -> Option<ChainType> {
    config.chain.chain_type.parse().ok()
}

fn expected_genesis_hash(network_type: ChainType) -> Result<Option<[u8; 32]>, GenesisError> {
    const MAINNET_GENESIS_STATE_HASH: Option<[u8; 32]> = None;
    const TESTNET_GENESIS_STATE_HASH: Option<[u8; 32]> = None;

    match network_type {
        ChainType::Mainnet => MAINNET_GENESIS_STATE_HASH
            .ok_or_else(|| GenesisError::MissingGenesisAnchor {
                network: network_type.to_string(),
            })
            .map(Some),
        ChainType::Testnet => TESTNET_GENESIS_STATE_HASH
            .ok_or_else(|| GenesisError::MissingGenesisAnchor {
                network: network_type.to_string(),
            })
            .map(Some),
        ChainType::Devnet => Ok(None),
    }
}

/// Compute cryptographic hash of complete genesis state.
pub fn compute_genesis_state_hash(accumulator: &GenesisAssetAccumulator) -> [u8; 32] {
    let mut hasher = DomainHasher::<GenesisStateHashDomain>::new_with_label("state");

    for asset in &accumulator.coins {
        hasher.update(asset.commitment.as_bytes());
        hasher.update(asset.serial_id.to_le_bytes());
    }
    for asset in &accumulator.tokens {
        hasher.update(asset.commitment.as_bytes());
        hasher.update(asset.serial_id.to_le_bytes());
    }
    for asset in &accumulator.nfts {
        hasher.update(asset.commitment.as_bytes());
        hasher.update(asset.serial_id.to_le_bytes());
    }
    for asset in &accumulator.voids {
        hasher.update(asset.commitment.as_bytes());
        hasher.update(asset.serial_id.to_le_bytes());
    }
    for right in &accumulator.rights {
        hasher.update(right.right_id.as_bytes());
        hasher.update(right.right_index.to_le_bytes());
        hasher.update(right.definition_id);
        hasher.update(right.serial_id.to_le_bytes());
        hasher.update(right.domain_name.as_bytes());
        hasher.update(right.metadata_purpose.as_bytes());
        hasher.update([right.leaf.version]);
        hasher.update(right.leaf.terminal_id);
        hasher.update(right.leaf.right_class.as_str().as_bytes());
        hasher.update(right.leaf.issuer_scope);
        hasher.update(right.leaf.provider_scope);
        hasher.update(right.leaf.holder_commitment);
        hasher.update(right.leaf.control_commitment);
        hasher.update(right.leaf.beneficiary_commitment);
        hasher.update(right.leaf.payload_commitment);
        hasher.update(right.leaf.valid_from.to_le_bytes());
        hasher.update(right.leaf.valid_until.to_le_bytes());
        hasher.update(right.leaf.challenge_from.to_le_bytes());
        hasher.update(right.leaf.challenge_until.to_le_bytes());
        hasher.update(right.leaf.use_nonce);
        hasher.update(right.leaf.revocation_policy_id);
        hasher.update(right.leaf.transition_policy_id);
        hasher.update(right.leaf.challenge_policy_id);
        hasher.update(right.leaf.disclosure_policy_id);
        hasher.update(right.leaf.retention_policy_id);
    }
    for voucher in &accumulator.vouchers {
        hasher.update(voucher.voucher_index.to_le_bytes());
        hasher.update(voucher.root_generation.to_le_bytes());
        hasher.update(voucher.terminal_id);
        hasher.update(voucher.issuer_commitment);
        hasher.update(voucher.holder_commitment);
        hasher.update(voucher.beneficiary_commitment);
        hasher.update(voucher.refund_target_commitment);
        hasher.update(voucher.config.id.as_bytes());
        hasher.update(voucher.config.domain_name.as_bytes());
        hasher.update(voucher.config.issuer_fixture.as_bytes());
        hasher.update(voucher.config.holder_fixture.as_bytes());
        hasher.update(voucher.config.beneficiary_fixture.as_bytes());
        match &voucher.config.backing {
            VoucherBackingReferenceV1::ReserveCommitment(bytes) => {
                hasher.update(b"reserve_commitment");
                hasher.update(*bytes);
            }
            VoucherBackingReferenceV1::ConsumedAsset {
                definition_id,
                serial_id,
            } => {
                hasher.update(b"consumed_asset");
                hasher.update(*definition_id);
                hasher.update(serial_id.to_le_bytes());
            }
            VoucherBackingReferenceV1::GenesisReserve { reserve_id } => {
                hasher.update(b"genesis_reserve");
                hasher.update(reserve_id.as_bytes());
            }
        }
        hasher.update(voucher.config.face_value.to_le_bytes());
        hasher.update(voucher.config.remaining_value.to_le_bytes());
        hasher.update(voucher.config.policy_id.bytes());
        hasher.update(voucher.config.action_pool_id.bytes());
        hasher.update([voucher.config.lifecycle as u8]);
        hasher.update(voucher.config.validity.valid_from.to_le_bytes());
        hasher.update(voucher.config.validity.valid_until.to_le_bytes());
        hasher.update([u8::from(voucher.config.acceptance.receiver_must_accept)]);
        hasher.update([u8::from(voucher.config.acceptance.allow_reject)]);
        hasher.update(voucher.config.acceptance.refund_target_fixture.as_bytes());
        hasher.update(voucher.config.replay_nonce);
        if let Some(bytes) = voucher.config.disclosure_commitment {
            hasher.update([1]);
            hasher.update(bytes);
        } else {
            hasher.update([0]);
        }
        if let Some(bytes) = voucher.config.audit_commitment {
            hasher.update([1]);
            hasher.update(bytes);
        } else {
            hasher.update([0]);
        }
    }

    let hash = hasher.finalize();
    let mut result = [0u8; 32];
    result.copy_from_slice(&hash.as_ref()[..32]);
    result
}

/// Verify genesis state hash matches consensus parameters.
pub fn verify_genesis_consensus(
    network_type: ChainType,
    computed_hash: &[u8; 32],
) -> Result<(), GenesisError> {
    if let Some(expected) = expected_genesis_hash(network_type)? {
        if computed_hash != &expected {
            return Err(GenesisError::GenesisStateMismatch {
                expected,
                computed: *computed_hash,
                network: format!("{:?}", network_type),
            });
        }
    }

    Ok(())
}
