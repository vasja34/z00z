//! Secret store implementation.
//!
//! Phase 1 implementation notes:
//! - This component currently implements password verification + session lifecycle.
//! - Persistent at-rest encryption is not implemented yet because the expected
//!   password KDF + AEAD helpers are not available in `z00z_crypto`.
//! - All I/O and serialization uses `z00z_utils` abstractions (ONE SOURCE OF TRUTH).

use crate::db::wallet_store_crypto::{
    aad_master_key, derive_pw_key, AeadEnvelope, KdfParams, MasterKeyRecord,
};
use serde::{Deserialize, Serialize};
use std::convert::TryInto;
use std::path::{Path, PathBuf};
use z00z_crypto::expert::encoding::SafePassword;
use z00z_crypto::Hidden;
use z00z_utils::{
    io,
    io::IoError,
    rng::{RngCoreExt, SecureRngProvider},
    time::TimeProvider,
};

use super::{SecretStore, SecretStoreError};
use crate::wallet::session::SessionHandle;

const SECRET_STORE_FILE_VERSION: u32 = 2;
const MAX_WALLET_FILE_SIZE: u64 = 10 * 1024 * 1024;

#[derive(Debug, Clone, Serialize, Deserialize)]
struct SecretStoreFile {
    version: u32,
    kdf: KdfParams,
    master_key: MasterKeyRecord,
}

fn secret_store_aad(storage_path: &Path) -> Vec<u8> {
    aad_master_key(storage_path.to_string_lossy().as_bytes())
}

/// Default `SecretStore` implementation.
///
/// Stores a small metadata file containing a password verifier and uses a
/// time-limited `SessionHandle` to represent an unlocked session.
#[derive(Debug)]
pub struct SecretStoreImpl<T: TimeProvider, R: SecureRngProvider> {
    storage_path: PathBuf,
    session: Option<SessionHandle>,
    master_key: Option<Hidden<[u8; 32]>>,
    time_provider: T,
    rng_provider: R,
}

impl<T: TimeProvider, R: SecureRngProvider> SecretStoreImpl<T, R> {
    /// Create a new secret store.
    pub fn new(storage_path: PathBuf, time_provider: T, rng_provider: R) -> Self {
        Self {
            storage_path,
            session: None,
            master_key: None,
            time_provider,
            rng_provider,
        }
    }

    fn load_file(&self) -> Result<SecretStoreFile, SecretStoreError> {
        let file: SecretStoreFile =
            io::load_json_bounded(&self.storage_path, MAX_WALLET_FILE_SIZE)?;

        if file.version != SECRET_STORE_FILE_VERSION {
            return Err(SecretStoreError::Serialization(format!(
                "unsupported secret store version: {}",
                file.version
            )));
        }

        Ok(file)
    }

    fn save_file(&self, file: &SecretStoreFile) -> Result<(), SecretStoreError> {
        io::save_json(&self.storage_path, file)?;
        Ok(())
    }

    fn is_initialized(&self) -> Result<bool, SecretStoreError> {
        match self.load_file() {
            Ok(_) => Ok(true),
            Err(SecretStoreError::Io(IoError::Io(e)))
                if e.kind() == std::io::ErrorKind::NotFound =>
            {
                Ok(false)
            }
            Err(e) => Err(e),
        }
    }
}

impl<T: TimeProvider, R: SecureRngProvider> SecretStore for SecretStoreImpl<T, R> {
    fn init_new(&mut self, password: &str) -> Result<(), SecretStoreError> {
        if self.is_initialized()? {
            return Err(SecretStoreError::AlreadyInitialized);
        }

        let mut rng = self.rng_provider.rng();

        let mut salt = [0u8; 16];
        rng.fill_bytes_ext(&mut salt);
        let kdf = KdfParams::default_argon2id_with_salt(salt.to_vec());

        let safe_password = SafePassword::from(password.to_string());
        let pw_key = derive_pw_key(&safe_password, &kdf)
            .map_err(|e| SecretStoreError::Encryption(e.to_string()))?;

        let mut master_key = [0u8; 32];
        rng.fill_bytes_ext(&mut master_key);

        let aad = secret_store_aad(self.storage_path.as_path());
        use z00z_crypto::aead::seal;
        let envelope_bytes = seal(pw_key.reveal(), &aad, &master_key)
            .map_err(|e| SecretStoreError::Encryption(e.to_string()))?;
        let envelope = AeadEnvelope {
            envelope: envelope_bytes,
        };

        let file = SecretStoreFile {
            version: SECRET_STORE_FILE_VERSION,
            kdf: kdf.clone(),
            master_key: MasterKeyRecord {
                envelope,
                kdf_params: Some(kdf.clone()),
            },
        };

        self.save_file(&file)?;
        Ok(())
    }

    fn unlock(&mut self, password: &str) -> Result<SessionHandle, SecretStoreError> {
        if self.session.is_some() {
            return Err(SecretStoreError::AlreadyUnlocked);
        }

        let file = match self.load_file() {
            Ok(file) => file,
            Err(SecretStoreError::Io(IoError::Io(e)))
                if e.kind() == std::io::ErrorKind::NotFound =>
            {
                return Err(SecretStoreError::NotInitialized);
            }
            Err(e) => return Err(e),
        };
        file.kdf
            .validate_untrusted_persisted()
            .map_err(|e| SecretStoreError::Serialization(e.to_string()))?;

        if let Some(record_params) = file.master_key.kdf_params.as_ref() {
            if record_params != &file.kdf {
                return Err(SecretStoreError::Serialization(
                    "kdf params mismatch".to_string(),
                ));
            }
        }

        let safe_password = SafePassword::from(password.to_string());
        let pw_key = derive_pw_key(&safe_password, &file.kdf)
            .map_err(|_e| SecretStoreError::InvalidPassword)?;

        let aad = secret_store_aad(self.storage_path.as_path());
        use z00z_crypto::aead::open;
        let decrypted = open(pw_key.reveal(), &aad, &file.master_key.envelope.envelope)
            .map_err(|_| SecretStoreError::InvalidPassword)?;

        let master_key_bytes: [u8; 32] = decrypted.as_slice().try_into().map_err(|_| {
            SecretStoreError::Serialization("invalid master key length".to_string())
        })?;

        self.master_key = Some(Hidden::hide(master_key_bytes));

        let now_ms = self
            .time_provider
            .try_unix_timestamp_ms()
            .map_err(|e| SecretStoreError::Serialization(format!("clock unavailable: {e}")))?;
        let session = SessionHandle::new(now_ms);
        self.session = Some(session.clone());
        Ok(session)
    }

    fn lock(&mut self) -> Result<(), SecretStoreError> {
        if self.session.is_none() {
            return Err(SecretStoreError::NotUnlocked);
        }

        self.session = None;
        self.master_key = None;
        Ok(())
    }

    fn is_unlocked(&self) -> bool {
        self.session.is_some()
    }

    fn session(&self) -> Option<&SessionHandle> {
        self.session.as_ref()
    }

    fn is_session_expired(&self, now_ms: u64, timeout_ms: u64) -> bool {
        self.session
            .as_ref()
            .map(|s| s.is_expired(now_ms, timeout_ms))
            .unwrap_or(true)
    }

    fn update_activity(&mut self, now_ms: u64) -> Result<(), SecretStoreError> {
        let session = self.session.as_mut().ok_or(SecretStoreError::NotUnlocked)?;

        session.update_activity(now_ms);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::time::Duration;

    use uuid::Uuid;
    use z00z_utils::{
        rng::SystemRngProvider,
        time::{MockTimeProvider, TimeProvider},
    };

    fn temp_secret_store_path() -> PathBuf {
        let mut path = std::env::temp_dir();
        path.push(format!("z00z_secret_store_{}.json", Uuid::new_v4()));
        path
    }

    #[test]
    fn test_init_new_creates_file() {
        let path = temp_secret_store_path();
        let time = MockTimeProvider::from_unix_secs(1);
        let rng = SystemRngProvider;

        let mut store = SecretStoreImpl::new(path.clone(), time.clone(), rng);

        store.init_new("pw").unwrap();
        let session = store.unlock("pw").unwrap();

        assert!(store.is_unlocked());
        assert_eq!(store.session().unwrap().token, session.token);

        // Cleanup
        let _ = io::remove_file(&path);
    }

    #[test]
    fn test_init_no_plain_verifier() {
        let path = temp_secret_store_path();
        let time = MockTimeProvider::default();
        let rng = SystemRngProvider;

        let mut store = SecretStoreImpl::new(path.clone(), time, rng);
        store.init_new("pw").unwrap();

        let contents = io::read_to_string(&path).unwrap();
        assert!(
            !contents.contains("password_verifier"),
            "expected no plaintext password verifier field"
        );

        let _ = io::remove_file(&path);
    }

    #[test]
    fn test_unlock_invalid_password_fails() {
        let path = temp_secret_store_path();
        let time = MockTimeProvider::default();
        let rng = SystemRngProvider;

        let mut store = SecretStoreImpl::new(path.clone(), time, rng);
        store.init_new("correct").unwrap();

        let err = store.unlock("wrong").unwrap_err();
        assert!(matches!(err, SecretStoreError::InvalidPassword));

        // Cleanup
        let _ = io::remove_file(&path);
    }

    #[test]
    fn test_lock_requires_unlocked() {
        let path = temp_secret_store_path();
        let time = MockTimeProvider::default();
        let rng = SystemRngProvider;

        let mut store = SecretStoreImpl::new(path, time, rng);

        let err = store.lock().unwrap_err();
        assert!(matches!(err, SecretStoreError::NotUnlocked));
    }

    #[test]
    fn test_update_activity_changes_session() {
        let path = temp_secret_store_path();
        let time = MockTimeProvider::from_unix_secs(10);
        let rng = SystemRngProvider;

        let mut store = SecretStoreImpl::new(path.clone(), time.clone(), rng);
        store.init_new("pw").unwrap();

        let session = store.unlock("pw").unwrap();
        let before = session.last_activity_ms;

        // Advance time, then update
        time.advance_by(Duration::from_millis(500));
        let now_ms = time.compat_unix_timestamp_millis();
        store.update_activity(now_ms).unwrap();

        let after = store.session().unwrap().last_activity_ms;
        assert!(after >= before);

        // Cleanup
        let _ = io::remove_file(&path);
    }
}
