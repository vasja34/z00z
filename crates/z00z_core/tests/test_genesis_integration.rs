//! Integration Tests for Genesis Module
//!
//! Full end-to-end tests for genesis generation workflow.

use crate::genesis::helpers::*;
use std::fs;
use std::path::Path;
use tempfile::TempDir;
use z00z_core::assets::AssetWire;
use z00z_core::config_paths::DEVNET_GENESIS_CONFIG_REL;
use z00z_core::genesis::{
    create_asset_definition, generate_all_genesis_assets,
    genesis_config::load_genesis_config,
    serde::{export_genesis_assets, extract_genesis_assets, load_genesis_assets},
    ChainType, GenesisSeed,
};
use z00z_utils::codec::{BincodeCodec, Codec, JsonCodec};

#[test]
fn test_full_genesis_generation_flow() {
    let (logger, metrics) = create_test_observability();

    let temp_dir = TempDir::new().unwrap();
    let output_path = temp_dir.path().join("outputs");

    // Create test config with custom output path
    let mut config = create_test_config();
    config.outputs.assets_export_path = output_path.to_str().unwrap().to_string();

    let genesis_seed = GenesisSeed::from_config(&config).unwrap();

    // Create asset definitions
    let mut definitions = Vec::new();
    for asset_cfg in &config.assets {
        let definition =
            create_asset_definition(asset_cfg, genesis_seed.as_bytes(), ChainType::Devnet).unwrap();
        definitions.push(definition);
    }

    // Generate assets
    let accumulator = generate_all_genesis_assets(
        &definitions,
        genesis_seed.as_bytes(),
        ChainType::Devnet,
        logger.clone(),
        metrics.clone(),
    )
    .unwrap();

    // Verify generation
    assert_eq!(
        accumulator.total_count(),
        10,
        "Should generate 10 assets (serials from config)"
    );

    // Export to disk
    fs::create_dir_all(&output_path).unwrap();

    for (asset_cfg, definition) in config.assets.iter().zip(definitions.iter()) {
        let assets = accumulator.get_by_class(definition.class);
        if !assets.is_empty() {
            export_genesis_assets(assets, &asset_cfg.symbol, &config.outputs).unwrap();
        }
    }

    // Verify files created
    let json_path = output_path.join("genesis_TST.json");
    let bin_path = output_path.join("genesis_TST.bin");

    assert!(json_path.exists(), "JSON file should be created");
    assert!(bin_path.exists(), "Binary file should be created");

    // Load and verify JSON
    let json_data = fs::read_to_string(&json_path).unwrap();
    let json_codec = JsonCodec;
    let assets: Vec<AssetWire> = json_codec.deserialize(json_data.as_bytes()).unwrap();
    assert_eq!(assets.len(), 10);

    // Load and verify Bincode
    let bin_data = fs::read(&bin_path).unwrap();
    let bin_codec = BincodeCodec;
    let assets: Vec<AssetWire> = bin_codec.deserialize(&bin_data).unwrap();
    assert_eq!(assets.len(), 10);
}

#[test]
fn test_extract_and_load_roundtrip() {
    let temp_dir = TempDir::new().unwrap();
    let accumulator = generate_test_accumulator();

    // Extract to separate files
    let paths = extract_genesis_assets(&accumulator, temp_dir.path()).unwrap();

    assert_eq!(paths.len(), 4, "Should create 4 class files");

    // Verify files exist
    assert!(temp_dir.path().join("genesis_coins.json").exists());
    assert!(temp_dir.path().join("genesis_tokens.json").exists());
    assert!(temp_dir.path().join("genesis_nfts.json").exists());
    assert!(temp_dir.path().join("genesis_voids.json").exists());

    // Load back
    let loaded = load_genesis_assets(temp_dir.path()).unwrap();

    assert_eq!(accumulator.total_count(), loaded.total_count());
    assert_eq!(accumulator.coins.len(), loaded.coins.len());
}

#[test]
fn test_json_binary_export_size() {
    let accumulator = generate_test_accumulator();

    let json_codec = JsonCodec;
    let json = json_codec.serialize(&accumulator).unwrap();
    let bin_codec = BincodeCodec;
    let binary = bin_codec.serialize(&accumulator).unwrap();

    // Binary should be more compact than JSON
    assert!(
        binary.len() < json.len(),
        "Binary export should be more compact: {} bytes vs {} bytes",
        binary.len(),
        json.len()
    );
}

#[test]
fn test_asset_multi_generation_export() {
    let (logger, metrics) = create_test_observability();

    let temp_dir = TempDir::new().unwrap();
    let mut config = create_multi_asset_test_config();
    config.outputs.assets_export_path = temp_dir.path().to_str().unwrap().to_string();

    let genesis_seed = GenesisSeed::from_config(&config).unwrap();

    // Create asset definitions
    let mut definitions = Vec::new();
    for asset_cfg in &config.assets {
        let definition =
            create_asset_definition(asset_cfg, genesis_seed.as_bytes(), ChainType::Devnet).unwrap();
        definitions.push(definition);
    }

    // Generate assets
    let accumulator = generate_all_genesis_assets(
        &definitions,
        genesis_seed.as_bytes(),
        ChainType::Devnet,
        logger.clone(),
        metrics.clone(),
    )
    .unwrap();

    // Verify mixed asset types
    assert!(!accumulator.coins.is_empty(), "Should have assets");
    assert!(!accumulator.tokens.is_empty(), "Should have tokens");

    // Export
    fs::create_dir_all(temp_dir.path()).unwrap();

    for (asset_cfg, definition) in config.assets.iter().zip(definitions.iter()) {
        let assets = accumulator.get_by_class(definition.class);
        if !assets.is_empty() {
            export_genesis_assets(assets, &asset_cfg.symbol, &config.outputs).unwrap();
        }
    }

    // Verify both files exist
    assert!(temp_dir.path().join("genesis_TST.json").exists());
    assert!(temp_dir.path().join("genesis_TTK.json").exists());
}

#[test]
fn test_genesis_config_yaml_file() {
    // Test loading actual devnet config file
    let config_path = DEVNET_GENESIS_CONFIG_REL;

    if Path::new(config_path).exists() {
        let config = load_genesis_config(config_path).unwrap();

        assert_eq!(config.chain.chain_type, "devnet");
        assert!(!config.assets.is_empty(), "Config should have assets");
    }
}

// Note: atomic_write is a private helper function, tested indirectly through export_genesis_assets
// #[test]
// fn test_write_cleans_up() {
//     let temp_dir = TempDir::new().unwrap();
//     let file_path = temp_dir.path().join("test.dat");
//     let data = b"test data";
//
//     atomic_write(&file_path, data).unwrap();
//
//     assert!(file_path.exists());
//     let content = fs::read(&file_path).unwrap();
//     assert_eq!(content, data);
//
//     let entries: Vec<_> = fs::read_dir(temp_dir.path())
//         .unwrap()
//         .map(|e| e.unwrap().file_name())
//         .collect();
//
//     assert_eq!(entries.len(), 1, "Should only have final file, no .tmp");
// }

#[test]
fn test_export_creates_both_formats() {
    let temp_dir = TempDir::new().unwrap();
    let assets = vec![generate_test_asset()];

    let mut config = create_test_config();
    config.outputs.assets_export_path = temp_dir.path().to_str().unwrap().to_string();

    fs::create_dir_all(temp_dir.path()).unwrap();

    export_genesis_assets(&assets, "TEST", &config.outputs).unwrap();

    // Both JSON and binary should exist
    assert!(temp_dir.path().join("genesis_TEST.json").exists());
    assert!(temp_dir.path().join("genesis_TEST.bin").exists());
}
