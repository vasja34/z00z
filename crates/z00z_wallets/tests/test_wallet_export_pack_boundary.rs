#![cfg(not(target_arch = "wasm32"))]

use std::time::{Duration, SystemTime};

use z00z_crypto::expert::encoding::SafePassword;
use z00z_utils::io::{read_file, read_to_string, write_file};
use z00z_utils::rng::SystemRngProvider;
use z00z_utils::time::MockTimeProvider;
use z00z_wallets::backup::{
    encode_tx_history_jsonl, BackupExporter, BackupExporterImpl, BackupImporter,
    BackupImporterImpl, ForensicImportMode,
};
use z00z_wallets::db::{BackupManifestPayload, WalletProfilePayload};
use z00z_wallets::domains::hashing::compute_wallet_file_id;
use z00z_wallets::persistence::tx::{TxRecord, TxStatus};
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
fn test_backup_seed_encrypted() {
    let exporter = BackupExporterImpl::new_with_chain(
        "wallet-1".to_string(),
        "testnet".to_string(),
        "devnet".to_string(),
        test_export_pack(),
        MockTimeProvider::new(SystemTime::UNIX_EPOCH + Duration::from_secs(1)),
        SystemRngProvider,
    );

    let password = SafePassword::from("password");
    let bytes = exporter.export_to_bytes(&password).unwrap();
    let serialized = String::from_utf8(bytes).unwrap();
    let json: serde_json::Value = serde_json::from_str(&serialized).unwrap();

    assert!(!serialized.contains(TEST_SEED_PHRASE));
    assert_eq!(json["metadata"]["wallet_id"], "wallet-1");
    assert_eq!(json["metadata"]["network"], "");

    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("backup.json");
    std::fs::write(&path, serialized.as_bytes()).unwrap();

    let importer = BackupImporterImpl::new();
    let imported = importer
        .import(path.to_string_lossy().as_ref(), &password)
        .unwrap();
    assert_eq!(imported.export_pack.unwrap().seed_phrase, TEST_SEED_PHRASE);
}

#[test]
fn test_forensic_export_requires_jsonl() {
    let exporter = BackupExporterImpl::new_with_forensic_history(
        "wallet-1".to_string(),
        "testnet".to_string(),
        "devnet".to_string(),
        test_export_pack(),
        vec![sample_tx_record("tx-1")],
        MockTimeProvider::new(SystemTime::UNIX_EPOCH + Duration::from_secs(1)),
        SystemRngProvider,
    );

    let password = SafePassword::from("password");
    let dir = tempfile::tempdir().unwrap();
    let archive_path = dir.path().join("backup.json");
    let wallet_stem = wallet_stem("wallet-1");
    let jsonl_path = dir
        .path()
        .join(format!("wallet_{wallet_stem}_tx_history.jsonl"));
    let history_bytes = history_jsonl_bytes(&[sample_tx_record("tx-1")], true);
    write_file(&jsonl_path, &history_bytes).unwrap();

    let err = exporter
        .export(archive_path.to_string_lossy().as_ref(), &password)
        .unwrap_err();
    assert!(err.to_string().contains("export_with_history_bytes"));

    exporter
        .export_with_history_bytes(
            archive_path.to_string_lossy().as_ref(),
            &password,
            &history_bytes,
        )
        .unwrap();

    assert!(archive_path.exists());
    assert!(jsonl_path.exists());
    let jsonl = read_to_string(&jsonl_path).unwrap();
    assert!(jsonl.contains("\"tx_hash\":\"tx-1\""));
    assert!(!jsonl.contains(TEST_SEED_PHRASE));

    let importer = BackupImporterImpl::new();
    let imported = importer
        .import_with_mode(
            archive_path.to_string_lossy().as_ref(),
            &password,
            ForensicImportMode::WalletPlusHistory,
        )
        .unwrap();
    assert_eq!(imported.forensic_archive.unwrap().records.len(), 1);
    assert_eq!(read_file(&jsonl_path).unwrap(), history_bytes);

    let history =
        BackupImporterImpl::import_history_jsonl(jsonl_path.to_string_lossy().as_ref()).unwrap();
    assert_eq!(history.len(), 1);
    assert_eq!(history[0].tx_hash, "tx-1");
    assert_eq!(history[0].tx_bytes, vec![1, 2, 3, 4]);
}

#[test]
fn test_forensic_rejects_mismatched_jsonlbytes() {
    let exporter = BackupExporterImpl::new_with_forensic_history(
        "wallet-1".to_string(),
        "testnet".to_string(),
        "devnet".to_string(),
        test_export_pack(),
        vec![sample_tx_record("tx-1")],
        MockTimeProvider::new(SystemTime::UNIX_EPOCH + Duration::from_secs(1)),
        SystemRngProvider,
    );

    let password = SafePassword::from("password");
    let dir = tempfile::tempdir().unwrap();
    let archive_path = dir.path().join("backup.json");
    let mismatched_history_bytes = history_jsonl_bytes(&[], false);

    let err = exporter
        .export_with_history_bytes(
            archive_path.to_string_lossy().as_ref(),
            &password,
            &mismatched_history_bytes,
        )
        .unwrap_err();

    assert!(err
        .to_string()
        .contains("canonical tx-history JSONL does not match forensic history"));
    assert!(!archive_path.exists());
}

#[test]
fn forensic_export_emits_jsonl() {
    let records = vec![sample_tx_record("tx-1"), sample_tx_record("tx-2")];
    let exporter = BackupExporterImpl::new_with_forensic_history(
        "wallet-1".to_string(),
        "testnet".to_string(),
        "devnet".to_string(),
        test_export_pack(),
        records.clone(),
        MockTimeProvider::new(SystemTime::UNIX_EPOCH + Duration::from_secs(1)),
        SystemRngProvider,
    );

    let password = SafePassword::from("password");
    let dir = tempfile::tempdir().unwrap();
    let archive_path = dir.path().join("wallet-1-forensic.backup");
    let wallet_stem = wallet_stem("wallet-1");
    let wallet_file_path = dir.path().join(format!("wallet_{wallet_stem}.wlt"));
    let jsonl_path = dir
        .path()
        .join(format!("wallet_{wallet_stem}_tx_history.jsonl"));
    let rpc_export_dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("outputs")
        .join("tx_exports");
    let history_bytes = history_jsonl_bytes(&records, true);
    write_file(&jsonl_path, &history_bytes).unwrap();

    exporter
        .export_with_history_bytes(
            archive_path.to_string_lossy().as_ref(),
            &password,
            &history_bytes,
        )
        .unwrap();

    assert!(archive_path.exists());
    assert!(jsonl_path.exists());
    assert_eq!(wallet_file_path.parent(), jsonl_path.parent());
    assert_ne!(archive_path, jsonl_path);
    assert_ne!(wallet_file_path, jsonl_path);
    assert!(!jsonl_path.starts_with(&rpc_export_dir));

    let jsonl = read_to_string(&jsonl_path).unwrap();
    let lines = jsonl
        .lines()
        .filter(|line| !line.trim().is_empty())
        .collect::<Vec<_>>();
    assert_eq!(lines.len(), 2);
    assert_eq!(read_file(&jsonl_path).unwrap(), history_bytes);

    for (line, expected) in lines.iter().zip(records.iter()) {
        let parsed: serde_json::Value = serde_json::from_str(line).unwrap();
        assert_eq!(parsed["schema_version"], 1);
        assert_eq!(parsed["tx_hash"], expected.tx_hash);
        assert_eq!(parsed["record"]["tx_hash"], expected.tx_hash);
        assert_eq!(parsed["record"]["status"], "Confirmed");
        assert_eq!(
            parsed["record"]["timestamp_ms"].as_u64(),
            Some(expected.timestamp_ms)
        );
        assert_eq!(
            parsed["record"]["block_height"].as_u64(),
            expected.block_height
        );
        assert_ne!(parsed["record_hash"], serde_json::Value::Null);
        assert_ne!(parsed["tx_bytes_hash"], serde_json::Value::Null);
    }

    let importer = BackupImporterImpl::new();
    let imported = importer
        .import_with_mode(
            archive_path.to_string_lossy().as_ref(),
            &password,
            ForensicImportMode::WalletPlusHistory,
        )
        .unwrap();
    assert_eq!(
        imported.transactions,
        z00z_utils::io::read_file(&jsonl_path).unwrap()
    );
    assert_eq!(imported.forensic_archive.unwrap().records, records);
}

#[test]
fn operator_jsonl_redacts_secrets() {
    let records = vec![sample_tx_record("tx-redacted")];
    let exporter = BackupExporterImpl::new_with_forensic_history(
        "wallet-1".to_string(),
        "testnet".to_string(),
        "devnet".to_string(),
        test_export_pack(),
        records.clone(),
        MockTimeProvider::new(SystemTime::UNIX_EPOCH + Duration::from_secs(1)),
        SystemRngProvider,
    );

    let password = SafePassword::from("password");
    let dir = tempfile::tempdir().unwrap();
    let archive_path = dir.path().join("wallet-1-forensic.backup");
    let wallet_stem = wallet_stem("wallet-1");
    let jsonl_path = dir
        .path()
        .join(format!("wallet_{wallet_stem}_tx_history.jsonl"));
    let history_bytes = history_jsonl_bytes(&records, false);
    write_file(&jsonl_path, &history_bytes).unwrap();

    exporter
        .export_with_history_bytes(
            archive_path.to_string_lossy().as_ref(),
            &password,
            &history_bytes,
        )
        .unwrap();

    let jsonl = read_to_string(&jsonl_path).unwrap();
    assert!(jsonl.contains("\"tx_bytes\""));
    assert!(jsonl.contains("\"record_hash\""));
    assert!(jsonl.contains("\"tx_bytes_hash\""));

    for forbidden in [
        TEST_SEED_PHRASE,
        "wallet_identity_secret_sentinel",
        "decrypted_asset_pack_sentinel",
        "asset_secret_sentinel",
        "blinding_secret_sentinel",
    ] {
        assert!(
            !jsonl.contains(forbidden),
            "operator JSONL leaked sentinel: {forbidden}"
        );
    }
}

fn sample_tx_record(tx_hash: &str) -> TxRecord {
    TxRecord {
        tx_hash: tx_hash.to_string(),
        tx_bytes: vec![1, 2, 3, 4],
        imported: false,
        status: TxStatus::Confirmed,
        timestamp_ms: 1_700_000_000,
        block_height: Some(42),
        confirmation_evidence: None,
    }
}

fn history_jsonl_bytes(records: &[TxRecord], extra_blank_line: bool) -> Vec<u8> {
    let stem = wallet_stem("wallet-1");
    let mut bytes = encode_tx_history_jsonl(&stem, records).unwrap();
    if extra_blank_line {
        bytes.push(b'\n');
    }
    bytes
}

fn wallet_stem(wallet_id: &str) -> String {
    let hash = compute_wallet_file_id(wallet_id);
    hex::encode(&hash[..8])
}
