#![cfg(not(target_arch = "wasm32"))]

use std::sync::Arc;
use std::time::{Duration, SystemTime};

use z00z_crypto::expert::encoding::SafePassword;
use z00z_utils::{rng::SystemRngProvider, time::MockTimeProvider};
use z00z_wallets::{
    backup::{BackupExporter, BackupExporterImpl, ForensicImportMode},
    db::discover_wallet_store,
    db::{BackupManifestPayload, WalletProfilePayload},
    rpc::types::{common::PersistWalletId, wallet::PersistWalletSettings},
    services::{AppService, WalletService},
    wallet::{
        persistence::{PasswordVerifierState, ReceiverDeriverState, WalletExportPack},
        WalletError, WalletState,
    },
};

#[path = "test_inc/test_wallet_env.inc"]
mod test_common;

const TEST_PASSWORD: &str = "Aa1!bB2@cC3#dD4$eE5%";

fn chainless_pack(wallet_id: &str) -> WalletExportPack {
    let profile = WalletProfilePayload::new_with_checksum(
        PersistWalletId(wallet_id.to_string()),
        "Chainless Backup".to_string(),
        1,
        2,
        PasswordVerifierState {
            salt: [1u8; 32],
            verifier: [2u8; 32],
        },
        ReceiverDeriverState {
            next_payment_index: 0,
            next_change_index: 0,
        },
        PersistWalletSettings {
            auto_lock_timeout: 300,
            default_fee: "0.001".to_string(),
            currency_display: "Z00Z".to_string(),
            policy_rules: None,
            created_at: 1,
            updated_at: 2,
        },
        [3u8; 16],
        WalletState::Locked,
    );
    let mut manifest = BackupManifestPayload {
        version: BackupManifestPayload::VERSION,
        wallet_id: PersistWalletId(wallet_id.to_string()),
        created_at_ms: 1,
        network: "testnet".to_string(),
        chain: "".to_string(),
        profile_count: 1,
        owned_asset_count: 0,
        owned_object_count: 0,
        scan_state_count: 0,
        stealth_meta_count: 0,
        tofu_pins_count: 0,
        key_ref_count: 0,
        tx_record_count: 0,
        has_tx_history_sidecar: true,
        tx_history_plane: BackupManifestPayload::TX_HISTORY_JSONL.to_string(),
        checksum: None,
    };
    manifest.checksum = Some(manifest.compute_checksum());

    WalletExportPack {
        version: WalletExportPack::VERSION,
        manifest: Some(manifest),
        wallet_profile: Some(profile),
        owned_assets: Vec::new(),
        owned_objects: Vec::new(),
        scan_state: None,
        stealth_meta: None,
        tofu_pins: None,
        keys: None,
        tx_history_plane: Some(BackupManifestPayload::TX_HISTORY_JSONL.to_string()),
        seed_phrase: "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon art".to_string(),
        wallet_identity: None,
    }
}

fn wlt_path(output_dir: &std::path::Path, wallet_id: &str) -> std::path::PathBuf {
    let hash = z00z_wallets::domains::hashing::compute_wallet_file_id(wallet_id);
    let wallet_id_hex = hex::encode(&hash[..8]);
    output_dir.join(format!("wallet_{wallet_id_hex}.wlt"))
}

async fn make_backup(
    wallets: &Arc<WalletService>,
    app: &AppService,
    backup_dir: &std::path::Path,
) -> (String, z00z_wallets::rpc::types::common::PersistWalletId) {
    let created = {
        let _env = test_common::WalletEnvGuard::new("p2p", "devnet");
        app.create_wallet(
            "wallet-backup-identity".to_string(),
            TEST_PASSWORD.to_string(),
            None,
        )
        .await
        .expect("create wallet")
    };

    let backup_path = {
        let _env = test_common::WalletEnvGuard::new("p2p", "devnet");
        wallets
            .create_backup(
                &created.wallet_id,
                SafePassword::from(TEST_PASSWORD),
                Some(backup_dir.to_string_lossy().to_string()),
            )
            .await
            .expect("create backup")
            .backup_path
    };

    (backup_path, created.wallet_id)
}

async fn restore_in_env(
    wallets: &WalletService,
    backup_path: String,
) -> z00z_wallets::rpc::types::common::PersistWalletId {
    let restored = {
        let _env = test_common::WalletEnvGuard::new("p2p", "mainnet");
        wallets
            .restore_backup(
                backup_path,
                SafePassword::from(TEST_PASSWORD),
                Some("restored-wallet".to_string()),
            )
            .await
            .expect("restore backup")
    };

    {
        let _env = test_common::WalletEnvGuard::new("p2p", "mainnet");
        let _session = wallets
            .unlock_wallet_in_memory(&restored.wallet_id, &SafePassword::from(TEST_PASSWORD))
            .await
            .expect("unlock restored wallet with preserved identity");

        wallets
            .unregister_wallet(&restored.wallet_id)
            .await
            .expect("release restored wallet session");
    }

    restored.wallet_id
}

#[tokio::test]
async fn test_restore_backup_chain_identity() {
    let src_temp = tempfile::tempdir().expect("src tempdir");
    let src_output = src_temp.path().join("wallets-src");
    let src_wallets = Arc::new(WalletService::with_output_dir(src_output));
    let src_app = AppService::with_wallet_service(Arc::clone(&src_wallets));

    let backup_temp = tempfile::tempdir().expect("backup tempdir");
    let backup_dir = backup_temp.path().join("backups");
    let (backup_path, created_wallet_id) = make_backup(&src_wallets, &src_app, &backup_dir).await;

    let dst_temp = tempfile::tempdir().expect("dst tempdir");
    let dst_output = dst_temp.path().join("wallets-dst");
    let dst_wallets = WalletService::with_output_dir(dst_output.clone());

    let restored_wallet_id = restore_in_env(&dst_wallets, backup_path).await;

    let discovered = discover_wallet_store(&wlt_path(&dst_output, &restored_wallet_id.0))
        .expect("discover restored wallet");

    assert_eq!(discovered.wallet_id, restored_wallet_id);
    assert_eq!(discovered.wallet_id, created_wallet_id);
    assert_eq!(discovered.chain, "devnet");
}

#[tokio::test]
async fn test_restore_backup_empty_chain() {
    let temp = tempfile::tempdir().expect("backup tempdir");
    let backup_path = temp.path().join("chainless-backup.json");
    let exporter = BackupExporterImpl::new(
        "wallet-chainless".to_string(),
        "testnet".to_string(),
        chainless_pack("wallet-chainless"),
        MockTimeProvider::new(SystemTime::UNIX_EPOCH + Duration::from_secs(1)),
        SystemRngProvider,
    );
    exporter
        .export(
            backup_path.to_string_lossy().as_ref(),
            &SafePassword::from(TEST_PASSWORD),
        )
        .expect("export chainless backup");

    let dst_temp = tempfile::tempdir().expect("dst tempdir");
    let dst_output = dst_temp.path().join("wallets-dst");
    let wallets = WalletService::with_output_dir(dst_output.clone());

    let err = {
        let _env = test_common::WalletEnvGuard::new("p2p", "mainnet");
        wallets
            .restore_backup_with_mode(
                backup_path.to_string_lossy().to_string(),
                SafePassword::from(TEST_PASSWORD),
                Some("restored-wallet".to_string()),
                ForensicImportMode::WalletOnly,
            )
            .await
            .expect_err("chain-less backup must fail closed")
    };

    match err {
        WalletError::InvalidConfig(message) => {
            assert!(message.contains("chain-bound backup payload"));
        }
        other => panic!("unexpected restore error: {other:?}"),
    }

    let has_wallet = std::fs::read_dir(&dst_output)
        .ok()
        .and_then(|mut entries| entries.next())
        .is_some();
    assert!(
        !has_wallet,
        "failed restore must not leave wallet artifacts"
    );
}
