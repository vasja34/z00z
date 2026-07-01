use z00z_core::{assets::AssetWire, genesis::asset_std::asset_from_dev_class, AssetClass};
use z00z_wallets::claim::{verify_resume_wire, ClaimLifeStep, ClaimStateFile, ClaimStateRow};

fn mk_state(step: ClaimLifeStep, rows: Vec<ClaimStateRow>) -> ClaimStateFile {
    ClaimStateFile {
        run_id: "uniform_all|mock:42|consume=false".to_string(),
        mode: "uniform_all".to_string(),
        rng_kind: "mock:42".to_string(),
        step,
        started_at_unix: 1,
        claimed_rows: rows,
    }
}

#[test]
fn test_resume_idempotent() {
    let a = ClaimStateRow {
        wallet_id: "alice_wlt".to_string(),
        asset_id_hex: "11".repeat(32),
    };
    let b = ClaimStateRow {
        wallet_id: "alice_wlt".to_string(),
        asset_id_hex: "22".repeat(32),
    };
    let c = ClaimStateRow {
        wallet_id: "bob_wlt".to_string(),
        asset_id_hex: "33".repeat(32),
    };

    let single = mk_state(
        ClaimLifeStep::WalletsUpdated,
        vec![a.clone(), b.clone(), c.clone()],
    );
    let half = mk_state(ClaimLifeStep::ArtifactsWritten, vec![a.clone()]);
    let resume = mk_state(
        ClaimLifeStep::WalletsUpdated,
        vec![a.clone(), b.clone(), c.clone()],
    );

    let merged = half.merge(resume).expect("merge must pass");
    assert_eq!(merged.step, ClaimLifeStep::WalletsUpdated);
    assert_eq!(merged.claimed_rows.len(), single.claimed_rows.len());
}

#[test]
fn test_resume_no_bypass_verify() {
    let asset = asset_from_dev_class(AssetClass::Coin, 7, 42).expect("asset");
    let mut wire = AssetWire::from_asset(&asset);
    let owner = wire.owner_tag;
    wire.amount = wire.amount.saturating_add(1);

    let res = verify_resume_wire(&wire, owner);
    assert!(res.is_err(), "tampered pending wire must fail verify");
}
