use z00z_wallets::stealth::zkpack::{ZkPack, ZkPackEncrypted};

#[test]
fn test_zkpack_roundtrip() {
    let k_dh = [0x42u8; 32];
    let leaf_ad = [0xAAu8; 32];
    let r_pub = [0x01u8; 32];
    let asset_id = [0x02u8; 32];
    let serial_id = 1u32;
    let plaintext = b"v=1000|s_out=deadbeef";

    let enc: ZkPackEncrypted =
        ZkPack::encrypt(&k_dh, &leaf_ad, &r_pub, &asset_id, serial_id, plaintext);
    assert_eq!(enc.version, 1, "version must be 1");
    assert_eq!(enc.ciphertext.len(), plaintext.len(), "length-preserving");
    assert_ne!(
        enc.ciphertext.as_slice(),
        plaintext,
        "must actually encrypt"
    );

    let dec = ZkPack::decrypt(&k_dh, &leaf_ad, &r_pub, &asset_id, serial_id, &enc);
    assert_eq!(dec, Some(plaintext.to_vec()), "decryption roundtrip failed");
}

#[test]
fn test_zkpack_wrong_param_none() {
    let enc = ZkPack::encrypt(
        &[0x42u8; 32],
        &[0xAAu8; 32],
        &[0x01u8; 32],
        &[0x02u8; 32],
        1,
        b"secret",
    );

    assert_eq!(
        ZkPack::decrypt(
            &[0xFFu8; 32],
            &[0xAAu8; 32],
            &[0x01u8; 32],
            &[0x02u8; 32],
            1,
            &enc,
        ),
        None,
        "wrong k_dh must give None"
    );
    assert_eq!(
        ZkPack::decrypt(
            &[0x42u8; 32],
            &[0xBBu8; 32],
            &[0x01u8; 32],
            &[0x02u8; 32],
            1,
            &enc,
        ),
        None,
        "wrong leaf_ad must give None"
    );
    assert_eq!(
        ZkPack::decrypt(
            &[0x42u8; 32],
            &[0xAAu8; 32],
            &[0xFFu8; 32],
            &[0x02u8; 32],
            1,
            &enc,
        ),
        None,
        "wrong r_pub must give None"
    );
    assert_eq!(
        ZkPack::decrypt(
            &[0x42u8; 32],
            &[0xAAu8; 32],
            &[0x01u8; 32],
            &[0xFFu8; 32],
            1,
            &enc,
        ),
        None,
        "wrong asset_id must give None"
    );
    assert_eq!(
        ZkPack::decrypt(
            &[0x42u8; 32],
            &[0xAAu8; 32],
            &[0x01u8; 32],
            &[0x02u8; 32],
            2,
            &enc,
        ),
        None,
        "wrong serial_id must give None"
    );
}

#[test]
fn test_zkpack_ad_binding() {
    let k_dh = [0x42u8; 32];
    let asset_id = [0x02u8; 32];
    let r_pub_a = [0x01u8; 32];
    let leaf_ad_a = [0xAAu8; 32];
    let plaintext = b"leaf-A payload";

    let enc = ZkPack::encrypt(&k_dh, &leaf_ad_a, &r_pub_a, &asset_id, 0, plaintext);

    let result = ZkPack::decrypt(&k_dh, &leaf_ad_a, &r_pub_a, &asset_id, 1, &enc);
    assert_eq!(result, None, "serial_id binding must prevent replay");

    let result2 = ZkPack::decrypt(&k_dh, &[0xBBu8; 32], &r_pub_a, &asset_id, 0, &enc);
    assert_eq!(
        result2, None,
        "leaf_ad binding must prevent cross-leaf replay"
    );

    assert_eq!(
        ZkPack::decrypt(&k_dh, &leaf_ad_a, &r_pub_a, &asset_id, 0, &enc),
        Some(plaintext.to_vec())
    );
}

#[test]
fn test_zkpack_determinism() {
    let k_dh = [0x42u8; 32];
    let leaf_ad = [0xAAu8; 32];
    let r_pub = [0x01u8; 32];
    let asset_id = [0x02u8; 32];
    let plaintext = b"determinism check";

    let enc1 = ZkPack::encrypt(&k_dh, &leaf_ad, &r_pub, &asset_id, 0, plaintext);
    let enc2 = ZkPack::encrypt(&k_dh, &leaf_ad, &r_pub, &asset_id, 0, plaintext);

    assert_eq!(
        enc1.ciphertext, enc2.ciphertext,
        "same inputs must give same ciphertext"
    );
    assert_eq!(enc1.tag, enc2.tag, "same inputs must give same tag");
}

#[test]
fn test_zkpack_truncation_detection() {
    let k_dh = [0x42u8; 32];
    let leaf_ad = [0xAAu8; 32];
    let r_pub = [0x01u8; 32];
    let asset_id = [0x02u8; 32];

    let enc = ZkPack::encrypt(&k_dh, &leaf_ad, &r_pub, &asset_id, 0, b"hello world");

    let mut enc_trunc = enc.clone();
    enc_trunc.ciphertext.pop();
    assert_eq!(
        ZkPack::decrypt(&k_dh, &leaf_ad, &r_pub, &asset_id, 0, &enc_trunc),
        None,
        "truncated ct must fail auth"
    );

    let mut enc_ext = enc.clone();
    enc_ext.ciphertext.push(0x00);
    assert_eq!(
        ZkPack::decrypt(&k_dh, &leaf_ad, &r_pub, &asset_id, 0, &enc_ext),
        None,
        "extended ct must fail auth"
    );

    assert!(
        ZkPack::decrypt(&k_dh, &leaf_ad, &r_pub, &asset_id, 0, &enc).is_some(),
        "original ciphertext must decrypt"
    );
}

#[test]
fn test_unsupported_version_fails() {
    let k_dh = [0x52u8; 32];
    let leaf_ad = [0x53u8; 32];
    let r_pub = [0x54u8; 32];
    let asset_id = [0x55u8; 32];

    let mut enc = ZkPack::encrypt(&k_dh, &leaf_ad, &r_pub, &asset_id, 0, b"v1-only");
    enc.version = 2;

    assert_eq!(
        ZkPack::decrypt(&k_dh, &leaf_ad, &r_pub, &asset_id, 0, &enc),
        None,
        "unsupported zkpack versions must fail closed"
    );
}
