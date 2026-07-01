//! File-based secure key store for receiver secrets.

use std::path::{Path, PathBuf};

use thiserror::Error;

use z00z_crypto::expert::encoding::SafePassword;
use z00z_crypto::Hidden;
use z00z_utils::io::{create_dir_all, read_file, write_file, IoError};

use crate::key::{ReceiverSecret, StealthKeyError};

/// Error type for file key store operations.
#[derive(Debug, Error)]
pub enum FileKeyStoreError {
    /// Key id contains path separators or is empty.
    #[error("invalid key id")]
    InvalidKeyId,
    /// I/O operation failed.
    #[error(transparent)]
    Io(#[from] IoError),
    /// Receiver key encryption/decryption failed.
    #[error(transparent)]
    Key(#[from] StealthKeyError),
}

/// Encryption scheme for key files.
pub enum EncryptionScheme {
    /// Password-protected envelope using existing receiver-secret codec.
    Password(SafePassword),
}

impl EncryptionScheme {
    fn encrypt(&self, key: &ReceiverSecret) -> Result<Vec<u8>, StealthKeyError> {
        match self {
            Self::Password(password) => key.to_encrypted_password(password),
        }
    }

    fn decrypt(&self, data: &[u8]) -> Result<ReceiverSecret, StealthKeyError> {
        match self {
            Self::Password(password) => ReceiverSecret::from_encrypted_password(data, password),
        }
    }
}

/// Secure key-store interface for receiver secrets.
pub trait SecureKeyStore {
    /// Stores encrypted receiver secret under key id.
    fn store_key(
        &mut self,
        key_id: &str,
        key: &Hidden<ReceiverSecret>,
    ) -> Result<(), FileKeyStoreError>;
    /// Loads and decrypts receiver secret by key id.
    fn load_key(&self, key_id: &str) -> Result<Hidden<ReceiverSecret>, FileKeyStoreError>;
}

/// File-based implementation of `SecureKeyStore`.
pub struct FileKeyStore {
    path: PathBuf,
    encryption: EncryptionScheme,
}

impl FileKeyStore {
    /// Creates a new file-backed key store.
    pub fn new(path: PathBuf, encryption: EncryptionScheme) -> Self {
        Self { path, encryption }
    }

    fn key_path(&self, key_id: &str) -> Result<PathBuf, FileKeyStoreError> {
        if !is_valid_key_id(key_id) {
            return Err(FileKeyStoreError::InvalidKeyId);
        }
        Ok(self.path.join(key_id))
    }
}

impl SecureKeyStore for FileKeyStore {
    fn store_key(
        &mut self,
        key_id: &str,
        key: &Hidden<ReceiverSecret>,
    ) -> Result<(), FileKeyStoreError> {
        create_dir_all(&self.path)?;
        let file_path = self.key_path(key_id)?;
        let encrypted = self.encryption.encrypt(key.reveal())?;
        write_file(&file_path, &encrypted)?;
        Ok(())
    }

    fn load_key(&self, key_id: &str) -> Result<Hidden<ReceiverSecret>, FileKeyStoreError> {
        let file_path = self.key_path(key_id)?;
        let encrypted = read_file(&file_path)?;
        let secret = self.encryption.decrypt(&encrypted)?;
        Ok(Hidden::hide(secret))
    }
}

fn is_valid_key_id(key_id: &str) -> bool {
    if key_id.is_empty() {
        return false;
    }

    if key_id == "." || key_id == ".." {
        return false;
    }

    let path = Path::new(key_id);
    path.components().count() == 1
}

#[cfg(test)]
mod tests {
    use super::{EncryptionScheme, FileKeyStore, SecureKeyStore};
    use z00z_crypto::expert::encoding::SafePassword;
    use z00z_crypto::Hidden;
    use z00z_utils::io::{create_dir_all, read_file, write_file};

    use crate::key::{ReceiverSecret, StealthKeyError};

    fn test_space(name: &str) -> tempfile::TempDir {
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

    fn make_secret() -> Result<ReceiverSecret, StealthKeyError> {
        ReceiverSecret::generate()
    }

    #[test]
    fn test_file_key_store_roundtrip() {
        let dir = test_space("file-key-store-roundtrip");
        let base = dir.path().join("z00z_file_key_store_roundtrip");
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
    fn test_file_key_store_corruption() {
        let dir = test_space("file-key-store-corrupt");
        let base = dir.path().join("z00z_file_key_store_corrupt");
        let mut store = FileKeyStore::new(
            base.clone(),
            EncryptionScheme::Password(SafePassword::from("test-password")),
        );

        let secret = Hidden::hide(make_secret().expect("secret"));
        store.store_key("receiver-main", &secret).expect("store");

        let key_file = base.join("receiver-main");
        let mut bytes = read_file(&key_file).expect("read");
        bytes.truncate(bytes.len().saturating_sub(7));
        write_file(&key_file, &bytes).expect("rewrite");

        let result = store.load_key("receiver-main");
        assert!(matches!(
            result,
            Err(super::FileKeyStoreError::Key(
                StealthKeyError::InvalidEnvelope
            )) | Err(super::FileKeyStoreError::Key(
                StealthKeyError::DecryptFailed
            )) | Err(super::FileKeyStoreError::Key(
                StealthKeyError::UnsupportedVersion
            ))
        ));
    }

    #[test]
    fn test_key_store_invalid_id() {
        let dir = test_space("file-key-store-invalid");
        let base = dir.path().join("z00z_file_key_store_invalid");
        let mut store = FileKeyStore::new(
            base,
            EncryptionScheme::Password(SafePassword::from("test-password")),
        );

        let secret = Hidden::hide(make_secret().expect("secret"));
        let result = store.store_key("sub/path", &secret);
        assert!(matches!(
            result,
            Err(super::FileKeyStoreError::InvalidKeyId)
        ));

        let result_dot = store.store_key(".", &secret);
        assert!(matches!(
            result_dot,
            Err(super::FileKeyStoreError::InvalidKeyId)
        ));

        let result_dotdot = store.store_key("..", &secret);
        assert!(matches!(
            result_dotdot,
            Err(super::FileKeyStoreError::InvalidKeyId)
        ));
    }
}
