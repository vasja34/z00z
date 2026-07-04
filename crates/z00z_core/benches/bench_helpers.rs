//! Benchmark helpers for testing Asset operations
//!
//! Provides utility functions for benchmarks to use real nonce generation
//! instead of hardcoded `[0u8; 32]`.

use std::sync::atomic::{AtomicU64, Ordering};
use z00z_core::assets::{derive_nonce, Nonce};

/// Global counter for unique nonce generation in benchmarks
static BENCH_NONCE_COUNTER: AtomicU64 = AtomicU64::new(0);

/// Fixed wallet seed for benchmarks (deterministic but not zero)
const BENCH_WALLET_SEED: &[u8; 32] = b"bench_wallet_seed_32_bytes_long!";

/// Generate a unique nonce for benchmarking
///
/// Uses derive_nonce() with a global counter to ensure each call
/// produces a unique nonce. This replaces hardcoded `[0u8; 32]` nonces
/// in benchmarks.
///
/// # Example
/// ```ignore
/// let nonce = bench_nonce(&[1u8; 32]);
///
/// let next = bench_nonce(&[1u8; 32]);
/// assert_ne!(nonce, next); // Always unique
/// ```
pub fn bench_nonce(asset_id: &[u8; 32]) -> Nonce {
    let counter = BENCH_NONCE_COUNTER.fetch_add(1, Ordering::SeqCst);
    derive_nonce(BENCH_WALLET_SEED, counter, asset_id)
        .expect("bench_nonce should never fail")
}

/// Generate multiple unique nonces for batch benchmarks
pub fn bench_nonces(asset_id: &[u8; 32], count: usize) -> Vec<Nonce> {
    (0..count).map(|_| bench_nonce(asset_id)).collect()
}

/// Reset the global nonce counter (useful for reproducible benchmarks)
pub fn reset_bench_nonce_counter() {
    BENCH_NONCE_COUNTER.store(0, Ordering::SeqCst);
}
