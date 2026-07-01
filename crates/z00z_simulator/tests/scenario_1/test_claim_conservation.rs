use z00z_core::{genesis::asset_std::asset_from_dev_class, Asset, AssetClass};
use z00z_wallets::claim::{verify_claim_conservation, ConservationError};

fn mk_assets() -> Vec<Asset> {
    vec![
        asset_from_dev_class(AssetClass::Coin, 1, 10).expect("asset 1"),
        asset_from_dev_class(AssetClass::Coin, 2, 20).expect("asset 2"),
    ]
}

#[test]
fn test_stage3_conservation_ok() {
    let input_assets = mk_assets();
    let imported_assets = input_assets.clone();
    let result = verify_claim_conservation(&input_assets, &imported_assets);
    assert!(result.is_ok());
}

#[test]
fn test_stage3_conservation_fail_amount() {
    let input_assets = mk_assets();
    let mut imported_assets = input_assets.clone();
    imported_assets[0].amount = imported_assets[0]
        .amount
        .checked_add(1)
        .expect("amount bump");

    let result = verify_claim_conservation(&input_assets, &imported_assets);
    assert!(matches!(
        result,
        Err(ConservationError::AmountMismatch {
            input: 30,
            imported: 31
        })
    ));
}

#[test]
fn test_stage3_conservation_fail_commitment() {
    let input_assets = mk_assets();
    let mut imported_assets = input_assets.clone();
    let replacement = asset_from_dev_class(
        AssetClass::Coin,
        imported_assets[0].serial_id.saturating_add(1),
        imported_assets[0].amount,
    )
    .expect("replacement asset");
    imported_assets[0].commitment = replacement.commitment;

    let result = verify_claim_conservation(&input_assets, &imported_assets);
    assert!(matches!(result, Err(ConservationError::CommitmentMismatch)));
}
