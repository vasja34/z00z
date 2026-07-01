use super::*;
use crate::CryptoError;

#[test]
fn test_encrypt_decrypt_roundtrip() {
    let key = Key::from_slice(&[0x42u8; 32]);
    let nonce = random_nonce().unwrap();
    let aad = b"test aad";
    let plaintext = b"secret message";

    let ciphertext = xchacha20poly1305_encrypt(key, &nonce, aad, plaintext).unwrap();
    let decrypted = xchacha20poly1305_decrypt(key, &nonce, aad, &ciphertext).unwrap();

    assert_eq!(decrypted, plaintext);
}

#[test]
fn test_encrypt_random_nonce_helper() {
    let key = Key::from_slice(&[0x42u8; 32]);
    let aad = b"test aad";
    let plaintext = b"secret message";

    let (nonce, ciphertext) = encrypt_with_random_nonce(key, aad, plaintext).unwrap();
    assert_eq!(nonce.len(), 24);
    assert!(ciphertext.len() >= 16);

    let decrypted = xchacha20poly1305_decrypt(key, &nonce, aad, &ciphertext).unwrap();
    assert_eq!(decrypted, plaintext);
}

#[test]
fn test_nonces_produce_different_ciphertexts() {
    let key = Key::from_slice(&[0x42u8; 32]);
    let aad = b"test aad";
    let plaintext = b"secret message";

    let nonce1 = random_nonce().unwrap();
    let nonce2 = random_nonce().unwrap();

    let ciphertext1 = xchacha20poly1305_encrypt(key, &nonce1, aad, plaintext).unwrap();
    let ciphertext2 = xchacha20poly1305_encrypt(key, &nonce2, aad, plaintext).unwrap();

    assert_ne!(ciphertext1, ciphertext2);
}

#[test]
fn test_wrong_aad_fails() {
    let key = Key::from_slice(&[0x42u8; 32]);
    let nonce = random_nonce().unwrap();
    let plaintext = b"secret message";

    let ciphertext = xchacha20poly1305_encrypt(key, &nonce, b"correct aad", plaintext).unwrap();
    let result = xchacha20poly1305_decrypt(key, &nonce, b"wrong aad", &ciphertext);
    assert!(result.is_err());
}

#[test]
fn test_seal_open_roundtrip() {
    let key = [7u8; XCHACHA_KEY_SIZE];
    let aad = build_aad_multipart("test.domain", &[b"ctx"]).unwrap();
    let plaintext = b"hello";

    let envelope = seal(&key, &aad, plaintext).unwrap();
    assert!(envelope.len() >= MIN_ENVELOPE_SIZE);
    assert_eq!(envelope[0], XCHACHA20_POLY1305_ID);

    let recovered = open(&key, &aad, &envelope).unwrap();
    assert_eq!(recovered, plaintext);
}

#[test]
fn test_open_rejects_wrong_aad() {
    let key = [7u8; XCHACHA_KEY_SIZE];
    let aad1 = build_aad_multipart("test.domain", &[b"ctx1"]).unwrap();
    let aad2 = build_aad_multipart("test.domain", &[b"ctx2"]).unwrap();
    let plaintext = b"hello";

    let envelope = seal(&key, &aad1, plaintext).unwrap();
    let err = open(&key, &aad2, &envelope).unwrap_err();
    assert!(matches!(err, CryptoError::CryptoOperationFailed));
}

#[test]
fn test_open_rejects_too_short() {
    let key = [7u8; XCHACHA_KEY_SIZE];
    let aad = build_aad_multipart("test.domain", &[b"ctx"]).unwrap();

    let err = open(&key, &aad, &[]).unwrap_err();
    assert!(matches!(err, CryptoError::InvalidParameters { .. }));

    let short_envelope = vec![0u8; MIN_ENVELOPE_SIZE - 1];
    let err = open(&key, &aad, &short_envelope).unwrap_err();
    assert!(matches!(err, CryptoError::InvalidParameters { .. }));
}

#[test]
fn test_open_rejects_wrong_algorithm() {
    let key = [7u8; XCHACHA_KEY_SIZE];
    let aad = build_aad_multipart("test.domain", &[b"ctx"]).unwrap();
    let plaintext = b"hello";

    let mut envelope = seal(&key, &aad, plaintext).unwrap();
    envelope[0] = 0xFF;

    let err = open(&key, &aad, &envelope).unwrap_err();
    assert!(matches!(err, CryptoError::InvalidParameters { .. }));
}

#[test]
fn test_tampered_envelope_detected() {
    let key = [7u8; XCHACHA_KEY_SIZE];
    let aad = build_aad_multipart("test.domain", &[b"ctx"]).unwrap();
    let plaintext = b"hello";

    let envelope = seal(&key, &aad, plaintext).unwrap();
    let mut tampered = envelope.clone();
    let tag_offset = tampered.len() - 16;
    tampered[tag_offset] ^= 0x01;

    let err = open(&key, &aad, &tampered).unwrap_err();
    assert!(matches!(err, CryptoError::CryptoOperationFailed));
}

#[test]
fn test_empty_plaintext_works() {
    let key = [7u8; XCHACHA_KEY_SIZE];
    let aad = build_aad_multipart("test.domain", &[b"ctx"]).unwrap();
    let plaintext = b"";

    let envelope = seal(&key, &aad, plaintext).unwrap();
    let recovered = open(&key, &aad, &envelope).unwrap();

    assert_eq!(recovered.len(), 0);
    assert_eq!(recovered, plaintext);
}

#[test]
fn test_large_plaintext_works() {
    let key = [7u8; XCHACHA_KEY_SIZE];
    let aad = build_aad_multipart("test.domain", &[b"ctx"]).unwrap();
    let plaintext = vec![42u8; 10000];

    let envelope = seal(&key, &aad, &plaintext).unwrap();
    let recovered = open(&key, &aad, &envelope).unwrap();

    assert_eq!(recovered, plaintext);
}

#[test]
fn test_seal_rejects_oversized_plaintext() {
    let key = [0u8; XCHACHA_KEY_SIZE];
    let huge_plaintext = vec![0u8; MAX_AEAD_PLAINTEXT_SIZE + 1];
    let result = seal(&key, b"", &huge_plaintext);

    assert!(result.is_err());
    match result.unwrap_err() {
        CryptoError::InvalidParameters { param } => {
            assert!(param.contains("plaintext_size"));
        }
        _ => panic!("Expected InvalidParameters error"),
    }
}

#[test]
fn test_seal_rejects_oversized_aad() {
    let key = [0u8; XCHACHA_KEY_SIZE];
    let plaintext = b"test";
    let huge_aad = vec![0u8; MAX_AAD_SIZE + 1];
    let result = seal(&key, &huge_aad, plaintext);

    assert!(result.is_err());
    assert!(matches!(
        result.unwrap_err(),
        CryptoError::AadTooLarge {
            limit: MAX_AAD_SIZE,
            ..
        }
    ));
}

#[test]
fn test_open_rejects_oversized_envelope() {
    let key = [0u8; XCHACHA_KEY_SIZE];
    let huge_envelope = vec![0u8; MAX_AEAD_ENVELOPE_SIZE + 1];
    let result = open(&key, b"", &huge_envelope);

    assert!(result.is_err());
    match result.unwrap_err() {
        CryptoError::InvalidParameters { param } => {
            assert!(param.contains("envelope_size"));
        }
        _ => panic!("Expected InvalidParameters error"),
    }
}

#[test]
fn test_open_rejects_truncated_envelope() {
    let key = [0u8; XCHACHA_KEY_SIZE];
    let truncated = vec![0u8; MIN_ENVELOPE_SIZE - 1];
    let result = open(&key, b"", &truncated);

    assert!(result.is_err());
    match result.unwrap_err() {
        CryptoError::InvalidParameters { param } => {
            assert!(param.contains("envelope"));
        }
        _ => panic!("Expected InvalidParameters error"),
    }
}

#[test]
fn test_open_rejects_oversized_aad() {
    let key = [0u8; XCHACHA_KEY_SIZE];
    let plaintext = b"test";
    let aad = b"small";

    let envelope = seal(&key, aad, plaintext).unwrap();
    let huge_aad = vec![0u8; MAX_AAD_SIZE + 1];
    let result = open(&key, &huge_aad, &envelope);

    assert!(result.is_err());
    assert!(matches!(
        result.unwrap_err(),
        CryptoError::AadTooLarge {
            limit: MAX_AAD_SIZE,
            ..
        }
    ));
}

#[test]
fn test_aad_multipart_no_collision() {
    let aad1 = build_aad_multipart("test.domain", &[b"ab", b"cd"]).unwrap();
    let aad2 = build_aad_multipart("test.domain", &[b"a", b"bcd"]).unwrap();
    assert_ne!(aad1, aad2, "Length framing must prevent collisions");
}

#[test]
fn test_aad_size_limits() {
    let big = vec![0u8; MAX_AAD_SIZE + 1024];
    let err = build_aad_multipart("test.domain", &[big.as_slice()]).unwrap_err();
    assert!(matches!(err, CryptoError::AadTooLarge { .. }));

    let aad_ext = build_aad_multipart_extended("test.domain", &[big.as_slice()]).unwrap();
    let key = [7u8; XCHACHA_KEY_SIZE];
    assert!(matches!(
        seal(&key, &aad_ext, b"hi").unwrap_err(),
        CryptoError::AadTooLarge {
            limit: MAX_AAD_SIZE,
            ..
        }
    ));

    let envelope = seal_extended_aad(&key, &aad_ext, b"hi").unwrap();
    assert!(matches!(
        open(&key, &aad_ext, &envelope).unwrap_err(),
        CryptoError::AadTooLarge {
            limit: MAX_AAD_SIZE,
            ..
        }
    ));

    let recovered = open_extended_aad(&key, &aad_ext, &envelope).unwrap();
    assert_eq!(recovered, b"hi");
}

#[test]
fn test_seal_accepts_max_plaintext() {
    let key = [0u8; XCHACHA_KEY_SIZE];
    let max_plaintext = vec![0u8; MAX_AEAD_PLAINTEXT_SIZE];
    let result = seal(&key, b"", &max_plaintext);

    assert!(result.is_ok());
}

#[test]
fn test_open_accepts_max_envelope() {
    let key = [0u8; XCHACHA_KEY_SIZE];
    let plaintext = b"test";
    let envelope = seal(&key, b"", plaintext).unwrap();

    assert!(envelope.len() <= MAX_AEAD_ENVELOPE_SIZE);
    let result = open(&key, b"", &envelope);
    assert!(result.is_ok());
}

#[test]
fn test_error_sensitive_data_leak() {
    let errors = vec![AeadError::Crypto, AeadError::Random];

    for err in errors {
        let msg = err.to_string();
        assert!(!msg.contains("tag"), "Error leaks crypto detail: {}", msg);
        assert!(
            !msg.contains("verification"),
            "Error leaks crypto detail: {}",
            msg
        );
        assert!(!msg.contains("MAC"), "Error leaks crypto detail: {}", msg);
        assert!(
            !msg.contains("authentication"),
            "Error leaks crypto detail: {}",
            msg
        );
        assert!(
            !msg.contains("dev/urandom"),
            "Error leaks platform detail: {}",
            msg
        );
        assert!(
            !msg.contains("getrandom"),
            "Error leaks platform detail: {}",
            msg
        );
        assert!(
            !msg.contains("BCrypt"),
            "Error leaks platform detail: {}",
            msg
        );
        assert!(
            !msg.contains("SecRandom"),
            "Error leaks platform detail: {}",
            msg
        );
        assert!(!msg.contains("0x"), "Error contains hex value: {}", msg);
        assert!(msg.len() < 80, "Error message too verbose: {}", msg);
        assert!(!msg.is_empty(), "Error message is empty");
    }
}

#[test]
fn test_crypto_error_chain_safety() {
    use crate::error::CryptoError;

    let key1 = [0u8; XCHACHA_KEY_SIZE];
    let key2 = [1u8; XCHACHA_KEY_SIZE];
    let envelope = seal(&key1, b"aad", b"plaintext").unwrap();
    let result = open(&key2, b"aad", &envelope);

    if let Err(CryptoError::CryptoOperationFailed) = result {
        let msg = format!("{:?}", result);
        assert!(
            !msg.contains("tag"),
            "Error debug leaks crypto detail: {}",
            msg
        );
        assert!(
            !msg.contains("auth"),
            "Error debug leaks crypto detail: {}",
            msg
        );
    } else {
        panic!("Expected CryptoOperationFailed");
    }
}

#[test]
fn test_rng_failure_error_type() {
    use crate::error::CryptoError;

    let rng_error = CryptoError::RngFailure;

    match rng_error {
        CryptoError::RngFailure => {}
        CryptoError::CryptoOperationFailed => {
            panic!("RngFailure should NOT be generic CryptoOperationFailed");
        }
        _ => {
            panic!("Unexpected error variant");
        }
    }

    let msg = format!("{}", rng_error);
    let msg_lower = msg.to_lowercase();
    assert!(
        msg_lower.contains("rng") || msg_lower.contains("random"),
        "Error message should mention RNG/random: {}",
        msg
    );
}

#[cfg(not(target_arch = "wasm32"))]
#[test]
fn test_seal_with_mock_rng() {
    use z00z_utils::rng::MockRngProvider;

    let provider = MockRngProvider::with_u64_seed(42);
    let mut rng = provider.rng();

    let key = [1u8; 32];
    let plaintext = b"test message";
    let aad = b"additional data";

    let ct1 = seal_with_rng(&mut rng, &key, aad, plaintext).unwrap();

    let provider2 = MockRngProvider::with_u64_seed(42);
    let mut rng2 = provider2.rng();
    let ct2 = seal_with_rng(&mut rng2, &key, aad, plaintext).unwrap();

    assert_eq!(ct1, ct2, "Deterministic with same seed");
}

#[cfg(not(target_arch = "wasm32"))]
#[test]
fn test_random_nonce_with_mock() {
    use z00z_utils::rng::MockRngProvider;

    let provider = MockRngProvider::with_u64_seed(42);
    let mut rng = provider.rng();
    let nonce1 = random_nonce_with_rng(&mut rng).unwrap();

    let provider2 = MockRngProvider::with_u64_seed(42);
    let mut rng2 = provider2.rng();
    let nonce2 = random_nonce_with_rng(&mut rng2).unwrap();

    assert_eq!(nonce1.as_slice(), nonce2.as_slice());
}

#[cfg(not(target_arch = "wasm32"))]
#[test]
fn test_seal_decrypt_rng() {
    use z00z_utils::rng::MockRngProvider;

    let provider = MockRngProvider::with_u64_seed(123);
    let mut rng = provider.rng();

    let key = [42u8; 32];
    let plaintext = b"secret data";
    let aad = b"context info";

    let envelope = seal_with_rng(&mut rng, &key, aad, plaintext).unwrap();
    let decrypted = open(&key, aad, &envelope).unwrap();

    assert_eq!(decrypted, plaintext);
}
