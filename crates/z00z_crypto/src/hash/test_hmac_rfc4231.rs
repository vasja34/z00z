use super::*;
use hex_literal::hex;

#[test]
fn test_rfc4231_test_case1() {
    let key = [0x0b; 20];
    let data = b"Hi There";
    let expected = hex!("b0344c61d8db38535ca8afceaf0bf12b881dc200c9833da726e9376c2e32cff7");
    assert_eq!(hmac_sha256_raw(&key, data), expected);
}

#[test]
fn test_rfc4231_test_case2() {
    let key = b"Jefe";
    let data = b"what do ya want for nothing?";
    let expected = hex!("5bdcc146bf60754e6a042426089575c75a003f089d2739839dec58b964ec3843");
    assert_eq!(hmac_sha256_raw(key, data), expected);
}

#[test]
fn test_rfc4231_test_case3() {
    let key = [0xaa; 20];
    let data = [0xdd; 50];
    let expected = hex!("773ea91e36800e46854db8ebd09181a72959098b3ef8c122d9635514ced565fe");
    assert_eq!(hmac_sha256_raw(&key, &data), expected);
}

#[test]
fn test_rfc4231_test_case4() {
    let key = hex!("0102030405060708090a0b0c0d0e0f10111213141516171819");
    let data = [0xcd; 50];
    let expected = hex!("82558a389a443c0ea4cc819899f2083a85f0faa3e578f8077a2e3ff46729665b");
    assert_eq!(hmac_sha256_raw(&key, &data), expected);
}

#[test]
fn test_rfc4231_test_case6() {
    let key = [0xaa; 131];
    let data = b"Test Using Larger Than Block-Size Key - Hash Key First";
    let expected = hex!("60e431591ee0b67f0d8a26aacbf5b77f8e0bc6213728c5140546040f0ee37f54");
    assert_eq!(hmac_sha256_raw(&key, data), expected);
}

#[test]
fn test_rfc4231_test_case7() {
    let key = [0xaa; 131];
    let data = b"This is a test using a larger than block-size key and a larger than block-size data. The key needs to be hashed before being used by the HMAC algorithm.";
    let expected = hex!("9b09ffa71b942fcb27635fbcd5b0e944bfdc63644f0713938a7f51535c3a35e2");
    assert_eq!(hmac_sha256_raw(&key, data), expected);
}

#[test]
fn test_verify_hmac_success() {
    let key = b"secret-key";
    let domain = "wallet.auth";
    let label = "v1";
    let msg = b"message-to-authenticate";

    let mac = hmac_sha256(key, domain, label, msg);
    assert!(verify_hmac(key, domain, label, msg, &mac));
}

#[test]
fn test_verify_hmac_failure() {
    let key = b"secret-key";
    let domain = "wallet.auth";
    let label = "v1";
    let msg = b"message-to-authenticate";

    let _mac = hmac_sha256(key, domain, label, msg);
    let wrong_mac = [0u8; 32];

    assert!(!verify_hmac(key, domain, label, msg, &wrong_mac));
}

#[test]
fn test_verify_hmac_wrong_message() {
    let key = b"secret-key";
    let domain = "wallet.auth";
    let label = "v1";
    let msg = b"message-to-authenticate";
    let wrong_msg = b"different-message";

    let mac = hmac_sha256(key, domain, label, msg);
    assert!(!verify_hmac(key, domain, label, wrong_msg, &mac));
}

#[test]
fn test_hmac_domain_separation() {
    let key = b"test-key";
    let msg = b"message";

    let mac1 = hmac_sha256(key, "domain1", "label", msg);
    let mac2 = hmac_sha256(key, "domain2", "label", msg);
    let mac3 = hmac_sha256(key, "domain1", "label2", msg);

    assert_ne!(mac1, mac2);
    assert_ne!(mac1, mac3);
    assert_ne!(mac2, mac3);

    let mac1_again = hmac_sha256(key, "domain1", "label", msg);
    assert_eq!(mac1, mac1_again);
}

#[test]
fn test_length_prefixing_collision_resistance() {
    let key = b"key";
    let mac1 = hmac_sha256(key, "ab", "c", b"data");
    let mac2 = hmac_sha256(key, "a", "bc", b"data");

    assert_ne!(mac1, mac2, "Length prefixing failed to prevent collision!");
}

#[test]
fn test_raw_no_domain_separation() {
    let key = b"key";
    let msg = b"message";

    let mac_raw = hmac_sha256_raw(key, msg);
    let mac_domain = hmac_sha256(key, "domain", "label", msg);
    assert_ne!(mac_raw, mac_domain);
}

#[test]
fn test_verify_hmac_constant_time() {
    let key = b"secret";
    let domain = "auth";
    let label = "v1";
    let msg = b"data";

    let correct_mac = hmac_sha256(key, domain, label, msg);
    let mut wrong_mac = correct_mac;
    wrong_mac[0] ^= 1;

    assert!(verify_hmac(key, domain, label, msg, &correct_mac));
    assert!(!verify_hmac(key, domain, label, msg, &wrong_mac));
}
