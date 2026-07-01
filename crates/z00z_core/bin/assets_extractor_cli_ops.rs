use super::{
    load_bincode_bounded, save_bincode, save_json, Asset, AssetFilters, ExtractorConfig,
    OutputFormat, Path, PathBuf, RangeInclusive, MAX_ASSET_FILE_SIZE,
};

pub(super) fn extract_assets(
    config: &ExtractorConfig,
) -> Result<Vec<Asset>, Box<dyn std::error::Error>> {
    if config.verbose {
        println!("🔍 Scanning genesis files...");
    }

    // Scan for genesis files
    let genesis_files = scan_genesis_files(&config.input_dir)?;

    if genesis_files.is_empty() {
        return Err("No genesis files found in directory".into());
    }

    // Load all assets
    let mut all_assets = Vec::new();
    for file_path in &genesis_files {
        if file_path.extension().and_then(|s| s.to_str()) == Some("bin") {
            let assets = load_bincode_assets(file_path)?;

            if config.verbose {
                let file_name = file_path
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("unknown");
                println!("   ✓ Found {} ({} assets)", file_name, assets.len());
            }

            all_assets.extend(assets);
        }
    }

    if config.verbose {
        println!("\n⚙️  Applying filters...");
    }

    // Apply filters
    let mut filtered = all_assets;

    if let Some(class) = config.filters.class {
        filtered.retain(|a| a.definition.class == class);
        if config.verbose {
            println!("   ✓ Class filter: {} matches", filtered.len());
        }
    }

    if let Some(ref range) = config.filters.serial_range {
        filtered.retain(|a| range.contains(&a.serial_id));
        if config.verbose {
            println!("   ✓ Serial range: {} matches", filtered.len());
        }
    }

    if let Some(min) = config.filters.amount_min {
        filtered.retain(|a| a.amount >= min);
        if config.verbose {
            println!("   ✓ Amount min filter: {} matches", filtered.len());
        }
    }

    if let Some(max) = config.filters.amount_max {
        filtered.retain(|a| a.amount <= max);
        if config.verbose {
            println!("   ✓ Amount max filter: {} matches", filtered.len());
        }
    }

    if let Some(ref symbol) = config.filters.definition_symbol {
        filtered.retain(|a| a.definition.symbol == *symbol);
        if config.verbose {
            println!("   ✓ Symbol filter: {} matches", filtered.len());
        }
    }

    if config.verbose {
        let total_amount: u64 = filtered.iter().map(|a| a.amount).sum();
        println!("\n📊 Extraction Summary:");
        println!("   - Matched filters: {} assets", filtered.len());
        println!("   - Total amount: {}", format_number(total_amount));
    }

    Ok(filtered)
}

/// Save extracted assets to file
pub(super) fn save_extracted(
    assets: &[Asset],
    config: &ExtractorConfig,
) -> Result<(), Box<dyn std::error::Error>> {
    if config.verbose {
        println!("\n💾 Writing to: {}", config.output_file.display());
        println!("   Format: {:?}", config.output_format);
        println!("   Include proofs: {}", config.include_proofs);
    }

    match config.output_format {
        OutputFormat::Json => save_json_assets(assets, &config.output_file, config.include_proofs)?,
        OutputFormat::Bincode => save_bincode_assets(assets, &config.output_file)?,
        OutputFormat::Csv => save_csv(assets, &config.output_file)?,
    }

    Ok(())
}

/// Save as JSON
fn save_json_assets(
    assets: &[Asset],
    path: &Path,
    include_proofs: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    if include_proofs {
        // `save_json` expects a sized value, so pass an owned Vec
        let owned: Vec<Asset> = assets.to_vec();
        save_json(path, &owned)?;
    } else {
        // Strip proofs for smaller file size
        let simplified: Vec<_> = assets
            .iter()
            .map(|a| {
                let mut clone = a.clone();
                clone.range_proof = None;
                clone
            })
            .collect();
        save_json(path, &simplified)?;
    }
    Ok(())
}

/// Save as Bincode
fn save_bincode_assets(assets: &[Asset], path: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let owned: Vec<Asset> = assets.to_vec();
    save_bincode(path, &owned)?;
    Ok(())
}

/// Save as CSV (metadata only)
fn save_csv(assets: &[Asset], path: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let mut csv_content = String::from(
        "symbol,class,serial_id,amount,commitment_hex,nonce_hex,has_proof,has_signature\n",
    );

    for asset in assets {
        csv_content.push_str(&format!(
            "{},{:?},{},{},{},{},{},{}\n",
            asset.definition.symbol,
            asset.definition.class,
            asset.serial_id,
            asset.amount,
            hex::encode(asset.commitment.as_bytes()),
            hex::encode(asset.nonce),
            asset.range_proof.is_some(),
            asset.owner_signature.is_some()
        ));
    }

    std::fs::write(path, csv_content.as_bytes())?;
    Ok(())
}

/// Scan directory for genesis files
fn scan_genesis_files(dir: &Path) -> Result<Vec<PathBuf>, Box<dyn std::error::Error>> {
    let mut files: Vec<PathBuf> = Vec::new();
    for entry in std::fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if let Some(ext) = path.extension().and_then(|s| s.to_str()) {
            if ext == "bin" || ext == "json" {
                files.push(path);
            }
        }
    }

    files.sort();
    Ok(files)
}

/// Load assets from bincode file
fn load_bincode_assets(path: &Path) -> Result<Vec<Asset>, Box<dyn std::error::Error>> {
    let assets: Vec<Asset> = load_bincode_bounded(path, MAX_ASSET_FILE_SIZE)?;
    Ok(assets)
}

/// Print active filters
pub(super) fn print_filters(filters: &AssetFilters) {
    println!("🎯 Filters:");

    if let Some(class) = filters.class {
        println!("   - Class: {:?}", class);
    }

    if let Some(ref range) = filters.serial_range {
        println!("   - Serial range: {}-{}", range.start(), range.end());
    }

    if let Some(min) = filters.amount_min {
        println!("   - Amount min: {}", format_number(min));
    }

    if let Some(max) = filters.amount_max {
        println!("   - Amount max: {}", format_number(max));
    }

    if let Some(ref symbol) = filters.definition_symbol {
        println!("   - Symbol: {}", symbol);
    }

    println!();
}

/// Format number with thousand separators
fn format_number(n: u64) -> String {
    n.to_string()
        .as_bytes()
        .rchunks(3)
        .rev()
        .map(|chunk| std::str::from_utf8(chunk).unwrap())
        .collect::<Vec<_>>()
        .join(",")
}

/// Parse range string (e.g., "0-99")
pub(super) fn parse_range(s: &str) -> Result<RangeInclusive<u32>, Box<dyn std::error::Error>> {
    let parts: Vec<&str> = s.split('-').collect();
    if parts.len() != 2 {
        return Err("Invalid range format. Use: start-end (e.g., 0-99)".into());
    }

    let start: u32 = parts[0].parse()?;
    let end: u32 = parts[1].parse()?;

    if start > end {
        return Err("Range start must be <= end".into());
    }

    Ok(start..=end)
}
