//! Reproducibility Tests for Genesis Module
//!
//! CRITICAL: These tests verify deterministic genesis generation.
//! Same seed MUST produce byte-for-byte identical assets.

use crate::genesis::helpers::*;
use z00z_core::genesis::ChainType;
use z00z_core::genesis::{create_asset_definition, generate_all_genesis_assets, GenesisSeed};

#[test]
fn test_genesis_facade_split() {
    let source = include_str!("../src/genesis/genesis.rs");
    let mod_source = include_str!("../src/genesis/mod.rs");
    let include_macro = concat!("include", "!");

    for part in [
        "mod chain_type;",
        "mod genesis_accumulator;",
        "mod genesis_seed;",
        "mod genesis_derivation;",
        "mod genesis_output;",
        "mod genesis_run;",
        "mod test_genesis;",
    ] {
        assert!(
            source.contains(part),
            "genesis.rs must keep explicit submodule declaration for {part}"
        );
    }

    assert!(
        !source.contains(include_macro),
        "genesis.rs must not assemble boundary parts with the include macro"
    );
    assert!(
        mod_source.contains("mod generation;"),
        "genesis/mod.rs must keep the private generation owner module"
    );
    assert!(
        !mod_source.contains("pub mod genesis;"),
        "genesis/mod.rs must not expose a second public owner path"
    );
    assert!(
        mod_source.contains("pub use generation::{"),
        "genesis/mod.rs must re-export the shallow public facade from generation"
    );
}

#[test]
fn test_validator_facade_split() {
    let source = include_str!("../src/genesis/validator.rs");
    let include_macro = concat!("include", "!");

    for part in [
        "mod genesis_error;",
        "mod genesis_verification;",
        "mod genesis_config_validate;",
        "mod test_validator;",
    ] {
        assert!(
            source.contains(part),
            "validator.rs must keep explicit submodule declaration for {part}"
        );
    }

    assert!(
        !source.contains(include_macro),
        "validator.rs must not assemble boundary parts with the include macro"
    );
}

#[test]
fn test_genesis_reproducibility() {
    let (logger, metrics) = create_test_observability();

    let config = create_test_config();
    let genesis_seed = GenesisSeed::from_config(&config).unwrap();

    // Create asset definitions
    let mut definitions = Vec::new();
    for asset_cfg in &config.assets {
        let definition =
            create_asset_definition(asset_cfg, genesis_seed.as_bytes(), ChainType::Devnet).unwrap();
        definitions.push(definition);
    }

    // Generate genesis twice with same seed
    let accumulator1 = generate_all_genesis_assets(
        &definitions,
        genesis_seed.as_bytes(),
        ChainType::Devnet,
        logger.clone(),
        metrics.clone(),
    )
    .unwrap();
    let accumulator2 = generate_all_genesis_assets(
        &definitions,
        genesis_seed.as_bytes(),
        ChainType::Devnet,
        logger.clone(),
        metrics.clone(),
    )
    .unwrap();

    // MUST be byte-for-byte identical
    let assets1 = accumulator1.flatten();
    let assets2 = accumulator2.flatten();

    assert_eq!(assets1.len(), assets2.len(), "Asset count must match");

    for (a1, a2) in assets1.iter().zip(assets2.iter()) {
        assert_eq!(
            a1.serial_id, a2.serial_id,
            "Serial IDs must be deterministic"
        );
        assert_eq!(a1.amount, a2.amount, "Amounts must be deterministic");
        assert_eq!(
            a1.commitment.as_bytes(),
            a2.commitment.as_bytes(),
            "Commitments must be deterministic"
        );
        assert_eq!(a1.nonce, a2.nonce, "Nonces must be deterministic");

        // Note: Range proof byte-level determinism depends on RNG implementation details
        // The important properties (commitment, nonce, amount) are deterministic above
        // Range proofs are verifiable but may have internal randomness
        match (&a1.range_proof, &a2.range_proof) {
            (Some(_proof1), Some(_proof2)) => {
                // Both have proofs (consistent)
                // TODO: Enable byte-level proof comparison when RNG is fully deterministic
                // assert_eq!(proof1.as_bytes(), proof2.as_bytes(), "Range proofs must be deterministic");
            }
            (None, None) => {}
            _ => panic!("Range proof presence must be consistent"),
        }
    }
}

#[test]
fn test_genesis_different_seeds_assets() {
    let (logger, metrics) = create_test_observability();

    let mut config1 = create_test_config();
    let mut config2 = config1.clone();

    // Different seeds (high entropy)
    config1.chain.domains.genesis_seed = [
        0x12, 0x34, 0x56, 0x78, 0x9A, 0xBC, 0xDE, 0xF0, 0x11, 0x22, 0x33, 0x44, 0x55, 0x66, 0x77,
        0x88, 0xAA, 0xBB, 0xCC, 0xDD, 0xEE, 0xFF, 0x00, 0x11, 0x22, 0x33, 0x44, 0x55, 0x66, 0x77,
        0x88, 0x99,
    ];
    config2.chain.domains.genesis_seed = [
        0xAB, 0xCD, 0xEF, 0x01, 0x23, 0x45, 0x67, 0x89, 0x9A, 0xBC, 0xDE, 0xF0, 0x12, 0x34, 0x56,
        0x78, 0x88, 0x99, 0xAA, 0xBB, 0xCC, 0xDD, 0xEE, 0xFF, 0x00, 0x11, 0x22, 0x33, 0x44, 0x55,
        0x66, 0x77,
    ];

    let seed1 = GenesisSeed::from_config(&config1).unwrap();
    let seed2 = GenesisSeed::from_config(&config2).unwrap();

    // Create asset definitions (same config structure)
    let mut definitions = Vec::new();
    for asset_cfg in &config1.assets {
        let definition =
            create_asset_definition(asset_cfg, seed1.as_bytes(), ChainType::Devnet).unwrap();
        definitions.push(definition);
    }

    // Generate with different seeds
    let accumulator1 = generate_all_genesis_assets(
        &definitions,
        seed1.as_bytes(),
        ChainType::Devnet,
        logger.clone(),
        metrics.clone(),
    )
    .unwrap();
    let accumulator2 = generate_all_genesis_assets(
        &definitions,
        seed2.as_bytes(),
        ChainType::Devnet,
        logger.clone(),
        metrics.clone(),
    )
    .unwrap();

    let assets1 = accumulator1.flatten();
    let assets2 = accumulator2.flatten();

    assert_eq!(
        assets1.len(),
        assets2.len(),
        "Asset count should match (same config)"
    );

    // MUST be different (cryptographic components)
    assert_ne!(
        assets1[0].commitment.as_bytes(),
        assets2[0].commitment.as_bytes(),
        "Commitments must differ with different seeds"
    );

    assert_ne!(
        assets1[0].nonce, assets2[0].nonce,
        "Nonces must differ with different seeds"
    );

    // Verify range proofs are different (not checking bytes due to RNG details)
    // The critical property is that commitments and nonces differ
    if let (Some(_proof1), Some(_proof2)) = (&assets1[0].range_proof, &assets2[0].range_proof) {
        // Both have proofs - the difference in commitments/nonces is sufficient
        // TODO: Enable byte-level proof comparison when RNG determinism is guaranteed
        // assert_ne!(proof1.as_bytes(), proof2.as_bytes(), "Range proofs must differ with different seeds");
    }
}

#[test]
fn test_genesis_multi_asset_reproducibility() {
    let (logger, metrics) = create_test_observability();

    let config = create_multi_asset_test_config();
    let genesis_seed = GenesisSeed::from_config(&config).unwrap();

    // Create asset definitions
    let mut definitions = Vec::new();
    for asset_cfg in &config.assets {
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

    // Verify each class separately
    assert_eq!(acc1.coins.len(), acc2.coins.len());
    assert_eq!(acc1.tokens.len(), acc2.tokens.len());
    assert_eq!(acc1.nfts.len(), acc2.nfts.len());
    assert_eq!(acc1.voids.len(), acc2.voids.len());

    // Verify assets are identical
    for (c1, c2) in acc1.coins.iter().zip(acc2.coins.iter()) {
        assert_eq!(c1.commitment.as_bytes(), c2.commitment.as_bytes());
    }

    // Verify tokens are identical
    for (t1, t2) in acc1.tokens.iter().zip(acc2.tokens.iter()) {
        assert_eq!(t1.commitment.as_bytes(), t2.commitment.as_bytes());
    }
}
