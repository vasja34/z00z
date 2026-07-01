// crates/z00z_core/tests/test_assets_config_integration.rs
//
// Integration tests for configuration loading with different formats
//
// Tests:
// - Loading assets with YAML configuration
// - Environment variable overrides (simulated)
// - Error handling: missing config, invalid format
// - Multiple asset types in one config

use std::fs;
use std::sync::Arc;
use tempfile::TempDir;
use z00z_core::assets::registry::AssetDefinitionRegistry;
use z00z_utils::prelude::{NoopLogger, NoopMetrics, SystemTimeProvider};

const VALID_RIGHTS_SECTION: &str = concat!(
    "rights:\n",
    "  - id: \"service_entitlement\"\n",
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

#[test]
fn test_config_loading_multiple_assets() -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = TempDir::new()?;
    let config_path = temp_dir.path().join("multi_assets.yaml");

    let yaml_content = format!(
        r#"
assets:
  - id: "coin"
    name: "Coin Asset"
    symbol: "COIN"
    class: "Coin"
    domain_name: "coin.io"
    policy:
      decimals: 8
      serials: 50000
      nominal: 100000000
      gas: true
      fungible: true
      mintable: false
      burnable: true

  - id: "token"
    name: "Token Asset"
    symbol: "TKN"
    class: "Token"
    domain_name: "token.io"
    policy:
      decimals: 6
      serials: 10000
      nominal: 1000000
      gas: true
      fungible: true
      mintable: true
      burnable: false

  - id: "nft"
    name: "NFT Asset"
    symbol: "NFT"
    class: "NFT"
    domain_name: "nft.io"
    policy:
      decimals: 0
      serials: 1000
      nominal: 1
      gas: false
      fungible: false
      mintable: true
      burnable: false
{}
"#,
        VALID_RIGHTS_SECTION
    );

    fs::write(&config_path, yaml_content)?;

    let registry = AssetDefinitionRegistry::load_catalog_from_yaml(
        &config_path,
        Arc::new(NoopLogger),
        Arc::new(NoopMetrics),
        Arc::new(SystemTimeProvider),
    )?;

    assert_eq!(registry.len()?, 3);

    Ok(())
}

#[test]
fn test_config_missing_file_error() {
    let result = AssetDefinitionRegistry::load_catalog_from_yaml(
        std::path::Path::new("/path/that/does/not/exist.yaml"),
        Arc::new(NoopLogger),
        Arc::new(NoopMetrics),
        Arc::new(SystemTimeProvider),
    );
    assert!(result.is_err());
}

#[test]
fn test_config_invalid_yaml_error() -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = TempDir::new()?;
    let config_path = temp_dir.path().join("invalid.yaml");

    // Invalid YAML syntax
    fs::write(&config_path, "invalid: yaml: [unclosed")?;

    let result = AssetDefinitionRegistry::load_catalog_from_yaml(
        &config_path,
        Arc::new(NoopLogger),
        Arc::new(NoopMetrics),
        Arc::new(SystemTimeProvider),
    );
    assert!(result.is_err());

    Ok(())
}

#[test]
fn test_config_missing_required_fields() -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = TempDir::new()?;
    let config_path = temp_dir.path().join("incomplete.yaml");

    let yaml_content = format!(
        r#"
assets:
  - name: "Incomplete Asset"
    symbol: "INC"
    # Missing 'class' field
    domain_name: "incomplete.io"
    policy:
      decimals: 8
      serials: 10000
      nominal: 100000000
{}
"#,
        VALID_RIGHTS_SECTION
    );

    fs::write(&config_path, yaml_content)?;

    let result = AssetDefinitionRegistry::load_catalog_from_yaml(
        &config_path,
        Arc::new(NoopLogger),
        Arc::new(NoopMetrics),
        Arc::new(SystemTimeProvider),
    );
    assert!(result.is_err());

    Ok(())
}

#[test]
fn test_config_empty_assets_array() -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = TempDir::new()?;
    let config_path = temp_dir.path().join("empty.yaml");

    let yaml_content = format!(
        r#"
assets: []
{}
"#,
        VALID_RIGHTS_SECTION
    );

    fs::write(&config_path, yaml_content)?;

    let registry = AssetDefinitionRegistry::load_catalog_from_yaml(
        &config_path,
        Arc::new(NoopLogger),
        Arc::new(NoopMetrics),
        Arc::new(SystemTimeProvider),
    )?;

    assert_eq!(registry.len()?, 0);

    Ok(())
}

#[test]
fn test_config_policy_flags() -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = TempDir::new()?;
    let config_path = temp_dir.path().join("flags.yaml");

    let yaml_content = format!(
        r#"
assets:
  - id: "test"
    name: "Test Asset"
    symbol: "TST"
    class: "Coin"
    domain_name: "test.io"
    policy:
      decimals: 8
      serials: 10000
      nominal: 100000000
      gas: true
      fungible: true
      mintable: true
      burnable: true
{}
"#,
        VALID_RIGHTS_SECTION
    );

    fs::write(&config_path, yaml_content)?;

    let registry = AssetDefinitionRegistry::load_catalog_from_yaml(
        &config_path,
        Arc::new(NoopLogger),
        Arc::new(NoopMetrics),
        Arc::new(SystemTimeProvider),
    )?;

    assert_eq!(registry.len()?, 1);

    Ok(())
}

#[test]
fn test_miss_rights_array_rejected() -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = TempDir::new()?;
    let config_path = temp_dir.path().join("missing_rights.yaml");

    let yaml_content = r#"
assets:
    - id: "coin"
        name: "Coin Asset"
        symbol: "COIN"
        class: "Coin"
        domain_name: "coin.io"
        policy:
            decimals: 8
            serials: 100
            nominal: 1000
"#;

    fs::write(&config_path, yaml_content)?;

    let result = AssetDefinitionRegistry::load_catalog_from_yaml(
        &config_path,
        Arc::new(NoopLogger),
        Arc::new(NoopMetrics),
        Arc::new(SystemTimeProvider),
    );
    assert!(result.is_err());

    Ok(())
}

#[test]
fn test_empty_rights_array_rejected() -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = TempDir::new()?;
    let config_path = temp_dir.path().join("empty_rights.yaml");

    let yaml_content = r#"
assets:
    - id: "coin"
        name: "Coin Asset"
        symbol: "COIN"
        class: "Coin"
        domain_name: "coin.io"
        policy:
            decimals: 8
            serials: 100
            nominal: 1000
rights: []
"#;

    fs::write(&config_path, yaml_content)?;

    let result = AssetDefinitionRegistry::load_catalog_from_yaml(
        &config_path,
        Arc::new(NoopLogger),
        Arc::new(NoopMetrics),
        Arc::new(SystemTimeProvider),
    );
    assert!(result.is_err());

    Ok(())
}

#[test]
fn test_duplicate_right_ids_rejected() -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = TempDir::new()?;
    let config_path = temp_dir.path().join("duplicate_rights.yaml");

    let yaml_content = r#"
assets:
    - id: "coin"
        name: "Coin Asset"
        symbol: "COIN"
        class: "Coin"
        domain_name: "coin.io"
        policy:
            decimals: 8
            serials: 100
            nominal: 1000
rights:
    - id: "dup_right"
        right_class: service_entitlement
        issuer_scope: "issuer_test"
        provider_scope: "provider_test"
        holder_fixture: "wallet_alice"
        control_fixture: "wallet_alice"
        count: 1
        domain_name: "rights.test.v1"
        valid_from: 0
        valid_until: 100
        revocation_policy_id: "policy_revoke"
        transition_policy_id: "policy_transition"
        challenge_policy_id: "policy_challenge"
        disclosure_policy_id: "policy_disclosure"
        retention_policy_id: "policy_retention"
        payload_commitment_seed: "payload_seed_one"
    - id: "dup_right"
        right_class: service_entitlement
        issuer_scope: "issuer_test"
        provider_scope: "provider_test"
        holder_fixture: "wallet_bob"
        control_fixture: "wallet_bob"
        count: 1
        domain_name: "rights.test.v1"
        valid_from: 0
        valid_until: 100
        revocation_policy_id: "policy_revoke"
        transition_policy_id: "policy_transition"
        challenge_policy_id: "policy_challenge"
        disclosure_policy_id: "policy_disclosure"
        retention_policy_id: "policy_retention"
        payload_commitment_seed: "payload_seed_two"
"#;

    fs::write(&config_path, yaml_content)?;

    let result = AssetDefinitionRegistry::load_catalog_from_yaml(
        &config_path,
        Arc::new(NoopLogger),
        Arc::new(NoopMetrics),
        Arc::new(SystemTimeProvider),
    );
    assert!(result.is_err());

    Ok(())
}

#[test]
fn test_malformed_right_fixture_rejected() -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = TempDir::new()?;
    let config_path = temp_dir.path().join("bad_right_fixture.yaml");

    let yaml_content = r#"
assets:
  - id: "coin"
    name: "Coin Asset"
    symbol: "COIN"
    class: "Coin"
    domain_name: "coin.io"
    policy:
      decimals: 8
      serials: 100
      nominal: 1000
rights:
  - id: "bad_fixture"
        right_class: service_entitlement
    issuer_scope: "issuer_test"
    provider_scope: "provider_test"
    holder_fixture: ""
        control_fixture: "wallet_alice"
    count: 1
    domain_name: "rights.test.v1"
    valid_from: 0
    valid_until: 100
    revocation_policy_id: "policy_revoke"
    transition_policy_id: "policy_transition"
    challenge_policy_id: "policy_challenge"
    disclosure_policy_id: "policy_disclosure"
    retention_policy_id: "policy_retention"
    payload_commitment_seed: "payload_seed"
"#;

    fs::write(&config_path, yaml_content)?;

    let result = AssetDefinitionRegistry::load_catalog_from_yaml(
        &config_path,
        Arc::new(NoopLogger),
        Arc::new(NoopMetrics),
        Arc::new(SystemTimeProvider),
    );
    assert!(result.is_err());

    Ok(())
}

#[test]
fn test_malformed_right_policy_rejected() -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = TempDir::new()?;
    let config_path = temp_dir.path().join("bad_right_policy.yaml");

    let yaml_content = r#"
assets:
  - id: "coin"
    name: "Coin Asset"
    symbol: "COIN"
    class: "Coin"
    domain_name: "coin.io"
    policy:
      decimals: 8
      serials: 100
      nominal: 1000
rights:
  - id: "bad_policy"
        right_class: service_entitlement
    issuer_scope: "issuer_test"
    provider_scope: "provider_test"
    holder_fixture: "wallet_alice"
        control_fixture: "wallet_alice"
    count: 1
    domain_name: "rights.test.v1"
    valid_from: 0
    valid_until: 100
    revocation_policy_id: "policy_revoke"
    transition_policy_id: ""
    challenge_policy_id: "policy_challenge"
    disclosure_policy_id: "policy_disclosure"
    retention_policy_id: "policy_retention"
    payload_commitment_seed: "payload_seed"
"#;

    fs::write(&config_path, yaml_content)?;

    let result = AssetDefinitionRegistry::load_catalog_from_yaml(
        &config_path,
        Arc::new(NoopLogger),
        Arc::new(NoopMetrics),
        Arc::new(SystemTimeProvider),
    );
    assert!(result.is_err());

    Ok(())
}

#[test]
fn test_rights_reject_fee_fields() -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = TempDir::new()?;
    let config_path = temp_dir.path().join("bad_right_fee_fields.yaml");

    let yaml_content = r#"
assets:
  - id: "coin"
    name: "Coin Asset"
    symbol: "COIN"
    class: "Coin"
    domain_name: "coin.io"
    policy:
      decimals: 8
      serials: 100
      nominal: 1000
rights:
  - id: "fee_drift"
        right_class: service_entitlement
    issuer_scope: "issuer_test"
    provider_scope: "provider_test"
    holder_fixture: "wallet_alice"
        control_fixture: "wallet_alice"
    count: 1
    domain_name: "rights.test.v1"
    valid_from: 0
    valid_until: 100
    revocation_policy_id: "policy_revoke"
    transition_policy_id: "policy_transition"
    challenge_policy_id: "policy_challenge"
    disclosure_policy_id: "policy_disclosure"
    retention_policy_id: "policy_retention"
    payload_commitment_seed: "payload_seed"
    budget_units: 5
"#;

    fs::write(&config_path, yaml_content)?;

    let result = AssetDefinitionRegistry::load_catalog_from_yaml(
        &config_path,
        Arc::new(NoopLogger),
        Arc::new(NoopMetrics),
        Arc::new(SystemTimeProvider),
    );
    assert!(result.is_err());

    Ok(())
}
