use super::*;
use crate::assets::wire::DefinitionWire;
use crate::assets::AssetClass;
use std::sync::Mutex;

static GLOBAL_REG_LOCK: Lazy<Mutex<()>> = Lazy::new(|| Mutex::new(()));

// ========================================================================
// Test Helpers - Duplicated from tests/test_assets_fixtures.rs for unit tests
// ========================================================================
//
// Note: Integration tests use the shared fixtures module, but unit tests
// in this file need local helpers because they can't import from tests/.
// These helpers intentionally duplicate the fixtures to maintain
// consistency across unit and integration test environments.

/// Create a test registry with standard test dependencies.
///
/// Equivalent to `fixtures::create_test_registry()`.
fn create_test_registry() -> AssetDefinitionRegistry {
    AssetDefinitionRegistry::new(
        Arc::new(NoopLogger),
        Arc::new(NoopMetrics),
        Arc::new(SystemTimeProvider),
    )
}

/// Load registry from YAML config file for testing.
///
/// Equivalent to `fixtures::load_test_config()`.
fn load_test_config(path: &Path) -> Result<AssetDefinitionRegistry, AssetError> {
    AssetDefinitionRegistry::load_catalog_from_yaml(
        path,
        Arc::new(NoopLogger),
        Arc::new(NoopMetrics),
        Arc::new(SystemTimeProvider),
    )
}

/// Create a burnable coin asset definition for testing.
///
/// Equivalent to `fixtures::create_test_def()`.
fn create_test_def(id: u8) -> AssetDefinition {
    AssetDefinition::new(
        [id; 32],
        AssetClass::Coin,
        format!("Test Coin {id}"),
        format!("T{id}"),
        8,
        1000,
        100_000_000,
        "test.io".to_string(),
        1,
        1,
        BURNABLE,
        None,
    )
    .expect("valid test definition")
}

#[test]
fn test_registry_creation() {
    let registry = create_test_registry();
    assert_eq!(registry.len().unwrap(), 0);
    assert!(registry.is_empty().unwrap());
}

#[test]
fn test_insert_and_get() {
    let registry = create_test_registry();
    let def = create_test_def(1);
    let def_id = def.id;

    let arc1 = registry.insert(def).unwrap();
    assert_eq!(registry.len().unwrap(), 1);
    assert_eq!(registry.get_version().unwrap(), 1);

    let arc2 = registry.get(&def_id).unwrap().expect("definition exists");
    assert!(Arc::ptr_eq(&arc1, &arc2));
}

#[test]
fn test_insert_rejects_mismatch() {
    let registry = create_test_registry();
    let def = AssetDefinition {
        id: [9u8; 32],
        class: AssetClass::Coin,
        name: "Test Coin 9".to_string(),
        symbol: "T9".to_string(),
        decimals: 8,
        serials: 1000,
        nominal: 100_000_000,
        domain_name: "test.io".to_string(),
        version: 1,
        crypto_version: 1,
        policy_flags: BURNABLE,
        metadata: None,
    };

    assert!(matches!(
        registry.insert(def),
        Err(AssetError::Integrity(_))
    ));
}

#[test]
fn test_insert_duplicate_returns_arc() {
    let registry = create_test_registry();
    let def = create_test_def(1);

    let arc1 = registry.insert(def.clone()).unwrap();
    let arc2 = registry.insert(def).unwrap();

    // Same Arc instance (not just equal content)
    assert!(Arc::ptr_eq(&arc1, &arc2));
    assert_eq!(registry.len().unwrap(), 1); // Only one entry
    assert_eq!(registry.get_version().unwrap(), 1);
}

#[test]
fn test_contains() {
    let registry = create_test_registry();
    let def = create_test_def(1);
    let def_id = def.id;

    assert!(!registry.contains(&[1u8; 32]).unwrap());

    registry.insert(def).unwrap();

    assert!(registry.contains(&def_id).unwrap());
    assert!(!registry.contains(&[2u8; 32]).unwrap());
}

#[test]
fn test_multiple_definitions() {
    let registry = create_test_registry();
    let mut ids = Vec::new();

    for i in 0..10 {
        let def = create_test_def(i);
        ids.push(def.id);
        registry.insert(def).unwrap();
    }

    assert_eq!(registry.len().unwrap(), 10);

    for id in ids {
        assert!(registry.contains(&id).unwrap());
    }
}

#[test]
fn test_get_nonexistent() {
    let registry = create_test_registry();
    assert!(registry.get(&[42u8; 32]).unwrap().is_none());
}

#[test]
fn test_arc_sharing_memory_efficiency() {
    let registry = create_test_registry();
    let def = create_test_def(1);
    let def_id = def.id;

    let arc1 = registry.insert(def).unwrap();
    let arc2 = registry.get(&def_id).unwrap().unwrap();
    let arc3 = registry.get(&def_id).unwrap().unwrap();

    // All point to same allocation
    assert!(Arc::ptr_eq(&arc1, &arc2));
    assert!(Arc::ptr_eq(&arc2, &arc3));

    // Only 1 strong reference in registry + 3 clones = 4 total
    assert_eq!(Arc::strong_count(&arc1), 4);
}

#[test]
fn test_global_registry_access() {
    let _guard = GLOBAL_REG_LOCK.lock().expect("global registry test lock");

    // Clear for test isolation
    GLOBAL_ASSET_REGISTRY
        .clear_for_testing()
        .expect("failed to clear");

    // Test that global registry works
    let def = create_test_def(99);
    let def_id = def.id;
    let arc1 = GLOBAL_ASSET_REGISTRY.insert(def).unwrap();

    let arc2 = GLOBAL_ASSET_REGISTRY
        .get(&def_id)
        .unwrap()
        .expect("definition exists in global registry");
    assert!(Arc::ptr_eq(&arc1, &arc2));

    // Clean up after test
    GLOBAL_ASSET_REGISTRY
        .clear_for_testing()
        .expect("failed to clear");
}

#[test]
fn test_global_registry_singleton() {
    let _guard = GLOBAL_REG_LOCK.lock().expect("global registry test lock");

    // Clear for test isolation
    GLOBAL_ASSET_REGISTRY
        .clear_for_testing()
        .expect("failed to clear");

    // Insert from one access
    let def1 = create_test_def(88);
    let def1_id = def1.id;
    GLOBAL_ASSET_REGISTRY.insert(def1).unwrap();

    // Retrieve from another access (same singleton instance)
    assert!(GLOBAL_ASSET_REGISTRY.contains(&def1_id).unwrap());

    // Clean up after test
    GLOBAL_ASSET_REGISTRY
        .clear_for_testing()
        .expect("failed to clear");
}

#[test]
fn test_clear_for_testing() {
    let registry = create_test_registry();

    // Add some definitions
    for i in 0..5 {
        let def = create_test_def(i);
        registry.insert(def).expect("insert should succeed");
    }

    assert_eq!(registry.len().unwrap(), 5, "should have 5 definitions");
    assert!(
        !registry.is_empty().unwrap(),
        "registry should not be empty"
    );

    // Clear registry
    registry.clear_for_testing().expect("clear should succeed");

    // Verify cleared
    assert_eq!(
        registry.len().unwrap(),
        0,
        "registry should be empty after clear"
    );
    assert!(registry.is_empty().unwrap(), "registry should be empty");
    assert_eq!(
        registry.get_version().unwrap(),
        0,
        "version should be reset to 0"
    );

    // Verify can insert after clearing
    let def = create_test_def(42);
    let def_id = def.id;
    registry
        .insert(def)
        .expect("insert after clear should work");
    assert_eq!(
        registry.len().unwrap(),
        1,
        "should have 1 definition after re-insert"
    );
    assert!(
        registry.get(&def_id).unwrap().is_some(),
        "definition should be retrievable"
    );
}

#[test]
fn test_load_from_yaml_config() {
    // Path to test YAML config
    let config_path = Path::new(crate::config_paths::DEVNET_ASSETS_CONFIG_REL);

    // Skip test if file doesn't exist (CI environment)
    if !config_path.exists() {
        return;
    }

    // Load registry from YAML
    let registry = load_test_config(config_path).expect("failed to load assets config");

    // Should have loaded multiple assets (z00z, zUSD, zNFT, void)
    assert!(
        registry.len().unwrap() >= 3,
        "registry should contain at least 3 assets from YAML, got {}",
        registry.len().unwrap()
    );
}

#[test]
fn test_yaml_flags_parsing() {
    let config_path = Path::new(crate::config_paths::DEVNET_ASSETS_CONFIG_REL);
    if !config_path.exists() {
        return;
    }

    let registry = load_test_config(config_path).expect("failed to load assets config");

    // Find z00z coin asset (should have is_gas=true, is_fungible=true)
    let z00z_assets: Vec<_> = registry
        .defs_read()
        .unwrap()
        .values()
        .filter(|def| def.symbol == "Z00Z")
        .cloned()
        .collect();

    assert_eq!(z00z_assets.len(), 1, "should find exactly one Z00Z asset");
    let z00z = &z00z_assets[0];

    assert!(
        z00z.is_gas(),
        "Z00Z should be gas asset (is_gas=true in config)"
    );
    assert!(
        z00z.is_fungible(),
        "Z00Z should be fungible (is_fungible=true in config)"
    );

    // Find zNFT (should have is_fungible=false)
    let nft_assets: Vec<_> = registry
        .defs_read()
        .unwrap()
        .values()
        .filter(|def| def.symbol == "zNFT")
        .cloned()
        .collect();

    assert_eq!(nft_assets.len(), 1, "should find exactly one zNFT asset");
    let nft = &nft_assets[0];

    assert!(
        !nft.is_fungible(),
        "zNFT should be non-fungible (is_fungible=false in config)"
    );
    assert!(!nft.is_gas(), "zNFT should not be gas asset");
}

#[test]
fn test_yaml_snapshot_versioning() {
    let config_path = Path::new(crate::config_paths::DEVNET_ASSETS_CONFIG_REL);
    if !config_path.exists() {
        return;
    }

    // Load registry from YAML (acts as "initial validator snapshot")
    let registry = load_test_config(config_path).expect("failed to load assets config");

    // Create snapshot from loaded registry
    let snapshot = registry.create_snapshot().unwrap();
    let current_version = registry.get_version().unwrap();

    // Verify snapshot has version info
    assert!(
        snapshot.version.version > 0,
        "snapshot version should be > 0"
    );
    assert_ne!(
        [0u8; 32], snapshot.version.hash,
        "snapshot hash should be non-zero"
    );
    assert!(
        snapshot.version.timestamp > 0,
        "snapshot timestamp should be > 0"
    );

    // Verify all definitions are in snapshot
    assert_eq!(
        snapshot.definitions.len(),
        registry.len().unwrap(),
        "snapshot should contain all definitions"
    );

    // Verify hash is consistent
    let snapshot2 = registry.create_snapshot().unwrap();
    assert_eq!(
        snapshot.version.hash, snapshot2.version.hash,
        "consecutive snapshots should have same hash for same registry state"
    );
    assert_eq!(
        snapshot.version.version, current_version,
        "snapshot version must reflect current registry version"
    );
    assert_eq!(
        snapshot2.version.version, current_version,
        "repeated snapshot export must not advance registry version"
    );
}

#[test]
fn test_snapshot_roundtrip_via_update() {
    let config_path = Path::new(crate::config_paths::DEVNET_ASSETS_CONFIG_REL);
    if !config_path.exists() {
        return;
    }

    // Load registry from YAML
    let registry1 = load_test_config(config_path).expect("failed to load initial config");

    let snapshot = registry1.create_snapshot().unwrap();
    let asset_count = snapshot.definitions.len();

    // Create new registry and update from snapshot
    let registry2 = create_test_registry();
    registry2
        .update_from_snapshot(snapshot)
        .expect("failed to update from snapshot");

    // Verify both registries have same assets
    assert_eq!(
        registry2.len().unwrap(),
        asset_count,
        "registry2 should have same number of assets after update"
    );

    // Verify IDs match
    let ids1: Vec<[u8; 32]> = {
        let defs = registry1.defs_read().unwrap();
        let mut ids: Vec<_> = defs.keys().copied().collect();
        ids.sort();
        ids
    };

    let ids2: Vec<[u8; 32]> = {
        let defs = registry2.defs_read().unwrap();
        let mut ids: Vec<_> = defs.keys().copied().collect();
        ids.sort();
        ids
    };

    assert_eq!(
        ids1, ids2,
        "registries should have identical asset IDs after snapshot update"
    );
}

#[test]
fn test_downgrade_prevention() {
    use crate::assets::snapshot::{RegistrySnapshot, RegistryVersion};

    // Create initial registry with version 0 (empty)
    let registry = create_test_registry();
    assert_eq!(
        registry.get_version().unwrap(),
        0,
        "empty registry should start at version 0"
    );

    // Create a snapshot for version 5 with proper hash
    let empty_defs: Vec<DefinitionWire> = vec![];
    let computed_hash = RegistryVersion::compute_hash(5, &empty_defs);

    let snapshot_5 = RegistrySnapshot {
        version: RegistryVersion {
            version: 5,
            hash: computed_hash, // Use real hash for empty list
            timestamp: 0,
        },
        definitions: vec![],
    };

    // Apply version 5
    registry
        .update_from_snapshot(snapshot_5)
        .expect("should accept version 5");
    assert_eq!(
        registry.get_version().unwrap(),
        5,
        "version should be 5 after update"
    );

    // Try to apply version 1 (downgrade attempt)
    let downgrade_snapshot = RegistrySnapshot {
        version: RegistryVersion {
            version: 1,
            hash: [42u8; 32],
            timestamp: 0,
        },
        definitions: vec![],
    };

    let result = registry.update_from_snapshot(downgrade_snapshot);
    assert!(result.is_err(), "downgrade to v1 should be rejected");
    assert!(
        result.unwrap_err().to_string().contains("Downgrade"),
        "error message should mention downgrade"
    );

    // Verify version hasn't changed
    assert_eq!(
        registry.get_version().unwrap(),
        5,
        "version should still be 5 after rejected downgrade"
    );

    // Try to apply same version (should be rejected)
    let duplicate_snapshot = RegistrySnapshot {
        version: RegistryVersion {
            version: 5,
            hash: [42u8; 32],
            timestamp: 1,
        },
        definitions: vec![],
    };

    let result = registry.update_from_snapshot(duplicate_snapshot);
    assert!(
        result.is_err(),
        "same version update should be rejected (not strictly newer)"
    );

    // Version 6 should work
    let final_snapshot = RegistrySnapshot {
        version: RegistryVersion {
            version: 6,
            hash: RegistryVersion::compute_hash(6, &empty_defs),
            timestamp: 2,
        },
        definitions: vec![],
    };

    registry
        .update_from_snapshot(final_snapshot)
        .expect("should accept version 6");
    assert_eq!(
        registry.get_version().unwrap(),
        6,
        "version should be 6 after valid update"
    );
}

#[test]
fn test_snapshot_rejects_noncanonical_definition() {
    use crate::assets::snapshot::{RegistrySnapshot, RegistryVersion};
    use crate::assets::wire::DefinitionWire;

    let registry = create_test_registry();
    let def = AssetDefinition::new(
        [0u8; 32],
        AssetClass::Coin,
        "Snapshot Coin".into(),
        "SNP".into(),
        8,
        10,
        100,
        "snapshot.local".into(),
        1,
        1,
        0,
        None,
    )
    .expect("definition");

    let mut wire = DefinitionWire::from(&def);
    wire.id[0] ^= 0x55;
    let snapshot_5 = RegistrySnapshot {
        version: RegistryVersion {
            version: 1,
            hash: RegistryVersion::compute_hash(1, &[wire.clone()]),
            timestamp: 0,
        },
        definitions: vec![wire],
    };

    let err = registry
        .update_from_snapshot(snapshot_5)
        .expect_err("noncanonical definition snapshot should be rejected");
    assert!(
        err.to_string().contains("id mismatch"),
        "unexpected error: {err}"
    );
}

#[test]
fn test_snapshot_rejects_stable_id() {
    use crate::assets::snapshot::{RegistrySnapshot, RegistryVersion};
    use crate::assets::wire::DefinitionWire;

    let registry = create_test_registry();
    let def = AssetDefinition::new(
        [0u8; 32],
        AssetClass::Coin,
        "Snapshot Coin".into(),
        "SNP".into(),
        8,
        10,
        100,
        "snapshot.local".into(),
        1,
        1,
        0,
        None,
    )
    .expect("definition");

    let wire = DefinitionWire::from(&def);
    let mut tampered = wire.clone();
    tampered.symbol = "ALT".into();

    let snapshot = RegistrySnapshot {
        version: RegistryVersion {
            version: 1,
            hash: RegistryVersion::compute_hash(1, &[wire]),
            timestamp: 0,
        },
        definitions: vec![tampered],
    };

    let err = registry
        .update_from_snapshot(snapshot)
        .expect_err("stable-id payload drift must fail");
    assert!(
        err.to_string().contains("Snapshot hash mismatch"),
        "unexpected error: {err}"
    );
}

#[test]
fn test_snapshot_rejects_duplicate_ids() {
    use crate::assets::snapshot::{RegistrySnapshot, RegistryVersion};
    use crate::assets::wire::DefinitionWire;

    let registry = create_test_registry();
    let first = AssetDefinition::new(
        [0u8; 32],
        AssetClass::Coin,
        "Snapshot Coin".into(),
        "SNP".into(),
        8,
        10,
        100,
        "snapshot.local".into(),
        1,
        1,
        0,
        None,
    )
    .expect("first definition");
    let second = AssetDefinition::new(
        [0u8; 32],
        AssetClass::Coin,
        "Snapshot Coin Alt".into(),
        "SPA".into(),
        8,
        10,
        100,
        "snapshot.local".into(),
        1,
        1,
        0,
        None,
    )
    .expect("second definition");

    let first_wire = DefinitionWire::from(&first);
    let mut second_wire = DefinitionWire::from(&second);
    second_wire.id = first_wire.id;

    let snapshot = RegistrySnapshot {
        version: RegistryVersion {
            version: 1,
            hash: RegistryVersion::compute_hash(1, &[first_wire.clone(), second_wire.clone()]),
            timestamp: 0,
        },
        definitions: vec![first_wire, second_wire],
    };

    let err = registry
        .update_from_snapshot(snapshot)
        .expect_err("duplicate ids must fail");
    assert!(
        err.to_string().contains("Duplicate snapshot definition id"),
        "unexpected error: {err}"
    );
}

#[test]
fn test_create_snapshot_registry_state() {
    let registry = create_test_registry();
    let def = AssetDefinition {
        id: [9u8; 32],
        class: AssetClass::Coin,
        name: "Snapshot Coin".to_string(),
        symbol: "SNP".to_string(),
        decimals: 8,
        serials: 10,
        nominal: 100,
        domain_name: "snapshot.local".to_string(),
        version: 1,
        crypto_version: 1,
        policy_flags: 0,
        metadata: None,
    };

    registry
        .insert_prechecked(def)
        .expect("prechecked insert should succeed for setup");

    let err = registry
        .create_snapshot()
        .expect_err("noncanonical registry state must not snapshot");
    assert!(
        err.to_string().contains("id mismatch"),
        "unexpected error: {err}"
    );
}

#[test]
fn test_empty_snapshot_roundtrip() {
    let registry = create_test_registry();
    let snapshot = registry.create_snapshot().expect("empty snapshot export");

    assert_eq!(snapshot.version.version, 0);

    let restored = create_test_registry();
    restored
        .update_from_snapshot(snapshot)
        .expect("empty snapshot apply");

    assert_eq!(restored.get_version().unwrap(), 0);
    assert_eq!(restored.len().unwrap(), 0);
}

#[test]
fn test_rejects_version_zero_payload() {
    use crate::assets::snapshot::{RegistrySnapshot, RegistryVersion};

    let registry = create_test_registry();
    let def = AssetDefinition::new(
        [0u8; 32],
        AssetClass::Coin,
        "Snapshot Coin".into(),
        "SNP".into(),
        8,
        10,
        100,
        "snapshot.local".into(),
        1,
        1,
        0,
        None,
    )
    .expect("definition");
    let wire = DefinitionWire::from(&def);
    let snapshot = RegistrySnapshot {
        version: RegistryVersion {
            version: 0,
            hash: RegistryVersion::compute_hash(0, &[wire.clone()]),
            timestamp: 0,
        },
        definitions: vec![wire],
    };

    let err = registry
        .update_from_snapshot(snapshot)
        .expect_err("non-empty version-zero snapshot must fail");
    assert!(err.to_string().contains("Invalid snapshot version 0"));
}

#[test]
fn test_create_snapshot_advance_version() {
    let config_path = Path::new(crate::config_paths::DEVNET_ASSETS_CONFIG_REL);
    if !config_path.exists() {
        return;
    }

    let registry = load_test_config(config_path).expect("failed to load config");
    let version_before = registry.get_version().expect("version before snapshot");

    let snapshot1 = registry.create_snapshot().expect("first snapshot");
    let version_after_first = registry
        .get_version()
        .expect("version after first snapshot");
    let snapshot2 = registry.create_snapshot().expect("second snapshot");
    let version_after_second = registry
        .get_version()
        .expect("version after second snapshot");

    assert_eq!(version_before, version_after_first);
    assert_eq!(version_before, version_after_second);
    assert_eq!(snapshot1.version.version, version_before);
    assert_eq!(snapshot2.version.version, version_before);
}

#[test]
fn test_insert_batch() {
    use crate::assets::AssetClass;

    let registry = create_test_registry();

    // Create multiple definitions
    let def1 = AssetDefinition::new(
        [1u8; 32],
        AssetClass::Coin,
        "Bitcoin".into(),
        "BTC".into(),
        8,
        1000,
        100_000_000,
        "bitcoin.org".into(),
        1,
        1,
        0,
        None,
    )
    .unwrap();

    let def2 = AssetDefinition::new(
        [2u8; 32],
        AssetClass::Token,
        "Ethereum".into(),
        "ETH".into(),
        18,
        2000,
        200_000_000,
        "ethereum.org".into(),
        1,
        1,
        0,
        None,
    )
    .unwrap();

    let def3 = AssetDefinition::new(
        [3u8; 32],
        AssetClass::Nft,
        "CryptoPunk".into(),
        "PUNK".into(),
        0,
        10000,
        1,
        "cryptopunks.app".into(),
        1,
        1,
        0,
        None,
    )
    .unwrap();

    // Test empty batch
    let empty_result = registry.insert_batch(vec![]).unwrap();
    assert_eq!(
        empty_result.len(),
        0,
        "empty batch should return empty result"
    );

    // Insert batch
    let defs = vec![def1.clone(), def2.clone(), def3.clone()];
    let arcs = registry.insert_batch(defs).unwrap();

    assert_eq!(arcs.len(), 3, "should return 3 Arc references");
    assert_eq!(
        registry.len().unwrap(),
        3,
        "registry should contain 3 definitions"
    );
    assert_eq!(registry.get_version().unwrap(), 1);

    // Verify each definition is accessible
    assert!(
        registry.get(&def1.id).unwrap().is_some(),
        "BTC should be in registry"
    );
    assert!(
        registry.get(&def2.id).unwrap().is_some(),
        "ETH should be in registry"
    );
    assert!(
        registry.get(&def3.id).unwrap().is_some(),
        "PUNK should be in registry"
    );

    // Test idempotency: insert same definitions again
    let defs2 = vec![def1.clone(), def2.clone()];
    let arcs2 = registry.insert_batch(defs2).unwrap();

    assert_eq!(arcs2.len(), 2, "should return 2 Arc references");
    assert_eq!(
        registry.len().unwrap(),
        3,
        "registry should still contain 3 definitions"
    );
    assert_eq!(registry.get_version().unwrap(), 1);

    // Verify Arc pointer equality (same instance returned)
    assert!(
        Arc::ptr_eq(&arcs[0], &arcs2[0]),
        "BTC Arc should be the same instance"
    );
    assert!(
        Arc::ptr_eq(&arcs[1], &arcs2[1]),
        "ETH Arc should be the same instance"
    );

    // Test mixed batch: some new, some existing
    let def4 = AssetDefinition::new(
        [4u8; 32],
        AssetClass::Token,
        "Tether".into(),
        "USDT".into(),
        6,
        3000,
        1_000_000,
        "tether.to".into(),
        1,
        1,
        0,
        None,
    )
    .unwrap();

    let mixed_defs = vec![def1, def4.clone()];
    let mixed_arcs = registry.insert_batch(mixed_defs).unwrap();

    assert_eq!(mixed_arcs.len(), 2, "should return 2 Arc references");
    assert_eq!(
        registry.len().unwrap(),
        4,
        "registry should now contain 4 definitions"
    );
    assert_eq!(registry.get_version().unwrap(), 2);

    // First should be existing, second should be new
    assert!(
        Arc::ptr_eq(&arcs[0], &mixed_arcs[0]),
        "BTC Arc should be the same instance"
    );
    assert!(
        registry.get(&def4.id).unwrap().is_some(),
        "USDT should be in registry"
    );
}
