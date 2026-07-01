use z00z_core::{genesis::asset_std::asset_from_dev_class, AssetClass};
use z00z_wallets::claim::{
    rehydrate_rows, verify_resume_wire, ClaimLifeStep, ClaimStateFile, ClaimStateRow,
};

fn mk_state(step: ClaimLifeStep) -> ClaimStateFile {
    ClaimStateFile {
        run_id: "uniform_all|mock:9|consume=true".to_string(),
        mode: "uniform_all".to_string(),
        rng_kind: "mock:9".to_string(),
        step,
        started_at_unix: 1,
        claimed_rows: vec![],
    }
}

#[test]
fn test_merge_state_parity() {
    let mut old = mk_state(ClaimLifeStep::ArtifactsWritten);
    old.claimed_rows.push(ClaimStateRow {
        wallet_id: "alice".to_string(),
        asset_id_hex: "11".repeat(32),
    });

    let mut next = mk_state(ClaimLifeStep::WalletsUpdated);
    next.claimed_rows.push(ClaimStateRow {
        wallet_id: "bob".to_string(),
        asset_id_hex: "22".repeat(32),
    });

    let merged = old.clone().merge(next.clone()).expect("core merge");
    assert_eq!(merged.step, ClaimLifeStep::WalletsUpdated);
    assert_eq!(merged.claimed_rows.len(), 2);
}

#[test]
fn test_rehydrate_rows_parity() {
    let rows = vec![ClaimStateRow {
        wallet_id: "alice".to_string(),
        asset_id_hex: "xyz".to_string(),
    }];

    let core_err = rehydrate_rows(&rows).expect_err("core err");
    assert!(core_err.contains("invalid asset hex"));
}

#[test]
fn test_verify_wire_parity() {
    let asset = asset_from_dev_class(AssetClass::Coin, 1, 100).expect("asset");
    let wire = z00z_core::assets::AssetWire::from_asset(&asset);

    assert!(verify_resume_wire(&wire, wire.owner_tag).is_ok());
}
