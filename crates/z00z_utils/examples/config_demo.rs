//! Configuration management example demonstrating multiple config sources.
//!
//! This example shows how to:
//! - Load configuration from YAML files
//! - Override with environment variables
//! - Use LayeredConfig for priority-based configuration
//! - Access nested configuration with dot notation
//! - Type conversion and validation
//!
//! Run with: `cargo run --package z00z_utils --example config_demo`

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use tempfile::TempDir;
use z00z_utils::codec::{Codec, YamlCodec};
use z00z_utils::config::YamlConfig;
use z00z_utils::prelude::*;

/// Example application configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
struct AppSettings {
    server: ServerConfig,
    database: DatabaseConfig,
    features: BTreeMap<String, bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ServerConfig {
    host: String,
    port: u16,
    workers: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct DatabaseConfig {
    url: String,
    pool_size: u16,
    timeout_ms: u32,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Z00Z Utils: Configuration Demo ===\n");

    // Create temporary directory and config file
    let temp_dir = TempDir::new()?;
    let config_path = temp_dir.path().join("config.yaml");

    // Example 1: Create and save a config file
    println!("--- Creating Config File ---");
    let config_content = r#"
server:
  host: 127.0.0.1
  port: 8080
  workers: 4

database:
  url: postgresql://localhost:5432/mydb
  pool_size: 10
  timeout_ms: 5000

features:
  auth_enabled: true
  caching_enabled: true
  debug_mode: false
"#;

    std::fs::write(&config_path, config_content)?;
    println!("✓ Config file created at: {}\n", config_path.display());

    // Example 1a: Deserialize the full config into a strongly-typed struct
    println!("--- Deserializing into AppSettings ---");
    let codec = YamlCodec;
    let settings: AppSettings = codec.deserialize(config_content.as_bytes())?;
    println!(
        "Loaded settings: host={} port={} workers={} db_pool_size={}\n",
        settings.server.host,
        settings.server.port,
        settings.server.workers,
        settings.database.pool_size
    );

    // Example 2: Load config from YAML file
    println!("--- Loading Config from YAML ---");
    let yaml_config = YamlConfig::from_file(config_path.clone())?;

    // Access string values
    let host = yaml_config
        .get_typed::<String>("server.host")?
        .expect("host should exist");
    println!("Server host: {}", host);

    let db_url = yaml_config
        .get_typed::<String>("database.url")?
        .expect("database.url should exist");
    println!("Database URL: {}", db_url);

    // Access numeric values
    let port = yaml_config
        .get_typed::<u16>("server.port")?
        .expect("port should exist");
    println!("Server port: {}\n", port);

    // Example 3: Type conversions
    println!("--- Type Conversions ---");
    let workers = yaml_config
        .get_typed::<u32>("server.workers")?
        .expect("workers should exist");
    println!("Worker threads: {} (type: u32)", workers);

    let pool_size = yaml_config
        .get_typed::<u16>("database.pool_size")?
        .expect("pool_size should exist");
    println!("DB pool size: {} (type: u16)", pool_size);

    let timeout = yaml_config
        .get_typed::<u32>("database.timeout_ms")?
        .expect("timeout should exist");
    println!("Timeout: {}ms (type: u32)\n", timeout);

    // Example 4: Boolean values
    println!("--- Boolean Configuration ---");
    let auth_enabled = yaml_config
        .get_typed::<bool>("features.auth_enabled")?
        .expect("auth_enabled should exist");
    println!("Auth enabled: {}", auth_enabled);

    let debug_mode = yaml_config
        .get_typed::<bool>("features.debug_mode")?
        .expect("debug_mode should exist");
    println!("Debug mode: {}\n", debug_mode);

    // Example 5: Handling missing keys
    println!("--- Missing Key Handling ---");
    let missing_key = yaml_config.get_typed::<String>("nonexistent.key")?;
    match missing_key {
        Some(value) => println!("Found: {}", value),
        None => println!("✓ Correctly returned None for missing key\n"),
    }

    // Example 6: Environment lookup semantics
    println!("--- Environment Variable Override ---");
    println!("LayeredConfig checks the environment first with the same key string.");
    println!("Platform environment semantics still apply, so services should prefer explicit env adapters for nested keys.\n");

    // Example 7: Layered configuration (same key string env lookup > YAML > none)
    println!("--- Layered Configuration ---");
    let layered = LayeredConfig::with_yaml(config_path)?;

    // This will come from YAML (env not set)
    let host = layered
        .get_typed::<String>("server.host")?
        .expect("host should exist");
    println!("Host (layered): {} (from YAML)", host);
    println!("(Environment lookup uses the same key string before YAML fallback)");

    println!("\n--- Optional YAML Policy ---");
    let optional = LayeredConfig::with_optional_yaml(temp_dir.path().join("missing.yaml"))?;
    let missing = optional.get("server.host")?;
    assert_eq!(missing, None);
    println!(
        "Optional YAML path downgrades only missing files; malformed YAML still fails closed."
    );

    // Example 8: Error handling
    println!("--- Error Handling ---");
    println!("✓ Configuration loaded successfully!");
    println!("✓ All type conversions succeeded!");
    println!("✓ Missing keys handled gracefully!");
    println!("✓ Environment lookup semantics are explicit!");
    println!("✓ Optional YAML downgrade is explicit, not implicit!");

    println!("\n=== Example Completed Successfully ===");
    Ok(())
}
