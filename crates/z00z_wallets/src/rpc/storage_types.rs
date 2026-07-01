//! Storage administration types.
//!
//! Request/response types for storage RPC methods.

use serde::{Deserialize, Serialize};

use super::common::RuntimeJobStatus;

// ============================================================================
// Compact Storage
// ============================================================================

/// Parameters for wallet.storage.compact_storage method.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RuntimeCompactStorageParams {
    /// Force compaction even if not needed
    pub force: bool,
    /// Dry run mode (estimate only, don't compact)
    pub dry_run: bool,
}

/// Response from wallet.storage.compact_storage method.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimeCompactStorageResponse {
    /// Bytes reclaimed by compaction
    pub bytes_reclaimed: u64,
    /// Duration in milliseconds
    pub duration_ms: u64,
    /// Size before compaction
    pub size_before_bytes: u64,
    /// Size after compaction
    pub size_after_bytes: u64,
    /// Whether compaction was actually performed
    pub performed: bool,
}

// ============================================================================
// Get Storage Stats
// ============================================================================

/// Parameters for wallet.storage.get_storage_stats method.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RuntimeGetStorageStatsParams {
    /// Include detailed per-wallet statistics
    pub include_details: bool,
}

/// Storage statistics response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimeStorageStats {
    /// Total database size in bytes
    pub total_size_bytes: u64,
    /// Number of wallets stored
    pub wallet_count: u32,
    /// Number of transactions stored
    pub transaction_count: u64,
    /// Fragmentation percentage (0-100)
    pub fragmentation_percent: f32,
    /// Bytes used by deleted records (reclaimable)
    pub deleted_bytes: u64,
    /// Database file path
    pub db_path: String,
    /// Last compaction timestamp (milliseconds since Unix epoch).
    pub last_compact_at: Option<u64>,
}

// ============================================================================
// Export Storage
// ============================================================================

/// Parameters for wallet.storage.export_storage method.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimeExportStorageParams {
    /// Export file path
    pub path: String,
    /// Export format.
    ///
    /// Valid values: "json", "sql", "binary".
    ///
    /// Kept as `String` for extensibility and backwards compatibility.
    pub format: String,
    /// Include deleted records in export
    pub include_deleted: bool,
}

/// Response from wallet.storage.export_storage method.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimeExportStorageResponse {
    #[serde(flatten)]
    pub job: RuntimeJobStatus,
    /// Export file path.
    pub export_path: String,
    /// Export format.
    ///
    /// Valid values: "json", "sql", "binary".
    pub format: String,
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use z00z_utils::codec::{Codec, JsonCodec};

    #[test]
    fn test_compact_storage_params_default() {
        let params = RuntimeCompactStorageParams::default();
        assert!(!params.force);
        assert!(!params.dry_run);
    }

    #[test]
    fn test_compact_storage_params_serialization() {
        let params = RuntimeCompactStorageParams {
            force: true,
            dry_run: false,
        };

        let codec = JsonCodec;
        let bytes = codec.serialize(&params).unwrap();
        let deserialized: RuntimeCompactStorageParams = codec.deserialize(&bytes).unwrap();

        assert!(deserialized.force);
        assert!(!deserialized.dry_run);
    }

    #[test]
    fn test_compact_storage_response_serialization() {
        let response = RuntimeCompactStorageResponse {
            bytes_reclaimed: 1024000,
            duration_ms: 350,
            size_before_bytes: 5000000,
            size_after_bytes: 3976000,
            performed: true,
        };

        let codec = JsonCodec;
        let bytes = codec.serialize(&response).unwrap();
        let deserialized: RuntimeCompactStorageResponse = codec.deserialize(&bytes).unwrap();

        assert_eq!(deserialized.bytes_reclaimed, 1024000);
        assert!(deserialized.performed);
    }

    #[test]
    fn test_storage_stats_serialization() {
        let stats = RuntimeStorageStats {
            total_size_bytes: 10485760,
            wallet_count: 5,
            transaction_count: 150,
            fragmentation_percent: 12.5,
            deleted_bytes: 524288,
            db_path: "/data/wallets.db".to_string(),
            last_compact_at: Some(1_700_000_000_000),
        };

        let codec = JsonCodec;
        let bytes = codec.serialize(&stats).unwrap();
        let deserialized: RuntimeStorageStats = codec.deserialize(&bytes).unwrap();

        assert_eq!(deserialized.wallet_count, 5);
        assert_eq!(deserialized.fragmentation_percent, 12.5);
        assert_eq!(deserialized.last_compact_at, Some(1_700_000_000_000));
    }

    #[test]
    fn test_export_storage_params_serialization() {
        let params = RuntimeExportStorageParams {
            path: "/backup/export.json".to_string(),
            format: "json".to_string(),
            include_deleted: true,
        };

        let codec = JsonCodec;
        let bytes = codec.serialize(&params).unwrap();
        let deserialized: RuntimeExportStorageParams = codec.deserialize(&bytes).unwrap();

        assert_eq!(deserialized.path, "/backup/export.json");
        assert_eq!(deserialized.format, "json");
        assert!(deserialized.include_deleted);
    }

    #[test]
    fn test_export_storage_response_serialization() {
        let response = RuntimeExportStorageResponse {
            job: RuntimeJobStatus {
                job_id: Some("export_job_42".to_string()),
                status: Some("completed".to_string()),
                progress: None,
                eta_seconds: None,
            },
            export_path: "/backup/export.json".to_string(),
            format: "json".to_string(),
        };

        let codec = JsonCodec;
        let bytes = codec.serialize(&response).unwrap();
        let deserialized: RuntimeExportStorageResponse = codec.deserialize(&bytes).unwrap();

        // Schema compatibility: export response stays job_id+status based (no progress/eta fields).
        let json = String::from_utf8(bytes).unwrap();
        assert!(json.contains("\"job_id\""));
        assert!(json.contains("\"status\""));
        assert!(!json.contains("\"progress\""));
        assert!(!json.contains("\"eta_seconds\""));

        assert_eq!(deserialized.job.job_id.as_deref(), Some("export_job_42"));
        assert_eq!(deserialized.job.status.as_deref(), Some("completed"));
        assert_eq!(deserialized.export_path, "/backup/export.json");
        assert_eq!(deserialized.format, "json");
    }

    #[test]
    fn test_get_storage_stats_params() {
        let params = RuntimeGetStorageStatsParams::default();
        assert!(!params.include_details);
    }
}
