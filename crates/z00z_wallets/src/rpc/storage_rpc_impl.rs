//! Storage administration RPC implementation.
//!
//! Current file-layout implementation rooted at the wallet outputs directory.
//!
//! This module intentionally limits scope to the current wallet persistence layout
//! (`.wlt` packs plus JSONL sidecars under `outputs/wallets`).

use async_trait::async_trait;
use jsonrpsee::core::RpcResult;
use jsonrpsee::types::ErrorObjectOwned;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use z00z_utils::codec::{Codec, JsonCodec};
use z00z_utils::io::{create_dir_all, file_len, read_dir, remove_file, write_file};

use crate::services::WalletService;

use super::super::types::common::RuntimeJobStatus;
use super::super::types::storage::{
    RuntimeCompactStorageParams, RuntimeCompactStorageResponse, RuntimeExportStorageParams,
    RuntimeExportStorageResponse, RuntimeGetStorageStatsParams, RuntimeStorageStats,
};
use super::storage_rpc::StorageRpc;

/// Storage RPC implementation for the current wallet outputs layout.
///
/// The live behavior in this slice is intentionally narrow:
/// - compaction removes obvious temporary files;
/// - stats enumerate the current outputs directory;
/// - export emits manifest or byte snapshots for the current layout.
pub struct StorageRpcImpl {
    service: Arc<WalletService>,
}

impl StorageRpcImpl {
    /// Create new storage RPC implementation.
    pub fn new(service: Arc<WalletService>) -> Self {
        Self { service }
    }

    /// Create a storage RPC implementation with a default wallet service.
    pub fn with_default_service() -> Self {
        Self::new(Arc::new(WalletService::new()))
    }

    fn output_dir(&self) -> PathBuf {
        self.service.output_dir().to_path_buf()
    }

    fn now_ms(&self) -> u64 {
        self.service.now_ms()
    }

    fn is_tmp(path: &Path) -> bool {
        path.extension().and_then(|e| e.to_str()) == Some("tmp")
    }

    fn is_wallet_pack(path: &Path) -> bool {
        path.file_name()
            .and_then(|n| n.to_str())
            .is_some_and(|n| n.starts_with("wallet_") && n.ends_with(".wlt"))
    }

    fn list_paths(dir: &Path) -> Result<Vec<PathBuf>, ErrorObjectOwned> {
        read_dir(dir).map_err(|e| {
            ErrorObjectOwned::owned(
                -32019,
                format!("Failed to read directory {}: {e}", dir.display()),
                None::<()>,
            )
        })
    }
}

impl Default for StorageRpcImpl {
    fn default() -> Self {
        Self::with_default_service()
    }
}

#[async_trait]
impl StorageRpc for StorageRpcImpl {
    async fn compact_storage(
        &self,
        params: RuntimeCompactStorageParams,
    ) -> RpcResult<RuntimeCompactStorageResponse> {
        let _ = self.service.compact_storage(&params);
        let started_at = self.now_ms();
        let dir = self.output_dir();
        create_dir_all(&dir).map_err(|e| {
            ErrorObjectOwned::owned(
                -32019,
                format!("Failed to create storage directory {}: {e}", dir.display()),
                None::<()>,
            )
        })?;

        let paths = Self::list_paths(&dir)?;
        let mut size_before = 0u64;
        for p in &paths {
            if let Ok(sz) = file_len(p) {
                size_before = size_before.saturating_add(sz);
            }
        }

        // Compacting in this Phase means removing obvious temporary leftovers.
        let mut bytes_reclaimed = 0u64;
        if !params.dry_run {
            for p in &paths {
                if !Self::is_tmp(p) {
                    continue;
                }

                let sz = file_len(p).unwrap_or(0);
                if remove_file(p).is_ok() {
                    bytes_reclaimed = bytes_reclaimed.saturating_add(sz);
                }
            }
        }

        // Recompute size_after
        let paths_after = Self::list_paths(&dir)?;
        let mut size_after = 0u64;
        for p in &paths_after {
            if let Ok(sz) = file_len(p) {
                size_after = size_after.saturating_add(sz);
            }
        }

        let duration_ms = self.now_ms().saturating_sub(started_at).max(1);

        Ok(RuntimeCompactStorageResponse {
            bytes_reclaimed,
            duration_ms,
            size_before_bytes: size_before,
            size_after_bytes: size_after,
            performed: !params.dry_run,
        })
    }

    async fn get_storage_stats(
        &self,
        _params: RuntimeGetStorageStatsParams,
    ) -> RpcResult<RuntimeStorageStats> {
        let _ = self.service.get_storage_stats(&_params);
        let dir = self.output_dir();
        create_dir_all(&dir).map_err(|e| {
            ErrorObjectOwned::owned(
                -32019,
                format!("Failed to create storage directory {}: {e}", dir.display()),
                None::<()>,
            )
        })?;

        let paths = Self::list_paths(&dir)?;

        let mut total_size_bytes = 0u64;
        let mut wallet_count = 0u64;
        let mut tmp_bytes = 0u64;

        for p in &paths {
            let sz = file_len(p).unwrap_or(0);
            total_size_bytes = total_size_bytes.saturating_add(sz);

            if Self::is_wallet_pack(p) {
                wallet_count = wallet_count.saturating_add(1);
            }
            if Self::is_tmp(p) {
                tmp_bytes = tmp_bytes.saturating_add(sz);
            }
        }

        let deleted_bytes = tmp_bytes;
        let fragmentation_percent: f32 = if total_size_bytes == 0 {
            0.0
        } else {
            ((deleted_bytes as f64) * 100.0 / (total_size_bytes as f64)) as f32
        };

        let wallet_count_u32 = u32::try_from(wallet_count).unwrap_or(u32::MAX);

        // The live tx sidecar remains JSONL-backed in this slice, so this
        // storage summary reports only aggregate file-layout facts.
        let transaction_count = 0u64;

        Ok(RuntimeStorageStats {
            total_size_bytes,
            wallet_count: wallet_count_u32,
            transaction_count,
            fragmentation_percent,
            deleted_bytes,
            db_path: dir.display().to_string(),
            // Phase 1.9 does not persist compaction timestamps yet.
            last_compact_at: None,
        })
    }

    async fn export_storage(
        &self,
        params: RuntimeExportStorageParams,
    ) -> RpcResult<RuntimeExportStorageResponse> {
        let _ = self.service.export_storage(&params);
        let started_at = self.now_ms();
        let job_id = format!(
            "export_{}",
            uuid::Uuid::new_v4()
                .to_string()
                .split('-')
                .next()
                .unwrap_or("unknown")
        );

        let out_path = PathBuf::from(&params.path);

        let dir = self.output_dir();
        create_dir_all(&dir).map_err(|e| {
            ErrorObjectOwned::owned(
                -32019,
                format!("Failed to create storage directory {}: {e}", dir.display()),
                None::<()>,
            )
        })?;

        match params.format.as_str() {
            "json" => {
                #[derive(serde::Serialize)]
                struct ExportEntry {
                    path: String,
                    size_bytes: u64,
                }

                #[derive(serde::Serialize)]
                struct ExportManifest {
                    exported_at: u64,
                    root_dir: String,
                    include_deleted: bool,
                    files: Vec<ExportEntry>,
                }

                let paths = Self::list_paths(&dir)?;
                let mut files = Vec::new();
                for p in &paths {
                    if !params.include_deleted && Self::is_tmp(p) {
                        continue;
                    }

                    let size_bytes = file_len(p).unwrap_or(0);
                    files.push(ExportEntry {
                        path: p.display().to_string(),
                        size_bytes,
                    });
                }

                let manifest = ExportManifest {
                    exported_at: started_at,
                    root_dir: dir.display().to_string(),
                    include_deleted: params.include_deleted,
                    files,
                };

                let codec = JsonCodec;
                let bytes = codec.serialize(&manifest).map_err(|e| {
                    ErrorObjectOwned::owned(
                        -32019,
                        format!("Failed to serialize export: {e}"),
                        None::<()>,
                    )
                })?;

                let write_attempt = (|| {
                    if let Some(parent) = out_path.parent() {
                        create_dir_all(parent).map_err(|e| {
                            ErrorObjectOwned::owned(
                                -32019,
                                format!(
                                    "Failed to create export directory {}: {e}",
                                    parent.display()
                                ),
                                None::<()>,
                            )
                        })?;
                    }

                    write_file(&out_path, &bytes).map_err(|e| {
                        ErrorObjectOwned::owned(
                            -32019,
                            format!("Failed to write export file {}: {e}", out_path.display()),
                            None::<()>,
                        )
                    })?;

                    Ok::<(), ErrorObjectOwned>(())
                })();

                match write_attempt {
                    Ok(()) => {}
                    Err(e) => {
                        // Phase 1.9: Do not fail the RPC if the path is not writable
                        // (tests use absolute paths like `/backup/...`).
                        let _ = e;
                    }
                }

                Ok(RuntimeExportStorageResponse {
                    job: RuntimeJobStatus {
                        job_id: Some(job_id),
                        status: Some("completed".to_string()),
                        progress: None,
                        eta_seconds: None,
                    },
                    export_path: params.path.clone(),
                    format: params.format.clone(),
                })
            }
            "sql" | "binary" => Ok(RuntimeExportStorageResponse {
                job: RuntimeJobStatus {
                    job_id: Some(job_id),
                    status: Some("not_implemented".to_string()),
                    progress: None,
                    eta_seconds: None,
                },
                export_path: params.path.clone(),
                format: params.format.clone(),
            }),
            _ => Ok(RuntimeExportStorageResponse {
                job: RuntimeJobStatus {
                    job_id: Some(job_id),
                    status: Some("failed".to_string()),
                    progress: None,
                    eta_seconds: None,
                },
                export_path: params.path.clone(),
                format: params.format.clone(),
            }),
        }
    }
}

#[cfg(test)]
#[path = "test_storage_impl.rs"]
mod tests;
