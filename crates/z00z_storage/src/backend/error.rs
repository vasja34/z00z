use thiserror::Error;
use z00z_utils::{codec::CodecError, io::IoError};

/// Errors for the private durable storage backend.
#[derive(Debug, Error)]
pub(crate) enum StoreBackendError {
    /// Backend directory setup failed.
    #[error("backend open failure: {0}")]
    Open(String),
    /// Persisted storage generation is not accepted by the live reload path.
    #[error("backend unsupported generation: {0}")]
    UnsupportedGeneration(String),
    /// Transaction staging failed before commit.
    #[error("backend transaction failure: {0}")]
    Tx(String),
    /// Durable commit failed.
    #[error("backend commit failure: {0}")]
    Commit(String),
    /// Backend codec conversion failed.
    #[error("backend codec error: {0}")]
    Codec(#[from] CodecError),
    /// Backend I/O setup failed.
    #[error("backend io error: {0}")]
    Io(#[from] IoError),
}
