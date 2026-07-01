#![cfg(not(target_arch = "wasm32"))]

//! Phase 2 production-hardening tests.
//!
//! These tests cover the remaining Phase 2 checklist items:
//! - Encrypted persistence save/load (default API)
//! - Corruption detection (tampering causes decrypt/load failure)
//! - File permission hardening on Unix (0600)
//!
//! Plaintext persistence is not supported.

use std::path::{Path, PathBuf};
use std::sync::Arc;

use z00z_crypto::expert::encoding::SafePassword;
use z00z_wallets::rpc::types::common::PersistWalletId;
use z00z_wallets::services::{AppService, WalletService};
use z00z_wallets::WalletError;

const PASSWORD: &str = "Aa1!bB2@cC3#dD4$eE5%";
const TEST_SEED_PHRASE_24: &str = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon art";
const WALLETS_LIB_SRC: &str = include_str!("../src/lib.rs");
const WALLETS_DB_MOD_SRC: &str = include_str!("../src/db/mod.rs");
const WALLETS_WALLET_MOD_SRC: &str = include_str!("../src/wallet/mod.rs");
const WALLETS_REDB_STORE_MOD_SRC: &str = include_str!("../src/redb_store/mod.rs");
const SIMULATOR_LIB_SRC: &str = include_str!("../../z00z_simulator/src/lib.rs");
const HJMT_CACHE_SRC: &str = include_str!("../../z00z_storage/src/settlement/hjmt_cache.rs");
const HJMT_SCHED_SRC: &str = include_str!("../../z00z_storage/src/settlement/hjmt_scheduler.rs");
const RELEASE_GUARD_SCRIPT: &str =
    include_str!("../../../scripts/audit/audit_release_feature_guards.sh");
const RELEASE_GUARD_WORKFLOW: &str =
    include_str!("../../../.github/workflows/release-safety-guards.yml");

fn wallet_enc_path(output_dir: &Path, wallet_id: &PersistWalletId) -> PathBuf {
    use z00z_wallets::domains::hashing::compute_wallet_file_id;

    let hash = compute_wallet_file_id(&wallet_id.0);
    let wallet_id_hex = hex::encode(&hash[..8]);
    output_dir.join(format!("wallet_{}.wlt", wallet_id_hex))
}

#[tokio::test]
async fn test_encrypted_save_load_roundtrip() {
    let temp_dir = tempfile::tempdir().unwrap();
    let output_dir = temp_dir.path().to_path_buf();

    let service = Arc::new(WalletService::with_output_dir(output_dir.clone()));
    let app = AppService::with_wallet_service(Arc::clone(&service));

    let save_password = SafePassword::from(PASSWORD);

    let wallet_id = app
        .create_wallet(
            "Encrypted Roundtrip Wallet".to_string(),
            PASSWORD.to_string(),
            Some(TEST_SEED_PHRASE_24.to_string()),
        )
        .await
        .unwrap()
        .wallet_id;

    service
        .save_wallet(wallet_id.clone(), save_password, None)
        .await
        .unwrap();
    let enc_path = wallet_enc_path(output_dir.as_path(), &wallet_id);
    assert!(enc_path.exists());

    service.unregister_wallet(&wallet_id).await.unwrap();
    service.load_wallet(&wallet_id, PASSWORD).await.unwrap();
}

#[tokio::test]
async fn test_encrypted_load_fails_wrong() {
    let temp_dir = tempfile::tempdir().unwrap();
    let output_dir = temp_dir.path().to_path_buf();

    let service = Arc::new(WalletService::with_output_dir(output_dir.clone()));
    let app = AppService::with_wallet_service(Arc::clone(&service));

    let save_password = SafePassword::from(PASSWORD);

    let wallet_id = app
        .create_wallet(
            "Wrong Password Wallet".to_string(),
            PASSWORD.to_string(),
            Some(TEST_SEED_PHRASE_24.to_string()),
        )
        .await
        .unwrap()
        .wallet_id;

    service
        .save_wallet(wallet_id.clone(), save_password, None)
        .await
        .unwrap();

    service.unregister_wallet(&wallet_id).await.unwrap();
    let err = service
        .load_wallet(&wallet_id, "not-the-password")
        .await
        .unwrap_err();
    assert!(matches!(err, WalletError::InvalidPassword));
}

#[tokio::test]
async fn test_encrypted_load_fails_tampering() {
    use redb::{ReadableTable, TableDefinition};
    use z00z_utils::compression::{zstd_compress, zstd_decompress_bounded};
    use z00z_utils::io;

    let temp_dir = tempfile::tempdir().unwrap();
    let output_dir = temp_dir.path().to_path_buf();

    let service = Arc::new(WalletService::with_output_dir(output_dir.clone()));
    let app = AppService::with_wallet_service(Arc::clone(&service));

    let save_password = SafePassword::from(PASSWORD);

    let wallet_id = app
        .create_wallet(
            "Tamper Detection Wallet".to_string(),
            PASSWORD.to_string(),
            Some(TEST_SEED_PHRASE_24.to_string()),
        )
        .await
        .unwrap()
        .wallet_id;

    service
        .save_wallet(wallet_id.clone(), save_password, None)
        .await
        .unwrap();

    let enc_path = wallet_enc_path(&output_dir, &wallet_id);

    // Corrupt the live encrypted wallet object directly inside the RedB tables.
    // Flipping an arbitrary byte in the `.wlt` file is not deterministic because it may land in
    // unused/free pages.
    const META_TABLE: TableDefinition<&str, &[u8]> = TableDefinition::new("meta");
    const OBJECTS_TABLE: TableDefinition<&[u8], &[u8]> = TableDefinition::new("objects");
    const META_WALLET_PROFILE_OBJECT_ID: &str = "wallet.profile_object_id";
    {
        // `.wlt` is zstd-by-content on disk; RedB requires a file path.
        // Decompress into tmpfs-backed work file, tamper deterministically, then recompress.
        const MAX_DECOMPRESSED_WLT_BYTES: usize = 128 * 1024 * 1024;

        let zstd = io::read_file(&enc_path).unwrap();
        let db_bytes = zstd_decompress_bounded(&zstd, MAX_DECOMPRESSED_WLT_BYTES).unwrap();

        let work_path = tempfile::Builder::new()
            .prefix("z00z_wallet_tamper_")
            .suffix(".wlt.work")
            .tempfile_in("/dev/shm")
            .unwrap()
            .into_temp_path();

        let work_path_buf = work_path.to_path_buf();

        io::atomic_write_file_private(&work_path_buf, &db_bytes).unwrap();

        let db = redb::Database::open(&work_path_buf).unwrap();
        let write_txn = db.begin_write().unwrap();
        {
            let meta = write_txn.open_table(META_TABLE).unwrap();
            let object_id_bytes = meta
                .get(META_WALLET_PROFILE_OBJECT_ID)
                .unwrap()
                .expect("phase-047 live wallet state must have a profile object");
            let object_key = object_id_bytes.value().to_vec();

            let mut objects = write_txn.open_table(OBJECTS_TABLE).unwrap();
            let mut record_bytes = {
                let record_guard = objects.get(object_key.as_slice()).unwrap().unwrap();
                record_guard.value().to_vec()
            };
            assert!(!record_bytes.is_empty());

            // Flip one byte inside the serialized record to trigger AEAD authentication failure.
            let flip_index = record_bytes.len() / 2;
            record_bytes[flip_index] ^= 0b0000_0001;

            objects
                .insert(object_key.as_slice(), record_bytes.as_slice())
                .unwrap();
        }
        write_txn.commit().unwrap();

        let tampered_bytes = io::read_file(&work_path_buf).unwrap();
        let tampered_zstd = zstd_compress(&tampered_bytes).unwrap();
        io::atomic_write_file_private(&enc_path, &tampered_zstd).unwrap();
    }

    // Clear in-memory state so load hits disk.
    service.unregister_wallet(&wallet_id).await.unwrap();

    let err = service.load_wallet(&wallet_id, PASSWORD).await.unwrap_err();
    assert!(matches!(err, WalletError::InvalidPassword));
}

#[cfg(unix)]
#[tokio::test]
async fn test_wallet_file_permissions_are() {
    use std::os::unix::fs::PermissionsExt;

    let temp_dir = tempfile::tempdir().unwrap();
    let output_dir = temp_dir.path().to_path_buf();

    let service = Arc::new(WalletService::with_output_dir(output_dir.clone()));
    let app = AppService::with_wallet_service(Arc::clone(&service));

    let save_password = SafePassword::from(PASSWORD);

    let wallet_id = app
        .create_wallet(
            "Permissions Wallet".to_string(),
            PASSWORD.to_string(),
            Some(TEST_SEED_PHRASE_24.to_string()),
        )
        .await
        .unwrap()
        .wallet_id;

    service
        .save_wallet(wallet_id.clone(), save_password, None)
        .await
        .unwrap();

    let enc_path = wallet_enc_path(&output_dir, &wallet_id);
    let mode = std::fs::metadata(&enc_path).unwrap().permissions().mode() & 0o777;
    assert_eq!(mode, 0o600);
}

#[test]
fn test_release_feature_compile_guards_are_fail_closed() {
    assert!(WALLETS_LIB_SRC.contains(
        "`test-params-fast` MUST NOT be compiled into release-capable z00z_wallets builds"
    ));
    assert!(WALLETS_LIB_SRC.contains(
        "`wallet_debug_tools` MUST NOT be compiled into release-capable z00z_wallets builds"
    ));
    assert!(SIMULATOR_LIB_SRC.contains(
        "`test-params-fast` MUST NOT be compiled into release-capable z00z_simulator builds"
    ));
    assert!(SIMULATOR_LIB_SRC.contains(
        "`wallet_debug_tools` MUST NOT be compiled into release-capable z00z_simulator builds"
    ));
}

#[test]
fn test_debug_export_surface_is_internal_only() {
    assert!(
        !WALLETS_DB_MOD_SRC.contains("pub use self::redb_store::debug_export_wallet"),
        "db facade must not re-export debug_export_wallet",
    );
    assert!(
        !WALLETS_WALLET_MOD_SRC.contains("pub use crate::db::debug_export_wallet"),
        "wallet facade must not re-export debug_export_wallet",
    );
    assert!(
        WALLETS_REDB_STORE_MOD_SRC.contains("pub(crate) use self::debug::{"),
        "redb store debug exports must stay crate-private",
    );
    assert!(
        WALLETS_LIB_SRC.contains("pub mod internal_debug_tools"),
        "wallet crate must keep the explicit internal debug surface",
    );
}

#[test]
fn test_release_builds_hide_storage_test_hooks() {
    assert!(
        HJMT_CACHE_SRC.contains("#[cfg(debug_assertions)]\n    pub fn set_forest_cache_test_limit"),
        "forest cache test-limit hook must be debug-only",
    );
    assert!(
        HJMT_CACHE_SRC
            .contains("#[cfg(debug_assertions)]\n    pub fn corrupt_forest_cache_for_test"),
        "forest cache corruption hook must be debug-only",
    );
    assert!(
        HJMT_CACHE_SRC
            .contains("#[cfg(debug_assertions)]\n    pub fn corrupt_journal_key_for_test"),
        "forest cache journal-drift hook must be debug-only",
    );
    assert!(
        HJMT_SCHED_SRC.contains("#[cfg(debug_assertions)]\n    pub fn set_sched_limits_for_test"),
        "scheduler limit hook must be debug-only",
    );
    assert!(
        HJMT_SCHED_SRC.contains("#[cfg(debug_assertions)]\n    pub fn set_sched_cancel_for_test"),
        "scheduler cancel hook must be debug-only",
    );
    assert!(
        HJMT_SCHED_SRC.contains("#[cfg(debug_assertions)]\n    pub fn set_sched_test_skew_ms"),
        "scheduler skew hook must be debug-only",
    );
    assert!(
        HJMT_SCHED_SRC.contains("#[cfg(debug_assertions)]\n    pub fn reset_sched_for_test"),
        "scheduler reset hook must be debug-only",
    );
}

#[test]
fn test_release_guard_artifacts_are_checked_in() {
    assert!(RELEASE_GUARD_SCRIPT
        .contains("cargo check -p z00z_wallets --release --features test-params-fast"));
    assert!(RELEASE_GUARD_SCRIPT
        .contains("cargo check -p z00z_simulator --release --features wallet_debug_tools"));
    assert!(RELEASE_GUARD_WORKFLOW.contains("bash scripts/audit/audit_release_feature_guards.sh"));
}
