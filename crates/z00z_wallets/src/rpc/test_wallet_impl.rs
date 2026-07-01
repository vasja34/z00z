use super::*;
use crate::domains::AeadEnvelopeDomain;
use crate::key::Z00ZKeyBranch;
use crate::rpc::methods::{AppRpcImpl, AppRpcServer};
use crate::security::password::PasswordValidator;
use crate::services::AppService;
use crate::wallet::persistence::WalletExportPack;
use crate::wallet::AutoLockPolicy;
use base64::Engine as _;
use std::future::Future;
use std::pin::Pin;
use std::time::Duration;
use tempfile::TempDir;
use z00z_crypto::expert::{encoding::SafePassword, traits::DomainSeparation};
use z00z_utils::codec::{BincodeCodec, Codec, JsonCodec};
use z00z_utils::time::{MockTimeProvider, SystemTimeProvider, TimeProvider};

const EXPORT_MAGIC: &[u8] = b"z00z-wexp\0";
const TEST_SEED_PHRASE_24: &str = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon art";

#[derive(Debug, Clone)]
struct MockSleeper {
    time: Arc<MockTimeProvider>,
}

impl MockSleeper {
    fn new(time: Arc<MockTimeProvider>) -> Self {
        Self { time }
    }
}

impl crate::services::wallet_service::Sleeper for MockSleeper {
    fn sleep<'a>(&'a self, duration: Duration) -> Pin<Box<dyn Future<Output = ()> + Send + 'a>> {
        Box::pin(async move {
            self.time.advance_by(duration);
        })
    }
}

fn decode_export_seed_salt(
    export: &crate::rpc::types::common::RuntimeEncryptedResponse,
    password: &SafePassword,
) -> [u8; 16] {
    let context = [Z00ZKeyBranch::WalletBackup.as_aad_byte()];
    let aad = z00z_crypto::aead::build_aad_multipart(AeadEnvelopeDomain::domain(), &[&context[..]])
        .expect("wallet export aad");
    let payload = base64::engine::general_purpose::STANDARD
        .decode(export.ciphertext.as_bytes())
        .expect("payload base64");
    let prefix_len = EXPORT_MAGIC.len();
    assert!(payload.len() > prefix_len + 4, "framed export payload");

    let container: crate::security::encryption::EncryptedWalletContainer = BincodeCodec
        .deserialize(&payload[prefix_len + 4..])
        .expect("container decode");
    let plaintext =
        crate::security::encryption::WalletEncryption::decrypt_wallet(password, &aad, &container)
            .expect("decrypt export");
    let pack = BincodeCodec
        .deserialize::<WalletExportPack>(plaintext.as_ref())
        .expect("export pack decode");
    pack.wallet_profile
        .as_ref()
        .and_then(|profile| profile.seed_salt)
        .expect("seed salt present")
}

#[tokio::test]
async fn test_wallet_list_created_wallets() {
    let dir = TempDir::new().unwrap();
    let service = Arc::new(WalletService::with_output_dir_and_time(
        dir.path().join("wallets"),
        Arc::new(SystemTimeProvider),
    ));
    let app_service = Arc::new(AppService::with_wallet_service(service.clone()));
    let rpc = AppRpcImpl::new(app_service);

    let resp = rpc
        .create_wallet("test".to_string(), "StrongPassw0rd!".to_string(), None)
        .await
        .unwrap();
    let wallet_id = resp.wallet_id;

    let wallets = rpc.list_wallets().await.unwrap();
    assert_eq!(wallets.len(), 1);
    assert_eq!(wallets[0].id, wallet_id);
}

#[tokio::test]
async fn test_wallet_create_weak_password() {
    let service = Arc::new(WalletService::with_dependencies(Arc::new(
        SystemTimeProvider,
    )));
    let app_service = Arc::new(AppService::with_wallet_service(service));
    let rpc = AppRpcImpl::new(app_service);

    let err = rpc
        .create_wallet("test".to_string(), "short".to_string(), None)
        .await
        .unwrap_err();

    assert_eq!(err.code(), -32602);
}

#[tokio::test]
async fn test_wallet_create_24_word() {
    let service = Arc::new(WalletService::with_dependencies(Arc::new(
        SystemTimeProvider,
    )));
    let app_service = Arc::new(AppService::with_wallet_service(service));
    let rpc = AppRpcImpl::new(app_service);

    let resp = rpc
        .create_wallet("test".to_string(), "StrongPassw0rd!".to_string(), None)
        .await
        .unwrap();

    assert_eq!(resp.seed_phrase.split_whitespace().count(), 24);
}

#[tokio::test]
async fn test_wallet_create_common_password() {
    let service = Arc::new(WalletService::with_dependencies(Arc::new(
        SystemTimeProvider,
    )));
    let app_service = Arc::new(AppService::with_wallet_service(service));
    let rpc = AppRpcImpl::new(app_service);

    let err = rpc
        .create_wallet("test".to_string(), "Password123!".to_string(), None)
        .await
        .unwrap_err();

    assert_eq!(err.code(), -32602);
}

#[tokio::test]
async fn test_wallet_create_strength_score() {
    let service = Arc::new(WalletService::with_dependencies(Arc::new(
        SystemTimeProvider,
    )));
    let app_service = Arc::new(AppService::with_wallet_service(service));
    let rpc = AppRpcImpl::new(app_service);

    let password = "Aa1!Bb2@Cc3#Dd4$";
    let resp = rpc
        .create_wallet("test".to_string(), password.to_string(), None)
        .await
        .unwrap();

    let expected = PasswordValidator::default().strength_score(password);
    assert_eq!(resp.password_strength_score, expected);
}

#[tokio::test]
async fn test_wallet_delete_password_confirmation() {
    let service = Arc::new(WalletService::with_dependencies(Arc::new(
        SystemTimeProvider,
    )));
    let app_service = Arc::new(AppService::with_wallet_service(service));
    let rpc = AppRpcImpl::new(app_service);

    let err = rpc
        .delete_wallet(PersistWalletId("w".to_string()), "".to_string())
        .await
        .unwrap_err();

    assert_eq!(err.code(), -32602);
    assert_eq!(err.message(), "Password required");
}

#[tokio::test]
async fn test_wallet_import_invalid_backup() {
    let dir = TempDir::new().unwrap();
    let service = Arc::new(WalletService::with_output_dir_and_time(
        dir.path().join("wallets"),
        Arc::new(SystemTimeProvider),
    ));
    let app_service = Arc::new(AppService::with_wallet_service(service));
    let rpc = AppRpcImpl::new(app_service);

    let err = rpc
        .import_wallet("not-json".to_string(), "p".to_string(), "n".to_string())
        .await
        .unwrap_err();

    assert_eq!(err.code(), -32602);
}

#[tokio::test]
async fn test_wallet_import_valid_backup() {
    let src_dir = TempDir::new().unwrap();
    let src_service = WalletService::with_output_dir(src_dir.path().join("wallets_src"));
    let src_password = SafePassword::from("StrongPassw0rd!");
    let wallet_id = src_service
        .create_wallet_in_memory("test", src_password, TEST_SEED_PHRASE_24)
        .await
        .unwrap();
    let export_password = SafePassword::from("StrongPassw0rd!");
    let exported = src_service
        .export_wallet_payload(&wallet_id, &export_password)
        .await
        .unwrap();
    let codec = JsonCodec;
    let data = String::from_utf8(codec.serialize(&exported).unwrap()).unwrap();

    let dst_dir = TempDir::new().unwrap();
    let service = Arc::new(WalletService::with_output_dir(
        dst_dir.path().join("wallets_dst"),
    ));
    let app_service = Arc::new(AppService::with_wallet_service(service));
    let rpc = AppRpcImpl::new(app_service);
    let resp = rpc
        .import_wallet(data, "StrongPassw0rd!".to_string(), "n".to_string())
        .await
        .unwrap();

    assert!(resp.status.success);
}

#[tokio::test]
async fn test_wallet_export_payload_unlocked() {
    let service = Arc::new(WalletService::new());
    let create_password = SafePassword::from("StrongPassw0rd!");
    let unlock_password = SafePassword::from("StrongPassw0rd!");
    let wallet_id = service
        .create_wallet_in_memory("test", create_password, TEST_SEED_PHRASE_24)
        .await
        .unwrap();
    let _session = service
        .unlock_wallet_in_memory(&wallet_id, &unlock_password)
        .await
        .unwrap();

    let app_service = Arc::new(AppService::with_wallet_service(service));
    let rpc = AppRpcImpl::new(app_service);

    let resp = rpc
        .export_wallet(wallet_id, "StrongPassw0rd!".to_string())
        .await
        .unwrap();

    let encrypted = resp.encrypted_payload.expect("encrypted payload");
    assert!(encrypted.is_encrypted());
}

#[tokio::test]
async fn test_wallet_export_wallet_returns() {
    let dir = TempDir::new().unwrap();
    let service = Arc::new(WalletService::with_output_dir(dir.path().join("wallets")));
    let create_password = SafePassword::from("StrongPassw0rd!");
    let unlock_password = SafePassword::from("StrongPassw0rd!");
    let wallet_id = service
        .create_wallet_in_memory("test", create_password, TEST_SEED_PHRASE_24)
        .await
        .unwrap();
    let session = service
        .unlock_wallet_in_memory(&wallet_id, &unlock_password)
        .await
        .unwrap();

    let wallet_rpc = WalletRpcImpl::new(service.clone());
    let app_service = Arc::new(AppService::with_wallet_service(service));
    let rpc = AppRpcImpl::new(app_service);

    wallet_rpc.lock_wallet(session).await.unwrap();

    let err = rpc
        .export_wallet(wallet_id, "p".to_string())
        .await
        .unwrap_err();

    assert_eq!(err.code(), -32003);
    assert_eq!(err.message(), "Wallet locked");
}

#[tokio::test]
async fn test_wallet_unlock_limits_5() {
    let dir = TempDir::new().unwrap();
    let service = Arc::new(WalletService::with_output_dir(dir.path().join("wallets")));
    let password = SafePassword::from("StrongPassw0rd!");
    let wallet_id = service
        .create_wallet_in_memory("test", password, TEST_SEED_PHRASE_24)
        .await
        .unwrap();

    let rpc = WalletRpcImpl::new(Arc::clone(&service));

    for _ in 0..5 {
        let token = rpc
            .unlock_wallet(wallet_id.clone(), "StrongPassw0rd!".to_string())
            .await
            .unwrap();
        assert_eq!(token.wallet_id, wallet_id);
    }

    let err = rpc
        .unlock_wallet(wallet_id, "StrongPassw0rd!".to_string())
        .await
        .unwrap_err();

    assert_eq!(err.code(), SecurityErrorCode::RateLimitExceeded.code());
}

#[tokio::test]
async fn test_wallet_unlock_paces_invalid() {
    let time = Arc::new(MockTimeProvider::default());
    let dir = TempDir::new().unwrap();
    let mut service =
        WalletService::with_output_dir_and_time(dir.path().join("wallets"), time.clone());
    service.set_sleeper(Arc::new(MockSleeper::new(time.clone())));
    let service = Arc::new(service);
    let password = SafePassword::from("StrongPassw0rd!");
    let wallet_id = service
        .create_wallet_in_memory("test", password, TEST_SEED_PHRASE_24)
        .await
        .unwrap();
    let rpc = WalletRpcImpl::new(service);

    let t0 = time.compat_unix_timestamp_millis();

    let err = rpc
        .unlock_wallet(wallet_id.clone(), "".to_string())
        .await
        .unwrap_err();
    assert_eq!(err.code(), SecurityErrorCode::AuthenticationFailed.code());

    let t1 = time.compat_unix_timestamp_millis();
    assert_eq!(t1.saturating_sub(t0), 200);

    let token = rpc
        .unlock_wallet(wallet_id.clone(), "StrongPassw0rd!".to_string())
        .await
        .unwrap();
    assert_eq!(token.wallet_id, wallet_id);
}

#[tokio::test]
async fn test_wallet_unlock_auth_fail() {
    use crate::domains::hashing::compute_wallet_file_id;
    use redb::{ReadableTable, TableDefinition};
    use z00z_utils::compression::{zstd_compress, zstd_decompress_bounded};
    use z00z_utils::io;

    const MAX_DECOMPRESSED_WLT_BYTES: usize = 128 * 1024 * 1024;
    const SECRETS_TABLE: TableDefinition<&str, &[u8]> = TableDefinition::new("secrets");
    const SECRETS_MASTER_KEY: &str = "master_key";

    let dir = TempDir::new().unwrap();
    let wallets_dir = dir.path().join("wallets");
    let service = Arc::new(WalletService::with_output_dir(wallets_dir.clone()));
    let password = SafePassword::from("StrongPassw0rd!");

    let wallet_id = service
        .create_wallet_in_memory("test", password, TEST_SEED_PHRASE_24)
        .await
        .unwrap();

    let hash = compute_wallet_file_id(&wallet_id.0);
    let wallet_id_hex = hex::encode(&hash[..8]);
    let wlt_path = wallets_dir.join(format!("wallet_{wallet_id_hex}.wlt"));
    assert!(wlt_path.exists());

    let zstd = io::read_file(&wlt_path).unwrap();
    let db_bytes = zstd_decompress_bounded(&zstd, MAX_DECOMPRESSED_WLT_BYTES).unwrap();
    let work_path = tempfile::Builder::new()
        .prefix("z00z_wallet_unlock_corrupt_")
        .suffix(".wlt.work")
        .tempfile_in(std::env::temp_dir())
        .unwrap()
        .into_temp_path();
    let work_path_buf = work_path.to_path_buf();
    io::atomic_write_file_private(&work_path_buf, &db_bytes).unwrap();

    let db = redb::Database::open(&work_path_buf).unwrap();
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

    let tampered_bytes = io::read_file(&work_path_buf).unwrap();
    let tampered_zstd = zstd_compress(&tampered_bytes).unwrap();
    io::atomic_write_file_private(&wlt_path, &tampered_zstd).unwrap();

    let rpc = WalletRpcImpl::new(service);
    let err = rpc
        .unlock_wallet(wallet_id, "StrongPassw0rd!".to_string())
        .await
        .unwrap_err();
    assert_eq!(err.code(), SecurityErrorCode::AuthenticationFailed.code());
    assert_eq!(
        err.message(),
        SecurityErrorCode::AuthenticationFailed.message()
    );
}

#[tokio::test]
async fn test_wallet_unlock_kdf_rejected() {
    use crate::db::wallet_store_crypto::{KdfAlgo, KdfParams};
    use crate::domains::hashing::compute_wallet_file_id;
    use redb::{ReadableTable, TableDefinition};
    use z00z_utils::compression::{zstd_compress, zstd_decompress_bounded};
    use z00z_utils::io;

    const MAX_DECOMPRESSED_WLT_BYTES: usize = 128 * 1024 * 1024;
    const META_TABLE: TableDefinition<&str, &[u8]> = TableDefinition::new("meta");
    const META_WALLET_KDF: &str = "wallet.kdf";

    let dir = TempDir::new().unwrap();
    let wallets_dir = dir.path().join("wallets");
    let service = Arc::new(WalletService::with_output_dir(wallets_dir.clone()));
    let password = SafePassword::from("StrongPassw0rd!");

    let wallet_id = service
        .create_wallet_in_memory("test", password, TEST_SEED_PHRASE_24)
        .await
        .unwrap();

    let hash = compute_wallet_file_id(&wallet_id.0);
    let wallet_id_hex = hex::encode(&hash[..8]);
    let wlt_path = wallets_dir.join(format!("wallet_{wallet_id_hex}.wlt"));
    assert!(wlt_path.exists());

    let zstd = io::read_file(&wlt_path).unwrap();
    let db_bytes = zstd_decompress_bounded(&zstd, MAX_DECOMPRESSED_WLT_BYTES).unwrap();
    let work_path = tempfile::Builder::new()
        .prefix("z00z_wallet_unlock_kdf_")
        .suffix(".wlt.work")
        .tempfile_in(std::env::temp_dir())
        .unwrap()
        .into_temp_path();
    let work_path_buf = work_path.to_path_buf();
    io::atomic_write_file_private(&work_path_buf, &db_bytes).unwrap();

    let db = redb::Database::open(&work_path_buf).unwrap();
    let write_txn = db.begin_write().unwrap();
    {
        let mut meta = write_txn.open_table(META_TABLE).unwrap();
        let codec = BincodeCodec;

        let kdf_bytes = meta.get(META_WALLET_KDF).unwrap().unwrap().value().to_vec();
        let mut kdf: KdfParams = codec.deserialize(&kdf_bytes).unwrap();
        kdf.algo = KdfAlgo::Scrypt;

        let new_bytes = codec.serialize(&kdf).unwrap();
        meta.insert(META_WALLET_KDF, new_bytes.as_slice()).unwrap();
    }
    write_txn.commit().unwrap();

    let tampered_bytes = io::read_file(&work_path_buf).unwrap();
    let tampered_zstd = zstd_compress(&tampered_bytes).unwrap();
    io::atomic_write_file_private(&wlt_path, &tampered_zstd).unwrap();

    let rpc = WalletRpcImpl::new(service);
    let err = rpc
        .unlock_wallet(wallet_id, "StrongPassw0rd!".to_string())
        .await
        .unwrap_err();
    assert_eq!(err.code(), -32023);
    assert_eq!(err.message(), "Unsupported format");
}

#[tokio::test]
async fn test_wallet_show_phrase_encrypted() {
    let dir = TempDir::new().unwrap();
    let service = Arc::new(WalletService::with_output_dir(dir.path().join("wallets")));

    let password = SafePassword::from("StrongPassw0rd!");
    let seed_phrase = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon art";

    let wallet_id = service
        .create_wallet_in_memory("test", password.clone(), seed_phrase)
        .await
        .unwrap();

    let session = service
        .unlock_wallet_in_memory(&wallet_id, &password)
        .await
        .unwrap();

    let export = service
        .export_wallet_payload(&wallet_id, &password)
        .await
        .unwrap();
    let salt = decode_export_seed_salt(&export, &password);

    let rpc = WalletRpcImpl::new(service);

    let resp = rpc
        .show_seed_phrase(
            session,
            "StrongPassw0rd!".to_string(),
            "I understand".to_string(),
        )
        .await
        .unwrap();

    assert!(resp.encrypted_payload.is_encrypted());

    let nonce_hex = resp
        .encrypted_payload
        .metadata
        .nonce
        .strip_prefix("0x")
        .unwrap_or(&resp.encrypted_payload.metadata.nonce);
    let nonce_bytes = hex::decode(nonce_hex).unwrap();
    assert_eq!(nonce_bytes.len(), z00z_crypto::aead::XCHACHA_NONCE_SIZE);
    let mut nonce = [0u8; z00z_crypto::aead::XCHACHA_NONCE_SIZE];
    nonce.copy_from_slice(&nonce_bytes);

    let aad = z00z_crypto::aead::build_aad_multipart(
        "wallet.seed_phrase_response",
        &[wallet_id.0.as_bytes()],
    )
    .unwrap();

    let mut key =
        crate::security::encryption::WalletEncryption::derive_key(&password, &salt).unwrap();

    let mut envelope = Vec::new();
    envelope.push(z00z_crypto::aead::XCHACHA20_POLY1305_ID);
    envelope.extend_from_slice(&nonce);
    envelope.extend_from_slice(&hex::decode(&resp.encrypted_payload.ciphertext).unwrap());

    let recovered = z00z_crypto::aead::open(&key, &aad, &envelope).unwrap();
    key.fill(0);
    assert_eq!(String::from_utf8(recovered).unwrap(), seed_phrase);
}

#[tokio::test]
async fn test_wallet_show_phrase_rate() {
    let time = Arc::new(MockTimeProvider::default());
    let service = Arc::new(WalletService::with_dependencies(time));
    let session = SessionToken {
        token: "deadbeef".to_string(),
        wallet_id: crate::rpc::types::common::PersistWalletId("w".to_string()),
        created_at: 0,
        expires_at: 0,
        last_activity_at: 0,
        permissions: vec![],
    };

    let rpc = WalletRpcImpl::new(service);

    for _ in 0..4 {
        let err = rpc
            .show_seed_phrase(
                session.clone(),
                "pw".to_string(),
                "I understand".to_string(),
            )
            .await
            .unwrap_err();
        assert_eq!(err.code(), SecurityErrorCode::SessionInvalid.code());
    }

    let err = rpc
        .show_seed_phrase(session, "pw".to_string(), "I understand".to_string())
        .await
        .unwrap_err();

    assert_eq!(err.code(), SecurityErrorCode::SessionInvalid.code());
}

#[tokio::test]
async fn test_wallet_limit_auto_lock() {
    let time = Arc::new(MockTimeProvider::from_unix_secs(1));
    let dir = TempDir::new().unwrap();
    let mut service =
        WalletService::with_output_dir_and_time(dir.path().join("wallets"), time.clone());
    service.set_test_auto_lock_policy(AutoLockPolicy::new(Duration::from_millis(100), vec![]));
    let service = Arc::new(service);

    let password = SafePassword::from("StrongPassw0rd!");
    let wallet_id = service
        .create_wallet_in_memory("test", password.clone(), TEST_SEED_PHRASE_24)
        .await
        .unwrap();
    let session = service
        .unlock_wallet_in_memory(&wallet_id, &password)
        .await
        .unwrap();

    let rpc = WalletRpcImpl::new(service.clone());

    for _ in 0..3 {
        let err = rpc
            .show_seed_phrase(
                session.clone(),
                "StrongPassw0rd!".to_string(),
                "nope".to_string(),
            )
            .await
            .unwrap_err();
        assert_eq!(err.code(), -32602);
    }

    time.advance_by(Duration::from_millis(99));

    let err = rpc
        .show_seed_phrase(
            session.clone(),
            "StrongPassw0rd!".to_string(),
            "nope".to_string(),
        )
        .await
        .unwrap_err();
    assert_eq!(err.code(), SecurityErrorCode::RateLimitExceeded.code());

    time.advance_by(Duration::from_millis(1));

    let expired = service.check_auto_lock().await.unwrap();
    assert_eq!(expired, vec![wallet_id]);
}

#[tokio::test]
async fn test_wallet_password_auto_lock() {
    let time = Arc::new(MockTimeProvider::from_unix_secs(1));
    let dir = TempDir::new().unwrap();
    let mut service =
        WalletService::with_output_dir_and_time(dir.path().join("wallets"), time.clone());
    service.set_test_auto_lock_policy(AutoLockPolicy::new(Duration::from_millis(100), vec![]));
    let service = Arc::new(service);

    let password = SafePassword::from("StrongPassw0rd!");
    let wallet_id = service
        .create_wallet_in_memory("test", password.clone(), TEST_SEED_PHRASE_24)
        .await
        .unwrap();
    let session = service
        .unlock_wallet_in_memory(&wallet_id, &password)
        .await
        .unwrap();

    let rpc = WalletRpcImpl::new(service.clone());

    time.advance_by(Duration::from_millis(99));

    let err = rpc
        .show_seed_phrase(
            session,
            "WrongPassw0rd!".to_string(),
            "I understand".to_string(),
        )
        .await
        .unwrap_err();
    assert_eq!(err.code(), SecurityErrorCode::AuthenticationFailed.code());

    time.advance_by(Duration::from_millis(1));

    let expired = service.check_auto_lock().await.unwrap();
    assert_eq!(expired, vec![wallet_id]);
}

#[tokio::test]
async fn test_wallet_show_needs_confirm() {
    let dir = TempDir::new().unwrap();
    let service = Arc::new(WalletService::with_output_dir(dir.path().join("wallets")));

    let password = SafePassword::from("StrongPassw0rd!");
    let seed_phrase = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon art";

    let wallet_id = service
        .create_wallet_in_memory("test", password.clone(), seed_phrase)
        .await
        .unwrap();

    let session = service
        .unlock_wallet_in_memory(&wallet_id, &password)
        .await
        .unwrap();

    let rpc = WalletRpcImpl::new(service);

    let err = rpc
        .show_seed_phrase(session, "StrongPassw0rd!".to_string(), "nope".to_string())
        .await
        .unwrap_err();

    assert_eq!(err.code(), -32602, "InvalidParams must map to -32602");
}

#[tokio::test]
async fn test_wallet_delete_password_provided() {
    let dir = TempDir::new().unwrap();
    let service = Arc::new(WalletService::with_output_dir(dir.path().join("wallets")));
    let app_service = Arc::new(AppService::with_wallet_service(service));
    let rpc = AppRpcImpl::new(app_service);

    let wallet_id = rpc
        .create_wallet("test".to_string(), "StrongPassw0rd!".to_string(), None)
        .await
        .unwrap()
        .wallet_id;

    let resp = rpc
        .delete_wallet(wallet_id, "StrongPassw0rd!".to_string())
        .await
        .unwrap();

    assert!(resp.status.success);
}
