//! Basic codec example demonstrating all three codec implementations.
//!
//! This example shows how to:
//! - Serialize a data structure to different formats (JSON, YAML, Bincode)
//! - Deserialize back from each format
//! - Compare the output formats
//!
//! Run with: `cargo run --package z00z_utils --example basic_codec`

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use z00z_utils::prelude::*;

/// Example data structure with various field types
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
struct AppConfig {
    name: String,
    version: u32,
    port: u16,
    debug: bool,
    features: Vec<String>,
    settings: BTreeMap<String, String>,
}

impl AppConfig {
    fn example() -> Self {
        Self {
            name: "MyApp".to_string(),
            version: 100,
            port: 8080,
            debug: true,
            features: vec!["feature1".to_string(), "feature2".to_string()],
            settings: {
                let mut m = BTreeMap::new();
                m.insert("timeout_ms".to_string(), "5000".to_string());
                m.insert("retry_count".to_string(), "3".to_string());
                m
            },
        }
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Z00Z Utils: Basic Codec Example ===\n");

    let config = AppConfig::example();
    println!("Original struct:");
    println!("{:#?}\n", config);

    // Create codec instances
    let json_codec = z00z_utils::codec::JsonCodec;
    let yaml_codec = z00z_utils::codec::YamlCodec;
    let bincode_codec = z00z_utils::codec::BincodeCodec;

    // Serialize with each codec
    println!("--- JSON Codec ---");
    let json_bytes = json_codec.serialize(&config)?;
    println!("Serialized size: {} bytes", json_bytes.len());
    println!("Format:\n{}\n", String::from_utf8_lossy(&json_bytes));

    // Deserialize from JSON and verify
    let json_deserialized: AppConfig = json_codec.deserialize(&json_bytes)?;
    assert_eq!(config, json_deserialized, "JSON round-trip failed!");
    println!("✓ JSON round-trip successful\n");

    // YAML
    println!("--- YAML Codec ---");
    let yaml_bytes = yaml_codec.serialize(&config)?;
    println!("Serialized size: {} bytes", yaml_bytes.len());
    println!("Format:\n{}\n", String::from_utf8_lossy(&yaml_bytes));

    let yaml_deserialized: AppConfig = yaml_codec.deserialize(&yaml_bytes)?;
    assert_eq!(config, yaml_deserialized, "YAML round-trip failed!");
    println!("✓ YAML round-trip successful\n");

    // Bincode
    println!("--- Bincode Codec ---");
    let bincode_bytes = bincode_codec.serialize(&config)?;
    println!("Serialized size: {} bytes", bincode_bytes.len());
    println!("Format: (binary, not displayable as text)\n");

    let bincode_deserialized: AppConfig = bincode_codec.deserialize(&bincode_bytes)?;
    assert_eq!(config, bincode_deserialized, "Bincode round-trip failed!");
    println!("✓ Bincode round-trip successful\n");

    // Compare sizes
    println!("--- Codec Size Comparison ---");
    println!("JSON:    {} bytes", json_bytes.len());
    println!("YAML:    {} bytes", yaml_bytes.len());
    println!("Bincode: {} bytes", bincode_bytes.len());
    println!(
        "\nSmallest: {} bytes ({}% reduction from JSON)",
        bincode_bytes.len(),
        100 - (bincode_bytes.len() as f64 / json_bytes.len() as f64 * 100.0) as u32
    );

    // Interoperability test: serialize with JSON, deserialize with JSON
    println!("\n--- Interoperability Test ---");
    let json_serialized = json_codec.serialize(&config)?;
    let cross_deserialized: AppConfig = json_codec.deserialize(&json_serialized)?;
    assert_eq!(config, cross_deserialized);
    println!("✓ All codecs correctly round-trip data!");

    println!("\n=== Example Completed Successfully ===");
    Ok(())
}
