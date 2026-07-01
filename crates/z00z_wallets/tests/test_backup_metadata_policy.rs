#![cfg(not(target_arch = "wasm32"))]

use std::time::{Duration, SystemTime};

use z00z_crypto::expert::encoding::SafePassword;
use z00z_utils::codec::{Codec, JsonCodec, Value};
use z00z_utils::io::{read_file, read_to_string, write_file};
use z00z_utils::rng::SystemRngProvider;
use z00z_utils::time::MockTimeProvider;
use z00z_wallets::backup::{
    BackupExporter, BackupExporterImpl, BackupImporter, BackupImporterImpl,
};
use z00z_wallets::db::{BackupManifestPayload, WalletProfilePayload};
use z00z_wallets::rpc::types::{common::PersistWalletId, wallet::PersistWalletSettings};
use z00z_wallets::wallet::persistence::{
    PasswordVerifierState, ReceiverDeriverState, WalletExportPack,
};
use z00z_wallets::wallet::WalletState;

const TEST_SEED_PHRASE: &str = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon art";

fn test_export_pack() -> WalletExportPack {
    let profile = WalletProfilePayload::new_with_checksum(
        PersistWalletId("wallet-1".to_string()),
        "Test Wallet".to_string(),
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
        [6u8; 16],
        WalletState::Locked,
    );
    let mut manifest = BackupManifestPayload {
        version: BackupManifestPayload::VERSION,
        wallet_id: PersistWalletId("wallet-1".to_string()),
        created_at_ms: 1,
        network: "testnet".to_string(),
        chain: "devnet".to_string(),
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
        seed_phrase: TEST_SEED_PHRASE.to_string(),
        wallet_identity: None,
    }
}

#[test]
fn test_backup_redacts_network() {
    let exporter = BackupExporterImpl::new_with_chain(
        "wallet-1".to_string(),
        "testnet".to_string(),
        "devnet".to_string(),
        test_export_pack(),
        MockTimeProvider::new(SystemTime::UNIX_EPOCH + Duration::from_secs(1)),
        SystemRngProvider,
    );

    let password = SafePassword::from("password");
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("backup.json");
    exporter
        .export(path.to_string_lossy().as_ref(), &password)
        .unwrap();

    let importer = BackupImporterImpl::new();
    let metadata = importer
        .read_metadata(path.to_string_lossy().as_ref())
        .unwrap();
    assert_eq!(metadata.version, 4);
    assert_eq!(metadata.wallet_id, "wallet-1");
    assert!(metadata.network.is_empty());

    let imported = importer
        .import(path.to_string_lossy().as_ref(), &password)
        .unwrap();
    assert_eq!(imported.wallet_id, "wallet-1");
    assert_eq!(imported.network, "testnet");
}

#[test]
fn test_backup_header_redacted() {
    let exporter = BackupExporterImpl::new_with_chain(
        "wallet-1".to_string(),
        "testnet".to_string(),
        "devnet".to_string(),
        test_export_pack(),
        MockTimeProvider::new(SystemTime::UNIX_EPOCH + Duration::from_secs(1)),
        SystemRngProvider,
    );

    let password = SafePassword::from("password");
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("backup.json");
    exporter
        .export(path.to_string_lossy().as_ref(), &password)
        .unwrap();

    let body = read_to_string(&path).unwrap();
    assert!(body.contains("\"wallet_id\":\"wallet-1\""));
    assert!(!body.contains("testnet"));
    assert!(!body.contains("devnet"));
    assert!(!body.contains(TEST_SEED_PHRASE));
}

#[test]
fn test_backup_tamper_closed() {
    let exporter = BackupExporterImpl::new_with_chain(
        "wallet-1".to_string(),
        "testnet".to_string(),
        "devnet".to_string(),
        test_export_pack(),
        MockTimeProvider::new(SystemTime::UNIX_EPOCH + Duration::from_secs(1)),
        SystemRngProvider,
    );

    let password = SafePassword::from("password");
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("backup.json");
    exporter
        .export(path.to_string_lossy().as_ref(), &password)
        .unwrap();

    let codec = JsonCodec;
    let mut body: Value = codec
        .deserialize(&read_file(&path).expect("backup file"))
        .expect("backup json");
    body["metadata"]["wallet_id"] = Value::String("wallet-2".to_string());
    write_file(&path, &codec.serialize(&body).expect("backup json bytes")).unwrap();

    let importer = BackupImporterImpl::new();
    assert!(
        !importer
            .verify_password(path.to_string_lossy().as_ref(), &password)
            .unwrap(),
        "tampered metadata must break the wallet-backup AAD contract"
    );
    assert!(importer
        .import(path.to_string_lossy().as_ref(), &password)
        .is_err());
}
