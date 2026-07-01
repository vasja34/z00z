//! Integration tests for RPC types serialization
//!
//! These tests verify that all RPC types can be properly serialized and deserialized
//! through the JSON-RPC layer, ensuring end-to-end compatibility.

#![cfg(not(target_arch = "wasm32"))]

use z00z_utils::codec::{Codec, JsonCodec};
use z00z_wallets::rpc::types::{
    app::{RuntimeLogEntry, RuntimeViewLogsResponse},
    backup::{
        PersistBackupInfo, RuntimeBackupSettingsResponse, RuntimeCreateBackupResponse,
        RuntimeListBackupsResponse, RuntimeRestoreBackupResponse,
    },
    chain::{
        RuntimeBlockInfo, RuntimeReceiveScanOutcome, RuntimeScanState, RuntimeScanStatus,
        RuntimeStartScanParams, RuntimeStartScanResponse,
    },
    common::{PersistWalletId, RuntimeOperationStatus},
    key::{
        PersistReceiverInfo, RuntimeDeriveReceiverResponse, RuntimeExportPublicKeyResponse,
        RuntimeKeyDeriveResponse, RuntimeLabelReceiverResponse, RuntimeListReceiversResponse,
        RuntimePubMaterialExportResponse, RuntimeReceiverFilter, RuntimeRotateKeyResponse,
    },
    network::{RuntimeChainSettings, RuntimeChainSettingsResponse, RuntimeSwitchChainResponse},
    security::{AuditResult, PersistAuditLogEntry, RiskLevel},
    tx::{PersistReceiptInfo, PersistTxId, RuntimeConfirmationReceipt},
};

#[tokio::test]
async fn test_app_log_types_roundtrip() {
    let codec = JsonCodec;

    // Test log entry
    let entry = RuntimeLogEntry {
        timestamp: 1_700_000_000_000,
        level: "INFO".to_string(),
        message: "Test message".to_string(),
    };
    let bytes = codec.serialize(&entry).unwrap();
    let deserialized: RuntimeLogEntry = codec.deserialize(&bytes).unwrap();
    assert_eq!(deserialized.timestamp, entry.timestamp);
    assert_eq!(deserialized.level, entry.level);
    assert_eq!(deserialized.message, entry.message);

    // Test view logs response
    let response = RuntimeViewLogsResponse {
        logs: vec![entry.clone()],
    };
    let bytes = codec.serialize(&response).unwrap();
    let deserialized: RuntimeViewLogsResponse = codec.deserialize(&bytes).unwrap();
    assert_eq!(deserialized.logs.len(), 1);
    assert_eq!(deserialized.logs[0].timestamp, 1_700_000_000_000);
}

#[tokio::test]
async fn test_backup_types_roundtrip() {
    let codec = JsonCodec;

    // Test create backup response
    let create = RuntimeCreateBackupResponse {
        status: RuntimeOperationStatus {
            success: true,
            message: "Backup created".to_string(),
        },
        backup_path: "/path/to/backup".to_string(),
        encrypted: true,
    };
    let bytes = codec.serialize(&create).unwrap();
    let deserialized: RuntimeCreateBackupResponse = codec.deserialize(&bytes).unwrap();
    assert!(deserialized.status.success);
    assert_eq!(deserialized.backup_path, "/path/to/backup");

    // Test list backups response
    let info = PersistBackupInfo {
        id: "backup-001".to_string(),
        wallet_id: PersistWalletId("wallet-123".to_string()),
        created_at: 1_700_000_000_000,
        size_bytes: 1024,
        encrypted: true,
    };
    let list = RuntimeListBackupsResponse {
        items: vec![info],
        next_cursor: None,
        has_more: false,
        total_count: Some(1),
    };
    let bytes = codec.serialize(&list).unwrap();
    let deserialized: RuntimeListBackupsResponse = codec.deserialize(&bytes).unwrap();
    assert_eq!(deserialized.items.len(), 1);

    // Test restore backup response
    let restore = RuntimeRestoreBackupResponse {
        status: RuntimeOperationStatus {
            success: true,
            message: "Restored".to_string(),
        },
        wallet_id: PersistWalletId("wallet-restored".to_string()),
    };
    let bytes = codec.serialize(&restore).unwrap();
    let deserialized: RuntimeRestoreBackupResponse = codec.deserialize(&bytes).unwrap();
    assert!(deserialized.status.success);

    // Test backup settings response
    use z00z_wallets::rpc::types::backup::PersistBackupSettings;
    let settings = RuntimeBackupSettingsResponse {
        settings: PersistBackupSettings {
            auto_backup_enabled: true,
            backup_interval_hours: 24,
            backup_location: "/backups".to_string(),
            encrypt_backups: true,
        },
    };
    let bytes = codec.serialize(&settings).unwrap();
    let deserialized: RuntimeBackupSettingsResponse = codec.deserialize(&bytes).unwrap();
    assert!(deserialized.settings.auto_backup_enabled);

    let settings_json: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    let settings_object = settings_json["settings"]
        .as_object()
        .expect("backup settings object");
    assert_eq!(settings_object.len(), 4);
    for key in settings_object.keys() {
        assert!(
            !key.to_ascii_lowercase().contains("forensic"),
            "PersistBackupSettings must not persist a forensic toggle: {key}"
        );
        assert!(
            !key.to_ascii_lowercase().contains("history"),
            "PersistBackupSettings must not silently enable tx-history import/export: {key}"
        );
    }
}

#[tokio::test]
async fn test_key_types_roundtrip() {
    let codec = JsonCodec;

    // Test key derive responses
    let key_derive = RuntimeKeyDeriveResponse {
        public_key: "0xabc123".to_string(),
    };
    let bytes = codec.serialize(&key_derive).unwrap();
    let deserialized: RuntimeKeyDeriveResponse = codec.deserialize(&bytes).unwrap();
    assert_eq!(deserialized.public_key, "0xabc123");

    let export_key = RuntimeExportPublicKeyResponse {
        public_key: "0xdef456".to_string(),
    };
    let bytes = codec.serialize(&export_key).unwrap();
    let deserialized: RuntimeExportPublicKeyResponse = codec.deserialize(&bytes).unwrap();
    assert_eq!(deserialized.public_key, "0xdef456");

    // Test derive key response
    let derive = RuntimeDeriveReceiverResponse {
        public_key: "0x123".to_string(),
        path: "m/44'/0'/0'/0/0".to_string(),
    };
    let bytes = codec.serialize(&derive).unwrap();
    let deserialized: RuntimeDeriveReceiverResponse = codec.deserialize(&bytes).unwrap();
    assert_eq!(deserialized.path, "m/44'/0'/0'/0/0");

    // Test pub material export
    let pub_material = RuntimePubMaterialExportResponse {
        schema_version: 1,
        encrypted_pub_material: "base64data".to_string(),
        algorithm: "XChaCha20-Poly1305".to_string(),
        account: 0,
        fingerprint: "fingerprint123".to_string(),
    };
    let bytes = codec.serialize(&pub_material).unwrap();
    let deserialized: RuntimePubMaterialExportResponse = codec.deserialize(&bytes).unwrap();
    assert_eq!(deserialized.schema_version, 1);

    // Test rotate key response
    let rotate = RuntimeRotateKeyResponse {
        new_fingerprint: "new_fp".to_string(),
        rotated_at: 1_700_000_000_000,
        records_rewrapped: 42,
    };
    let bytes = codec.serialize(&rotate).unwrap();
    let deserialized: RuntimeRotateKeyResponse = codec.deserialize(&bytes).unwrap();
    assert_eq!(deserialized.records_rewrapped, 42);

    // Test address filter
    let filter = RuntimeReceiverFilter {
        used: Some(true),
        change: Some(false),
    };
    let bytes = codec.serialize(&filter).unwrap();
    let deserialized: RuntimeReceiverFilter = codec.deserialize(&bytes).unwrap();
    assert_eq!(deserialized.used, Some(true));

    let receivers = RuntimeListReceiversResponse {
        items: vec![PersistReceiverInfo {
            receiver_id: "aa".repeat(32),
            path: "m/44'/1337'/0'/0/0".to_string(),
            public_key: "bb".repeat(32),
            balance: None,
            used: false,
            internal: false,
            label: Some("Primary".to_string()),
            index: 0,
        }],
        next_cursor: None,
        has_more: false,
        total_count: Some(1),
    };
    let bytes = codec.serialize(&receivers).unwrap();
    let deserialized: RuntimeListReceiversResponse = codec.deserialize(&bytes).unwrap();
    assert_eq!(deserialized.items.len(), 1);
    assert_eq!(deserialized.items[0].receiver_id, "aa".repeat(32));

    let label = RuntimeLabelReceiverResponse {
        receiver_id: "cc".repeat(32),
        label: "Receiver Label".to_string(),
        status: RuntimeOperationStatus {
            success: true,
            message: "ok".to_string(),
        },
    };
    let bytes = codec.serialize(&label).unwrap();
    let deserialized: RuntimeLabelReceiverResponse = codec.deserialize(&bytes).unwrap();
    assert_eq!(deserialized.receiver_id, "cc".repeat(32));
}

#[tokio::test]
async fn test_network_types_roundtrip() {
    let codec = JsonCodec;

    // Test chain settings
    use z00z_core::ChainType;
    let settings = RuntimeChainSettings {
        chain_type: ChainType::Mainnet,
        rpc_endpoint: "https://mainnet.example.com".to_string(),
        use_tor: false,
    };
    let bytes = codec.serialize(&settings).unwrap();
    let deserialized: RuntimeChainSettings = codec.deserialize(&bytes).unwrap();
    assert_eq!(deserialized.chain_type, ChainType::Mainnet);

    // Test chain settings response
    let response = RuntimeChainSettingsResponse { settings };
    let bytes = codec.serialize(&response).unwrap();
    let deserialized: RuntimeChainSettingsResponse = codec.deserialize(&bytes).unwrap();
    assert_eq!(
        deserialized.settings.rpc_endpoint,
        "https://mainnet.example.com"
    );

    // Test switch chain response
    let switch = RuntimeSwitchChainResponse {
        status: RuntimeOperationStatus {
            success: true,
            message: "Switched".to_string(),
        },
        chain: ChainType::Testnet,
    };
    let bytes = codec.serialize(&switch).unwrap();
    let deserialized: RuntimeSwitchChainResponse = codec.deserialize(&bytes).unwrap();
    assert_eq!(deserialized.chain, ChainType::Testnet);
}

#[tokio::test]
async fn test_local_chain_observation_types_roundtrip() {
    let codec = JsonCodec;

    let params = RuntimeStartScanParams {
        wallet_id: PersistWalletId("wallet-local".to_string()),
        from_height: Some(25),
    };
    let bytes = codec.serialize(&params).unwrap();
    let deserialized: RuntimeStartScanParams = codec.deserialize(&bytes).unwrap();
    assert_eq!(deserialized.wallet_id.0, "wallet-local");
    assert_eq!(deserialized.from_height, Some(25));

    let response = RuntimeStartScanResponse {
        job: z00z_wallets::rpc::types::common::RuntimeJobStatus {
            job_id: Some("scan-wallet-local".to_string()),
            status: None,
            progress: Some(0.0),
            eta_seconds: Some(600),
        },
        scan_range: None,
    };
    let bytes = codec.serialize(&response).unwrap();
    let deserialized: RuntimeStartScanResponse = codec.deserialize(&bytes).unwrap();
    assert_eq!(
        deserialized.job.job_id.as_deref(),
        Some("scan-wallet-local")
    );

    let status = RuntimeScanStatus {
        job: z00z_wallets::rpc::types::common::RuntimeJobStatus {
            job_id: Some("scan-wallet-local".to_string()),
            status: None,
            progress: Some(1.0),
            eta_seconds: None,
        },
        state: RuntimeScanState::Idle,
        current_height: 25,
        target_height: 25,
        last_receive_outcome: Some(RuntimeReceiveScanOutcome::ImportedHit),
    };
    let bytes = codec.serialize(&status).unwrap();
    let deserialized: RuntimeScanStatus = codec.deserialize(&bytes).unwrap();
    assert!(deserialized.is_scanned());
    assert_eq!(
        deserialized.last_receive_outcome,
        Some(RuntimeReceiveScanOutcome::ImportedHit)
    );

    let tip = RuntimeBlockInfo {
        height: 25,
        hash: "0x01".repeat(32),
        timestamp: 1_700_000_000_000,
        tx_count: 1,
    };
    let bytes = codec.serialize(&tip).unwrap();
    let deserialized: RuntimeBlockInfo = codec.deserialize(&bytes).unwrap();
    assert_eq!(deserialized.height, 25);
}

#[tokio::test]
async fn test_receipt_roots_no_proof() {
    let codec = JsonCodec;

    let receipt = PersistReceiptInfo {
        tx_id: PersistTxId::new("tx-compat".to_string()),
        block_height: 12,
        block_hash: "11".repeat(32),
        tx_index: 0,
        confirmations: 1,
        confirmed_at: 1_700_000_000_000,
        verified: true,
        merkle_proof: None,
    };
    let bytes = codec.serialize(&receipt).unwrap();
    let json: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    assert!(
        !json.as_object().unwrap().contains_key("merkle_proof"),
        "compatibility receipts must not expose placeholder proof fields"
    );

    let confirmation = RuntimeConfirmationReceipt {
        tx_id: PersistTxId::new("tx-live".to_string()),
        tx_hash_hex: "aa".repeat(32),
        block_height: 12,
        checkpoint_id_hex: "bb".repeat(32),
        prev_root_hex: "cc".repeat(32),
        new_root_hex: "dd".repeat(32),
        spent_asset_ids_hex: vec!["ee".repeat(32)],
        created_asset_ids_hex: vec!["ff".repeat(32)],
        confirmed_at: 1_700_000_000_123,
        verified: true,
    };
    let bytes = codec.serialize(&confirmation).unwrap();
    let json: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    let object = json.as_object().expect("confirmation receipt object");
    assert!(object.contains_key("checkpoint_id_hex"));
    assert!(object.contains_key("prev_root_hex"));
    assert!(object.contains_key("new_root_hex"));
}

#[tokio::test]
async fn test_security_types_roundtrip() {
    let codec = JsonCodec;

    // Test audit log entry
    let audit = PersistAuditLogEntry {
        timestamp: 1_700_000_000_000,
        wallet_id: Some(PersistWalletId("wallet-123".to_string())),
        method: "wallet.send".to_string(),
        client_ip: Some("192.168.1.1".to_string()),
        user_agent: Some("Mozilla/5.0".to_string()),
        result: AuditResult::Success,
        risk_level: RiskLevel::High,
        context: Some("amount: 1000".to_string()),
    };
    let bytes = codec.serialize(&audit).unwrap();
    let deserialized: PersistAuditLogEntry = codec.deserialize(&bytes).unwrap();
    assert_eq!(deserialized.timestamp, 1_700_000_000_000);
    assert_eq!(deserialized.method, "wallet.send");
    assert_eq!(deserialized.risk_level, RiskLevel::High);

    // Test audit log with none fields
    let audit_none = PersistAuditLogEntry {
        timestamp: 0,
        wallet_id: None,
        method: "test".to_string(),
        client_ip: None,
        user_agent: None,
        result: AuditResult::Success,
        risk_level: RiskLevel::Low,
        context: None,
    };
    let bytes = codec.serialize(&audit_none).unwrap();
    let deserialized: PersistAuditLogEntry = codec.deserialize(&bytes).unwrap();
    assert!(deserialized.wallet_id.is_none());
    assert!(deserialized.client_ip.is_none());
}

#[tokio::test]
async fn test_all_risk_levels_serializable() {
    let codec = JsonCodec;
    let levels = [
        RiskLevel::Critical,
        RiskLevel::High,
        RiskLevel::Medium,
        RiskLevel::Low,
    ];

    for level in levels {
        let audit = PersistAuditLogEntry {
            timestamp: 0,
            wallet_id: None,
            method: "test".to_string(),
            client_ip: None,
            user_agent: None,
            result: AuditResult::Success,
            risk_level: level,
            context: None,
        };
        let bytes = codec.serialize(&audit).unwrap();
        let deserialized: PersistAuditLogEntry = codec.deserialize(&bytes).unwrap();
        assert_eq!(deserialized.risk_level, level);
    }
}

#[tokio::test]
async fn test_empty_collections_roundtrip() {
    let codec = JsonCodec;

    // Empty logs
    let response = RuntimeViewLogsResponse { logs: vec![] };
    let bytes = codec.serialize(&response).unwrap();
    let deserialized: RuntimeViewLogsResponse = codec.deserialize(&bytes).unwrap();
    assert!(deserialized.logs.is_empty());

    // Empty backups
    let list = RuntimeListBackupsResponse {
        items: vec![],
        next_cursor: None,
        has_more: false,
        total_count: Some(0),
    };
    let bytes = codec.serialize(&list).unwrap();
    let deserialized: RuntimeListBackupsResponse = codec.deserialize(&bytes).unwrap();
    assert!(deserialized.items.is_empty());
}
