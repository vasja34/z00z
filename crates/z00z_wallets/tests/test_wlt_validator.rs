#![cfg(not(target_arch = "wasm32"))]
#![cfg(not(target_arch = "wasm32"))]

use z00z_crypto::expert::encoding::SafePassword;
use z00z_utils::io;
use z00z_utils::rng::SystemRngProvider;
use z00z_wallets::db::{create_wallet_store, validate_wallet_file_codes, WalletIdentity};
use z00z_wallets::rpc::types::common::PersistWalletId;

fn default_identity() -> WalletIdentity {
    WalletIdentity {
        network: "p2p".to_string(),
        chain: "devnet".to_string(),
    }
}

#[test]
fn test_validator_rejects_non_zstd() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("not_a_wallet.wlt");

    io::atomic_write_file_private(&path, b"not a zstd frame").unwrap();

    let diags = validate_wallet_file_codes(&path).unwrap();
    assert!(diags.contains(&"CONTAINER_INVALID".to_string()));
}

#[test]
fn test_validator_rejects_bad_zstd() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("bad_frame.wlt");

    // Zstd magic bytes + junk payload.
    let bytes = vec![0x28, 0xB5, 0x2F, 0xFD, 1, 2, 3, 4, 5];
    io::atomic_write_file_private(&path, &bytes).unwrap();

    let diags = validate_wallet_file_codes(&path).unwrap();
    assert!(diags.contains(&"DECOMPRESS_FAIL".to_string()));
}

#[test]
fn test_validator_reports_missing_meta() {
    use redb::TableDefinition;
    use z00z_utils::compression::zstd_compress;

    const META_TABLE: TableDefinition<&str, &[u8]> = TableDefinition::new("meta");
    const SECRETS_TABLE: TableDefinition<&str, &[u8]> = TableDefinition::new("secrets");

    let dir = tempfile::tempdir().unwrap();
    let redb_path = dir.path().join("empty.redb");
    let wlt_path = dir.path().join("missing_keys.wlt");

    // Create a structurally valid RedB file with empty required tables.
    let db = redb::Database::create(&redb_path).unwrap();
    let write_txn = db.begin_write().unwrap();
    {
        let _ = write_txn.open_table(META_TABLE).unwrap();
        let _ = write_txn.open_table(SECRETS_TABLE).unwrap();
    }
    write_txn.commit().unwrap();

    let redb_bytes = io::read_file(&redb_path).unwrap();
    let zstd = zstd_compress(&redb_bytes).unwrap();
    io::atomic_write_file_private(&wlt_path, &zstd).unwrap();

    let diags = validate_wallet_file_codes(&wlt_path).unwrap();
    assert!(diags.contains(&"META_INVALID".to_string()));
    assert!(diags.contains(&"INTEGRITY_MISSING".to_string()));
    assert!(diags.contains(&"SECRETS_MISSING".to_string()));
}

#[test]
fn test_validator_accepts_valid_wallet() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("wallet_ok.wlt");

    let wallet_id = PersistWalletId("wallet_test".to_string());
    let password = SafePassword::from("pw1");
    let seed_phrase = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon art";

    create_wallet_store(
        &path,
        &wallet_id,
        &password,
        seed_phrase,
        &default_identity(),
        SystemRngProvider,
    )
    .unwrap();

    let diags = validate_wallet_file_codes(&path).unwrap();
    assert!(diags.is_empty());
}
