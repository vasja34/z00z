//! Backup and restore functionality.
//!
//! This module provides abstractions for exporting and importing wallet backups
//! with encryption and metadata.

pub mod backup_exporter;
#[cfg(not(target_arch = "wasm32"))]
pub mod backup_exporter_impl;
pub mod backup_importer;
#[cfg(not(target_arch = "wasm32"))]
pub mod backup_importer_impl;
#[cfg(not(target_arch = "wasm32"))]
mod backup_wire;
#[cfg(not(target_arch = "wasm32"))]
pub mod wallet_backup;

// Re-export backup types
pub use backup_exporter::{
    BackupExporter, BackupExporterError, BackupExporterResult, BackupMetadata,
};
#[cfg(not(target_arch = "wasm32"))]
pub use backup_exporter_impl::BackupExporterImpl;
pub use backup_importer::{
    BackupImporter, BackupImporterError, BackupImporterResult, ImportedWalletData,
};
#[cfg(not(target_arch = "wasm32"))]
pub use backup_importer_impl::BackupImporterImpl;
#[cfg(not(target_arch = "wasm32"))]
pub use backup_wire::{
    decode_tx_history_jsonl, decode_tx_history_rows, encode_tx_history_jsonl,
    encode_tx_history_rows, fold_tx_history_rows, ForensicImportMode, WalletForensicPack,
    WalletTxHistoryEntryKind, WalletTxHistoryJsonlEntry, WalletTxHistoryManifest,
    WalletTxHistoryManifestEntry,
};

// Re-export wallet backup crypto facade
#[cfg(not(target_arch = "wasm32"))]
pub use wallet_backup::{BackupKdf, SaltPad, WalletBackupCrypto};
