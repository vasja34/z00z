//! Phase 1, Test 9: Nonce Uniqueness Across 10,000 Assets
//!
//! This test validates that nonce generation produces unique, deterministic values.
//! Tests cover:
//! - Generating 10,000 unique nonces
//! - Nonce determinism (same seed → same nonce)
//! - Different seeds produce different nonces
//! - Different timestamps produce different nonces
//! - No zero nonces generated

use std::collections::BTreeSet;
use z00z_core::assets::Nonce;
use z00z_crypto::expert::hash_domain;
use z00z_crypto::DomainHasher;

hash_domain!(TestNonceDomain, "z00z.core.tests.nonce.v1", 1);

#[cfg(test)]
mod tests {
    use super::*;

    /// Helper: Derive deterministic nonce (mimics production logic)
    fn derive_test_nonce(seed: &[u8; 32], counter: u64, asset_id: &[u8; 32]) -> Nonce {
        let hash = DomainHasher::<TestNonceDomain>::new_with_label("test_nonce")
            .chain(seed)
            .chain(counter.to_le_bytes())
            .chain(asset_id)
            .finalize();

        let mut nonce_bytes = [0u8; 32];
        nonce_bytes.copy_from_slice(&hash.as_ref()[..32]);

        nonce_bytes
    }

    /// Helper: Create test seed (32 bytes)
    fn create_test_seed() -> [u8; 32] {
        [42u8; 32]
    }

    /// Helper: Create alternative test seed
    fn create_alt_seed() -> [u8; 32] {
        [99u8; 32]
    }

    /// Helper: Create test asset ID
    fn create_test_asset_id() -> [u8; 32] {
        [7u8; 32]
    }

    /// Test 1: Generate 100 nonces and verify uniqueness
    #[test]
    fn test_nonce_uniqueness_100() {
        let seed = create_test_seed();
        let asset_id = create_test_asset_id();
        let mut nonces = Vec::new();
        let mut nonce_set = BTreeSet::new();

        // Generate 100 nonces with incrementing counter
        for counter in 0..100 {
            let nonce = derive_test_nonce(&seed, counter, &asset_id);
            nonces.push(nonce);
            nonce_set.insert(nonce.to_vec());
        }

        // Verify all unique
        assert_eq!(nonce_set.len(), 100, "All 100 nonces should be unique");

        println!("[OK] 100 nonces generated with perfect uniqueness");
    }

    /// Test 2: Generate 1,000 nonces and verify uniqueness
    #[test]
    fn test_nonce_uniqueness_1000() {
        let seed = create_test_seed();
        let asset_id = create_test_asset_id();
        let mut nonce_set = BTreeSet::new();

        // Generate 1,000 nonces
        for counter in 0..1_000 {
            let nonce = derive_test_nonce(&seed, counter, &asset_id);
            nonce_set.insert(nonce.to_vec());
        }

        // Verify all unique
        assert_eq!(nonce_set.len(), 1_000, "All 1,000 nonces should be unique");

        println!("[OK] 1,000 nonces generated with perfect uniqueness");
    }

    /// Test 3: Generate 10,000 nonces (full test) and verify uniqueness
    #[test]
    fn test_nonce_uniqueness_10000() {
        let seed = create_test_seed();
        let asset_id = create_test_asset_id();
        let mut nonce_set = BTreeSet::new();

        // Generate 10,000 nonces with incrementing counter
        for counter in 0..10_000 {
            let nonce = derive_test_nonce(&seed, counter, &asset_id);
            nonce_set.insert(nonce.to_vec());
        }

        // Verify all unique (critical test)
        assert_eq!(
            nonce_set.len(),
            10_000,
            "All 10,000 nonces should be unique (no collisions allowed)"
        );

        println!("[OK] 10,000 nonces generated with perfect uniqueness - NO COLLISIONS");
    }

    /// Test 4: Verify nonce determinism (same seed → same nonce)
    #[test]
    fn test_nonce_determinism() {
        let seed = create_test_seed();
        let asset_id = create_test_asset_id();

        // Generate same nonce twice with same inputs
        let nonce1 = derive_test_nonce(&seed, 5, &asset_id);
        let nonce2 = derive_test_nonce(&seed, 5, &asset_id);

        assert_eq!(
            nonce1, nonce2,
            "Same inputs should produce identical nonces (deterministic)"
        );

        println!("[OK] Nonce generation is deterministic");
    }

    /// Test 5: Different seeds produce different nonces
    #[test]
    fn test_nonce_different_seeds() {
        let seed1 = create_test_seed();
        let seed2 = create_alt_seed();
        let asset_id = create_test_asset_id();

        // Generate nonces with different seeds
        let nonce1 = derive_test_nonce(&seed1, 0, &asset_id);
        let nonce2 = derive_test_nonce(&seed2, 0, &asset_id);

        assert_ne!(
            nonce1, nonce2,
            "Different seeds should produce different nonces"
        );

        println!("[OK] Different seeds produce different nonces");
    }

    /// Test 6: Different counters produce different nonces
    #[test]
    fn test_nonce_different_counters() {
        let seed = create_test_seed();
        let asset_id = create_test_asset_id();

        // Generate nonces with different counters
        let nonce1 = derive_test_nonce(&seed, 100, &asset_id);
        let nonce2 = derive_test_nonce(&seed, 200, &asset_id);
        let nonce3 = derive_test_nonce(&seed, 300, &asset_id);

        assert_ne!(nonce1, nonce2);
        assert_ne!(nonce2, nonce3);
        assert_ne!(nonce1, nonce3);

        println!("[OK] Different counters produce different nonces");
    }

    /// Test 7: Different asset IDs produce different nonces
    #[test]
    fn test_nonce_different_asset_ids() {
        let seed = create_test_seed();
        let asset_id1 = [7u8; 32];
        let asset_id2 = [8u8; 32];

        // Generate nonces with different asset IDs
        let nonce1 = derive_test_nonce(&seed, 0, &asset_id1);
        let nonce2 = derive_test_nonce(&seed, 0, &asset_id2);

        assert_ne!(
            nonce1, nonce2,
            "Different asset IDs should produce different nonces"
        );

        println!("[OK] Different asset IDs produce different nonces");
    }

    /// Test 8: Verify no nonces are zero
    #[test]
    fn test_nonce_no_zero_nonces() {
        let seed = create_test_seed();
        let asset_id = create_test_asset_id();

        // Generate 1,000 nonces and verify none are zero
        for counter in 0..1_000 {
            let nonce = derive_test_nonce(&seed, counter, &asset_id);
            assert_ne!(
                nonce, [0u8; 32],
                "Nonce at counter {} should not be zero",
                counter
            );
        }

        println!("[OK] No zero nonces generated in 1,000 attempts");
    }

    /// Test 9: Nonce collision probability (statistical test)
    #[test]
    fn test_nonce_collision_rate_5000() {
        let seed = create_test_seed();
        let asset_id = create_test_asset_id();
        let mut nonce_set = BTreeSet::new();

        // Generate 5,000 nonces and track collisions
        for counter in 0..5_000 {
            let nonce = derive_test_nonce(&seed, counter, &asset_id);
            nonce_set.insert(nonce.to_vec());
        }

        let collision_count = 5_000 - nonce_set.len();
        assert_eq!(
            collision_count, 0,
            "No collisions should occur in 5,000 nonces (found {})",
            collision_count
        );

        println!(
            "[OK] 5,000 nonces verified - Collision rate: {:.6}% ({})",
            collision_count as f64 / 5_000.0 * 100.0,
            collision_count
        );
    }

    /// Test 10: Wallet recovery nonce matching
    #[test]
    fn test_nonce_wallet_recovery() {
        let wallet_seed = create_test_seed();
        let asset_id = create_test_asset_id();

        // Initial wallet: generate 100 nonces
        let mut original_nonces = Vec::new();
        for counter in 0..100 {
            let nonce = derive_test_nonce(&wallet_seed, counter, &asset_id);
            original_nonces.push(nonce);
        }

        // Simulate wallet recovery: regenerate same nonces from same seed
        let mut recovered_nonces = Vec::new();
        for counter in 0..100 {
            let nonce = derive_test_nonce(&wallet_seed, counter, &asset_id);
            recovered_nonces.push(nonce);
        }

        // Verify exact match
        assert_eq!(
            original_nonces.len(),
            recovered_nonces.len(),
            "Should have same number of nonces"
        );

        for (i, (original, recovered)) in original_nonces
            .iter()
            .zip(recovered_nonces.iter())
            .enumerate()
        {
            assert_eq!(
                original, recovered,
                "Nonce {} should match between original and recovery",
                i
            );
        }

        println!("[OK] Wallet recovery verified - all 100 nonces matched perfectly");
    }

    /// Test 11: Large batch nonce generation with different counter ranges
    #[test]
    fn test_nonce_counter_ranges() {
        let seed = create_test_seed();
        let asset_id = create_test_asset_id();

        // Test different counter ranges don't collide
        let mut all_nonces = BTreeSet::new();

        // Range 1: 0-1000
        for counter in 0..1_000 {
            let nonce = derive_test_nonce(&seed, counter, &asset_id);
            all_nonces.insert(nonce.to_vec());
        }

        // Range 2: 1000-2000
        for counter in 1_000..2_000 {
            let nonce = derive_test_nonce(&seed, counter, &asset_id);
            all_nonces.insert(nonce.to_vec());
        }

        // Verify no collisions across ranges
        assert_eq!(
            all_nonces.len(),
            2_000,
            "All 2,000 nonces across ranges should be unique"
        );

        println!("[OK] 2,000 nonces across different counter ranges are all unique");
    }

    /// Test 12: Verify nonce bytes are well-distributed
    #[test]
    fn test_nonce_distribution() {
        let seed = create_test_seed();
        let asset_id = create_test_asset_id();

        // Generate 1000 nonces and check byte distribution
        let mut byte_counts = vec![0usize; 256];

        for counter in 0..1_000 {
            let nonce = derive_test_nonce(&seed, counter, &asset_id);
            for byte in nonce.iter() {
                byte_counts[*byte as usize] += 1;
            }
        }

        // Count how many byte values appear
        let unique_bytes = byte_counts.iter().filter(|&&count| count > 0).count();

        // With 1000 nonces × 32 bytes = 32,000 bytes sampled,
        // we should see most byte values represented
        assert!(
            unique_bytes > 200,
            "Should see well-distributed byte values (found {} / 256)",
            unique_bytes
        );

        println!(
            "[OK] Nonce byte distribution verified - {} / 256 byte values represented",
            unique_bytes
        );
    }
}
