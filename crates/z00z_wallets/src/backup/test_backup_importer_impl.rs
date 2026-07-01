use super::super::backup_wire::{
    encode_tx_history_jsonl, BackupCompression, ForensicImportMode, WalletForensicPack,
    WalletTxHistoryEntryKind, WalletTxHistoryJsonlEntry, BACKUP_SALT_BYTES,
};
use super::*;
use crate::backup::{BackupExporter, BackupExporterImpl};
use crate::db::{BackupManifestPayload, OwnedAssetPayload, WalletProfilePayload};
use crate::persistence::tx::{TxRecord, TxStatus};
use crate::rpc::types::common::PersistWalletId;
use crate::rpc::types::wallet::PersistWalletSettings;
use crate::wallet::persistence::{PasswordVerifierState, ReceiverDeriverState, WalletExportPack};
use crate::wallet::WalletState;
use z00z_utils::codec::{BincodeCodec, Codec, JsonCodec, Value};
use z00z_utils::io::{read_file, write_file};
use z00z_utils::rng::SystemRngProvider;
use z00z_utils::time::MockTimeProvider;

fn history_bytes(wallet_stem: &str, records: &[TxRecord]) -> Vec<u8> {
    encode_tx_history_jsonl(wallet_stem, records).expect("encode tx-history JSONL")
}

fn history_entry(wallet_stem: &str, record: TxRecord) -> WalletTxHistoryJsonlEntry {
    WalletTxHistoryJsonlEntry::build_event(
        wallet_stem,
        1,
        record.timestamp_ms,
        WalletTxHistoryEntryKind::Created,
        None,
        record,
    )
    .expect("build tx-history JSONL entry")
}

#[test]
fn test_read_metadata_roundtrip() {
    let dir = tempfile::tempdir().unwrap();
    let file_path = dir.path().join("backup.json");
    let path_str = file_path.to_string_lossy().to_string();

    let exporter = BackupExporterImpl::new(
        "wallet-1".to_string(),
        "testnet".to_string(),
        test_export_pack(),
        MockTimeProvider::from_unix_secs(1),
        SystemRngProvider,
    );

    let password = SafePassword::from("password");
    exporter.export(&path_str, &password).unwrap();

    let importer = BackupImporterImpl::new();
    let md = importer.read_metadata(&path_str).unwrap();
    assert_eq!(md.version, 4);
    assert_eq!(md.wallet_id, "wallet-1");
    assert!(md.network.is_empty());
}

#[test]
fn test_import_returns_metadata_fields() {
    let dir = tempfile::tempdir().unwrap();
    let file_path = dir.path().join("backup.json");
    let path_str = file_path.to_string_lossy().to_string();

    let time = MockTimeProvider::from_unix_secs(1);
    let exporter = BackupExporterImpl::new(
        "wallet-1".to_string(),
        "testnet".to_string(),
        test_export_pack(),
        time.clone(),
        SystemRngProvider,
    );

    let password = SafePassword::from("password");
    exporter.export(&path_str, &password).unwrap();

    let importer = BackupImporterImpl::new();
    let imported = importer.import(&path_str, &password).unwrap();
    assert_eq!(imported.wallet_id, "wallet-1");
    assert_eq!(imported.network, "testnet");
    assert!(imported.chain.is_empty());
    assert!(imported.export_pack.is_some());
    assert!(imported.forensic_archive.is_none());
    assert!(imported.keys.is_empty());
    assert!(imported.transactions.is_empty());
    let assets: Vec<OwnedAssetPayload> = BincodeCodec.deserialize(&imported.assets).unwrap();
    assert!(assets.is_empty());
}

#[test]
fn test_import_ignores_forensic_archive() {
    let bytes = forensic_export_bytes();
    let importer = BackupImporterImpl::new();
    let password = SafePassword::from("password");

    let imported = importer.import_from_bytes(&bytes, &password).unwrap();

    assert_eq!(imported.wallet_id, "wallet-1");
    assert_eq!(imported.network, "testnet");
    assert_eq!(imported.chain, "mainnet");
    assert!(imported.export_pack.is_some());
    assert!(imported.forensic_archive.is_none());
}

#[test]
fn test_history_returns_forensic_archive() {
    let bytes = forensic_export_bytes();
    let importer = BackupImporterImpl::new();
    let password = SafePassword::from("password");

    let imported = importer
        .import_from_bytes_with_mode(&bytes, &password, ForensicImportMode::WalletPlusHistory)
        .unwrap();

    assert!(imported.export_pack.is_some());
    let forensic = imported.forensic_archive.expect("forensic archive");
    assert_eq!(forensic.version, 1);
    assert_eq!(forensic.schema_version, 1);
    assert_eq!(forensic.records.len(), 2);
    assert_eq!(forensic.manifest.record_count, 2);
    assert_eq!(
        imported.transactions,
        forensic.history_jsonl_bytes().unwrap()
    );

    let imported_records = crate::backup::decode_tx_history_jsonl(&imported.transactions)
        .expect("decode tx-history JSONL");
    assert_eq!(imported_records.len(), 2);
    assert_eq!(imported_records[0].tx_hash, "tx-1");
    assert_eq!(imported_records[1].tx_hash, "tx-2");
}

#[test]
fn test_history_returns_transaction_blob() {
    let bytes = forensic_export_bytes();
    let importer = BackupImporterImpl::new();
    let password = SafePassword::from("password");

    let imported = importer
        .import_from_bytes_with_mode(&bytes, &password, ForensicImportMode::TxHistoryOnly)
        .unwrap();

    assert!(imported.export_pack.is_none());
    let forensic = imported.forensic_archive.expect("forensic archive");
    assert_eq!(
        imported.transactions,
        forensic.history_jsonl_bytes().unwrap()
    );

    let imported_records = crate::backup::decode_tx_history_jsonl(&imported.transactions)
        .expect("decode tx-history JSONL");
    assert_eq!(imported_records.len(), 2);
    assert_eq!(imported_records[0].tx_hash, "tx-1");
    assert_eq!(imported_records[1].tx_hash, "tx-2");
}

#[test]
fn test_tx_history_forensic_archive() {
    let exporter = BackupExporterImpl::new_with_chain(
        "wallet-1".to_string(),
        "testnet".to_string(),
        "mainnet".to_string(),
        test_export_pack(),
        MockTimeProvider::from_unix_secs(1),
        SystemRngProvider,
    );

    let password = SafePassword::from("password");
    let bytes = exporter.export_to_bytes(&password).unwrap();

    let importer = BackupImporterImpl::new();
    let err = importer
        .import_from_bytes_with_mode(&bytes, &password, ForensicImportMode::TxHistoryOnly)
        .unwrap_err();

    assert!(
        matches!(err, BackupImporterError::InvalidFormat(message) if message.contains("forensic archive section is required"))
    );
}

#[test]
fn jsonl_replay_preserves_view() {
    let dir = tempfile::tempdir().unwrap();
    let history_path = dir.path().join("wallet_history.jsonl");
    let records = vec![
        sample_tx_record("tx-1", vec![1, 2, 3, 4]),
        sample_tx_record("tx-2", vec![5, 6, 7, 8]),
    ];

    let bytes = history_bytes("history", &records);
    write_file(history_path.to_string_lossy().as_ref(), &bytes).unwrap();

    let imported =
        BackupImporterImpl::import_history_jsonl(history_path.to_string_lossy().as_ref()).unwrap();
    assert_eq!(imported, records);
    assert_eq!(imported[0].tx_hash, "tx-1");
    assert_eq!(imported[0].tx_bytes, vec![1, 2, 3, 4]);
    assert_eq!(imported[0].status, TxStatus::Confirmed);
    assert_eq!(imported[0].timestamp_ms, 1_700_000_000);
    assert_eq!(imported[0].block_height, Some(42));
}

#[test]
fn jsonl_replay_rejects_bad() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("wallet_history.jsonl");
    let record = sample_tx_record("tx-1", vec![1, 2, 3, 4]);
    let entry = history_entry("history", record.clone());

    let mut tampered_record = entry.clone();
    tampered_record.record_hash[0] ^= 0x01;
    assert_jsonl_rejected(&path, &[tampered_record], "record hash mismatch");

    let mut tampered_bytes = entry.clone();
    tampered_bytes.tx_bytes_hash[0] ^= 0x01;
    assert_jsonl_rejected(&path, &[tampered_bytes], "tx_bytes hash mismatch");

    let mut mismatched_label = entry.clone();
    mismatched_label.tx_hash = "tx-2".to_string();
    assert_jsonl_rejected(&path, &[mismatched_label], "tx hash label mismatch");

    assert_jsonl_rejected(
        &path,
        &[entry.clone(), entry.clone()],
        "tx-history sequence mismatch",
    );

    write_file(path.to_string_lossy().as_ref(), b"{not json}\n").unwrap();
    let err =
        BackupImporterImpl::import_history_jsonl(path.to_string_lossy().as_ref()).unwrap_err();
    assert!(matches!(err, BackupImporterError::InvalidFormat(_)));

    let mut missing_field: Value = JsonCodec
        .deserialize(&JsonCodec.serialize(&entry).unwrap())
        .unwrap();
    missing_field.as_object_mut().unwrap().remove("record_hash");
    let missing_field_bytes = JsonCodec.serialize(&missing_field).unwrap();
    write_file(
        path.to_string_lossy().as_ref(),
        missing_field_bytes.as_slice(),
    )
    .unwrap();
    let err =
        BackupImporterImpl::import_history_jsonl(path.to_string_lossy().as_ref()).unwrap_err();
    assert!(matches!(err, BackupImporterError::InvalidFormat(_)));
}

#[test]
fn test_rejects_tamper_record_hash() {
    let metadata = BackupMetadata {
        version: 4,
        created_at: 1,
        wallet_id: "wallet-1".to_string(),
        network: "testnet".to_string(),
    };
    let records = vec![
        sample_tx_record("tx-1", vec![1, 2, 3, 4]),
        sample_tx_record("tx-2", vec![5, 6, 7, 8]),
    ];
    let history_jsonl = history_bytes("wallet-1", &records);
    let forensic = WalletForensicPack::build_with_history_jsonl(
        metadata.clone(),
        "testnet".to_string(),
        "mainnet".to_string(),
        records,
        history_jsonl,
    )
    .unwrap();

    forensic
        .validate(&metadata, "testnet", "mainnet")
        .expect("forensic archive");

    let mut tampered = forensic.clone();
    tampered.manifest.entries[0].tx_bytes_hash[0] ^= 0x01;

    let err = tampered
        .validate(&metadata, "testnet", "mainnet")
        .unwrap_err();

    assert!(
        err.contains("tx_bytes hash mismatch"),
        "unexpected error: {err}"
    );
}

fn assert_jsonl_rejected(
    path: &std::path::Path,
    entries: &[WalletTxHistoryJsonlEntry],
    expected: &str,
) {
    let mut bytes = Vec::new();
    for entry in entries {
        bytes.extend_from_slice(&JsonCodec.serialize(entry).unwrap());
        bytes.push(b'\n');
    }
    write_file(path.to_string_lossy().as_ref(), &bytes).unwrap();

    let err =
        BackupImporterImpl::import_history_jsonl(path.to_string_lossy().as_ref()).unwrap_err();
    assert!(
        matches!(err, BackupImporterError::InvalidFormat(message) if message.contains(expected))
    );
}

#[test]
fn test_import_rejects_kdf_contract() {
    let bytes = build_string_kdf_contract_bytes();
    let importer = BackupImporterImpl::new();
    let password = SafePassword::from("password");

    let err = importer.import_from_bytes(&bytes, &password).unwrap_err();
    assert!(matches!(err, BackupImporterError::Deserialization(_)));
}

#[test]
fn test_import_rejects_backup_versions() {
    let exporter = BackupExporterImpl::new_with_chain(
        "wallet-1".to_string(),
        "testnet".to_string(),
        "mainnet".to_string(),
        test_export_pack(),
        MockTimeProvider::from_unix_secs(1),
        SystemRngProvider,
    );

    let password = SafePassword::from("password");
    let bytes = exporter.export_to_bytes(&password).unwrap();
    let mut decoded: BackupContainer = JsonCodec.deserialize(&bytes).unwrap();
    let importer = BackupImporterImpl::new();

    for version in [1u32, 2u32, 3u32] {
        decoded.metadata.version = version;
        let mutated = JsonCodec.serialize(&decoded).unwrap();
        let err = importer.import_from_bytes(&mutated, &password).unwrap_err();
        assert!(matches!(
            err,
            BackupImporterError::VersionMismatch {
                expected,
                found,
            } if expected == BACKUP_FORMAT_VERSION && found == version
        ));
    }
}

#[test]
fn test_verify_password_true_decodable() {
    let dir = tempfile::tempdir().unwrap();
    let file_path = dir.path().join("backup.json");
    let path_str = file_path.to_string_lossy().to_string();

    let exporter = BackupExporterImpl::new(
        "wallet-1".to_string(),
        "testnet".to_string(),
        test_export_pack(),
        MockTimeProvider::from_unix_secs(1),
        SystemRngProvider,
    );

    let password = SafePassword::from("password");
    exporter.export(&path_str, &password).unwrap();

    let importer = BackupImporterImpl::new();
    assert!(importer.verify_password(&path_str, &password).unwrap());
}

#[test]
fn test_verify_password_wrong_password() {
    let dir = tempfile::tempdir().unwrap();
    let file_path = dir.path().join("backup.json");
    let path_str = file_path.to_string_lossy().to_string();

    let exporter = BackupExporterImpl::new(
        "wallet-1".to_string(),
        "testnet".to_string(),
        test_export_pack(),
        MockTimeProvider::from_unix_secs(1),
        SystemRngProvider,
    );

    exporter
        .export(&path_str, &SafePassword::from("password"))
        .unwrap();

    let importer = BackupImporterImpl::new();
    assert!(!importer
        .verify_password(&path_str, &SafePassword::from("wrong-password"))
        .unwrap());
}

#[test]
fn test_import_rejects_corrupted_checksum() {
    let exporter = BackupExporterImpl::new(
        "wallet-1".to_string(),
        "testnet".to_string(),
        test_export_pack(),
        MockTimeProvider::from_unix_secs(1),
        SystemRngProvider,
    );

    let password = SafePassword::from("password");
    let bytes = exporter.export_to_bytes(&password).unwrap();

    let mut decoded: BackupContainer = JsonCodec.deserialize(&bytes).unwrap();
    decoded.ciphertext[0] ^= 0x01;
    let corrupted = JsonCodec.serialize(&decoded).unwrap();

    let importer = BackupImporterImpl::new();
    let err = importer
        .import_from_bytes(&corrupted, &password)
        .unwrap_err();
    assert!(matches!(err, BackupImporterError::IntegrityMismatch));
}

#[test]
fn test_verify_password_corrupted_backup() {
    let exporter = BackupExporterImpl::new(
        "wallet-1".to_string(),
        "testnet".to_string(),
        test_export_pack(),
        MockTimeProvider::from_unix_secs(1),
        SystemRngProvider,
    );

    let password = SafePassword::from("password");
    let bytes = exporter.export_to_bytes(&password).unwrap();

    let mut decoded: BackupContainer = JsonCodec.deserialize(&bytes).unwrap();
    decoded.ciphertext[0] ^= 0x01;
    let corrupted = JsonCodec.serialize(&decoded).unwrap();

    let dir = tempfile::tempdir().unwrap();
    let file_path = dir.path().join("backup.json");
    let path_str = file_path.to_string_lossy().to_string();
    write_file(&path_str, &corrupted).unwrap();

    let importer = BackupImporterImpl::new();
    assert!(!importer.verify_password(&path_str, &password).unwrap());
}

#[test]
fn test_import_roundtrip_preserves_chain() {
    let dir = tempfile::tempdir().unwrap();
    let file_path = dir.path().join("backup.json");
    let path_str = file_path.to_string_lossy().to_string();

    let exporter = BackupExporterImpl::new_with_chain(
        "wallet-1".to_string(),
        "testnet".to_string(),
        "mainnet".to_string(),
        test_export_pack(),
        MockTimeProvider::from_unix_secs(1),
        SystemRngProvider,
    );

    let password = SafePassword::from("password");
    exporter.export(&path_str, &password).unwrap();

    let importer = BackupImporterImpl::new();
    let imported = importer.import(&path_str, &password).unwrap();
    assert_eq!(imported.chain, "mainnet");
}

#[test]
fn test_rejects_invalid_profile_checksum() {
    let mut export_pack = test_export_pack();
    export_pack
        .wallet_profile
        .as_mut()
        .expect("wallet profile")
        .checksum = Some([9u8; 32]);

    let exporter = BackupExporterImpl::new(
        "wallet-1".to_string(),
        "testnet".to_string(),
        export_pack,
        MockTimeProvider::from_unix_secs(1),
        SystemRngProvider,
    );

    let password = SafePassword::from("password");
    let bytes = exporter.export_to_bytes(&password).unwrap();

    let importer = BackupImporterImpl::new();
    let err = importer.import_from_bytes(&bytes, &password).unwrap_err();
    assert!(matches!(err, BackupImporterError::InvalidFormat(_)));
}

#[test]
fn test_verify_invalid_profile_checksum() {
    let mut export_pack = test_export_pack();
    export_pack
        .wallet_profile
        .as_mut()
        .expect("wallet profile")
        .checksum = Some([9u8; 32]);

    let exporter = BackupExporterImpl::new(
        "wallet-1".to_string(),
        "testnet".to_string(),
        export_pack,
        MockTimeProvider::from_unix_secs(1),
        SystemRngProvider,
    );

    let dir = tempfile::tempdir().unwrap();
    let file_path = dir.path().join("backup.json");
    let path_str = file_path.to_string_lossy().to_string();
    let password = SafePassword::from("password");

    exporter.export(&path_str, &password).unwrap();

    let importer = BackupImporterImpl::new();
    assert!(!importer.verify_password(&path_str, &password).unwrap());
}

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
        chain: "mainnet".to_string(),
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

fn forensic_export_bytes() -> Vec<u8> {
    let dir = tempfile::tempdir().unwrap();
    let archive_path = dir.path().join("backup.json");
    let records = vec![
        sample_tx_record("tx-1", vec![1, 2, 3, 4]),
        sample_tx_record("tx-2", vec![5, 6, 7, 8]),
    ];
    let exporter = BackupExporterImpl::new_with_forensic_history(
        "wallet-1".to_string(),
        "testnet".to_string(),
        "mainnet".to_string(),
        test_export_pack(),
        records.clone(),
        MockTimeProvider::from_unix_secs(1),
        SystemRngProvider,
    );

    let history_bytes = history_bytes("1", &records);

    exporter
        .export_with_history_bytes(
            archive_path.to_string_lossy().as_ref(),
            &SafePassword::from("password"),
            &history_bytes,
        )
        .expect("forensic backup bytes");

    read_file(&archive_path).unwrap()
}

fn sample_tx_record(tx_hash: &str, tx_bytes: Vec<u8>) -> TxRecord {
    TxRecord {
        tx_hash: tx_hash.to_string(),
        tx_bytes,
        imported: false,
        status: TxStatus::Confirmed,
        timestamp_ms: 1_700_000_000,
        block_height: Some(42),
        confirmation_evidence: None,
    }
}

fn build_string_kdf_contract_bytes() -> Vec<u8> {
    let metadata = BackupMetadata {
        version: 1,
        created_at: 1,
        wallet_id: "wallet-1".to_string(),
        network: "testnet".to_string(),
    };
    let encryption = BackupEncryption {
        algorithm: "xchacha20poly1305".to_string(),
        kdf: BackupKdf::default([7u8; BACKUP_SALT_BYTES]),
        salt: Some([7u8; BACKUP_SALT_BYTES]),
        nonce: [0u8; BACKUP_NONCE_BYTES],
    };
    let compression = BackupCompression {
        algorithm: "zstd".to_string(),
    };
    let checksum = [0u8; 32];
    let ciphertext = vec![0u8; 8];

    let container = BackupContainer {
        metadata,
        encryption,
        compression,
        checksum,
        ciphertext,
    };
    let mut json: Value = JsonCodec
        .deserialize(&JsonCodec.serialize(&container).unwrap())
        .unwrap();
    json["encryption"]["kdf"] = Value::from("argon2id");
    JsonCodec.serialize(&json).unwrap()
}
