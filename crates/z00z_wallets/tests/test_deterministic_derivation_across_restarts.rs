#![cfg(not(target_arch = "wasm32"))]

use std::sync::Arc;

use tempfile::TempDir;
use z00z_crypto::expert::encoding::SafePassword;
use z00z_utils::rng::SystemRngProvider;
use z00z_utils::time::{SystemTimeProvider, TimeProvider};
use z00z_wallets::key::Bip44Path;
use z00z_wallets::rpc::types::common::PersistWalletId;
use z00z_wallets::rpc::types::wallet::WalletSource;
use z00z_wallets::services::{AppService, ChainService, WalletService};

#[path = "test_inc/test_wallet_env.inc"]
mod test_common;

fn test_seed_phrase_24() -> &'static str {
    "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon art"
}

fn wallet_wlt_path(
    output_dir: &std::path::Path,
    wallet_id: &PersistWalletId,
) -> std::path::PathBuf {
    // Keep in sync with `WalletService::wlt_file_path`.
    use z00z_wallets::domains::hashing::compute_wallet_file_id;
    let hash = compute_wallet_file_id(&wallet_id.0);
    let wallet_id_hex = hex::encode(&hash[..8]);
    output_dir.join(format!("wallet_{wallet_id_hex}.wlt"))
}

fn build_services(output_dir: std::path::PathBuf) -> (AppService, Arc<WalletService>, TempDir) {
    let dir = tempfile::tempdir().unwrap();
    let time_provider: Arc<dyn TimeProvider> = Arc::new(SystemTimeProvider);

    let wallets = Arc::new(WalletService::create_service_custom_output_directory(
        output_dir,
        Arc::clone(&time_provider),
        SystemRngProvider,
    ));

    let chain_service = Arc::new(ChainService::with_dependencies(Arc::clone(&time_provider)));

    let app = AppService::with_dependencies_and_services(
        time_provider,
        Arc::clone(&wallets),
        chain_service,
    );

    (app, wallets, dir)
}

#[tokio::test]
async fn test_same_seed_pubkey() {
    // Ensure wallet identity is stable and matches recovery inputs.
    let _env = test_common::WalletEnvGuard::new("p2p", "devnet");

    let output_dir = tempfile::tempdir().unwrap();
    let output_dir_path = output_dir.path().to_path_buf();

    let (app, wallets, _tmp) = build_services(output_dir_path.clone());

    let password = "Aa1!bB2@cC3#dD4$eE5%";
    let seed = test_seed_phrase_24();

    let wallet_a = app
        .recover_from_seed(
            "wallet_a".to_string(),
            password.to_string(),
            seed.to_string(),
            seed.to_string(),
            "p2p".to_string(),
            "devnet".to_string(),
        )
        .await
        .unwrap();

    // Derive once in the initial process (baseline for restart determinism).
    wallets
        .unlock_wallet_in_memory(&wallet_a.wallet_id, &SafePassword::from(password))
        .await
        .unwrap();

    let initial_a = wallets
        .derive_public_key_for_path(&wallet_a.wallet_id, Bip44Path::payment(0).unwrap())
        .await
        .unwrap();
    let initial_keys = wallets.receiver_keys(&wallet_a.wallet_id).await.unwrap();
    let initial_card = initial_keys.export_receiver_card().unwrap();

    // Release the underlying `.wlt` advisory lock held by the open session so that the
    // restarted service can open the same wallet path.
    wallets.lock_wallet(&wallet_a.wallet_id).await.unwrap();

    // Simulate restart/device: new service instance + open by path.
    let time_provider: Arc<dyn TimeProvider> = Arc::new(SystemTimeProvider);
    let wallets_restart = WalletService::create_service_custom_output_directory(
        output_dir_path.clone(),
        Arc::clone(&time_provider),
        SystemRngProvider,
    );

    let wallet_a_path = wallet_wlt_path(&output_dir_path, &wallet_a.wallet_id);
    assert!(wallet_a_path.exists(), "expected wallet_a .wlt to exist");

    wallets_restart
        .open_wallet_source(WalletSource::Path {
            path: wallet_a_path.to_string_lossy().to_string(),
        })
        .await
        .unwrap();

    wallets_restart
        .unlock_wallet_in_memory(&wallet_a.wallet_id, &SafePassword::from(password))
        .await
        .unwrap();

    let restart_a = wallets_restart
        .derive_public_key_for_path(&wallet_a.wallet_id, Bip44Path::payment(0).unwrap())
        .await
        .unwrap();
    let restart_keys = wallets_restart
        .receiver_keys(&wallet_a.wallet_id)
        .await
        .unwrap();
    let restart_card = restart_keys.export_receiver_card().unwrap();

    assert_eq!(initial_a, restart_a, "wallet_a changed across restart");
    assert_ne!(restart_a, [0u8; 32]);
    assert_eq!(initial_card.owner_handle, restart_card.owner_handle);
    assert_eq!(initial_card.view_pk, restart_card.view_pk);
    assert_eq!(initial_card.identity_pk, restart_card.identity_pk);
}
