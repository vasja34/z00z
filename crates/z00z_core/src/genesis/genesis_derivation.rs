use super::*;

/// Derive deterministic blinding factor for genesis asset.
pub fn derive_genesis_blinding(
    genesis_seed: &[u8; 32],
    definition_id: &[u8; 32],
    serial_id: u32,
    network_type: ChainType,
) -> Result<Z00ZScalar, GenesisError> {
    let mut input = Vec::with_capacity(32 + 32 + 4);
    input.extend_from_slice(genesis_seed);
    input.extend_from_slice(definition_id);
    input.extend_from_slice(&serial_id.to_le_bytes());

    let hash_output = match network_type {
        ChainType::Devnet => {
            DomainHasher::<GenesisBlindingDomainDevnet>::new_with_label("genesis_blinding")
                .chain(&input)
                .finalize()
        }
        ChainType::Testnet => {
            DomainHasher::<GenesisBlindingDomainTestnet>::new_with_label("genesis_blinding")
                .chain(&input)
                .finalize()
        }
        ChainType::Mainnet => {
            DomainHasher::<GenesisBlindingDomainMainnet>::new_with_label("genesis_blinding")
                .chain(&input)
                .finalize()
        }
    };

    let mut hash_bytes = [0u8; 64];
    hash_bytes.copy_from_slice(hash_output.as_ref());

    Z00ZScalar::from_uniform_bytes(&hash_bytes).map_err(|e| {
        GenesisError::BlindingDerivationFailed {
            definition_id: *definition_id,
            serial_id,
            error: format!("from_uniform_bytes failed: {}", e),
        }
    })
}

/// Derive deterministic RNG seed for genesis asset.
pub fn derive_deterministic_rng_seed(
    genesis_seed: &[u8; 32],
    definition_id: &[u8; 32],
    serial_id: u32,
    network_type: ChainType,
) -> [u8; 32] {
    let mut input = Vec::with_capacity(32 + 32 + 4);
    input.extend_from_slice(genesis_seed);
    input.extend_from_slice(definition_id);
    input.extend_from_slice(&serial_id.to_le_bytes());

    let hash_output = match network_type {
        ChainType::Devnet => DomainHasher::<GenesisRngSeedDomainDevnet>::new_with_label("rng_seed")
            .chain(&input)
            .finalize(),
        ChainType::Testnet => {
            DomainHasher::<GenesisRngSeedDomainTestnet>::new_with_label("rng_seed")
                .chain(&input)
                .finalize()
        }
        ChainType::Mainnet => {
            DomainHasher::<GenesisRngSeedDomainMainnet>::new_with_label("rng_seed")
                .chain(&input)
                .finalize()
        }
    };

    let mut result = [0u8; 32];
    result.copy_from_slice(&hash_output.as_ref()[..32]);
    result
}

pub(super) fn create_prechecked_asset_definition(
    cfg: &AssetConfigEntry,
    _genesis_seed: &[u8; 32],
    _network_type: ChainType,
) -> Result<AssetDefinition, GenesisError> {
    AssetDefinition::new(
        [0u8; 32],
        cfg.class,
        cfg.name.clone(),
        cfg.symbol.clone(),
        cfg.policy.decimals,
        cfg.policy.serials,
        cfg.policy.nominal,
        cfg.domain_name.clone(),
        1,
        1,
        cfg.policy.asset_flags(cfg.class),
        cfg.metadata.clone(),
    )
    .map_err(|e| GenesisError::AssetCreationFailed {
        definition_id: [0u8; 32],
        serial_id: 0,
        error: e.to_string(),
    })
}

/// Create AssetDefinition from configuration entry using genesis-specific network-aware identity.
pub fn create_asset_definition(
    cfg: &AssetConfigEntry,
    genesis_seed: &[u8; 32],
    network_type: ChainType,
) -> Result<AssetDefinition, GenesisError> {
    create_prechecked_asset_definition(cfg, genesis_seed, network_type)
}

#[cfg(feature = "deterministic-rng")]
fn generate_assets_prechecked(
    definition_arc: Arc<AssetDefinition>,
    genesis_seed: &[u8; 32],
    network_type: ChainType,
) -> Result<Vec<Asset>, GenesisError> {
    let assets: Vec<Asset> = (0..definition_arc.serials)
        .into_par_iter()
        .map(|serial_id| -> Result<Asset, GenesisError> {
            let amount = definition_arc.nominal;
            let blinding =
                derive_genesis_blinding(genesis_seed, &definition_arc.id, serial_id, network_type)?;
            let nonce = derive_genesis_nonce(genesis_seed, &definition_arc.id, serial_id);
            let rng_seed = derive_deterministic_rng_seed(
                genesis_seed,
                &definition_arc.id,
                serial_id,
                network_type,
            );
            let provider = DeterministicRngProvider::from_seed(rng_seed);
            let mut rng = provider.rng();

            Asset::new_prechecked(
                Arc::clone(&definition_arc),
                serial_id,
                amount,
                &blinding,
                nonce,
                &mut rng,
                false,
            )
            .map_err(|e| GenesisError::AssetCreationFailed {
                definition_id: definition_arc.id,
                serial_id,
                error: e.to_string(),
            })
        })
        .collect::<Result<Vec<_>, _>>()?;

    Ok(assets)
}

#[cfg(not(feature = "deterministic-rng"))]
fn generate_assets_prechecked(
    definition_arc: Arc<AssetDefinition>,
    genesis_seed: &[u8; 32],
    network_type: ChainType,
) -> Result<Vec<Asset>, GenesisError> {
    let _ = (definition_arc, genesis_seed, network_type);
    Err(GenesisError::InvalidConfig(
        "z00z_core deterministic-rng feature is required for genesis asset generation".to_string(),
    ))
}

#[cfg(feature = "deterministic-rng")]
pub(super) fn generate_assets_checked(
    definition_arc: Arc<AssetDefinition>,
    genesis_seed: &[u8; 32],
    network_type: ChainType,
) -> Result<Vec<Asset>, GenesisError> {
    definition_arc
        .validate()
        .map_err(|e| GenesisError::AssetCreationFailed {
            definition_id: definition_arc.id,
            serial_id: 0,
            error: e.to_string(),
        })?;

    let assets: Vec<Asset> = (0..definition_arc.serials)
        .into_par_iter()
        .map(|serial_id| -> Result<Asset, GenesisError> {
            let amount = definition_arc.nominal;
            let blinding =
                derive_genesis_blinding(genesis_seed, &definition_arc.id, serial_id, network_type)?;
            let nonce = derive_genesis_nonce(genesis_seed, &definition_arc.id, serial_id);
            let rng_seed = derive_deterministic_rng_seed(
                genesis_seed,
                &definition_arc.id,
                serial_id,
                network_type,
            );
            let provider = DeterministicRngProvider::from_seed(rng_seed);
            let mut rng = provider.rng();
            Asset::new(
                Arc::clone(&definition_arc),
                serial_id,
                amount,
                &blinding,
                nonce,
                &mut rng,
            )
            .map_err(|e| GenesisError::AssetCreationFailed {
                definition_id: definition_arc.id,
                serial_id,
                error: e.to_string(),
            })
        })
        .collect::<Result<Vec<_>, _>>()?;

    Ok(assets)
}

#[cfg(not(feature = "deterministic-rng"))]
pub(super) fn generate_assets_checked(
    definition_arc: Arc<AssetDefinition>,
    genesis_seed: &[u8; 32],
    network_type: ChainType,
) -> Result<Vec<Asset>, GenesisError> {
    let _ = (definition_arc, genesis_seed, network_type);
    Err(GenesisError::InvalidConfig(
        "z00z_core deterministic-rng feature is required for genesis asset generation".to_string(),
    ))
}

fn generate_all_genesis_assets_prechecked(
    definitions: &[AssetDefinition],
    genesis_seed: &[u8; 32],
    network_type: ChainType,
    logger: Arc<dyn Logger>,
    metrics: Arc<dyn MetricsSink>,
) -> Result<GenesisAssetAccumulator, GenesisError> {
    let start_time = Instant::now();
    let total_expected: usize = definitions.iter().map(|d| d.serials as usize).sum();

    logger.info(&format!(
        "genesis_start: network={}, definitions={}, expected_assets={}",
        network_type.as_str(),
        definitions.len(),
        total_expected
    ));

    let all_assets_by_class: Result<Vec<Vec<Asset>>, GenesisError> = definitions
        .par_iter()
        .enumerate()
        .map(|(idx, definition)| {
            let def_start = Instant::now();
            logger.debug(&format!(
                "genesis_definition_start: index={}/{}, symbol={}, serials={}, nominal={}",
                idx + 1,
                definitions.len(),
                definition.symbol,
                definition.serials,
                definition.nominal
            ));

            let definition_arc = Arc::new(definition.clone());
            let result = generate_assets_prechecked(definition_arc, genesis_seed, network_type);

            if let Ok(ref assets) = result {
                let duration = def_start.elapsed();
                logger.info(&format!(
                    "genesis_definition_complete: index={}/{}, symbol={}, assets={}, duration_secs={:.2}, rate_per_sec={:.0}",
                    idx + 1,
                    definitions.len(),
                    definition.symbol,
                    assets.len(),
                    duration.as_secs_f64(),
                    assets.len() as f64 / duration.as_secs_f64()
                ));
            }

            result
        })
        .collect();

    let all_assets = all_assets_by_class?;
    let mut accumulator = GenesisAssetAccumulator::new();
    for (definition, assets) in definitions.iter().zip(all_assets) {
        for asset in assets {
            accumulator.push(asset, definition.class);
        }
    }

    let total_duration = start_time.elapsed();
    let total_generated = accumulator.total_count();

    logger.info(&format!(
        "genesis_complete: total_assets={}, duration_secs={:.2}, rate_per_sec={:.0}, coins={}, tokens={}, nfts={}, voids={}",
        total_generated,
        total_duration.as_secs_f64(),
        total_generated as f64 / total_duration.as_secs_f64(),
        accumulator.coins.len(),
        accumulator.tokens.len(),
        accumulator.nfts.len(),
        accumulator.voids.len()
    ));

    metrics.set_gauge("genesis_total_assets", total_generated as f64);
    metrics.observe_histogram(
        "genesis_total_duration_ms",
        total_duration.as_millis() as f64,
    );

    Ok(accumulator)
}

pub(crate) fn generate_genesis_settlement_corpus_prechecked(
    definitions: &[AssetDefinition],
    rights: &[RightsConfigEntry],
    vouchers: &[crate::vouchers::VoucherBootstrapEntryV1],
    policies: &[GenesisPolicyRecord],
    genesis_seed: &[u8; 32],
    chain_id: u32,
    network_type: ChainType,
    logger: Arc<dyn Logger>,
    metrics: Arc<dyn MetricsSink>,
) -> Result<GenesisSettlementCorpus, GenesisError> {
    let mut corpus = generate_all_genesis_assets_prechecked(
        definitions,
        genesis_seed,
        network_type,
        logger,
        metrics,
    )?;
    let policy_lookup = policy_lookup(policies)?;
    corpus.rights = generate_genesis_rights_with_policies(
        rights,
        &policy_lookup,
        genesis_seed,
        chain_id,
        network_type,
        GENESIS_ROOT_GENERATION,
    )?;
    corpus.vouchers = generate_genesis_vouchers(
        vouchers,
        &policy_lookup,
        genesis_seed,
        chain_id,
        network_type,
        GENESIS_ROOT_GENERATION,
    )?;
    ensure_terminal_collision_free(&corpus)?;
    Ok(corpus)
}

/// Generate all genesis assets with nested parallelism.
pub fn generate_all_genesis_assets(
    definitions: &[AssetDefinition],
    genesis_seed: &[u8; 32],
    network_type: ChainType,
    logger: Arc<dyn Logger>,
    metrics: Arc<dyn MetricsSink>,
) -> Result<GenesisAssetAccumulator, GenesisError> {
    let start_time = Instant::now();
    let total_expected: usize = definitions.iter().map(|d| d.serials as usize).sum();

    logger.info(&format!(
        "genesis_start: network={}, definitions={}, expected_assets={}",
        network_type.as_str(),
        definitions.len(),
        total_expected
    ));

    let all_assets_by_class: Result<Vec<Vec<Asset>>, GenesisError> = definitions
        .par_iter()
        .enumerate()
        .map(|(idx, definition)| {
            let def_start = Instant::now();
            logger.debug(&format!(
                "genesis_definition_start: index={}/{}, symbol={}, serials={}, nominal={}",
                idx + 1,
                definitions.len(),
                definition.symbol,
                definition.serials,
                definition.nominal
            ));

            let definition_arc = Arc::new(definition.clone());
            let result = generate_assets_checked(definition_arc, genesis_seed, network_type);

            if let Ok(ref assets) = result {
                let duration = def_start.elapsed();
                logger.info(&format!(
                    "genesis_definition_complete: index={}/{}, symbol={}, assets={}, duration_secs={:.2}, rate_per_sec={:.0}",
                    idx + 1,
                    definitions.len(),
                    definition.symbol,
                    assets.len(),
                    duration.as_secs_f64(),
                    assets.len() as f64 / duration.as_secs_f64()
                ));

                metrics.inc_counter("genesis_assets_generated", assets.len() as u64);
                metrics.observe_histogram(
                    "genesis_definition_duration_ms",
                    duration.as_millis() as f64,
                );
            }

            result
        })
        .collect();

    let all_assets = all_assets_by_class?;
    let mut accumulator = GenesisAssetAccumulator::new();
    for (definition, assets) in definitions.iter().zip(all_assets) {
        for asset in assets {
            accumulator.push(asset, definition.class);
        }
    }

    let total_duration = start_time.elapsed();
    let total_generated = accumulator.total_count();

    logger.info(&format!(
        "genesis_complete: total_assets={}, duration_secs={:.2}, rate_per_sec={:.0}, coins={}, tokens={}, nfts={}, voids={}",
        total_generated,
        total_duration.as_secs_f64(),
        total_generated as f64 / total_duration.as_secs_f64(),
        accumulator.coins.len(),
        accumulator.tokens.len(),
        accumulator.nfts.len(),
        accumulator.voids.len()
    ));

    metrics.set_gauge("genesis_total_assets", total_generated as f64);
    metrics.observe_histogram(
        "genesis_total_duration_ms",
        total_duration.as_millis() as f64,
    );

    Ok(accumulator)
}

pub fn generate_genesis_settlement_corpus(
    definitions: &[AssetDefinition],
    rights: &[RightsConfigEntry],
    vouchers: &[crate::vouchers::VoucherBootstrapEntryV1],
    policies: &[GenesisPolicyRecord],
    genesis_seed: &[u8; 32],
    chain_id: u32,
    network_type: ChainType,
    logger: Arc<dyn Logger>,
    metrics: Arc<dyn MetricsSink>,
) -> Result<GenesisSettlementCorpus, GenesisError> {
    let mut corpus =
        generate_all_genesis_assets(definitions, genesis_seed, network_type, logger, metrics)?;
    let policy_lookup = policy_lookup(policies)?;
    corpus.rights = generate_genesis_rights_with_policies(
        rights,
        &policy_lookup,
        genesis_seed,
        chain_id,
        network_type,
        GENESIS_ROOT_GENERATION,
    )?;
    corpus.vouchers = generate_genesis_vouchers(
        vouchers,
        &policy_lookup,
        genesis_seed,
        chain_id,
        network_type,
        GENESIS_ROOT_GENERATION,
    )?;
    ensure_terminal_collision_free(&corpus)?;
    Ok(corpus)
}
