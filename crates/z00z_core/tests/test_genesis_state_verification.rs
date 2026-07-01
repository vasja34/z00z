//! Genesis State Hash Verification
//!
//! Verify that the canonical `configs/devnet_genesis_config.yaml` surface
//! produces deterministic commitments and a stable state hash.

use crate::genesis::helpers::create_test_observability;
use std::str::FromStr;
use std::sync::Arc;
use z00z_core::assets::AssetDefinition;
use z00z_core::config_paths::devnet_genesis_path;
use z00z_core::genesis::genesis_config::{load_genesis_config, GenesisConfig};
use z00z_core::genesis::validator::compute_genesis_state_hash;
use z00z_core::genesis::{
    create_asset_definition, generate_all_genesis_assets, generate_genesis_policies,
    generate_genesis_settlement_corpus, ChainType, GenesisSeed,
};
use z00z_utils::prelude::{NoopLogger, NoopMetrics};

fn load_config() -> GenesisConfig {
    let path = devnet_genesis_path();
    load_genesis_config(path.to_str().expect("utf8 path")).expect("Failed to parse genesis config")
}

fn with_serials(definition: &AssetDefinition, serials: u32) -> AssetDefinition {
    AssetDefinition::new(
        [0u8; 32],
        definition.class,
        definition.name.clone(),
        definition.symbol.clone(),
        definition.decimals,
        serials,
        definition.nominal,
        definition.domain_name.clone(),
        definition.version,
        definition.crypto_version,
        definition.policy_flags,
        definition.metadata.clone(),
    )
    .expect("reduced-serial test definition must remain valid")
}

fn assert_chain_type(config: &GenesisConfig) {
    assert_eq!(config.chain.chain_type, "devnet");
}

fn create_definition(config: &GenesisConfig, network_type: ChainType) -> AssetDefinition {
    create_asset_definition(
        &config.assets[0],
        &config.chain.domains.genesis_seed,
        network_type,
    )
    .expect("Failed to create asset definition")
}

fn build_state_hash(config: &GenesisConfig) -> [u8; 32] {
    let genesis_seed = GenesisSeed::from_config(config).expect("genesis seed");
    let network_type = ChainType::from_str(&config.chain.chain_type).expect("chain type");
    let definitions = config
        .assets
        .iter()
        .map(|asset| create_asset_definition(asset, genesis_seed.as_bytes(), network_type))
        .collect::<Result<Vec<_>, _>>()
        .expect("asset definitions");
    let policies =
        generate_genesis_policies(&config.assets, &config.policies).expect("genesis policies");
    let corpus = generate_genesis_settlement_corpus(
        &definitions,
        &config.rights,
        &config.vouchers,
        &policies,
        genesis_seed.as_bytes(),
        config.chain.id,
        network_type,
        Arc::new(NoopLogger),
        Arc::new(NoopMetrics),
    )
    .expect("genesis corpus");

    compute_genesis_state_hash(&corpus)
}

fn generate_small_genesis(
    definition: &AssetDefinition,
    genesis_seed: &[u8; 32],
    network_type: ChainType,
) -> z00z_core::genesis::GenesisAssetAccumulator {
    let (logger, metrics) = create_test_observability();

    generate_all_genesis_assets(
        &[with_serials(definition, 10)],
        genesis_seed,
        network_type,
        logger,
        metrics,
    )
    .expect("Genesis generation failed")
}

#[test]
fn test_canonical_genesis_state_is_deterministic() {
    let config = load_config();
    assert_chain_type(&config);
    assert_eq!(
        config.assets.len(),
        4,
        "canonical config must keep four assets"
    );
    assert_eq!(
        config.rights.len(),
        5,
        "canonical config must keep five rights"
    );

    let seed = config.chain.domains.genesis_seed;
    let definition = create_definition(&config, ChainType::Devnet);
    let first = generate_small_genesis(&definition, &seed, ChainType::Devnet);
    let second = generate_small_genesis(&definition, &seed, ChainType::Devnet);

    assert_eq!(
        first.coins[0].commitment, second.coins[0].commitment,
        "canonical config must produce deterministic commitments",
    );
    assert_eq!(
        build_state_hash(&config),
        build_state_hash(&config),
        "canonical config must produce a stable state hash",
    );
}
