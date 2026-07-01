//! # Genesis Assets Extractor CLI
//!
//! Extracts specific assets from genesis output files based on filters.
//!
//! ## Usage
//!
//! ```bash
//! # Extract all coins
//! cargo run --bin assets_extractor_cli -- \
//!   --input <DIR> --class coin --output coins.json
//!
//! # Extract serial range
//! cargo run --bin assets_extractor_cli -- \
//!   --input <DIR> --serial-range 0-99 --output first_100.bin
//!
//! # Extract with amount filter
//! cargo run --bin assets_extractor_cli -- \
//!   --input <DIR> --amount-min 10000 --output high_value.json
//! ```
//! cargo run --bin assets_extractor_cli -- --input crates/z00z_core/outputs/genesis/genesis_devnet_20251209_061900 --class coin --serial-range 0-99 --output coins_0-99.json --verbose
//!
//! ## Spec Reference
//!
//! - **Specification**: `specs/002-z00z-core-genesis/genesis_spec_cli.md`
//! - **Section**: Tool 2 - Genesis Assets Extractor CLI

use std::ops::RangeInclusive;
use std::path::{Path, PathBuf};
use z00z_core::assets::{Asset, AssetClass};
use z00z_utils::io::{load_bincode_bounded, save_bincode, save_json};

const MAX_ASSET_FILE_SIZE: u64 = 100 * 1024 * 1024;

/// CLI configuration
#[derive(Debug, Clone)]
struct ExtractorConfig {
    input_dir: PathBuf,
    output_file: PathBuf,
    filters: AssetFilters,
    output_format: OutputFormat,
    include_proofs: bool,
    verbose: bool,
}

/// Asset filtering criteria
#[derive(Debug, Clone, Default)]
struct AssetFilters {
    class: Option<AssetClass>,
    serial_range: Option<RangeInclusive<u32>>,
    amount_min: Option<u64>,
    amount_max: Option<u64>,
    definition_symbol: Option<String>,
}

/// Output format options
#[derive(Debug, Clone, Copy)]
enum OutputFormat {
    Json,
    Bincode,
    Csv,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = parse_args();

    println!("📤 === GENESIS ASSETS EXTRACTOR ===\n");

    if config.verbose {
        println!("📂 Source: {}", config.input_dir.display());
        print_filters(&config.filters);
    }

    let assets = extract_assets(&config)?;

    if assets.is_empty() {
        eprintln!("⚠️  No assets matched the specified filters");
        std::process::exit(2);
    }

    save_extracted(&assets, &config)?;

    let output_size = std::fs::metadata(&config.output_file)?.len();

    println!("\n✅ Extraction complete!");
    println!("   Extracted: {} assets", assets.len());
    println!(
        "   Output: {} ({} KB)",
        config.output_file.display(),
        output_size / 1024
    );

    Ok(())
}

mod assets_extractor_cli_ops;
use assets_extractor_cli_ops::{extract_assets, print_filters, save_extracted};

mod assets_extractor_cli_args;
use assets_extractor_cli_args::parse_args;
