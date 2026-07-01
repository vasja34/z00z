use super::*;

pub(super) fn serialize_assets(
    assets: &[Asset],
    args: &Args,
) -> Result<SerializationStats, Box<dyn std::error::Error>> {
    let mut stats = SerializationStats {
        json_size_bytes: None,
        json_time_ms: None,
        bincode_size_bytes: None,
        bincode_time_ms: None,
    };

    let time_provider = SystemTimeProvider;
    let timestamp =
        format_unix_timestamp_milliseconds_compact(time_provider.compat_unix_timestamp_millis());

    match args.format {
        Format::Json => {
            // Serialize to JSON using Debug formatting
            let start = Instant::now();

            // Group assets by class
            let mut by_class: BTreeMap<String, Vec<AssetExportEntry>> = BTreeMap::new();
            for asset in assets {
                let class_name = format!("{:?}", asset.definition.class).to_lowercase();
                by_class
                    .entry(class_name)
                    .or_default()
                    .push(AssetExportEntry::from_asset(asset));
            }

            // Collect class names for later size calculation
            let class_names: Vec<String> = by_class.keys().cloned().collect();

            // Write per-class JSON files
            for (class_name, class_assets) in by_class {
                let filename = format!("{}s_{}.json", class_name, timestamp);
                let path = args.output.join("json").join(&filename);

                // Serialize to pretty JSON using z00z_utils codec abstraction
                let codec = JsonCodec;
                let json_bytes = codec
                    .serialize_pretty(&class_assets)
                    .map_err(|e| format!("Failed to serialize to JSON: {}", e))?;

                fs::write(&path, json_bytes)
                    .map_err(|e| format!("Failed to write {}: {}", path.display(), e))?;

                if args.verbose {
                    println!("   ✓ Wrote {}", path.display());
                }
            }

            let json_time_ms = start.elapsed().as_millis();

            // Calculate total JSON size using saved class names
            let total_size: usize = class_names
                .iter()
                .map(|class_name| {
                    let filename = format!("{}s_{}.json", class_name, timestamp);
                    let path = args.output.join("json").join(&filename);
                    fs::metadata(&path).map(|m| m.len() as usize).unwrap_or(0)
                })
                .sum();

            stats.json_size_bytes = Some(total_size);
            stats.json_time_ms = Some(json_time_ms);
        }
        Format::Bincode => {
            // Serialize to bincode - ALL assets together in one file
            let start = Instant::now();

            let filename = format!("assets_{}.bin", timestamp);
            let path = args.output.join("bin").join(&filename);

            let export_entries: Vec<AssetExportEntry> =
                assets.iter().map(AssetExportEntry::from_asset).collect();

            // Serialize all assets using z00z_utils codec abstraction
            let codec = BincodeCodec;
            let bytes = codec
                .serialize(&export_entries)
                .map_err(|e| format!("Failed to serialize to bincode: {}", e))?;

            fs::write(&path, &bytes)
                .map_err(|e| format!("Failed to write bincode file {}: {}", path.display(), e))?;

            let bincode_time_ms = start.elapsed().as_millis();
            let bincode_size = bytes.len();

            if args.verbose {
                println!("   ✓ Wrote {}", path.display());
                println!(
                    "   ✓ Bincode size: {} bytes ({:.2} KB)",
                    bincode_size,
                    bincode_size as f64 / 1024.0
                );
            }

            // Deserialization integrity test
            let loaded: Vec<AssetExportEntry> = codec
                .deserialize(&bytes)
                .map_err(|e| format!("Failed to deserialize from bincode: {}", e))?;

            if loaded.len() != assets.len() {
                return Err(format!(
                    "Deserialization mismatch: expected {} assets, got {}",
                    assets.len(),
                    loaded.len()
                )
                .into());
            }

            if args.verbose {
                println!(
                    "   ✓ Deserialization test passed: {}/{} assets",
                    loaded.len(),
                    assets.len()
                );
            }

            stats.bincode_size_bytes = Some(bincode_size);
            stats.bincode_time_ms = Some(bincode_time_ms);
        }
    }

    Ok(stats)
}

pub(super) fn print_serialization_stats(stats: &SerializationStats) {
    if let Some(size) = stats.json_size_bytes {
        println!(
            "   ✓ JSON size: {} bytes ({:.2} KB)",
            size,
            size as f64 / 1024.0
        );
    }
    if let Some(time) = stats.json_time_ms {
        println!("   ✓ JSON serialization time: {}ms", time);
    }
    if let Some(size) = stats.bincode_size_bytes {
        println!(
            "   ✓ Bincode size: {} bytes ({:.2} KB)",
            size,
            size as f64 / 1024.0
        );
    }
    if let Some(time) = stats.bincode_time_ms {
        println!("   ✓ Bincode serialization time: {}ms", time);
    }

    // Print compression ratio if both formats available
    if let (Some(json_size), Some(bincode_size)) = (stats.json_size_bytes, stats.bincode_size_bytes)
    {
        let ratio = (bincode_size as f64 / json_size as f64) * 100.0;
        println!(
            "   ✓ Compression ratio: {:.1}% (bincode is {:.1}x smaller)",
            ratio,
            json_size as f64 / bincode_size as f64
        );
    }
}

// ============================================================================
// Phase 6: Generate Reports
// ============================================================================

pub(super) fn generate_reports(
    output_dir: &Path,
    timestamp: &str,
    gen_stats: &GenerationStats,
    verify_stats: &VerificationStats,
    ser_stats: &SerializationStats,
    assets: &[Asset],
    args: &Args,
) -> Result<(), Box<dyn std::error::Error>> {
    let report_path = output_dir
        .join("reports")
        .join(format!("report_{}.txt", timestamp));

    let mut report = String::new();

    // Header
    report.push_str("Z00Z Asset Generation Report\n");
    report.push_str("============================\n");
    let time_provider = SystemTimeProvider;
    report.push_str(&format!(
        "Generated: {}\n",
        format_system_time_utc(time_provider.now())
    ));

    // Command
    report.push_str(&format!(
        "Command: cargo run --release --bin assets_generation_cli -- --config {} --format {:?}\n\n",
        args.config.display(),
        args.format
    ));

    // Generation Statistics
    report.push_str("📊 Generation Statistics\n");
    report.push_str("------------------------\n");
    report.push_str(&format!(
        "Total assets generated: {}\n",
        gen_stats.total_assets
    ));
    report.push_str(&format!(
        "Generation time: {:.2}s\n",
        gen_stats.generation_time_ms as f64 / 1000.0
    ));
    report.push_str(&format!(
        "Throughput: {:.2} assets/sec\n",
        gen_stats.total_assets as f64 / (gen_stats.generation_time_ms as f64 / 1000.0)
    ));
    report.push_str(&format!(
        "Per-asset time: {:.2}ms\n\n",
        gen_stats.generation_time_ms as f64 / gen_stats.total_assets as f64
    ));

    // Per-Class Breakdown
    report.push_str("📈 Per-Class Breakdown\n");
    report.push_str("----------------------\n");
    for (class, count) in &gen_stats.per_class_counts {
        let total_amount = gen_stats.per_class_amounts.get(class).unwrap_or(&0);
        report.push_str(&format!(
            "  {}: {} assets, total amount: {}\n",
            class, count, total_amount
        ));
    }
    report.push('\n');

    // Verification Statistics
    report.push_str("🛡️  Verification Statistics\n");
    report.push_str("---------------------------\n");
    report.push_str(&format!(
        "Total verified: {}\n",
        verify_stats.total_verified
    ));
    report.push_str(&format!(
        "Commitments valid: {} ({:.1}%)\n",
        verify_stats.commitments_valid,
        verify_stats.commitments_valid as f64 / verify_stats.total_verified as f64 * 100.0
    ));
    report.push_str(&format!(
        "Range proofs present: {} ({:.1}%)\n",
        verify_stats.range_proofs_present,
        verify_stats.range_proofs_present as f64 / verify_stats.total_verified as f64 * 100.0
    ));
    report.push_str(&format!(
        "Signatures valid: {} ({:.1}%)\n",
        verify_stats.signatures_valid,
        verify_stats.signatures_valid as f64 / verify_stats.total_verified as f64 * 100.0
    ));
    if verify_stats.homomorphic_tests_passed > 0 || verify_stats.homomorphic_tests_failed > 0 {
        let total_homo_tests =
            verify_stats.homomorphic_tests_passed + verify_stats.homomorphic_tests_failed;
        report.push_str(&format!(
            "Homomorphic tests passed: {} ({:.1}%)\n",
            verify_stats.homomorphic_tests_passed,
            verify_stats.homomorphic_tests_passed as f64 / total_homo_tests as f64 * 100.0
        ));
    }
    report.push('\n');

    // Serialization Statistics
    report.push_str("💾 Serialization Statistics\n");
    report.push_str("---------------------------\n");
    if let Some(size) = ser_stats.json_size_bytes {
        report.push_str(&format!(
            "JSON size: {} bytes ({:.2} KB)\n",
            size,
            size as f64 / 1024.0
        ));
    }
    if let Some(time) = ser_stats.json_time_ms {
        report.push_str(&format!("JSON serialization time: {}ms\n", time));
    }
    if let Some(size) = ser_stats.bincode_size_bytes {
        report.push_str(&format!(
            "Bincode size: {} bytes ({:.2} KB)\n",
            size,
            size as f64 / 1024.0
        ));
    }
    if let Some(time) = ser_stats.bincode_time_ms {
        report.push_str(&format!("Bincode serialization time: {}ms\n", time));
    }
    // Compression ratio
    if let (Some(json_size), Some(bincode_size)) =
        (ser_stats.json_size_bytes, ser_stats.bincode_size_bytes)
    {
        let ratio = (bincode_size as f64 / json_size as f64) * 100.0;
        report.push_str(&format!(
            "Compression ratio: {:.1}% (bincode is {:.1}x smaller)\n",
            ratio,
            json_size as f64 / bincode_size as f64
        ));
    }
    report.push('\n');

    // Memory Estimate
    report.push_str("💾 Memory Estimates\n");
    report.push_str("-------------------\n");
    let est_size_per_asset = std::mem::size_of::<Asset>();
    let total_est_memory = std::mem::size_of_val(assets);
    report.push_str(&format!(
        "Estimated size per asset: {} bytes\n",
        est_size_per_asset
    ));
    report.push_str(&format!(
        "Total estimated memory: {} bytes ({:.2} KB)\n",
        total_est_memory,
        total_est_memory as f64 / 1024.0
    ));
    report.push('\n');

    // Write report
    fs::write(&report_path, report).map_err(|e| format!("Failed to write report: {}", e))?;

    println!("   ✓ Report saved to: {}", report_path.display());

    Ok(())
}
