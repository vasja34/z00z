// ═════════════════════════════════════════════════════════════════════════════
// SHARED TEST FIXTURES AND HELPERS FOR ASSET INTEGRATION TESTS
// ═════════════════════════════════════════════════════════════════════════════
//
// This module centralizes common test utilities to follow DRY principle and ensure
// consistency across all asset lifecycle and validation tests.
//
// # Helper Function Conventions
//
// All helper functions follow standardized naming patterns (see HELPERS.md):
// - **Random**: `random_<type>()` returns deterministic values (seeded RNG)
// - **Factory**: `create_<class>_asset()` creates assets of each class
// - **Assertion**: `assert_<property>()` validates recurring patterns
//
// This root-level module is the canonical helper owner for asset integration tests.
//
// # Module Contents
//
// - **Constants**: `CHAIN_ID`, `DOMAIN_TAG` (protocol configuration)
// - **Helpers**: Random number generation with seeded RNG for reproducibility
// - **Factories**: Asset creation functions for each asset class

use std::sync::Arc;
use z00z_core::assets::{AssetClass, AssetDefinition, AssetDefinitionRegistry};
use z00z_core::BlindingFactor;
use z00z_crypto::expert::hash_domain;
use z00z_utils::prelude::{DeterministicRngProvider, NoopLogger, NoopMetrics, SystemTimeProvider};

// ============================================================================
// Test Registry Helper — For Integration Tests
// ============================================================================

/// Create a test asset definition registry with appropriate dependencies.
///
/// Uses NoopLogger, NoopMetrics, SystemTimeProvider for consistent testing.
pub fn create_test_registry() -> AssetDefinitionRegistry {
    AssetDefinitionRegistry::new(
        Arc::new(NoopLogger),
        Arc::new(NoopMetrics),
        Arc::new(SystemTimeProvider),
    )
}

// Load asset definitions from YAML config file for testing.
//
// ============================================================================
// Protocol Constants — Shared Across All Tests
// ============================================================================

/// Global chain ID used consistently across all asset tests.
/// Prevents accidental cross-chain asset ID collisions.
pub const CHAIN_ID: [u8; 32] = [
    0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0a, 0x0b, 0x0c, 0x0d, 0x0e, 0x0f, 0x10,
    0x11, 0x12, 0x13, 0x14, 0x15, 0x16, 0x17, 0x18, 0x19, 0x1a, 0x1b, 0x1c, 0x1d, 0x1e, 0x1f, 0x20,
];

// Domain separation for asset test operations.
hash_domain!(AssetTestDomain, "z00z.core.assets.test.v1", 1);

/// Global domain separation string used consistently across all asset tests.
/// Ensures asset ID derivation doesn't collide with other hash domains.
pub const DOMAIN: &[u8] = b"z00z.core.assets.test.v1";

// ============================================================================
// Seeded Random Number Generation — Deterministic & Reproducible
// ============================================================================

/// Seed for all deterministic RNG operations in tests.
/// Using a fixed seed ensures test reproducibility across runs and environments.
/// All tests will generate identical random values, making debugging easier.
///
/// # Reproducibility Guarantee
/// With this fixed seed, test results are identical:
/// - Across different machines
/// - Across different runs on same machine
/// - Across different test execution orders (when run independently)
const TEST_RNG_SEED: [u8; 32] = [
    0x01, 0x23, 0x45, 0x67, 0x89, 0xab, 0xcd, 0xef, 0x01, 0x23, 0x45, 0x67, 0x89, 0xab, 0xcd, 0xef,
    0x01, 0x23, 0x45, 0x67, 0x89, 0xab, 0xcd, 0xef, 0x01, 0x23, 0x45, 0x67, 0x89, 0xab, 0xcd, 0xef,
];

/// Generates a random blinding factor with seeded RNG for reproducibility.
///
/// # Determinism
/// Returns the same sequence of blinding factors on every test run.
/// This enables deterministic test results and easier debugging.
///
/// # Implementation Note
/// Uses a static counter to ensure each call produces a DIFFERENT value
/// while maintaining determinism across test runs.
///
/// # Usage
/// ```ignore
/// let blinding1 = random_blinding();  // First unique value
/// let blinding2 = random_blinding();  // Second unique value (different from first)
/// ```
pub fn random_blinding() -> BlindingFactor {
    use std::sync::atomic::{AtomicU64, Ordering};
    static COUNTER: AtomicU64 = AtomicU64::new(0);

    let counter = COUNTER.fetch_add(1, Ordering::SeqCst);
    let mut seed = TEST_RNG_SEED;
    // Mix counter into seed to get different RNG state
    seed[0] ^= (counter & 0xFF) as u8;
    seed[1] ^= ((counter >> 8) & 0xFF) as u8;
    seed[2] ^= ((counter >> 16) & 0xFF) as u8;
    seed[3] ^= ((counter >> 24) & 0xFF) as u8;

    let provider = DeterministicRngProvider::from_seed(seed);
    let mut rng = provider.rng();
    BlindingFactor::random(&mut rng)
}

/// Generates a random nonce with seeded RNG for reproducibility.
///
/// # Determinism
/// Returns the same sequence of nonces on every test run.
/// Guarantees consistent test behavior across environments.
///
/// # Implementation Note
/// Uses a static counter to ensure each call produces a DIFFERENT value
/// while maintaining determinism across test runs.
///
/// # Warning
/// The seed is fixed, so the sequence is predictable. This is intentional for testing.
/// In production, use `OsRng` instead.
pub fn random_nonce() -> [u8; 32] {
    use rand::RngCore;
    use std::sync::atomic::{AtomicU64, Ordering};
    static COUNTER: AtomicU64 = AtomicU64::new(1000); // Different start to avoid collision

    let counter = COUNTER.fetch_add(1, Ordering::SeqCst);
    let mut seed = TEST_RNG_SEED;
    // Mix counter into seed to get different RNG state
    seed[4] ^= (counter & 0xFF) as u8;
    seed[5] ^= ((counter >> 8) & 0xFF) as u8;
    seed[6] ^= ((counter >> 16) & 0xFF) as u8;
    seed[7] ^= ((counter >> 24) & 0xFF) as u8;

    let mut nonce = [0u8; 32];
    let provider = DeterministicRngProvider::from_seed(seed);
    let mut rng = provider.rng();
    rng.fill_bytes(&mut nonce);
    nonce
}

// ============================================================================
// Asset Factory Functions — Canonical Creation Methods
// ============================================================================

/// Creates a native asset for testing.
///
/// # Parameters
/// - `asset_id`: 32-byte unique identifier
/// - `decimals`: Decimal places for amount representation (typically 8 for native assets)
///
/// # Returns
/// A `Coin` class asset with standard protocol parameters.
///
/// # Example
/// ```ignore
/// let coin_asset = create_coin_asset([0u8; 32], 8);
/// assert_eq!(coin_asset.definition.class, AssetClass::Coin);
/// assert_eq!(coin_asset.definition.decimals, 8);
/// ```
pub fn create_coin_asset(asset_id: [u8; 32], decimals: u8) -> AssetDefinition {
    let tag = format!("{:02x}{:02x}", asset_id[0], asset_id[1]);
    AssetDefinition::new(
        asset_id,
        AssetClass::Coin,
        format!("Test Coin {tag}"),
        format!("TC{tag}"),
        decimals,
        1,                             // serials
        1,                             // nominal
        format!("coin_{tag}.test.io"), // domain_name
        1,                             // version
        1,                             // crypto_version
        0,                             // flags
        None,                          // metadata
    )
    .expect("failed to create native asset definition")
}

// Removed: create_test_def - was unused (shadowed by local functions in other test files)

/// Creates a fungible token asset for testing.
///
/// # Parameters
/// - `asset_id`: 32-byte unique identifier
/// - `decimals`: Decimal places (0-18 typical range)
///
/// # Returns
/// A `Token` class asset with custom decimal granularity.
///
/// # Note
/// Tokens support arbitrary decimal precision, unlike Void assets which must be indivisible.
pub fn create_token_asset(asset_id: [u8; 32], decimals: u8) -> AssetDefinition {
    let tag = format!("{:02x}{:02x}", asset_id[0], asset_id[1]);
    AssetDefinition::new(
        asset_id,
        AssetClass::Token,
        format!("Test Token {tag}"),
        format!("TT{tag}"),
        decimals,
        1, // serials
        1, // nominal
        format!("token_{tag}.test.io"),
        1, // version
        1, // crypto_version
        0, // flags
        None,
    )
    .expect("failed to create token asset definition")
}

/// Creates a non-fungible token asset for testing.
///
/// # Parameters
/// - `asset_id`: 32-byte unique identifier
/// - `serial_id`: Optional unique instance number (1..=50000 typical)
///
/// # Returns
/// An `Nft` class asset with decimals forced to 0 (indivisible).
///
/// # Constraints
/// - Decimals always 0 (NFTs are indivisible per spec §2.1)
/// - Optional serial_id for numbered token instances
///
/// # Example
/// ```ignore
/// let nft = create_nft_asset([1u8; 32], Some(1));
/// assert_eq!(nft.decimals, 0);
/// ```
pub fn create_nft_asset(asset_id: [u8; 32], serial_id: Option<u32>) -> AssetDefinition {
    let serial = serial_id.unwrap_or(0);
    let tag = format!("{:02x}{:02x}", asset_id[0], asset_id[1]);

    AssetDefinition::new(
        asset_id,
        AssetClass::Nft,
        format!("Test NFT {tag}"),
        format!("TN{tag}"),
        0,          // decimals - must be 0 for NFT
        serial + 1, // serials - at least serial+1 to allow this serial_id
        1,          // nominal
        format!("nft_{tag}.test.io"),
        1, // version
        1, // crypto_version
        0, // flags
        None,
    )
    .expect("failed to create NFT asset definition")
}

/// Creates a void asset (protocol sink) for testing.
///
/// # Parameters
/// - `asset_id`: 32-byte unique identifier
///
/// # Returns
/// A `Void` class asset with decimals forced to 0 (never holds value).
///
/// # Constraints
/// - Decimals always 0 (Void outputs never contain value)
/// - Used only as burn sinks or fee collectors (spec §2.1)
///
/// # Example
/// ```ignore
/// let void_sink = create_void_asset([2u8; 32]);
/// assert_eq!(void_sink.class, AssetClass::Void);
/// assert_eq!(void_sink.decimals, 0);
/// ```
pub fn create_void_asset(asset_id: [u8; 32]) -> AssetDefinition {
    let tag = format!("{:02x}{:02x}", asset_id[0], asset_id[1]);
    AssetDefinition::new(
        asset_id,
        AssetClass::Void,
        format!("Test Void {tag}"),
        format!("TV{tag}"),
        0, // decimals - must be 0 for Void
        1, // serials
        1, // nominal
        format!("void_{tag}.test.io"),
        1, // version
        1, // crypto_version
        0, // flags
        None,
    )
    .expect("failed to create Void asset definition")
}

/// Creates the native Z00Z asset used in integration tests.
///
/// This is the default native asset for testing protocols that require it.
/// Useful as a standard reference throughout test suite.
///
/// # Returns
/// A `Coin` class asset with 8 decimal places (satoshi-like precision).
pub fn native_coin_asset() -> AssetDefinition {
    create_coin_asset([0u8; 32], 8)
}

// ============================================================================
// Test Utilities
// ============================================================================

/// Gets the global test domain separation string.
///
/// Used consistently across asset ID derivation tests to prevent collisions
/// with other cryptographic domains in the protocol.
pub fn test_domain() -> &'static [u8] {
    DOMAIN
}

/// Gets the global test chain ID.
///
/// Used for asset ID derivation and validation tests that depend on
/// network identity.
pub fn test_chain_id() -> [u8; 32] {
    CHAIN_ID
}

// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_domain_chain_id_helpers() {
        assert_eq!(test_domain(), DOMAIN);
        assert_eq!(test_chain_id(), CHAIN_ID);
    }

    #[test]
    fn test_fixtures_deterministic_blinding() {
        // Same seed should produce same blinding factors across runs
        let b1 = random_blinding();
        let b2 = random_blinding();

        // Different values from same seed (sequential generation)
        // Note: In a real seeded RNG, these should be different
        // but generated from same seed state progression
        assert_ne!(b1.as_bytes(), b2.as_bytes());
    }

    #[test]
    fn test_fixtures_deterministic_nonce() {
        let n1 = random_nonce();
        let n2 = random_nonce();

        // Both are valid 32-byte arrays
        assert_eq!(n1.len(), 32);
        assert_eq!(n2.len(), 32);

        // Different nonces generated from same seed
        assert_ne!(n1, n2);
    }

    #[test]
    fn test_create_coin_asset() {
        let asset = create_coin_asset([1u8; 32], 8);
        let same_asset = create_coin_asset([1u8; 32], 8);
        assert_eq!(asset.class, AssetClass::Coin);
        assert_eq!(asset.decimals, 8);
        assert_eq!(asset.id, same_asset.id);
    }

    #[test]
    fn test_create_token_asset() {
        let asset = create_token_asset([2u8; 32], 6);
        assert_eq!(asset.class, AssetClass::Token);
        assert_eq!(asset.decimals, 6);
    }

    #[test]
    fn test_create_nft_asset() {
        let asset = create_nft_asset([3u8; 32], Some(42));
        assert_eq!(asset.class, AssetClass::Nft);
        assert_eq!(asset.decimals, 0);
        assert_eq!(asset.serials, 43); // serial_id + 1
    }

    #[test]
    fn test_create_void_asset() {
        let asset = create_void_asset([4u8; 32]);
        assert_eq!(asset.class, AssetClass::Void);
        assert_eq!(asset.decimals, 0);
    }

    #[test]
    fn test_native_coin_asset() {
        let coin = native_coin_asset();
        assert_eq!(coin.class, AssetClass::Coin);
        assert_eq!(coin.decimals, 8);
    }
}
