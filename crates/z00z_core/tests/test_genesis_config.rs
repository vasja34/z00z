//! Config Parsing Tests for Genesis Module
//!
//! Tests for YAML configuration parsing and validation.

use crate::genesis::helpers::*;
use z00z_core::assets::ObjectFamily;
use z00z_core::genesis::{
    genesis_config::{GenesisConfig, PolicyProfileConfig, ProfileAnchor, WalletProfileConfig},
    validator::{validate_config_schema, GenesisError},
    GenesisSeed,
};

const VALID_RIGHTS_YAML: &str = concat!(
    "rights:\n",
    "  - id: service_entitlement\n",
    "    right_class: service_entitlement\n",
    "    issuer_scope: \"issuer_test\"\n",
    "    provider_scope: \"provider_test\"\n",
    "    holder_fixture: \"wallet_alice\"\n",
    "    control_fixture: \"wallet_alice\"\n",
    "    beneficiary_fixture: \"wallet_alice\"\n",
    "    count: 1\n",
    "    domain_name: \"rights.test.v1\"\n",
    "    valid_from: 0\n",
    "    valid_until: 100\n",
    "    challenge_from: 0\n",
    "    challenge_until: 0\n",
    "    revocation_policy_id: \"policy_revoke\"\n",
    "    transition_policy_id: \"policy_transition\"\n",
    "    challenge_policy_id: \"policy_challenge\"\n",
    "    disclosure_policy_id: \"policy_disclosure\"\n",
    "    retention_policy_id: \"policy_retention\"\n",
    "    payload_commitment_seed: \"payload_seed\"\n",
    "    metadata:\n",
    "      purpose: \"create, transfer, revoke\"\n",
);

fn create_test_wallet_profile() -> WalletProfileConfig {
    WalletProfileConfig {
        id: "service_entitlement_v1".to_string(),
        object_family: ObjectFamily::Right,
        live_anchor: ProfileAnchor::One("service_entitlement".to_string()),
        transitions: vec!["grant".to_string(), "consume".to_string()],
        fail_closed: vec!["unknown_policy".to_string()],
        product_anchor: None,
        backing_kind: None,
        transferability: None,
        redeem_target: None,
        expiry_rule: None,
        disclosure_policy: Some("policy_disclosure".to_string()),
        retention_policy: Some("policy_retention".to_string()),
        provider_scope: Some("scoped".to_string()),
        beneficiary_scope: Some("scoped".to_string()),
        audit_trail: None,
        challenge_window: None,
        action_whitelist: vec![],
        quota: None,
        service_scope: None,
        replay_mode: None,
        locked_asset_id: None,
        locked_amount: None,
        binding_surfaces: vec![],
        payload_binding: None,
        lock_window: None,
        revoke_mode: None,
        ordinary_spend: None,
        accept_policy: None,
        partial_redeem: None,
        residual_policy: None,
    }
}

fn create_test_policy_profile(id: &str) -> PolicyProfileConfig {
    PolicyProfileConfig {
        id: id.to_string(),
        enforced_actions: vec!["create".to_string()],
        selected_fields: vec!["invoice_reference".to_string()],
        require_purpose: true,
        require_expiry: true,
        bind_policy_id: true,
        bind_checkpoint_anchor: false,
        bind_retained_document_hash: false,
        disclosure_receipt_required: false,
        retention_profile: None,
        retained_object_type: None,
        required_bindings: vec![],
        retention_years: None,
        disclosure_receipt_type: None,
        applies_to_profiles: vec![],
    }
}

#[test]
fn test_genesis_config_parsing_success() {
    let yaml = format!(
        r#"
chain:
    id: 3
    type: devnet
    name: "z00z-devnet-1"
    magic_bytes: [90, 48, 48, 90]
    domains:
        genesis_seed: [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25, 26, 27, 28, 29, 30, 31, 32]
assets:
    - id: z00z
      class: Coin
      name: "Z00Z Coin"
      symbol: Z00Z
      domain_name: "genesis.z00z.network"
      policy:
        decimals: 8
        serials: 100
        nominal: 20000
outputs:
    assets_export_path: "outputs/genesis/assets"
    snapshot_export_path: "outputs/genesis/snapshots"
performance:
    num_threads: auto
{}
"#,
        VALID_RIGHTS_YAML
    );

    let config: GenesisConfig = serde_yaml::from_str(&yaml).unwrap();

    assert_eq!(config.chain.id, 3);
    assert_eq!(config.chain.chain_type, "devnet");
    assert_eq!(config.assets.len(), 1);
    assert_eq!(config.assets[0].symbol, "Z00Z");
    assert_eq!(config.assets[0].policy.serials, 100);
    assert_eq!(config.assets[0].policy.decimals, 8);
    assert_eq!(config.assets[0].policy.nominal, 20000);
    assert_eq!(
        config.outputs.snapshot_export_path,
        "outputs/genesis/snapshots"
    );
}

#[test]
fn test_genesis_config_invalid_decimals() {
    // YAML with invalid decimals (exceeds u8::MAX)
    let yaml = format!(
        r#"
chain:
    id: 1
    type: devnet
    name: "z00z-devnet-1"
    magic_bytes: [90, 48, 48, 90]
    domains:
        genesis_seed: [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25, 26, 27, 28, 29, 30, 31, 32]
assets:
    - id: test
      class: Coin
      name: "Test"
      symbol: TST
      domain_name: "test.z00z"
      policy:
        decimals: 256
        serials: 10
        nominal: 1000
outputs:
    assets_export_path: "outputs/test"
    snapshot_export_path: "outputs/test/snapshots"
performance:
    num_threads: auto
{}
"#,
        VALID_RIGHTS_YAML
    );

    let result: Result<GenesisConfig, _> = serde_yaml::from_str(&yaml);
    assert!(result.is_err(), "Should reject invalid decimals");
}

#[test]
fn test_genesis_config_rejects_deprecated_chain_field_even_with_name_present() {
    let yaml = format!(
        r#"
chain:
    id: 3
    type: devnet
    name: "z00z-devnet-1"
    chain: "deprecated-devnet-name"
    magic_bytes: [90, 48, 48, 90]
    domains:
        genesis_seed: [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25, 26, 27, 28, 29, 30, 31, 32]
assets:
    - id: z00z
      class: Coin
      name: "Z00Z Coin"
      symbol: Z00Z
      domain_name: "genesis.z00z.network"
      policy:
        decimals: 8
        serials: 100
        nominal: 20000
outputs:
    assets_export_path: "outputs/genesis/assets"
    snapshot_export_path: "outputs/genesis/snapshots"
performance:
    num_threads: auto
{}
"#,
        VALID_RIGHTS_YAML
    );

    let result: Result<GenesisConfig, _> = serde_yaml::from_str(&yaml);
    assert!(
        result.is_err(),
        "Should reject deprecated chain.chain field"
    );
}

#[test]
fn test_genesis_config_rejects_deprecated_network_key() {
    let yaml = format!(
        r#"
chain:
    id: 3
    type: devnet
    name: "z00z-devnet-1"
    magic_bytes: [90, 48, 48, 90]
    domains:
        genesis_seed: [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25, 26, 27, 28, 29, 30, 31, 32]
network:
    id: 3
    type: devnet
    name: "deprecated-network-alias"
    magic_bytes: [90, 48, 48, 90]
    domains:
        genesis_seed: [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25, 26, 27, 28, 29, 30, 31, 32]
assets:
    - id: z00z
      class: Coin
      name: "Z00Z Coin"
      symbol: Z00Z
      domain_name: "genesis.z00z.network"
      policy:
        decimals: 8
        serials: 100
        nominal: 20000
outputs:
    assets_export_path: "outputs/genesis/assets"
    snapshot_export_path: "outputs/genesis/snapshots"
performance:
    num_threads: auto
{}
"#,
        VALID_RIGHTS_YAML
    );

    let result: Result<GenesisConfig, _> = serde_yaml::from_str(&yaml);
    assert!(
        result.is_err(),
        "Should reject deprecated top-level network key"
    );
}

#[test]
fn test_genesis_config_rejects_unknown_wallet_profile_right_policy_reference() {
    let mut config = create_test_config();
    let mut wallet_profile = create_test_wallet_profile();
    wallet_profile.disclosure_policy = Some("missing_policy".to_string());
    config.wallet_profiles.push(wallet_profile);

    let err = validate_config_schema(&config).unwrap_err();
    assert!(
        err.to_string()
            .contains("unknown rights.disclosure_policy_id missing_policy"),
        "unexpected error: {err}"
    );
}

#[test]
fn test_genesis_config_rejects_unknown_wallet_profile_locked_asset_reference() {
    let mut config = create_test_config();
    let mut wallet_profile = create_test_wallet_profile();
    wallet_profile.locked_asset_id = Some("missing_asset".to_string());
    config.wallet_profiles.push(wallet_profile);

    let err = validate_config_schema(&config).unwrap_err();
    assert!(
        err.to_string()
            .contains("unknown locked_asset_id missing_asset"),
        "unexpected error: {err}"
    );
}

#[test]
fn test_genesis_config_rejects_unknown_policy_profile_retention_reference() {
    let mut config = create_test_config();
    let mut policy_profile = create_test_policy_profile("corporate_eu_transfer_v1");
    policy_profile.retention_profile = Some("missing_retention_profile".to_string());
    config.policy_profiles.push(policy_profile);

    let err = validate_config_schema(&config).unwrap_err();
    assert!(
        err.to_string()
            .contains("unknown retention_profile missing_retention_profile"),
        "unexpected error: {err}"
    );
}

#[test]
fn test_genesis_config_rejects_unknown_policy_profile_applies_to_reference() {
    let mut config = create_test_config();
    config
        .policy_profiles
        .push(create_test_policy_profile("corporate_eu_transfer_v1"));
    let mut policy_profile = create_test_policy_profile("sanctions_screened_counterparty_v1");
    policy_profile.applies_to_profiles = vec!["missing_profile".to_string()];
    config.policy_profiles.push(policy_profile);

    let err = validate_config_schema(&config).unwrap_err();
    assert!(
        err.to_string()
            .contains("unknown applies_to_profiles entry missing_profile"),
        "unexpected error: {err}"
    );
}

#[test]
fn test_rejects_zero_seed_genesis() {
    let mut config = create_test_config();
    config.chain.domains.genesis_seed = [0u8; 32]; // Insecure!

    let result = GenesisSeed::from_config(&config);
    assert!(result.is_err(), "Should reject all-zeros seed");

    if let Err(GenesisError::InsecureGenesisSeed(msg)) = result {
        assert!(msg.contains("zero"), "Error should mention zero seed");
    } else {
        panic!("Wrong error type");
    }
}

#[test]
fn test_rejects_seed_genesis_validation() {
    let mut config = create_test_config();
    config.chain.domains.genesis_seed = [0xFFu8; 32]; // All ones

    let result = GenesisSeed::from_config(&config);
    assert!(result.is_err(), "Should reject all-ones seed");

    if let Err(GenesisError::InsecureGenesisSeed(msg)) = result {
        assert!(msg.contains("ones"), "Error should mention ones seed");
    } else {
        panic!("Wrong error type");
    }
}

#[test]
fn test_rejects_sequential_seed_genesis() {
    let mut config = create_test_config();
    config.chain.domains.genesis_seed = [
        1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25,
        26, 27, 28, 29, 30, 31, 32,
    ];

    let result = GenesisSeed::from_config(&config);
    assert!(result.is_err(), "Should reject sequential pattern");

    if let Err(GenesisError::InsecureGenesisSeed(msg)) = result {
        assert!(
            msg.contains("Sequential"),
            "Error should mention sequential pattern"
        );
    } else {
        panic!("Wrong error type");
    }
}

#[test]
fn test_seed_genesis_validation_accepts() {
    let mut config = create_test_config();
    // Valid seed with good entropy
    config.chain.domains.genesis_seed = [
        0x12, 0x34, 0x56, 0x78, 0x9A, 0xBC, 0xDE, 0xF0, 0x11, 0x22, 0x33, 0x44, 0x55, 0x66, 0x77,
        0x88, 0xAA, 0xBB, 0xCC, 0xDD, 0xEE, 0xFF, 0x00, 0x11, 0x22, 0x33, 0x44, 0x55, 0x66, 0x77,
        0x88, 0x99,
    ];

    let result = GenesisSeed::from_config(&config);
    assert!(result.is_ok(), "Should accept valid seed");
}

#[test]
fn test_seed_genesis_low_entropy() {
    let mut config = create_test_config();
    config.chain.chain_type = "devnet".to_string();

    // Repeating test seeds still fail closed even on devnet.
    config.chain.domains.genesis_seed = [42u8; 32];

    let result = GenesisSeed::from_config(&config);
    assert!(
        result.is_err(),
        "Repeating test seed should be rejected on devnet"
    );
}

#[test]
fn test_protected_network_seed_rejected() {
    let mut config = create_test_config();
    config.chain.chain_type = "mainnet".to_string();
    config.chain.domains.genesis_seed = [42u8; 32];

    let result = GenesisSeed::from_config(&config);
    assert!(
        result.is_err(),
        "Known test seed should be rejected on mainnet"
    );
}

#[test]
fn test_unknown_chain_type_rejected() {
    let mut config = create_test_config();
    config.chain.chain_type = "staging".to_string();

    let result = GenesisSeed::from_config(&config);
    assert!(result.is_err(), "Unknown chain type should fail closed");
}

#[test]
fn test_genesis_config_multi_asset() {
    let config = create_multi_asset_test_config();

    assert_eq!(config.assets.len(), 2);
    assert_eq!(config.assets[0].class, z00z_core::assets::AssetClass::Coin);
    assert_eq!(config.assets[1].class, z00z_core::assets::AssetClass::Token);
}

#[test]
fn test_genesis_config_output_path() {
    let config = create_test_config();

    assert_eq!(config.outputs.assets_export_path, "outputs/test");
}
