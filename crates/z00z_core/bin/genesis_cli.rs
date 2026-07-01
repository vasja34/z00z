//! # Genesis CLI
//!
//! Command-line interface for the canonical Z00Z genesis pipeline.
//!
//! ## Usage
//!
//! ```bash
//! # Generate devnet genesis
//! time cargo run --release --bin genesis_cli -- --config configs/devnet_genesis_config.yaml --verbose
//! ```
//!
//! ## Spec Reference
//!
//! - **Lines 850-940**: CLI interface specification (genesis_spec_release_3.md)
//! - **Task 3.5**: CLI implementation checklist (genesis_tasks_release_3.md)

use std::{env, process};
use z00z_core::genesis::run_genesis;

/// CLI configuration structure
///
/// **Spec Reference**: Lines 865-869 (genesis_spec_release_3.md)
#[derive(Debug, Clone)]
pub struct CliConfig {
    /// Path to genesis config YAML file
    pub config_file: String,
    /// Enable verbose logging
    pub verbose: bool,
}

/// Main entry point for the canonical genesis CLI
///
/// **Spec Reference**: Lines 871-883 (genesis_spec_release_3.md)
///
/// ## Example
///
/// ```bash
/// cargo run --bin genesis_cli -- --config configs/devnet_genesis_config.yaml
/// ```
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = parse_args();

    if config.verbose {
        println!("🚀 Starting genesis generation...");
        println!("📄 Config file: {}", config.config_file);
    }

    // Build CLI command string for snapshot archive
    let cli_command = format!(
        "cargo run --release --bin genesis_cli -- --config {}{}",
        config.config_file,
        if config.verbose { " --verbose" } else { "" }
    );

    // Run genesis generation with command tracking
    match run_genesis(&config.config_file, Some(&cli_command)) {
        Ok(()) => {
            if config.verbose {
                println!("✅ Genesis generation completed successfully!");
            }
            Ok(())
        }
        Err(e) => {
            eprintln!("❌ Genesis generation failed: {}", e);
            Err(Box::new(e))
        }
    }
}

/// Parse command-line arguments
///
/// **Spec Reference**: Lines 885-893 (genesis_spec_release_3.md)
///
/// ## Arguments
///
/// - `--config <FILE>`: Path to genesis config YAML (required)
/// - `--verbose`: Enable detailed logging (optional)
fn parse_args() -> CliConfig {
    let args: Vec<String> = env::args().collect();

    let mut config_file: Option<String> = None;
    let mut verbose = false;

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--config" => {
                if i + 1 < args.len() {
                    config_file = Some(args[i + 1].clone());
                    i += 2;
                } else {
                    eprintln!("Error: --config requires a file path");
                    print_usage();
                    process::exit(1);
                }
            }
            "--verbose" => {
                verbose = true;
                i += 1;
            }
            "--help" | "-h" => {
                print_usage();
                process::exit(0);
            }
            _ => {
                eprintln!("Error: Unknown argument: {}", args[i]);
                print_usage();
                process::exit(1);
            }
        }
    }

    if config_file.is_none() {
        eprintln!("Error: --config is required");
        print_usage();
        process::exit(1);
    }

    CliConfig {
        config_file: config_file.unwrap(),
        verbose,
    }
}

/// Print CLI usage information
fn print_usage() {
    println!("Z00Z Genesis CLI");
    println!();
    println!("USAGE:");
    println!("    genesis_cli --config <FILE> [--verbose]");
    println!();
    println!("OPTIONS:");
    println!("    --config <FILE>    Path to genesis config YAML (required)");
    println!("    --verbose          Enable detailed logging");
    println!("    --help, -h         Show this help message");
    println!();
    println!("EXAMPLES:");
    println!("    genesis_cli --config configs/devnet_genesis_config.yaml");
    println!("    genesis_cli --config configs/devnet_genesis_config.yaml --verbose");
}
