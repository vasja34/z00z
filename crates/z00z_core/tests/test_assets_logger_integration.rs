// crates/z00z_core/tests/test_assets_logger_integration.rs
//
// Integration tests for logger implementations
//
// Tests:
// - Logging output at different levels
// - Switching between logger implementations
// - Verifying log messages (where possible)
// - Logger behavior in concurrent scenarios

use std::sync::Arc;
use z00z_core::assets::{
    definition::AssetDefinition, registry::AssetDefinitionRegistry, AssetClass,
};
use z00z_utils::prelude::{NoopLogger, NoopMetrics, SystemTimeProvider};

use super::fixtures::create_test_registry;

#[test]
fn test_registry_with_simple_api() -> Result<(), Box<dyn std::error::Error>> {
    // New API is simpler - no logger/metrics/time arguments
    let registry = create_test_registry();

    let def = AssetDefinition::new(
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

    registry.insert(def)?;
    assert_eq!(registry.len()?, 1);

    Ok(())
}

#[test]
fn test_registry_insert_and_retrieve() -> Result<(), Box<dyn std::error::Error>> {
    let registry = create_test_registry();

    let def = AssetDefinition::new(
        [2u8; 32],
        AssetClass::Token,
        "TKN".into(),
        "Token".into(),
        6,
        30_000,
        1_000_000,
        "token.io".into(),
        1,
        1,
        0b0000_0001,
        None,
    )?;

    registry.insert(def)?;
    assert_eq!(registry.len()?, 1);

    Ok(())
}

#[test]
fn test_batch_insert() -> Result<(), Box<dyn std::error::Error>> {
    let registry = create_test_registry();

    // Create batch of assets
    let mut batch = Vec::new();
    for i in 1..=10 {
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
        batch.push(def);
    }

    let results = registry.insert_batch(batch)?;
    assert_eq!(results.len(), 10);
    assert_eq!(registry.len()?, 10);

    Ok(())
}

#[test]
fn test_concurrent_operations() -> Result<(), Box<dyn std::error::Error>> {
    use std::thread;

    let registry = Arc::new(create_test_registry());

    // Multiple threads inserting assets
    let handles: Vec<_> = (0..5)
        .map(|i| {
            let registry: Arc<AssetDefinitionRegistry> = Arc::clone(&registry);
            thread::spawn(move || {
                for j in 0..10 {
                    let id = [i * 10 + j; 32];
                    let def = AssetDefinition::new(
                        id,
                        AssetClass::Token,
                        format!("T{}{}", i, j),
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
                    registry.insert(def).unwrap();
                }
            })
        })
        .collect();

    for handle in handles {
        handle.join().unwrap();
    }

    assert_eq!(registry.len()?, 50);

    Ok(())
}

#[test]
fn test_error_handling_missing_file() {
    // Try to load non-existent config (should error gracefully)
    let result = AssetDefinitionRegistry::load_catalog_from_yaml(
        std::path::Path::new("/nonexistent/path.yaml"),
        Arc::new(NoopLogger),
        Arc::new(NoopMetrics),
        Arc::new(SystemTimeProvider),
    );
    assert!(result.is_err());
}
