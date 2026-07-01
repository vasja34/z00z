//! Multi-Asset Genesis Integration Test
//!
//! Verifies AssetDefinition validation for all 4 asset types (native asset, token, NFT, void).
//! Genesis itself only generates native assets, but this test validates that
//! the asset system can handle all asset classes correctly.

use z00z_core::assets::{AssetClass, AssetDefinition};

#[test]
fn test_asset_definition_coin_validation() {
    let coin_id = [0x01; 32];
    let coin_def = AssetDefinition::new(
        coin_id,
        AssetClass::Coin,
        "TestCoin".to_string(),
        "TCN".to_string(),
        8,
        1000,
        100_000_000,
        "test.z00z".to_string(),
        1,
        1,
        0,
        None,
    );
    assert!(coin_def.is_ok(), "Native asset definition should be valid");

    let def = coin_def.unwrap();
    assert_eq!(def.class, AssetClass::Coin);
    assert_eq!(def.decimals, 8);
}

#[test]
fn test_asset_definition_token_validation() {
    let token_id = [0x02; 32];
    let token_def = AssetDefinition::new(
        token_id,
        AssetClass::Token,
        "TestToken".to_string(),
        "TTK".to_string(),
        6,
        500,
        1_000_000,
        "token.z00z".to_string(),
        1,
        1,
        0,
        None,
    );
    assert!(token_def.is_ok(), "Token asset definition should be valid");

    let def = token_def.unwrap();
    assert_eq!(def.class, AssetClass::Token);
    assert_eq!(def.decimals, 6);
}

#[test]
fn test_asset_definition_nft_validation() {
    let nft_id = [0x03; 32];
    let nft_def = AssetDefinition::new(
        nft_id,
        AssetClass::Nft,
        "TestNFT".to_string(),
        "NFT".to_string(),
        0,
        1,
        1,
        "nft.z00z".to_string(),
        1,
        1,
        0,
        None,
    );
    assert!(
        nft_def.is_ok(),
        "NFT asset definition should be valid with 0 decimals"
    );

    let def = nft_def.unwrap();
    assert_eq!(def.class, AssetClass::Nft);
    assert_eq!(def.decimals, 0, "NFTs must have 0 decimals");
}

#[test]
fn test_asset_definition_void_validation() {
    let void_id = [0x04; 32];
    let void_def = AssetDefinition::new(
        void_id,
        AssetClass::Void,
        "TestVoid".to_string(),
        "VD".to_string(),
        0,
        1,
        0,
        "void.z00z".to_string(),
        1,
        1,
        0,
        None,
    );
    assert!(
        void_def.is_ok(),
        "Void asset definition should be valid with 0 decimals"
    );

    let def = void_def.unwrap();
    assert_eq!(def.class, AssetClass::Void);
    assert_eq!(def.decimals, 0, "Void outputs must have 0 decimals");
}

#[test]
fn test_asset_class_uniqueness() {
    // Verify all asset classes are distinct
    assert_ne!(AssetClass::Coin, AssetClass::Token);
    assert_ne!(AssetClass::Coin, AssetClass::Nft);
    assert_ne!(AssetClass::Coin, AssetClass::Void);
    assert_ne!(AssetClass::Token, AssetClass::Nft);
    assert_ne!(AssetClass::Token, AssetClass::Void);
    assert_ne!(AssetClass::Nft, AssetClass::Void);
}
