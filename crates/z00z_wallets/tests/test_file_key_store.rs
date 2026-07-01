use z00z_crypto::expert::encoding::SafePassword;
use z00z_crypto::Hidden;
use z00z_utils::io::create_dir_all;
use z00z_wallets::key::{ReceiverSecret, StealthKeyError};
use z00z_wallets::security::{EncryptionScheme, FileKeyStore, FileKeyStoreError, SecureKeyStore};

fn temp_path(name: &str) -> (tempfile::TempDir, std::path::PathBuf) {
    let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join("target")
        .join("test-tmp");
    create_dir_all(&root).expect("temp root");
    let dir = tempfile::Builder::new()
        .prefix(name)
        .rand_bytes(6)
        .tempdir_in(&root)
        .expect("tempdir");
    let path = dir.path().join(name);
    (dir, path)
}

fn make_secret() -> Result<ReceiverSecret, StealthKeyError> {
    ReceiverSecret::generate()
}

#[test]
fn test_store_roundtrip_safe_password() {
    let (_dir, base) = temp_path("z00z_file_key_store_safe_pw");
    let mut store = FileKeyStore::new(
        base.clone(),
        EncryptionScheme::Password(SafePassword::from("test-password")),
    );

    let secret = Hidden::hide(make_secret().expect("secret"));
    store.store_key("receiver-main", &secret).expect("store");

    let loaded = store.load_key("receiver-main").expect("load");
    assert_eq!(secret.reveal().as_bytes(), loaded.reveal().as_bytes());
}

#[test]
fn test_store_wrong_password_fails() {
    let (_dir, base) = temp_path("z00z_file_key_store_wrong_pw");
    let mut store = FileKeyStore::new(
        base.clone(),
        EncryptionScheme::Password(SafePassword::from("test-password-a")),
    );

    let secret = Hidden::hide(make_secret().expect("secret"));
    store.store_key("receiver-main", &secret).expect("store");

    let wrong_store = FileKeyStore::new(
        base.clone(),
        EncryptionScheme::Password(SafePassword::from("test-password-b")),
    );
    let err = wrong_store.load_key("receiver-main").unwrap_err();

    assert!(matches!(err, FileKeyStoreError::Key(_)));
}

#[test]
fn test_store_invalid_id_rejected() {
    let (_dir, base) = temp_path("z00z_file_key_store_bad_id");
    let mut store = FileKeyStore::new(
        base,
        EncryptionScheme::Password(SafePassword::from("test-password")),
    );

    let secret = Hidden::hide(make_secret().expect("secret"));
    let err = store.store_key("sub/path", &secret).unwrap_err();

    assert!(matches!(err, FileKeyStoreError::InvalidKeyId));
}
