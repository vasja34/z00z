//! Test Helper Functions for Genesis Module
//!
//! Provides reusable test fixtures and utilities for genesis testing.

use std::sync::Arc;
use z00z_core::assets::nonce::derive_nonce;
use z00z_core::assets::{Asset, AssetClass, AssetDefinition};
use z00z_core::genesis::{
    derive_deterministic_rng_seed, derive_genesis_blinding,
    genesis_config::{
        AssetConfigEntry, ChainConfig, DomainsConfig, GenesisConfig, OutputsConfig,
        PerformanceConfig, PolicyConfig, ThreadCountConfig, ThreadCountMode,
    },
    ChainType, GenesisAssetAccumulator,
};
use z00z_core::rights::{RightClassConfig, RightsConfigEntry};
#[cfg(feature = "deterministic-rng")]
use z00z_utils::prelude::{Logger, MetricsSink, NoopLogger, NoopMetrics};
use z00z_utils::rng::DeterministicRngProvider;

/// Create test logger and metrics for genesis tests
#[cfg(feature = "deterministic-rng")]
pub fn create_test_observability() -> (Arc<dyn Logger>, Arc<dyn MetricsSink>) {
    (Arc::new(NoopLogger), Arc::new(NoopMetrics))
}

/// Create a basic test AssetDefinition for the native asset class
pub fn create_test_definition() -> AssetDefinition {
    AssetDefinition::new(
        [1u8; 32],
        AssetClass::Coin,
        "TestCoin".to_string(),
        "TST".to_string(),
        8,
        10,
        1_000_000,
        "test.z00z".to_string(),
        1,
        1,
        0,
        None,
    )
    .unwrap()
}

/// Generate a single test genesis asset
pub fn generate_test_asset() -> Asset {
    let definition = Arc::new(create_test_definition());
    let genesis_seed = [42u8; 32];
    let serial_id = 0u32;
    let amount = definition.nominal;

    let blinding =
        derive_genesis_blinding(&genesis_seed, &definition.id, serial_id, ChainType::Devnet)
            .unwrap();
    let nonce = derive_nonce(&genesis_seed, serial_id as u64, 0, &[0u8; 32]);
    let rng_seed =
        derive_deterministic_rng_seed(&genesis_seed, &definition.id, serial_id, ChainType::Devnet);
    let provider = DeterministicRngProvider::from_seed(rng_seed);
    let mut rng = provider.rng();

    Asset::new(definition, serial_id, amount, &blinding, nonce, &mut rng).unwrap()
}

/// Generate test asset with specific serial ID
pub fn generate_test_asset_with_id(serial_id: u32) -> Asset {
    let definition = Arc::new(create_test_definition());
    let genesis_seed = [42u8; 32];
    let amount = definition.nominal;

    let blinding =
        derive_genesis_blinding(&genesis_seed, &definition.id, serial_id, ChainType::Devnet)
            .unwrap();
    let nonce = derive_nonce(&genesis_seed, serial_id as u64, 0, &[0u8; 32]);
    let rng_seed =
        derive_deterministic_rng_seed(&genesis_seed, &definition.id, serial_id, ChainType::Devnet);
    let provider = DeterministicRngProvider::from_seed(rng_seed);
    let mut rng = provider.rng();

    Asset::new(definition, serial_id, amount, &blinding, nonce, &mut rng).unwrap()
}

/// Generate test accumulator with 5 assets
pub fn generate_test_accumulator() -> GenesisAssetAccumulator {
    let mut accumulator = GenesisAssetAccumulator::new();

    // Generate 5 test assets
    for i in 0..5 {
        let coin = generate_test_asset_with_id(i);
        accumulator.push(coin, AssetClass::Coin);
    }

    accumulator
}

pub fn create_test_rights_config() -> Vec<RightsConfigEntry> {
    vec![RightsConfigEntry {
        id: "test_right".to_string(),
        right_class: RightClassConfig::ServiceEntitlement,
        issuer_scope: "issuer_test".to_string(),
        provider_scope: "provider_test".to_string(),
        holder_fixture: "wallet_alice".to_string(),
        control_fixture: "wallet_alice".to_string(),
        beneficiary_fixture: Some("wallet_alice".to_string()),
        count: 2,
        domain_name: "rights.test.z00z".to_string(),
        valid_from: 0,
        valid_until: 100,
        challenge_from: 0,
        challenge_until: 0,
        revocation_policy_id: "policy_revoke".to_string(),
        transition_policy_id: "policy_transition".to_string(),
        challenge_policy_id: "policy_challenge".to_string(),
        disclosure_policy_id: "policy_disclosure".to_string(),
        retention_policy_id: "policy_retention".to_string(),
        payload_commitment_seed: "payload_seed".to_string(),
        metadata: Some(std::collections::BTreeMap::from([(
            "purpose".to_string(),
            "create, transfer, revoke".to_string(),
        )])),
    }]
}

/// Create minimal test GenesisConfig
pub fn create_test_config() -> GenesisConfig {
    GenesisConfig {
        chain: ChainConfig {
            id: 1,
            chain_type: "devnet".to_string(),
            name: "z00z-devnet-1".to_string(),
            magic_bytes: [0x5A, 0x30, 0x30, 0x5A], // "Z00Z" in hex
            domains: DomainsConfig {
                genesis_seed: [
                    0x12, 0x34, 0x56, 0x78, 0x9A, 0xBC, 0xDE, 0xF0, 0x11, 0x22, 0x33, 0x44, 0x55,
                    0x66, 0x77, 0x88, 0xAA, 0xBB, 0xCC, 0xDD, 0xEE, 0xFF, 0x00, 0x11, 0x22, 0x33,
                    0x44, 0x55, 0x66, 0x77, 0x88, 0x99,
                ],
            },
        },
        assets: vec![AssetConfigEntry {
            id: "test_coin".to_string(),
            class: AssetClass::Coin,
            name: "Test Coin".to_string(),
            symbol: "TST".to_string(),
            description: None,
            domain_name: "test.z00z".to_string(),
            policy: PolicyConfig {
                decimals: 8,
                serials: 10,
                nominal: 1_000_000,
                ..PolicyConfig::default()
            },
            metadata: None,
        }],
        rights: create_test_rights_config(),
        policies: vec![],
        vouchers: vec![],
        wallet_profiles: vec![],
        policy_profiles: vec![],
        outputs: OutputsConfig {
            assets_export_path: "outputs/test".to_string(),
            snapshot_export_path: "outputs/test/snapshots".to_string(),
            logging_path: "crates/z00z_core/outputs/log/".to_string(),
        },
        performance: PerformanceConfig {
            num_threads: ThreadCountConfig::Named(ThreadCountMode::Auto),
        },
    }
}

/// Create test config with multiple asset types
pub fn create_multi_asset_test_config() -> GenesisConfig {
    GenesisConfig {
        chain: ChainConfig {
            id: 1,
            chain_type: "devnet".to_string(),
            name: "z00z-devnet-1".to_string(),
            magic_bytes: [0x5A, 0x30, 0x30, 0x5A], // "Z00Z" in hex
            domains: DomainsConfig {
                genesis_seed: [
                    0x12, 0x34, 0x56, 0x78, 0x9A, 0xBC, 0xDE, 0xF0, 0x11, 0x22, 0x33, 0x44, 0x55,
                    0x66, 0x77, 0x88, 0xAA, 0xBB, 0xCC, 0xDD, 0xEE, 0xFF, 0x00, 0x11, 0x22, 0x33,
                    0x44, 0x55, 0x66, 0x77, 0x88, 0x99,
                ],
            },
        },
        assets: vec![
            AssetConfigEntry {
                id: "test_coin".to_string(),
                class: AssetClass::Coin,
                name: "Test Coin".to_string(),
                symbol: "TST".to_string(),
                description: None,
                domain_name: "test.z00z".to_string(),
                policy: PolicyConfig {
                    decimals: 8,
                    serials: 5,
                    nominal: 1_000_000,
                    ..PolicyConfig::default()
                },
                metadata: None,
            },
            AssetConfigEntry {
                id: "test_token".to_string(),
                class: AssetClass::Token,
                name: "Test Token".to_string(),
                symbol: "TTK".to_string(),
                description: None,
                domain_name: "token.test.z00z".to_string(),
                policy: PolicyConfig {
                    decimals: 6,
                    serials: 3,
                    nominal: 500_000,
                    ..PolicyConfig::default()
                },
                metadata: None,
            },
        ],
        rights: create_test_rights_config(),
        policies: vec![],
        vouchers: vec![],
        wallet_profiles: vec![],
        policy_profiles: vec![],
        outputs: OutputsConfig {
            assets_export_path: "outputs/test".to_string(),
            snapshot_export_path: "outputs/test/snapshots".to_string(),
            logging_path: "crates/z00z_core/outputs/log/".to_string(),
        },
        performance: PerformanceConfig {
            num_threads: ThreadCountConfig::Named(ThreadCountMode::Auto),
        },
    }
}
