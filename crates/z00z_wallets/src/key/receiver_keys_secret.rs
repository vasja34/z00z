/// Errors produced by stealth receiver key generation, storage, and signing.
#[derive(Debug, Error)]
pub enum StealthKeyError {
    /// The receiver secret decoded to the all-zero value.
    #[error("receiver secret cannot be zero")]
    ZeroSecret,
    /// The encrypted secret payload is malformed.
    #[error("receiver secret encryption payload is invalid")]
    InvalidEnvelope,
    /// The encrypted secret payload used an unknown version byte.
    #[error("unsupported receiver secret encryption version")]
    UnsupportedVersion,
    /// The supplied storage password is empty.
    #[error("password cannot be empty")]
    EmptyPassword,
    /// Argon2id could not derive a storage key.
    #[error("key derivation failed")]
    KeyDeriveFailed,
    /// AEAD sealing failed.
    #[error("encryption failed")]
    EncryptFailed,
    /// AEAD opening failed.
    #[error("decryption failed")]
    DecryptFailed,
    /// A derived scalar was not a usable secret key.
    #[error("invalid secret key")]
    InvalidSecretKey,
    /// A hash-to-scalar derivation produced zero.
    #[error("zero scalar rejected")]
    ZeroScalarRejected,
    /// A derived public point was invalid or identity-like.
    #[error("identity point rejected")]
    IdentityPointRejected,
    /// Identity signing failed.
    #[error("signature failed")]
    SignatureFailed,
    /// Identity signature verification failed.
    #[error("signature verification failed")]
    SignatureVerifyFailed,
    /// Public key serialization failed.
    #[error("public key encoding failed")]
    PublicKeyEncodingFailed,
    /// Receiver card export or signing failed.
    #[error(transparent)]
    ReceiverCard(#[from] ReceiverCardError),
    /// File-system persistence failed.
    #[error(transparent)]
    Io(#[from] IoError),
}

/// Receiver master secret used to derive stealth key material.
#[derive(Zeroize, ZeroizeOnDrop)]
pub struct ReceiverSecret([u8; 32]);

impl ReceiverSecret {
    /// Generates a fresh receiver secret using the workspace RNG abstraction.
    pub fn generate() -> Result<Self, StealthKeyError> {
        let mut rng = SystemRngProvider.rng();
        let mut bytes = [0u8; 32];
        rng.fill_bytes_ext(&mut bytes);
        Self::from_raw(bytes)
    }

    /// Returns the raw secret bytes.
    pub fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }

    /// Constructs a receiver secret from raw bytes after usability validation.
    pub fn from_bytes(bytes: [u8; 32]) -> Result<Self, StealthKeyError> {
        Self::from_raw(bytes)
    }

    /// Verifies that the secret can derive valid view-key material.
    pub fn validate_usable(&self) -> Result<(), StealthKeyError> {
        #[cfg(any(test, feature = "test-params-fast"))]
        if FAIL_USABLE.with(|flag| flag.replace(false)) {
            return Err(StealthKeyError::InvalidSecretKey);
        }

        let view_sk = derive_view_secret_key(self)?;
        let _view_pk = derive_view_public_key(&view_sk)?;
        Ok(())
    }

    /// Wraps the secret in a redacted `Hidden` container.
    pub fn into_hidden(self) -> Hidden<Self> {
        Hidden::hide(self)
    }

    /// Encrypts the secret with Argon2id plus AEAD for at-rest storage.
    pub fn to_encrypted(&self, password: &[u8]) -> Result<Vec<u8>, StealthKeyError> {
        let mut rng = SystemRngProvider.rng();
        let mut salt = [0u8; SALT_LEN];
        rng.fill_bytes_ext(&mut salt);

        let key = Zeroizing::new(derive_storage_key(password, &salt)?);
        let envelope = aead::seal(&key, ENC_AAD, self.as_bytes())
            .map_err(|_| StealthKeyError::EncryptFailed)?;

        let mut out = Vec::with_capacity(1 + SALT_LEN + envelope.len());
        out.push(SEC_VER_1);
        out.extend_from_slice(&salt);
        out.extend_from_slice(&envelope);
        Ok(out)
    }

    /// Encrypts the secret using a `SafePassword` wrapper.
    pub fn to_encrypted_password(
        &self,
        password: &SafePassword,
    ) -> Result<Vec<u8>, StealthKeyError> {
        self.to_encrypted(password.reveal().as_slice())
    }

    /// Decrypts a previously stored receiver secret.
    pub fn from_encrypted(data: &[u8], password: &[u8]) -> Result<Self, StealthKeyError> {
        if data.len() < 1 + SALT_LEN {
            return Err(StealthKeyError::InvalidEnvelope);
        }
        if data[0] != SEC_VER_1 {
            return Err(StealthKeyError::UnsupportedVersion);
        }

        let salt: [u8; SALT_LEN] = data[1..1 + SALT_LEN]
            .try_into()
            .map_err(|_| StealthKeyError::InvalidEnvelope)?;
        let envelope = &data[1 + SALT_LEN..];
        let key = Zeroizing::new(derive_storage_key(password, &salt)?);

        let plaintext = Zeroizing::new(
            aead::open(&key, ENC_AAD, envelope).map_err(|_| StealthKeyError::DecryptFailed)?,
        );
        if plaintext.len() != 32 {
            return Err(StealthKeyError::InvalidEnvelope);
        }

        let bytes: [u8; 32] = plaintext
            .as_slice()
            .try_into()
            .map_err(|_| StealthKeyError::InvalidEnvelope)?;
        Self::from_raw(bytes)
    }

    /// Decrypts a previously stored receiver secret using a `SafePassword` wrapper.
    pub fn from_encrypted_password(
        data: &[u8],
        password: &SafePassword,
    ) -> Result<Self, StealthKeyError> {
        Self::from_encrypted(data, password.reveal().as_slice())
    }

    /// Persists the encrypted secret to disk.
    pub fn store(&self, path: &Path, password: &[u8]) -> Result<(), StealthKeyError> {
        let encrypted = self.to_encrypted(password)?;
        write_file(path, &encrypted)?;
        Ok(())
    }

    /// Persists the encrypted secret to disk using a `SafePassword` wrapper.
    pub fn store_password(
        &self,
        path: &Path,
        password: &SafePassword,
    ) -> Result<(), StealthKeyError> {
        let encrypted = self.to_encrypted_password(password)?;
        write_file(path, &encrypted)?;
        Ok(())
    }

    /// Loads and decrypts a receiver secret from disk.
    pub fn load(path: &Path, password: &[u8]) -> Result<Self, StealthKeyError> {
        let data = read_file(path)?;
        Self::from_encrypted(&data, password)
    }

    /// Loads and decrypts a receiver secret from disk using a `SafePassword` wrapper.
    pub fn load_password(path: &Path, password: &SafePassword) -> Result<Self, StealthKeyError> {
        let data = read_file(path)?;
        Self::from_encrypted_password(&data, password)
    }

    fn from_raw(bytes: [u8; 32]) -> Result<Self, StealthKeyError> {
        if bytes.ct_eq(&[0u8; 32]).unwrap_u8() == 1 {
            return Err(StealthKeyError::ZeroSecret);
        }
        let secret = Self(bytes);
        secret.validate_usable()?;
        Ok(secret)
    }

    #[cfg(test)]
    fn from_test_bytes(bytes: [u8; 32]) -> Result<Self, StealthKeyError> {
        Self::from_raw(bytes)
    }

    #[cfg(any(test, feature = "test-params-fast"))]
    /// Forces `validate_usable` to fail on the next invocation in test-only modes.
    pub fn set_fail_usable(fail: bool) {
        FAIL_USABLE.with(|flag| flag.set(fail));
    }
}

impl ConstantTimeEq for ReceiverSecret {
    fn ct_eq(&self, other: &Self) -> Choice {
        self.0.ct_eq(&other.0)
    }
}

/// Derives the stable receiver routing handle from the master secret.
pub fn derive_owner_handle(receiver_secret: &ReceiverSecret) -> [u8; 32] {
    hash_zk::<ReceiverIdDomain>("", &[receiver_secret.as_bytes()])
}

/// Derives the base view secret key from the receiver secret.
pub fn derive_view_secret_key(
    receiver_secret: &ReceiverSecret,
) -> Result<Z00ZScalar, StealthKeyError> {
    let key = hash_to_scalar_zk::<ViewKeyDomain>("", &[receiver_secret.as_bytes()])
        .map_err(|_| StealthKeyError::InvalidSecretKey)?;

    if key.as_bytes() == [0u8; 32] {
        return Err(StealthKeyError::ZeroScalarRejected);
    }

    Ok(key)
}

/// Metadata describing a derived view-key version.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ViewKeyVersion {
    /// Monotonic view-key version number.
    pub version: u32,
    /// UNIX timestamp recorded at version creation time.
    pub timestamp: u64,
    /// Hash of the previous view key when rotation occurs.
    pub prev_hash: Option<[u8; 32]>,
}

/// Derives a versioned view secret key for key-rotation workflows.
pub fn derive_rotated_view_secret_key(
    receiver_secret: &ReceiverSecret,
    version: u32,
) -> Result<Z00ZScalar, StealthKeyError> {
    let version_bytes = version.to_le_bytes();
    let key = hash_to_scalar_zk::<WalletViewKeyHashProdDomain>(
        "VIEW_V",
        &[receiver_secret.as_bytes(), &version_bytes],
    )
    .map_err(|_| StealthKeyError::InvalidSecretKey)?;

    if key.as_bytes() == [0u8; 32] {
        return Err(StealthKeyError::ZeroScalarRejected);
    }

    Ok(key)
}

/// Creates a metadata record describing the current view-key version.
pub fn make_view_key_version(version: u32, prev_hash: Option<[u8; 32]>) -> ViewKeyVersion {
    ViewKeyVersion {
        version,
        timestamp: SystemTimeProvider.compat_unix_timestamp(),
        prev_hash,
    }
}

/// Derives the public view key corresponding to a secret view scalar.
pub fn derive_view_public_key(view_sk: &Z00ZScalar) -> Result<Z00ZRistrettoPoint, StealthKeyError> {
    let key = Z00ZRistrettoPoint::from_secret_key(view_sk);
    if key.as_bytes() == [0u8; 32] {
        return Err(StealthKeyError::IdentityPointRejected);
    }
    Ok(key)
}
