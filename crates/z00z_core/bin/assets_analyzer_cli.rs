//! # Genesis Assets Analyzer CLI
//!
//! Analyzes genesis output files and provides comprehensive metadata.
//!
//! ## Usage
//!
//! ```bash
//! # Analyze entire genesis directory
//! cargo run --bin assets_analyzer_cli -- --input <DIR>
//!
//! # Filter by asset class
//! cargo run --bin assets_analyzer_cli -- --input <DIR> --class coin
//! cargo run --bin assets_analyzer_cli -- --input crates/z00z_core/outputs/genesis/genesis_devnet_20251209_061900 --class coin --verbose 2>&1 | head -60
//! argo run --bin assets_analyzer_cli -- --input crates/z00z_core/outputs/genesis/genesis_devnet_20251209_061900 --verbose
//!
//! # Verbose output with detailed statistics
//! cargo run --bin assets_analyzer_cli -- --input <DIR> --verbose
//! ```
//!
//! ## Spec Reference
//!
//! - **Specification**: `specs/002-z00z-core-genesis/genesis_spec_cli.md`
//! - **Section**: Tool 1 - Genesis Assets Analyzer CLI

use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use z00z_core::assets::{Asset, AssetClass};
use z00z_utils::io::{load_bincode_bounded, load_json_bounded};

const MAX_ASSET_FILE_SIZE: u64 = 100 * 1024 * 1024;

/// CLI configuration
#[derive(Debug, Clone)]
struct AnalyzerConfig {
    input_dir: Option<PathBuf>,
    input_files: Vec<PathBuf>,
    filter_class: Option<AssetClass>,
    verbose: bool,
}

/// Asset class statistics
#[derive(Debug, Clone, Default)]
struct ClassStats {
    count: usize,
    total_supply: u64,
    nominal_value: u64,
    symbols: Vec<String>,
    serial_distribution: HashMap<u32, SerialStats>,
}

/// Serial ID statistics
#[derive(Debug, Clone)]
struct SerialStats {
    count: usize,
    amount: u64,
    nominal: u64,
    has_proof: bool,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = parse_args();

    println!("🔍 === GENESIS ASSETS ANALYSIS ===\n");

    let assets = load_all_assets(&config)?;

    if assets.is_empty() {
        eprintln!("⚠️  No assets found in directory");
        std::process::exit(2);
    }

    // Analyze and get structured stats for report generation
    let stats = analyze_and_display(&assets, &config)?;

    // Determine where to write the report: prefer input dir, otherwise parent of first file
    let report_dir = if let Some(ref d) = config.input_dir {
        d.as_path()
    } else {
        config
            .input_files
            .first()
            .and_then(|p| p.parent())
            .map(|p| p as &std::path::Path)
            .unwrap_or(std::path::Path::new("."))
    };

    // Write markdown report into the chosen directory
    write_report(&stats, report_dir)?;

    Ok(())
}

mod assets_analyzer_cli_ops;
use assets_analyzer_cli_ops::{analyze_and_display, load_all_assets, write_report};

mod assets_analyzer_cli_args;
use assets_analyzer_cli_args::parse_args;
