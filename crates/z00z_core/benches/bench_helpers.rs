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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bench_nonce_uniqueness() {
        reset_bench_nonce_counter();

        let asset_id = [1u8; 32];
        let nonce1 = bench_nonce(&asset_id);
        let nonce2 = bench_nonce(&asset_id);
        let nonce3 = bench_nonce(&asset_id);

        assert_ne!(nonce1, nonce2);
        assert_ne!(nonce2, nonce3);
        assert_ne!(nonce1, nonce3);
    }

    #[test]
    fn test_bench_nonce_not_zero() {
        let asset_id = [2u8; 32];
        let nonce = bench_nonce(&asset_id);
        assert_ne!(nonce, [0u8; 32], "Benchmark nonces must never be zero");
    }

    #[test]
    fn test_bench_nonces_batch() {
        reset_bench_nonce_counter();

        let asset_id = [3u8; 32];
        let nonces = bench_nonces(&asset_id, 100);

        assert_eq!(nonces.len(), 100);

        // Verify all unique
        for i in 0..nonces.len() {
            for j in (i + 1)..nonces.len() {
                assert_ne!(nonces[i], nonces[j], "Nonces at {} and {} must be unique", i, j);
            }
        }
    }
}
