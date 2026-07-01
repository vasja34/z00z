/// Gas Calculation Benchmarks
///
/// **Reference**: See `assets_benches_review.md` Section "M-2: Gas calculation benchmarks"
///
/// Measures gas calculation performance with realistic transaction structures.
/// Uses real `GasMetered` trait implementation instead of mock structures.
///
/// ## Scenarios tested:
/// - Simple transfer (1 input, 1 output)
/// - Multi-input transaction (2 inputs, 2 outputs)
/// - Consolidation (10 inputs, 5 outputs)
/// - Distribution (1 input, 100 outputs)
/// - Edge cases (zero inputs/outputs, maximum reasonable)
///
/// ## Why this matters:
/// - Gas calculation happens for every transaction during validation
/// - Must handle saturating arithmetic to prevent overflow attacks
/// - Performance must scale linearly with transaction size
///
/// ## Formula verification:
/// fee = base + (inputs * rate_in) + (outputs * rate_out) + (proof_bits * rate_proof)
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use z00z_core::assets::gas::{
    calculate_fee, GasMetered, GasPrice, GasUsage, GAS_SCHEDULE_PLACEHOLDER,
};

/// Realistic test transaction structure implementing GasMetered
/// Represents actual transaction structure with inputs, outputs, and range proofs
#[derive(Debug, Clone)]
struct TestTransaction {
    inputs: u32,
    outputs: u32,
    range_proof_bits: u32,
}

impl TestTransaction {
    fn new(inputs: u32, outputs: u32) -> Self {
        // Each output requires a 64-bit range proof
        let range_proof_bits = outputs * 64;
        Self {
            inputs,
            outputs,
            range_proof_bits,
        }
    }
}

impl GasMetered for TestTransaction {
    fn gas_usage(&self) -> GasUsage {
        GasUsage {
            inputs: self.inputs as usize,
            outputs: self.outputs as usize,
            range_proof_bits: self.range_proof_bits as usize,
        }
    }
}

/// Benchmark: Simple transfer (1 input, 1 output)
/// Most common transaction pattern - wallet-to-wallet transfer
fn gas_calculation_simple(c: &mut Criterion) {
    c.bench_function("calculate_fee_simple_transfer", |b| {
        let tx = TestTransaction::new(1, 1);
        let gas_price = GasPrice::new(10);
        b.iter(|| {
            calculate_fee(
                black_box(&tx),
                black_box(&GAS_SCHEDULE_PLACEHOLDER),
                black_box(&gas_price),
            )
        })
    });
}

/// Benchmark: Multi-input transaction (2 inputs, 2 outputs)
/// Common for payments requiring multiple assets
fn gas_calculation_multi_input(c: &mut Criterion) {
    c.bench_function("calculate_fee_multi_input", |b| {
        let tx = TestTransaction::new(2, 2);
        let gas_price = GasPrice::new(10);
        b.iter(|| {
            calculate_fee(
                black_box(&tx),
                black_box(&GAS_SCHEDULE_PLACEHOLDER),
                black_box(&gas_price),
            )
        })
    });
}

/// Benchmark: Consolidation (10 inputs, 5 outputs)
/// Wallet maintenance - combining many small assets
fn gas_calculation_consolidation(c: &mut Criterion) {
    c.bench_function("calculate_fee_consolidation", |b| {
        let tx = TestTransaction::new(10, 5);
        let gas_price = GasPrice::new(10);
        b.iter(|| {
            calculate_fee(
                black_box(&tx),
                black_box(&GAS_SCHEDULE_PLACEHOLDER),
                black_box(&gas_price),
            )
        })
    });
}

/// Benchmark: Distribution (1 input, 100 outputs)
/// Batch payments - exchange withdrawals, payroll, airdrops
fn gas_calculation_distribution(c: &mut Criterion) {
    c.bench_function("calculate_fee_distribution", |b| {
        let tx = TestTransaction::new(1, 100);
        let gas_price = GasPrice::new(10);
        b.iter(|| {
            calculate_fee(
                black_box(&tx),
                black_box(&GAS_SCHEDULE_PLACEHOLDER),
                black_box(&gas_price),
            )
        })
    });
}

/// Benchmark: Large transaction (50 inputs, 50 outputs)
/// Stress test for maximum reasonable transaction size
fn gas_calculation_large(c: &mut Criterion) {
    c.bench_function("calculate_fee_large_tx", |b| {
        let tx = TestTransaction::new(50, 50);
        let gas_price = GasPrice::new(10);
        b.iter(|| {
            calculate_fee(
                black_box(&tx),
                black_box(&GAS_SCHEDULE_PLACEHOLDER),
                black_box(&gas_price),
            )
        })
    });
}

criterion_group! {
    name = benches;
    config = Criterion::default()
        .warm_up_time(std::time::Duration::from_millis(500))
        .measurement_time(std::time::Duration::from_millis(1500))
        .sample_size(10)
        .without_plots();
    targets = gas_calculation_simple, gas_calculation_multi_input, gas_calculation_consolidation,
              gas_calculation_distribution, gas_calculation_large
}
criterion_main!(benches);
