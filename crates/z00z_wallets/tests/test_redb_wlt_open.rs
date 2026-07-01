#![cfg(not(target_arch = "wasm32"))]

use std::path::Path;

use redb::{ReadableDatabase, TableDefinition};
use z00z_crypto::expert::encoding::SafePassword;
use z00z_utils::codec::{BincodeCodec, Codec};
use z00z_utils::compression::{zstd_compress, zstd_decompress_bounded};
use z00z_utils::io;
use z00z_utils::rng::SystemRngProvider;
use z00z_wallets::db::wallet_store_crypto::{
    KdfParams, MasterKeyRecord, REDB_WALLET_SCHEMA_VERSION,
};
use z00z_wallets::db::{create_wallet_store, open_wallet_store, WalletIdentity};
use z00z_wallets::domains::hashing::compute_wallet_file_id;
use z00z_wallets::rpc::types::common::PersistWalletId;
use z00z_wallets::WalletError;

const META_TABLE: TableDefinition<&str, &[u8]> = TableDefinition::new("meta");
const SECRETS_TABLE: TableDefinition<&str, &[u8]> = TableDefinition::new("secrets");
const OBJECTS_TABLE: TableDefinition<&[u8], &[u8]> = TableDefinition::new("objects");

const META_WALLET_ID: &str = "wallet.id";
const META_SCHEMA_VERSION: &str = "wallet.schema_version";
const META_WALLET_KDF: &str = "wallet.kdf";
const META_AAD_SECRET_VERSION: &str = "wallet.aad_secret_version";
const META_HKDF_INFO_VERSION: &str = "wallet.hkdf_info_version";
const SECRETS_MASTER_KEY: &str = "master_key";
const SECRETS_SEED_MAIN: &str = "seed_main";

fn default_identity() -> WalletIdentity {
    WalletIdentity {
        network: "p2p".to_string(),
        chain: "devnet".to_string(),
    }
}

fn create_test_wallet(path: &Path) {
    let rng = SystemRngProvider;
    let wallet_id = PersistWalletId("wallet_test".to_string());
    let password = SafePassword::from("pw1");
    let seed_phrase = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon art";

    let identity = default_identity();
    create_wallet_store(path, &wallet_id, &password, seed_phrase, &identity, rng).unwrap();
}

#[test]
fn wlt_open_without_jsonl() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("wallet_test.wlt");

    create_test_wallet(&path);

    let wallet_id = PersistWalletId("wallet_test".to_string());
    let hash = compute_wallet_file_id(&wallet_id.0);
    let wallet_stem = hex::encode(&hash[..8]);
    let jsonl_path = dir
        .path()
        .join(format!("wallet_{wallet_stem}_tx_history.jsonl"));
    assert!(!jsonl_path.exists());

    let password = SafePassword::from("pw1");
    let identity = default_identity();
    open_wallet_store(&path, &wallet_id, &password, &identity)
        .expect(".wlt open without canonical JSONL history");

    assert!(!jsonl_path.exists());
}

#[test]
fn test_open_fails_identity_mismatch() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("wallet_test.wlt");

    create_test_wallet(&path);

    let wallet_id = PersistWalletId("wallet_test".to_string());
    let password = SafePassword::from("pw1");

    // Network mismatch.
    let wrong_network = WalletIdentity {
        network: "tor".to_string(),
        chain: "devnet".to_string(),
    };
    let err = open_wallet_store(&path, &wallet_id, &password, &wrong_network).unwrap_err();
    assert!(matches!(err, WalletError::WalletNetworkMismatch { .. }));

    // Chain mismatch.
    let wrong_chain = WalletIdentity {
        network: "p2p".to_string(),
        chain: "mainnet".to_string(),
    };
    let err = open_wallet_store(&path, &wallet_id, &password, &wrong_chain).unwrap_err();
    assert!(matches!(err, WalletError::WalletChainMismatch { .. }));
}

fn with_decompressed_redb<F: FnOnce(&redb::Database)>(wlt_path: &Path, f: F) {
    const MAX_DECOMPRESSED_WLT_BYTES: usize = 128 * 1024 * 1024;

    let zstd = io::read_file(wlt_path).unwrap();
    let db_bytes = zstd_decompress_bounded(&zstd, MAX_DECOMPRESSED_WLT_BYTES).unwrap();

    let work_path = tempfile::Builder::new()
        .prefix("z00z_wallet_redb_")
        .suffix(".wlt.work")
        .tempfile_in("/dev/shm")
        .unwrap()
        .into_temp_path();
    let work_path_buf = work_path.to_path_buf();

    io::atomic_write_file_private(&work_path_buf, &db_bytes).unwrap();
    let db = redb::Database::open(&work_path_buf).unwrap();

    f(&db);

    // Persist any committed changes back into the zstd `.wlt`.
    let updated_bytes = io::read_file(&work_path_buf).unwrap();
    let updated_zstd = zstd_compress(&updated_bytes).unwrap();
    io::atomic_write_file_private(wlt_path, &updated_zstd).unwrap();
}

#[test]
fn test_open_rejects_kdf() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("wallet_test.wlt");

    create_test_wallet(&path);

    let wallet_id = PersistWalletId("wallet_test".to_string());
    let password = SafePassword::from("pw1");
    let identity = default_identity();

    // Rewrite the wallet metadata to advertise an unsupported KDF contract.
    // Open must now fail closed instead of trying any migration lane.
    with_decompressed_redb(&path, |db| {
        let codec = BincodeCodec;

        let read_txn = db.begin_read().unwrap();
        let meta = read_txn.open_table(META_TABLE).unwrap();
        let secrets = read_txn.open_table(SECRETS_TABLE).unwrap();

        let kdf_raw = meta.get(META_WALLET_KDF).unwrap().unwrap().value().to_vec();
        let kdf: KdfParams = codec.deserialize(&kdf_raw).unwrap();

        let record_raw = secrets
            .get(SECRETS_MASTER_KEY)
            .unwrap()
            .unwrap()
            .value()
            .to_vec();
        let record: MasterKeyRecord = codec.deserialize(&record_raw).unwrap();

        drop(secrets);
        drop(meta);
        drop(read_txn);

        let mut kdf_v1 = kdf.clone();
        kdf_v1.version = 1;
        let mut record_v1 = record.clone();
        record_v1.kdf_params.as_mut().unwrap().version = 1;

        let kdf_v1_blob = codec.serialize(&kdf_v1).unwrap();
        let record_v1_blob = codec.serialize(&record_v1).unwrap();

        let write_txn = db.begin_write().unwrap();
        {
            let mut meta = write_txn.open_table(META_TABLE).unwrap();
            meta.insert(META_WALLET_KDF, kdf_v1_blob.as_slice())
                .unwrap();

            let mut secrets = write_txn.open_table(SECRETS_TABLE).unwrap();
            secrets
                .insert(SECRETS_MASTER_KEY, record_v1_blob.as_slice())
                .unwrap();
        }
        write_txn.commit().unwrap();
    });

    let err = open_wallet_store(&path, &wallet_id, &password, &identity).unwrap_err();
    assert!(matches!(err, WalletError::InvalidPassword));
}

#[test]
fn test_create_open_correct_password() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("wallet_test.wlt");

    create_test_wallet(&path);

    let wallet_id = PersistWalletId("wallet_test".to_string());
    let password = SafePassword::from("pw1");
    let identity = default_identity();

    let session = open_wallet_store(&path, &wallet_id, &password, &identity).unwrap();
    assert_eq!(session.opened().schema_version, REDB_WALLET_SCHEMA_VERSION);
}

#[test]
fn test_create_open_wrong_password() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("wallet_test.wlt");

    create_test_wallet(&path);

    let wallet_id = PersistWalletId("wallet_test".to_string());
    let wrong_password = SafePassword::from("pw2");
    let identity = default_identity();

    let err = open_wallet_store(&path, &wallet_id, &wrong_password, &identity).unwrap_err();
    assert!(matches!(err, WalletError::InvalidPassword));
}

#[test]
fn test_create_open_tampered_master() {
    use redb::ReadableTable;

    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("wallet_test.wlt");

    create_test_wallet(&path);

    {
        with_decompressed_redb(&path, |db| {
            let write_txn = db.begin_write().unwrap();
            {
                let mut secrets = write_txn.open_table(SECRETS_TABLE).unwrap();
                let mut record = {
                    let master_key_bytes = secrets.get(SECRETS_MASTER_KEY).unwrap().unwrap();
                    master_key_bytes.value().to_vec()
                };
                assert!(!record.is_empty());

                let flip_index = record.len() / 2;
                record[flip_index] ^= 0b0000_0001;

                secrets
                    .insert(SECRETS_MASTER_KEY, record.as_slice())
                    .unwrap();
            }
            write_txn.commit().unwrap();
        });
    }

    let wallet_id = PersistWalletId("wallet_test".to_string());
    let password = SafePassword::from("pw1");
    let identity = default_identity();

    let err = open_wallet_store(&path, &wallet_id, &password, &identity).unwrap_err();
    assert!(matches!(err, WalletError::InvalidPassword));
}

#[test]
fn test_wallet_writes_required_tables() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("wallet_test.wlt");

    create_test_wallet(&path);

    with_decompressed_redb(&path, |db| {
        let read_txn = db.begin_read().unwrap();
        let meta = read_txn.open_table(META_TABLE).unwrap();
        let secrets = read_txn.open_table(SECRETS_TABLE).unwrap();

        assert!(meta.get(META_WALLET_ID).unwrap().is_some());
        assert!(meta.get(META_SCHEMA_VERSION).unwrap().is_some());
        assert!(meta.get(META_WALLET_KDF).unwrap().is_some());
        assert!(secrets.get(SECRETS_MASTER_KEY).unwrap().is_some());
        assert!(secrets.get(SECRETS_SEED_MAIN).unwrap().is_some());
    });
}

#[test]
fn test_wrong_password_matches_corruption() {
    use redb::ReadableTable;

    let dir = tempfile::tempdir().unwrap();
    let good_path = dir.path().join("wallet_good.wlt");
    let bad_path = dir.path().join("wallet_bad.wlt");

    create_test_wallet(&good_path);
    let good_bytes = io::read_file(&good_path).unwrap();
    io::atomic_write_file_private(&bad_path, &good_bytes).unwrap();

    let wallet_id = PersistWalletId("wallet_test".to_string());
    let identity = default_identity();

    let wrong_password = SafePassword::from("pw2");
    let err_wrong =
        open_wallet_store(&good_path, &wallet_id, &wrong_password, &identity).unwrap_err();

    {
        with_decompressed_redb(&bad_path, |db| {
            let write_txn = db.begin_write().unwrap();
            {
                let mut secrets = write_txn.open_table(SECRETS_TABLE).unwrap();
                let mut record = {
                    let master_key_bytes = secrets.get(SECRETS_MASTER_KEY).unwrap().unwrap();
                    master_key_bytes.value().to_vec()
                };
                assert!(!record.is_empty());

                let flip_index = record.len() / 2;
                record[flip_index] ^= 0b0000_0001;

                secrets
                    .insert(SECRETS_MASTER_KEY, record.as_slice())
                    .unwrap();
            }
            write_txn.commit().unwrap();
        });
    }

    let correct_password = SafePassword::from("pw1");
    let err_corrupt =
        open_wallet_store(&bad_path, &wallet_id, &correct_password, &identity).unwrap_err();

    assert!(matches!(err_wrong, WalletError::InvalidPassword));
    assert!(matches!(err_corrupt, WalletError::InvalidPassword));
    assert_eq!(err_wrong.to_string(), err_corrupt.to_string());
}

#[test]
fn test_open_fails_integrity_mismatch() {
    use redb::ReadableTable;

    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("wallet_test.wlt");

    create_test_wallet(&path);

    {
        with_decompressed_redb(&path, |db| {
            let write_txn = db.begin_write().unwrap();
            {
                let mut objects = write_txn.open_table(OBJECTS_TABLE).unwrap();
                let (key, value) = {
                    let mut iter = objects.iter().unwrap();
                    let (k, v) = iter.next().unwrap().unwrap();
                    (k.value().to_vec(), v.value().to_vec())
                };

                let mut record = value;
                assert!(!record.is_empty());
                let flip_index = record.len() / 2;
                record[flip_index] ^= 0b0000_0001;

                objects.insert(key.as_slice(), record.as_slice()).unwrap();
            }
            write_txn.commit().unwrap();
        });
    }

    let wallet_id = PersistWalletId("wallet_test".to_string());
    let password = SafePassword::from("pw1");
    let identity = default_identity();

    let err = open_wallet_store(&path, &wallet_id, &password, &identity).unwrap_err();
    assert!(matches!(err, WalletError::InvalidPassword));
}

#[test]
fn test_open_rejects_secret_version() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("wallet_test.wlt");

    create_test_wallet(&path);

    with_decompressed_redb(&path, |db| {
        let read_txn = db.begin_read().unwrap();
        let meta = read_txn.open_table(META_TABLE).unwrap();
        assert!(meta.get(META_AAD_SECRET_VERSION).unwrap().is_some());
        drop(meta);
        drop(read_txn);

        let write_txn = db.begin_write().unwrap();
        {
            let mut meta = write_txn.open_table(META_TABLE).unwrap();
            meta.remove(META_AAD_SECRET_VERSION).unwrap();
        }
        write_txn.commit().unwrap();
    });

    let wallet_id = PersistWalletId("wallet_test".to_string());
    let password = SafePassword::from("pw1");
    let identity = default_identity();

    let err = open_wallet_store(&path, &wallet_id, &password, &identity).unwrap_err();
    assert!(
        matches!(err, WalletError::InvalidConfig(ref msg) if msg == "missing secret AAD format version")
    );
}

#[test]
fn test_open_rejects_unsupported_secret() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("wallet_test.wlt");

    create_test_wallet(&path);

    with_decompressed_redb(&path, |db| {
        let codec = BincodeCodec;
        let write_txn = db.begin_write().unwrap();
        {
            let mut meta = write_txn.open_table(META_TABLE).unwrap();
            meta.insert(
                META_AAD_SECRET_VERSION,
                codec.serialize(&1u32).unwrap().as_slice(),
            )
            .unwrap();
        }
        write_txn.commit().unwrap();
    });

    let wallet_id = PersistWalletId("wallet_test".to_string());
    let password = SafePassword::from("pw1");
    let identity = default_identity();

    let err = open_wallet_store(&path, &wallet_id, &password, &identity).unwrap_err();
    assert!(
        matches!(err, WalletError::InvalidConfig(ref msg) if msg == "unsupported secret AAD format version")
    );
}

#[test]
fn test_open_rejects_info_version() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("wallet_test.wlt");

    create_test_wallet(&path);

    with_decompressed_redb(&path, |db| {
        let read_txn = db.begin_read().unwrap();
        let meta = read_txn.open_table(META_TABLE).unwrap();
        assert!(meta.get(META_HKDF_INFO_VERSION).unwrap().is_some());
        drop(meta);
        drop(read_txn);

        let write_txn = db.begin_write().unwrap();
        {
            let mut meta = write_txn.open_table(META_TABLE).unwrap();
            meta.remove(META_HKDF_INFO_VERSION).unwrap();
        }
        write_txn.commit().unwrap();
    });

    let wallet_id = PersistWalletId("wallet_test".to_string());
    let password = SafePassword::from("pw1");
    let identity = default_identity();

    let err = open_wallet_store(&path, &wallet_id, &password, &identity).unwrap_err();
    assert!(
        matches!(err, WalletError::InvalidConfig(ref msg) if msg == "missing hkdf info version")
    );
}

#[test]
fn test_open_rejects_unsupported_info() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("wallet_test.wlt");

    create_test_wallet(&path);

    with_decompressed_redb(&path, |db| {
        let codec = BincodeCodec;
        let write_txn = db.begin_write().unwrap();
        {
            let mut meta = write_txn.open_table(META_TABLE).unwrap();
            meta.insert(
                META_HKDF_INFO_VERSION,
                codec.serialize(&1u32).unwrap().as_slice(),
            )
            .unwrap();
        }
        write_txn.commit().unwrap();
    });

    let wallet_id = PersistWalletId("wallet_test".to_string());
    let password = SafePassword::from("pw1");
    let identity = default_identity();

    let err = open_wallet_store(&path, &wallet_id, &password, &identity).unwrap_err();
    assert!(
        matches!(err, WalletError::InvalidConfig(ref msg) if msg == "unsupported hkdf info version")
    );
}
