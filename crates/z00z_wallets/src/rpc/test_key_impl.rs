use super::*;
use crate::config::default_wallet_config_path;
use crate::receiver::request::{decode_request_compact, encode_request_compact};
use crate::receiver::RequestParams;
use crate::rpc::logging::RpcLoggingConfig;
use crate::rpc::types::common::PersistWalletId;
use crate::wallet::AutoLockPolicy;
use base64::Engine;
use std::ffi::OsString;
use std::future::Future;
use std::time::Duration;
use z00z_crypto::expert::encoding::SafePassword;
use z00z_utils::codec::{Codec, JsonCodec};
use z00z_utils::rng::SystemRngProvider;
use z00z_utils::time::{MockTimeProvider, SystemTimeProvider};

tokio::task_local! {
    static WALLET_CONFIG_ENV_LOCK_HELD: ();
}

fn assert_session_guard_error(err: ErrorObjectOwned) {
    assert!(
        matches!(err.code(), -32402 | -32403 | -32003),
        "expected session guard error, got {}: {}",
        err.code(),
        err.message()
    );
}

struct WalletConfigEnvRestore {
    prev_path: Option<OsString>,
    prev_network: Option<OsString>,
    prev_chain: Option<OsString>,
}

impl WalletConfigEnvRestore {
    fn capture() -> Self {
        Self {
            prev_path: std::env::var_os("Z00Z_WALLET_CONFIG_PATH"),
            prev_network: std::env::var_os("Z00Z_WALLET_NETWORK"),
            prev_chain: std::env::var_os("Z00Z_WALLET_CHAIN"),
        }
    }
}

impl Drop for WalletConfigEnvRestore {
    fn drop(&mut self) {
        match &self.prev_path {
            Some(value) => std::env::set_var("Z00Z_WALLET_CONFIG_PATH", value),
            None => std::env::remove_var("Z00Z_WALLET_CONFIG_PATH"),
        }
        match &self.prev_network {
            Some(value) => std::env::set_var("Z00Z_WALLET_NETWORK", value),
            None => std::env::remove_var("Z00Z_WALLET_NETWORK"),
        }
        match &self.prev_chain {
            Some(value) => std::env::set_var("Z00Z_WALLET_CHAIN", value),
            None => std::env::remove_var("Z00Z_WALLET_CHAIN"),
        }
    }
}

async fn with_wallet_env<F, Fut, T>(run: F) -> T
where
    F: FnOnce() -> Fut,
    Fut: Future<Output = T>,
{
    if WALLET_CONFIG_ENV_LOCK_HELD.try_with(|_| ()).is_ok() {
        return run().await;
    }

    let _lock = RpcLoggingConfig::__lock_wallet_config_env();
    let restore = WalletConfigEnvRestore::capture();
    let cfg_path = default_wallet_config_path();
    std::env::set_var("Z00Z_WALLET_CONFIG_PATH", cfg_path);
    std::env::remove_var("Z00Z_WALLET_NETWORK");
    std::env::remove_var("Z00Z_WALLET_CHAIN");
    let _restore = restore;
    WALLET_CONFIG_ENV_LOCK_HELD.scope((), run()).await
}

async fn with_default_wallet_env<F, Fut, T>(run: F) -> T
where
    F: FnOnce() -> Fut,
    Fut: Future<Output = T>,
{
    if WALLET_CONFIG_ENV_LOCK_HELD.try_with(|_| ()).is_ok() {
        return run().await;
    }

    let _lock = RpcLoggingConfig::__lock_wallet_config_env_async().await;
    let restore = WalletConfigEnvRestore::capture();
    std::env::remove_var("Z00Z_WALLET_CONFIG_PATH");
    std::env::remove_var("Z00Z_WALLET_NETWORK");
    std::env::remove_var("Z00Z_WALLET_CHAIN");
    let _restore = restore;
    WALLET_CONFIG_ENV_LOCK_HELD.scope((), run()).await
}

fn create_test_wallet_service(
    time_provider: Arc<dyn z00z_utils::time::TimeProvider>,
) -> (Arc<WalletService>, tempfile::TempDir) {
    let _lock = RpcLoggingConfig::__lock_wallet_config_env();
    let _restore = WalletConfigEnvRestore::capture();
    std::env::remove_var("Z00Z_WALLET_CONFIG_PATH");
    std::env::remove_var("Z00Z_WALLET_NETWORK");
    std::env::remove_var("Z00Z_WALLET_CHAIN");

    let dir = tempfile::tempdir().unwrap();
    let service = WalletService::create_service_custom_output_directory(
        dir.path().to_path_buf(),
        time_provider,
        SystemRngProvider,
    );
    (Arc::new(service), dir)
}

fn create_wallet_service_env(
    time_provider: Arc<dyn z00z_utils::time::TimeProvider>,
) -> (Arc<WalletService>, tempfile::TempDir) {
    let dir = tempfile::tempdir().unwrap();
    let service = WalletService::create_service_custom_output_directory(
        dir.path().to_path_buf(),
        time_provider,
        SystemRngProvider,
    );
    (Arc::new(service), dir)
}

fn test_wallet_service() -> (Arc<WalletService>, tempfile::TempDir) {
    create_test_wallet_service(Arc::new(SystemTimeProvider))
}

fn test_seed_phrase_24() -> &'static str {
    "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon art"
}

fn test_wallet_service_with_time(
    time: Arc<MockTimeProvider>,
) -> (Arc<WalletService>, tempfile::TempDir) {
    create_test_wallet_service(time)
}

async fn setup_wallet_and_session(
    service: &Arc<WalletService>,
) -> (PersistWalletId, SessionToken, SafePassword) {
    with_default_wallet_env(|| async {
        let password = SafePassword::from("StrongPassw0rd!");
        let wallet_id = service
            .create_wallet_in_memory("test-wallet", password.clone(), test_seed_phrase_24())
            .await
            .unwrap();
        let session = service
            .unlock_wallet_in_memory(&wallet_id, &password)
            .await
            .unwrap();
        (wallet_id, session, password)
    })
    .await
}

#[tokio::test]
async fn test_session_expired_rpc_code() {
    let time = Arc::new(MockTimeProvider::default());
    let (service, _dir) = test_wallet_service_with_time(time.clone());
    let key_rpc = KeyRpcImpl::new(service);
    let (_wallet_id, session, _password) = setup_wallet_and_session(&key_rpc.service).await;

    time.advance_by(AutoLockPolicy::default().timeout + Duration::from_millis(1));

    let err = key_rpc
        .derive_receiver(session, "m/44'/0'/0'/0/0".to_string())
        .await
        .unwrap_err();

    assert_eq!(err.code(), -32402);
}

#[tokio::test]
async fn test_receiver_ignores_env_drift() {
    with_default_wallet_env(|| async {
        let (service, _dir) = create_wallet_service_env(Arc::new(SystemTimeProvider));
        let key_rpc = KeyRpcImpl::new(service);
        let (_wallet_id, session, _password) = setup_wallet_and_session(&key_rpc.service).await;

        let missing_dir = tempfile::tempdir().unwrap();
        let missing_cfg = missing_dir.path().join("missing-wallet-config.yaml");
        std::env::set_var("Z00Z_WALLET_CONFIG_PATH", &missing_cfg);

        let response = key_rpc
            .derive_receiver(session, "m/44'/1337'/0'/0/0".to_string())
            .await
            .unwrap();

        assert_eq!(response.path, "m/44'/1337'/0'/0/0");
        assert!(!response.public_key.is_empty());
    })
    .await;
}

#[tokio::test]
async fn test_derive_key_stub() {
    let (service, _dir) = test_wallet_service();
    let key_rpc = KeyRpcImpl::new(service);
    let (_wallet_id, session, _password) = setup_wallet_and_session(&key_rpc.service).await;

    let response = key_rpc
        .derive_receiver(session, "m/44'/1337'/0'/0/0".to_string())
        .await
        .unwrap();

    assert!(!response.public_key.is_empty());
    assert_eq!(response.path, "m/44'/1337'/0'/0/0");
}

#[tokio::test]
async fn test_get_receiver_card_ok() {
    with_wallet_env(|| async {
        let (service, _dir) = create_wallet_service_env(Arc::new(SystemTimeProvider));
        let key_rpc = KeyRpcImpl::new(service);
        let (_wallet_id, session, _password) = setup_wallet_and_session(&key_rpc.service).await;

        let response = key_rpc.get_receiver_card(session).await.unwrap();

        assert_eq!(response.owner_handle.len(), 64);
        assert_eq!(response.view_key.len(), 64);
        assert_eq!(response.identity_key.len(), 64);
        assert_eq!(response.signature.len(), 128);
        assert!(!response.card_compact.is_empty());
        assert_eq!(response.card_epoch, 0);
        assert_eq!(response.registry_entry_id.len(), 64);
        assert!(response.owner_handle_display.starts_with("z00z1"));

        let decoded = ReceiverCardRecord::from_compact(&response.card_compact, None)
            .unwrap()
            .decode_card()
            .unwrap();
        decoded.validate_signature().unwrap();
        assert_eq!(response.owner_handle, hex::encode(decoded.owner_handle));
        assert_eq!(response.view_key, hex::encode(decoded.view_pk));
        assert_eq!(response.identity_key, hex::encode(decoded.identity_pk));
        assert_eq!(response.signature, hex::encode(decoded.signature));

        let expected_receiver_display = format_receiver_handle(&decoded.owner_handle).unwrap();
        assert_eq!(response.owner_handle_display, expected_receiver_display);

        let validation = key_rpc
            .validate_receiver_card(response.card_compact.clone())
            .await
            .unwrap();
        assert!(validation.result.valid);
        assert_eq!(validation.format.as_deref(), Some("receiver_card"));
    })
    .await;
}

#[tokio::test]
async fn test_receiver_card_bad_session() {
    with_wallet_env(|| async {
        let (service, _dir) = create_wallet_service_env(Arc::new(SystemTimeProvider));
        let key_rpc = KeyRpcImpl::new(service);
        let (_wallet_id, mut session, _password) = setup_wallet_and_session(&key_rpc.service).await;

        session.token = "invalid-session-token".to_string();
        let err = key_rpc.get_receiver_card(session).await.unwrap_err();
        assert_eq!(err.code(), -32403);
    })
    .await;
}

#[tokio::test]
async fn test_create_payment_request_ok() {
    let (service, _dir) = test_wallet_service();
    let key_rpc = KeyRpcImpl::new(service);
    let (_wallet_id, session, _password) = setup_wallet_and_session(&key_rpc.service).await;

    let payment_id_hex = "11".repeat(16);
    let response = key_rpc
        .create_payment_request(
            session,
            Some(77),
            3600,
            Some(RuntimePaymentRequestMetaInput {
                memo: Some("test memo".to_string()),
                payment_id: Some(payment_id_hex.clone()),
            }),
        )
        .await
        .unwrap();

    assert_eq!(response.amount, Some(77));
    assert_eq!(response.chain_id, 3);
    assert_eq!(response.owner_handle.len(), 64);
    assert_eq!(response.signature.len(), 128);

    let decoded = decode_request_compact(&response.request_compact).unwrap();
    decoded.verify().unwrap();
    assert_eq!(response.req_id, hex::encode(decoded.req_id));
    assert_eq!(response.owner_handle, hex::encode(decoded.owner_handle));
    assert_eq!(
        decoded
            .metadata
            .as_ref()
            .and_then(|item| item.memo.as_ref()),
        Some(&"test memo".to_string())
    );
    assert_eq!(
        decoded.metadata.and_then(|item| item.payment_id),
        Some([0x11u8; 16])
    );
}

#[tokio::test]
async fn test_rejects_bad_payment_id() {
    let (service, _dir) = test_wallet_service();
    let key_rpc = KeyRpcImpl::new(service);
    let (_wallet_id, session, _password) = setup_wallet_and_session(&key_rpc.service).await;

    let err = key_rpc
        .create_payment_request(
            session,
            None,
            60,
            Some(RuntimePaymentRequestMetaInput {
                memo: None,
                payment_id: Some("aa".repeat(8)),
            }),
        )
        .await
        .unwrap_err();

    assert_eq!(err.code(), -32602);
}

#[tokio::test]
async fn test_validate_req_ok() {
    with_wallet_env(|| async {
        let (service, _dir) = create_wallet_service_env(Arc::new(SystemTimeProvider));
        let key_rpc = KeyRpcImpl::new(service);
        let (_wallet_id, session, _password) = setup_wallet_and_session(&key_rpc.service).await;

        let created = key_rpc
            .create_payment_request(session.clone(), Some(19), 600, None)
            .await
            .unwrap();

        let result = key_rpc
            .validate_payment_request(session, created.request_compact)
            .await
            .unwrap();

        assert!(result.result.valid);
        assert_eq!(result.outcome, Some("confirm".to_string()));
        assert!(result.req_id.is_some());
        assert!(result.owner_handle.is_some());
        assert!(result.expiry.is_some());
    })
    .await;
}

#[tokio::test]
async fn test_req_tofu_pins() {
    with_wallet_env(|| async {
        let (service, _dir) = create_wallet_service_env(Arc::new(SystemTimeProvider));
        let key_rpc = KeyRpcImpl::new(service);
        let (_wallet_id, session, _password) = setup_wallet_and_session(&key_rpc.service).await;

        let created = key_rpc
            .create_payment_request(session.clone(), Some(19), 600, None)
            .await
            .unwrap();

        let first = key_rpc
            .validate_payment_request(session.clone(), created.request_compact.clone())
            .await
            .unwrap();
        let second = key_rpc
            .validate_payment_request(session, created.request_compact)
            .await
            .unwrap();

        assert_eq!(first.outcome, Some("confirm".to_string()));
        assert_eq!(second.outcome, Some("approved".to_string()));
    })
    .await;
}

#[tokio::test]
async fn test_req_chain_mismatch() {
    with_wallet_env(|| async {
        let (service, _dir) = create_wallet_service_env(Arc::new(SystemTimeProvider));
        let key_rpc = KeyRpcImpl::new(service);
        let (wallet_id, session, _password) = setup_wallet_and_session(&key_rpc.service).await;
        let keys = key_rpc.service.receiver_keys(&wallet_id).await.unwrap();
        let request = PaymentRequest::generate(
            &keys,
            RequestParams {
                amount: Some(19),
                expiry_seconds: 600,
                memo: None,
                payment_id: None,
            },
            2,
        )
        .unwrap();

        let err = key_rpc
            .validate_payment_request(session, encode_request_compact(&request))
            .await
            .unwrap_err();

        assert_eq!(err.code(), -32602);
        assert!(err.message().contains("REQUEST_CHAIN_MISMATCH"));
    })
    .await;
}

#[tokio::test]
async fn test_validate_req_expired_code() {
    with_wallet_env(|| async {
        let (service, _dir) = create_wallet_service_env(Arc::new(SystemTimeProvider));
        let key_rpc = KeyRpcImpl::new(service);
        let (wallet_id, session, _password) = setup_wallet_and_session(&key_rpc.service).await;
        let keys = key_rpc.service.receiver_keys(&wallet_id).await.unwrap();
        let mut request = PaymentRequest::generate(
            &keys,
            RequestParams {
                amount: Some(19),
                expiry_seconds: 600,
                memo: None,
                payment_id: None,
            },
            3,
        )
        .unwrap();
        request.expiry = 0;
        request.sign(keys.reveal_identity_sk()).unwrap();

        let err = key_rpc
            .validate_payment_request(session, encode_request_compact(&request))
            .await
            .unwrap_err();

        assert_eq!(err.code(), -32602);
        assert!(err.message().contains("REQUEST_EXPIRED"));
    })
    .await;
}

#[tokio::test]
async fn test_req_bad_signature() {
    with_wallet_env(|| async {
        let (service, _dir) = create_wallet_service_env(Arc::new(SystemTimeProvider));
        let key_rpc = KeyRpcImpl::new(service);
        let (wallet_id, session, _password) = setup_wallet_and_session(&key_rpc.service).await;
        let keys = key_rpc.service.receiver_keys(&wallet_id).await.unwrap();
        let mut request = PaymentRequest::generate(
            &keys,
            RequestParams {
                amount: Some(19),
                expiry_seconds: 600,
                memo: None,
                payment_id: None,
            },
            3,
        )
        .unwrap();
        request.amount = Some(20);

        let err = key_rpc
            .validate_payment_request(session, encode_request_compact(&request))
            .await
            .unwrap_err();

        assert_eq!(err.code(), -32602);
        assert!(err.message().contains("REQUEST_INVALID_SIGNATURE"));
    })
    .await;
}

#[tokio::test]
async fn test_receiver_card_malformed() {
    let (service, _dir) = test_wallet_service();
    let key_rpc = KeyRpcImpl::new(service);

    let validation = key_rpc
        .validate_receiver_card("stub-card".to_string())
        .await
        .unwrap();

    assert!(!validation.result.valid);
    assert_eq!(validation.format, None);
}

#[tokio::test]
async fn test_validate_req_bad_compact() {
    with_wallet_env(|| async {
        let (service, _dir) = create_wallet_service_env(Arc::new(SystemTimeProvider));
        let key_rpc = KeyRpcImpl::new(service);
        let (_wallet_id, session, _password) = setup_wallet_and_session(&key_rpc.service).await;

        let err = key_rpc
            .validate_payment_request(session, "%%%not-base64%%%".to_string())
            .await
            .unwrap_err();

        assert_eq!(err.code(), -32602);
        assert!(err.message().contains("REQUEST_PAYLOAD_MALFORMED"));
    })
    .await;
}

#[tokio::test]
async fn test_validate_req_big_payload() {
    with_wallet_env(|| async {
        let (service, _dir) = create_wallet_service_env(Arc::new(SystemTimeProvider));
        let key_rpc = KeyRpcImpl::new(service);
        let (_wallet_id, session, _password) = setup_wallet_and_session(&key_rpc.service).await;

        let big = base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(vec![7u8; 9000]);

        let err = key_rpc
            .validate_payment_request(session, big)
            .await
            .unwrap_err();

        assert_eq!(err.code(), -32602);
        assert!(err.message().contains("REQUEST_PAYLOAD_OVERSIZED_OR_EMPTY"));
    })
    .await;
}

mod export_public_material_stub {
    use super::*;

    #[tokio::test]
    async fn test_export_public_material_stub() {
        let (service, _dir) = test_wallet_service();
        let key_rpc = KeyRpcImpl::new(service);
        let (_wallet_id, session, _password) = setup_wallet_and_session(&key_rpc.service).await;

        let response = key_rpc
            .export_public_material(session, 0, "password123".to_string())
            .await
            .unwrap();

        assert_eq!(response.schema_version, 2);
        assert_eq!(response.account, 0);
        assert_eq!(response.algorithm, "xchacha20poly1305");
        assert!(!response.encrypted_pub_material.is_empty());
        assert!(!response.fingerprint.is_empty());
    }
}

#[tokio::test]
async fn test_export_public_material_roundtrip() {
    use crate::security::encryption::WalletEncryption;
    use base64::Engine as _;
    use z00z_crypto::aead;

    let (service, _dir) = test_wallet_service();
    let key_rpc = KeyRpcImpl::new(service);
    let (wallet_id, session, _wallet_password) = setup_wallet_and_session(&key_rpc.service).await;
    let account = 7u32;
    let password = "password123";

    let response = key_rpc
        .export_public_material(session, account, password.to_string())
        .await
        .unwrap();

    let packed = base64::engine::general_purpose::STANDARD
        .decode(response.encrypted_pub_material.as_bytes())
        .unwrap();
    assert!(packed.len() > 16 + 24 + 16);

    let mut salt = [0u8; 16];
    salt.copy_from_slice(&packed[..16]);
    let envelope = &packed[16..];

    const AAD_DOMAIN: &str = "z00z.wallet.key.export_public_material";
    let mut aad_context = Vec::with_capacity(wallet_id.0.len() + 1 + 4);
    aad_context.extend_from_slice(wallet_id.0.as_bytes());
    aad_context.push(0);
    aad_context.extend_from_slice(&account.to_le_bytes());
    let aad = aead::build_aad_multipart(AAD_DOMAIN, &[aad_context.as_slice()]).unwrap();

    let safe_pw = SafePassword::from(password);
    let mut key = WalletEncryption::derive_key(&safe_pw, &salt).unwrap();
    let decrypted = aead::open(&key, &aad, envelope).unwrap();
    key.fill(0);

    let plaintext = String::from_utf8(decrypted).unwrap();
    assert!(plaintext.starts_with("z00z-pub-material-v1:"));
    assert!(plaintext.contains(&format!("account={account}")));
}

#[tokio::test]
async fn test_rotate_master_receipt() {
    let (service, _dir) = test_wallet_service();
    let key_rpc = KeyRpcImpl::new(service);
    let (_wallet_id, session, password) = setup_wallet_and_session(&key_rpc.service).await;

    let response = key_rpc
        .rotate_master_key(
            session,
            String::from_utf8(password.reveal().to_vec()).expect("password utf8"),
            "ROTATE".to_string(),
        )
        .await
        .unwrap();

    assert!(response.rotated_at > 0);
    assert!(!response.new_fingerprint.is_empty());
    assert!(response.records_rewrapped > 0);
}

#[tokio::test]
async fn test_rotate_master_revokes_session() {
    with_wallet_env(|| async {
        let (service, _dir) = create_wallet_service_env(Arc::new(SystemTimeProvider));
        let key_rpc = KeyRpcImpl::new(service);
        let (_wallet_id, session, password) = setup_wallet_and_session(&key_rpc.service).await;
        let password_text = String::from_utf8(password.reveal().to_vec()).expect("password utf8");

        key_rpc
            .rotate_master_key(session.clone(), password_text, "ROTATE".to_string())
            .await
            .unwrap();

        assert_session_guard_error(key_rpc.get_receiver_card(session).await.unwrap_err());
    })
    .await;
}

#[tokio::test]
async fn test_rotate_master_restart_ok() {
    with_wallet_env(|| async {
        let (service, dir) = create_wallet_service_env(Arc::new(SystemTimeProvider));
        let key_rpc = KeyRpcImpl::new(Arc::clone(&service));
        let (wallet_id, session, password) = setup_wallet_and_session(&key_rpc.service).await;
        let password_text = String::from_utf8(password.reveal().to_vec()).expect("password utf8");

        let response = key_rpc
            .rotate_master_key(session, password_text, "ROTATE".to_string())
            .await
            .unwrap();
        assert!(response.records_rewrapped > 0);

        service.lock_wallet(&wallet_id).await.unwrap();

        let restarted = Arc::new(WalletService::create_service_custom_output_directory(
            dir.path().to_path_buf(),
            Arc::new(SystemTimeProvider),
            SystemRngProvider,
        ));
        let restarted_rpc = KeyRpcImpl::new(Arc::clone(&restarted));

        let restarted_session = restarted
            .unlock_wallet_in_memory(&wallet_id, &password)
            .await
            .unwrap();
        let card = restarted_rpc
            .get_receiver_card(restarted_session)
            .await
            .unwrap();

        assert!(!card.owner_handle.is_empty());
    })
    .await;
}

#[tokio::test]
async fn test_rotate_master_failpoint_recovers() {
    let (service, dir) = test_wallet_service();
    let (wallet_id, session, password) = setup_wallet_and_session(&service).await;

    crate::db::redb_store::set_rotate_master_fp_commit(true);
    let err = service
        .rotate_master_key_persisted(&session, &password)
        .await
        .unwrap_err();
    assert!(matches!(
        err,
        crate::WalletError::InvalidConfig(message)
            if message.contains("injected rotate_master_key failure")
    ));

    service.lock_wallet(&wallet_id).await.unwrap();

    let restarted = Arc::new(WalletService::create_service_custom_output_directory(
        dir.path().to_path_buf(),
        Arc::new(SystemTimeProvider),
        SystemRngProvider,
    ));
    let restarted_rpc = KeyRpcImpl::new(Arc::clone(&restarted));
    let restarted_session = restarted
        .unlock_wallet_in_memory(&wallet_id, &password)
        .await
        .unwrap();
    let password_text = String::from_utf8(password.reveal().to_vec()).expect("password utf8");

    let response = restarted_rpc
        .rotate_master_key(restarted_session, password_text, "ROTATE".to_string())
        .await
        .unwrap();

    assert!(response.records_rewrapped > 0);
}

#[tokio::test]
async fn test_rotate_master_recovers_slot() {
    let (service, _dir) = test_wallet_service();
    let key_rpc = KeyRpcImpl::new(Arc::clone(&service));
    let (wallet_id, session, password) = setup_wallet_and_session(&service).await;
    let password_text = String::from_utf8(password.reveal().to_vec()).expect("password utf8");

    crate::db::redb_store::set_rotate_master_fp_commit(true);
    key_rpc
        .rotate_master_key(session, password_text.clone(), "ROTATE".to_string())
        .await
        .unwrap_err();

    let retry_session = service
        .unlock_wallet_in_memory(&wallet_id, &password)
        .await
        .unwrap();
    let response = key_rpc
        .rotate_master_key(retry_session, password_text, "ROTATE".to_string())
        .await
        .unwrap();

    assert!(response.records_rewrapped > 0);
}

#[tokio::test]
async fn test_master_rejects_bad_password() {
    let (service, _dir) = test_wallet_service();
    let key_rpc = KeyRpcImpl::new(service);
    let (_wallet_id, session, _password) = setup_wallet_and_session(&key_rpc.service).await;

    let err = key_rpc
        .rotate_master_key(session, "WrongPassw0rd!".to_string(), "ROTATE".to_string())
        .await
        .unwrap_err();

    assert_eq!(err.code(), -32007);
}

#[tokio::test]
async fn test_bad_password_keeps_slot() {
    let (service, _dir) = test_wallet_service();
    let key_rpc = KeyRpcImpl::new(service);
    let (_wallet_id, session, password) = setup_wallet_and_session(&key_rpc.service).await;
    let password_text = String::from_utf8(password.reveal().to_vec()).expect("password utf8");

    let err = key_rpc
        .rotate_master_key(
            session.clone(),
            "WrongPassw0rd!".to_string(),
            "ROTATE".to_string(),
        )
        .await
        .unwrap_err();
    assert_eq!(err.code(), -32007);

    let response = key_rpc
        .rotate_master_key(session, password_text, "ROTATE".to_string())
        .await
        .unwrap();

    assert!(response.records_rewrapped > 0);
}

#[tokio::test]
async fn test_rejects_non_literal_confirmation() {
    let (service, _dir) = test_wallet_service();
    let key_rpc = KeyRpcImpl::new(service);
    let (_wallet_id, session, password) = setup_wallet_and_session(&key_rpc.service).await;

    let err = key_rpc
        .rotate_master_key(
            session,
            String::from_utf8(password.reveal().to_vec()).expect("password utf8"),
            " ROTATE\n".to_string(),
        )
        .await
        .unwrap_err();

    assert_eq!(err.code(), -32602);
}

#[tokio::test]
async fn test_rate_limits_second_attempt() {
    let time = Arc::new(MockTimeProvider::default());
    let (service, _dir) = test_wallet_service_with_time(time);
    let key_rpc = KeyRpcImpl::new(service);
    let (wallet_id, session, password) = setup_wallet_and_session(&key_rpc.service).await;
    let password = String::from_utf8(password.reveal().to_vec()).expect("password utf8");

    key_rpc
        .rotate_master_key(session.clone(), password.clone(), "ROTATE".to_string())
        .await
        .unwrap();

    let second_session = key_rpc
        .service
        .unlock_wallet_in_memory(&wallet_id, &SafePassword::from(password.as_str()))
        .await
        .unwrap();

    let err = key_rpc
        .rotate_master_key(second_session, password, "ROTATE".to_string())
        .await
        .unwrap_err();

    assert_eq!(
        err.code(),
        crate::rpc::types::security::SecurityErrorCode::RateLimitExceeded.code()
    );
}

#[tokio::test]
async fn test_rotate_master_stale_session() {
    let time = Arc::new(MockTimeProvider::default());
    let (service, _dir) = test_wallet_service_with_time(time.clone());
    let key_rpc = KeyRpcImpl::new(service);
    let (_wallet_id, session, password) = setup_wallet_and_session(&key_rpc.service).await;

    time.advance_by(AutoLockPolicy::default().timeout + Duration::from_millis(1));

    assert_session_guard_error(
        key_rpc
            .rotate_master_key(
                session,
                String::from_utf8(password.reveal().to_vec()).expect("password utf8"),
                "ROTATE".to_string(),
            )
            .await
            .unwrap_err(),
    );
}

#[tokio::test]
async fn test_list_receivers_ids() {
    let (service, _dir) = test_wallet_service();
    let key_rpc = KeyRpcImpl::new(service);
    let (_wallet_id, session, _password) = setup_wallet_and_session(&key_rpc.service).await;

    let derived = key_rpc
        .derive_receiver(session.clone(), "m/44'/1337'/0'/0/0".to_string())
        .await
        .unwrap();

    let response = key_rpc
        .list_receivers(session, Some(10), None, None)
        .await
        .unwrap();

    assert_eq!(response.items.len(), 1);
    assert_eq!(response.items[0].receiver_id, derived.public_key);
    assert_eq!(response.items[0].public_key, derived.public_key);
    assert!(!response.items[0].internal);

    let json = JsonCodec
        .serialize(&response)
        .and_then(|bytes| JsonCodec.deserialize::<z00z_utils::codec::Value>(&bytes))
        .unwrap();
    assert!(json.get("items").is_some());
    assert!(json.get("addresses").is_none());
}

#[tokio::test]
async fn test_label_receiver_id() {
    let (service, _dir) = test_wallet_service();
    let key_rpc = KeyRpcImpl::new(service);
    let (_wallet_id, session, _password) = setup_wallet_and_session(&key_rpc.service).await;

    let derived = key_rpc
        .derive_receiver(session.clone(), "m/44'/1337'/0'/0/0".to_string())
        .await
        .unwrap();

    let response = key_rpc
        .label_receiver(
            session,
            derived.public_key.clone(),
            "Primary Receiver".to_string(),
        )
        .await
        .unwrap();

    assert!(response.status.success);
    assert_eq!(response.receiver_id, derived.public_key);
    assert_eq!(response.label, "Primary Receiver");

    let json = JsonCodec
        .serialize(&response)
        .and_then(|bytes| JsonCodec.deserialize::<z00z_utils::codec::Value>(&bytes))
        .unwrap();
    assert!(json.get("receiver_id").is_some());
    assert!(json.get("address").is_none());
}

#[tokio::test]
async fn test_derive_bad_path_code() {
    let (service, _dir) = test_wallet_service();
    let key_rpc = KeyRpcImpl::new(service);
    let (_wallet_id, session, _password) = setup_wallet_and_session(&key_rpc.service).await;

    let err = key_rpc
        .derive_receiver(session, "m/44/1337/0/0/0".to_string())
        .await
        .unwrap_err();

    assert_eq!(err.code(), -32602);
    assert!(err.message().contains("Invalid BIP44 path"));
}

#[tokio::test]
async fn test_derive_key_nonz00z_code() {
    let (service, _dir) = test_wallet_service();
    let key_rpc = KeyRpcImpl::new(service);
    let (_wallet_id, session, _password) = setup_wallet_and_session(&key_rpc.service).await;

    let err = key_rpc
        .derive_receiver(session, "m/44'/0'/0'/0/0".to_string())
        .await
        .unwrap_err();

    assert_eq!(err.code(), -32602);
    assert!(err.message().contains("non-standard"));
}

#[tokio::test]
async fn test_derive_bad_hardening_code() {
    let (service, _dir) = test_wallet_service();
    let key_rpc = KeyRpcImpl::new(service);
    let (_wallet_id, session, _password) = setup_wallet_and_session(&key_rpc.service).await;

    let err = key_rpc
        .derive_receiver(session, "m/44'/1337'/0'/0'/0".to_string())
        .await
        .unwrap_err();

    assert_eq!(err.code(), -32602);
    assert!(err.message().contains("Invalid BIP44 path"));
}

#[tokio::test]
async fn test_derive_bad_change_code() {
    let (service, _dir) = test_wallet_service();
    let key_rpc = KeyRpcImpl::new(service);
    let (_wallet_id, session, _password) = setup_wallet_and_session(&key_rpc.service).await;

    let err = key_rpc
        .derive_receiver(session, "m/44'/1337'/0'/2/0".to_string())
        .await
        .unwrap_err();

    assert_eq!(err.code(), -32602);
    assert!(err.message().contains("Invalid BIP44 path"));
}

#[tokio::test]
async fn test_derive_session_expired_code() {
    let time = Arc::new(MockTimeProvider::default());
    let (service, _dir) = test_wallet_service_with_time(time.clone());
    let key_rpc = KeyRpcImpl::new(service);
    let (_wallet_id, session, _password) = setup_wallet_and_session(&key_rpc.service).await;

    time.advance_by(AutoLockPolicy::default().timeout + Duration::from_millis(1));

    let err = key_rpc
        .derive_receiver(session, "m/44'/1337'/0'/0/0".to_string())
        .await
        .unwrap_err();

    assert_eq!(err.code(), -32402);
    assert!(err.message().contains("Session expired"));
}

#[tokio::test]
async fn test_derive_key_canonical_path() {
    let (service, _dir) = test_wallet_service();
    let key_rpc = KeyRpcImpl::new(service);
    let (_wallet_id, session, _password) = setup_wallet_and_session(&key_rpc.service).await;

    let test_cases = vec![
        ("m/44'/1337'/0'/0/0", "m/44'/1337'/0'/0/0"),
        ("m/44'/1337'/0'/0/5", "m/44'/1337'/0'/0/5"),
        ("m/44'/1337'/1'/1/10", "m/44'/1337'/1'/1/10"),
    ];

    for (input_path, expected_path) in test_cases {
        let response = key_rpc
            .derive_receiver(session.clone(), input_path.to_string())
            .await
            .unwrap();

        assert_eq!(response.path, expected_path);
        assert_eq!(response.path, input_path);
    }
}

#[tokio::test]
async fn test_derive_no_input_echo() {
    let (service, _dir) = test_wallet_service();
    let key_rpc = KeyRpcImpl::new(service);
    let (_wallet_id, session, _password) = setup_wallet_and_session(&key_rpc.service).await;

    let err = key_rpc
        .derive_receiver(session, "m/44/1337/0/0/0".to_string())
        .await
        .unwrap_err();

    assert!(!err.message().contains("m/44/1337/0/0/0"));
    assert!(err.message().contains("Invalid BIP44 path"));
}

#[tokio::test]
async fn test_derive_key_response_structure() {
    let (service, _dir) = test_wallet_service();
    let key_rpc = KeyRpcImpl::new(service);
    let (_wallet_id, session, _password) = setup_wallet_and_session(&key_rpc.service).await;

    let response = key_rpc
        .derive_receiver(session, "m/44'/1337'/0'/0/0".to_string())
        .await
        .unwrap();

    assert!(!response.public_key.is_empty());
    assert_eq!(response.path, "m/44'/1337'/0'/0/0");
    assert!(hex::decode(&response.public_key).is_ok());
}

#[tokio::test]
async fn test_derive_key_deterministic_response() {
    let (service, _dir) = test_wallet_service();
    let key_rpc = KeyRpcImpl::new(service);
    let (_wallet_id, session, _password) = setup_wallet_and_session(&key_rpc.service).await;

    let response1 = key_rpc
        .derive_receiver(session.clone(), "m/44'/1337'/0'/0/0".to_string())
        .await
        .unwrap();

    let response2 = key_rpc
        .derive_receiver(session.clone(), "m/44'/1337'/0'/0/0".to_string())
        .await
        .unwrap();

    assert_eq!(response1.public_key, response2.public_key);
    assert_eq!(response1.path, response2.path);
}
