use super::*;

#[test]
fn test_zstd_decompress_bomb_rejected() {
    let bomb_decompressed = vec![0u8; 100_000_000]; // 100MB
    let bomb_compressed = zstd_compress(&bomb_decompressed).unwrap();

    let result = zstd_decompress_bounded(&bomb_compressed, 10_000_000);
    assert!(matches!(
        result,
        Err(CompressionError::OutputTooLarge { .. })
    ));
}

#[test]
fn test_lz4_bomb_rejected() {
    let bomb_decompressed = vec![0u8; 100_000_000]; // 100MB
    let bomb_compressed = lz4_compress(&bomb_decompressed).unwrap();

    let result = lz4_decompress_bounded(&bomb_compressed, 10_000_000);
    assert!(matches!(
        result,
        Err(CompressionError::OutputTooLarge { .. })
    ));
}

#[test]
fn test_bounded_decompress_valid() {
    let payload = b"hello world".repeat(1024);

    let zstd = zstd_compress(&payload).unwrap();
    let zstd_out = zstd_decompress_bounded(&zstd, 1024 * 1024).unwrap();
    assert_eq!(zstd_out, payload);

    let lz4 = lz4_compress(&payload).unwrap();
    let lz4_out = lz4_decompress_bounded(&lz4, 1024 * 1024).unwrap();
    assert_eq!(lz4_out, payload);
}

#[test]
fn test_lz4_framed_payload_rejected() {
    let payload = b"hello world".repeat(1024);
    let framed_payload = lz4_flex::compress_prepend_size(&payload);

    let out = lz4_decompress_bounded(&framed_payload, 1024 * 1024);
    assert!(matches!(out, Err(CompressionError::DecompressionFailed(_))));
}
