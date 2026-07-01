//! Genesis Security Validation Tests
//!
//! Property-based and statistical tests validating security properties outlined in
//! `docs/genesis/genesis_spec_crypto_review.md`.
//!
//! ## Tests Covered:
//! 1. Domain separation (devnet/testnet/mainnet isolation)
//! 2. Blinding factor uniformity (chi-square statistical test)
//! 3. Nonce uniqueness (collision detection)
//! 4. Genesis reproducibility (determinism)
//! 5. Property-based tests with proptest

use crate::genesis::helpers;
use blake2::Digest;
use proptest::prelude::*;
use std::collections::HashSet;
use z00z_core::genesis::{derive_genesis_blinding, ChainType};

/// Property-based test: Blinding factor derivation is deterministic
///
/// Property: derive_genesis_blinding(seed, asset_id, serial, network) always produces
/// the same blinding factor for identical inputs.
#[test]
fn test_proptest_blinding_determinism() {
    proptest!(|(
        seed in prop::array::uniform32(any::<u8>()),
        asset_id in prop::array::uniform32(any::<u8>()),
        serial in any::<u32>(),
        network in prop_oneof![
            Just(ChainType::Devnet),
            Just(ChainType::Testnet),
            Just(ChainType::Mainnet),
        ]
    )| {
        let b1 = derive_genesis_blinding(&seed, &asset_id, serial, network)
            .expect("blinding derivation failed");
        let b2 = derive_genesis_blinding(&seed, &asset_id, serial, network)
            .expect("blinding derivation failed");

        prop_assert_eq!(b1.as_bytes(), b2.as_bytes(), "Blinding factors must be deterministic");
    });
}

/// Property-based test: Different serial IDs produce different blinding factors
///
/// Property: For serial1 ≠ serial2, derive_genesis_blinding produces different blindings.
/// Expected collision probability: < 2^-128 (negligible).
#[test]
fn test_proptest_serial_uniqueness() {
    proptest!(|(
        seed in prop::array::uniform32(any::<u8>()),
        asset_id in prop::array::uniform32(any::<u8>()),
        serial1 in any::<u32>(),
        serial2 in any::<u32>()
    )| {
        prop_assume!(serial1 != serial2);

        let b1 = derive_genesis_blinding(&seed, &asset_id, serial1, ChainType::Mainnet)
            .expect("blinding derivation failed");
        let b2 = derive_genesis_blinding(&seed, &asset_id, serial2, ChainType::Mainnet)
            .expect("blinding derivation failed");

        prop_assert_ne!(
            b1.as_bytes(), b2.as_bytes(),
            "Different serial IDs must produce different blinding factors (collision extremely unlikely)"
        );
    });
}

/// Property-based test: Domain separation prevents cross-network collisions
///
/// Property: For different networks, derive_genesis_blinding produces different blindings.
/// Validates mainnet/testnet/devnet isolation per security spec.
#[test]
fn test_proptest_domain_separation() {
    proptest!(|(
        seed in prop::array::uniform32(any::<u8>()),
        asset_id in prop::array::uniform32(any::<u8>()),
        serial in any::<u32>()
    )| {
        let b_devnet = derive_genesis_blinding(&seed, &asset_id, serial, ChainType::Devnet)
            .expect("blinding derivation failed");
        let b_testnet = derive_genesis_blinding(&seed, &asset_id, serial, ChainType::Testnet)
            .expect("blinding derivation failed");
        let b_mainnet = derive_genesis_blinding(&seed, &asset_id, serial, ChainType::Mainnet)
            .expect("blinding derivation failed");

        let b_devnet_bytes = b_devnet.as_bytes().to_vec();
        let b_testnet_bytes = b_testnet.as_bytes().to_vec();
        let b_mainnet_bytes = b_mainnet.as_bytes().to_vec();

        prop_assert_ne!(&b_devnet_bytes, &b_testnet_bytes, "Devnet and testnet must use different blindings");
        prop_assert_ne!(&b_testnet_bytes, &b_mainnet_bytes, "Testnet and mainnet must use different blindings");
        prop_assert_ne!(&b_mainnet_bytes, &b_devnet_bytes, "Mainnet and devnet must use different blindings");
    });
}

/// Property-based test: Different asset IDs produce different blinding factors
///
/// Property: For asset_id1 ≠ asset_id2, derive_genesis_blinding produces different blindings.
#[test]
fn test_proptest_asset_id_uniqueness() {
    proptest!(|(
        seed in prop::array::uniform32(any::<u8>()),
        asset_id1 in prop::array::uniform32(any::<u8>()),
        asset_id2 in prop::array::uniform32(any::<u8>()),
        serial in any::<u32>()
    )| {
        prop_assume!(asset_id1 != asset_id2);

        let b1 = derive_genesis_blinding(&seed, &asset_id1, serial, ChainType::Mainnet)
            .expect("blinding derivation failed");
        let b2 = derive_genesis_blinding(&seed, &asset_id2, serial, ChainType::Mainnet)
            .expect("blinding derivation failed");

        prop_assert_ne!(
            b1.as_bytes(), b2.as_bytes(),
            "Different asset IDs must produce different blinding factors"
        );
    });
}

/// Statistical test: Blinding factor distribution uniformity (Chi-Square test)
///
/// Generates 10,000 blinding factors and validates that the first byte follows
/// a uniform distribution (chi-square test, α = 0.01).
///
/// Reference: `docs/genesis/genesis_spec_crypto_review.md` Section 5.3
#[test]
fn test_blinding_uniformity_chi_square() {
    const SAMPLES: usize = 10_000;
    const BUCKETS: usize = 256;
    const EXPECTED_PER_BUCKET: f64 = SAMPLES as f64 / BUCKETS as f64; // ~39.06

    let genesis_seed = [0x42; 32];
    let asset_id = [0x01; 32];
    let mut buckets = [0u64; BUCKETS];

    // Generate 10,000 blinding factors
    for serial in 0..SAMPLES as u32 {
        let blinding =
            derive_genesis_blinding(&genesis_seed, &asset_id, serial, ChainType::Mainnet)
                .expect("blinding derivation failed");

        // Extract first byte for chi-square test
        let first_byte = blinding.as_bytes()[0];
        buckets[first_byte as usize] += 1;
    }

    // Compute chi-square statistic: χ² = Σ((observed - expected)² / expected)
    let mut chi_square: f64 = 0.0;
    for &count in &buckets {
        let observed = count as f64;
        let diff = observed - EXPECTED_PER_BUCKET;
        chi_square += (diff * diff) / EXPECTED_PER_BUCKET;
    }

    // Critical value for χ²(255, α=0.01) ≈ 310.5
    // If χ² < 310.5, we fail to reject null hypothesis (uniform distribution)
    const CHI_SQUARE_CRITICAL: f64 = 310.5;

    println!("✅ Blinding factor uniformity (Chi-Square test):");
    println!("   χ² statistic: {:.2}", chi_square);
    println!("   Critical value (α=0.01): {:.2}", CHI_SQUARE_CRITICAL);
    println!(
        "   Result: {} (uniform: χ² < critical)",
        if chi_square < CHI_SQUARE_CRITICAL {
            "PASS"
        } else {
            "FAIL"
        }
    );

    assert!(
        chi_square < CHI_SQUARE_CRITICAL,
        "Chi-square test failed: χ² = {:.2} exceeds critical value {:.2}",
        chi_square,
        CHI_SQUARE_CRITICAL
    );
}

/// Collision test: Birthday bound for 2^16 blinding factors
///
/// Generates 65,536 blinding factors (2^16) with sequential serial IDs and verifies
/// no collisions occur. This validates uniqueness at the birthday bound.
///
/// Reference: `docs/genesis/genesis_spec_crypto_review.md` Section 5.3
#[test]
fn test_blinding_collision_birthday_bound() {
    const SAMPLES: u32 = 65_536; // 2^16
    let genesis_seed = [0xDE; 32];
    let asset_id = [0xCA; 32];

    let mut seen = HashSet::new();

    for serial in 0..SAMPLES {
        let blinding =
            derive_genesis_blinding(&genesis_seed, &asset_id, serial, ChainType::Mainnet)
                .expect("blinding derivation failed");

        let blinding_bytes = blinding.as_bytes().to_vec();
        let collision = !seen.insert(blinding_bytes);
        assert!(
            !collision,
            "Collision detected at serial {} (out of {} samples)",
            serial, SAMPLES
        );
    }

    println!("✅ Birthday bound collision test:");
    println!("   Generated {} unique blinding factors", SAMPLES);
    println!("   0 collisions detected");
}

/// Domain separation test: Verify network isolation prevents replay attacks
///
/// Validates that genesis assets on different networks have different blinding factors,
/// even with identical seed/asset_id/serial parameters.
#[test]
fn test_domain_separation_prevents_replay() {
    let genesis_seed = [0x00; 32];
    let asset_id = [0xFF; 32];
    let serial = 12345;

    let b_devnet = derive_genesis_blinding(&genesis_seed, &asset_id, serial, ChainType::Devnet)
        .expect("devnet blinding derivation failed");
    let b_testnet = derive_genesis_blinding(&genesis_seed, &asset_id, serial, ChainType::Testnet)
        .expect("testnet blinding derivation failed");
    let b_mainnet = derive_genesis_blinding(&genesis_seed, &asset_id, serial, ChainType::Mainnet)
        .expect("mainnet blinding derivation failed");

    let b_devnet_bytes = b_devnet.as_bytes();
    let b_testnet_bytes = b_testnet.as_bytes();
    let b_mainnet_bytes = b_mainnet.as_bytes();

    // All three networks must produce different blindings
    assert_ne!(
        b_devnet_bytes, b_testnet_bytes,
        "Devnet and testnet must have isolated genesis states"
    );
    assert_ne!(
        b_testnet_bytes, b_mainnet_bytes,
        "Testnet and mainnet must have isolated genesis states"
    );
    assert_ne!(
        b_mainnet_bytes, b_devnet_bytes,
        "Mainnet and devnet must have isolated genesis states"
    );

    println!("✅ Domain separation verified:");
    println!("   Devnet ≠ Testnet ≠ Mainnet (same seed/asset/serial)");
}

/// Genesis reproducibility test: Same inputs produce identical outputs
///
/// Validates determinism property outlined in security spec.
#[test]
fn test_genesis_reproducibility() {
    let (logger, metrics) = helpers::create_test_observability();
    use z00z_core::genesis::{create_asset_definition, generate_all_genesis_assets, GenesisSeed};

    let config = helpers::create_test_config();
    let genesis_seed = GenesisSeed::from_config(&config).unwrap();

    // Create asset definitions
    let mut definitions = Vec::new();
    for asset_cfg in &config.assets {
        let definition =
            create_asset_definition(asset_cfg, genesis_seed.as_bytes(), ChainType::Devnet).unwrap();
        definitions.push(definition);
    }

    // Generate genesis assets twice with same parameters
    let accumulator1 = generate_all_genesis_assets(
        &definitions,
        genesis_seed.as_bytes(),
        ChainType::Devnet,
        logger.clone(),
        metrics.clone(),
    )
    .expect("first generation failed");
    let accumulator2 = generate_all_genesis_assets(
        &definitions,
        genesis_seed.as_bytes(),
        ChainType::Devnet,
        logger.clone(),
        metrics.clone(),
    )
    .expect("second generation failed");

    // Verify identical outputs
    let assets1 = accumulator1.flatten();
    let assets2 = accumulator2.flatten();

    assert_eq!(
        assets1.len(),
        assets2.len(),
        "Genesis asset count must be deterministic"
    );

    for (a1, a2) in assets1.iter().zip(assets2.iter()) {
        assert_eq!(
            a1.commitment.as_bytes(),
            a2.commitment.as_bytes(),
            "Commitments must be reproducible"
        );
        // Note: Range proofs may not be byte-identical due to internal RNG state,
        // but commitments and amounts are deterministic (the important properties)
        assert_eq!(a1.amount, a2.amount, "Amounts must match");
        assert_eq!(a1.asset_id(), a2.asset_id(), "Asset IDs must match");
        assert_eq!(a1.serial_id, a2.serial_id, "Serial IDs must match");
    }

    println!("✅ Genesis reproducibility verified:");
    println!("   {} assets generated identically twice", assets1.len());
}

/// Nonce uniqueness test: Verify all genesis assets use unique nonces
///
/// Validates that range proofs use independent randomness, preventing nonce reuse attacks.
#[test]
fn test_genesis_nonce_uniqueness() {
    let (logger, metrics) = helpers::create_test_observability();
    use z00z_core::genesis::{create_asset_definition, generate_all_genesis_assets, GenesisSeed};

    let config = helpers::create_test_config();
    let genesis_seed = GenesisSeed::from_config(&config).unwrap();

    // Create asset definitions
    let mut definitions = Vec::new();
    for asset_cfg in &config.assets {
        let definition =
            create_asset_definition(asset_cfg, genesis_seed.as_bytes(), ChainType::Testnet)
                .unwrap();
        definitions.push(definition);
    }

    // Generate genesis assets
    let accumulator = generate_all_genesis_assets(
        &definitions,
        genesis_seed.as_bytes(),
        ChainType::Testnet,
        logger.clone(),
        metrics.clone(),
    )
    .expect("genesis generation failed");
    let assets = accumulator.flatten();

    // Collect all nonces (first 64 bytes of range proof as heuristic)
    // This is a heuristic check - actual nonce extraction depends on Bulletproofs+ internals
    let mut nonces = HashSet::new();
    for asset in &assets {
        let proof_bytes = asset.range_proof.as_ref().expect("range proof missing");
        let nonce_hash =
            blake2::Blake2b512::digest(&proof_bytes[..std::cmp::min(64, proof_bytes.len())]);

        let collision = !nonces.insert(nonce_hash.as_slice().to_vec());
        assert!(
            !collision,
            "Nonce collision detected for asset {:?} serial {}",
            asset.asset_id(),
            asset.serial_id
        );
    }

    println!("✅ Nonce uniqueness verified:");
    println!(
        "   {} unique nonces (out of {} assets)",
        nonces.len(),
        assets.len()
    );
}
