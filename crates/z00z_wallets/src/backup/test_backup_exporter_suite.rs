use super::*;
use crate::db::{BackupManifestPayload, WalletProfilePayload};
use crate::rpc::types::common::PersistWalletId;
use crate::rpc::types::wallet::PersistWalletSettings;
use crate::wallet::persistence::{PasswordVerifierState, ReceiverDeriverState, WalletExportPack};
use crate::wallet::WalletState;
use z00z_utils::compression::zstd_decompress_bounded;
use z00z_utils::rng::SystemRngProvider;
use z00z_utils::time::MockTimeProvider;

#[test]
fn test_get_metadata_expected_fields() {
    let exporter = BackupExporterImpl::new(
        "wallet-1".to_string(),
        "testnet".to_string(),
        test_export_pack(),
        MockTimeProvider::from_unix_secs(1),
        SystemRngProvider,
    );

    let md = exporter.get_metadata().unwrap();
    assert_eq!(md.version, 4);
    assert_eq!(md.wallet_id, "wallet-1");
    assert!(md.network.is_empty());
}

#[test]
fn test_export_verify_backup_roundtrip() {
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

    let ok = exporter.verify_backup(&path_str, &password).unwrap();
    assert!(ok);
}

#[test]
fn test_verify_backup_wrong_wallet() {
    let dir = tempfile::tempdir().unwrap();
    let file_path = dir.path().join("backup.json");
    let path_str = file_path.to_string_lossy().to_string();

    let exporter_a = BackupExporterImpl::new(
        "wallet-a".to_string(),
        "testnet".to_string(),
        test_export_pack_with_id("wallet-a"),
        MockTimeProvider::from_unix_secs(1),
        SystemRngProvider,
    );

    let password = SafePassword::from("password");
    exporter_a.export(&path_str, &password).unwrap();

    let exporter_b = BackupExporterImpl::new(
        "wallet-b".to_string(),
        "testnet".to_string(),
        test_export_pack_with_id("wallet-b"),
        MockTimeProvider::from_unix_secs(1),
        SystemRngProvider,
    );

    let ok = exporter_b.verify_backup(&path_str, &password).unwrap();
    assert!(!ok);
}

#[test]
fn test_verify_password_returns_false() {
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

    let ok = exporter
        .verify_backup(&path_str, &SafePassword::from("wrong-password"))
        .unwrap();
    assert!(!ok);
}

#[test]
fn test_verify_backup_detects_tamper() {
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
    let bytes = exporter.export_to_bytes(&password).unwrap();

    let codec = JsonCodec;
    let mut decoded: BackupContainer = codec.deserialize(&bytes).unwrap();
    let last = decoded
        .ciphertext
        .last_mut()
        .expect("ciphertext must not be empty");
    *last ^= 0x01;

    let aad = BackupAssociatedData {
        metadata: decoded.metadata.clone(),
        encryption: decoded.encryption.clone(),
        compression: decoded.compression.clone(),
    };
    let aad_bytes =
        BackupExporterImpl::<MockTimeProvider, SystemRngProvider>::build_aad_bytes(&aad).unwrap();
    decoded.checksum = BackupExporterImpl::<MockTimeProvider, SystemRngProvider>::compute_checksum(
        &aad_bytes,
        &decoded.ciphertext,
    );

    write_file(&path_str, &codec.serialize(&decoded).unwrap()).unwrap();

    let ok = exporter.verify_backup(&path_str, &password).unwrap();
    assert!(!ok);
}

#[test]
fn test_export_to_bytes_roundtrip() {
    let exporter = BackupExporterImpl::new(
        "wallet-1".to_string(),
        "testnet".to_string(),
        test_export_pack(),
        MockTimeProvider::from_unix_secs(1),
        SystemRngProvider,
    );

    let password = SafePassword::from("password");
    let bytes = exporter.export_to_bytes(&password).unwrap();
    let decoded: BackupContainer = JsonCodec.deserialize(&bytes).unwrap();

    assert_eq!(decoded.metadata.version, 4);
    assert_eq!(decoded.metadata.wallet_id, "wallet-1");
}

#[test]
fn test_uses_nonce_stripped_checksum() {
    let exporter = BackupExporterImpl::new(
        "wallet-1".to_string(),
        "testnet".to_string(),
        test_export_pack(),
        MockTimeProvider::from_unix_secs(1),
        SystemRngProvider,
    );

    let container = exporter
        .export_to_container(&SafePassword::from("password"))
        .unwrap();

    let aad = BackupAssociatedData {
        metadata: container.metadata.clone(),
        encryption: container.encryption.clone(),
        compression: container.compression.clone(),
    };

    let expected =
        BackupExporterImpl::<MockTimeProvider, SystemRngProvider>::build_aad_bytes(&aad).unwrap();

    let mut nonce_variant = aad.clone();
    nonce_variant.encryption.nonce = [0x55u8; BACKUP_NONCE_BYTES];
    let variant =
        BackupExporterImpl::<MockTimeProvider, SystemRngProvider>::build_aad_bytes(&nonce_variant)
            .unwrap();

    assert_eq!(
        container.checksum,
        WalletBackupCrypto::checksum(&expected, &container.ciphertext)
    );
    assert_eq!(expected, variant);
}

#[test]
fn test_container_roundtrip_decrypts_payload() {
    let exporter = BackupExporterImpl::new_with_chain(
        "wallet-1".to_string(),
        "testnet".to_string(),
        "mainnet".to_string(),
        test_export_pack(),
        MockTimeProvider::from_unix_secs(1),
        SystemRngProvider,
    );

    let password = SafePassword::from("password");
    let container = exporter.export_to_container(&password).unwrap();
    let plaintext = BackupExporterImpl::<MockTimeProvider, SystemRngProvider>::decrypt_payload(
        &container, &password,
    )
    .unwrap()
    .unwrap();

    let payload: BackupPayload = JsonCodec.deserialize(&plaintext).unwrap();
    assert_eq!(payload.network, "testnet");
    assert_eq!(payload.chain, "mainnet");
    assert_eq!(
        payload
            .export_pack
            .wallet_profile
            .as_ref()
            .expect("explicit wallet profile")
            .wallet_id
            .0,
        "wallet-1"
    );
}

#[test]
fn test_rejects_mismatched_wallet_id() {
    let exporter = BackupExporterImpl::new(
        "wallet-a".to_string(),
        "testnet".to_string(),
        test_export_pack_with_id("wallet-b"),
        MockTimeProvider::from_unix_secs(1),
        SystemRngProvider,
    );

    let err = exporter
        .export_to_container(&SafePassword::from("password"))
        .unwrap_err();
    assert!(matches!(err, BackupExporterError::ExportFailed(_)));
}

#[test]
fn test_bytes_decodes_zstd_payload() {
    let exporter = BackupExporterImpl::new(
        "wallet-1".to_string(),
        "testnet".to_string(),
        test_export_pack(),
        MockTimeProvider::from_unix_secs(1),
        SystemRngProvider,
    );

    let bytes = exporter
        .export_to_bytes(&SafePassword::from("password"))
        .unwrap();
    let container: BackupContainer = JsonCodec.deserialize(&bytes).unwrap();

    let aad =
        BackupExporterImpl::<MockTimeProvider, SystemRngProvider>::resolve_aad_bytes(&container)
            .unwrap()
            .unwrap();
    let kdf = BackupExporterImpl::<MockTimeProvider, SystemRngProvider>::resolve_kdf(
        &container.encryption,
    )
    .unwrap();
    let key =
        WalletBackupCrypto::derive_key_with_kdf(&SafePassword::from("password"), &kdf).unwrap();
    let compressed = WalletBackupCrypto::decrypt(&key, &aad, &container.ciphertext).unwrap();
    let plain = zstd_decompress_bounded(&compressed, BACKUP_MAX_PLAINTEXT_BYTES).unwrap();
    let payload: BackupPayload = JsonCodec.deserialize(&plain).unwrap();

    assert_eq!(
        payload
            .export_pack
            .wallet_profile
            .as_ref()
            .expect("explicit wallet profile")
            .wallet_id
            .0,
        "wallet-1"
    );
}

fn test_export_pack() -> WalletExportPack {
    test_export_pack_with_id("wallet-1")
}

fn test_export_pack_with_id(wallet_id: &str) -> WalletExportPack {
    const TEST_SEED_PHRASE: &str = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon art";

    let profile = WalletProfilePayload::new_with_checksum(
        PersistWalletId(wallet_id.to_string()),
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
        [5u8; 16],
        WalletState::Locked,
    );
    let mut manifest = BackupManifestPayload {
        version: BackupManifestPayload::VERSION,
        wallet_id: PersistWalletId(wallet_id.to_string()),
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
        seed_phrase: TEST_SEED_PHRASE.to_string(),
        wallet_identity: None,
    }
}
