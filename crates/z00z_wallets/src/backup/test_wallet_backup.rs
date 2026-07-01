use super::*;

#[test]
fn test_derive_key_works() {
    let password = SafePassword::from("test-password");
    let salt = [0x42u8; 16];
    let key = WalletBackupCrypto::derive_key(&password, &salt).unwrap();
    assert_eq!(key.len(), 32);
}

#[test]
fn test_aad_tag_is_deterministic() {
    let aad = b"test-aad";
    let tag1 = wallet_backup_aad_tag(aad);
    let tag2 = wallet_backup_aad_tag(aad);
    assert_eq!(tag1, tag2);
}

#[test]
fn test_checksum_binds_and_ciphertext() {
    let aad = b"aad-data";
    let ct = b"ciphertext";
    let checksum = wallet_backup_checksum(aad, ct);
    assert_eq!(checksum.len(), 32);
}

#[test]
fn test_encrypt_decrypt_roundtrip() {
    use z00z_crypto::aead::open;
    use z00z_crypto::aead::test_only::seal_with_nonce_TEST_ONLY;

    let key = [0x55u8; 32];
    let nonce = [0x99u8; 24];
    let aad = b"test-aad";
    let plaintext = b"secret-data";

    let ciphertext = seal_with_nonce_TEST_ONLY(&key, aad, plaintext, nonce).unwrap();
    let decrypted = open(&key, aad, &ciphertext).unwrap();

    assert_eq!(decrypted, plaintext);
}

#[test]
fn test_facade_methods_work() {
    let password = SafePassword::from("password");
    let salt = [0x01u8; 16];
    let aad = b"metadata";
    let key = WalletBackupCrypto::derive_key(&password, &salt).unwrap();
    let tag = WalletBackupCrypto::aad_tag(aad);
    let checksum = WalletBackupCrypto::checksum(aad, b"ct");

    assert_eq!(key.len(), 32);
    assert_eq!(tag.len(), 32);
    assert_eq!(checksum.len(), 32);
}

#[test]
fn test_backup_kdf_roundtrip() {
    let kdf = BackupKdf::default([0x44u8; 16]);
    let params = kdf.to_params().unwrap();
    let roundtrip = BackupKdf::from_params(params).unwrap();
    assert_eq!(roundtrip, kdf);
}
