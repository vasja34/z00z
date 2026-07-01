use super::{
    fs, load_bincode_bounded, load_json_bounded, AnalyzerConfig, Asset, AssetClass, ClassStats,
    HashMap, SerialStats, MAX_ASSET_FILE_SIZE,
};

pub(super) fn load_all_assets(
    config: &AnalyzerConfig,
) -> Result<Vec<Asset>, Box<dyn std::error::Error>> {
    let mut all_assets = Vec::new();

    // If specific files provided, load only them
    if !config.input_files.is_empty() {
        for path in &config.input_files {
            if config.verbose {
                let file_name = path
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("unknown");
                println!("📂 Loading file: {}", file_name);
            }

            if path.extension().and_then(|s| s.to_str()) == Some("bin") {
                let assets: Vec<Asset> = load_bincode_bounded(path, MAX_ASSET_FILE_SIZE)?;
                let filtered = filter_assets(assets, config);
                all_assets.extend(filtered);
            } else if path.extension().and_then(|s| s.to_str()) == Some("json") {
                let assets: Vec<Asset> = load_json_bounded(path, MAX_ASSET_FILE_SIZE)?;
                let filtered = filter_assets(assets, config);
                all_assets.extend(filtered);
            }
        }

        return Ok(all_assets);
    }

    // Otherwise read directory
    let input_dir = config
        .input_dir
        .as_ref()
        .ok_or("input directory not provided")?;
    let entries = fs::read_dir(input_dir)?;

    for entry in entries {
        let entry = entry?;
        let path = entry.path();

        if path.is_file() {
            if let Some(ext) = path.extension().and_then(|s| s.to_str()) {
                if ext == "bin" || ext == "json" {
                    if config.verbose {
                        let file_name = path
                            .file_name()
                            .and_then(|n| n.to_str())
                            .unwrap_or("unknown");
                        println!("📂 Loading: {}", file_name);
                    }

                    let assets: Vec<Asset> = if ext == "bin" {
                        load_bincode_bounded(&path, MAX_ASSET_FILE_SIZE)?
                    } else {
                        load_json_bounded(&path, MAX_ASSET_FILE_SIZE)?
                    };
                    let filtered = filter_assets(assets, config);
                    all_assets.extend(filtered);
                }
            }
        }
    }

    Ok(all_assets)
}

fn filter_assets(assets: Vec<Asset>, config: &AnalyzerConfig) -> Vec<Asset> {
    if let Some(class) = config.filter_class {
        assets
            .into_iter()
            .filter(|a| a.definition.class == class)
            .collect()
    } else {
        assets
    }
}

/// Analyze assets and display results
pub(super) fn analyze_and_display(
    assets: &[Asset],
    config: &AnalyzerConfig,
) -> Result<HashMap<AssetClass, ClassStats>, Box<dyn std::error::Error>> {
    let mut class_stats: HashMap<AssetClass, ClassStats> = HashMap::new();

    // Analyze each asset
    for asset in assets {
        let stats = class_stats.entry(asset.definition.class).or_default();

        stats.count += 1;
        stats.total_supply += asset.amount;
        stats.nominal_value = asset.definition.nominal;

        if !stats.symbols.contains(&asset.definition.symbol) {
            stats.symbols.push(asset.definition.symbol.clone());
        }

        // Track serial ID statistics
        let serial_stat = stats
            .serial_distribution
            .entry(asset.serial_id)
            .or_insert_with(|| SerialStats {
                count: 0,
                amount: asset.amount,
                nominal: asset.definition.nominal,
                has_proof: asset.range_proof.is_some(),
            });

        serial_stat.count += 1;
    }

    // Display results
    display_summary(&class_stats);

    if config.verbose {
        display_serial_distribution(&class_stats);
    }

    // Also show serial_id | nominal table when verbose
    if config.verbose {
        display_serial_nominal(&class_stats);
    }

    Ok(class_stats)
}

/// Write a markdown report into the same input directory
pub(super) fn write_report(
    stats: &HashMap<AssetClass, ClassStats>,
    input_dir: &std::path::Path,
) -> Result<(), Box<dyn std::error::Error>> {
    // Derive report filename from input directory name.
    // Expected dir name: `genesis_{network}_{YYYYMMDD_HHMMSS}`
    let dir_name = input_dir
        .file_name()
        .and_then(|s| s.to_str())
        .ok_or("Invalid input directory name")?;
    let suffix = if let Some(stripped) = dir_name.strip_prefix("genesis_") {
        stripped
    } else {
        dir_name
    };

    let report_name = format!("genesis_analysis_{}.md", suffix);
    let report_path = input_dir.join(&report_name);

    let mut md = String::new();
    md.push_str(&format!("# Genesis Analysis Report - {}\n\n", dir_name));
    md.push_str(&format!("**Directory**: `{}`\n\n", input_dir.display()));

    // Summary
    md.push_str("## Summary\n\n");
    let mut total_assets = 0usize;
    let mut total_supply: u128 = 0;

    // Table header
    md.push_str("| Class | Count | Total Supply | Nominal Value | Symbols |\n");
    md.push_str("|---|---:|---:|---:|---|\n");

    let mut sorted: Vec<_> = stats.iter().collect();
    sorted.sort_by_key(|(class, _)| format!("{:?}", class));

    for (class, stat) in &sorted {
        total_assets += stat.count;
        total_supply += stat.total_supply as u128;
        let symbols = if stat.symbols.is_empty() {
            "-".to_string()
        } else {
            stat.symbols.join(", ")
        };
        md.push_str(&format!(
            "| {:?} | {} | {} | {} | {} |\n",
            class, stat.count, stat.total_supply, stat.nominal_value, symbols
        ));
    }

    md.push_str(&format!("\n**Total assets:** {}\n\n", total_assets));
    md.push_str(&format!("**Total supply:** {}\n\n", total_supply));

    // Files list and sizes
    md.push_str("## Files\n\n");
    md.push_str("| Name | Format | Size (bytes) |\n");
    md.push_str("|---|---|---:|\n");

    for entry in std::fs::read_dir(input_dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_file() {
            let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
            let format = path.extension().and_then(|e| e.to_str()).unwrap_or("");
            let size = std::fs::metadata(&path).map(|m| m.len()).unwrap_or(0);
            md.push_str(&format!("| {} | {} | {} |\n", name, format, size));
        }
    }

    // Serial distribution (brief) — include counts per class
    md.push_str("\n## Per-Class Serial Distribution (counts)\n\n");
    for (class, stat) in &sorted {
        md.push_str(&format!(
            "- **{:?}**: {} serial IDs ({} assets)\n",
            class,
            stat.serial_distribution.len(),
            stat.count
        ));
    }

    // Per-class serial_id | nominal tables
    md.push_str("\n## Per-Class Serial Nominal\n\n");
    for (class, stat) in &sorted {
        md.push_str(&format!("### {:?}\n\n", class));
        md.push_str("| Serial ID | Nominal |\n");
        md.push_str("|---:|---:|\n");

        let mut serials: Vec<_> = stat.serial_distribution.iter().collect();
        serials.sort_by_key(|(id, _)| *id);
        for (serial_id, serial_stat) in serials {
            md.push_str(&format!("| {} | {} |\n", serial_id, serial_stat.nominal));
        }

        md.push('\n');
    }

    // Write file
    std::fs::write(&report_path, md)?;

    println!("📄 Report written: {}", report_path.display());

    Ok(())
}

/// Display summary table
fn display_summary(stats: &HashMap<AssetClass, ClassStats>) {
    println!("📊 Asset Summary");
    println!("┌────────────┬─────────┬──────────────┬────────────────┐");
    println!("│ Class      │ Count   │ Total Supply │ Nominal Value  │");
    println!("├────────────┼─────────┼──────────────┼────────────────┤");

    let mut total_count = 0;
    let mut sorted: Vec<_> = stats.iter().collect();
    sorted.sort_by_key(|(class, _)| format!("{:?}", class));

    for (class, stat) in &sorted {
        total_count += stat.count;

        println!(
            "│ {:10} │ {:7} │ {:12} │ {:14} │",
            format!("{:?}", class),
            format_number(stat.count as u64),
            format_number(stat.total_supply),
            format_number(stat.nominal_value)
        );
    }

    println!("├────────────┼─────────┼──────────────┼────────────────┤");
    println!(
        "│ {:10} │ {:7} │ {:12} │ {:14} │",
        "TOTAL",
        format_number(total_count as u64),
        "-",
        "-"
    );
    println!("└────────────┴─────────┴──────────────┴────────────────┘");
    println!();
}

/// Display serial ID distribution for each class
fn display_serial_distribution(stats: &HashMap<AssetClass, ClassStats>) {
    println!("🎯 Serial ID Distribution");
    println!();

    for (class, stat) in stats {
        println!("  {:?} ({} total)", class, format_number(stat.count as u64));
        println!("  ┌──────────────┬───────┬──────────────┬───────────┐");
        println!("  │ Serial ID    │ Count │ Amount       │ Has Proof │");
        println!("  ├──────────────┼───────┼──────────────┼───────────┤");

        let mut serials: Vec<_> = stat.serial_distribution.iter().collect();
        serials.sort_by_key(|(id, _)| *id);

        for (serial_id, serial_stat) in serials {
            println!(
                "  │ {:12} │ {:5} │ {:12} │ {:9} │",
                serial_id,
                serial_stat.count,
                format_number(serial_stat.amount),
                if serial_stat.has_proof { "✓" } else { "✗" }
            );
        }

        println!("  └──────────────┴───────┴──────────────┴───────────┘");
        println!();
    }
}

/// Display per-class serial_id | nominal table
fn display_serial_nominal(stats: &HashMap<AssetClass, ClassStats>) {
    println!("🔎 Serial ID | Nominal");
    println!();

    for (class, stat) in stats {
        println!("  {:?}", class);
        println!("  ┌──────────────┬──────────┐");
        println!("  │ Serial ID    │ Nominal  │");
        println!("  ├──────────────┼──────────┤");

        let mut serials: Vec<_> = stat.serial_distribution.iter().collect();
        serials.sort_by_key(|(id, _)| *id);

        for (serial_id, serial_stat) in serials {
            println!(
                "  │ {:12} │ {:8} │",
                serial_id,
                format_number(serial_stat.nominal)
            );
        }

        println!("  └──────────────┴──────────┘");
        println!();
    }
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
