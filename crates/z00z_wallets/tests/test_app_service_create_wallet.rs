#![cfg(not(target_arch = "wasm32"))]
#![cfg(not(target_arch = "wasm32"))]

use std::path::{Path, PathBuf};
use std::sync::Arc;

use redb::{ReadableDatabase, TableDefinition};
use z00z_crypto::expert::encoding::SafePassword;
use z00z_utils::compression::zstd_decompress_bounded;
use z00z_utils::io;
use z00z_utils::rng::SystemRngProvider;
use z00z_utils::time::{SystemTimeProvider, TimeProvider};

use z00z_wallets::db::WalletIdentity;
use z00z_wallets::services::{AppService, WalletService};

const META_TABLE: TableDefinition<&str, &[u8]> = TableDefinition::new("meta");
const SECRETS_TABLE: TableDefinition<&str, &[u8]> = TableDefinition::new("secrets");

const META_WALLET_ID: &str = "wallet.id";
const META_SCHEMA_VERSION: &str = "wallet.schema_version";
const META_WALLET_KDF: &str = "wallet.kdf";
const META_WALLET_INITIALIZED: &str = "wallet.initialized";
const META_WALLET_NETWORK: &str = "wallet.network";
const META_WALLET_CHAIN: &str = "wallet.chain";

const SECRETS_MASTER_KEY: &str = "master_key";
const SECRETS_SEED_MAIN: &str = "seed_main";

const TEST_SEED_PHRASE_24: &str = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon art";

fn default_identity() -> WalletIdentity {
    WalletIdentity {
        network: "p2p".to_string(),
        chain: "devnet".to_string(),
    }
}

fn find_single_wlt_file(dir: &Path) -> PathBuf {
    let entries = std::fs::read_dir(dir).expect("read_dir failed");

    let mut wlt_paths = Vec::new();
    for entry in entries {
        let entry = entry.expect("read_dir entry failed");
        let path = entry.path();
        if path.extension().and_then(|s| s.to_str()) == Some("wlt") {
            wlt_paths.push(path);
        }
    }

    assert_eq!(
        wlt_paths.len(),
        1,
        "expected exactly 1 .wlt file in output dir"
    );

    wlt_paths.remove(0)
}

fn with_decompressed_redb<F: FnOnce(&redb::Database)>(wlt_path: &Path, f: F) {
    const MAX_DECOMPRESSED_WLT_BYTES: usize = 128 * 1024 * 1024;

    let zstd = io::read_file(wlt_path).expect("read .wlt failed");
    let db_bytes =
        zstd_decompress_bounded(&zstd, MAX_DECOMPRESSED_WLT_BYTES).expect("zstd decompress failed");

    let work_path = tempfile::Builder::new()
        .prefix("z00z_wallet_redb_")
        .suffix(".wlt.work")
        .tempfile_in("/dev/shm")
        .expect("tempfile_in /dev/shm failed")
        .into_temp_path();
    let work_path_buf = work_path.to_path_buf();

    io::atomic_write_file_private(&work_path_buf, &db_bytes)
        .expect("write decompressed redb bytes failed");

    let db = redb::Database::open(&work_path_buf).expect("redb open failed");
    f(&db);
}

#[tokio::test]
async fn test_app_create_persists_schema() {
    let dir = tempfile::tempdir().expect("tempdir failed");
    let output_dir = dir.path().join("wallets");
    io::create_dir_all(&output_dir).expect("create output dir failed");

    let time_provider: Arc<dyn TimeProvider> = Arc::new(SystemTimeProvider);
    let rng_provider = SystemRngProvider;
    let wallets = Arc::new(WalletService::create_service_custom_output_directory(
        output_dir.clone(),
        Arc::clone(&time_provider),
        rng_provider,
    ));

    let app = AppService::with_wallet_service(wallets);

    let response = app
        .create_wallet(
            "alice".to_string(),
            "M7$kqV9!tP4xZ2nL".to_string(),
            Some(TEST_SEED_PHRASE_24.to_string()),
        )
        .await
        .expect("create_wallet failed");

    let wlt_path = find_single_wlt_file(&output_dir);
    assert!(wlt_path.exists(), "wallet .wlt file missing");

    // Ensure open/unlock works with the same password and identity.
    let identity = default_identity();
    let safe_password = SafePassword::from("M7$kqV9!tP4xZ2nL");
    let _session = z00z_wallets::db::open_wallet_store(
        &wlt_path,
        &response.wallet_id,
        &safe_password,
        &identity,
    )
    .expect("open_wallet_store failed");

    // Verify required schema keys exist in the persisted RedB tables.
    with_decompressed_redb(&wlt_path, |db| {
        let read_txn = db.begin_read().expect("begin_read failed");
        let meta = read_txn
            .open_table(META_TABLE)
            .expect("open meta table failed");
        let secrets = read_txn
            .open_table(SECRETS_TABLE)
            .expect("open secrets table failed");

        assert!(meta.get(META_WALLET_ID).expect("meta get failed").is_some());
        assert!(meta
            .get(META_SCHEMA_VERSION)
            .expect("meta get failed")
            .is_some());
        assert!(meta
            .get(META_WALLET_KDF)
            .expect("meta get failed")
            .is_some());
        assert!(meta
            .get(META_WALLET_INITIALIZED)
            .expect("meta get failed")
            .is_some());
        assert!(meta
            .get(META_WALLET_NETWORK)
            .expect("meta get failed")
            .is_some());
        assert!(meta
            .get(META_WALLET_CHAIN)
            .expect("meta get failed")
            .is_some());

        assert!(secrets
            .get(SECRETS_MASTER_KEY)
            .expect("secrets get failed")
            .is_some());
        assert!(secrets
            .get(SECRETS_SEED_MAIN)
            .expect("secrets get failed")
            .is_some());
    });
}
