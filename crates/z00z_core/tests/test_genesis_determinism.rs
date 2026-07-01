//! Determinism Tests for Genesis Module
//!
//! Verifies that genesis state construction is deterministic:
//! - Building twice with same seed produces identical assets
//! - Commitments match exactly
//! - Nonces match exactly
//! - Serial IDs and amounts match
//!
//! This complements reproducibility.rs with additional edge case testing.

use crate::genesis::helpers::{create_test_config, create_test_observability};
use z00z_core::genesis::ChainType;
use z00z_core::genesis::{create_asset_definition, generate_all_genesis_assets, GenesisSeed};

#[test]
fn test_determinism_single_asset() {
    let (logger, metrics) = create_test_observability();

    let config = create_test_config();
    let genesis_seed = GenesisSeed::from_config(&config).unwrap();

    // Create single asset definition
    let mut definitions = Vec::new();
    if let Some(asset_cfg) = config.assets.first() {
        let definition =
            create_asset_definition(asset_cfg, genesis_seed.as_bytes(), ChainType::Devnet).unwrap();
        definitions.push(definition);
    }

    // Generate twice
    let acc1 = generate_all_genesis_assets(
        &definitions,
        genesis_seed.as_bytes(),
        ChainType::Devnet,
        logger.clone(),
        metrics.clone(),
    )
    .unwrap();
    let acc2 = generate_all_genesis_assets(
        &definitions,
        genesis_seed.as_bytes(),
        ChainType::Devnet,
        logger.clone(),
        metrics.clone(),
    )
    .unwrap();

    let assets1 = acc1.flatten();
    let assets2 = acc2.flatten();

    assert_eq!(assets1.len(), assets2.len());

    for (a1, a2) in assets1.iter().zip(assets2.iter()) {
        assert_eq!(a1.serial_id, a2.serial_id);
        assert_eq!(a1.amount, a2.amount);
        assert_eq!(a1.commitment.as_bytes(), a2.commitment.as_bytes());
        assert_eq!(a1.nonce, a2.nonce);
    }
}

#[test]
fn test_determinism_commitment_consistency() {
    let (logger, metrics) = create_test_observability();

    let config = create_test_config();
    let genesis_seed = GenesisSeed::from_config(&config).unwrap();

    let mut definitions = Vec::new();
    for asset_cfg in &config.assets {
        let definition =
            create_asset_definition(asset_cfg, genesis_seed.as_bytes(), ChainType::Devnet).unwrap();
        definitions.push(definition);
    }

    let acc = generate_all_genesis_assets(
        &definitions,
        genesis_seed.as_bytes(),
        ChainType::Devnet,
        logger.clone(),
        metrics.clone(),
    )
    .unwrap();
    let assets = acc.flatten();

    // Verify no duplicate commitments (all unique)
    let mut commitments = std::collections::BTreeSet::new();
    for asset in &assets {
        let commitment_bytes = asset.commitment.as_bytes().to_vec();
        assert!(
            commitments.insert(commitment_bytes),
            "Found duplicate commitment for serial_id: {}",
            asset.serial_id
        );
    }
}
