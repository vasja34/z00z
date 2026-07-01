use super::*;
use crate::services::WalletService;
use std::ffi::OsString;
use std::sync::Arc;
use tempfile::tempdir;
use z00z_utils::io::write_file;

fn new_rpc() -> StorageRpcImpl {
    let _lock = crate::rpc::logging::RpcLoggingConfig::__lock_wallet_config_env();
    let _restore = WalletConfigEnvRestore::capture();
    std::env::remove_var("Z00Z_WALLET_CONFIG_PATH");
    std::env::remove_var("Z00Z_WALLET_NETWORK");
    std::env::remove_var("Z00Z_WALLET_CHAIN");

    StorageRpcImpl::with_default_service()
}

struct WalletConfigEnvRestore {
    prev_path: Option<OsString>,
    prev_network: Option<OsString>,
    prev_chain: Option<OsString>,
}

impl WalletConfigEnvRestore {
    fn capture() -> Self {
        Self {
            prev_path: std::env::var_os("Z00Z_WALLET_CONFIG_PATH"),
            prev_network: std::env::var_os("Z00Z_WALLET_NETWORK"),
            prev_chain: std::env::var_os("Z00Z_WALLET_CHAIN"),
        }
    }
}

impl Drop for WalletConfigEnvRestore {
    fn drop(&mut self) {
        match &self.prev_path {
            Some(value) => std::env::set_var("Z00Z_WALLET_CONFIG_PATH", value),
            None => std::env::remove_var("Z00Z_WALLET_CONFIG_PATH"),
        }
        match &self.prev_network {
            Some(value) => std::env::set_var("Z00Z_WALLET_NETWORK", value),
            None => std::env::remove_var("Z00Z_WALLET_NETWORK"),
        }
        match &self.prev_chain {
            Some(value) => std::env::set_var("Z00Z_WALLET_CHAIN", value),
            None => std::env::remove_var("Z00Z_WALLET_CHAIN"),
        }
    }
}

fn new_rpc_with_output_dir(output_dir: std::path::PathBuf) -> StorageRpcImpl {
    let _lock = crate::rpc::logging::RpcLoggingConfig::__lock_wallet_config_env();
    let _restore = WalletConfigEnvRestore::capture();
    std::env::remove_var("Z00Z_WALLET_CONFIG_PATH");
    std::env::remove_var("Z00Z_WALLET_NETWORK");
    std::env::remove_var("Z00Z_WALLET_CHAIN");

    StorageRpcImpl::new(Arc::new(WalletService::with_output_dir(output_dir)))
}

#[tokio::test]
async fn test_compact_storage_normal() {
    let rpc = new_rpc();
    let params = RuntimeCompactStorageParams {
        force: false,
        dry_run: false,
    };

    let response = rpc.compact_storage(params).await.unwrap();

    assert!(response.performed);
    assert!(response.duration_ms > 0);
    assert!(response.size_before_bytes >= response.bytes_reclaimed);
}

#[tokio::test]
async fn test_compact_storage_force() {
    let rpc = new_rpc();
    let params = RuntimeCompactStorageParams {
        force: true,
        dry_run: false,
    };

    let response = rpc.compact_storage(params).await.unwrap();

    assert!(response.performed);
    assert!(response.duration_ms > 0);
    assert!(response.size_before_bytes >= response.bytes_reclaimed);
}

#[tokio::test]
async fn test_compact_storage_dry_run() {
    let rpc = new_rpc();
    let params = RuntimeCompactStorageParams {
        force: false,
        dry_run: true,
    };

    let response = rpc.compact_storage(params).await.unwrap();

    assert!(!response.performed);
    assert!(response.duration_ms > 0);
}

#[tokio::test]
async fn test_get_storage_stats_basic() {
    let rpc = new_rpc();
    let params = RuntimeGetStorageStatsParams {
        include_details: false,
    };

    let stats = rpc.get_storage_stats(params).await.unwrap();
    assert_eq!(stats.transaction_count, 0);
    assert!(stats.fragmentation_percent >= 0.0);
}

#[tokio::test]
async fn test_get_storage_stats_detailed() {
    let rpc = new_rpc();
    let params = RuntimeGetStorageStatsParams {
        include_details: true,
    };

    let stats = rpc.get_storage_stats(params).await.unwrap();

    assert!(!stats.db_path.is_empty());
}

#[tokio::test]
async fn test_stats_count_wlts() {
    let dir = tempdir().unwrap();
    let rpc = new_rpc_with_output_dir(dir.path().to_path_buf());

    write_file(dir.path().join("wallet_alpha.wlt"), b"alpha").unwrap();
    write_file(dir.path().join("wallet_beta.wlt"), b"beta").unwrap();
    write_file(
        dir.path().join("wallet_alpha_tx_history.jsonl"),
        b"{\"tx_id\":\"1\"}\n",
    )
    .unwrap();
    write_file(dir.path().join("stale.tmp"), b"x").unwrap();

    let stats = rpc
        .get_storage_stats(RuntimeGetStorageStatsParams {
            include_details: true,
        })
        .await
        .unwrap();

    assert_eq!(stats.wallet_count, 2);
    assert_eq!(stats.transaction_count, 0);
    assert_eq!(stats.deleted_bytes, 1);
    assert_eq!(stats.db_path, dir.path().display().to_string());
}

#[tokio::test]
async fn test_export_storage_json() {
    let rpc = new_rpc();
    let params = RuntimeExportStorageParams {
        path: "/backup/export.json".to_string(),
        format: "json".to_string(),
        include_deleted: false,
    };

    let response = rpc.export_storage(params).await.unwrap();

    assert_eq!(response.job.status.as_deref(), Some("completed"));
    assert!(response
        .job
        .job_id
        .as_deref()
        .is_some_and(|id| id.starts_with("export_")));
    assert_eq!(response.export_path, "/backup/export.json");
    assert_eq!(response.format, "json");
}

#[tokio::test]
async fn test_export_storage_invalid_format() {
    let rpc = new_rpc();
    let params = RuntimeExportStorageParams {
        path: "/backup/export.xyz".to_string(),
        format: "invalid_format".to_string(),
        include_deleted: false,
    };

    let response = rpc.export_storage(params).await.unwrap();

    assert_eq!(response.job.status.as_deref(), Some("failed"));
    assert_eq!(response.export_path, "/backup/export.xyz");
    assert_eq!(response.format, "invalid_format");
}

#[tokio::test]
async fn test_export_storage_all_formats() {
    let rpc = new_rpc();

    for format in &["json", "sql", "binary"] {
        let params = RuntimeExportStorageParams {
            path: format!("/backup/export.{}", format),
            format: format.to_string(),
            include_deleted: false,
        };

        let response = rpc.export_storage(params).await.unwrap();
        if *format == "json" {
            assert_eq!(response.job.status.as_deref(), Some("completed"));
        } else {
            assert_eq!(response.job.status.as_deref(), Some("not_implemented"));
        }

        assert_eq!(response.export_path, format!("/backup/export.{}", format));
        assert_eq!(response.format, *format);
    }
}
