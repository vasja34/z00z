// crates/z00z_core/bin/assets_generation_cli.rs
//
// End-to-End Assets CLI Example
//
// Demonstrates:
// - YAML config parsing directly into AssetDefinition (built-in serde)
// - Parallel asset generation with real cryptography
// - Full cryptographic verification (commitments, proofs, signatures)
// - JSON + Bincode serialization (using built-in Asset Serialize/Deserialize)
// - Comprehensive reporting with detailed statistics
//
// Usage:
//   cargo run --release --bin assets_generation_cli
//   cargo run --release --bin assets_generation_cli -- --format bincode
//   cargo run --release --bin assets_generation_cli -- --verbose --threads 8
//   cargo run --release --bin assets_generation_cli -- --config configs/devnet_assets_config.yaml --format bincode

// ============================================================================
// Imports
// ============================================================================

// External crates
use clap::{Parser, ValueEnum};
use rayon::prelude::*;

// Tari crypto
use z00z_crypto::expert::hash_domain;
use z00z_crypto::{create_commitment, DomainHasher, Z00ZScalar};

// Z00Z utils - RNG abstraction
use z00z_utils::codec::{BincodeCodec, Codec, JsonCodec};
use z00z_utils::rng::DeterministicRngProvider;
use z00z_utils::time::{
    format_system_time_utc, format_unix_timestamp_milliseconds_compact, SystemTimeProvider,
    TimeProvider,
};

// Standard library
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use z00z_utils::time::Instant;

// Z00Z core
use z00z_core::assets::{
    assets::{Asset, AssetClass, AssetError},
    definition::AssetDefinition,
    nonce::Nonce,
    registry::AssetDefinitionRegistry,
};
use z00z_core::config_paths::DEVNET_ASSETS_CONFIG_REL;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
struct AssetExportEntry {
    pub definition: AssetDefinition,
    pub asset_id: [u8; 32],
    pub serial_id: u32,
    pub amount: u64,
    pub commitment: z00z_core::assets::Commitment,
    pub range_proof: Option<z00z_core::assets::RangeProof>,
    pub nonce: [u8; 32],
    pub lock_height: Option<u64>,
    pub owner_pub: Option<z00z_crypto::Z00ZRistrettoPoint>,
    pub owner_signature: Option<z00z_core::assets::KernelSignature>,
    pub is_frozen: bool,
    pub is_slashed: bool,
    pub is_burned: bool,
}

impl AssetExportEntry {
    pub fn from_asset(asset: &Asset) -> Self {
        Self {
            definition: (*asset.definition).clone(),
            asset_id: asset.asset_id(),
            serial_id: asset.serial_id,
            amount: asset.amount,
            commitment: asset.commitment.clone(),
            range_proof: asset.range_proof.clone(),
            nonce: asset.nonce,
            lock_height: asset.lock_height,
            owner_pub: asset.owner_pub.clone(),
            owner_signature: asset.owner_signature.clone(),
            is_frozen: asset.is_frozen,
            is_slashed: asset.is_slashed,
            is_burned: asset.is_burned,
        }
    }
}

// Z00Z crypto - public API
use z00z_crypto::verify_range_proof;

// ============================================================================
// Helper Structures
// ============================================================================

/// Asset with its cryptographic secrets (for verification)
struct AssetWithSecrets {
    asset: Asset,
    blinding_factor: Z00ZScalar,
}

impl std::fmt::Debug for AssetWithSecrets {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AssetWithSecrets")
            .field("asset", &self.asset)
            .field("blinding_factor", &"<redacted>")
            .finish()
    }
}

impl Clone for AssetWithSecrets {
    fn clone(&self) -> Self {
        Self {
            asset: self.asset.clone(),
            blinding_factor: self.blinding_factor.dangerous_clone(),
        }
    }
}

// ============================================================================
// CLI Arguments
// ============================================================================

/// Serialization format for asset export
#[derive(Debug, Clone, Copy, ValueEnum)]
enum Format {
    /// JSON format (human-readable) - uses Debug formatting
    Json,
    /// Bincode format (compact binary) - uses serde bincode
    Bincode,
}

/// Z00Z Asset Generation CLI
///
/// Generate assets from YAML config with full cryptographic verification
#[derive(Parser, Debug)]
#[command(name = "assets_generation_cli")]
#[command(about = "Generate Z00Z assets with real cryptography", long_about = None)]
struct Args {
    /// Serialization format: json or bincode
    #[arg(short, long, value_enum, default_value = "json")]
    format: Format,

    /// Path to YAML config file
    #[arg(short, long, default_value = DEVNET_ASSETS_CONFIG_REL)]
    config: PathBuf,

    /// Output directory
    #[arg(short, long, default_value = "crates/z00z_core/outputs/assets")]
    output: PathBuf,

    /// Number of parallel threads (default: number of CPU cores)
    #[arg(short, long)]
    threads: Option<usize>,

    /// Enable verbose output
    #[arg(short, long)]
    verbose: bool,
}

// ============================================================================
// Statistics Tracking
// ============================================================================

#[derive(Debug, Clone)]
struct GenerationStats {
    total_assets: usize,
    generation_time_ms: u128,
    per_class_counts: BTreeMap<String, usize>,
    per_class_amounts: BTreeMap<String, u64>,
}

#[derive(Debug, Clone)]
struct VerificationStats {
    total_verified: usize,
    commitments_valid: usize,
    commitments_invalid: usize,
    range_proofs_present: usize,
    range_proofs_missing: usize,
    signatures_valid: usize,
    signatures_invalid: usize,
    homomorphic_tests_passed: usize,
    homomorphic_tests_failed: usize,
}

#[derive(Debug, Clone)]
struct SerializationStats {
    json_size_bytes: Option<usize>,
    json_time_ms: Option<u128>,
    bincode_size_bytes: Option<usize>,
    bincode_time_ms: Option<u128>,
}

// ============================================================================
// Main Entry Point
// ============================================================================

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    // Print configuration summary
    if args.verbose {
        println!("🚀 Z00Z Asset Generation CLI");
        println!("================================");
        println!("📄 Config file: {}", args.config.display());
        println!("📁 Output directory: {}", args.output.display());
        println!(
            "🧵 Threads: available (CPU cores: {})",
            std::thread::available_parallelism()
                .map(|n| n.get())
                .unwrap_or(0)
        );
        println!("📦 Format: {:?}", args.format);
        println!("================================\n");
    }

    // NOTE: Rayon thread pool initialization removed - caused deadlock
    // Rayon will automatically use all available cores

    // Phase 1: Validation
    println!("✅ Phase 1: Validation");
    validate_inputs(&args)?;
    println!("   ✓ All validations passed\n");

    // Phase 2: Load Configuration
    println!("⚙️  Phase 2: Loading Configuration");
    let registry = load_config(&args)?;
    let definitions = get_definitions_from_registry(&registry);
    println!("   ✓ Loaded {} asset definitions", definitions.len());
    for def in &definitions {
        println!("     - {} ({:?})", def.symbol, def.class);
    }
    println!();

    // Phase 3: Generate Assets
    println!("🔧 Phase 3: Generating Assets (Parallel)");
    let (assets_with_secrets, gen_stats) = generate_assets(&definitions, args.verbose)?;
    println!(
        "   ✓ Generated {} assets in {:.2}s",
        gen_stats.total_assets,
        gen_stats.generation_time_ms as f64 / 1000.0
    );
    println!(
        "   ✓ Throughput: {:.2} assets/sec",
        gen_stats.total_assets as f64 / (gen_stats.generation_time_ms as f64 / 1000.0)
    );
    println!();

    // Phase 4: Cryptographic Verification
    println!("🛡️  Phase 4: Cryptographic Verification");
    let verify_stats = verify_assets(&assets_with_secrets, args.verbose)?;
    println!("   ✓ Verified {} assets", verify_stats.total_verified);
    println!(
        "   ✓ Commitments valid: {}/{}",
        verify_stats.commitments_valid, verify_stats.total_verified
    );
    println!(
        "   ✓ Range proofs present: {}/{}",
        verify_stats.range_proofs_present, verify_stats.total_verified
    );
    println!(
        "   ✓ Signatures valid: {}/{}",
        verify_stats.signatures_valid, verify_stats.total_verified
    );
    if verify_stats.homomorphic_tests_passed > 0 || verify_stats.homomorphic_tests_failed > 0 {
        println!(
            "   ✓ Homomorphic tests passed: {}/{}",
            verify_stats.homomorphic_tests_passed,
            verify_stats.homomorphic_tests_passed + verify_stats.homomorphic_tests_failed
        );
    }
    println!();

    // Phase 5: Serialization
    println!("💾 Phase 5: Serialization");
    // Extract just assets for serialization (without secrets)
    let assets: Vec<Asset> = assets_with_secrets
        .iter()
        .map(|aws| aws.asset.clone())
        .collect();
    let ser_stats = serialize_assets(&assets, &args)?;
    print_serialization_stats(&ser_stats);
    println!();

    // Phase 6: Generate Reports
    println!("📊 Phase 6: Generating Reports");
    let time_provider = SystemTimeProvider;
    let timestamp =
        format_unix_timestamp_milliseconds_compact(time_provider.compat_unix_timestamp_millis());
    generate_reports(
        &args.output,
        &timestamp,
        &gen_stats,
        &verify_stats,
        &ser_stats,
        &assets,
        &args,
    )?;
    println!(
        "   ✓ Reports generated in {}/reports/",
        args.output.display()
    );
    println!();

    println!("🎉 Success! All phases completed.");
    println!("📁 Output directory: {}", args.output.display());

    Ok(())
}

mod assets_generation_cli_phase;
use assets_generation_cli_phase::*;

mod assets_generation_cli_report;
use assets_generation_cli_report::*;
