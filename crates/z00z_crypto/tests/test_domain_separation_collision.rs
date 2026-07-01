//! Domain separation collision tests.
//!
//! These tests verify that the domain separation implementation
//! prevents hash collisions across different input structures.

#[test]
fn test_derive_hash_chunk_boundaries() {
    use z00z_crypto::derive_hash;

    // Chunk boundaries MUST be preserved
    let chunk1_a: &[u8] = b"ab";
    let chunk1_b: &[u8] = b"c";
    let h1 = derive_hash(b"test", &[chunk1_a, chunk1_b]);

    let chunk2_a: &[u8] = b"a";
    let chunk2_b: &[u8] = b"bc";
    let h2 = derive_hash(b"test", &[chunk2_a, chunk2_b]);

    assert_ne!(
        h1, h2,
        "Chunk boundaries must be preserved: ['ab','c'] != ['a','bc']"
    );
}

#[test]
#[should_panic(expected = "null bytes")]
fn test_dst_rejects_null_domain() {
    use z00z_crypto::hash::blake2b_256;

    // dst() is called internally, should panic on null bytes
    blake2b_256("user\0name", "label", &[b"data"]);
}

#[test]
#[should_panic(expected = "null bytes")]
fn test_dst_rejects_null_label() {
    use z00z_crypto::hash::blake2b_256;

    // dst() is called internally, should panic on null bytes
    blake2b_256("domain", "lab\0el", &[b"data"]);
}

#[test]
fn test_dst_no_collisions() {
    use z00z_crypto::hash::blake2b_256;

    // Domain/label combinations MUST NOT collide
    let h1 = blake2b_256("wallet", "key", &[]);
    let h2 = blake2b_256("wallet.key", "", &[]);

    assert_ne!(
        h1, h2,
        "Domain/label must not collide: ('wallet','key') != ('wallet.key','')"
    );
}

#[test]
fn test_derive_hash_empty_chunks() {
    use z00z_crypto::derive_hash;

    // Empty chunks should be properly length-prefixed
    let empty: &[u8] = b"";
    let data: &[u8] = b"data";
    let h1 = derive_hash(b"test", &[empty, data]);
    let h2 = derive_hash(b"test", &[data]);

    assert_ne!(
        h1, h2,
        "Empty chunks must be distinguished: ['','data'] != ['data']"
    );
}

#[test]
fn test_derive_hash_domain_sensitivity() {
    use z00z_crypto::derive_hash;

    // Different domains MUST produce different hashes
    let data: &[u8] = b"data";
    let h1 = derive_hash(b"domain1", &[data]);
    let h2 = derive_hash(b"domain2", &[data]);

    assert_ne!(h1, h2, "Different domains must produce different hashes");
}

#[test]
fn test_derive_accepts_invalid_utf8() {
    use z00z_crypto::derive_hash;

    // Invalid UTF-8 domain bytes are supported and must remain deterministic.
    let invalid_utf8 = &[0xFF, 0xFE, 0xFD];
    let data: &[u8] = b"data";
    let hash1 = derive_hash(invalid_utf8, &[data]);
    let hash2 = derive_hash(invalid_utf8, &[data]);

    assert_eq!(
        hash1, hash2,
        "invalid UTF-8 domain hashing must be deterministic"
    );
}

#[test]
fn test_context_preserves_proof_boundaries() {
    use z00z_crypto::range_ctx_hash;

    let asset_id = [0x11u8; 32];
    let commitment = [0x22u8; 32];
    let first = range_ctx_hash(&asset_id, 1, 1, 1, 1, &commitment, &[0xAA, 0xBB, 0xCC]);
    let second = range_ctx_hash(
        &asset_id,
        1,
        1,
        1,
        1,
        &commitment,
        &[0xAA, 0xBB, 0xCC, 0x00],
    );

    assert_ne!(
        first, second,
        "range_ctx_hash must preserve framed proof-byte boundaries"
    );
}
