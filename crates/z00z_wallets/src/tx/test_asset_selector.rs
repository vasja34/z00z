use super::*;
use std::sync::Arc;
use z00z_core::assets::{AssetClass, AssetDefinition};
use z00z_core::Commitment;
use z00z_utils::rng::SystemRngProvider;

fn make_asset(id_byte: u8, amount: u64) -> Asset {
    let def = AssetDefinition {
        id: [id_byte; 32],
        class: AssetClass::Coin,
        name: "Test Asset".to_string(),
        symbol: "TST".to_string(),
        decimals: 0,
        serials: 1,
        nominal: 1,
        domain_name: "test.local".to_string(),
        version: 1,
        crypto_version: 1,
        policy_flags: 0,
        metadata: None,
    };

    Asset {
        definition: Arc::new(def),
        serial_id: 0,
        amount,
        commitment: Commitment::default(),
        range_proof: None,
        nonce: [0u8; 32],
        lock_height: None,
        owner_pub: None,
        owner_signature: None,
        is_frozen: false,
        is_slashed: false,
        is_burned: false,
        r_pub: None,
        owner_tag: None,
        enc_pack: None,
        secret: None,
        tag16: None,
        leaf_ad_id: None,
    }
}

#[test]
fn test_new_creates_selector() {
    let selector = AssetSelectorImpl::new(SystemRngProvider);
    assert!(format!("{:?}", selector).contains("AssetSelectorImpl"));
}

#[test]
fn test_select_empty_assets_no() {
    let selector = AssetSelectorImpl::new(SystemRngProvider);
    let result = selector.select(&[], 1000, 10, SelectionStrategy::MinInputs);
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), AssetSelectorError::NoAssets));
}

#[test]
fn test_select_returns_not_implemented() {
    let selector = AssetSelectorImpl::new(SystemRngProvider);
    let assets = vec![make_asset(1, 10), make_asset(2, 100)];
    let result = selector
        .select(&assets, 50, 0, SelectionStrategy::MinInputs)
        .unwrap();
    assert_eq!(result.inputs.len(), 1);
    assert_eq!(result.total_amount, 100);
    assert_eq!(result.change_amount, 50);
}

#[test]
fn test_calculate_change_returns_zero() {
    let selector = AssetSelectorImpl::new(SystemRngProvider);
    let inputs = vec![make_asset(1, 10), make_asset(2, 100)];
    let change = selector.calculate_change(&inputs, 50, 10).unwrap();
    assert_eq!(change, 50);
}

#[test]
fn test_select_insufficient_funds() {
    let selector = AssetSelectorImpl::new(SystemRngProvider);
    let assets = vec![make_asset(1, 10), make_asset(2, 20)];
    let err = selector
        .select(&assets, 100, 0, SelectionStrategy::MinInputs)
        .unwrap_err();
    assert!(matches!(err, AssetSelectorError::InsufficientFunds { .. }));
}

#[test]
fn test_canonical_multi_are_reachable() {
    let seed = [7u8; 32];
    let direct = super::multi::derive_output_id(seed, 10, 2);
    let root = crate::tx::derive_output_id(seed, 10, 2);

    assert_eq!(direct, root);

    let check_statement_fn = crate::tx::check_statement;
    let check_batch_fn = crate::tx::check_batch;
    let _ = (check_statement_fn, check_batch_fn);
}
