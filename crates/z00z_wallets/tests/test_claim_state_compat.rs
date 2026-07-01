use std::path::PathBuf;

use z00z_utils::io::load_json;
use z00z_wallets::claim::{ClaimLifeStep, ClaimStateFile};

fn fixture_path(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join(name)
}

#[test]
fn test_claim_state_loads_compat() {
    let prior: ClaimStateFile = load_json(fixture_path("claim_state_prior.json")).expect("prior");
    let current: ClaimStateFile =
        load_json(fixture_path("claim_state_current.json")).expect("current");

    assert_eq!(prior.step, ClaimLifeStep::ArtifactsWritten);
    assert!(prior.claimed_rows.is_empty());

    assert_eq!(current.step, ClaimLifeStep::WalletsUpdated);
    assert_eq!(current.claimed_rows.len(), 2);
}

#[test]
fn test_claim_state_merge_compat() {
    let prior: ClaimStateFile = load_json(fixture_path("claim_state_prior.json")).expect("prior");
    let current: ClaimStateFile =
        load_json(fixture_path("claim_state_current.json")).expect("current");

    let merged = prior.merge(current).expect("merge");
    assert_eq!(merged.step, ClaimLifeStep::WalletsUpdated);
    assert_eq!(merged.claimed_rows.len(), 2);
}
