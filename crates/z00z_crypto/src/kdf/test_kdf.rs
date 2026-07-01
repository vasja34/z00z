use super::*;
use crate::kdf::kdf_domains;
use zeroize::Zeroize;

#[test]
fn test_production_presets_secure() {
    let moderate = Argon2Params::moderate();
    assert!(moderate.memory >= 64 * 1024, "Memory too low");
    assert!(moderate.iterations >= 3, "Iterations too low");

    let strong = Argon2Params::strong();
    assert!(strong.memory >= 64 * 1024, "Memory too low");
    assert!(strong.iterations >= 3, "Iterations too low");
}

#[test]
fn test_argon2id_32() {
    let password = b"test_password";
    let salt = [42u8; 32];
    let params = Argon2Params::test_fast();
    let key = derive_argon2id32_key(password, &salt, &params).unwrap();
    assert_eq!(key.reveal().len(), 32);
}

#[test]
fn test_hkdf_expand_32() {
    let ikm = b"ikm";
    let salt = b"salt";
    let info = b"info";
    let key = hkdf_expand_32(ikm, salt, info).unwrap();
    assert_eq!(key.reveal().len(), 32);
}

#[test]
fn test_argon2id_deterministic() {
    let password = b"password";
    let salt = [1u8; 32];
    let params = Argon2Params::test_fast();

    let key1 = derive_argon2id32_key(password, &salt, &params).unwrap();
    let key2 = derive_argon2id32_key(password, &salt, &params).unwrap();

    assert_eq!(
        key1.reveal(),
        key2.reveal(),
        "Argon2id should be deterministic"
    );
}

#[test]
fn test_hkdf_different_info() {
    let ikm = b"ikm";
    let salt = b"salt";

    let key1 = hkdf_expand_32(ikm, salt, b"info1").unwrap();
    let key2 = hkdf_expand_32(ikm, salt, b"info2").unwrap();

    assert_ne!(
        key1.reveal(),
        key2.reveal(),
        "Different info should produce different keys"
    );
}

#[test]
fn test_hkdf_empty_salt() {
    let ikm = [0xAA; 32];
    let result = hkdf_expand_32(&ikm, &[], b"test");
    assert!(result.is_ok());
    assert_eq!(result.unwrap().reveal().len(), 32);
}

#[test]
fn test_hkdf_with_salt() {
    let ikm = b"low entropy";
    let salt = b"random-salt";
    let result = hkdf_expand_32(ikm, salt, b"test");
    assert!(result.is_ok());
    assert_eq!(result.unwrap().reveal().len(), 32);
}

#[test]
fn test_hkdf_rejects_empty_info() {
    let ikm = [0xAA; 32];
    let salt = [0xBB; 16];

    let result = hkdf_expand_32(&ikm, &salt, b"");
    assert!(matches!(result, Err(KdfError::HkdfInfoEmpty)));
}

#[test]
fn test_hkdf_version_separation() {
    let ikm = [0xAA; 32];
    let salt = [0xBB; 16];

    let key_1 = hkdf_expand_32(&ikm, &salt, kdf_domains::HKDF_INFO_WALLET_ENCRYPTION).unwrap();
    let key_2 = hkdf_expand_32(&ikm, &salt, b"z00z.wallet.encryption.v2").unwrap();

    assert_ne!(key_1.reveal(), key_2.reveal());
}

#[test]
fn test_hkdf_salt_affects_output() {
    let ikm = [0xBB; 32];
    let key1 = hkdf_expand_32(&ikm, b"salt1", b"info").unwrap();
    let key2 = hkdf_expand_32(&ikm, b"salt2", b"info").unwrap();
    assert_ne!(
        key1.reveal(),
        key2.reveal(),
        "Different salts should produce different keys"
    );
}

#[test]
fn test_hkdf_salt_vs_none() {
    let ikm = [0xCC; 32];
    let key_no_salt = hkdf_expand_32(&ikm, &[], b"info").unwrap();
    let key_with_salt = hkdf_expand_32(&ikm, b"some-salt", b"info").unwrap();
    assert_ne!(
        key_no_salt.reveal(),
        key_with_salt.reveal(),
        "Empty salt vs actual salt should produce different keys"
    );
}

#[test]
fn test_hkdf_rejects_weak_ikm() {
    let short_ikm = [0u8; 16];
    let result = hkdf_expand_32(&short_ikm, &[], b"context");
    assert!(matches!(result, Err(KdfError::HkdfSaltRequired)));
}

#[test]
fn test_hkdf_allows_32b_ikm() {
    let full_ikm = [0u8; 32];
    let result = hkdf_expand_32(&full_ikm, &[], b"context");
    assert!(result.is_ok());
}

#[test]
fn test_hkdf_allows_salt_ikm() {
    let short_ikm = [0u8; 8];
    let salt = b"random_salt";
    let result = hkdf_expand_32(&short_ikm, salt, b"context");
    assert!(result.is_ok());
}

#[test]
fn test_hkdf_allows_long_ikm() {
    let long_ikm = [0u8; 64];
    let result = hkdf_expand_32(&long_ikm, &[], b"context");
    assert!(result.is_ok());
}

#[test]
fn test_reject_malicious_argon2_params() {
    let malicious = Argon2Params {
        memory: MAX_MEM_LIMIT_KIB + 1,
        iterations: MAX_OPS_LIMIT + 1,
        parallelism: MAX_PARALLELISM + 1,
    };

    assert!(malicious.validate_untrusted().is_err());
}

#[test]
fn test_reject_zero_params() {
    let p = Argon2Params {
        memory: 0,
        iterations: 0,
        parallelism: 0,
    };
    assert!(p.validate_untrusted().is_err());
}

#[test]
#[cfg(not(target_arch = "wasm32"))]
fn test_desktop_limits_expected() {
    assert_eq!(MAX_MEM_LIMIT_KIB, 256 * 1024);
}

#[test]
#[cfg(target_arch = "wasm32")]
fn test_wasm_limits_expected() {
    assert_eq!(MAX_MEM_LIMIT_KIB, 64 * 1024);
}

#[test]
fn test_hkdf_rfc5869_case1() {
    let ikm = [0x0b; 22];
    let salt = [
        0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0a, 0x0b, 0x0c,
    ];
    let info = [0xf0, 0xf1, 0xf2, 0xf3, 0xf4, 0xf5, 0xf6, 0xf7, 0xf8, 0xf9];
    let expected = [
        0x3c, 0xb2, 0x5f, 0x25, 0xfa, 0xac, 0xd5, 0x7a, 0x90, 0x43, 0x4f, 0x64, 0xd0, 0x36, 0x2f,
        0x2a, 0x2d, 0x2d, 0x0a, 0x90, 0xcf, 0x1a, 0x5a, 0x4c, 0x5d, 0xb0, 0x2d, 0x56, 0xec, 0xc4,
        0xc5, 0xbf,
    ];

    let result = hkdf_expand_32(&ikm, &salt, &info).unwrap();
    assert_eq!(result.reveal(), &expected, "RFC 5869 test case 1 failed");
}

#[test]
fn test_hkdf_rfc5869_case2() {
    let ikm = [
        0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0a, 0x0b, 0x0c, 0x0d, 0x0e,
        0x0f, 0x10, 0x11, 0x12, 0x13, 0x14, 0x15, 0x16, 0x17, 0x18, 0x19, 0x1a, 0x1b, 0x1c, 0x1d,
        0x1e, 0x1f, 0x20, 0x21, 0x22, 0x23, 0x24, 0x25, 0x26, 0x27, 0x28, 0x29, 0x2a, 0x2b, 0x2c,
        0x2d, 0x2e, 0x2f, 0x30, 0x31, 0x32, 0x33, 0x34, 0x35, 0x36, 0x37, 0x38, 0x39, 0x3a, 0x3b,
        0x3c, 0x3d, 0x3e, 0x3f, 0x40, 0x41, 0x42, 0x43, 0x44, 0x45, 0x46, 0x47, 0x48, 0x49, 0x4a,
        0x4b, 0x4c, 0x4d, 0x4e, 0x4f,
    ];
    let salt = [
        0x60, 0x61, 0x62, 0x63, 0x64, 0x65, 0x66, 0x67, 0x68, 0x69, 0x6a, 0x6b, 0x6c, 0x6d, 0x6e,
        0x6f, 0x70, 0x71, 0x72, 0x73, 0x74, 0x75, 0x76, 0x77, 0x78, 0x79, 0x7a, 0x7b, 0x7c, 0x7d,
        0x7e, 0x7f, 0x80, 0x81, 0x82, 0x83, 0x84, 0x85, 0x86, 0x87, 0x88, 0x89, 0x8a, 0x8b, 0x8c,
        0x8d, 0x8e, 0x8f, 0x90, 0x91, 0x92, 0x93, 0x94, 0x95, 0x96, 0x97, 0x98, 0x99, 0x9a, 0x9b,
        0x9c, 0x9d, 0x9e, 0x9f, 0xa0, 0xa1, 0xa2, 0xa3, 0xa4, 0xa5, 0xa6, 0xa7, 0xa8, 0xa9, 0xaa,
        0xab, 0xac, 0xad, 0xae, 0xaf,
    ];
    let info = [
        0xb0, 0xb1, 0xb2, 0xb3, 0xb4, 0xb5, 0xb6, 0xb7, 0xb8, 0xb9, 0xba, 0xbb, 0xbc, 0xbd, 0xbe,
        0xbf, 0xc0, 0xc1, 0xc2, 0xc3, 0xc4, 0xc5, 0xc6, 0xc7, 0xc8, 0xc9, 0xca, 0xcb, 0xcc, 0xcd,
        0xce, 0xcf, 0xd0, 0xd1, 0xd2, 0xd3, 0xd4, 0xd5, 0xd6, 0xd7, 0xd8, 0xd9, 0xda, 0xdb, 0xdc,
        0xdd, 0xde, 0xdf, 0xe0, 0xe1, 0xe2, 0xe3, 0xe4, 0xe5, 0xe6, 0xe7, 0xe8, 0xe9, 0xea, 0xeb,
        0xec, 0xed, 0xee, 0xef, 0xf0, 0xf1, 0xf2, 0xf3, 0xf4, 0xf5, 0xf6, 0xf7, 0xf8, 0xf9, 0xfa,
        0xfb, 0xfc, 0xfd, 0xfe, 0xff,
    ];
    let expected = [
        0xb1, 0x1e, 0x39, 0x8d, 0xc8, 0x03, 0x27, 0xa1, 0xc8, 0xe7, 0xf7, 0x8c, 0x59, 0x6a, 0x49,
        0x34, 0x4f, 0x01, 0x2e, 0xda, 0x2d, 0x4e, 0xfa, 0xd8, 0xa0, 0x50, 0xcc, 0x4c, 0x19, 0xaf,
        0xa9, 0x7c,
    ];

    let result = hkdf_expand_32(&ikm, &salt, &info).unwrap();
    assert_eq!(result.reveal(), &expected, "RFC 5869 test case 2 failed");
}

#[test]
fn test_rfc5869_case3() {
    let ikm = [0x0b; 22];
    let info: &[u8] = &[];
    let expected = [
        0x8d, 0xa4, 0xe7, 0x75, 0xa5, 0x63, 0xc1, 0x8f, 0x71, 0x5f, 0x80, 0x2a, 0x06, 0x3c, 0x5a,
        0x31, 0xb8, 0xa1, 0x1f, 0x5c, 0x5e, 0xe1, 0x87, 0x9e, 0xc3, 0x45, 0x4e, 0x5f, 0x3c, 0x73,
        0x8d, 0x2d,
    ];

    let hkdf = Hkdf::<Sha256>::new(None, &ikm);
    let mut result = [0u8; 32];
    hkdf.expand(info, &mut result).unwrap();
    assert_eq!(
        result, expected,
        "RFC 5869 test case 3 (no salt/info) failed"
    );
}

#[test]
fn test_error_sensitive_data_leak() {
    let errors = vec![
        KdfError::Argon2Params,
        KdfError::Argon2Execution,
        KdfError::HkdfExpansion,
        KdfError::HkdfInfoEmpty,
    ];

    for err in errors {
        let msg = err.to_string();
        assert!(!msg.contains("1048576"), "Error leaks memory size: {}", msg);
        assert!(!msg.contains(" 256"), "Error leaks numeric value: {}", msg);
        assert!(!msg.contains("256MB"), "Error leaks memory size: {}", msg);
        assert!(!msg.contains("MB"), "Error leaks memory unit: {}", msg);
        assert!(!msg.contains("KiB"), "Error leaks memory unit: {}", msg);
        assert!(!msg.contains(" 3 "), "Error leaks iteration count: {}", msg);
        assert!(
            !msg.contains("iterations:"),
            "Error leaks parameter: {}",
            msg
        );
        assert!(!msg.contains("memory:"), "Error leaks parameter: {}", msg);
        assert!(
            !msg.contains("parallelism:"),
            "Error leaks parameter: {}",
            msg
        );
        assert!(msg.len() < 80, "Error message too verbose: {}", msg);
    }
}

#[test]
fn test_validate_untrusted_error_safety() {
    let malicious = Argon2Params {
        memory: 999_999_999,
        iterations: 3,
        parallelism: 1,
    };
    let result = malicious.validate_untrusted();
    assert!(
        result.is_err(),
        "Expected validation to fail for malicious params"
    );

    let err = result.unwrap_err();
    let msg = err.to_string();
    assert!(
        !msg.contains("999999999"),
        "validate_untrusted leaks attempted value: {}",
        msg
    );
    assert!(
        !msg.contains("999"),
        "validate_untrusted leaks partial value: {}",
        msg
    );
}

#[test]
fn test_argon2_at_max_rejected() {
    let params = Argon2Params {
        memory: 256 * 1024,
        iterations: 5,
        parallelism: 8,
    };
    let result = params.validate_untrusted();
    assert!(
        result.is_err(),
        "Should reject parameters at max that exceed total cost"
    );
}

#[test]
fn test_argon2_near_limit_ok() {
    let params = Argon2Params {
        memory: 230_000,
        iterations: 5,
        parallelism: 7,
    };
    let total = 230_000u64 * 5 * 7;
    assert!(total <= MAX_ARGON2_TOTAL_COST, "Test setup error");

    let result = params.validate_untrusted();
    assert!(
        result.is_ok(),
        "Should accept parameters below total cost limit"
    );
}

#[test]
fn test_argon2_high_params_ok() {
    let params = Argon2Params {
        memory: 200_000,
        iterations: 5,
        parallelism: 7,
    };
    let total = 200_000u64 * 5 * 7;
    assert!(
        total <= MAX_ARGON2_TOTAL_COST,
        "Test setup: {} > {}",
        total,
        MAX_ARGON2_TOTAL_COST
    );

    let result = params.validate_untrusted();
    assert!(
        result.is_ok(),
        "Should accept high-security parameters within limits"
    );
}

#[test]
fn test_all_argon2_presets_valid() {
    let presets = vec![
        ("interactive", Argon2Params::interactive()),
        ("moderate", Argon2Params::moderate()),
        ("strong", Argon2Params::strong()),
        ("debug", Argon2Params::debug()),
        ("test_fast", Argon2Params::test_fast()),
    ];

    for (name, params) in presets {
        let result = params.validate_untrusted();
        assert!(
            result.is_ok(),
            "Preset '{}' should pass validation but got error: {:?}",
            name,
            result
        );

        let total_cost_result = (params.memory as u64)
            .checked_mul(params.iterations as u64)
            .and_then(|v| v.checked_mul(params.parallelism as u64));

        assert!(
            total_cost_result.is_some(),
            "Preset '{}' overflows total cost multiplication",
            name
        );
        let total_cost = total_cost_result.unwrap();
        assert!(
            total_cost <= MAX_ARGON2_TOTAL_COST,
            "Preset '{}' exceeds MAX_ARGON2_TOTAL_COST: {} > {}",
            name,
            total_cost,
            MAX_ARGON2_TOTAL_COST
        );
    }
}

#[test]
fn test_argon2_exceeds_total() {
    let params = Argon2Params {
        memory: 240_000,
        iterations: 5,
        parallelism: 8,
    };

    let total = 240_000u64 * 5 * 8;
    assert!(
        total > MAX_ARGON2_TOTAL_COST,
        "Test setup: {} should exceed {}",
        total,
        MAX_ARGON2_TOTAL_COST
    );

    let result = params.validate_untrusted();
    assert!(
        result.is_err(),
        "Should reject params that exceed total cost despite passing individual checks"
    );
}

#[test]
fn test_secret_bytes32_zeroizes_drop() {
    let mut test_data = [0x42u8; 32];
    let mut secret = SecretBytes32::new(test_data);
    secret.zeroize();
    assert_eq!(
        secret.reveal(),
        &[0u8; 32],
        "Explicit zeroize should clear data"
    );
    test_data.zeroize();
    assert_eq!(test_data, [0u8; 32]);
}

#[test]
fn test_bytes32_inner_auto_zero() {
    let test_data = [0x42u8; 32];
    let secret = SecretBytes32::new(test_data);

    let bytes = secret.into_inner();
    assert_eq!(bytes, test_data, "into_inner should return non-zeroed data");

    let mut bytes_mut = bytes;
    bytes_mut.zeroize();
    assert_eq!(bytes_mut, [0u8; 32], "Caller must manually zeroize");
}

#[test]
fn test_bytes32_debug_no_leak() {
    let secret = SecretBytes32::new([0x42u8; 32]);
    let debug_str = format!("{:?}", secret);
    assert!(
        !debug_str.contains("42"),
        "Debug should not reveal secret bytes"
    );
    assert!(debug_str.contains("***"), "Debug should show placeholder");
}

#[test]
fn test_secret_bytes32_eq_identical() {
    let bytes1 = SecretBytes32::new([42u8; 32]);
    let bytes2 = SecretBytes32::new([42u8; 32]);
    assert!(
        bytes1.ct_eq(&bytes2),
        "Identical values should compare equal"
    );
}

#[test]
fn test_secret_bytes32_eq_different() {
    let bytes1 = SecretBytes32::new([42u8; 32]);
    let bytes2 = SecretBytes32::new([99u8; 32]);
    assert!(
        !bytes1.ct_eq(&bytes2),
        "Different values should compare unequal"
    );
}

#[test]
fn test_secret_bytes32_subtle_trait() {
    use subtle::ConstantTimeEq;

    let bytes1 = SecretBytes32::new([42u8; 32]);
    let bytes2 = SecretBytes32::new([42u8; 32]);
    let choice = <SecretBytes32 as ConstantTimeEq>::ct_eq(&bytes1, &bytes2);
    assert!(bool::from(choice), "ConstantTimeEq trait should work");
}

#[test]
fn test_types_eq_compile_check() {
    let bytes1 = SecretBytes32::new([42u8; 32]);
    let bytes2 = SecretBytes32::new([42u8; 32]);
    assert!(bytes1.ct_eq(&bytes2));
}

#[test]
fn test_view_sk_deterministic() {
    let receiver_secret = [0x42; 32];
    let sk1 = derive_view_sk(&receiver_secret).unwrap();
    let sk2 = derive_view_sk(&receiver_secret).unwrap();
    assert_eq!(sk1.as_bytes(), sk2.as_bytes());
}

#[test]
fn test_owner_handle_deterministic() {
    let receiver_secret = [0x42; 32];
    let h1 = derive_owner_handle(&receiver_secret);
    let h2 = derive_owner_handle(&receiver_secret);
    assert_eq!(h1, h2);
}

#[test]
fn test_owner_tag_deterministic() {
    let owner_handle = [0x42; 32];
    let k_dh = [0xAA; 32];
    let tag1 = compute_owner_tag(&owner_handle, &k_dh);
    let tag2 = compute_owner_tag(&owner_handle, &k_dh);
    assert_eq!(tag1, tag2);
}

#[test]
fn test_asset_id_deterministic() {
    let s_out = [0x42; 32];
    let id1 = derive_asset_id(&s_out);
    let id2 = derive_asset_id(&s_out);
    assert_eq!(id1, id2);
}

#[test]
fn test_leaf_ad_deterministic() {
    let asset_id = [0x42; 32];
    let serial_id = 1u32;
    let r_pub = [0xAA; 32];
    let owner_tag = [0xBB; 32];
    let c_amount = [0xCC; 32];

    let ad1 = derive_leaf_ad(&asset_id, serial_id, &r_pub, &owner_tag, &c_amount);
    let ad2 = derive_leaf_ad(&asset_id, serial_id, &r_pub, &owner_tag, &c_amount);
    assert_eq!(ad1, ad2);
}

#[test]
fn test_different_receiver_secret() {
    let secret1 = [0x01; 32];
    let secret2 = [0x02; 32];

    let handle1 = derive_owner_handle(&secret1);
    let handle2 = derive_owner_handle(&secret2);
    assert_ne!(handle1, handle2);

    let sk1 = derive_view_sk(&secret1).unwrap();
    let sk2 = derive_view_sk(&secret2).unwrap();
    assert_ne!(sk1.as_bytes(), sk2.as_bytes());
}

#[test]
fn test_different_k_dh() {
    let owner_handle = [0x42; 32];
    let k_dh1 = [0xAA; 32];
    let k_dh2 = [0xBB; 32];

    let tag1 = compute_owner_tag(&owner_handle, &k_dh1);
    let tag2 = compute_owner_tag(&owner_handle, &k_dh2);
    assert_ne!(tag1, tag2);
}

#[test]
fn test_different_s_out() {
    let s1 = [0x01; 32];
    let s2 = [0x02; 32];
    let id1 = derive_asset_id(&s1);
    let id2 = derive_asset_id(&s2);
    assert_ne!(id1, id2);
}

#[test]
fn test_different_serial_id() {
    let asset_id = [0x42; 32];
    let r_pub = [0xAA; 32];
    let owner_tag = [0xBB; 32];
    let c_amount = [0xCC; 32];

    let ad1 = derive_leaf_ad(&asset_id, 1, &r_pub, &owner_tag, &c_amount);
    let ad2 = derive_leaf_ad(&asset_id, 2, &r_pub, &owner_tag, &c_amount);
    assert_ne!(ad1, ad2);
}

#[test]
fn test_full_derivation_chain() {
    let receiver_secret = [0x42; 32];
    let k_dh = [0xAA; 32];
    let s_out = [0x11; 32];
    let serial_id = 1u32;
    let r_pub = [0xBB; 32];
    let c_amount = [0xCC; 32];

    let owner_handle = derive_owner_handle(&receiver_secret);
    let view_sk = derive_view_sk(&receiver_secret).unwrap();
    let owner_tag = compute_owner_tag(&owner_handle, &k_dh);
    let asset_id = derive_asset_id(&s_out);
    let leaf_ad = derive_leaf_ad(&asset_id, serial_id, &r_pub, &owner_tag, &c_amount);

    assert_eq!(owner_handle.len(), 32);
    assert_eq!(owner_tag.len(), 32);
    assert_eq!(asset_id.len(), 32);
    assert_eq!(leaf_ad.len(), 32);
    assert_eq!(view_sk.as_bytes().len(), 32);
}

#[test]
fn test_h2scalar_deterministic() {
    let a = hash_to_scalar_domain(b"domain", &[b"data"]);
    let b = hash_to_scalar_domain(b"domain", &[b"data"]);
    assert_eq!(a.to_bytes(), b.to_bytes());
}

#[test]
fn test_h2scalar_domain_sep() {
    let a = hash_to_scalar_domain(b"domain_a", &[b"data"]);
    let b = hash_to_scalar_domain(b"domain_b", &[b"data"]);
    assert_ne!(a.to_bytes(), b.to_bytes());
}

#[test]
fn test_h2scalar_not_zero() {
    for i in 0..256u16 {
        let data = i.to_le_bytes();
        let scalar = hash_to_scalar_domain(b"test", &[&data]);
        assert!(!scalar.is_zero());
    }
}

#[test]
fn test_h2scalar_frame_guard() {
    let a = hash_to_scalar_domain(b"domain", &[b"ab", b"c"]);
    let b = hash_to_scalar_domain(b"domain", &[b"a", b"bc"]);
    assert_ne!(a.to_bytes(), b.to_bytes());
}

#[test]
fn test_kdf_consensus_deterministic() -> Result<(), CryptoError> {
    let a = kdf_consensus(b"info", b"ikm")?;
    let b = kdf_consensus(b"info", b"ikm")?;
    assert_eq!(a, b);
    Ok(())
}

#[test]
fn test_kdf_wallet_deterministic() -> Result<(), CryptoError> {
    let a = kdf_wallet(b"info", b"ikm")?;
    let b = kdf_wallet(b"info", b"ikm")?;
    assert_eq!(a, b);
    Ok(())
}

#[test]
fn test_kdf_domain_sep() -> Result<(), CryptoError> {
    let a = kdf_consensus(b"info", b"ikm")?;
    let b = kdf_wallet(b"info", b"ikm")?;
    assert_ne!(a, b);
    Ok(())
}

#[test]
fn test_kdf_info_binding() -> Result<(), CryptoError> {
    let a = kdf_wallet(b"info_a", b"ikm")?;
    let b = kdf_wallet(b"info_b", b"ikm")?;
    assert_ne!(a, b);
    Ok(())
}

#[test]
fn test_wallet_var_len() -> Result<(), CryptoError> {
    let out = kdf_wallet_variable(b"info", b"ikm", 48)?;
    assert_eq!(out.len(), 48);
    Ok(())
}

#[test]
fn test_pack_nonce_unique() -> Result<(), CryptoError> {
    let k_dh = [0xAAu8; 32];
    let n0 = derive_pack_nonce(&k_dh, 0)?;
    let n1 = derive_pack_nonce(&k_dh, 1)?;
    assert_ne!(n0, n1);
    Ok(())
}

#[test]
fn test_pack_key_deterministic() -> Result<(), CryptoError> {
    let k_dh = [0xABu8; 32];
    let a = derive_pack_key(&k_dh)?;
    let b = derive_pack_key(&k_dh)?;
    assert_eq!(a, b);
    Ok(())
}

#[test]
fn test_db_key_deterministic() -> Result<(), CryptoError> {
    let master = [0xBCu8; 32];
    let a = derive_db_encryption_key(&master)?;
    let b = derive_db_encryption_key(&master)?;
    assert_eq!(a, b);
    Ok(())
}

#[test]
fn test_dual_keys_distinct() -> Result<(), CryptoError> {
    let master = [0xCDu8; 32];
    let (enc, mac) = derive_encrypt_and_mac_keys(&master)?;
    assert_ne!(enc, mac);
    Ok(())
}

#[test]
fn test_sym_key_from_ecdh() -> Result<(), CryptoError> {
    let scalar = Z00ZScalar::one();
    let dh = Z00ZRistrettoPoint::from_secret_key(&scalar);
    let key = derive_symmetric_key_from_ecdh(&dh)?;
    assert_eq!(key.len(), 32);
    Ok(())
}

#[test]
fn test_reject_identity_dh() {
    let identity = Z00ZRistrettoPoint::identity();
    let err = kdf_from_dh(&identity).expect_err("identity must fail");
    assert_eq!(err, CryptoError::IdentityPoint);
}

#[test]
fn test_hedged_same_entropy() {
    let ctx = b"ctx";
    let msg = b"msg";
    let ent = [0x42u8; 32];
    let r1 = generate_hedged_r(ctx, msg, &ent);
    let r2 = generate_hedged_r(ctx, msg, &ent);
    assert_eq!(r1.to_bytes(), r2.to_bytes());
}

#[test]
fn test_hedged_diff_entropy() {
    let ctx = b"ctx";
    let msg = b"msg";
    let ent1 = [0x42u8; 32];
    let ent2 = [0x43u8; 32];
    let r1 = generate_hedged_r(ctx, msg, &ent1);
    let r2 = generate_hedged_r(ctx, msg, &ent2);
    assert_ne!(r1.to_bytes(), r2.to_bytes());
}
