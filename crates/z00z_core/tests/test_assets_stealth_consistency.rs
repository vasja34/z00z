#![cfg(not(target_arch = "wasm32"))]

use z00z_core::assets::{AssetError, AssetWire};

#[test]
fn test_non_stealth_wire_ok() {
    let asset = z00z_core::genesis::asset_std::asset_from_dev_class(
        z00z_core::assets::AssetClass::Coin,
        1,
        10,
    )
    .expect("valid std asset");

    let wire = AssetWire::from_asset(&asset);
    let rebuilt = wire.to_asset().expect("non-stealth wire must import");

    assert!(rebuilt.validate_stealth_consistency().is_ok());
    assert!(rebuilt.is_transparent());
}

#[test]
fn test_partial_stealth_reject() {
    let asset = z00z_core::genesis::asset_std::asset_from_dev_class(
        z00z_core::assets::AssetClass::Coin,
        2,
        10,
    )
    .expect("valid std asset");

    let mut wire = AssetWire::from_asset(&asset);
    wire.r_pub = Some([9u8; 32]);
    wire.owner_tag = None;
    wire.enc_pack = None;

    let err = wire.to_asset().expect_err("partial stealth must reject");
    assert!(matches!(err, AssetError::InvalidStealth(_)));
}

#[test]
fn test_full_stealth_need_tag() {
    let asset = z00z_core::genesis::asset_std::asset_from_dev_class(
        z00z_core::assets::AssetClass::Coin,
        3,
        10,
    )
    .expect("valid std asset");

    let mut wire = AssetWire::from_asset(&asset);
    wire.r_pub = Some([1u8; 32]);
    wire.owner_tag = Some([2u8; 32]);
    wire.enc_pack = Some(z00z_crypto::ZkPackEncrypted {
        version: 1,
        ciphertext: vec![3u8; 8],
        tag: [0u8; 16],
    });
    wire.tag16 = None;

    let err = wire
        .to_asset()
        .expect_err("full stealth without tag16 must reject");
    assert!(matches!(err, AssetError::InvalidStealth(_)));
}

#[test]
fn test_needs_full_ad_id() {
    let asset = z00z_core::genesis::asset_std::asset_from_dev_class(
        z00z_core::assets::AssetClass::Coin,
        4,
        10,
    )
    .expect("valid std asset");

    let mut wire = AssetWire::from_asset(&asset);
    wire.r_pub = Some([1u8; 32]);
    wire.owner_tag = Some([2u8; 32]);
    wire.enc_pack = Some(z00z_crypto::ZkPackEncrypted {
        version: 1,
        ciphertext: vec![3u8; 8],
        tag: [0u8; 16],
    });
    wire.tag16 = Some(7);
    wire.leaf_ad_id = None;

    let err = wire
        .to_asset()
        .expect_err("full stealth without leaf_ad_id must reject");
    assert!(matches!(err, AssetError::InvalidStealth(_)));
}
