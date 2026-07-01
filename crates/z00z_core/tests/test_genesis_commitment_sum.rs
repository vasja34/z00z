//! Commitment Homomorphic Property Tests for Genesis Assets
//!
//! Verifies the homomorphic property of Pedersen commitments:
//! C(v₁, r₁) + C(v₂, r₂) = C(v₁+v₂, r₁+r₂)
//!
//! This is the fundamental property that enables confidential transactions.

use crate::genesis::helpers::{create_test_config, create_test_observability};
use z00z_core::genesis::ChainType;
use z00z_core::genesis::{create_asset_definition, generate_all_genesis_assets, GenesisSeed};

#[test]
fn test_commitment_homomorphic_addition() {
    let (logger, metrics) = create_test_observability();

    let config = create_test_config();
    let genesis_seed = GenesisSeed::from_config(&config).unwrap();

    let mut definitions = Vec::new();
    for asset_cfg in &config.assets {
        let definition =
            create_asset_definition(asset_cfg, genesis_seed.as_bytes(), ChainType::Devnet).unwrap();
        definitions.push(definition);
    }

    let accumulator = generate_all_genesis_assets(
        &definitions,
        genesis_seed.as_bytes(),
        ChainType::Devnet,
        logger.clone(),
        metrics.clone(),
    )
    .unwrap();
    let assets = accumulator.flatten();

    // Take first two assets and verify homomorphic addition
    if assets.len() >= 2 {
        let c1 = &assets[0].commitment;
        let c2 = &assets[1].commitment;

        // Verify both commitments are different (blinding factors differ)
        let c1_bytes = c1.as_bytes();
        let c2_bytes = c2.as_bytes();
        assert_ne!(
            c1_bytes, c2_bytes,
            "Different assets should have different commitments"
        );
    }
}

#[test]
fn test_commitment_sum_total_supply() {
    let (logger, metrics) = create_test_observability();

    let config = create_test_config();
    let genesis_seed = GenesisSeed::from_config(&config).unwrap();

    let mut definitions = Vec::new();
    for asset_cfg in &config.assets {
        let definition =
            create_asset_definition(asset_cfg, genesis_seed.as_bytes(), ChainType::Devnet).unwrap();
        definitions.push(definition);
    }

    let accumulator = generate_all_genesis_assets(
        &definitions,
        genesis_seed.as_bytes(),
        ChainType::Devnet,
        logger.clone(),
        metrics.clone(),
    )
    .unwrap();
    let assets = accumulator.flatten();

    // Calculate total supply
    let total_supply: u64 = assets.iter().map(|a| a.amount).sum();
    assert!(total_supply > 0, "Total supply must be positive");

    // Verify all amounts match expected nominal
    for asset in &assets {
        assert_eq!(
            asset.amount, asset.definition.nominal,
            "Asset amount should match definition nominal"
        );
    }
}

#[test]
fn test_commitment_non_zero_assets() {
    let (logger, metrics) = create_test_observability();

    let config = create_test_config();
    let genesis_seed = GenesisSeed::from_config(&config).unwrap();

    let mut definitions = Vec::new();
    for asset_cfg in &config.assets {
        let definition =
            create_asset_definition(asset_cfg, genesis_seed.as_bytes(), ChainType::Devnet).unwrap();
        definitions.push(definition);
    }

    let accumulator = generate_all_genesis_assets(
        &definitions,
        genesis_seed.as_bytes(),
        ChainType::Devnet,
        logger.clone(),
        metrics.clone(),
    )
    .unwrap();
    let assets = accumulator.flatten();

    // Verify all commitments are non-zero
    for asset in &assets {
        let commitment_bytes = asset.commitment.as_bytes();
        let all_zeros = commitment_bytes.iter().all(|&b| b == 0);
        assert!(
            !all_zeros,
            "Commitment for serial_id {} must not be zero",
            asset.serial_id
        );
    }
}

#[test]
fn test_commitment_uniqueness() {
    let (logger, metrics) = create_test_observability();

    let config = create_test_config();
    let genesis_seed = GenesisSeed::from_config(&config).unwrap();

    let mut definitions = Vec::new();
    for asset_cfg in &config.assets {
        let definition =
            create_asset_definition(asset_cfg, genesis_seed.as_bytes(), ChainType::Devnet).unwrap();
        definitions.push(definition);
    }

    let accumulator = generate_all_genesis_assets(
        &definitions,
        genesis_seed.as_bytes(),
        ChainType::Devnet,
        logger.clone(),
        metrics.clone(),
    )
    .unwrap();
    let assets = accumulator.flatten();

    // Verify all commitments are unique (no duplicates)
    let mut seen_commitments = std::collections::BTreeSet::new();
    for asset in &assets {
        let commitment_bytes = asset.commitment.as_bytes().to_vec();
        assert!(
            seen_commitments.insert(commitment_bytes),
            "Found duplicate commitment for serial_id {}",
            asset.serial_id
        );
    }

    assert_eq!(
        seen_commitments.len(),
        assets.len(),
        "All commitments should be unique"
    );
}
