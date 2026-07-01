use super::*;
use crate::domains::hashing::{
    redb_wallet_hkdf_info_data, redb_wallet_hkdf_info_index, redb_wallet_hkdf_info_integrity,
};

#[test]
fn test_domain_identifiers_are_stable() {
    let data_info1 = redb_wallet_hkdf_info_data();
    let data_info2 = redb_wallet_hkdf_info_data();
    assert_eq!(
        data_info1, data_info2,
        "Data key info should be deterministic"
    );

    let index_info1 = redb_wallet_hkdf_info_index();
    let index_info2 = redb_wallet_hkdf_info_index();
    assert_eq!(
        index_info1, index_info2,
        "Index key info should be deterministic"
    );

    let integrity_info1 = redb_wallet_hkdf_info_integrity();
    let integrity_info2 = redb_wallet_hkdf_info_integrity();
    assert_eq!(
        integrity_info1, integrity_info2,
        "Integrity key info should be deterministic"
    );

    assert_ne!(
        data_info1, index_info1,
        "Data and index domains should differ"
    );
    assert_ne!(
        data_info1, integrity_info1,
        "Data and integrity domains should differ"
    );
    assert_ne!(
        index_info1, integrity_info1,
        "Index and integrity domains should differ"
    );

    assert_eq!(AAD_MASTER_KEY_LABEL, b"master-key:v1");
    assert_eq!(AAD_SECRET_PREFIX, b"secret:v1:");
}

#[test]
fn test_aead_roundtrip_and_tamper() {
    use z00z_crypto::aead::open;
    use z00z_crypto::aead::test_only::seal_with_nonce_TEST_ONLY;

    let key = [9u8; 32];
    let nonce = [7u8; 24];
    let wallet_id = b"wallet_test";
    let aad = aad_master_key(wallet_id);
    let plaintext = b"secret";

    let envelope_bytes = seal_with_nonce_TEST_ONLY(&key, &aad, plaintext, nonce).unwrap();
    let env = AeadEnvelope {
        envelope: envelope_bytes,
    };

    let recovered = open(&key, &aad, &env.envelope).unwrap();
    assert_eq!(recovered, plaintext);
    assert!(env.envelope.len() >= 1 + 24 + 16);
    assert_eq!(env.envelope[0], z00z_crypto::aead::XCHACHA20_POLY1305_ID);
    assert!(env.is_xchacha20poly1305());

    let mut tampered = env.clone();
    let tag_offset = tampered.envelope.len() - 16;
    tampered.envelope[tag_offset] ^= 0x01;

    let err = open(&key, &aad, &tampered.envelope).unwrap_err();
    assert!(matches!(err, CryptoError::CryptoOperationFailed));
}

#[test]
fn test_hkdf_derivation_is_deterministic() {
    let master = [3u8; 32];
    let keys1 = derive_wallet_keys(&master).unwrap();
    let keys2 = derive_wallet_keys(&master).unwrap();

    assert_eq!(keys1.data_key.reveal(), keys2.data_key.reveal());
    assert_eq!(keys1.index_key.reveal(), keys2.index_key.reveal());
    assert_eq!(keys1.integrity_key.reveal(), keys2.integrity_key.reveal());
}

#[test]
fn test_rejects_params_unknown_version() {
    let mut params = KdfParams::default_argon2id_with_salt(vec![7u8; 16]);
    params.version = 999;

    let err = params.validate_untrusted_persisted().unwrap_err();
    assert!(matches!(err, CryptoError::InvalidParameters { .. }));
}

#[test]
fn test_salt32_zero_padding() {
    let salt_16 = [0xAAu8; 16];
    let padded = super::wallet_store_crypto_kdf::pad_salt32_zero(&salt_16);
    assert_eq!(&padded[..16], &salt_16);
    assert_eq!(&padded[16..], &[0u8; 16]);
}

#[test]
fn test_salt32_zero_padding_boundaries() {
    let empty: [u8; 0] = [];
    assert_eq!(
        super::wallet_store_crypto_kdf::pad_salt32_zero(&empty),
        [0u8; 32]
    );

    let salt_1 = [0x11u8; 1];
    let padded_1 = super::wallet_store_crypto_kdf::pad_salt32_zero(&salt_1);
    assert_eq!(padded_1[0], 0x11);
    assert_eq!(&padded_1[1..], &[0u8; 31]);

    let salt_31 = [0x22u8; 31];
    let padded_31 = super::wallet_store_crypto_kdf::pad_salt32_zero(&salt_31);
    assert_eq!(&padded_31[..31], &salt_31);
    assert_eq!(padded_31[31], 0);

    let salt_32 = [0x33u8; 32];
    assert_eq!(
        super::wallet_store_crypto_kdf::pad_salt32_zero(&salt_32),
        salt_32
    );

    let salt_33 = [0x44u8; 33];
    let padded_33 = super::wallet_store_crypto_kdf::pad_salt32_zero(&salt_33);
    assert_eq!(&padded_33[..], &salt_33[..32]);
}

#[test]
fn test_rejects_old_kdf_version() {
    let password = SafePassword::from("pw1");

    let mut old_params = KdfParams::default_argon2id_with_salt(vec![7u8; 16]);
    old_params.version = 1;

    let err = derive_pw_key(&password, &old_params).unwrap_err();
    assert!(matches!(err, CryptoError::InvalidParameters { .. }));
}

#[test]
fn test_current_kdf_zero_padding() {
    let password = SafePassword::from("pw1");
    let params = KdfParams::default_argon2id_with_salt(vec![7u8; 16]);
    let argon2_params = Argon2Params {
        memory: (params.mem_limit / 1024) as u32,
        iterations: params.ops_limit,
        parallelism: params.parallelism,
    };

    let direct =
        super::wallet_store_crypto_kdf::derive_key_zero_padding(&password, &params, &argon2_params)
            .expect("direct current lane");
    let derived = derive_pw_key(&password, &params).expect("derive current lane");

    assert_eq!(derived.reveal(), &direct);
    assert_eq!(params.version, KdfParams::VERSION);
}

#[test]
fn test_rejects_params_untrusted_validation() {
    let mut params = KdfParams::default_argon2id_with_salt(vec![7u8; 16]);
    params.mem_limit = (MAX_MEM_LIMIT_KIB as u64 + 1) * 1024;

    let err = params.validate_untrusted_persisted().unwrap_err();
    assert!(matches!(err, CryptoError::InvalidParameters { .. }));
}

#[test]
fn test_rejects_high_ops_limit() {
    let mut params = KdfParams::default_argon2id_with_salt(vec![7u8; 16]);
    params.ops_limit = MAX_OPS_LIMIT + 1;

    let err = params.validate_untrusted_persisted().unwrap_err();
    assert!(matches!(err, CryptoError::InvalidParameters { .. }));
}

#[test]
fn test_rejects_kdf_params_untrusted() {
    let mut params = KdfParams::default_argon2id_with_salt(vec![7u8; 16]);
    params.parallelism = MAX_PARALLELISM + 1;

    let err = params.validate_untrusted_persisted().unwrap_err();
    assert!(matches!(err, CryptoError::InvalidParameters { .. }));
}

#[test]
fn test_rejects_low_mem_limit() {
    let mut params = KdfParams::default_argon2id_with_salt(vec![7u8; 16]);
    params.mem_limit = (MIN_MEM_LIMIT_KIB as u64 - 1) * 1024;

    let err = params.validate_untrusted_persisted().unwrap_err();
    assert!(matches!(err, CryptoError::InvalidParameters { .. }));
}

#[test]
fn test_rejects_low_ops_limit() {
    let mut params = KdfParams::default_argon2id_with_salt(vec![7u8; 16]);
    params.ops_limit = MIN_OPS_LIMIT - 1;

    let err = params.validate_untrusted_persisted().unwrap_err();
    assert!(matches!(err, CryptoError::InvalidParameters { .. }));
}

#[test]
fn test_derive_pw_key_current() {
    let password = SafePassword::from("pw1");
    let params = KdfParams::default_argon2id_with_salt(vec![7u8; 16]);

    let key1 = derive_pw_key(&password, &params).unwrap();
    let key2 = derive_pw_key(&password, &params).unwrap();

    assert_eq!(key1.reveal(), key2.reveal());
}

#[test]
fn test_aad_collision_prevention() {
    let wallet_id = [0u8; 16];

    let aad1 = aad_secret(&wallet_id, "ab:c");
    let aad2 = aad_secret(&wallet_id, "ab");
    let aad3 = aad_secret(&wallet_id, "ab\0c");

    assert_ne!(aad1, aad2);
    assert_ne!(aad1, aad3);
    assert_ne!(aad2, aad3);
}
#[cfg(not(target_arch = "wasm32"))]
#[test]
fn test_proptest_aad_distinct() {
    use proptest::prelude::*;

    let wallet_id = [11u8; 16];
    let alphabet = prop_oneof![Just('a'), Just('b'), Just('c'), Just(':'), Just('\0')];

    proptest!(|(a in prop::collection::vec(alphabet.clone(), 0..32), b in prop::collection::vec(alphabet, 0..32))| {
        let a: String = a.into_iter().collect();
        let b: String = b.into_iter().collect();
        prop_assume!(a != b);

        prop_assert_ne!(aad_secret(&wallet_id, &a), aad_secret(&wallet_id, &b));
    });
}
