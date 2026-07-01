use z00z_crypto::{try_hash_to_scalar_domain, try_hmac_sha256, try_hmac_sha256_raw, Z00ZScalar};

#[test]
fn test_try_hash_non_zero() {
    let scalar_a = try_hash_to_scalar_domain(b"phase25", &[b"claim", b"v2"]).unwrap();
    let scalar_b = try_hash_to_scalar_domain(b"phase25", &[b"claim", b"v2"]).unwrap();

    assert_eq!(scalar_a.to_bytes(), scalar_b.to_bytes());
    assert!(!scalar_a.is_zero());
}

#[test]
fn test_try_hmac_nonzero_outputs() {
    let key = b"wallet-key";
    let msg = b"wallet-message";

    let mac_a = try_hmac_sha256(key, "wallet.auth", "v1", msg).unwrap();
    let mac_b = try_hmac_sha256_raw(key, msg).unwrap();

    assert_ne!(mac_a, [0u8; 32]);
    assert_ne!(mac_b, [0u8; 32]);
}

#[test]
fn test_try_from_non_zero() {
    let seed = [0x11u8; 64];
    let scalar_a = Z00ZScalar::try_from_hash(&seed).unwrap();
    let scalar_b = Z00ZScalar::try_from_hash(&seed).unwrap();

    assert_eq!(scalar_a.to_bytes(), scalar_b.to_bytes());
    assert!(!scalar_a.is_zero());
}
