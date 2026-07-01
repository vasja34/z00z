//! Snapshot tests for wire format serialization compatibility
//!
//! These tests ensure that changes to DefinitionWire and AssetWire
//! don't break serialization compatibility with existing data.
//!
//! Run with:
//! ```bash
//! cargo test --test assets_tests wire_format_snapshots -- --nocapture
//! ```

use std::sync::Arc;
use z00z_core::assets::{AssetClass, AssetDefinition, AssetError, AssetWire, DefinitionWire};
use z00z_crypto::expert::keys::RistrettoSecretKey;
use z00z_crypto::expert::traits::SecretKeyTrait;
use z00z_crypto::vendor::tari::PedersenCommitmentFactory;
use z00z_crypto::ZkPackEncrypted;
use z00z_crypto::{HomomorphicCommitmentFactory, Z00ZCommitment};

/// Golden snapshot for the canonical DefinitionWire format
const DEFINITION_WIRE_SNAPSHOT: &str = r#"{
  "id": [1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1],
  "class": "Coin",
  "name": "Test Coin",
  "symbol": "TEST",
  "decimals": 8,
  "serials": 100,
  "nominal": 20000,
  "domain_name": "test.coin",
  "version": 1,
  "crypto_version": 1,
  "policy_flags": 0,
  "metadata": null
}"#;

fn make_definition(
    class: AssetClass,
    name: &str,
    symbol: &str,
    decimals: u8,
    serials: u32,
    nominal: u64,
    domain_name: &str,
    metadata: Option<std::collections::BTreeMap<String, String>>,
) -> AssetDefinition {
    AssetDefinition::new(
        [0u8; 32],
        class,
        name.to_string(),
        symbol.to_string(),
        decimals,
        serials,
        nominal,
        domain_name.to_string(),
        1,
        1,
        0,
        metadata,
    )
    .expect("test definition must be canonical")
}

fn make_wire_asset(definition: Arc<AssetDefinition>) -> AssetWire {
    let factory = PedersenCommitmentFactory::default();
    let blinding = RistrettoSecretKey::from_uniform_bytes(&[5u8; 64]).unwrap();

    AssetWire {
        definition: AssetDefinition::try_from(DefinitionWire::from(&*definition))
            .expect("embedded definition wire must remain canonical"),
        serial_id: 42,
        amount: 1,
        commitment: Z00ZCommitment::from_commitment(factory.commit(&blinding, &1u64.into())),
        range_proof: Some(vec![0xDE, 0xAD, 0xBE, 0xEF]),
        nonce: [123u8; 32],
        lock_height: None,
        is_burned: false,
        is_frozen: false,
        is_slashed: false,
        owner_pub: None,
        owner_signature: None,
        r_pub: None,
        owner_tag: None,
        enc_pack: None,
        secret: None,
        tag16: None,
        leaf_ad_id: None,
    }
}

fn assert_wire_match(restored: &AssetWire, wire: &AssetWire) {
    assert_eq!(restored.definition.id, wire.definition.id);
    assert_eq!(restored.serial_id, wire.serial_id);
    assert_eq!(restored.amount, wire.amount);
    assert_eq!(restored.commitment, wire.commitment);
    assert_eq!(restored.range_proof, wire.range_proof);
    assert_eq!(restored.nonce, wire.nonce);
    assert_eq!(restored.lock_height, wire.lock_height);
    assert_eq!(restored.is_burned, wire.is_burned);
    assert_eq!(restored.is_frozen, wire.is_frozen);
    assert_eq!(restored.is_slashed, wire.is_slashed);
    assert_eq!(restored.owner_pub, wire.owner_pub);
    assert_eq!(restored.owner_signature, wire.owner_signature);
    assert_eq!(restored.r_pub, wire.r_pub);
    assert_eq!(restored.owner_tag, wire.owner_tag);
    assert_eq!(restored.enc_pack, wire.enc_pack);
    assert_eq!(restored.secret, wire.secret);
    assert_eq!(restored.tag16, wire.tag16);
    assert_eq!(restored.leaf_ad_id, wire.leaf_ad_id);
}

fn make_stealth_wire_asset(definition: Arc<AssetDefinition>) -> AssetWire {
    let mut wire = make_wire_asset(definition);
    wire.serial_id = 7;
    wire.lock_height = Some(77);
    wire.is_burned = true;
    wire.is_frozen = true;
    wire.is_slashed = true;
    wire.r_pub = Some([7u8; 32]);
    wire.owner_tag = Some([8u8; 32]);
    wire.enc_pack = Some(ZkPackEncrypted {
        version: 1,
        ciphertext: vec![1, 2, 3, 4, 5],
        tag: [9u8; 16],
    });
    wire.tag16 = Some(0xCAFE);
    wire.leaf_ad_id = Some([10u8; 32]);
    wire
}

/// Test DefinitionWire serialization stability
#[test]
fn test_definition_wire_serialization_snapshot() {
    let definition = AssetDefinition {
        id: [1u8; 32],
        class: AssetClass::Coin,
        name: "Test Coin".to_string(),
        symbol: "TEST".to_string(),
        decimals: 8,
        serials: 100,
        nominal: 20000,
        domain_name: "test.coin".to_string(),
        version: 1,
        crypto_version: 1,
        policy_flags: 0,
        metadata: None,
    };

    let wire = DefinitionWire::from(&definition);
    let serialized = serde_json::to_string_pretty(&wire).expect("Serialization should succeed");

    // Verify serialization matches snapshot
    let expected: serde_json::Value =
        serde_json::from_str(DEFINITION_WIRE_SNAPSHOT).expect("Snapshot should be valid JSON");
    let actual: serde_json::Value =
        serde_json::from_str(&serialized).expect("Serialized should be valid JSON");

    assert_eq!(
        actual,
        expected,
        "DefinitionWire serialization changed!\n\
         This is a BREAKING CHANGE that may affect stored data.\n\
         Expected:\n{}\n\nActual:\n{}",
        serde_json::to_string_pretty(&expected).unwrap(),
        serde_json::to_string_pretty(&actual).unwrap()
    );
}

/// Test DefinitionWire roundtrip (serialize → deserialize → serialize)
#[test]
fn test_definition_wire_roundtrip() {
    let original = make_definition(
        AssetClass::Token,
        "USD Stablecoin",
        "zUSD",
        6,
        50,
        100000,
        "zusd.token",
        Some([("issuer".to_string(), "Z00Z Foundation".to_string())].into()),
    );

    // Convert to wire format
    let wire1 = DefinitionWire::from(&original);

    // Serialize
    let json = serde_json::to_string(&wire1).unwrap();

    // Deserialize
    let wire2: DefinitionWire = serde_json::from_str(&json).unwrap();

    // Convert back to definition
    let restored = AssetDefinition::try_from(wire2.clone())
        .expect("definition wire roundtrip must remain canonical");

    // Verify all fields match
    assert_eq!(restored.id, original.id);
    assert_eq!(restored.class, original.class);
    assert_eq!(restored.name, original.name);
    assert_eq!(restored.symbol, original.symbol);
    assert_eq!(restored.decimals, original.decimals);
    assert_eq!(restored.serials, original.serials);
    assert_eq!(restored.nominal, original.nominal);
    assert_eq!(restored.domain_name, original.domain_name);
    assert_eq!(restored.version, original.version);
    assert_eq!(restored.crypto_version, original.crypto_version);
    assert_eq!(restored.policy_flags, original.policy_flags);
    assert_eq!(restored.metadata, original.metadata);

    // Serialize again and verify idempotence
    let wire3 = DefinitionWire::from(&restored);
    let json3 = serde_json::to_string(&wire3).unwrap();
    assert_eq!(json, json3, "Serialization must be idempotent");
}

/// Test AssetWire with embedded definition
#[test]
fn test_asset_wire_with_definition() {
    let definition = Arc::new(make_definition(
        AssetClass::Nft,
        "Event Ticket",
        "TICKET",
        0,
        1000,
        0,
        "event.tickets",
        None,
    ));
    let wire = make_wire_asset(definition.clone());

    // Verify embedded definition
    assert_eq!(wire.definition.id, definition.id);
    assert_eq!(wire.definition.class, AssetClass::Nft);
    assert_eq!(wire.definition.symbol, "TICKET");
    assert_eq!(wire.serial_id, 42);
    assert_eq!(wire.amount, 1);

    // Serialize and deserialize
    let json = serde_json::to_string(&wire).unwrap();
    let restored: AssetWire = serde_json::from_str(&json).unwrap();

    // Verify all fields survived roundtrip
    assert_wire_match(&restored, &wire);
}

#[test]
fn test_asset_stealth_fields_roundtrip() {
    let definition = Arc::new(make_definition(
        AssetClass::Token,
        "Stealth Token",
        "STLTH",
        6,
        10,
        1_000,
        "stealth.token",
        None,
    ));
    let wire = make_stealth_wire_asset(definition);

    let json = serde_json::to_string(&wire).unwrap();
    let restored: AssetWire = serde_json::from_str(&json).unwrap();

    assert_wire_match(&restored, &wire);
}

#[test]
fn test_asset_rejects_secret_field() {
    let definition = Arc::new(make_definition(
        AssetClass::Token,
        "Stealth Token",
        "STLTH",
        6,
        10,
        1_000,
        "stealth.token",
        None,
    ));
    let mut wire = make_stealth_wire_asset(definition);
    wire.secret = Some([11u8; 32]);

    let error = wire.validate().expect_err("secret must be rejected");
    assert!(matches!(
        error,
        AssetError::InvalidAsset(ref message) if message.contains("secret is forbidden")
    ));
}

/// Test flags backward compatibility
#[test]
fn test_flags_backward_compatibility() {
    // Old format (policy_flags = 3: gas + fungible)
    let old_json = r#"{
        "id": [4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4],
        "class": "Coin",
        "name": "Old Coin",
        "symbol": "OLD",
        "decimals": 8,
        "serials": 100,
        "nominal": 1000,
        "domain_name": "old.coin",
        "version": 1,
        "crypto_version": 1,
        "policy_flags": 3,
        "metadata": null
    }"#;

    let wire: DefinitionWire =
        serde_json::from_str(old_json).expect("Old format should deserialize");

    // Verify flags are parsed correctly
    let gas = wire.policy_flags & 0b0000_0001 != 0;
    let fungible = wire.policy_flags & 0b0000_0010 != 0;
    let mintable = wire.policy_flags & 0b0000_0100 != 0;

    assert!(gas, "gas flag should be set");
    assert!(fungible, "fungible flag should be set");
    assert!(!mintable, "mintable flag should NOT be set in old format");

    // New format (policy_flags = 23: gas + fungible + mintable + burnable)
    let new_json = r#"{
        "id": [5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5],
        "class": "Token",
        "name": "New Token",
        "symbol": "NEW",
        "decimals": 6,
        "serials": 50,
        "nominal": 5000,
        "domain_name": "new.token",
        "version": 1,
        "crypto_version": 1,
        "policy_flags": 23,
        "metadata": null
    }"#;

    let wire_new: DefinitionWire =
        serde_json::from_str(new_json).expect("New format should deserialize");

    let mintable_new = wire_new.policy_flags & 0b0000_0100 != 0;
    let burnable_new = wire_new.policy_flags & 0b0001_0000 != 0;
    assert!(mintable_new, "mintable flag should be set in new format");
    assert!(burnable_new, "burnable flag should be set in new format");
}

/// Test version migration (v1 → v2 future-proofing)
#[test]
fn test_version_field_preserved() {
    let wire_json = r#"{
        "id": [6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6],
        "class": "Coin",
        "name": "Version Test",
        "symbol": "VER",
        "decimals": 8,
        "serials": 10,
        "nominal": 100,
        "domain_name": "version.test",
        "version": 1,
        "crypto_version": 1,
        "policy_flags": 0,
        "metadata": null
    }"#;

    let wire: DefinitionWire = serde_json::from_str(wire_json).unwrap();
    assert_eq!(wire.version, 1, "Version field must be preserved");
    assert_eq!(wire.crypto_version, 1, "Crypto version must be preserved");

    // Roundtrip
    let json_out = serde_json::to_string(&wire).unwrap();
    let wire2: DefinitionWire = serde_json::from_str(&json_out).unwrap();
    assert_eq!(wire2.version, 1);
    assert_eq!(wire2.crypto_version, 1);
}

/// Test metadata field handling (None vs Some)
#[test]
fn test_metadata_serialization() {
    // Case 1: No metadata
    let def1 = make_definition(
        AssetClass::Token,
        "No Metadata",
        "NONE",
        6,
        10,
        100,
        "no.meta",
        None,
    );

    let wire1 = DefinitionWire::from(&def1);
    let json1 = serde_json::to_string(&wire1).unwrap();
    assert!(
        json1.contains("\"metadata\":null"),
        "metadata should be null"
    );

    // Case 2: With metadata
    let def2 = make_definition(
        AssetClass::Token,
        "With Metadata",
        "META",
        6,
        10,
        100,
        "with.meta",
        Some(
            [
                ("key1".to_string(), "value1".to_string()),
                ("key2".to_string(), "value2".to_string()),
            ]
            .into(),
        ),
    );

    let wire2 = DefinitionWire::from(&def2);
    let json2 = serde_json::to_string(&wire2).unwrap();
    assert!(
        json2.contains("\"key1\""),
        "metadata key1 should be present"
    );
    assert!(
        json2.contains("\"value1\""),
        "metadata value1 should be present"
    );

    // Roundtrip
    let restored: DefinitionWire = serde_json::from_str(&json2).unwrap();
    let def_restored = AssetDefinition::try_from(restored)
        .expect("definition wire roundtrip must remain canonical");
    assert_eq!(def_restored.metadata, def2.metadata);
}
