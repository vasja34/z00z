use thiserror::Error;
use z00z_utils::{codec::CodecError, io::IoError};

/// Errors for canonical snapshot build, load, save, and replay access.
///
/// # Examples
///
/// ```
/// use z00z_storage::snapshot::PrepSnapshotError;
///
/// let err = PrepSnapshotError::VersionMix;
/// assert_eq!(err.to_string(), "unsupported snapshot version");
/// ```
#[derive(Debug, Error)]
pub enum PrepSnapshotError {
    /// Serialization or deserialization failed.
    #[error("codec error: {0}")]
    Codec(#[from] CodecError),
    /// Storage I/O failed.
    #[error("io error: {0}")]
    Io(#[from] IoError),
    /// One root-semantics boundary failed.
    #[error("snapshot root mismatch")]
    RootMix,
    /// One definition namespace binding failed.
    #[error("snapshot path mismatch")]
    PathMix,
    /// One serial bucket binding failed.
    #[error("snapshot serial mismatch")]
    SerialMix,
    /// One terminal-id binding failed.
    #[error("snapshot terminal-id mismatch")]
    TerminalIdMix,
    /// One terminal leaf payload binding failed.
    #[error("snapshot leaf mismatch")]
    LeafMix,
    /// One terminal leaf hash binding failed.
    #[error("snapshot leaf-hash mismatch")]
    LeafHashMix,
    /// One witness blob could not be decoded.
    #[error("witness decode failed: {0}")]
    WitDecode(CodecError),
    /// One storage-owned witness check failed after decode.
    #[error("witness validation failed")]
    WitMix,
    /// Derived snapshot id does not match the expected external id.
    #[error("snapshot id mismatch")]
    IdMix,
    /// Snapshot schema version is not supported.
    #[error("unsupported snapshot version")]
    VersionMix,
    /// One canonical path appears more than once in the same snapshot.
    #[error("duplicate snapshot path")]
    DupPath,
    /// One terminal id appears more than once in the same snapshot.
    #[error("duplicate terminal id")]
    DupTerminalId,
    /// Decoded proof context path does not match the canonical snapshot entry.
    #[error("replay path mismatch")]
    ReplayPathMix,
    /// Decoded proof context leaf does not match the canonical snapshot entry.
    #[error("replay leaf mismatch")]
    ReplayLeafMix,
    /// Backend-specific storage failure.
    #[error("snapshot backend failure: {0}")]
    Backend(String),
}
