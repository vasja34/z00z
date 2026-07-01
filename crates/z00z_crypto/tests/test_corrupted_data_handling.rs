use z00z_crypto::{
    aead::{build_aad_multipart, open, seal, MIN_ENVELOPE_SIZE, XCHACHA_KEY_SIZE},
    hashing::try_take_32,
    CryptoError, Z00ZScalar,
};

fn canonical_scalar_bytes() -> [u8; 32] {
    let mut bytes = [0u8; 32];
    bytes[31] = 3;
    bytes
}

#[test]
fn test_corrupted_data_bit_flips() {
    let bytes = canonical_scalar_bytes();
    assert!(Z00ZScalar::try_from_bytes(bytes).is_ok());

    for bit in 0..(32 * 8) {
        let mut corrupted = bytes;
        let byte_idx = bit / 8;
        let bit_idx = bit % 8;
        corrupted[byte_idx] ^= 1u8 << bit_idx;

        let _ = Z00ZScalar::try_from_bytes(corrupted);
    }

    let definitely_invalid = [0xFFu8; 32];
    assert!(matches!(
        Z00ZScalar::try_from_bytes(definitely_invalid),
        Err(CryptoError::InvalidParameters { .. })
    ));
}

#[test]
fn test_corrupted_data_truncated_hash() {
    for len in 0..32 {
        let input = vec![7u8; len];
        assert!(try_take_32(&input).is_none());
    }

    assert!(try_take_32([1u8; 32]).is_some());
    assert!(try_take_32([2u8; 64]).is_some());
}

#[test]
fn test_corrupted_data_malformed_envelope() {
    let key = [9u8; XCHACHA_KEY_SIZE];
    let aad = build_aad_multipart("z00z.tx", &[b"corruption-test"]).unwrap();
    let plaintext = b"transaction-bytes";

    let envelope = seal(&key, &aad, plaintext).expect("seal should work");

    let mut tampered = envelope.clone();
    let last = tampered.len() - 1;
    tampered[last] ^= 0x01;
    assert!(matches!(
        open(&key, &aad, &tampered),
        Err(CryptoError::CryptoOperationFailed)
    ));

    let truncated = &envelope[..(MIN_ENVELOPE_SIZE - 1)];
    assert!(matches!(
        open(&key, &aad, truncated),
        Err(CryptoError::InvalidParameters { .. })
    ));

    let mut wrong_algo = envelope;
    wrong_algo[0] = 0xFF;
    assert!(matches!(
        open(&key, &aad, &wrong_algo),
        Err(CryptoError::InvalidParameters { .. })
    ));
}
