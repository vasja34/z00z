//! Codec trait and error types for serialization/deserialization

use serde::{de::DeserializeOwned, Serialize};
use thiserror::Error;

/// Error types for codec operations
#[derive(Debug, Error)]
pub enum CodecError {
    /// JSON serialization/deserialization error
    #[error("JSON codec error: {0}")]
    Json(String),

    /// Bincode serialization/deserialization error
    #[error("Bincode codec error: {0}")]
    Bincode(String),

    /// YAML serialization/deserialization error
    #[error("YAML codec error: {0}")]
    Yaml(String),

    /// Deserialization size limit exceeded
    #[error("Deserialization size limit exceeded: {size} bytes > {limit} bytes")]
    DeserializeSizeLimitExceeded {
        /// Actual input size in bytes.
        size: usize,
        /// Maximum allowed size in bytes.
        limit: u64,
    },

    /// Trailing bytes after deserialization
    #[error("Trailing bytes after deserialization: {consumed} bytes consumed, {total} total")]
    TrailingBytes {
        /// Number of bytes consumed by the deserializer.
        consumed: usize,
        /// Total number of bytes provided as input.
        total: usize,
    },

    /// I/O error
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
}

/// Codec trait for serializing and deserializing data
///
/// Provides a common interface for different serialization formats.
/// Implementations should handle specific format details transparently.
///
/// # Examples
///
/// ```no_run
/// use z00z_utils::codec::{Codec, JsonCodec};
/// use serde::{Serialize, Deserialize};
///
/// #[derive(Serialize, Deserialize, Debug, PartialEq)]
/// struct Config {
///     name: String,
///     port: u16,
/// }
///
/// let codec = JsonCodec;
/// let config = Config { name: "server".into(), port: 8080 };
///
/// // Serialize
/// let bytes = codec.serialize(&config)?;
///
/// // Deserialize
/// let decoded: Config = codec.deserialize(&bytes)?;
/// assert_eq!(config, decoded);
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
pub trait Codec {
    /// Error type for this codec
    type Error: std::error::Error + Send + Sync + 'static;

    /// Serialize a value to bytes
    fn serialize<T: Serialize>(&self, value: &T) -> Result<Vec<u8>, Self::Error>;

    /// Deserialize bytes to a value
    fn deserialize<T: DeserializeOwned>(&self, bytes: &[u8]) -> Result<T, Self::Error>;

    /// Serialize a value to formatted/pretty-printed bytes
    /// Default implementation delegates to serialize()
    fn serialize_pretty<T: Serialize>(&self, value: &T) -> Result<Vec<u8>, Self::Error> {
        self.serialize(value)
    }

    /// Get the name of this codec (e.g., "json", "bincode", "yaml")
    fn name(&self) -> &'static str;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_codec_error_display() {
        let err = CodecError::Json("test error".to_string());
        assert_eq!(err.to_string(), "JSON codec error: test error");

        let err = CodecError::Bincode("test error".to_string());
        assert_eq!(err.to_string(), "Bincode codec error: test error");

        let err = CodecError::Yaml("test error".to_string());
        assert_eq!(err.to_string(), "YAML codec error: test error");

        let err = CodecError::DeserializeSizeLimitExceeded {
            size: 11,
            limit: 10,
        };
        assert_eq!(
            err.to_string(),
            "Deserialization size limit exceeded: 11 bytes > 10 bytes"
        );

        let err = CodecError::TrailingBytes {
            consumed: 1,
            total: 2,
        };
        assert_eq!(
            err.to_string(),
            "Trailing bytes after deserialization: 1 bytes consumed, 2 total"
        );
    }

    #[test]
    fn test_codec_error_from_io() {
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
        let codec_err: CodecError = io_err.into();
        match codec_err {
            CodecError::Io(_) => {} // Expected
            _ => panic!("Expected Io error"),
        }
    }
}
