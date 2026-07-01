//! Compression helpers.
//!
//! Used by wallet backup containers and streaming wallet-file workflows.
//!
//! Notes:
//! - This module centralizes compression dependencies to preserve the
//!   ONE SOURCE OF TRUTH principle.
//! - Z00Z currently uses Zstd as the default compressor, with optional LZ4
//!   support for low-latency use cases.

use std::{
    io::{Read, Write},
    path::Path,
};

const LZ4_FRAME_MAGIC_LE: [u8; 4] = [0x04, 0x22, 0x4D, 0x18];

fn is_lz4_frame(data: &[u8]) -> bool {
    data.len() >= 4 && data[..4] == LZ4_FRAME_MAGIC_LE
}

/// Compression error.
#[derive(Debug, thiserror::Error)]
pub enum CompressionError {
    /// Compression failed.
    #[error("compression failed: {0}")]
    CompressionFailed(String),
    /// Decompression failed.
    #[error("decompression failed: {0}")]
    DecompressionFailed(String),

    /// Decompressed output exceeded the configured limit.
    #[error("Decompressed output too large: {actual} bytes exceeds limit {limit} bytes")]
    OutputTooLarge {
        /// Actual bytes observed (lower bound).
        actual: usize,
        /// Maximum allowed decompressed bytes.
        limit: usize,
    },
}

/// Compress bytes using Zstd.
///
/// Uses Zstd's default compression level (implementation-defined).
pub fn zstd_compress(data: &[u8]) -> Result<Vec<u8>, CompressionError> {
    zstd::stream::encode_all(data, 0)
        .map_err(|e| CompressionError::CompressionFailed(e.to_string()))
}

/// Decompress Zstd-compressed bytes with a strict output size limit.
///
/// This prevents decompression bombs by ensuring the produced output never exceeds
/// `max_output_bytes`.
pub fn zstd_decompress_bounded(
    data: &[u8],
    max_output: usize,
) -> Result<Vec<u8>, CompressionError> {
    let mut decoder = zstd::stream::read::Decoder::new(data)
        .map_err(|e| CompressionError::DecompressionFailed(e.to_string()))?;

    let mut out = Vec::new();
    let mut limited = (&mut decoder).take(max_output as u64);
    limited
        .read_to_end(&mut out)
        .map_err(|e| CompressionError::DecompressionFailed(e.to_string()))?;

    let mut check_buf = [0u8; 1];
    let extra = decoder
        .read(&mut check_buf)
        .map_err(|e| CompressionError::DecompressionFailed(e.to_string()))?;
    if extra > 0 {
        return Err(CompressionError::OutputTooLarge {
            actual: out.len().saturating_add(extra),
            limit: max_output,
        });
    }

    Ok(out)
}

/// Compress bytes using LZ4.
pub fn lz4_compress(data: &[u8]) -> Result<Vec<u8>, CompressionError> {
    let mut encoder = lz4::EncoderBuilder::new()
        .build(Vec::new())
        .map_err(|e| CompressionError::CompressionFailed(e.to_string()))?;

    encoder
        .write_all(data)
        .map_err(|e| CompressionError::CompressionFailed(e.to_string()))?;

    let (out, result) = encoder.finish();
    result
        .map_err(|e| CompressionError::CompressionFailed(e.to_string()))
        .map(|()| out)
}

/// Decompress LZ4-compressed bytes with a strict output size limit.
pub fn lz4_decompress_bounded(data: &[u8], max_output: usize) -> Result<Vec<u8>, CompressionError> {
    if !is_lz4_frame(data) {
        return Err(CompressionError::DecompressionFailed(
            "unsupported lz4 format: expected framed lz4 stream".to_string(),
        ));
    }

    let mut decoder = lz4::Decoder::new(data)
        .map_err(|e| CompressionError::DecompressionFailed(e.to_string()))?;

    let mut output = Vec::new();
    let mut limited = (&mut decoder).take(max_output as u64);
    limited
        .read_to_end(&mut output)
        .map_err(|e| CompressionError::DecompressionFailed(e.to_string()))?;

    let mut check_buf = [0u8; 1];
    let extra = decoder
        .read(&mut check_buf)
        .map_err(|e| CompressionError::DecompressionFailed(e.to_string()))?;
    if extra != 0 {
        return Err(CompressionError::OutputTooLarge {
            actual: output.len().saturating_add(extra),
            limit: max_output,
        });
    }

    Ok(output)
}

/// Decompress a file as Zstd with a strict output limit.
pub fn decompress_file_bounded_zstd(
    path: impl AsRef<Path>,
    max_output: usize,
) -> Result<Vec<u8>, CompressionError> {
    const DEFAULT_MAX_COMPRESSED_SIZE: u64 = 100 * 1024 * 1024;

    let bytes = crate::io::read_file_bounded(path, DEFAULT_MAX_COMPRESSED_SIZE)
        .map_err(|e| CompressionError::DecompressionFailed(e.to_string()))?;
    zstd_decompress_bounded(&bytes, max_output)
}

/// Decompress a file as LZ4 with a strict output limit.
pub fn decompress_file_bounded_lz4(
    path: impl AsRef<Path>,
    max_output: usize,
) -> Result<Vec<u8>, CompressionError> {
    const DEFAULT_MAX_COMPRESSED_SIZE: u64 = 100 * 1024 * 1024;

    let bytes = crate::io::read_file_bounded(path, DEFAULT_MAX_COMPRESSED_SIZE)
        .map_err(|e| CompressionError::DecompressionFailed(e.to_string()))?;
    lz4_decompress_bounded(&bytes, max_output)
}

/// Stream-compress data from reader to writer using Zstd.
///
/// This is a streaming version of `zstd_compress` that doesn't allocate
/// the entire output in memory. Suitable for large files.
///
/// # Arguments
///
/// * `reader` - Source data to compress
/// * `writer` - Destination for compressed data
/// * `level` - Zstd compression level (0 = default, 1-22 = custom)
///
/// # Returns
///
/// `Ok(())` on success, `CompressionError` on failure.
///
/// # Example
///
/// ```
/// use std::io::Cursor;
/// use z00z_utils::compression::zstd_encode_to_writer;
///
/// let input = b"Hello, world!";
/// let mut output = Vec::new();
/// let mut reader = Cursor::new(&input[..]);
/// zstd_encode_to_writer(&mut reader, &mut output, 3).unwrap();
/// ```
pub fn zstd_encode_to_writer(
    reader: &mut impl Read,
    mut writer: impl Write,
    level: i32,
) -> Result<(), CompressionError> {
    let mut encoder = zstd::stream::Encoder::new(&mut writer, level)
        .map_err(|e| CompressionError::CompressionFailed(e.to_string()))?;

    std::io::copy(reader, &mut encoder)
        .map_err(|e| CompressionError::CompressionFailed(e.to_string()))?;

    encoder
        .finish()
        .map_err(|e| CompressionError::CompressionFailed(e.to_string()))?;

    Ok(())
}

/// Stream-decompress data from reader to writer with size bound.
///
/// This is a streaming version of `zstd_decompress_bounded` that doesn't
/// allocate the entire output in memory. Suitable for large files.
///
/// # Arguments
///
/// * `reader` - Compressed data source
/// * `writer` - Destination for decompressed data
/// * `max_output_bytes` - Maximum allowed decompressed size (DoS protection)
///
/// # Returns
///
/// `Ok(())` on success, `CompressionError` on failure.
///
/// # Example
///
/// ```
/// use std::io::Cursor;
/// use z00z_utils::compression::{zstd_encode_to_writer, zstd_decode_bounded_to_writer};
///
/// let input = b"Hello, world!";
/// let mut compressed = Vec::new();
/// let mut reader = Cursor::new(&input[..]);
/// zstd_encode_to_writer(&mut reader, &mut compressed, 3).unwrap();
///
/// let mut decompressed = Vec::new();
/// let mut comp_reader = Cursor::new(&compressed[..]);
/// zstd_decode_bounded_to_writer(&mut comp_reader, &mut decompressed, 1000).unwrap();
/// assert_eq!(decompressed, input);
/// ```
pub fn zstd_decode_bounded_to_writer(
    reader: &mut impl Read,
    mut writer: impl Write,
    max_output_bytes: usize,
) -> Result<(), CompressionError> {
    let mut decoder = zstd::stream::read::Decoder::new(reader)
        .map_err(|e| CompressionError::DecompressionFailed(e.to_string()))?;

    let mut bytes_written = 0usize;
    let mut buffer = [0u8; 8192]; // 8KB buffer for streaming

    loop {
        let bytes_read = decoder
            .read(&mut buffer)
            .map_err(|e| CompressionError::DecompressionFailed(e.to_string()))?;

        if bytes_read == 0 {
            break;
        }

        bytes_written = bytes_written.checked_add(bytes_read).ok_or_else(|| {
            CompressionError::DecompressionFailed("output size overflow".to_string())
        })?;

        if bytes_written > max_output_bytes {
            return Err(CompressionError::OutputTooLarge {
                actual: bytes_written,
                limit: max_output_bytes,
            });
        }

        writer
            .write_all(&buffer[..bytes_read])
            .map_err(|e| CompressionError::DecompressionFailed(e.to_string()))?;
    }

    Ok(())
}

#[cfg(test)]
mod test_compression;
