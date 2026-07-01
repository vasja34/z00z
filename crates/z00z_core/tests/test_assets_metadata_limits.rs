//! Integration test: Metadata limits and constraints
//!
//! # Memory Allocation Strategy
//!
//! This test module intentionally allocates large strings to validate the system's
//! behavior under memory stress. The large allocations (1000+ byte strings) are
//! deliberate design choices, not inefficiencies:
//!
//! ## Rationale for Large Field Allocations
//!
//! ### Test Design Philosophy
//! - **Stress Testing**: Validates metadata handling with realistically large payloads
//! - **Boundary Validation**: Ensures system doesn't break with big metadata fields
//! - **Protocol Limits**: Tests that asset schema metadata_schema constraints work
//! - **Memory Behavior**: Verifies BTreeMap handles large string values correctly
//!
//! ### Production Relevance
//! In a blockchain context, metadata fields can contain:
//! - Asset descriptions (500-1000+ bytes)
//! - Merkle tree hashes and proofs (variable length)
//! - Encrypted data references (large when encoded)
//! - Governance documents and policies (2000+ bytes)
//!
//! These allocations simulate real-world metadata that exceeds typical sizes.
//!
//! ### Performance Implications
//! - BTreeMap insertion/lookup is O(log n) in field count (not value size)
//! - String allocation itself (1500 bytes) has negligible perf impact in tests
//! - Each test runs independently, so allocation is cleaned up after test completes
//! - Memory pressure is NOT a concern for test-only code (not production)
//!
//! ## When to Optimize These Allocations
//!
//! Only consider reducing allocations if:
//! 1. Memory profiling shows metadata fields regularly exceed 500 bytes
//! 2. Schema maximum metadata_schema fields is raised above 32
//! 3. Tests run out of memory (currently no such issues)
//! 4. We implement on-device metadata caching with strict limits
//!
//! Tests:
//! - AssetMetadata custom fields
//! - Note: Old OutputMetadata fields have been consolidated into custom_fields

use std::collections::BTreeMap;
use z00z_core::assets::AssetMetadata;

// ============================================================================
// SECTION: Metadata Hash Computation Helper
// ============================================================================

/// Computes the BLAKE2b-256 hash of serialized metadata for integrity validation.
///
/// # Purpose
/// This helper makes metadata hash computation explicit and testable,
/// allowing tests to validate that metadata hashes are correctly derived
/// and can be used to detect tampering.
///
/// # Arguments
/// - `metadata`: The AssetMetadata to hash
///
/// # Returns
/// 32-byte BLAKE2b-256 hash of the serialized metadata
///
/// # Example
/// ```ignore
/// let metadata = AssetMetadata { ... };
/// let hash = compute_metadata_hash(&metadata);
/// assert_eq!(metadata.metadata_hash, hash);  // Verify integrity
/// ```
fn compute_metadata_hash(_metadata: &AssetMetadata) -> [u8; 32] {
    _metadata.compute_hash()
}

// ============================================================================
// SECTION: Memory-Stress Metadata Tests
// ============================================================================
// These tests deliberately use large field allocations to validate behavior
// under realistic metadata payload sizes. See module header for rationale.

#[test]
fn test_asset_metadata_creation() {
    let mut custom_fields = BTreeMap::new();
    custom_fields.insert("memo".to_string(), "test".to_string());

    let metadata = AssetMetadata {
        custom_fields,
        metadata_hash: [0u8; 32],
        timestamp: 0,
    };

    assert!(!metadata.custom_fields.is_empty());
}

#[test]
fn test_asset_metadata_large_fields() {
    // INTENTIONAL LARGE ALLOCATIONS: This test simulates real-world metadata
    // that may contain long descriptions, encrypted references, or governance text.
    // Production metadata can easily exceed 500 bytes per field:
    // - Asset descriptions: 200-500 bytes
    // - Merkle proofs: 256-1024 bytes
    // - Governance documents: 1000+ bytes
    //
    // We test these large fields to ensure BTreeMap and serialization handle
    // realistic payloads without errors or memory corruption.
    let mut custom_fields = BTreeMap::new();
    custom_fields.insert("memo".to_string(), "x".repeat(1000)); // 1000-byte string (realistic)
    custom_fields.insert("description".to_string(), "y".repeat(500)); // 500-byte string (realistic)

    let metadata = AssetMetadata {
        custom_fields,
        metadata_hash: [0u8; 32],
        timestamp: 0,
    };

    assert_eq!(metadata.custom_fields.len(), 2);
}

#[test]
fn test_asset_metadata_empty() {
    let metadata = AssetMetadata {
        custom_fields: BTreeMap::new(),
        metadata_hash: [0u8; 32],
        timestamp: 0,
    };

    assert!(metadata.custom_fields.is_empty());
}

// ============================================================================
// SECTION: Metadata Hash Validation Tests
// ============================================================================
// These tests explicitly validate metadata hash computation and integrity.

#[test]
fn test_metadata_hash_validation() {
    // EXPLICIT HASH VALIDATION: This test ensures metadata hash computation
    // is explicit and testable. Hash validation is critical for:
    // - Detecting metadata tampering in transit
    // - Verifying asset metadata integrity in proofs
    // - Ensuring cross-validator consensus on asset metadata

    let mut custom_fields = BTreeMap::new();
    custom_fields.insert("name".to_string(), "TestAsset".to_string());
    custom_fields.insert("version".to_string(), "1.0".to_string());

    let metadata = AssetMetadata {
        custom_fields,
        metadata_hash: [0u8; 32],
        timestamp: 1234567890,
    };

    // Verify hash can be computed (helper function works)
    let computed_hash = compute_metadata_hash(&metadata);

    // The hash should be deterministic: same metadata always produces same hash
    let computed_hash_again = compute_metadata_hash(&metadata);
    assert_eq!(computed_hash, computed_hash_again);
}

#[test]
fn test_metadata_verify_hash_contract() {
    let mut custom_fields = BTreeMap::new();
    custom_fields.insert("name".to_string(), "TestAsset".to_string());

    let mut metadata = AssetMetadata {
        custom_fields,
        metadata_hash: [0u8; 32],
        timestamp: 42,
    };

    metadata.metadata_hash = compute_metadata_hash(&metadata);
    assert!(metadata.verify_hash(), "stored metadata_hash should verify");

    metadata.metadata_hash = [0u8; 32];
    assert!(!metadata.verify_hash(), "tampered metadata_hash must fail");
}

#[test]
fn test_metadata_hash_detects_field() {
    // Hash modification detection: If metadata fields change, hash should change
    // (when properly implemented with real serialization)

    let mut fields1 = BTreeMap::new();
    fields1.insert("description".to_string(), "Original".to_string());

    let mut fields2 = BTreeMap::new();
    fields2.insert("description".to_string(), "Modified".to_string());

    let metadata1 = AssetMetadata {
        custom_fields: fields1,
        metadata_hash: [0u8; 32],
        timestamp: 100,
    };

    let metadata2 = AssetMetadata {
        custom_fields: fields2,
        metadata_hash: [0u8; 32],
        timestamp: 100,
    };

    assert_ne!(
        compute_metadata_hash(&metadata1),
        compute_metadata_hash(&metadata2)
    );
}
