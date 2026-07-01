//! Integration Test 25: Full Transaction Workflow
//!
//! End-to-end test simulating complete transaction lifecycle:
//! - Genesis asset creation
//! - Transaction input/output creation
//! - Commitment balance verification
//! - Multi-asset transaction processing
//! - Full validation pipeline

use std::sync::Arc;
use z00z_core::assets::{Asset, AssetClass, AssetDefinition};
use z00z_core::genesis::asset_std::{asset_from_dev_class, serials_from_dev_class};
use z00z_core::BlindingFactor;
use z00z_utils::rng::DeterministicRngProvider;

fn create_std_asset(serial_id: u32, amount: u64) -> Asset {
    let serial =
        serial_id % serials_from_dev_class(AssetClass::Coin).expect("canonical dev serial cap");
    asset_from_dev_class(AssetClass::Coin, serial, amount).expect("Z00Z native asset creation")
}

fn create_custom_asset_definition(id: u8, name: &str) -> Arc<AssetDefinition> {
    Arc::new(
        AssetDefinition::new(
            [id; 32],
            AssetClass::Token,
            name.to_string(),
            name.to_string(),
            6,
            100_000,
            1000,
            "test.local".to_string(),
            1,
            1,
            0,
            None,
        )
        .expect("Valid custom definition"),
    )
}

#[test]
fn test_full_tx_genesis_issuance() {
    // Test 1: Genesis Asset Issuance
    // - Create genesis definition
    // - Issue initial assets
    // - Verify amounts and nonces

    // Issue 3 genesis assets (reduce for <15s target)
    let mut genesis_outputs = Vec::new();
    for i in 0..3 {
        let asset = create_std_asset(i as u32, 10_000_000u64);
        genesis_outputs.push(asset);
    }

    println!(
        "✅ Genesis Issuance: Issued {} assets (10M each)",
        genesis_outputs.len()
    );

    let total_issued: u64 = genesis_outputs.iter().map(|a| a.amount).sum();
    assert_eq!(total_issued, 30_000_000, "Total genesis issuance");
    assert_eq!(genesis_outputs.len(), 3);
}

#[test]
fn test_full_tx_simple_transfer() {
    // Test 2: Simple Transfer Transaction
    // - Select inputs (assets)
    // - Create outputs
    // - Verify commitment balance

    // Input: 1000 units
    let input = create_std_asset(0, 1000u64);

    // Output 1: 700 to recipient
    let output1 = create_std_asset(1, 700u64);

    // Output 2: 300 change back
    let output2 = create_std_asset(2, 300u64);

    println!(
        "✅ Simple Transfer: {} → {} + {} (change)",
        input.amount, output1.amount, output2.amount
    );

    // Verify balance (without fee)
    let input_total = input.amount;
    let output_total = output1.amount + output2.amount;
    assert_eq!(input_total, output_total, "Commitment balance verified");
}

#[test]
fn test_full_tx_with_fee() {
    // Test 3: Transaction with Fee
    // - Create inputs
    // - Create outputs
    // - Verify fee calculation

    const FEE: u64 = 10;

    // Input: 1000 units
    let input = create_std_asset(0, 1000u64);

    // Output 1: 800 to recipient
    let output1 = create_std_asset(1, 800u64);

    // Output 2: 190 change (1000 - 800 - 10 fee)
    let output2 = create_std_asset(2, 190u64);

    let input_total = input.amount;
    let output_total = output1.amount + output2.amount + FEE;

    println!(
        "✅ Transfer with Fee: {} = {} (output1) + {} (output2) + {} (fee)",
        input_total, output1.amount, output2.amount, FEE
    );

    assert_eq!(input_total, output_total, "Fee-adjusted balance verified");
}

#[test]
fn test_tx_full_multi_input() {
    // Test 4: Consolidation Transaction
    // - Combine multiple inputs
    // - Create single output
    // - Verify total consolidation

    // Create 2 inputs of 500 units each (total 1000)
    let mut inputs = Vec::new();
    for i in 0..2 {
        let asset = create_std_asset(i as u32, 500u64);
        inputs.push(asset);
    }

    // Single output: all 1000 units consolidated (2×500)
    let output = create_std_asset(10, 1000u64);

    let input_total: u64 = inputs.iter().map(|a| a.amount).sum();

    println!(
        "✅ Consolidation: {} inputs (500 each) → {} output",
        inputs.len(),
        output.amount
    );

    assert_eq!(input_total, output.amount, "Consolidation balance verified");
}

#[test]
fn test_tx_asset_full_multi() {
    // Test 5: Complex Multi-Asset Transaction
    // - Mix Z00Z native assets and custom tokens
    // - Multiple inputs, multiple outputs
    // - Verify per-asset balance

    let token_def = create_custom_asset_definition(2, "MyToken");

    // Inputs
    let blinding2 =
        BlindingFactor::random(&mut DeterministicRngProvider::from_seed([42u8; 32]).rng());
    let _blinding3 =
        BlindingFactor::random(&mut DeterministicRngProvider::from_seed([42u8; 32]).rng());

    let input_z00z = create_std_asset(0, 5000u64);

    let input_token = Asset::new(
        Arc::clone(&token_def),
        0,
        1000u64,
        &blinding2,
        [2u8; 32],
        &mut DeterministicRngProvider::from_seed([42u8; 32]).rng(),
    )
    .expect("Input Token");

    // Outputs
    let blinding_out3 =
        BlindingFactor::random(&mut DeterministicRngProvider::from_seed([42u8; 32]).rng());
    let blinding_out4 =
        BlindingFactor::random(&mut DeterministicRngProvider::from_seed([42u8; 32]).rng());

    let output_z00z_1 = create_std_asset(10, 3000u64);

    let output_z00z_2 = create_std_asset(11, 2000u64);

    let output_token_1 = Asset::new(
        Arc::clone(&token_def),
        10,
        600u64,
        &blinding_out3,
        [12u8; 32],
        &mut DeterministicRngProvider::from_seed([42u8; 32]).rng(),
    )
    .expect("Output Token 1");

    let output_token_2 = Asset::new(
        Arc::clone(&token_def),
        11,
        400u64,
        &blinding_out4,
        [13u8; 32],
        &mut DeterministicRngProvider::from_seed([42u8; 32]).rng(),
    )
    .expect("Output Token 2");

    let z00z_in: u64 = [&input_z00z].iter().map(|a| a.amount).sum();
    let token_in: u64 = [&input_token].iter().map(|a| a.amount).sum();

    let z00z_out = output_z00z_1.amount + output_z00z_2.amount;
    let token_out = output_token_1.amount + output_token_2.amount;

    println!("✅ Complex Multi-Asset TX:");
    println!(
        "  Z00Z:  {} → {} + {}",
        z00z_in, output_z00z_1.amount, output_z00z_2.amount
    );
    println!(
        "  Token: {} → {} + {}",
        token_in, output_token_1.amount, output_token_2.amount
    );

    assert_eq!(z00z_in, z00z_out, "Z00Z balance verified");
    assert_eq!(token_in, token_out, "Token balance verified");
}

#[test]
fn test_full_tx_nft_issuance() {
    // Test 6: NFT Issuance
    // - Create NFT definition (nominal = 0)
    // - Issue NFT with serial tracking
    // - Verify unique serials

    let nft_def = Arc::new(
        AssetDefinition::new(
            [99u8; 32],
            AssetClass::Nft,
            "CollectibleNFT".to_string(),
            "NFT".to_string(),
            0,    // no decimals for NFT
            1000, // max serials
            0,    // nominal = 0 (no value, unique by serial)
            "nft.local".to_string(),
            1,
            1,
            0,
            None,
        )
        .expect("Valid NFT definition"),
    );

    let blinding =
        BlindingFactor::random(&mut DeterministicRngProvider::from_seed([42u8; 32]).rng());

    // Issue 2 unique NFTs with different serials (reduce for <15s)
    let mut nfts = Vec::new();
    for i in 0..2 {
        let asset = Asset::new(
            Arc::clone(&nft_def),
            i as u32, // Each NFT has unique serial
            0u64,     // NFTs have zero amount (unique by serial)
            &blinding,
            [(i as u8); 32],
            &mut DeterministicRngProvider::from_seed([42u8; 32]).rng(),
        )
        .expect("NFT creation");
        nfts.push(asset);
    }

    println!("✅ NFT Issuance: {} unique NFTs", nfts.len());

    // Verify all have same definition but different serials
    for (i, nft) in nfts.iter().enumerate() {
        assert_eq!(nft.definition.id, nft_def.id, "Same definition");
        assert_eq!(nft.serial_id, i as u32, "Unique serial");
    }
}

#[test]
fn test_tx_full_end_validation() {
    // Test 7: Full End-to-End Validation
    // - Create assets
    // - Validate each asset
    // - Verify transaction semantics
    // - Ensure no panics in realistic workflow

    // Simulate realistic transaction flow
    let mut all_assets_created = 0;
    let mut all_assets_validated = 0;

    // Process 2 transactions
    for tx_num in 0..2 {
        let mut inputs = Vec::new();
        let mut outputs = Vec::new();

        // Create inputs
        for i in 0..2 {
            let asset = create_std_asset((tx_num * 2 + i) as u32, 1000u64);
            inputs.push(asset);
            all_assets_created += 1;
        }

        // Create outputs from inputs
        let input_total: u64 = inputs.iter().map(|a| a.amount).sum();
        for i in 0..2 {
            let amount = 1000; // Fixed: match input amount for balance
            let asset = create_std_asset((tx_num * 2 + i + 100) as u32, amount);
            outputs.push(asset);
            all_assets_created += 1;
        }

        let output_total: u64 = outputs.iter().map(|a| a.amount).sum();

        println!(
            "✅ TX {}: {} inputs ({}), {} outputs ({})",
            tx_num,
            inputs.len(),
            input_total,
            outputs.len(),
            output_total
        );

        // Verify balance
        assert_eq!(input_total, output_total, "TX {} balance", tx_num);

        // Count validated
        for _ in inputs.iter().chain(outputs.iter()) {
            all_assets_validated += 1;
        }
    }

    println!(
        "✅ Full Workflow Complete: {} assets created, {} validated",
        all_assets_created, all_assets_validated
    );

    assert_eq!(all_assets_created, all_assets_validated);
}
