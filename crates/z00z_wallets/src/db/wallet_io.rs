#![cfg(not(target_arch = "wasm32"))]
use std::path::Path;
use z00z_utils::io;

use crate::{WalletError, WalletResult};

pub(crate) fn create_dir_all(path: &Path) -> WalletResult<()> {
    io::create_dir_all(path)
        .map_err(|e| WalletError::InvalidConfig(format!("create_dir_all failed: {e}")))
}

pub(crate) fn read_file(path: &Path) -> WalletResult<Vec<u8>> {
    io::read_file(path).map_err(|e| WalletError::InvalidConfig(format!("read_file failed: {e}")))
}

pub(crate) fn atomic_write_file_private(path: &Path, data: &[u8]) -> WalletResult<()> {
    io::atomic_write_file_private(path, data)
        .map_err(|e| WalletError::InvalidConfig(format!("atomic_write_file_private failed: {e}")))
}

pub(crate) fn remove_file_best_effort(path: &Path) {
    let _ = io::remove_file(path);
}

pub(crate) fn path_exists(path: &Path) -> WalletResult<bool> {
    io::path_exists(path)
        .map_err(|e| WalletError::InvalidConfig(format!("path_exists failed: {e}")))
}

pub(crate) fn set_private_file_permissions(path: &Path) -> WalletResult<()> {
    #[cfg(unix)]
    {
        io::set_permissions_mode(path, 0o600).map_err(|_| {
            WalletError::InvalidConfig("wallet permission hardening failed".to_string())
        })?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_use_std_fs_remove() {
        let src = include_str!("wallet_io.rs");
        let remove_file = ["std::fs", "::remove_file"].concat();
        let set_permissions = ["std::fs", "::set_permissions"].concat();
        assert!(!src.contains(&remove_file));
        assert!(!src.contains(&set_permissions));
    }
}
