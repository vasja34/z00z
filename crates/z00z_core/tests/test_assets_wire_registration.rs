//! Phase 1, Test 3: AssetWire Serialization with Embedded Definition
//!
//! Tests serialization/deserialization of AssetWire with embedded AssetDefinition.
//! Verifies that definitions auto-register in global registry on deserialization
//! and that Arc<AssetDefinition> pointers are preserved.
//!
//! **Real Structures**:
//! - Asset (full asset state with commitment, range_proof, nonce)
//! - AssetWire (serializable form with embedded definition)
//! - GLOBAL_ASSET_REGISTRY (registry auto-registration)
//!
//! **Cryptography**: Real `tari_crypto` Pedersen commitments and Bulletproofs+
//!
//! Runtime: ~70 seconds (with mock_crypto feature flag)

use std::sync::Arc;
use z00z_core::assets::{
    Asset, AssetClass, AssetDefinition, AssetWire, BlindingFactor, GLOBAL_ASSET_REGISTRY,
};
use z00z_utils::rng::DeterministicRngProvider;

#[cfg(test)]
mod tests {
    use super::*;

    /// Helper: Create a full AssetDefinition with realistic parameters
    fn create_test_definition(
        asset_id: [u8; 32],
        class: AssetClass,
        name: &str,
    ) -> AssetDefinition {
        AssetDefinition::new(
            asset_id,
            class,
            name.to_string(),
            "TST".to_string(),
            8,                          // decimals
            1000,                       // serials
            100_000_000,                // nominal (1 native asset = 10^8 units)
            "test.z00z.io".to_string(), // domain_name
            1,                          // version
            1,                          // crypto_version
            0,                          // flags
            None,                       // metadata
        )
        .expect("valid test definition")
    }

    /// Helper: Create a test Asset with real cryptography
    fn create_test_asset(asset_id: [u8; 32], class: AssetClass, amount: u64) -> Asset {
        let def = create_test_definition(asset_id, class, "Test Asset");
        let arc_def = Arc::new(def);

        let blinding =
            BlindingFactor::random(&mut DeterministicRngProvider::from_seed([42u8; 32]).rng());
        let nonce = [42u8; 32];

        Asset::new(
            Arc::clone(&arc_def),
            100,
            amount,
            &blinding,
            nonce,
            &mut DeterministicRngProvider::from_seed([42u8; 32]).rng(),
        )
        .expect("asset creation should succeed")
    }

    /// Test 1: Asset → AssetWire conversion with embedded definition
    #[test]
    fn test_asset_to_wire_conversion() {
        let mut asset_id = [0u8; 32];
        asset_id[0] = 11;

        // Create Asset with real cryptography
        let asset = create_test_asset(asset_id, AssetClass::Coin, 5_000_000);

        // Verify Asset has required cryptographic components
        assert!(
            asset.range_proof().is_some(),
            "Asset should have range proof"
        );
        let original_commitment = asset.commitment().clone();

        // Convert Asset to AssetWire (embeds definition)
        let wire = AssetWire::from_asset(&asset);

        // Verify embedded definition is complete
        assert_eq!(wire.definition.id, asset.definition.id);
        assert_eq!(wire.definition.class, AssetClass::Coin);
        assert!(!wire.definition.name.is_empty());
        assert_eq!(wire.definition.decimals, 8);

        // Verify wire contains asset data
        assert_eq!(wire.serial_id, asset.serial_id());
        assert_eq!(wire.amount, asset.amount());
        assert_eq!(wire.nonce, asset.nonce().clone());

        // Verify commitment preserved
        assert_eq!(wire.commitment, original_commitment);

        println!("[OK] Asset → AssetWire conversion verified");
    }

    /// Test 2: AssetWire serialization and deserialization
    #[test]
    fn test_wire_serialization_roundtrip() {
        let mut asset_id = [0u8; 32];
        asset_id[1] = 22;

        // Create Asset
        let asset = create_test_asset(asset_id, AssetClass::Token, 10_000_000);
        let original_commitment = asset.commitment().clone();

        // Convert to AssetWire
        let wire = AssetWire::from_asset(&asset);

        // Serialize to bincode
        let bytes = bincode::serde::encode_to_vec(&wire, bincode::config::standard())
            .expect("serialization should succeed");
        let serialized_size = bytes.len();
        println!("[OK] AssetWire serialized to {} bytes", serialized_size);
        assert!(
            serialized_size < 5120,
            "serialized wire should be < 5KB, got {} bytes",
            serialized_size
        );

        // Deserialize from bincode
        let (deserialized_wire, _): (AssetWire, _) =
            bincode::serde::decode_from_slice(&bytes, bincode::config::standard())
                .expect("deserialization should succeed");

        // Verify all fields preserved
        assert_eq!(deserialized_wire.definition.id, wire.definition.id);
        assert_eq!(deserialized_wire.serial_id, wire.serial_id);
        assert_eq!(deserialized_wire.amount, wire.amount);
        assert_eq!(deserialized_wire.commitment, original_commitment);
        assert_eq!(deserialized_wire.nonce, wire.nonce);

        println!("[OK] AssetWire serialization roundtrip verified");
    }

    /// Test 3: Definition auto-registration on deserialization
    #[test]
    fn test_definition_auto_register() {
        let mut asset_id = [0u8; 32];
        asset_id[2] = 33;

        // Create Asset
        let asset = create_test_asset(asset_id, AssetClass::Coin, 7_500_000);

        // Convert to AssetWire
        let wire = AssetWire::from_asset(&asset);
        let definition_id = wire.definition.id;

        // Serialize
        let bytes = bincode::serde::encode_to_vec(&wire, bincode::config::standard())
            .expect("serialization should succeed");

        // Deserialize (this should auto-register the definition)
        let (deserialized_wire, _): (AssetWire, _) =
            bincode::serde::decode_from_slice(&bytes, bincode::config::standard())
                .expect("deserialization should succeed");

        // Convert back to Asset (uses registry)
        let reconstructed = deserialized_wire
            .to_asset()
            .expect("asset reconstruction should succeed");

        // Verify definition was auto-registered
        assert!(GLOBAL_ASSET_REGISTRY
            .contains(&definition_id)
            .expect("registry check failed"));
        assert_eq!(reconstructed.definition.id, definition_id);

        // Verify Arc pointer points to registry definition
        let registry_def = GLOBAL_ASSET_REGISTRY
            .get(&definition_id)
            .expect("definition should be in registry")
            .expect("registry entry should contain definition");
        assert!(
            Arc::ptr_eq(&reconstructed.definition, &registry_def),
            "Asset definition pointer should match registry"
        );

        println!("[OK] Definition auto-registration verified");
    }

    /// Test 4: Arc<AssetDefinition> pointer identity preservation
    #[test]
    fn test_arc_pointer_identity() {
        let mut asset_id = [0u8; 32];
        asset_id[3] = 44;

        // Create Asset
        let asset = create_test_asset(asset_id, AssetClass::Coin, 12_500_000);

        // Roundtrip through AssetWire
        let wire = AssetWire::from_asset(&asset);
        let definition_id = wire.definition.id;
        let bytes = bincode::serde::encode_to_vec(&wire, bincode::config::standard())
            .expect("serialization should succeed");
        let (deserialized_wire, _): (AssetWire, _) =
            bincode::serde::decode_from_slice(&bytes, bincode::config::standard())
                .expect("deserialization should succeed");

        // Reconstruct Asset
        let reconstructed = deserialized_wire
            .to_asset()
            .expect("asset reconstruction should succeed");

        // Both reconstructed and registry lookup should share the same Arc definition
        // (original asset's Arc is different because it was created fresh, not from registry)
        let registry_def = GLOBAL_ASSET_REGISTRY.get(&definition_id).unwrap().unwrap();

        // Verify reconstructed asset definition matches registry
        assert!(
            Arc::ptr_eq(&reconstructed.definition, &registry_def),
            "Reconstructed asset definition should match registry definition"
        );

        // Verify the definitions are semantically equal (same id, name, etc.)
        assert_eq!(asset.definition.id, reconstructed.definition.id);
        assert_eq!(asset.definition.name, reconstructed.definition.name);
        assert_eq!(asset.definition.decimals, reconstructed.definition.decimals);

        println!("[OK] Arc pointer identity verified");
    }

    /// Test 5: Batch AssetWire serialization (20 assets - reduced for speed)
    #[test]
    fn test_batch_wire_serialization() {
        let mut total_size = 0u64;
        let num_assets = 20;

        for i in 0..num_assets {
            let mut asset_id = [0u8; 32];
            asset_id[4] = i as u8; // i % 256
            asset_id[5] = (i / 256) as u8; // i / 256

            // Create Asset with varying amounts
            let amount = 1_000_000 * ((i as u64) + 1);
            let asset = create_test_asset(asset_id, AssetClass::Coin, amount);

            // Convert and serialize
            let wire = AssetWire::from_asset(&asset);
            let bytes = bincode::serde::encode_to_vec(&wire, bincode::config::standard())
                .expect("serialization should succeed");

            // Deserialize and verify
            let (deserialized_wire, _): (AssetWire, _) =
                bincode::serde::decode_from_slice(&bytes, bincode::config::standard())
                    .expect("deserialization should succeed");

            // Reconstruct asset
            let reconstructed = deserialized_wire
                .to_asset()
                .expect("asset reconstruction should succeed");

            // Verify critical fields
            assert_eq!(reconstructed.definition.id, asset.definition.id);
            assert_eq!(reconstructed.amount(), amount);

            total_size += bytes.len() as u64;
        }

        let avg_size = total_size / num_assets as u64;
        println!(
            "[OK] Phase 1, Test 3 batch verified: {} assets, avg size {} bytes",
            num_assets, avg_size
        );

        // Verify average size is reasonable
        assert!(
            avg_size < 5120,
            "average wire size should be < 5KB, got {} bytes",
            avg_size
        );
    }

    /// Test 6: Range proof preservation through serialization
    #[test]
    fn test_proof_preservation() {
        let mut asset_id = [0u8; 32];
        asset_id[6] = 66;

        // Create Asset
        let asset = create_test_asset(asset_id, AssetClass::Coin, 15_000_000);

        // Verify original has range proof
        assert!(
            asset.range_proof().is_some(),
            "Original asset should have range proof"
        );

        // Convert to AssetWire
        let wire = AssetWire::from_asset(&asset);

        // Verify wire includes proof
        assert!(
            wire.range_proof.is_some(),
            "AssetWire should include range proof"
        );

        // Serialize and deserialize
        let bytes = bincode::serde::encode_to_vec(&wire, bincode::config::standard())
            .expect("serialization should succeed");
        let (deserialized_wire, _): (AssetWire, _) =
            bincode::serde::decode_from_slice(&bytes, bincode::config::standard())
                .expect("deserialization should succeed");

        // Verify proof survived serialization
        assert!(
            deserialized_wire.range_proof.is_some(),
            "Deserialized wire should have range proof"
        );

        // Reconstruct Asset
        let reconstructed = deserialized_wire
            .to_asset()
            .expect("asset reconstruction should succeed");

        // Verify reconstructed asset has proof
        assert!(
            reconstructed.range_proof().is_some(),
            "Reconstructed asset should have range proof"
        );

        println!("[OK] Range proof preservation verified");
    }
}
