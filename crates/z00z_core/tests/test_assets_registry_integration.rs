// crates/z00z_core/tests/test_assets_registry_integration.rs
//
// Integration tests for AssetRegistry with real file I/O
//
// Tests:
// - Full asset lifecycle
// - Loading from YAML configuration files
// - Concurrent access patterns
// - Error handling

use std::sync::Arc;
use std::{fs, thread};
use tempfile::TempDir;
use z00z_core::assets::{
    definition::AssetDefinition,
    registry::{AssetDefinitionRegistry, GLOBAL_ASSET_REGISTRY},
    AssetClass,
};
use z00z_utils::prelude::{NoopLogger, NoopMetrics, SystemTimeProvider};

use super::fixtures::create_test_registry;

const VALID_RIGHTS_SECTION: &str = concat!(
    "rights:\n",
    "  - id: \"registry_right\"\n",
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

fn create_registry_definition(id: [u8; 32], name: &str, symbol: &str) -> AssetDefinition {
    AssetDefinition::new(
        id,
        AssetClass::Token,
        name.into(),
        symbol.into(),
        6,
        25_000,
        1_000_000,
        "registry.test".into(),
        1,
        1,
        0b0000_0001,
        None,
    )
    .expect("valid registry definition")
}

#[test]
fn test_registry_full_lifecycle() -> Result<(), Box<dyn std::error::Error>> {
    // Create registry with new API (no dependencies needed)
    let registry = create_test_registry();

    // Create and insert assets
    let def1 = AssetDefinition::new(
        [1u8; 32],
        AssetClass::Coin,
        "Z00Z".into(),
        "Z00Z Coin".into(),
        8,
        50_000,
        100_000_000,
        "z00z.io".into(),
        1,
        1,
        0b0000_0001,
        None,
    )?;

    let arc1 = registry.insert(def1)?;
    assert_eq!(registry.len()?, 1);

    // Retrieve and verify
    let retrieved = registry.get(&arc1.id)?.expect("definition should exist");
    assert!(Arc::ptr_eq(&arc1, &retrieved));

    Ok(())
}

#[test]
fn test_registry_yaml_config_loading() -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = TempDir::new()?;
    let config_path = temp_dir.path().join("test_assets.yaml");

    // Create test YAML config
    let yaml_content = format!(
        r#"
assets:
  - id: "test_coin"
    name: "Test Coin"
    symbol: "TST"
    class: "Coin"
    domain_name: "test.io"
    policy:
      decimals: 8
      serials: 10000
      nominal: 100000000
      gas: true
      fungible: true
      mintable: false
      burnable: false
{}
"#,
        VALID_RIGHTS_SECTION
    );

    fs::write(&config_path, yaml_content)?;

    // Create registry using load_catalog_from_yaml (new API)
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
fn test_registry_error_handling_missing() {
    let result = AssetDefinitionRegistry::load_catalog_from_yaml(
        std::path::Path::new("/nonexistent/config.yaml"),
        Arc::new(NoopLogger),
        Arc::new(NoopMetrics),
        Arc::new(SystemTimeProvider),
    );
    assert!(result.is_err());
}

#[test]
fn test_registry_concurrent_access() -> Result<(), Box<dyn std::error::Error>> {
    let registry = Arc::new(create_test_registry());

    // Spawn multiple threads inserting assets
    let handles: Vec<_> = (0..5)
        .map(|i| {
            let registry: Arc<AssetDefinitionRegistry> = Arc::clone(&registry);
            thread::spawn(move || {
                let def = AssetDefinition::new(
                    [i; 32],
                    AssetClass::Token,
                    format!("TKN{}", i),
                    format!("Token {}", i),
                    6,
                    25_000,
                    1_000_000,
                    "test.io".into(),
                    1,
                    1,
                    0b0000_0001,
                    None,
                )
                .unwrap();
                registry.insert(def).unwrap();
            })
        })
        .collect();

    for handle in handles {
        handle.join().unwrap();
    }

    assert_eq!(registry.len()?, 5);

    Ok(())
}

/// Test concurrent mixed operations to verify no deadlocks occur
///
/// This test exercises the lock ordering documented in AssetDefinitionRegistry:
/// - Multiple threads performing get() (read lock on definitions)
/// - Multiple threads performing insert() (write lock on definitions)
/// - Multiple threads calling get_version() (read lock on version)
///
/// If lock ordering is violated, this test will deadlock and timeout.
#[test]
fn test_registry_concurrent_mixed_operations() -> Result<(), Box<dyn std::error::Error>> {
    use std::thread;
    use std::time::Duration;
    use z00z_utils::time::Instant;

    let registry = Arc::new(create_test_registry());
    let mut initial_ids = Vec::new();

    // Pre-populate with some definitions
    for i in 0..10 {
        let def = AssetDefinition::new(
            [i; 32],
            AssetClass::Token,
            format!("TKN{}", i),
            format!("Token {}", i),
            6,
            25_000,
            1_000_000,
            "test.io".into(),
            1,
            1,
            0b0000_0001,
            None,
        )?;
        initial_ids.push(def.id);
        registry.insert(def)?;
    }

    // Spawn threads performing mixed operations
    let mut handles = vec![];

    // Reader threads (get operations)
    for _ in 0..5 {
        let reg = Arc::clone(&registry);
        let ids = initial_ids.clone();
        handles.push(thread::spawn(move || {
            for _ in 0..100 {
                let _ = reg.get(&ids[0]);
                let _ = reg.len();
            }
        }));
    }

    // Writer threads (insert operations)
    for i in 10..15 {
        let reg = Arc::clone(&registry);
        handles.push(thread::spawn(move || {
            for j in 0..20 {
                let id = [i + j; 32];
                let def = AssetDefinition::new(
                    id,
                    AssetClass::Token,
                    format!("TKN{}{}", i, j),
                    format!("Token {}{}", i, j),
                    6,
                    25_000,
                    1_000_000,
                    "test.io".into(),
                    1,
                    1,
                    0b0000_0001,
                    None,
                )
                .unwrap();
                let _ = reg.insert(def);
            }
        }));
    }

    // Version reader threads
    for _ in 0..3 {
        let reg = Arc::clone(&registry);
        handles.push(thread::spawn(move || {
            for _ in 0..100 {
                let _ = reg.get_version();
            }
        }));
    }

    // Join with timeout to detect deadlocks
    let join_start = Instant::now();
    let timeout = Duration::from_secs(10);

    for handle in handles {
        if join_start.elapsed() > timeout {
            panic!(
                "Test deadlock detected! Threads did not complete within {} seconds",
                timeout.as_secs()
            );
        }
        handle.join().expect("Thread should not panic");
    }

    // Verify registry is still functional
    assert!(registry.len()? >= 10);

    println!(
        "✅ Concurrent mixed operations completed in {:?} without deadlock",
        join_start.elapsed()
    );

    Ok(())
}

#[test]
fn test_registry_explicit_owner_stays_local_until_global_sync(
) -> Result<(), Box<dyn std::error::Error>> {
    let definitions = vec![
        create_registry_definition(
            [
                0x63, 0x06, 0x10, 0x91, 0xAA, 0x44, 0x02, 0x17, 0x9C, 0xDE, 0x55, 0x70, 0x12, 0x5A,
                0xC1, 0xEE, 0x01, 0x88, 0x33, 0x7D, 0xFE, 0x19, 0xA4, 0x28, 0x60, 0x14, 0x92, 0x4B,
                0x0C, 0x71, 0xE5, 0x3A,
            ],
            "Registry Alpha",
            "RGA",
        ),
        create_registry_definition(
            [
                0x63, 0x06, 0x21, 0xA2, 0xBC, 0x55, 0x13, 0x28, 0xAD, 0xEF, 0x66, 0x81, 0x23, 0x6B,
                0xD2, 0xF0, 0x12, 0x99, 0x44, 0x8E, 0xE1, 0x2A, 0xB5, 0x39, 0x71, 0x25, 0xA3, 0x5C,
                0x1D, 0x82, 0xF6, 0x4B,
            ],
            "Registry Beta",
            "RGB",
        ),
    ];
    let first_id = definitions[0].id;
    let second_id = definitions[1].id;
    assert!(!GLOBAL_ASSET_REGISTRY.contains(&first_id)?);
    assert!(!GLOBAL_ASSET_REGISTRY.contains(&second_id)?);
    let registry = AssetDefinitionRegistry::from_definitions(&definitions)?;

    assert!(registry.contains(&first_id)?);
    assert!(registry.contains(&second_id)?);
    assert!(!GLOBAL_ASSET_REGISTRY.contains(&first_id)?);
    assert!(!GLOBAL_ASSET_REGISTRY.contains(&second_id)?);

    registry.sync_global_fallback()?;

    assert!(GLOBAL_ASSET_REGISTRY.contains(&first_id)?);
    assert!(GLOBAL_ASSET_REGISTRY.contains(&second_id)?);

    Ok(())
}
