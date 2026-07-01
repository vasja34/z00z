#![allow(clippy::useless_conversion)]
// crates/z00z_core/examples/asset_snapshot.rs
//
// Example demonstrating asset snapshot save/load operations
//
// This example shows:
// - Creating asset snapshots
// - Saving snapshots in JSON (human-readable) format
// - Saving snapshots in Bincode (compact) format
// - Loading snapshots back into registry
// - Comparing file sizes and formats

use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tempfile::TempDir;
use z00z_core::assets::{
    definition::AssetDefinition, registry::AssetDefinitionRegistry, AssetClass,
};
use z00z_utils::io::read_to_string;
use z00z_utils::prelude::{save_bincode, save_json};

type AppErr = Box<dyn std::error::Error>;

fn main() -> Result<(), AppErr> {
    println!("🎯 Asset Snapshot Example\n");

    let temp_dir = TempDir::new()?;
    let (json_path, bincode_path) = make_paths(&temp_dir);
    print_paths(&json_path, &bincode_path);

    let registry = make_reg();
    let sample_id = seed_reg(&registry)?;
    let snapshot = make_snapshot(&registry)?;
    let (json_size, bincode_size) = write_snapshot(&snapshot, &json_path, &bincode_path)?;

    print_sizes(json_size, bincode_size);
    print_preview(&json_path)?;
    check_restore(&registry, snapshot, sample_id)?;
    print_done();

    Ok(())
}

fn make_paths(temp_dir: &TempDir) -> (PathBuf, PathBuf) {
    (
        temp_dir.path().join("snapshot.json"),
        temp_dir.path().join("snapshot.bincode"),
    )
}

fn print_paths(json_path: &Path, bincode_path: &Path) {
    println!("📁 Snapshot files:");
    println!("   JSON: {}", json_path.display());
    println!("   Bincode: {}", bincode_path.display());
    println!();
}

fn make_reg() -> AssetDefinitionRegistry {
    let logger = Arc::new(z00z_utils::logger::NoopLogger);
    let metrics = Arc::new(z00z_utils::metrics::NoopMetrics);
    let time = Arc::new(z00z_utils::time::SystemTimeProvider);
    AssetDefinitionRegistry::new(logger, metrics, time)
}

fn seed_reg(registry: &AssetDefinitionRegistry) -> Result<[u8; 32], AppErr> {
    println!("📦 Creating sample assets...");

    let mut sample_id = None;
    for i in 1..=5 {
        let def = AssetDefinition::new(
            [i; 32],
            if i % 2 == 0 {
                AssetClass::Coin
            } else {
                AssetClass::Token
            },
            format!("Asset {}", i).into(),
            format!("AST{}", i).into(),
            if i % 2 == 0 { 8 } else { 6 },
            25_000 + (i as u32 * 1_000),
            1_000_000 * (i as u64),
            format!("https://asset{}.z00z.io", i).into(),
            1,
            1,
            0b0000_0001,
            None,
        )?;
        let inserted = registry.insert(def)?;
        if i == 1 {
            sample_id = Some(inserted.id);
        }
    }

    println!("✅ Created {} assets", registry.len()?);
    println!();

    Ok(sample_id.expect("sample definition id"))
}

fn make_snapshot(
    registry: &AssetDefinitionRegistry,
) -> Result<z00z_core::assets::snapshot::RegistrySnapshot, AppErr> {
    println!("📸 Creating registry snapshot...");
    let snapshot = registry.create_snapshot()?;
    println!("📊 Snapshot created:");
    println!("   Version: {:?}", snapshot.version);
    println!("   Timestamp: {}", snapshot.version.timestamp);
    println!("   Assets: {}", snapshot.definitions.len());
    println!();
    Ok(snapshot)
}

fn write_snapshot(
    snapshot: &z00z_core::assets::snapshot::RegistrySnapshot,
    json_path: &Path,
    bincode_path: &Path,
) -> Result<(u64, u64), AppErr> {
    println!("💾 Saving snapshot as JSON...");
    save_json(json_path, snapshot)?;
    let json_size = fs::metadata(json_path)?.len();
    println!("✅ JSON saved: {} bytes", json_size);
    println!();

    println!("💾 Saving snapshot as Bincode...");
    save_bincode(bincode_path, snapshot)?;
    let bincode_size = fs::metadata(bincode_path)?.len();
    println!("✅ Bincode saved: {} bytes", bincode_size);
    println!();

    Ok((json_size, bincode_size))
}

fn print_sizes(json_size: u64, bincode_size: u64) {
    println!("📊 Format comparison:");
    println!("   JSON:    {:>8} bytes (human-readable)", json_size);
    println!("   Bincode: {:>8} bytes (compact)", bincode_size);
    println!(
        "   Ratio:   {:.2}x (JSON/Bincode)",
        json_size as f64 / bincode_size as f64
    );
    println!();
}

fn print_preview(json_path: &Path) -> Result<(), AppErr> {
    println!("📄 JSON preview (first 500 chars):");
    let json = read_to_string(json_path)?;
    if json.len() > 500 {
        println!("{}...", &json[..500]);
    } else {
        println!("{}", json);
    }
    println!();
    Ok(())
}

fn check_restore(
    registry: &AssetDefinitionRegistry,
    snapshot: z00z_core::assets::snapshot::RegistrySnapshot,
    sample_id: [u8; 32],
) -> Result<(), AppErr> {
    println!("🔄 Creating new registry and loading from snapshot...");
    let new_registry = make_reg();
    println!("   Initial size: {}", new_registry.len()?);
    new_registry.update_from_snapshot(snapshot)?;
    println!("   After restore: {}", new_registry.len()?);
    println!("✅ Snapshot loaded successfully");
    println!();

    println!("🔍 Verifying data integrity...");
    let original = registry.get(&sample_id)?.expect("asset should exist");
    let restored = new_registry
        .get(&sample_id)?
        .expect("restored asset should exist");

    println!("   Original asset name: {}", original.name);
    println!("   Restored asset name: {}", restored.name);
    println!("   Original asset symbol: {}", original.symbol);
    println!("   Restored asset symbol: {}", restored.symbol);
    println!("   Name match: {}", original.name == restored.name);
    println!("   Symbol match: {}", original.symbol == restored.symbol);
    println!();

    Ok(())
}

fn print_done() {
    println!("🎉 Example complete!");
    println!("💡 Key features demonstrated:");
    println!("   - Creating registry snapshots");
    println!("   - Saving in JSON format (human-readable)");
    println!("   - Saving in Bincode format (compact)");
    println!("   - Loading snapshots into new registry");
    println!("   - File size comparison between formats");
}
