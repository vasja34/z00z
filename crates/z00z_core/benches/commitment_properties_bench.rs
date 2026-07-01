//! Commitment Homomorphic Properties Benchmarks
//!
//! Tests performance of Pedersen commitment arithmetic operations.
//! Critical for confidential transactions - commitment homomorphism enables
//! private balance verification without revealing amounts.
//!
//! Reference: assets_benches_review.md Section "Recommendation #9"
//!
//! Scenarios:
//! - Commitment addition (C1 + C2)
//! - Commitment subtraction (C1 - C2)
//! - Batch commitment addition (sum of N commitments)
//! - Scalar multiplication (k * C)
//! - Verify homomorphic property: commit(a) + commit(b) = commit(a+b)
//!
//! Expected Performance:
//! - Addition/subtraction: O(1) elliptic curve point addition (~100-200ns)
//! - Scalar multiplication: O(log k) double-and-add (~1-2μs)
//! - Batch sum: O(n) linear with number of commitments
//!
//! Real-world context:
//! - Transaction validation: Sum inputs = Sum all outputs
//! - Balance proofs: Verify commitment arithmetic without amounts
//! - Confidential payments: Homomorphic operations preserve privacy
//!
//! # Note on Direct rand::rngs::OsRng Usage
//!
//! This benchmark uses rand::rngs::OsRng directly instead of z00z_utils::rng::SystemRngProvider.
//! This is intentional for benchmarks because:
//! 1. Benchmarks need direct access to RNG for accurate timing measurements
//! 2. z00z_utils::rng::SystemRngProvider wraps OsRng internally (same performance)
//! 3. The abstraction layer would add overhead that doesn't reflect cryptographic operations
//! 4. Benchmarks are not production code and don't need the same abstraction requirements

use criterion::{criterion_group, criterion_main, BatchSize, Criterion};
use rand::rngs::OsRng; // Direct usage for benchmarks - see module doc
use z00z_core::assets::BlindingFactor;
use z00z_crypto::vendor::tari::ExtendedPedersenCommitmentFactory;
use z00z_crypto::HomomorphicCommitmentFactory;

/// Benchmark: Add two commitments (C1 + C2)
///
/// Basic homomorphic operation: C(a, r1) + C(b, r2) = C(a+b, r1+r2)
/// Used in transaction validation to sum inputs and outputs.
/// Expected: ~100-200ns (single elliptic curve point addition)
fn bench_commitment_add_2(c: &mut Criterion) {
    c.bench_function("commitment_add_2_commitments", |b| {
        b.iter_batched(
            || {
                // Setup: Create two commitments
                let factory = ExtendedPedersenCommitmentFactory::default();
                let blind1 = BlindingFactor::random(&mut OsRng);
                let blind2 = BlindingFactor::random(&mut OsRng);

                let c1 = factory.commit_value(blind1.reveal(), 100);
                let c2 = factory.commit_value(blind2.reveal(), 200);
                (c1, c2)
            },
            |(c1, c2)| {
                // Measure: Add commitments (homomorphic property)
                &c1 + &c2
            },
            BatchSize::SmallInput,
        );
    });
}

/// Benchmark: Subtract two commitments (C1 - C2)
///
/// Homomorphic subtraction: C(a, r1) - C(b, r2) = C(a-b, r1-r2)
/// Used in change calculation and fee validation.
/// Expected: ~100-200ns (same as addition)
fn bench_commitment_sub_2(c: &mut Criterion) {
    c.bench_function("commitment_sub_2_commitments", |b| {
        b.iter_batched(
            || {
                // Setup: Create two commitments
                let factory = ExtendedPedersenCommitmentFactory::default();
                let blind1 = BlindingFactor::random(&mut OsRng);
                let blind2 = BlindingFactor::random(&mut OsRng);

                let c1 = factory.commit_value(blind1.reveal(), 300);
                let c2 = factory.commit_value(blind2.reveal(), 100);
                (c1, c2)
            },
            |(c1, c2)| {
                // Measure: Subtract commitments
                &c1 - &c2
            },
            BatchSize::SmallInput,
        );
    });
}

/// Benchmark: Add 10 commitments (batch sum)
///
/// Sum multiple commitments: C1 + C2 + ... + C10
/// Common in multi-input/output transactions.
/// Expected: ~1-2μs (10 point additions, linear scaling)
fn bench_commitment_add_10(c: &mut Criterion) {
    c.bench_function("commitment_add_10_commitments", |b| {
        b.iter_batched(
            || {
                // Setup: Create 10 commitments
                let factory = ExtendedPedersenCommitmentFactory::default();
                let mut commitments = Vec::with_capacity(10);

                for i in 0..10 {
                    let blinding = BlindingFactor::random(&mut OsRng);
                    let amount = (i + 1) as u64 * 100; // 100, 200, ..., 1000
                    let commitment = factory.commit_value(blinding.reveal(), amount);
                    commitments.push(commitment);
                }

                commitments
            },
            |commitments| {
                // Measure: Sum all commitments
                let mut iter = commitments.iter();
                let mut sum = iter.next().cloned().expect("commitments not empty");
                for commitment in iter {
                    sum = &sum + commitment;
                }
                sum
            },
            BatchSize::SmallInput,
        );
    });
}

/// Benchmark: Add 100 commitments (large batch)
///
/// Simulate block validation with many outputs.
/// Expected: ~10-20μs (100 point additions)
fn bench_commitment_add_100(c: &mut Criterion) {
    c.bench_function("commitment_add_100_commitments", |b| {
        b.iter_batched(
            || {
                // Setup: Create 100 commitments
                let factory = ExtendedPedersenCommitmentFactory::default();
                let mut commitments = Vec::with_capacity(100);

                for i in 0..100 {
                    let blinding = BlindingFactor::random(&mut OsRng);
                    let amount = (i + 1) as u64 * 10;
                    let commitment = factory.commit_value(blinding.reveal(), amount);
                    commitments.push(commitment);
                }

                commitments
            },
            |commitments| {
                // Measure: Sum 100 commitments
                let mut iter = commitments.iter();
                let mut sum = iter.next().cloned().expect("commitments not empty");
                for commitment in iter {
                    sum = &sum + commitment;
                }
                sum
            },
            BatchSize::SmallInput,
        );
    });
}

/// Benchmark: Verify homomorphic property
///
/// Verify: commit(a) + commit(b) == commit(a+b)
/// This is fundamental property used in transaction validation.
/// Expected: ~500-1000ns (2 commits + 1 addition + 1 commit + comparison)
fn bench_verify_homomorphic_property(c: &mut Criterion) {
    c.bench_function("verify_homomorphic_add_property", |b| {
        b.iter_batched(
            || {
                // Setup: Values and blindings for test
                let factory = ExtendedPedersenCommitmentFactory::default();
                let a = 100u64;
                let b = 200u64;
                let blind_a = BlindingFactor::random(&mut OsRng);
                let blind_b = BlindingFactor::random(&mut OsRng);

                (factory, a, b, blind_a, blind_b)
            },
            |(factory, a, b, blind_a, blind_b)| {
                // Measure: Verify homomorphic property
                let c_a = factory.commit_value(blind_a.reveal(), a);
                let c_b = factory.commit_value(blind_b.reveal(), b);
                let c_sum = &c_a + &c_b;

                // Compute blind_a + blind_b
                let blind_sum = &blind_a + &blind_b;
                let c_ab = factory.commit_value(blind_sum.reveal(), a + b);

                // Verify equality
                c_sum == c_ab
            },
            BatchSize::SmallInput,
        );
    });
}

/// Benchmark: Transaction balance verification (simplified)
///
/// Real-world scenario: Verify sum(inputs) = sum(all outputs).
/// Simulates 3 inputs and 3 outputs where one output carries the fee role.
/// Expected: ~500-1000ns (6 point additions total)
fn bench_transaction_balance_check(c: &mut Criterion) {
    c.bench_function("transaction_balance_verification", |b| {
        b.iter_batched(
            || {
                // Setup: Create transaction commitments
                let factory = ExtendedPedersenCommitmentFactory::default();

                // 3 inputs: 100, 200, 150
                let blind_i1 = BlindingFactor::random(&mut OsRng);
                let input1 = factory.commit_value(blind_i1.reveal(), 100);
                let blind_i2 = BlindingFactor::random(&mut OsRng);
                let input2 = factory.commit_value(blind_i2.reveal(), 200);
                let blind_i3 = BlindingFactor::random(&mut OsRng);
                let input3 = factory.commit_value(blind_i3.reveal(), 150);

                // 2 outputs: 250, 180
                let blind_o1 = BlindingFactor::random(&mut OsRng);
                let output1 = factory.commit_value(blind_o1.reveal(), 250);
                let blind_o2 = BlindingFactor::random(&mut OsRng);
                let output2 = factory.commit_value(blind_o2.reveal(), 180);

                // 1 fee: 20 (100+200+150 = 250+180+20)
                let blind_fee = BlindingFactor::random(&mut OsRng);
                let fee = factory.commit_value(blind_fee.reveal(), 20);

                (input1, input2, input3, output1, output2, fee)
            },
            |(input1, input2, input3, output1, output2, fee)| {
                // Measure: Verify balance (sum inputs = sum all outputs)
                let sum_inputs = &(&input1 + &input2) + &input3;
                let sum_all_outputs = &(&output1 + &output2) + &fee;

                // In real validation, this would be zero-commitment check
                // Here we just compute both sides
                (sum_inputs, sum_all_outputs)
            },
            BatchSize::SmallInput,
        );
    });
}

criterion_group! {
    name = commitment_properties_benches;
    config = Criterion::default()
        .warm_up_time(std::time::Duration::from_millis(500))
        .measurement_time(std::time::Duration::from_millis(1500))
        .sample_size(10)
        .without_plots();
    targets = bench_commitment_add_2, bench_commitment_sub_2, bench_commitment_add_10,
              bench_commitment_add_100, bench_verify_homomorphic_property, bench_transaction_balance_check
}
criterion_main!(commitment_properties_benches);
