#![allow(clippy::useless_conversion)]
// crates/z00z_core/examples/asset_registry_basic.rs
//
// Basic example demonstrating AssetRegistry with real dependencies
//
// This example shows:
// - Creating AssetRegistry with TracingLogger
// - Registering asset definitions
// - Retrieving assets by ID
// - Using the registry in a typical workflow

use std::sync::Arc;
use z00z_core::assets::{
    definition::AssetDefinition, registry::AssetDefinitionRegistry, AssetClass,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🎯 AssetRegistry Basic Example\n");

    // Create test dependencies
    let logger = Arc::new(z00z_utils::logger::NoopLogger);
    let metrics = Arc::new(z00z_utils::metrics::NoopMetrics);
    let time = Arc::new(z00z_utils::time::SystemTimeProvider);

    // Create registry with dependencies
    let registry = AssetDefinitionRegistry::new(logger, metrics, time);

    println!("✅ Created AssetRegistry with NoopLogger\n");

    // Create Z00Z native asset definition
    let z00z_def = AssetDefinition::new(
        [1u8; 32],                  // asset_id
        AssetClass::Coin,           // class
        "Z00Z Native Asset".into(), // name
        "Z00Z".into(),              // symbol
        8,                          // decimals
        50_000,                     // serials
        100_000_000,                // nominal
        "https://z00z.io".into(),   // domain_name
        1,                          // version
        1,                          // crypto_version
        0b0001_0001,                // flags (gas + burnable)
        None,                       // owner_signature
    )?;

    println!("📦 Created Z00Z native asset definition:");
    println!("   Symbol: {}", z00z_def.symbol);
    println!("   Name: {}", z00z_def.name);
    println!("   Decimals: {}", z00z_def.decimals);
    println!("   Serials: {}", z00z_def.serials);
    println!();

    // Insert into registry
    let arc_def = registry.insert(z00z_def)?;
    println!("✅ Inserted Z00Z into registry\n");

    // Create a token definition
    let token_def = AssetDefinition::new(
        [2u8; 32],
        AssetClass::Token,
        "Z00Z Token".into(),
        "ZTKN".into(),
        6,
        30_000,
        1_000_000,
        "https://z00z.io/token".into(),
        1,
        1,
        0b0000_0001, // gas only
        None,
    )?;

    println!("📦 Created ZTKN token definition:");
    println!("   Symbol: {}", token_def.symbol);
    println!("   Class: {:?}", token_def.class);
    println!();

    let _arc_token = registry.insert(token_def)?;
    println!("✅ Inserted ZTKN into registry\n");

    // Retrieve by ID
    let retrieved_z00z = registry.get(&[1u8; 32])?;
    if let Some(def) = retrieved_z00z {
        println!("🔍 Retrieved Z00Z by ID:");
        println!("   Symbol: {}", def.symbol);
        println!("   Pointer equality: {}", Arc::ptr_eq(&arc_def, &def));
        println!();
    }

    // Check registry size
    let size = registry.len()?;
    println!("📊 Registry statistics:");
    println!("   Total assets: {}", size);
    println!();

    // Demonstrate batch insert
    let mut batch = Vec::new();
    for i in 3..=5 {
        let def = AssetDefinition::new(
            [i; 32],
            AssetClass::Token,
            format!("Token {}", i).into(),
            format!("TKN{}", i).into(),
            6,
            25_000,
            500_000,
            "https://z00z.io".into(),
            1,
            1,
            0b0000_0001,
            None,
        )?;
        batch.push(def);
    }

    println!("📦 Inserting batch of {} tokens...", batch.len());
    let batch_results = registry.insert_batch(batch)?;
    println!("✅ Batch insert complete: {} assets", batch_results.len());
    println!();

    // Final statistics
    let final_size = registry.len()?;
    println!("📊 Final registry statistics:");
    println!("   Total assets: {}", final_size);
    println!();

    println!("🎉 Example complete!");
    println!("💡 Check the logs above to see TracingLogger output");

    Ok(())
}
