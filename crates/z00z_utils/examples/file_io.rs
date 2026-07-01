//! File I/O example demonstrating save and load operations with different codecs.
//!
//! This example shows how to:
//! - Save data structures to files in different formats
//! - Automatically create nested directories
//! - Load data back from files
//! - Handle errors gracefully
//!
//! Run with: `cargo run --package z00z_utils --example file_io`

use serde::{Deserialize, Serialize};
use tempfile::TempDir;

/// Example data structure for file operations
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
struct UserProfile {
    id: u64,
    username: String,
    email: String,
    age: u16,
    active: bool,
}

impl UserProfile {
    fn example() -> Self {
        Self {
            id: 12345,
            username: "john_doe".to_string(),
            email: "john@example.com".to_string(),
            age: 28,
            active: true,
        }
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Z00Z Utils: File I/O Example ===\n");

    // Create a temporary directory for demo (in real use, you'd use actual paths)
    let temp_dir = TempDir::new()?;
    let temp_path = temp_dir.path();

    let profile = UserProfile::example();
    println!("Original data:");
    println!("{:#?}\n", profile);

    // Example 1: JSON file operations
    println!("--- JSON File I/O ---");
    let json_path = temp_path.join("data").join("users").join("profile.json");
    println!("Saving to: {}", json_path.display());
    z00z_utils::io::save_json(&json_path, &profile)?;
    println!("✓ File saved (directory auto-created)");

    let json_loaded: UserProfile = z00z_utils::io::load_json(&json_path)?;
    println!("✓ File loaded");
    assert_eq!(profile, json_loaded);
    println!("✓ Data matches original\n");

    // Example 2: YAML file operations
    println!("--- YAML File I/O ---");
    let yaml_path = temp_path.join("config").join("users").join("profile.yaml");
    println!("Saving to: {}", yaml_path.display());
    z00z_utils::io::save_yaml(&yaml_path, &profile)?;
    println!("✓ File saved (directory auto-created)");

    let yaml_loaded: UserProfile = z00z_utils::io::load_yaml(&yaml_path)?;
    println!("✓ File loaded");
    assert_eq!(profile, yaml_loaded);
    println!("✓ Data matches original\n");

    // Example 3: Bincode file operations (binary format)
    println!("--- Bincode File I/O ---");
    let bincode_path = temp_path
        .join("cache")
        .join("users")
        .join("profile.bincode");
    println!("Saving to: {}", bincode_path.display());
    z00z_utils::io::save_bincode(&bincode_path, &profile)?;
    println!("✓ File saved (directory auto-created)");

    let bincode_loaded: UserProfile = z00z_utils::io::load_bincode(&bincode_path)?;
    println!("✓ File loaded");
    assert_eq!(profile, bincode_loaded);
    println!("✓ Data matches original\n");

    // Example 4: Cross-format loading (save with one codec, load with same)
    println!("--- File Content Verification ---");
    let json_content = std::fs::read_to_string(&json_path)?;
    println!("JSON file content:\n{}", json_content);

    let yaml_content = std::fs::read_to_string(&yaml_path)?;
    println!("YAML file content:\n{}", yaml_content);

    // Example 5: File sizes comparison
    println!("--- Format Size Comparison ---");
    let json_size = std::fs::metadata(&json_path)?.len();
    let yaml_size = std::fs::metadata(&yaml_path)?.len();
    let bincode_size = std::fs::metadata(&bincode_path)?.len();

    println!("JSON:    {} bytes", json_size);
    println!("YAML:    {} bytes", yaml_size);
    println!("Bincode: {} bytes", bincode_size);
    println!(
        "Bincode is {:.1}% smaller than JSON",
        100.0 - (bincode_size as f64 / json_size as f64 * 100.0)
    );

    // Example 6: Error handling example (try to load non-existent file)
    println!("\n--- Error Handling Example ---");
    let nonexistent_path = temp_path.join("nonexistent.json");
    match z00z_utils::io::load_json::<UserProfile>(&nonexistent_path) {
        Ok(_) => println!("Unexpected: file should not exist!"),
        Err(e) => {
            println!("✓ Correctly caught error when loading non-existent file:");
            println!("  Error: {}", e);
        }
    }

    // Example 7: Demonstrate atomic write behavior
    println!("\n--- File Overwrite Behavior ---");
    let overwrite_path = temp_path.join("overwrite.json");

    // First write
    let profile1 = UserProfile {
        id: 1,
        username: "user1".to_string(),
        ..profile.clone()
    };
    z00z_utils::io::save_json(&overwrite_path, &profile1)?;
    let loaded1: UserProfile = z00z_utils::io::load_json(&overwrite_path)?;
    println!("First write: id={}", loaded1.id);

    // Overwrite
    let profile2 = UserProfile {
        id: 2,
        username: "user2".to_string(),
        ..profile.clone()
    };
    z00z_utils::io::save_json(&overwrite_path, &profile2)?;
    let loaded2: UserProfile = z00z_utils::io::load_json(&overwrite_path)?;
    println!("After overwrite: id={}", loaded2.id);
    println!("✓ File correctly overwritten with new data");

    println!("\n=== Example Completed Successfully ===");
    Ok(())
}
