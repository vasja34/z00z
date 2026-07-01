use z00z_storage::fixture_support::guardrail::{
    assert_absent, assert_all_absent, assert_all_present, assert_each_absent, assert_present,
};

const BACKEND_MOD: &str = include_str!("../src/backend/mod.rs");
const BACKEND_REDB: &str = include_str!("../src/backend/redb/mod.rs");
const STORE_MOD: &str = include_str!("../src/settlement/store.rs");
const README_DOC: &str = include_str!("../src/settlement/README.md");

const WALLET_WITNESS: &str = include_str!("../../z00z_wallets/src/tx/state_witness.rs");
const WALLET_RESOLVED: &str = include_str!("../../z00z_wallets/src/tx/state_resolved_input.rs");
const WALLET_BACKEND: &str = include_str!("../../z00z_wallets/src/tx/spend_proof_backend.rs");
const AGG_TYPES: &str = include_str!("../../z00z_runtime/aggregators/src/types.rs");
const VERDICTS: &str = include_str!("../../z00z_runtime/validators/src/verdict.rs");
const WATCHER_EXPORT: &str = include_str!("../../z00z_runtime/watchers/src/evidence_export.rs");
const WATCHER_STATUS: &str = include_str!("../../z00z_runtime/watchers/src/status.rs");
const WATCHER_ENGINE: &str = include_str!("../../z00z_runtime/watchers/src/engine.rs");
const SIM_STAGE11: &str =
    include_str!("../../z00z_simulator/src/scenario_1/stage_11/jmt_wallet_scan.rs");

#[test]
fn test_keeps_single_journal() {
    assert_all_present(
        "backend mod",
        BACKEND_MOD,
        &["pub trait StorageBackend", "pub trait JournalBackend"],
    );
    assert_present(
        "redb adapter",
        BACKEND_REDB,
        "impl JournalBackend for StoragePlane",
    );
    assert_present(
        "store mod",
        STORE_MOD,
        "crate::backend::JournalBackend::recover_journal(&backend)?;",
    );
    assert_present(
        "settlement readme",
        README_DOC,
        "JournalBackend` is the single durability seam below settlement semantics.",
    );
    assert_present(
        "settlement readme",
        README_DOC,
        "A shared cross-aggregator WAL is not live protocol truth.",
    );
}

#[test]
fn test_keeps_semantic_seam() {
    assert_all_absent(
        "backend mod",
        BACKEND_MOD,
        &[
            "SettlementStore",
            "SettlementStateRoot",
            "RightLeaf",
            "FeeEnvelope",
            "BatchProofBlobV1",
        ],
    );
    assert_absent("redb adapter", BACKEND_REDB, "PublicationRecord");
    assert_present("store mod", STORE_MOD, "pub struct SettlementRecoveryState");
}

#[test]
fn test_keeps_downstream_clean() {
    assert_each_absent(
        &[
            ("wallet witness", WALLET_WITNESS),
            ("wallet resolved", WALLET_RESOLVED),
            ("wallet backend", WALLET_BACKEND),
            ("sim stage11", SIM_STAGE11),
            ("aggregator types", AGG_TYPES),
            ("validator verdict", VERDICTS),
            ("watcher export", WATCHER_EXPORT),
            ("watcher status", WATCHER_STATUS),
            ("watcher engine", WATCHER_ENGINE),
        ],
        &[
            "StorageBackend",
            "JournalBackend",
            "ReadTxn",
            "WriteTxn",
            "StoreBackendError",
        ],
    );
}
