//! Phase 1, Test 10: Wallet Recovery Nonce Reproduction
//!
//! This test verifies that wallet nonce sequences can be exactly reproduced
//! from the original wallet seed, which is critical for:
//! - Wallet recovery after data loss
//! - Multi-device wallet synchronization
//! - Asset history reconstruction
//! - Cold storage wallet operations
//!
//! All cryptography uses real Blake2b hashing from tari_crypto.

use std::collections::BTreeSet;
use z00z_crypto::expert::hash_domain;
use z00z_crypto::DomainHasher;

hash_domain!(TestNonceDomain, "z00z.core.tests.nonce.v1", 1);

#[cfg(test)]
mod tests {
    use super::*;

    /// Type alias for nonce (32-byte array)
    type Nonce = [u8; 32];

    // ============================================================================
    // Helper Functions
    // ============================================================================

    /// Create a test wallet seed (32 random bytes)
    fn create_wallet_seed() -> [u8; 32] {
        [1u8; 32] // Fixed seed for reproducible tests
    }

    /// Create an alternate wallet seed for multi-wallet tests
    fn create_alternate_wallet_seed() -> [u8; 32] {
        [2u8; 32]
    }

    /// Create a test asset ID for nonce derivation
    fn create_test_asset_id() -> [u8; 32] {
        [7u8; 32]
    }

    /// Create an alternate asset ID
    fn create_alternate_asset_id() -> [u8; 32] {
        [8u8; 32]
    }

    /// Derive a nonce from wallet seed, counter, and asset ID using domain-separated Blake2b
    ///
    /// This mimics the real `derive_nonce()` function from the assets module.
    /// Nonce = DomainSeparatedHasher(seed || counter || asset_id)
    fn derive_nonce_from_seed(seed: &[u8; 32], counter: u64, asset_id: &[u8; 32]) -> Nonce {
        let hash = DomainHasher::<TestNonceDomain>::new_with_label("test_nonce")
            .chain(seed)
            .chain(counter.to_le_bytes())
            .chain(asset_id)
            .finalize();

        let mut nonce = [0u8; 32];
        nonce.copy_from_slice(&hash.as_ref()[..32]);
        nonce
    }

    // ============================================================================
    // Test 1: Basic Wallet Recovery - 100 Assets
    // ============================================================================

    /// Test that nonces can be exactly reproduced from wallet seed
    ///
    /// Scenario: User creates 100 assets, then loses wallet data
    /// Expected: Regenerating from seed produces identical nonces
    #[test]
    fn test_wallet_recovery_100_assets() {
        let wallet_seed = create_wallet_seed();
        let asset_id = create_test_asset_id();

        // Phase 1: Initial wallet - generate 100 nonces
        let mut original_nonces = Vec::new();
        for counter in 0..100 {
            let nonce = derive_nonce_from_seed(&wallet_seed, counter, &asset_id);
            original_nonces.push(nonce);
        }

        assert_eq!(original_nonces.len(), 100, "Should generate 100 nonces");

        // Phase 2: Wallet recovery - regenerate nonces from same seed
        let mut recovered_nonces = Vec::new();
        for counter in 0..100 {
            let nonce = derive_nonce_from_seed(&wallet_seed, counter, &asset_id);
            recovered_nonces.push(nonce);
        }

        // Phase 3: Verify exact match
        assert_eq!(
            original_nonces.len(),
            recovered_nonces.len(),
            "Should recover same number of nonces"
        );

        for (i, (original, recovered)) in original_nonces
            .iter()
            .zip(recovered_nonces.iter())
            .enumerate()
        {
            assert_eq!(
                original, recovered,
                "Nonce {} should match exactly after recovery",
                i
            );
        }

        println!("[OK] Wallet recovery: 100 assets verified");
    }

    // ============================================================================
    // Test 2: Large-Scale Wallet Recovery - 1,000 Assets
    // ============================================================================

    /// Test wallet recovery with larger asset count (1,000)
    ///
    /// Scenario: Enterprise wallet with 1,000 assets
    /// Expected: All nonces reproducible, deterministic
    #[test]
    fn test_wallet_recovery_1000_assets() {
        let wallet_seed = create_wallet_seed();
        let asset_id = create_test_asset_id();

        // Phase 1: Generate 1,000 original nonces
        let original_nonces: Vec<Nonce> = (0..1_000)
            .map(|counter| derive_nonce_from_seed(&wallet_seed, counter, &asset_id))
            .collect();

        // Phase 2: Recover nonces from same seed
        let recovered_nonces: Vec<Nonce> = (0..1_000)
            .map(|counter| derive_nonce_from_seed(&wallet_seed, counter, &asset_id))
            .collect();

        // Phase 3: Verify all match
        assert_eq!(original_nonces.len(), 1_000);
        assert_eq!(recovered_nonces.len(), 1_000);

        for (i, (original, recovered)) in original_nonces
            .iter()
            .zip(recovered_nonces.iter())
            .enumerate()
        {
            assert_eq!(original, recovered, "Nonce {} should match exactly", i);
        }

        println!("[OK] Wallet recovery: 1,000 assets verified");
    }

    // ============================================================================
    // Test 3: Multi-Device Wallet Sync - Same Seed Different Devices
    // ============================================================================

    /// Test that different devices can sync wallet using same seed
    ///
    /// Scenario:
    /// - Device A (laptop) creates assets with nonces 0-50
    /// - Device B (phone) syncs from seed, generates same nonces
    /// - Devices exchange transaction history without conflicts
    #[test]
    fn test_wallet_multi_device_sync() {
        let wallet_seed = create_wallet_seed();
        let asset_id = create_test_asset_id();

        // Device A: Generate nonces for assets 0-50
        let device_a_nonces: Vec<Nonce> = (0..50)
            .map(|counter| derive_nonce_from_seed(&wallet_seed, counter, &asset_id))
            .collect();

        // Device B: Sync from seed, regenerate nonces for same range
        let device_b_nonces: Vec<Nonce> = (0..50)
            .map(|counter| derive_nonce_from_seed(&wallet_seed, counter, &asset_id))
            .collect();

        // Verify devices have identical nonces
        assert_eq!(
            device_a_nonces, device_b_nonces,
            "Devices should sync perfectly"
        );

        // Verify uniqueness within device
        let unique_nonces: BTreeSet<Nonce> = device_a_nonces.iter().cloned().collect();
        assert_eq!(
            unique_nonces.len(),
            50,
            "All 50 nonces should be unique within device"
        );

        println!("[OK] Multi-device wallet sync verified");
    }

    // ============================================================================
    // Test 4: Recovery with Custom Counter Start
    // ============================================================================

    /// Test wallet recovery starting from arbitrary counter value
    ///
    /// Scenario: User remembers wallet had 500 assets, starts recovery from counter 500
    /// Expected: Nonces 500+ are reproducible correctly
    #[test]
    fn test_wallet_recovery_custom_counter() {
        let wallet_seed = create_wallet_seed();
        let asset_id = create_test_asset_id();

        // Generate nonces from counter 500 to 550
        let start_counter = 500u64;
        let count = 50;

        let original_nonces: Vec<Nonce> = (start_counter..start_counter + count)
            .map(|counter| derive_nonce_from_seed(&wallet_seed, counter, &asset_id))
            .collect();

        // Recover same range from seed
        let recovered_nonces: Vec<Nonce> = (start_counter..start_counter + count)
            .map(|counter| derive_nonce_from_seed(&wallet_seed, counter, &asset_id))
            .collect();

        // Verify match
        assert_eq!(
            original_nonces, recovered_nonces,
            "Custom range should match exactly"
        );

        // Also verify they differ from nonces at counter 0-49
        let early_nonces: Vec<Nonce> = (0..50)
            .map(|counter| derive_nonce_from_seed(&wallet_seed, counter, &asset_id))
            .collect();

        assert_ne!(
            original_nonces[0], early_nonces[0],
            "Counter 500 nonce should differ from counter 0 nonce"
        );

        println!("[OK] Custom counter start recovery verified");
    }

    // ============================================================================
    // Test 5: Multi-Asset Wallet - Different Asset IDs
    // ============================================================================

    /// Test wallet managing multiple asset types (BTC, ETH, etc)
    ///
    /// Scenario:
    /// - Same wallet seed
    /// - Different asset IDs for different asset types
    /// - All nonces reproducible per asset type
    #[test]
    fn test_wallet_multi_asset_types() {
        let wallet_seed = create_wallet_seed();
        let asset_id_1 = create_test_asset_id();
        let asset_id_2 = create_alternate_asset_id();

        // Generate nonces for asset type 1 (0-50)
        let asset1_nonces: Vec<Nonce> = (0..50)
            .map(|counter| derive_nonce_from_seed(&wallet_seed, counter, &asset_id_1))
            .collect();

        // Generate nonces for asset type 2 (0-50)
        let asset2_nonces: Vec<Nonce> = (0..50)
            .map(|counter| derive_nonce_from_seed(&wallet_seed, counter, &asset_id_2))
            .collect();

        // Asset types should have different nonces (different asset IDs)
        assert_ne!(
            asset1_nonces[0], asset2_nonces[0],
            "Asset types should have different nonces"
        );

        // Recover asset1 nonces
        let recovered_asset1: Vec<Nonce> = (0..50)
            .map(|counter| derive_nonce_from_seed(&wallet_seed, counter, &asset_id_1))
            .collect();

        assert_eq!(
            asset1_nonces, recovered_asset1,
            "Asset type 1 nonces should be recoverable"
        );

        // Recover asset2 nonces
        let recovered_asset2: Vec<Nonce> = (0..50)
            .map(|counter| derive_nonce_from_seed(&wallet_seed, counter, &asset_id_2))
            .collect();

        assert_eq!(
            asset2_nonces, recovered_asset2,
            "Asset type 2 nonces should be recoverable"
        );

        println!("[OK] Multi-asset wallet recovery verified");
    }

    // ============================================================================
    // Test 6: Multi-Wallet Isolation - Different Seeds
    // ============================================================================

    /// Test that different wallet seeds produce completely different nonces
    ///
    /// Scenario:
    /// - User A's wallet seed
    /// - User B's wallet seed
    /// - No nonce collision possible between wallets
    #[test]
    fn test_wallet_multi_wallet_isolation() {
        let wallet_seed_a = create_wallet_seed();
        let wallet_seed_b = create_alternate_wallet_seed();
        let asset_id = create_test_asset_id();

        // Generate nonces for wallet A
        let wallet_a_nonces: Vec<Nonce> = (0..100)
            .map(|counter| derive_nonce_from_seed(&wallet_seed_a, counter, &asset_id))
            .collect();

        // Generate nonces for wallet B
        let wallet_b_nonces: Vec<Nonce> = (0..100)
            .map(|counter| derive_nonce_from_seed(&wallet_seed_b, counter, &asset_id))
            .collect();

        // Check for any collisions between wallets
        let wallet_a_set: BTreeSet<Nonce> = wallet_a_nonces.iter().cloned().collect();
        let wallet_b_set: BTreeSet<Nonce> = wallet_b_nonces.iter().cloned().collect();

        let intersection: BTreeSet<_> = wallet_a_set.intersection(&wallet_b_set).cloned().collect();

        assert_eq!(
            intersection.len(),
            0,
            "Different wallet seeds should have no nonce collisions"
        );

        println!("[OK] Multi-wallet isolation verified");
    }

    // ============================================================================
    // Test 7: Cold Storage Recovery - Batch Reproduction
    // ============================================================================

    /// Test cold storage wallet recovery scenario
    ///
    /// Scenario:
    /// - User has seed in cold storage (written on paper)
    /// - Wants to verify wallet state without touching hot wallet
    /// - Recovers 5 different asset types, 100 nonces each
    #[test]
    fn test_wallet_cold_storage_recovery() {
        let wallet_seed = create_wallet_seed();

        // 5 different asset IDs representing different asset types
        let asset_ids: Vec<[u8; 32]> = vec![[1u8; 32], [2u8; 32], [3u8; 32], [4u8; 32], [5u8; 32]];

        let mut all_recovered_nonces = Vec::new();

        // Recover nonces for each asset type
        for asset_id in &asset_ids {
            let nonces: Vec<Nonce> = (0..100)
                .map(|counter| derive_nonce_from_seed(&wallet_seed, counter, asset_id))
                .collect();

            all_recovered_nonces.extend(nonces);
        }

        // Should have 500 total nonces (5 assets × 100 nonces)
        assert_eq!(
            all_recovered_nonces.len(),
            500,
            "Should recover 500 total nonces"
        );

        // Verify all are unique across assets
        let unique_nonces: BTreeSet<Nonce> = all_recovered_nonces.iter().cloned().collect();
        assert_eq!(
            unique_nonces.len(),
            500,
            "All 500 recovered nonces should be unique"
        );

        println!("[OK] Cold storage recovery verified (500 nonces, 5 assets)");
    }

    // ============================================================================
    // Test 8: Wallet Recovery Consistency Over Time
    // ============================================================================

    /// Test that wallet recovery is consistent across multiple recovery attempts
    ///
    /// Scenario: User recovers wallet at different times, should get identical results
    /// Expected: No randomness, completely deterministic
    #[test]
    fn test_wallet_recovery_consistency() {
        let wallet_seed = create_wallet_seed();
        let asset_id = create_test_asset_id();

        // First recovery attempt
        let recovery1: Vec<Nonce> = (0..100)
            .map(|counter| derive_nonce_from_seed(&wallet_seed, counter, &asset_id))
            .collect();

        // Simulate time passing, then second recovery attempt
        std::thread::sleep(std::time::Duration::from_millis(10));

        let recovery2: Vec<Nonce> = (0..100)
            .map(|counter| derive_nonce_from_seed(&wallet_seed, counter, &asset_id))
            .collect();

        // Third recovery attempt
        std::thread::sleep(std::time::Duration::from_millis(10));

        let recovery3: Vec<Nonce> = (0..100)
            .map(|counter| derive_nonce_from_seed(&wallet_seed, counter, &asset_id))
            .collect();

        // All three should be identical (no randomness)
        assert_eq!(recovery1, recovery2, "Recovery 1 and 2 should match");
        assert_eq!(recovery2, recovery3, "Recovery 2 and 3 should match");
        assert_eq!(recovery1, recovery3, "Recovery 1 and 3 should match");

        println!("[OK] Wallet recovery consistency verified (3 attempts identical)");
    }

    // ============================================================================
    // Test 9: Wallet Recovery with Large Counter Range
    // ============================================================================

    /// Test wallet recovery for high counter values (simulating old wallet)
    ///
    /// Scenario: User has old wallet that used counters 10,000+
    /// Expected: Can still recover exact nonces from counter 10,000 onwards
    #[test]
    fn test_wallet_recovery_high_counter() {
        let wallet_seed = create_wallet_seed();
        let asset_id = create_test_asset_id();

        // Generate nonces at very high counter values
        let base_counter = 10_000u64;
        let count = 50;

        let original_nonces: Vec<Nonce> = (base_counter..base_counter + count)
            .map(|counter| derive_nonce_from_seed(&wallet_seed, counter, &asset_id))
            .collect();

        // Recover same nonces
        let recovered_nonces: Vec<Nonce> = (base_counter..base_counter + count)
            .map(|counter| derive_nonce_from_seed(&wallet_seed, counter, &asset_id))
            .collect();

        // Must match exactly
        assert_eq!(
            original_nonces, recovered_nonces,
            "High counter nonces should be exactly reproducible"
        );

        // Verify they differ from lower counter values
        let low_counter_nonce = derive_nonce_from_seed(&wallet_seed, 0, &asset_id);
        let high_counter_nonce = derive_nonce_from_seed(&wallet_seed, base_counter, &asset_id);

        assert_ne!(
            low_counter_nonce, high_counter_nonce,
            "Counter values should produce different nonces"
        );

        println!("[OK] High counter wallet recovery verified");
    }

    // ============================================================================
    // Test 10: Wallet Recovery Nonce Uniqueness After Recovery
    // ============================================================================

    /// Test that recovered nonces maintain uniqueness properties
    ///
    /// Scenario: After wallet recovery, all recovered nonces should be unique
    /// Expected: No collisions in recovered nonces
    #[test]
    fn test_wallet_recovery_uniqueness() {
        let wallet_seed = create_wallet_seed();
        let asset_id = create_test_asset_id();

        // Recover a large batch of nonces
        let recovered_nonces: Vec<Nonce> = (0..1_000)
            .map(|counter| derive_nonce_from_seed(&wallet_seed, counter, &asset_id))
            .collect();

        // Verify all unique
        let unique_nonces: BTreeSet<Nonce> = recovered_nonces.iter().cloned().collect();
        assert_eq!(
            unique_nonces.len(),
            1_000,
            "All 1,000 recovered nonces should be unique"
        );

        // Verify none are zero
        for (i, nonce) in recovered_nonces.iter().enumerate() {
            assert_ne!(
                *nonce, [0u8; 32],
                "Nonce {} should not be zero after recovery",
                i
            );
        }

        println!("[OK] Wallet recovery uniqueness verified (1,000 nonces unique)");
    }
}
