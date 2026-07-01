// crates/z00z_core/examples/asset_config_loading.rs
//
// Example demonstrating loading a secondary asset registry catalog from YAML
//
// This example shows:
// - Loading asset definitions from YAML catalog files
// - Using LayeredConfig for configuration management
// - Error handling for missing/invalid config files
// - Verifying loaded assets

use std::fs;
use std::path::PathBuf;
use std::sync::Arc;
use tempfile::TempDir;
use z00z_core::assets::registry::AssetDefinitionRegistry;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🎯 Asset Registry Catalog Loading Example\n");

    // Create a temporary directory for test configs
    let temp_dir = TempDir::new()?;
    let config_path = temp_dir.path().join("assets.yaml");

    println!("📁 Creating test registry-catalog file...");
    println!("   Path: {}", config_path.display());
    println!();

    // Create a sample asset registry catalog
    let yaml_content = r#"
assets:
  - id: "z00z_coin"
    name: "Z00Z Coin"
    symbol: "Z00Z"
    class: "Coin"
    policy:
      decimals: 8
      serials: 50000
      nominal: 100000000
      domain_name: "z00z.io"
      gas: true
      fungible: true
      mintable: false
      burnable: true

  - id: "ztkn_asset"
    name: "Z00Z Token"
    symbol: "ZTKN"
    class: "Token"
    policy:
      decimals: 6
      serials: 30000
      nominal: 1000000
      domain_name: "token.z00z.io"
      gas: true
      fungible: true
      mintable: true
      burnable: false

  - id: "znft_asset"
    name: "Z00Z NFT"
    symbol: "ZNFT"
    class: "NFT"
    policy:
      decimals: 0
      serials: 10000
      nominal: 1
      domain_name: "nft.z00z.io"
      gas: false
      fungible: false
      mintable: true
      burnable: false
"#;

    fs::write(&config_path, yaml_content)?;
    println!("✅ Registry-catalog file created\n");

    println!("📦 Loading assets from the registry catalog...");

    // Create test dependencies
    let logger = Arc::new(z00z_utils::logger::NoopLogger);
    let metrics = Arc::new(z00z_utils::metrics::NoopMetrics);
    let time = Arc::new(z00z_utils::time::SystemTimeProvider);

    let registry = AssetDefinitionRegistry::load_catalog_from_yaml(
        &config_path,
        logger.clone(),
        metrics.clone(),
        time.clone(),
    )?;
    println!("✅ Assets loaded successfully\n");

    // Verify loaded assets
    let size = registry.len()?;
    println!("📊 Registry statistics:");
    println!("   Total assets loaded: {}", size);
    println!();

    // Demonstrate error handling with missing file
    println!("🧪 Testing error handling...");
    let missing_path = PathBuf::from("/nonexistent/path/assets.yaml");
    println!("   Attempting to load from: {}", missing_path.display());

    match AssetDefinitionRegistry::load_catalog_from_yaml(
        &missing_path,
        logger.clone(),
        metrics.clone(),
        time.clone(),
    ) {
        Ok(_) => println!("   ⚠️ Unexpected success"),
        Err(e) => println!("   ✅ Expected error: {}", e),
    }
    println!();

    // Demonstrate invalid YAML handling
    println!("🧪 Testing invalid YAML handling...");
    let invalid_yaml_path = temp_dir.path().join("invalid.yaml");
    fs::write(&invalid_yaml_path, "invalid: yaml: content: [")?;
    println!(
        "   Created invalid YAML at: {}",
        invalid_yaml_path.display()
    );

    match AssetDefinitionRegistry::load_catalog_from_yaml(
        &invalid_yaml_path,
        logger.clone(),
        metrics.clone(),
        time.clone(),
    ) {
        Ok(_) => println!("   ⚠️ Unexpected success"),
        Err(e) => println!("   ✅ Expected error: {}", e),
    }
    println!();

    // Show final registry state
    println!("📊 Final registry statistics:");
    println!("   Total assets: {}", registry.len()?);
    println!();

    println!("🎉 Example complete!");
    println!("💡 Key features demonstrated:");
    println!("   - Loading assets from a secondary YAML registry catalog");
    println!("   - Handling missing catalog files");
    println!("   - Handling invalid YAML content");
    println!("   - Using z00z_utils load_yaml() for simplified I/O");

    Ok(())
}
