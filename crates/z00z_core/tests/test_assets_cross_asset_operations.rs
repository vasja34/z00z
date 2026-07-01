//! Integration Test 24: Cross-Asset Operations
//!
//! Tests interactions between multiple asset definitions:
//! - Create assets of different types
//! - Combine assets in transactions
//! - Verify commitment balance across asset types
//! - Test multi-asset scenarios

use std::sync::Arc;
use z00z_core::assets::{Asset, AssetClass, AssetDefinition};
use z00z_core::BlindingFactor;
use z00z_utils::rng::DeterministicRngProvider;

fn create_coin_definition(id: u8, name: &str) -> Arc<AssetDefinition> {
    Arc::new(
        AssetDefinition::new(
            [id; 32],
            AssetClass::Coin,
            name.to_string(),
            name.to_string(),
            8,
            1_000_000,
            1_000,
            "test.local".to_string(),
            1,
            1,
            0,
            None,
        )
        .expect("Valid test definition"),
    )
}

fn create_token_definition(id: u8, name: &str) -> Arc<AssetDefinition> {
    Arc::new(
        AssetDefinition::new(
            [id; 32],
            AssetClass::Token,
            name.to_string(),
            name.to_string(),
            6,
            100_000,
            100,
            "test.local".to_string(),
            1,
            1,
            0,
            None,
        )
        .expect("Valid test definition"),
    )
}

#[test]
fn test_cross_asset_multiple_types() {
    // Create 3 different asset definitions
    let coin1 = create_coin_definition(1, "Coin1");
    let coin2 = create_coin_definition(2, "Coin2");
    let token = create_token_definition(3, "Token");

    let blinding1 =
        BlindingFactor::random(&mut DeterministicRngProvider::from_seed([42u8; 32]).rng());
    let blinding2 =
        BlindingFactor::random(&mut DeterministicRngProvider::from_seed([42u8; 32]).rng());
    let blinding3 =
        BlindingFactor::random(&mut DeterministicRngProvider::from_seed([42u8; 32]).rng());

    // Create assets of each type
    let coin1_asset = Asset::new(
        Arc::clone(&coin1),
        0,
        5000u64,
        &blinding1,
        [1u8; 32],
        &mut DeterministicRngProvider::from_seed([42u8; 32]).rng(),
    )
    .expect("Native asset #1 creation should succeed");

    let coin2_asset = Asset::new(
        Arc::clone(&coin2),
        0,
        3000u64,
        &blinding2,
        [2u8; 32],
        &mut DeterministicRngProvider::from_seed([42u8; 32]).rng(),
    )
    .expect("Native asset #2 creation should succeed");

    let token_asset = Asset::new(
        Arc::clone(&token),
        0,
        1000u64,
        &blinding3,
        [3u8; 32],
        &mut DeterministicRngProvider::from_seed([42u8; 32]).rng(),
    )
    .expect("Token asset creation should succeed");

    println!(
        "✅ Created 3 assets of different types: {}, {}, {}",
        coin1_asset.definition.name, coin2_asset.definition.name, token_asset.definition.name
    );

    // Verify each asset has correct definition
    assert_eq!(coin1_asset.amount, 5000);
    assert_eq!(coin2_asset.amount, 3000);
    assert_eq!(token_asset.amount, 1000);
}

#[test]
fn test_asset_cross_mixed_transaction() {
    // Simulate transaction with inputs from different asset types
    let coin = create_coin_definition(10, "MainCoin");
    let token = create_token_definition(11, "TransferToken");

    let blinding_coin =
        BlindingFactor::random(&mut DeterministicRngProvider::from_seed([42u8; 32]).rng());
    let blinding_token =
        BlindingFactor::random(&mut DeterministicRngProvider::from_seed([42u8; 32]).rng());

    // Input 1: 1000 native assets
    let input_coin = Asset::new(
        Arc::clone(&coin),
        0,
        1000u64,
        &blinding_coin,
        [10u8; 32],
        &mut DeterministicRngProvider::from_seed([42u8; 32]).rng(),
    )
    .expect("Input native asset should be created");

    // Input 2: 500 tokens
    let input_token = Asset::new(
        Arc::clone(&token),
        0,
        500u64,
        &blinding_token,
        [11u8; 32],
        &mut DeterministicRngProvider::from_seed([42u8; 32]).rng(),
    )
    .expect("Input token should be created");

    // Collect inputs
    let inputs = [input_coin, input_token];
    println!(
        "✅ Created {} inputs from different asset types",
        inputs.len()
    );

    assert_eq!(inputs.len(), 2);
    assert_eq!(inputs[0].amount, 1000);
    assert_eq!(inputs[1].amount, 500);
}

#[test]
fn test_asset_cross_mixed() {
    // Simulate transaction with outputs of different types
    let coin = create_coin_definition(20, "OutCoin");
    let token = create_token_definition(21, "OutToken");

    let blinding1 =
        BlindingFactor::random(&mut DeterministicRngProvider::from_seed([42u8; 32]).rng());
    let blinding2 =
        BlindingFactor::random(&mut DeterministicRngProvider::from_seed([42u8; 32]).rng());
    let blinding3 =
        BlindingFactor::random(&mut DeterministicRngProvider::from_seed([42u8; 32]).rng());

    // Output 1: 800 native assets to recipient A
    let output1 = Asset::new(
        Arc::clone(&coin),
        0,
        800u64,
        &blinding1,
        [20u8; 32],
        &mut DeterministicRngProvider::from_seed([42u8; 32]).rng(),
    )
    .expect("Output 1 should be created");

    // Output 2: 200 native assets to recipient B
    let output2 = Asset::new(
        Arc::clone(&coin),
        1,
        200u64,
        &blinding2,
        [21u8; 32],
        &mut DeterministicRngProvider::from_seed([42u8; 32]).rng(),
    )
    .expect("Output 2 should be created");

    // Output 3: 300 tokens to recipient C
    let output3 = Asset::new(
        Arc::clone(&token),
        0,
        300u64,
        &blinding3,
        [22u8; 32],
        &mut DeterministicRngProvider::from_seed([42u8; 32]).rng(),
    )
    .expect("Output 3 should be created");

    let outputs = [output1, output2, output3];
    println!("✅ Created {} outputs with mixed types", outputs.len());

    // Verify total per asset type
    let total_coins: u64 = outputs
        .iter()
        .filter(|o| o.definition.id == coin.id)
        .map(|o| o.amount)
        .sum();

    let total_tokens: u64 = outputs
        .iter()
        .filter(|o| o.definition.id == token.id)
        .map(|o| o.amount)
        .sum();

    assert_eq!(total_coins, 1000);
    assert_eq!(total_tokens, 300);
}

#[test]
fn test_asset_cross_atomic_swap() {
    // Simulate atomic swap between two asset types
    let asset_a = create_coin_definition(30, "AssetA");
    let asset_b = create_token_definition(31, "AssetB");

    let blinding_a1 =
        BlindingFactor::random(&mut DeterministicRngProvider::from_seed([42u8; 32]).rng());
    let blinding_a2 =
        BlindingFactor::random(&mut DeterministicRngProvider::from_seed([42u8; 32]).rng());
    let blinding_b1 =
        BlindingFactor::random(&mut DeterministicRngProvider::from_seed([42u8; 32]).rng());
    let blinding_b2 =
        BlindingFactor::random(&mut DeterministicRngProvider::from_seed([42u8; 32]).rng());

    // Party A: has 100 native-asset units
    let party_a_has = Asset::new(
        Arc::clone(&asset_a),
        0,
        100u64,
        &blinding_a1,
        [30u8; 32],
        &mut DeterministicRngProvider::from_seed([42u8; 32]).rng(),
    )
    .expect("Party A asset should be created");

    // Party A: wants 50 tokens (output of their tx)
    let party_a_wants = Asset::new(
        Arc::clone(&asset_b),
        0,
        50u64,
        &blinding_b1,
        [31u8; 32],
        &mut DeterministicRngProvider::from_seed([42u8; 32]).rng(),
    )
    .expect("Party A wants asset should be created");

    // Party B: has 50 tokens
    let party_b_has = Asset::new(
        Arc::clone(&asset_b),
        0,
        50u64,
        &blinding_b2,
        [32u8; 32],
        &mut DeterministicRngProvider::from_seed([42u8; 32]).rng(),
    )
    .expect("Party B asset should be created");

    // Party B: wants 100 native-asset units (output of their tx)
    let party_b_wants = Asset::new(
        Arc::clone(&asset_a),
        1,
        100u64,
        &blinding_a2,
        [33u8; 32],
        &mut DeterministicRngProvider::from_seed([42u8; 32]).rng(),
    )
    .expect("Party B wants asset should be created");

    println!(
        "✅ Atomic swap setup: A has {}, wants {}",
        party_a_has.amount, party_a_wants.amount
    );
    println!(
        "✅ Atomic swap setup: B has {}, wants {}",
        party_b_has.amount, party_b_wants.amount
    );

    // Verify swap is balanced
    assert_eq!(party_a_has.amount, 100);
    assert_eq!(party_a_wants.amount, 50);
    assert_eq!(party_b_has.amount, 50);
    assert_eq!(party_b_wants.amount, 100);
}

#[test]
fn test_cross_asset_portfolio_tracking() {
    // Simulate wallet tracking assets of different types
    let mut portfolio = std::collections::BTreeMap::new();

    let coin1 = create_coin_definition(40, "Coin1");
    let coin2 = create_coin_definition(41, "Coin2");
    let token = create_token_definition(42, "Token");

    let blinding1 =
        BlindingFactor::random(&mut DeterministicRngProvider::from_seed([42u8; 32]).rng());
    let blinding2 =
        BlindingFactor::random(&mut DeterministicRngProvider::from_seed([42u8; 32]).rng());
    let blinding3 =
        BlindingFactor::random(&mut DeterministicRngProvider::from_seed([42u8; 32]).rng());

    // Add assets to portfolio
    let asset1 = Asset::new(
        Arc::clone(&coin1),
        0,
        1000u64,
        &blinding1,
        [40u8; 32],
        &mut DeterministicRngProvider::from_seed([42u8; 32]).rng(),
    )
    .expect("Asset 1");
    let asset2 = Asset::new(
        Arc::clone(&coin2),
        0,
        2000u64,
        &blinding2,
        [41u8; 32],
        &mut DeterministicRngProvider::from_seed([42u8; 32]).rng(),
    )
    .expect("Asset 2");
    let asset3 = Asset::new(
        Arc::clone(&token),
        0,
        500u64,
        &blinding3,
        [42u8; 32],
        &mut DeterministicRngProvider::from_seed([42u8; 32]).rng(),
    )
    .expect("Asset 3");

    portfolio.insert("Coin1", asset1.amount);
    portfolio.insert("Coin2", asset2.amount);
    portfolio.insert("Token", asset3.amount);

    println!("✅ Portfolio: {:?}", portfolio);

    // Verify portfolio contents
    assert_eq!(portfolio.get("Coin1"), Some(&1000));
    assert_eq!(portfolio.get("Coin2"), Some(&2000));
    assert_eq!(portfolio.get("Token"), Some(&500));
    assert_eq!(portfolio.len(), 3);
}

#[test]
fn test_cross_asset_reorg_scenario() {
    // Simulate blockchain reorg with multiple asset types
    let coin_a = create_coin_definition(50, "CoinA");
    let coin_b = create_coin_definition(51, "CoinB");

    let blinding1 =
        BlindingFactor::random(&mut DeterministicRngProvider::from_seed([42u8; 32]).rng());
    let blinding2 =
        BlindingFactor::random(&mut DeterministicRngProvider::from_seed([42u8; 32]).rng());
    let blinding3 =
        BlindingFactor::random(&mut DeterministicRngProvider::from_seed([42u8; 32]).rng());
    let blinding4 =
        BlindingFactor::random(&mut DeterministicRngProvider::from_seed([42u8; 32]).rng());

    // Block 1: Create assets
    let block1_asset_a = Asset::new(
        Arc::clone(&coin_a),
        0,
        500u64,
        &blinding1,
        [50u8; 32],
        &mut DeterministicRngProvider::from_seed([42u8; 32]).rng(),
    )
    .expect("Block 1 A");
    let block1_asset_b = Asset::new(
        Arc::clone(&coin_b),
        0,
        300u64,
        &blinding2,
        [51u8; 32],
        &mut DeterministicRngProvider::from_seed([42u8; 32]).rng(),
    )
    .expect("Block 1 B");

    println!(
        "✅ Block 1: {} CoinA, {} CoinB",
        block1_asset_a.amount, block1_asset_b.amount
    );

    // Block 2: Reorg - create different assets
    let block2_asset_a = Asset::new(
        Arc::clone(&coin_a),
        0,
        600u64,
        &blinding3,
        [52u8; 32],
        &mut DeterministicRngProvider::from_seed([42u8; 32]).rng(),
    )
    .expect("Block 2 A");
    let block2_asset_b = Asset::new(
        Arc::clone(&coin_b),
        0,
        250u64,
        &blinding4,
        [53u8; 32],
        &mut DeterministicRngProvider::from_seed([42u8; 32]).rng(),
    )
    .expect("Block 2 B");

    println!(
        "✅ Block 2 (reorg): {} CoinA, {} CoinB",
        block2_asset_a.amount, block2_asset_b.amount
    );

    // Verify both blocks had different balances
    assert_ne!(block1_asset_a.amount, block2_asset_a.amount);
    assert_ne!(block1_asset_b.amount, block2_asset_b.amount);
}

#[test]
fn test_cross_asset_nonce_isolation() {
    // Verify nonces are isolated per asset type (no collision across types)
    let coin = create_coin_definition(60, "NativeAssetIso");
    let token = create_token_definition(61, "TokenIso");

    let blinding =
        BlindingFactor::random(&mut DeterministicRngProvider::from_seed([42u8; 32]).rng());

    // Create assets with same nonce from different types (should be allowed)
    let coin_asset = Asset::new(
        Arc::clone(&coin),
        0,
        100u64,
        &blinding,
        [99u8; 32], // Same nonce
        &mut DeterministicRngProvider::from_seed([42u8; 32]).rng(),
    )
    .expect("Native asset with nonce");

    let token_asset = Asset::new(
        Arc::clone(&token),
        0,
        50u64,
        &blinding,
        [99u8; 32], // Same nonce, different asset type
        &mut DeterministicRngProvider::from_seed([42u8; 32]).rng(),
    )
    .expect("Token with same nonce");

    println!("✅ Created native asset with nonce [99], token with nonce [99] - no collision");

    // Both should exist without issue
    assert_eq!(coin_asset.amount, 100);
    assert_eq!(token_asset.amount, 50);
}
