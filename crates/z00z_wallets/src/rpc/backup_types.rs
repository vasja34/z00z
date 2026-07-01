//! Backup and restore RPC types
//!
//! Request and response types for backup.* JSON-RPC methods

use serde::{Deserialize, Serialize};

use super::common::{PersistWalletId, RuntimeOperationStatus, RuntimePaginatedResponse};

/// Create backup response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimeCreateBackupResponse {
    #[serde(flatten)]
    pub status: RuntimeOperationStatus,
    pub backup_path: String,
    pub encrypted: bool,
}

/// Backup list response.
pub type RuntimeListBackupsResponse = RuntimePaginatedResponse<PersistBackupInfo>;

/// Backup information.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersistBackupInfo {
    pub id: String,
    pub wallet_id: PersistWalletId,

    /// Creation time (Unix timestamp in milliseconds).
    pub created_at: u64,
    pub size_bytes: u64,
    pub encrypted: bool,
}

/// Restore backup response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimeRestoreBackupResponse {
    #[serde(flatten)]
    pub status: RuntimeOperationStatus,
    pub wallet_id: PersistWalletId,
}

/// Backup settings.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersistBackupSettings {
    pub auto_backup_enabled: bool,
    pub backup_interval_hours: u32,
    pub backup_location: String,
    pub encrypt_backups: bool,
}

/// Backup settings response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimeBackupSettingsResponse {
    pub settings: PersistBackupSettings,
}

#[cfg(test)]
mod tests {
    use super::*;
    use z00z_utils::codec::{Codec, JsonCodec};

    #[test]
    fn test_create_backup_response_serialization() {
        let response = RuntimeCreateBackupResponse {
            status: RuntimeOperationStatus {
                success: true,
                message: "Backup created".to_string(),
            },
            backup_path: "/path/to/backup".to_string(),
            encrypted: true,
        };

        let codec = JsonCodec;
        let bytes = codec.serialize(&response).unwrap();
        let deserialized: RuntimeCreateBackupResponse = codec.deserialize(&bytes).unwrap();

        assert!(deserialized.status.success);
        assert_eq!(deserialized.backup_path, "/path/to/backup");
        assert!(deserialized.encrypted);
    }

    #[test]
    fn test_runtime_backups_response_pagination() {
        let info = PersistBackupInfo {
            id: "backup-001".to_string(),
            wallet_id: PersistWalletId("wallet-123".to_string()),
            created_at: 1_700_000_000_000,
            size_bytes: 1024,
            encrypted: true,
        };

        let response = RuntimeListBackupsResponse {
            items: vec![info],
            next_cursor: None,
            has_more: false,
            total_count: Some(1),
        };

        let codec = JsonCodec;
        let bytes = codec.serialize(&response).unwrap();
        let deserialized: RuntimeListBackupsResponse = codec.deserialize(&bytes).unwrap();

        assert_eq!(deserialized.items.len(), 1);
        assert_eq!(deserialized.items[0].id, "backup-001");
        assert_eq!(deserialized.items[0].size_bytes, 1024);
    }

    #[test]
    fn test_persist_backup_timestamp_format() {
        let info = PersistBackupInfo {
            id: "backup-001".to_string(),
            wallet_id: PersistWalletId("wallet-123".to_string()),
            created_at: 1_700_000_000_000,
            size_bytes: 1024,
            encrypted: true,
        };

        // Verify timestamp is in milliseconds
        assert!(info.created_at > 1_000_000_000_000);
        assert!(info.created_at < 2_000_000_000_000);
    }

    #[test]
    fn test_runtime_restore_backup_response() {
        let response = RuntimeRestoreBackupResponse {
            status: RuntimeOperationStatus {
                success: true,
                message: "Restored".to_string(),
            },
            wallet_id: PersistWalletId("wallet-restored".to_string()),
        };

        let codec = JsonCodec;
        let bytes = codec.serialize(&response).unwrap();
        let deserialized: RuntimeRestoreBackupResponse = codec.deserialize(&bytes).unwrap();

        assert!(deserialized.status.success);
        assert_eq!(deserialized.wallet_id.0, "wallet-restored");
    }

    #[test]
    fn test_persist_backup_settings() {
        let settings = PersistBackupSettings {
            auto_backup_enabled: true,
            backup_interval_hours: 24,
            backup_location: "/backups".to_string(),
            encrypt_backups: true,
        };

        let codec = JsonCodec;
        let bytes = codec.serialize(&settings).unwrap();
        let deserialized: PersistBackupSettings = codec.deserialize(&bytes).unwrap();

        assert!(deserialized.auto_backup_enabled);
        assert_eq!(deserialized.backup_interval_hours, 24);
        assert!(deserialized.encrypt_backups);
    }

    #[test]
    fn test_runtime_backup_settings_response() {
        let response = RuntimeBackupSettingsResponse {
            settings: PersistBackupSettings {
                auto_backup_enabled: false,
                backup_interval_hours: 48,
                backup_location: "/custom".to_string(),
                encrypt_backups: false,
            },
        };

        let codec = JsonCodec;
        let bytes = codec.serialize(&response).unwrap();
        let deserialized: RuntimeBackupSettingsResponse = codec.deserialize(&bytes).unwrap();

        assert!(!deserialized.settings.auto_backup_enabled);
        assert_eq!(deserialized.settings.backup_interval_hours, 48);
    }
}
