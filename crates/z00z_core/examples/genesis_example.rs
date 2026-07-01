//! Genesis Generation Example
//!
//! Demonstrates full genesis generation workflow:
//! - Load YAML config
//! - Generate assets with nested parallelism
//! - Verify all proofs
//! - Export to JSON and Bincode

use z00z_core::config_paths::devnet_genesis_path;
use z00z_core::genesis::run_genesis;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Genesis Generation Example");
    println!("==========================\n");

    // Use the canonical devnet config from configs.
    let config_path = devnet_genesis_path();

    match run_genesis(config_path.to_str().ok_or("utf8 config path")?, None) {
        Ok(()) => {
            println!("\n✅ Example completed successfully!");
            println!(
                "   Check the configured assets export path in {}",
                config_path.display()
            );
            Ok(())
        }
        Err(e) => {
            eprintln!("\n❌ Error: {}", e);
            Err(Box::new(e))
        }
    }
}
