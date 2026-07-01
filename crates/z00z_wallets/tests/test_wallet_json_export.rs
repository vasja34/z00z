#![cfg(not(target_arch = "wasm32"))]

//! Example test demonstrating encrypted wallet JSON export/import.
//!
//! This uses the encrypted payload flow (`export_wallet`/`import_wallet`) and never
//! relies on plaintext persistence formats.

use std::sync::Arc;

use z00z_crypto::expert::encoding::SafePassword;
use z00z_wallets::services::{AppService, WalletService};
use z00z_wallets::wallet::WalletState;

#[tokio::test]
async fn test_create_wallet_encrypted_json() {
    println!("\n🧪 Z00Z Wallet Encrypted JSON Export Example");
    println!("===========================================\n");

    let temp = tempfile::tempdir().expect("tempdir");
    let output_dir_a = temp.path().join("wallets-a");
    let output_dir_b = temp.path().join("wallets-b");

    let wallets_a = Arc::new(WalletService::with_output_dir(output_dir_a));
    let app_a = AppService::with_wallet_service(Arc::clone(&wallets_a));

    let wallet_name = "Example Test Wallet";
    let password = "Aa1!bB2@cC3#dD4$eE5%".to_string();
    let safe_password = SafePassword::from(password.clone());

    println!("📝 Creating wallet:");
    println!("   Name: {wallet_name}");

    let created = app_a
        .create_wallet(wallet_name.to_string(), password.clone(), None)
        .await
        .expect("create wallet");

    // Export precondition: wallet must be unlocked.
    wallets_a
        .unlock_wallet_in_memory(&created.wallet_id, &safe_password)
        .await
        .expect("unlock");

    let exported = app_a
        .export_wallet(created.wallet_id.clone(), password.clone())
        .await
        .expect("export wallet");

    let payload = exported
        .encrypted_payload
        .expect("encrypted payload expected");

    let payload_json = serde_json::to_string_pretty(&payload).expect("serialize payload");
    println!("\n📄 Encrypted payload JSON preview:");
    println!("{}", payload_json);

    let wallets_b = Arc::new(WalletService::with_output_dir(output_dir_b));
    let app_b = AppService::with_wallet_service(Arc::clone(&wallets_b));

    let imported = app_b
        .import_wallet(payload_json, password, "imported".to_string())
        .await
        .expect("import wallet");

    let state = wallets_b
        .get_wallet_state(&imported.wallet_id)
        .await
        .expect("get wallet state");
    assert_eq!(state, WalletState::Locked);
}
#[tokio::test]
async fn test_multiple_wallets_encrypted_json() {
    let temp = tempfile::tempdir().expect("tempdir");
    let output_dir = temp.path().join("wallets");

    let wallets = Arc::new(WalletService::with_output_dir(output_dir));
    let app = AppService::with_wallet_service(Arc::clone(&wallets));

    for i in 0..3 {
        let name = format!("Wallet {i}");
        let password = format!("Aa1!bB2@cC3#dD4$eE5%_{i}");
        let safe_password = SafePassword::from(password.clone());

        let created = app
            .create_wallet(name.clone(), password.clone(), None)
            .await
            .expect("create wallet");

        wallets
            .unlock_wallet_in_memory(&created.wallet_id, &safe_password)
            .await
            .expect("unlock");

        let exported = app
            .export_wallet(created.wallet_id.clone(), password)
            .await
            .expect("export wallet");

        assert!(exported.encrypted_payload.is_some());
    }
}
