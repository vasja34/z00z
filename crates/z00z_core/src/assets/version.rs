/// Asset pack format version derived from `serial_id` range.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AssetPackVersion {
    /// Base fixed-width format for `serial_id` in `[0, 999_999]`.
    Basic,
    /// Extended memo-capable format for `serial_id` in `[1_000_000, 1_999_999]`.
    Memo,
    /// Unsupported non-live format for `serial_id >= 2_000_000`.
    Unknown,
}

/// Detect the currently implemented asset-pack lane by `serial_id` range.
///
/// This function is for `enc_pack` format detection only and MUST NOT be used
/// as a universal validity gate for NFT serial bounds.
#[must_use]
pub fn validate_serial_id_version(serial_id: u32) -> AssetPackVersion {
    match serial_id {
        0..=999_999 => AssetPackVersion::Basic,
        1_000_000..=1_999_999 => AssetPackVersion::Memo,
        _ => AssetPackVersion::Unknown,
    }
}
