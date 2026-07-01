//! Offline `.wlt` validator (local-only).
//!
//! This module provides a bounded, non-secret-leaking validator for `.wlt` containers.
//! It is intended for diagnosing corruption without widening password oracles.

#![cfg(not(target_arch = "wasm32"))]

use std::path::Path;

use redb::{ReadableDatabase, TableDefinition};
use z00z_utils::time::TimeProvider;

use crate::db::schema_keys::{
    META_SCHEMA_VERSION, META_WALLET_ID, META_WALLET_INTEGRITY, META_WALLET_KDF,
};
use crate::db::wallet_store::WalletIo;
use crate::wallet::errors::WalletDiagCode;
use crate::{WalletError, WalletResult};

// Whole-wallet size cap (DoS bounds).
// This is the maximum allowed size of the decompressed RedB file when loading a zstd `.wlt`.
const MAX_WLT_DECOMPRESSED_BYTES: usize = 128 * 1024 * 1024;

// Zstd frame magic number (little-endian): 0xFD2FB528
const WLT_ZSTD_MAGIC: [u8; 4] = [0x28, 0xB5, 0x2F, 0xFD];

const META_TABLE: TableDefinition<&str, &[u8]> = TableDefinition::new("meta");
const SECRETS_TABLE: TableDefinition<&str, &[u8]> = TableDefinition::new("secrets");

fn is_zstd_magic_bytes(bytes: &[u8]) -> bool {
    bytes.len() >= 4
        && bytes[0] == WLT_ZSTD_MAGIC[0]
        && bytes[1] == WLT_ZSTD_MAGIC[1]
        && bytes[2] == WLT_ZSTD_MAGIC[2]
        && bytes[3] == WLT_ZSTD_MAGIC[3]
}

/// Validate a `.wlt` file structure without requiring a password.
///
/// Returns a list of diagnostic codes describing issues found. An empty list means the
/// container passed basic structural validation.
pub(crate) fn validate_wallet_file(
    path: &Path,
    io: &dyn WalletIo,
) -> WalletResult<Vec<WalletDiagCode>> {
    let mut diags = Vec::new();

    let bytes = io.read_file(path)?;
    if !is_zstd_magic_bytes(&bytes) {
        diags.push(WalletDiagCode::ContainerInvalid);
        return Ok(diags);
    }

    let db_bytes = match z00z_utils::compression::zstd_decompress_bounded(
        &bytes,
        MAX_WLT_DECOMPRESSED_BYTES,
    ) {
        Ok(b) => b,
        Err(_) => {
            diags.push(WalletDiagCode::DecompressFail);
            return Ok(diags);
        }
    };

    let tmp_guard = TempPathGuard::new();
    z00z_utils::io::write_file(&tmp_guard.path, &db_bytes).map_err(|e| {
        WalletError::InvalidConfig(format!("validator temp file write failed: {e}"))
    })?;

    let db = match redb::Database::open(&tmp_guard.path) {
        Ok(db) => db,
        Err(_) => {
            diags.push(WalletDiagCode::DbOpenFail);
            return Ok(diags);
        }
    };

    let read_txn = match db.begin_read() {
        Ok(txn) => txn,
        Err(_) => {
            diags.push(WalletDiagCode::DbOpenFail);
            return Ok(diags);
        }
    };

    let meta = match read_txn.open_table(META_TABLE) {
        Ok(t) => t,
        Err(_) => {
            diags.push(WalletDiagCode::MetaInvalid);
            return Ok(diags);
        }
    };

    if meta.get(META_WALLET_ID).ok().flatten().is_none() {
        diags.push(WalletDiagCode::MetaInvalid);
    }
    if meta.get(META_SCHEMA_VERSION).ok().flatten().is_none() {
        diags.push(WalletDiagCode::MetaInvalid);
    }
    if meta.get(META_WALLET_KDF).ok().flatten().is_none() {
        diags.push(WalletDiagCode::MetaInvalid);
    }

    if let Some(mac) = meta.get(META_WALLET_INTEGRITY).ok().flatten() {
        if mac.value().len() != 32 {
            diags.push(WalletDiagCode::IntegrityMissing);
        }
    } else {
        diags.push(WalletDiagCode::IntegrityMissing);
    }

    let secrets = match read_txn.open_table(SECRETS_TABLE) {
        Ok(t) => t,
        Err(_) => {
            diags.push(WalletDiagCode::SecretsMissing);
            return Ok(diags);
        }
    };

    // Presence-only checks; never attempt to decrypt in the validator.
    if secrets.get("master_key").ok().flatten().is_none() {
        diags.push(WalletDiagCode::SecretsMissing);
    }
    if secrets.get("seed_main").ok().flatten().is_none() {
        diags.push(WalletDiagCode::SecretsMissing);
    }

    Ok(dedup_diags(diags))
}

/// Validate a `.wlt` file structure without requiring a password.
///
/// This is the public entrypoint intended for offline tooling.
/// It returns a list of stable diagnostic code strings (e.g., `META_INVALID`).
#[cfg(not(target_arch = "wasm32"))]
pub fn validate_wallet_file_codes(path: &Path) -> WalletResult<Vec<String>> {
    let io = crate::db::wallet_store::Z00ZWalletIo;
    let diags = validate_wallet_file(path, &io)?;
    Ok(diags.into_iter().map(|d| d.to_string()).collect())
}

fn dedup_diags(mut diags: Vec<WalletDiagCode>) -> Vec<WalletDiagCode> {
    diags.sort_by_key(diag_rank);
    diags.dedup();
    diags
}

fn diag_rank(d: &WalletDiagCode) -> u8 {
    match d {
        WalletDiagCode::ContainerInvalid => 1,
        WalletDiagCode::DecompressFail => 2,
        WalletDiagCode::DbOpenFail => 3,
        WalletDiagCode::MetaInvalid => 4,
        WalletDiagCode::IntegrityMissing => 5,
        WalletDiagCode::SecretsMissing => 6,
    }
}

struct TempPathGuard {
    path: std::path::PathBuf,
}

impl TempPathGuard {
    fn new() -> Self {
        let pid = std::process::id();
        let ts_ms = z00z_utils::time::SystemTimeProvider.compat_unix_timestamp_millis();
        let path = std::env::temp_dir().join(format!("z00z_wlt_validate_{pid}_{ts_ms}.redb"));
        Self { path }
    }
}

impl Drop for TempPathGuard {
    fn drop(&mut self) {
        let _ = z00z_utils::io::remove_file(&self.path);
    }
}
