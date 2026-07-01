// crates/z00z_core/src/assets/snapshot.rs
//
//! Registry Snapshot - Immutable snapshots for Validator → Wallet communication
//!
//! Provides [`RegistrySnapshot`] and [`RegistryVersion`] for versioning and integrity
//! checking of asset definition registry updates.
//!
//! ## Usage Example
//!
//! ```ignore
//! use z00z_core::assets::registry::AssetDefinitionRegistry;
//! use z00z_core::assets::snapshot::RegistrySnapshot;
//! use z00z_utils::prelude::{save_json, load_json};
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! // Create snapshot from registry
//! let registry = AssetDefinitionRegistry::new();
//! let snapshot = registry.to_snapshot();
//!
//! // Serialize to disk
//! save_json(&snapshot, "data/registry_snapshot.json")?;
//!
//! // Load from disk
//! let loaded: RegistrySnapshot = load_json("data/registry_snapshot.json")?;
//!
//! // Verify version integrity
//! assert_eq!(snapshot.version, loaded.version);
//! # Ok(())
//! # }
//! ```
//!
//! ## I/O Operations
//!
//! Use `z00z_utils::prelude::{load_json, save_json}` directly for snapshot I/O.
//! No wrapper functions needed - use the utils directly.
//!
//! This module contains only data structures and versioning logic.

use crate::domains::RegistryHashDomain;
use z00z_crypto::DomainHasher;
use z00z_utils::time::{SystemTimeProvider, TimeProvider};

#[allow(unused_imports)] // Used in #[cfg(test)]
use super::policy_flags::BURNABLE;
use super::wire::DefinitionWire;

// ============================================================================
// Hash Domain for Registry Versioning
// ============================================================================

// ============================================================================
// Registry Versioning
// ============================================================================

/// Registry version identifier
///
/// Tracks version, integrity hash, and timestamp of asset definition registry.
/// Used to validate updates from validator and prevent downgrade attacks.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
pub struct RegistryVersion {
    /// Sequential version number (increments on each update)
    pub version: u64,

    /// Blake2b-256 hash of the ordered canonical definition payload list
    pub hash: [u8; 32],

    /// Timestamp when this version was created (Unix milliseconds)
    pub timestamp: u64,
}

impl RegistryVersion {
    /// Calculate hash from the ordered canonical definition payload list
    ///
    /// # Arguments
    /// * `version` - Snapshot version paired with the definitions
    /// * `definitions` - Ordered registry definitions in wire format
    ///
    /// # Returns
    /// 32-byte Blake2b hash
    ///
    /// # Examples
    /// ```rust
    /// # use z00z_core::assets::{snapshot::RegistryVersion, AssetClass};
    /// # use z00z_core::assets::wire::DefinitionWire;
    /// let definitions = vec![DefinitionWire {
    ///     id: [1u8; 32],
    ///     class: AssetClass::Coin,
    ///     name: "Test".into(),
    ///     symbol: "TST".into(),
    ///     decimals: 8,
    ///     serials: 1,
    ///     nominal: 1,
    ///     domain_name: "test.io".into(),
    ///     version: 1,
    ///     crypto_version: 1,
    ///     policy_flags: 0,
    ///     metadata: None,
    /// }];
    /// let hash = RegistryVersion::compute_hash(1, &definitions);
    /// assert_eq!(hash.len(), 32);
    /// ```
    pub fn compute_hash(version: u64, definitions: &[DefinitionWire]) -> [u8; 32] {
        let mut hasher = DomainHasher::<RegistryHashDomain>::new_with_label("registry_hash");
        hasher.update(version.to_le_bytes());
        for definition in definitions {
            let payload = definition.payload_bytes();
            hasher.update((payload.len() as u32).to_le_bytes());
            hasher.update(payload);
        }
        let hash = hasher.finalize();
        let mut result = [0u8; 32];
        result.copy_from_slice(&hash.as_ref()[..32]);
        result
    }

    /// Get current Unix timestamp using TimeProvider
    ///
    /// Uses SystemTimeProvider for timestamp generation following ONE SOURCE OF TRUTH principle.
    ///
    /// # Examples
    /// ```rust
    /// # use z00z_core::assets::snapshot::RegistryVersion;
    /// let timestamp = RegistryVersion::now();
    /// assert!(timestamp > 0);
    /// ```
    pub fn now() -> u64 {
        let time_provider = SystemTimeProvider;
        time_provider.compat_unix_timestamp_millis()
    }
}

// ============================================================================
// Registry Snapshot
// ============================================================================

/// Complete registry snapshot
///
/// Immutable snapshot of entire asset definition registry.
/// This is what Validator node sends to Wallet client.
///
/// Contains:
/// - Version information (version number, hash, timestamp)
/// - All definitions in wire format (serde-friendly)
///
/// The snapshot can be:
/// - Serialized to JSON/Bincode for network transmission
/// - Validated using hash and version
/// - Applied atomically to update local registry
///
/// # Examples
/// ```rust,ignore
/// # use z00z_core::assets::snapshot::{RegistrySnapshot, RegistryVersion};
/// let snapshot = RegistrySnapshot {
///     version: RegistryVersion {
///         version: 1,
///         hash: [0u8; 32],
///         timestamp: 1234567890,
///     },
///     definitions: vec![],
/// };
/// ```
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct RegistrySnapshot {
    /// Version metadata (version number, hash, timestamp)
    pub version: RegistryVersion,

    /// All asset definitions in wire format
    pub definitions: Vec<DefinitionWire>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::assets::AssetClass;

    fn test_wire(id: [u8; 32], name: &str, symbol: &str, flags: u8) -> DefinitionWire {
        DefinitionWire {
            id,
            class: AssetClass::Coin,
            name: name.to_string(),
            symbol: symbol.to_string(),
            decimals: 8,
            serials: 1000,
            nominal: 100_000,
            domain_name: "test.io".to_string(),
            version: 1,
            crypto_version: 1,
            policy_flags: flags,
            metadata: None,
        }
    }

    #[test]
    fn test_registry_version_hash() {
        let definitions = vec![
            test_wire([1u8; 32], "One", "ONE", 0),
            test_wire([2u8; 32], "Two", "TWO", 0),
            test_wire([3u8; 32], "Three", "THR", 0),
        ];
        let hash1 = RegistryVersion::compute_hash(1, &definitions);
        let hash2 = RegistryVersion::compute_hash(1, &definitions);

        // Same input → same hash
        assert_eq!(hash1, hash2);
    }

    #[test]
    fn test_registry_version_hash_different() {
        let definitions1 = vec![
            test_wire([1u8; 32], "One", "ONE", 0),
            test_wire([2u8; 32], "Two", "TWO", 0),
        ];
        let definitions2 = vec![
            test_wire([1u8; 32], "One", "ONE", 0),
            test_wire([2u8; 32], "Two", "TWO", 0),
            test_wire([3u8; 32], "Three", "THR", 0),
        ];

        let hash1 = RegistryVersion::compute_hash(1, &definitions1);
        let hash2 = RegistryVersion::compute_hash(1, &definitions2);

        // Different input → different hash
        assert_ne!(hash1, hash2);
    }

    #[test]
    fn test_registry_version_payload_drift() {
        let base = test_wire([7u8; 32], "Seven", "SVN", 0);
        let mut changed = base.clone();
        changed.symbol = "ALT".to_string();

        let hash1 = RegistryVersion::compute_hash(1, &[base]);
        let hash2 = RegistryVersion::compute_hash(1, &[changed]);

        assert_ne!(hash1, hash2);
    }

    #[test]
    fn test_registry_version_name_drift() {
        let base = test_wire([8u8; 32], "Eight", "EIT", 0);
        let mut changed = base.clone();
        changed.name = "Eight Alt".to_string();

        let hash1 = RegistryVersion::compute_hash(1, &[base]);
        let hash2 = RegistryVersion::compute_hash(1, &[changed]);

        assert_ne!(hash1, hash2);
    }

    #[test]
    fn test_registry_version_flag_drift() {
        let base = test_wire([9u8; 32], "Nine", "NIN", 0);
        let mut changed = base.clone();
        changed.policy_flags = BURNABLE;

        let hash1 = RegistryVersion::compute_hash(1, &[base]);
        let hash2 = RegistryVersion::compute_hash(1, &[changed]);

        assert_ne!(hash1, hash2);
    }

    #[test]
    fn test_registry_version_metadata_drift() {
        let base = test_wire([10u8; 32], "Ten", "TEN", 0);
        let mut changed = base.clone();
        changed.metadata = Some(std::collections::BTreeMap::from([(
            "issuer".to_string(),
            "validator".to_string(),
        )]));

        let hash1 = RegistryVersion::compute_hash(1, &[base]);
        let hash2 = RegistryVersion::compute_hash(1, &[changed]);

        assert_ne!(hash1, hash2);
    }

    #[test]
    fn test_registry_version_sorted_payloads() {
        let mut left = vec![
            test_wire([3u8; 32], "Three", "THR", 0),
            test_wire([1u8; 32], "One", "ONE", 0),
            test_wire([2u8; 32], "Two", "TWO", 0),
        ];
        let mut right = left.clone();

        left.sort_by_key(|wire| wire.id);
        right.sort_by_key(|wire| wire.id);

        let hash1 = RegistryVersion::compute_hash(1, &left);
        let hash2 = RegistryVersion::compute_hash(1, &right);

        assert_eq!(hash1, hash2);
    }

    #[test]
    fn test_registry_version_version_drift() {
        let definitions = vec![test_wire([11u8; 32], "Eleven", "ELV", 0)];

        let hash1 = RegistryVersion::compute_hash(1, &definitions);
        let hash2 = RegistryVersion::compute_hash(2, &definitions);

        assert_ne!(hash1, hash2);
    }

    #[test]
    fn test_registry_version_timestamp() {
        let now = RegistryVersion::now();
        assert!(now > 0);
    }

    #[test]
    fn test_definition_wire_serialization() {
        let wire = DefinitionWire {
            id: [42u8; 32],
            class: AssetClass::Coin,
            name: "Test".to_string(),
            symbol: "TST".to_string(),
            decimals: 8,
            serials: 1000,
            nominal: 100_000,
            domain_name: "test.io".to_string(),
            version: 1,
            crypto_version: 1,
            policy_flags: BURNABLE, // burnable flag
            metadata: None,
        };

        use z00z_utils::codec::{Codec, JsonCodec};
        let codec = JsonCodec;
        let json = codec.serialize(&wire).unwrap();
        let deserialized: DefinitionWire = codec.deserialize(&json).unwrap();

        assert_eq!(wire.id, deserialized.id);
        assert_eq!(wire.name, deserialized.name);
    }

    #[test]
    fn test_snapshot_serialization() {
        use z00z_utils::codec::{Codec, JsonCodec};

        let snapshot = RegistrySnapshot {
            version: RegistryVersion {
                version: 1,
                hash: [0u8; 32],
                timestamp: 1234567890,
            },
            definitions: vec![],
        };

        let codec = JsonCodec;
        let json = codec.serialize(&snapshot).unwrap();
        let deserialized: RegistrySnapshot = codec.deserialize(&json).unwrap();

        assert_eq!(snapshot.version.version, deserialized.version.version);
        assert_eq!(snapshot.definitions.len(), deserialized.definitions.len());
    }
}
