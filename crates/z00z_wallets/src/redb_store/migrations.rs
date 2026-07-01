use super::{
    Database, Path, WalletError, WalletIo, WalletResult, INDEX_FORMAT_VERSION_HMAC, WLT_ZSTD_MAGIC,
};

pub(crate) fn is_zstd_magic_bytes(bytes: &[u8]) -> bool {
    bytes.len() >= 4
        && bytes[0] == WLT_ZSTD_MAGIC[0]
        && bytes[1] == WLT_ZSTD_MAGIC[1]
        && bytes[2] == WLT_ZSTD_MAGIC[2]
        && bytes[3] == WLT_ZSTD_MAGIC[3]
}

pub(crate) fn migrate_index_format_if_needed(
    db: &Database,
    io: &dyn WalletIo,
    original_path: &Path,
    work_path: &Path,
    stored_version: u32,
) -> WalletResult<()> {
    if stored_version == INDEX_FORMAT_VERSION_HMAC {
        return Ok(());
    }
    let _ = db;
    let _ = io;
    let _ = original_path;
    let _ = work_path;
    Err(WalletError::InvalidConfig(
        "unsupported index format version".to_string(),
    ))
}
