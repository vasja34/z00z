#![allow(clippy::useless_conversion, clippy::unwrap_or_default)]
// crates/z00z_core/examples/asset_registry_with_metrics.rs
//
// Example demonstrating AssetRegistry with metrics tracking
//
// This example shows:
// - Creating a custom MetricsSink implementation
// - Tracking registry operations with metrics
// - Observing performance measurements
// - Reporting metrics data

use std::collections::BTreeMap;
use std::sync::{Arc, Mutex};
use z00z_core::assets::{
    definition::AssetDefinition, registry::AssetDefinitionRegistry, AssetClass,
};
use z00z_utils::prelude::MetricsSink;

/// Simple in-memory metrics collector for demonstration
#[derive(Debug, Default)]
struct SimpleMetrics {
    counters: Mutex<BTreeMap<String, u64>>,
    gauges: Mutex<BTreeMap<String, f64>>,
    histograms: Mutex<BTreeMap<String, Vec<f64>>>,
}

impl SimpleMetrics {
    fn new() -> Self {
        Self::default()
    }

    fn report(&self) {
        println!("\n📊 Metrics Report:");
        println!("================\n");

        // Report counters
        let counters = self.counters.lock().unwrap();
        if !counters.is_empty() {
            println!("Counters:");
            for (name, value) in counters.iter() {
                println!("  {}: {}", name, value);
            }
            println!();
        }

        // Report gauges
        let gauges = self.gauges.lock().unwrap();
        if !gauges.is_empty() {
            println!("Gauges:");
            for (name, value) in gauges.iter() {
                println!("  {}: {:.2}", name, value);
            }
            println!();
        }

        // Report histograms
        let histograms = self.histograms.lock().unwrap();
        if !histograms.is_empty() {
            println!("Histograms:");
            for (name, values) in histograms.iter() {
                if !values.is_empty() {
                    let sum: f64 = values.iter().sum();
                    let avg = sum / values.len() as f64;
                    let min = values.iter().cloned().fold(f64::INFINITY, f64::min);
                    let max = values.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
                    println!("  {} (n={}):", name, values.len());
                    println!("    avg: {:.2} ms", avg);
                    println!("    min: {:.2} ms", min);
                    println!("    max: {:.2} ms", max);
                }
            }
            println!();
        }
    }
}

impl MetricsSink for SimpleMetrics {
    fn inc_counter(&self, name: &str, value: u64) {
        let mut counters = self.counters.lock().unwrap();
        *counters.entry(name.to_string()).or_insert(0) += value;
    }

    fn observe_histogram(&self, name: &str, value: f64) {
        let mut histograms = self.histograms.lock().unwrap();
        histograms
            .entry(name.to_string())
            .or_insert_with(Vec::new)
            .push(value);
    }

    fn set_gauge(&self, name: &str, value: f64) {
        let mut gauges = self.gauges.lock().unwrap();
        gauges.insert(name.to_string(), value);
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🎯 AssetRegistry with Metrics Example\n");

    // Create test dependencies
    let logger = Arc::new(z00z_utils::logger::NoopLogger);
    let metrics = Arc::new(SimpleMetrics::new());
    let time = Arc::new(z00z_utils::time::SystemTimeProvider);

    // Create registry with dependencies
    let registry = AssetDefinitionRegistry::new(logger, metrics.clone(), time);

    println!("✅ Created AssetRegistry with SimpleMetrics\n");

    // Insert individual assets (tracked by metrics)
    println!("📦 Inserting individual assets...");
    for i in 1..=10 {
        let def = AssetDefinition::new(
            [i; 32],
            if i % 2 == 0 {
                AssetClass::Coin
            } else {
                AssetClass::Token
            },
            format!("Asset {}", i),
            format!("AST{}", i),
            6,
            25_000u32,
            1_000_000,
            "z00z.io".to_string(),
            1u8,
            1u8,
            0b0000_0001u8,
            None,
        )?;
        registry.insert(def)?;
    }
    println!("✅ Inserted 10 individual assets\n");

    // Create batch for performance measurement
    println!("📦 Preparing batch insert...");
    let mut batch = Vec::new();
    for i in 11..=20 {
        let def = AssetDefinition::new(
            [i; 32],
            AssetClass::Token,
            format!("Token {}", i),
            format!("TKN{}", i),
            6,
            20_000u32,
            500_000,
            "z00z.io".to_string(),
            1u8,
            1u8,
            0b0000_0001u8,
            None,
        )?;
        batch.push(def);
    }

    let batch_size = batch.len();
    println!("✅ Batch prepared: {} assets", batch_size);
    println!("🚀 Executing batch insert...");
    registry.insert_batch(batch)?;
    println!("✅ Batch insert complete\n");

    // Check final registry size
    let size = registry.len()?;
    println!("📊 Registry size: {} assets\n", size);

    metrics.report();

    println!("🎉 Example complete!");
    println!("💡 Note: This example uses SimpleMetrics (in-memory collector)");

    Ok(())
}
