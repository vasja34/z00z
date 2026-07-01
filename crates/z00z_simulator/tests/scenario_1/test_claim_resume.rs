use z00z_wallets::claim::registry as claim_registry;
use z00z_wallets::claim::{rehydrate_rows, ClaimLifeStep, ClaimStateFile, ClaimStateRow};

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
fn test_resume_no_double_count() {
    claim_registry::clear_rows();

    let rows = vec![
        ClaimStateRow {
            wallet_id: "alice_wlt".to_string(),
            asset_id_hex: "aa".repeat(32),
        },
        ClaimStateRow {
            wallet_id: "alice_wlt".to_string(),
            asset_id_hex: "aa".repeat(32),
        },
        ClaimStateRow {
            wallet_id: "bob_wlt".to_string(),
            asset_id_hex: "bb".repeat(32),
        },
    ];
    let old = mk_state(ClaimLifeStep::ArtifactsWritten, rows.clone());
    let next = mk_state(ClaimLifeStep::WalletsUpdated, rows);
    let merged = old.clone().merge(next).expect("merge");

    rehydrate_rows(&merged.claimed_rows).expect("rehydrate");

    assert!(claim_registry::has_row([0xaa; 32]));
    assert!(claim_registry::has_row([0xbb; 32]));
    let uniq: std::collections::HashSet<_> = merged
        .claimed_rows
        .iter()
        .map(|row| row.asset_id_hex.clone())
        .collect();
    assert_eq!(uniq.len(), 2);
}
