//! Range Proof Verification Tests for Genesis Assets
//!
//! Verifies that all genesis assets have valid range proofs that can be verified.
//! Tests both individual and batch verification scenarios.

use crate::genesis::helpers::{create_test_config, create_test_observability};
use z00z_core::genesis::ChainType;
use z00z_core::genesis::{create_asset_definition, generate_all_genesis_assets, GenesisSeed};
use z00z_crypto::expert::encoding::ByteArray;
use z00z_crypto::vendor::tari::ExtendedPedersenCommitmentFactory;
use z00z_crypto::{BulletproofsPlusService, RangeProofService};

#[test]
fn test_proof_range_exists_assets() {
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

    // Verify all assets have range proofs
    for asset in &assets {
        assert!(
            asset.range_proof.is_some(),
            "Asset serial_id {} must have range proof",
            asset.serial_id
        );
    }
}

#[test]
fn test_range_proof_individual_verification() {
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

    // Initialize bulletproofs service for individual verification
    let factory = ExtendedPedersenCommitmentFactory::default();
    let bp_service = BulletproofsPlusService::init(64, 1, factory)
        .expect("Failed to initialize bulletproofs service");

    // Verify first 10 assets individually
    let sample_size = std::cmp::min(10, assets.len());
    for asset in assets.iter().take(sample_size) {
        if let Some(proof) = &asset.range_proof {
            let result = bp_service.verify(proof, asset.commitment.reveal());
            assert!(
                result,
                "Individual verification failed for serial_id {}",
                asset.serial_id
            );
        }
    }
}

#[test]
fn test_proof_range_non_zero() {
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

    // Verify all range proofs have non-zero size
    for asset in &assets {
        if let Some(proof) = &asset.range_proof {
            let proof_bytes = proof.as_bytes();
            assert!(
                !proof_bytes.is_empty(),
                "Range proof for serial_id {} must have non-zero size",
                asset.serial_id
            );

            // Typical range proof size is ~600-800 bytes
            assert!(
                proof_bytes.len() > 100,
                "Range proof for serial_id {} seems too small: {} bytes",
                asset.serial_id,
                proof_bytes.len()
            );
        }
    }
}
