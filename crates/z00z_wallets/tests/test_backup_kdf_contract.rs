#![cfg(not(target_arch = "wasm32"))]

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::time::{Duration, SystemTime};
use z00z_crypto::expert::encoding::SafePassword;
use z00z_utils::{rng::SystemRngProvider, time::MockTimeProvider};
use z00z_wallets::{
    backup::{BackupExporter, BackupExporterImpl, BackupImporter, BackupImporterImpl},
    db::{BackupManifestPayload, WalletProfilePayload},
    rpc::types::{common::PersistWalletId, wallet::PersistWalletSettings},
    wallet::{
        persistence::{PasswordVerifierState, ReceiverDeriverState, WalletExportPack},
        WalletState,
    },
};

const TEST_SEED: &str = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon art";

#[cfg(feature = "test-params-fast")]
const EXPECT_MEM: u64 = 16 * 1024 * 1024;
#[cfg(not(feature = "test-params-fast"))]
const EXPECT_MEM: u64 = 128 * 1024 * 1024;

#[cfg(feature = "test-params-fast")]
const EXPECT_OPS: u32 = 1;
#[cfg(not(feature = "test-params-fast"))]
const EXPECT_OPS: u32 = 5;

#[cfg(feature = "test-params-fast")]
const EXPECT_PAR: u32 = 2;
#[cfg(not(feature = "test-params-fast"))]
const EXPECT_PAR: u32 = 8;

#[derive(Debug, Clone, Serialize, Deserialize)]
struct InvalidMd {
    version: u32,
    created_at: u64,
    wallet_id: String,
    network: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct InvalidEnc {
    algorithm: String,
    kdf: String,
    salt: [u8; 16],
    nonce: [u8; 24],
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct InvalidComp {
    algorithm: String,
}

fn test_pack() -> WalletExportPack {
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
        [4u8; 16],
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
        seed_phrase: TEST_SEED.to_string(),
        wallet_identity: None,
    }
}

fn exporter() -> BackupExporterImpl<MockTimeProvider, SystemRngProvider> {
    BackupExporterImpl::new_with_chain(
        "wallet-1".to_string(),
        "testnet".to_string(),
        "devnet".to_string(),
        test_pack(),
        MockTimeProvider::new(SystemTime::UNIX_EPOCH + Duration::from_secs(1)),
        SystemRngProvider,
    )
}

fn string_kdf_contract_bytes() -> Vec<u8> {
    let metadata = InvalidMd {
        version: 1,
        created_at: 1,
        wallet_id: "wallet-1".to_string(),
        network: "testnet".to_string(),
    };
    let encryption = InvalidEnc {
        algorithm: "xchacha20poly1305".to_string(),
        kdf: "argon2id".to_string(),
        salt: [7u8; 16],
        nonce: [0u8; 24],
    };
    let compression = InvalidComp {
        algorithm: "zstd".to_string(),
    };
    let checksum = [0u8; 32];
    let ciphertext = vec![0u8; 8];

    serde_json::to_vec(&json!({
        "metadata": metadata,
        "encryption": encryption,
        "compression": compression,
        "checksum": checksum,
        "ciphertext": ciphertext,
    }))
    .unwrap()
}

#[test]
fn test_export_persists_backup_kdf() {
    let bytes = exporter()
        .export_to_bytes(&SafePassword::from("password"))
        .unwrap();
    let json: Value = serde_json::from_slice(&bytes).unwrap();
    let kdf = &json["encryption"]["kdf"];

    assert!(kdf.is_object(), "new backups must persist a kdf object");
    assert_eq!(kdf["version"], 2);
    assert_eq!(kdf["algorithm"], "argon2id");
    assert!(kdf["salt"].is_array());
    assert_eq!(kdf["mem_limit"], EXPECT_MEM);
    assert_eq!(kdf["ops_limit"], EXPECT_OPS);
    assert_eq!(kdf["parallelism"], EXPECT_PAR);
    assert_eq!(kdf["salt_pad"], "zero_pad_to32");
    assert!(json["encryption"].get("salt").is_none());
}

#[test]
fn test_rejects_unknown_backup_kdf() {
    let password = SafePassword::from("password");
    let mut json: Value =
        serde_json::from_slice(&exporter().export_to_bytes(&password).unwrap()).unwrap();
    json["encryption"]["kdf"]["version"] = Value::from(99);

    let importer = BackupImporterImpl::new();
    let err = importer.import_from_bytes(&serde_json::to_vec(&json).unwrap(), &password);
    assert!(matches!(
        err,
        Err(z00z_wallets::backup::BackupImporterError::InvalidFormat(_))
    ));
}

#[test]
fn test_import_rejects_kdf_contract() {
    let importer = BackupImporterImpl::new();
    let err = importer
        .import_from_bytes(
            &string_kdf_contract_bytes(),
            &SafePassword::from("password"),
        )
        .unwrap_err();

    assert!(matches!(
        err,
        z00z_wallets::backup::BackupImporterError::Deserialization(_)
    ));
}

#[test]
fn test_rejects_prior_backup_versions() {
    let password = SafePassword::from("password");
    let bytes = exporter().export_to_bytes(&password).unwrap();
    let mut json: Value = serde_json::from_slice(&bytes).unwrap();
    let importer = BackupImporterImpl::new();

    for version in [1u32, 2u32, 3u32] {
        json["metadata"]["version"] = Value::from(version);
        let err = importer
            .import_from_bytes(&serde_json::to_vec(&json).unwrap(), &password)
            .unwrap_err();
        assert!(matches!(
            err,
            z00z_wallets::backup::BackupImporterError::VersionMismatch {
                expected: 4,
                found,
            } if found == version
        ));
    }
}
