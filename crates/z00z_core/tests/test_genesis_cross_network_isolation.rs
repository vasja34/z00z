//! Cross-Network Isolation Tests
//!
//! CRITICAL SECURITY TESTS: Verify that devnet, testnet, and mainnet
//! use different cryptographic domains and produce different outputs.
//!
//! These tests ensure that assets/keys generated on one network
//! cannot be replayed or leaked to another network.

use z00z_core::assets::AssetClass;
use z00z_core::genesis::genesis_config::{AssetConfigEntry, PolicyConfig};
use z00z_core::genesis::ChainType;
use z00z_core::genesis::{
    create_asset_definition, derive_deterministic_rng_seed, derive_genesis_blinding,
};

/// Test that blinding factors differ across networks with same inputs
#[test]
fn test_cross_network_blinding_isolation() {
    let genesis_seed = [0x42u8; 32];
    let asset_id = [0x01u8; 32];
    let serial_id = 0u32;

    // Generate blinding for all three networks with IDENTICAL inputs
    let blinding_devnet =
        derive_genesis_blinding(&genesis_seed, &asset_id, serial_id, ChainType::Devnet).unwrap();

    let blinding_testnet =
        derive_genesis_blinding(&genesis_seed, &asset_id, serial_id, ChainType::Testnet).unwrap();

    let blinding_mainnet =
        derive_genesis_blinding(&genesis_seed, &asset_id, serial_id, ChainType::Mainnet).unwrap();

    // CRITICAL: All three MUST be different
    assert_ne!(
        blinding_devnet.as_bytes(),
        blinding_testnet.as_bytes(),
        "🔴 SECURITY VIOLATION: Devnet and testnet produce SAME blinding factor!"
    );

    assert_ne!(
        blinding_devnet.as_bytes(),
        blinding_mainnet.as_bytes(),
        "🔴 SECURITY VIOLATION: Devnet and mainnet produce SAME blinding factor!"
    );

    assert_ne!(
        blinding_testnet.as_bytes(),
        blinding_mainnet.as_bytes(),
        "🔴 SECURITY VIOLATION: Testnet and mainnet produce SAME blinding factor!"
    );

    println!("✅ Cross-network blinding isolation verified");
}

/// Test that genesis asset identity tracks canonical payload, not hidden network fallback.
#[test]
fn test_asset_cross_network_id() {
    let genesis_seed = [0x42u8; 32];

    // Same asset configuration
    let cfg = AssetConfigEntry {
        id: "z00z".to_string(),
        class: AssetClass::Coin,
        name: "Z00Z Coin".to_string(),
        symbol: "Z00Z".to_string(),
        description: None,
        domain_name: "z00z.local".to_string(),
        policy: PolicyConfig {
            decimals: 8,
            serials: 1000,
            nominal: 100_000,
            ..PolicyConfig::default()
        },
        metadata: None,
    };

    // Same canonical payload should produce the same id regardless of network input.
    let def_devnet = create_asset_definition(&cfg, &genesis_seed, ChainType::Devnet).unwrap();
    let def_testnet = create_asset_definition(&cfg, &genesis_seed, ChainType::Testnet).unwrap();
    let def_mainnet = create_asset_definition(&cfg, &genesis_seed, ChainType::Mainnet).unwrap();

    assert_eq!(def_devnet.id, def_testnet.id);
    assert_eq!(def_devnet.id, def_mainnet.id);
    assert_eq!(def_devnet.name, def_testnet.name);
    assert_eq!(def_devnet.symbol, def_testnet.symbol);
    assert_eq!(def_devnet.class, def_testnet.class);

    // Network-specific payload changes should still produce different canonical ids.
    let mut testnet_cfg = cfg.clone();
    testnet_cfg.domain_name = "z00z.testnet".to_string();
    let mut mainnet_cfg = cfg.clone();
    mainnet_cfg.domain_name = "z00z.mainnet".to_string();

    let testnet_payload_def =
        create_asset_definition(&testnet_cfg, &genesis_seed, ChainType::Testnet).unwrap();
    let mainnet_payload_def =
        create_asset_definition(&mainnet_cfg, &genesis_seed, ChainType::Mainnet).unwrap();

    assert_ne!(def_devnet.id, testnet_payload_def.id);
    assert_ne!(def_devnet.id, mainnet_payload_def.id);
    assert_ne!(testnet_payload_def.id, mainnet_payload_def.id);

    println!("✅ Canonical genesis asset identity follows payload, not fallback network coercion");
}

/// Test that RNG seeds differ across networks with same inputs
#[test]
fn test_seed_cross_network_rng() {
    let genesis_seed = [0x42u8; 32];
    let asset_id = [0x01u8; 32];
    let serial_id = 0u32;

    // Generate RNG seeds for all three networks
    let rng_seed_devnet =
        derive_deterministic_rng_seed(&genesis_seed, &asset_id, serial_id, ChainType::Devnet);

    let rng_seed_testnet =
        derive_deterministic_rng_seed(&genesis_seed, &asset_id, serial_id, ChainType::Testnet);

    let rng_seed_mainnet =
        derive_deterministic_rng_seed(&genesis_seed, &asset_id, serial_id, ChainType::Mainnet);

    // CRITICAL: All three MUST be different
    assert_ne!(
        rng_seed_devnet, rng_seed_testnet,
        "🔴 SECURITY VIOLATION: Devnet and testnet produce SAME RNG seed!"
    );

    assert_ne!(
        rng_seed_devnet, rng_seed_mainnet,
        "🔴 SECURITY VIOLATION: Devnet and mainnet produce SAME RNG seed!"
    );

    assert_ne!(
        rng_seed_testnet, rng_seed_mainnet,
        "🔴 SECURITY VIOLATION: Testnet and mainnet produce SAME RNG seed!"
    );

    println!("✅ Cross-network RNG seed isolation verified");
}

/// Negative test: Verify that SAME network DOES produce same results (determinism)
#[test]
fn test_same_network_determinism() {
    let genesis_seed = [0x42u8; 32];
    let asset_id = [0x01u8; 32];
    let serial_id = 0u32;

    // Generate twice on devnet
    let blinding1 =
        derive_genesis_blinding(&genesis_seed, &asset_id, serial_id, ChainType::Devnet).unwrap();

    let blinding2 =
        derive_genesis_blinding(&genesis_seed, &asset_id, serial_id, ChainType::Devnet).unwrap();

    // MUST be identical (deterministic)
    assert_eq!(
        blinding1.as_bytes(),
        blinding2.as_bytes(),
        "🔴 DETERMINISM VIOLATION: Same inputs produce different outputs!"
    );

    println!("✅ Same-network determinism verified");
}

/// Comprehensive test: All 3 network types produce different outputs
#[test]
fn test_all_three_networks_unique() {
    let genesis_seed = [0x99u8; 32];
    let asset_id = [0xAAu8; 32];
    let serial_id = 42u32;

    // Generate for all three networks
    let networks = [ChainType::Devnet, ChainType::Testnet, ChainType::Mainnet];

    let mut blindings = Vec::new();
    let mut rng_seeds = Vec::new();

    for network in &networks {
        let blinding =
            derive_genesis_blinding(&genesis_seed, &asset_id, serial_id, *network).unwrap();

        let rng_seed = derive_deterministic_rng_seed(&genesis_seed, &asset_id, serial_id, *network);

        blindings.push(blinding);
        rng_seeds.push(rng_seed);
    }

    // Verify all blindings are unique
    for i in 0..blindings.len() {
        for j in (i + 1)..blindings.len() {
            assert_ne!(
                blindings[i].as_bytes(),
                blindings[j].as_bytes(),
                "🔴 Networks {:?} and {:?} produce same blinding!",
                networks[i],
                networks[j]
            );
        }
    }

    // Verify all RNG seeds are unique
    for i in 0..rng_seeds.len() {
        for j in (i + 1)..rng_seeds.len() {
            assert_ne!(
                rng_seeds[i], rng_seeds[j],
                "🔴 Networks {:?} and {:?} produce same RNG seed!",
                networks[i], networks[j]
            );
        }
    }

    println!("✅ All three networks produce unique outputs");
}
