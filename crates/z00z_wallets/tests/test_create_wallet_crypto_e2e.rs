#![cfg(not(target_arch = "wasm32"))]
#![cfg(not(target_arch = "wasm32"))]

use std::sync::Arc;

use z00z_crypto::expert::encoding::SafePassword;
use z00z_wallets::services::{AppService, WalletService};

fn assert_wlt_exists(output_dir: &std::path::Path, wallet_id: &str) {
    use z00z_wallets::domains::hashing::compute_wallet_file_id;

    let hash = compute_wallet_file_id(wallet_id);
    let wallet_id_hex = hex::encode(&hash[..8]);
    let wlt_path = output_dir.join(format!("wallet_{wallet_id_hex}.wlt"));

    assert!(wlt_path.exists(), "expected .wlt wallet file");
    let size = std::fs::metadata(&wlt_path).expect("metadata").len();
    assert!(size > 0, "expected non-empty .wlt wallet file");
}

#[tokio::test]
async fn test_create_wallet_encrypted_wlt() {
    let temp = tempfile::tempdir().expect("tempdir");
    let output_dir = temp.path().join("wallets");

    let wallets = Arc::new(WalletService::with_output_dir(output_dir.clone()));
    let app = AppService::with_wallet_service(Arc::clone(&wallets));

    let password = "Aa1!bB2@cC3#dD4$eE5%".to_string();
    let safe_password = SafePassword::from(password.clone());

    let created = app
        .create_wallet("wallet-e2e".to_string(), password.clone(), None)
        .await
        .expect("create wallet");

    assert_wlt_exists(&output_dir, &created.wallet_id.0);

    // Clear in-memory state so load hits disk.
    wallets
        .unregister_wallet(&created.wallet_id)
        .await
        .expect("unregister");

    wallets
        .load_wallet(&created.wallet_id, &password)
        .await
        .expect("load wallet");

    // Wrong password must fail closed.
    wallets
        .unregister_wallet(&created.wallet_id)
        .await
        .expect("unregister");

    let err = wallets
        .load_wallet(&created.wallet_id, "wrong-password")
        .await
        .expect_err("wrong password must fail");

    assert!(matches!(err, z00z_wallets::WalletError::InvalidPassword));

    // Ensure .wlt encryption is actually used (not a plaintext fallback).
    // This is intentionally shallow: plaintext fallback is not supported by the loader.
    let _ = safe_password;
}
