use z00z_crypto::{
    aead,
    kdf::{derive_argon2id32_key, Argon2Params},
};
use z00z_utils::io::{create_dir_all, write_file};
use z00z_wallets::key::{ReceiverSecret, StealthKeyError};

const ENC_AAD: &[u8] = b"z00z.wallet.stealth.receiver_secret.v1";
const SEC_VER_1: u8 = 1;

#[cfg(feature = "test-params-fast")]
fn kdf_params() -> Argon2Params {
    Argon2Params {
        memory: 16,
        iterations: 1,
        parallelism: 1,
    }
}

#[cfg(not(feature = "test-params-fast"))]
fn kdf_params() -> Argon2Params {
    Argon2Params::moderate()
}

fn temp_dir(name: &str) -> tempfile::TempDir {
    let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join("target")
        .join("test-tmp");
    create_dir_all(&root).expect("temp root");
    tempfile::Builder::new()
        .prefix(name)
        .rand_bytes(6)
        .tempdir_in(&root)
        .expect("tempdir")
}

fn zero_envelope(password: &[u8]) -> Vec<u8> {
    let salt = [9u8; 32];
    let key = derive_argon2id32_key(password, &salt, &kdf_params()).expect("derive storage key");
    let sealed = aead::seal(key.reveal(), ENC_AAD, &[0u8; 32]).expect("seal zero secret");

    let mut out = Vec::with_capacity(1 + salt.len() + sealed.len());
    out.push(SEC_VER_1);
    out.extend_from_slice(&salt);
    out.extend_from_slice(&sealed);
    out
}

#[test]
fn test_receiver_bytes_rejects_zero() {
    let err = match ReceiverSecret::from_bytes([0u8; 32]) {
        Ok(_) => panic!("zero secret must be rejected"),
        Err(err) => err,
    };
    assert!(matches!(err, StealthKeyError::ZeroSecret));
}

#[test]
fn test_receiver_encrypted_rejects_zero() {
    let err = match ReceiverSecret::from_encrypted(&zero_envelope(b"password"), b"password") {
        Ok(_) => panic!("zero envelope must be rejected"),
        Err(err) => err,
    };
    assert!(matches!(err, StealthKeyError::ZeroSecret));
}

#[test]
fn test_receiver_load_rejects_zero() {
    let dir = temp_dir("receiver-secret-zero");
    let path = dir.path().join("receiver_secret_zero.bin");
    write_file(&path, &zero_envelope(b"password")).expect("write zero envelope");

    let err = match ReceiverSecret::load(&path, b"password") {
        Ok(_) => panic!("zero file must be rejected"),
        Err(err) => err,
    };

    assert!(matches!(err, StealthKeyError::ZeroSecret));
}
