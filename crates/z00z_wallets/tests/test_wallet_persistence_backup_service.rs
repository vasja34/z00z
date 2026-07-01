#![cfg(not(target_arch = "wasm32"))]

use std::sync::Arc;

use z00z_crypto::expert::encoding::SafePassword;

use z00z_wallets::rpc::types::common::PersistWalletId;
use z00z_wallets::services::{AppService, WalletService};
use z00z_wallets::wallet::WalletState;

mod helpers {
    use z00z_wallets::domains::hashing::compute_wallet_file_id;

    pub fn wallet_id_to_file_stem(wallet_id: &str) -> String {
        let hash = compute_wallet_file_id(wallet_id);
        format!("wallet_{}", hex::encode(&hash[..8]))
    }
}

fn wlt_path(output_dir: &std::path::Path, wallet_id: &PersistWalletId) -> std::path::PathBuf {
    output_dir
        .join(helpers::wallet_id_to_file_stem(&wallet_id.0))
        .with_extension("wlt")
}

#[tokio::test]
async fn test_save_wallet_writes_encrypted() {
    let temp = tempfile::tempdir().expect("tempdir");
    let output_dir = temp.path().join("wallets");

    let wallets = Arc::new(WalletService::with_output_dir(output_dir.clone()));
    let app = AppService::with_wallet_service(Arc::clone(&wallets));

    let password = "Aa1!bB2@cC3#dD4$eE5%".to_string();
    let safe_password = SafePassword::from(password.clone());

    let created = app
        .create_wallet("wallet-a".to_string(), password, None)
        .await
        .expect("create wallet");

    wallets
        .save_wallet(created.wallet_id.clone(), safe_password, None)
        .await
        .expect("save wallet");

    let wlt_path = wlt_path(&output_dir, &created.wallet_id);

    assert!(wlt_path.exists(), "expected .wlt wallet file");
    let size = std::fs::metadata(&wlt_path).expect("metadata").len();
    assert!(size > 0, "expected non-empty .wlt wallet file");
}

#[tokio::test]
async fn test_load_wallet_id_restores() {
    let temp = tempfile::tempdir().expect("tempdir");
    let output_dir = temp.path().join("wallets");

    let wallets = Arc::new(WalletService::with_output_dir(output_dir.clone()));
    let app = AppService::with_wallet_service(Arc::clone(&wallets));

    let password = "Aa1!bB2@cC3#dD4$eE5%".to_string();
    let safe_password = SafePassword::from(password.clone());

    let created = app
        .create_wallet("wallet-a".to_string(), password.clone(), None)
        .await
        .expect("create wallet");

    let mut settings = wallets
        .get_wallet_settings(&created.wallet_id)
        .await
        .expect("get settings");
    settings.currency_display = "USD".to_string();
    settings.auto_lock_timeout = 123;

    wallets
        .set_wallet_settings(created.wallet_id.clone(), settings.clone())
        .await
        .expect("set settings");

    wallets
        .save_wallet(created.wallet_id.clone(), safe_password, None)
        .await
        .expect("save wallet");

    let wallets2 = WalletService::with_output_dir(output_dir);
    wallets2
        .load_wallet(&created.wallet_id, &password)
        .await
        .expect("load wallet");

    let state = wallets2
        .get_wallet_state(&created.wallet_id)
        .await
        .expect("get wallet state");
    assert_eq!(state, WalletState::Locked);

    let loaded_settings = wallets2
        .get_wallet_settings(&created.wallet_id)
        .await
        .expect("get settings");
    assert_eq!(loaded_settings.currency_display, settings.currency_display);
    assert_eq!(
        loaded_settings.auto_lock_timeout,
        settings.auto_lock_timeout
    );
}

#[tokio::test]
async fn test_load_wallet_fallback_plaintext() {
    let temp = tempfile::tempdir().expect("tempdir");
    let output_dir = temp.path().join("wallets");

    let wallets = Arc::new(WalletService::with_output_dir(output_dir.clone()));
    let app = AppService::with_wallet_service(Arc::clone(&wallets));

    let password = "Aa1!bB2@cC3#dD4$eE5%".to_string();
    let safe_password = SafePassword::from(password.clone());

    let created = app
        .create_wallet("wallet-a".to_string(), password.clone(), None)
        .await
        .expect("create wallet");

    wallets
        .save_wallet(created.wallet_id.clone(), safe_password, None)
        .await
        .expect("save wallet");

    // Remove `.wlt` so only prior plaintext files remain on disk.
    let wlt_path = wlt_path(&output_dir, &created.wallet_id);
    std::fs::remove_file(&wlt_path).expect("remove wlt");

    // Write prior plaintext files which must not be used as a fallback.
    let bin_path = output_dir
        .join(helpers::wallet_id_to_file_stem(&created.wallet_id.0))
        .with_extension("bin");
    std::fs::write(&bin_path, b"not a valid bincode snapshot").expect("write corrupted bin");

    let json_path = bin_path.with_extension("json");
    std::fs::write(&json_path, b"not valid json").expect("write json");

    const MAX_TEST_FILE_SIZE: u64 = 1024 * 1024;

    let bin_before =
        z00z_utils::io::read_file_bounded(&bin_path, MAX_TEST_FILE_SIZE).expect("read bin");
    let json_before =
        z00z_utils::io::read_file_bounded(&json_path, MAX_TEST_FILE_SIZE).expect("read json");

    let wallets2 = WalletService::with_output_dir(output_dir);
    let err = wallets2
        .load_wallet(&created.wallet_id, &password)
        .await
        .unwrap_err();

    assert!(matches!(err, z00z_wallets::WalletError::InvalidConfig(_)));
    assert!(!wlt_path.exists(), "must not recreate .wlt from plaintext");

    let bin_after =
        z00z_utils::io::read_file_bounded(&bin_path, MAX_TEST_FILE_SIZE).expect("read bin after");
    let json_after =
        z00z_utils::io::read_file_bounded(&json_path, MAX_TEST_FILE_SIZE).expect("read json after");
    assert_eq!(bin_after, bin_before, "bin artifact must not be modified");
    assert_eq!(
        json_after, json_before,
        "json artifact must not be modified"
    );
}

#[tokio::test]
async fn test_export_import_roundtrip_restores() {
    let temp = tempfile::tempdir().expect("tempdir");
    let output_dir_a = temp.path().join("wallets-a");
    let output_dir_b = temp.path().join("wallets-b");

    let wallets_a = Arc::new(WalletService::with_output_dir(output_dir_a));
    let app_a = AppService::with_wallet_service(Arc::clone(&wallets_a));

    let password = "Aa1!bB2@cC3#dD4$eE5%".to_string();

    let created = app_a
        .create_wallet("wallet-a".to_string(), password.clone(), None)
        .await
        .expect("create wallet");

    // Ensure export precondition: wallet must be unlocked.
    let safe_password = SafePassword::from(password.clone());
    wallets_a
        .unlock_wallet_in_memory(&created.wallet_id, &safe_password)
        .await
        .expect("unlock");

    let mut settings = wallets_a
        .get_wallet_settings(&created.wallet_id)
        .await
        .expect("get settings");
    settings.currency_display = "EUR".to_string();
    settings.auto_lock_timeout = 999;

    wallets_a
        .set_wallet_settings(created.wallet_id.clone(), settings.clone())
        .await
        .expect("set settings");

    let exported = app_a
        .export_wallet(created.wallet_id.clone(), password.clone())
        .await
        .expect("export wallet");

    let payload = exported
        .encrypted_payload
        .expect("encrypted payload expected");

    assert!(
        !payload.ciphertext.trim().is_empty(),
        "ciphertext must be non-empty"
    );
    assert_ne!(payload.metadata.algorithm, "none");

    let payload_json = serde_json::to_string(&payload).expect("serialize payload");

    // Wrong password must fail.
    let wallets_b = Arc::new(WalletService::with_output_dir(output_dir_b));
    let app_b = AppService::with_wallet_service(Arc::clone(&wallets_b));

    let wrong = app_b
        .import_wallet(
            payload_json.clone(),
            "wrong-password".to_string(),
            "imported".to_string(),
        )
        .await;
    assert!(wrong.is_err(), "wrong password should fail");

    let imported = app_b
        .import_wallet(payload_json, password, "imported".to_string())
        .await
        .expect("import wallet");

    assert_eq!(imported.wallet_id, created.wallet_id);

    let imported_settings = wallets_b
        .get_wallet_settings(&imported.wallet_id)
        .await
        .expect("get imported settings");

    assert_eq!(
        imported_settings.currency_display,
        settings.currency_display
    );
    assert_eq!(
        imported_settings.auto_lock_timeout,
        settings.auto_lock_timeout
    );
}
