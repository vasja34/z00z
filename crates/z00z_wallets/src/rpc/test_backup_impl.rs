use super::*;
use crate::rpc::logging::RpcLoggingConfig;
use crate::rpc::types::security::SecurityErrorCode;
use std::ffi::OsString;
use tempfile::TempDir;
use z00z_crypto::expert::encoding::SafePassword;
use z00z_utils::time::{MockTimeProvider, SystemTimeProvider};

const TEST_SEED_PHRASE_24: &str = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon art";

async fn create_unlocked_session(
    service: &Arc<WalletService>,
) -> (PersistWalletId, SessionToken, String) {
    let password = "test-password".to_string();
    let wallet_id = service
        .create_wallet_in_memory(
            "test-wallet",
            SafePassword::from(password.clone()),
            TEST_SEED_PHRASE_24,
        )
        .await
        .expect("create_wallet_in_memory must succeed");

    let safe_password = SafePassword::from(password.clone());
    let session = service
        .unlock_wallet_in_memory(&wallet_id, &safe_password)
        .await
        .expect("unlock_wallet_in_memory must succeed");

    (wallet_id, session, password)
}

struct WalletConfigEnvRestore {
    prev_path: Option<OsString>,
}

impl WalletConfigEnvRestore {
    fn capture() -> Self {
        Self {
            prev_path: std::env::var_os("Z00Z_WALLET_CONFIG_PATH"),
        }
    }
}

impl Drop for WalletConfigEnvRestore {
    fn drop(&mut self) {
        match &self.prev_path {
            Some(value) => std::env::set_var("Z00Z_WALLET_CONFIG_PATH", value),
            None => std::env::remove_var("Z00Z_WALLET_CONFIG_PATH"),
        }
    }
}

fn write_wallet_config(dir: &TempDir, yaml: &str) -> std::path::PathBuf {
    let path = dir.path().join("wallet_config.yaml");
    z00z_utils::io::write_file(&path, yaml.as_bytes()).expect("wallet config must write");
    path
}

#[tokio::test]
async fn test_backup_create_rate_limit() {
    let _lock = RpcLoggingConfig::__lock_wallet_config_env_async().await;
    let time = MockTimeProvider::from_unix_secs(1);

    let service = Arc::new(WalletService::with_dependencies(Arc::new(
        SystemTimeProvider,
    )));
    let (wallet_id, _session, _password) = create_unlocked_session(&service).await;

    let rpc = Arc::new(BackupRpcImpl::with_dependencies(service, time.clone()));

    let mut handles = Vec::new();
    for _ in 0..16 {
        let rpc = Arc::clone(&rpc);
        let wallet_id = wallet_id.clone();
        handles.push(tokio::spawn(async move {
            rpc.backup_create_rate_limit_precheck(&wallet_id).await
        }));
    }

    let mut ok = 0usize;
    let mut rate_limited = 0usize;

    for handle in handles {
        match handle.await.unwrap() {
            Ok(()) => ok += 1,
            Err(err) => {
                assert_eq!(err.code(), SecurityErrorCode::RateLimitExceeded.code());
                rate_limited += 1;
            }
        }
    }

    assert_eq!(ok, 1);
    assert_eq!(rate_limited, 15);
}

#[tokio::test]
async fn test_use_yaml_base_directory() {
    let _lock = RpcLoggingConfig::__lock_wallet_config_env_async().await;
    let _restore = WalletConfigEnvRestore::capture();
    let cfg_dir = TempDir::new().unwrap();
    let backup_base = cfg_dir.path().join("yaml-backups");
    let cfg_path = write_wallet_config(
        &cfg_dir,
        &format!(
            r#"
wallet:
  backup:
    auto_backup_enabled: true
    backup_interval_hours: 6
    location: "{}"
    encrypt_backups: false
"#,
            backup_base.display()
        ),
    );
    std::env::set_var("Z00Z_WALLET_CONFIG_PATH", &cfg_path);

    let time = MockTimeProvider::from_unix_secs(1);

    let service = Arc::new(WalletService::with_dependencies(Arc::new(
        SystemTimeProvider,
    )));
    let (_wallet_id, session, _password) = create_unlocked_session(&service).await;

    let rpc = BackupRpcImpl::with_dependencies(service, time);

    let response = rpc
        .configure_backup_settings(session, None)
        .await
        .expect("default backup settings must resolve");

    let backup_location = std::path::PathBuf::from(response.settings.backup_location);
    assert!(
        backup_location.starts_with(&backup_base),
        "phase-047 requires wallet.backup.location to act as the base directory for per-wallet default backup paths"
    );
    assert_eq!(backup_location.parent(), Some(backup_base.as_path()));
    assert!(response.settings.auto_backup_enabled);
    assert_eq!(response.settings.backup_interval_hours, 6);
    assert!(!response.settings.encrypt_backups);
}

#[tokio::test]
async fn test_backup_create_list_restore() {
    let _lock = RpcLoggingConfig::__lock_wallet_config_env_async().await;
    let temp = TempDir::new().unwrap();
    let backup_dir = temp.path().join("backups");

    let time = MockTimeProvider::from_unix_secs(1);

    let service = Arc::new(WalletService::with_dependencies(Arc::new(
        SystemTimeProvider,
    )));
    let (wallet_id, session, password) = create_unlocked_session(&service).await;

    let rpc = BackupRpcImpl::with_dependencies(Arc::clone(&service), time.clone());

    let settings = PersistBackupSettings {
        auto_backup_enabled: false,
        backup_interval_hours: 24,
        backup_location: backup_dir.to_string_lossy().to_string(),
        encrypt_backups: true,
    };
    let resp = rpc
        .configure_backup_settings(session.clone(), Some(settings.clone()))
        .await
        .unwrap();
    assert_eq!(resp.settings.backup_location, settings.backup_location);

    let created = rpc
        .create_backup(session.clone(), password.clone(), None)
        .await
        .unwrap();
    assert!(created.status.success);
    assert!(created.encrypted);
    assert!(std::path::PathBuf::from(&created.backup_path).exists());

    let listed = rpc
        .list_backups(session.clone(), None, Some(10))
        .await
        .unwrap();
    assert_eq!(listed.items.len(), 1);
    assert!(listed.items[0].size_bytes > 0);

    service.unregister_wallet(&wallet_id).await.unwrap();

    let restored = rpc
        .restore_backup(created.backup_path.clone(), password, None)
        .await
        .unwrap();
    assert!(restored.status.success);
    assert_eq!(restored.wallet_id.0, wallet_id.0);

    let wallets = service.list_wallets_in_memory().await.unwrap();
    let restored_info = wallets
        .iter()
        .find(|item| item.id.0 == wallet_id.0)
        .expect("restored wallet info");
    assert_eq!(restored_info.name, "test-wallet");
}

#[tokio::test]
async fn test_backup_create_rate_limited() {
    let _lock = RpcLoggingConfig::__lock_wallet_config_env_async().await;
    let temp = TempDir::new().unwrap();
    let backup_dir = temp.path().join("backups");

    let time = MockTimeProvider::from_unix_secs(1);

    let service = Arc::new(WalletService::with_dependencies(Arc::new(
        SystemTimeProvider,
    )));
    let (_wallet_id, session, password) = create_unlocked_session(&service).await;

    let rpc = BackupRpcImpl::with_dependencies(service, time.clone());

    rpc.configure_backup_settings(
        session.clone(),
        Some(PersistBackupSettings {
            auto_backup_enabled: false,
            backup_interval_hours: 24,
            backup_location: backup_dir.to_string_lossy().to_string(),
            encrypt_backups: true,
        }),
    )
    .await
    .unwrap();

    let _ = rpc
        .create_backup(session.clone(), password.clone(), None)
        .await
        .unwrap();

    let err = rpc
        .create_backup(session.clone(), password, None)
        .await
        .unwrap_err();
    assert_eq!(err.code(), SecurityErrorCode::RateLimitExceeded.code());
}

#[tokio::test]
async fn test_backup_restore_wrong_password() {
    let _lock = RpcLoggingConfig::__lock_wallet_config_env_async().await;
    let temp = TempDir::new().unwrap();
    let backup_dir = temp.path().join("backups");

    let time = MockTimeProvider::from_unix_secs(1);

    let service = Arc::new(WalletService::with_dependencies(Arc::new(
        SystemTimeProvider,
    )));
    let (_wallet_id, session, password) = create_unlocked_session(&service).await;

    let rpc = BackupRpcImpl::with_dependencies(service, time);

    rpc.configure_backup_settings(
        session.clone(),
        Some(PersistBackupSettings {
            auto_backup_enabled: false,
            backup_interval_hours: 24,
            backup_location: backup_dir.to_string_lossy().to_string(),
            encrypt_backups: true,
        }),
    )
    .await
    .unwrap();

    let created = rpc
        .create_backup(session.clone(), password, None)
        .await
        .unwrap();

    let err = rpc
        .restore_backup(created.backup_path, "wrong".to_string(), None)
        .await
        .unwrap_err();
    assert_eq!(err.code(), SecurityErrorCode::AuthenticationFailed.code());
}
