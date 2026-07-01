/// Encrypted seed container (format v1) with birthday metadata.
#[derive(Clone, PartialEq, Eq)]
pub struct CipherSeedContainer {
    /// Container format version.
    pub version: u8,
    /// Birthday (days since unix epoch).
    pub birthday: u32,
    /// KDF algorithm identifier.
    pub kdf: KdfId,
    /// KDF parameters.
    pub kdf_params: Argon2idParams,
    /// AEAD algorithm identifier.
    pub aead: AeadId,
    /// Random salt for KDF (full entropy, no duplication).
    pub salt: [u8; 32],
    /// Random nonce for AEAD.
    pub nonce: [u8; 24],
    /// Encrypted seed (includes birthday, kdf_params, and seed inside AEAD).
    pub ciphertext: Vec<u8>,
}

impl fmt::Debug for CipherSeedContainer {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("CipherSeedContainer")
            .field("version", &self.version)
            .field("birthday", &self.birthday)
            .field("kdf", &self.kdf)
            .field("kdf_params", &self.kdf_params)
            .field("aead", &self.aead)
            .field("salt", &"<redacted>")
            .field("nonce", &"<redacted>")
            .field("ciphertext_len", &self.ciphertext.len())
            .finish()
    }
}

#[derive(Clone, Copy)]
struct CipherSeedMeta<'a> {
    wallet_id: &'a [u8],
    purpose: &'a [u8],
    birthday: u32,
    chain: ChainType,
}

fn ct_eq_ciphertext(a: &[u8], b: &[u8]) -> Choice {
    const MAX_CT_LEN: usize = 512;

    let len_a = a.len();
    let len_b = b.len();

    let len_a_ok = Choice::from((len_a <= MAX_CT_LEN) as u8);
    let len_b_ok = Choice::from((len_b <= MAX_CT_LEN) as u8);
    let len_ok = len_a_ok & len_b_ok;

    let len_eq = (len_a as u32).ct_eq(&(len_b as u32));

    let mut diff = 0u8;
    for i in 0..MAX_CT_LEN {
        let a_i = a.get(i).copied().unwrap_or(0u8);
        let b_i = b.get(i).copied().unwrap_or(0u8);
        diff |= a_i ^ b_i;
    }

    let bytes_eq = diff.ct_eq(&0u8);
    len_ok & len_eq & bytes_eq
}

impl ConstantTimeEq for CipherSeedContainer {
    fn ct_eq(&self, other: &Self) -> Choice {
        self.version.ct_eq(&other.version)
            & self.birthday.ct_eq(&other.birthday)
            & self.kdf.ct_eq(&other.kdf)
            & self.kdf_params.ct_eq(&other.kdf_params)
            & self.aead.ct_eq(&other.aead)
            & self.salt.ct_eq(&other.salt)
            & self.nonce.ct_eq(&other.nonce)
            & ct_eq_ciphertext(self.ciphertext.as_slice(), other.ciphertext.as_slice())
    }
}

/// Errors for cipher seed operations.
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum CipherSeedError {
    /// Authentication or decryption failed.
    ///
    /// This variant is returned for ALL authentication and decryption failures
    /// to prevent distinguishers. The generic message prevents timing or error-based
    /// side channels that could leak information about the failure cause.
    ///
    /// Note: The random jitter (80-120ms) in `auth_err()` provides timing attack protection
    /// by preventing timing oracle that could distinguish failure causes.
    /// Production deployments MUST implement rate limiting at the API/service layer
    /// to prevent DoS attacks via thread exhaustion from repeated auth failures.
    #[error("{0}")]
    AuthenticationFailed(String),
    /// Persisted container version is not supported.
    #[error("unsupported version: {0}")]
    UnsupportedVersion(u8),
    /// Invalid version
    #[error("unsupported version")]
    InvalidVersion,
    /// Invalid KDF
    #[error("unsupported KDF")]
    InvalidKdf,
    /// Invalid AEAD
    #[error("unsupported AEAD")]
    InvalidAead,
    /// KDF parameters out of bounds
    #[error("KDF parameters out of bounds")]
    InvalidKdfParams,
    /// Birthday timestamp overflowed (Unix seconds too large)
    #[error("birthday timestamp overflowed")]
    BirthdayOverflow,
    /// Cryptographic operation failed
    #[error("cryptographic operation failed")]
    CryptoOperationFailed,

    /// Invalid persisted container format
    #[error("invalid container format")]
    InvalidFormat,
    /// Input length exceeds bounds
    #[error("input too long: {field} (max {max})")]
    InputTooLong {
        /// Name of the input field
        field: &'static str,
        /// Maximum allowed length
        max: usize,
    },
    /// Invalid seed length (must be exactly 64 bytes)
    #[error("invalid seed length: expected {expected}, got {got}")]
    InvalidSeedLength {
        /// Expected seed length
        expected: usize,
        /// Actual seed length provided
        got: usize,
    },
}

/// Canonical in-memory representation for a BIP-39 seed (64 bytes).
///
/// This value is sensitive and is always expected to be wrapped in `Hidden<T>`.
#[derive(Zeroize)]
#[zeroize(drop)]
pub struct SeedBytes([u8; 64]);

impl fmt::Debug for SeedBytes {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SeedBytes")
            .field("bytes", &"<redacted>")
            .finish()
    }
}

impl SeedBytes {
    /// Canonical seed length in bytes.
    pub const LEN: usize = 64;

    /// Constructs a `SeedBytes` from a 64-byte slice.
    pub fn from_slice(seed: &[u8]) -> Result<Self, CipherSeedError> {
        if seed.len() != Self::LEN {
            return Err(CipherSeedError::InvalidSeedLength {
                expected: Self::LEN,
                got: seed.len(),
            });
        }

        let mut bytes = [0u8; 64];
        bytes.copy_from_slice(seed);
        Ok(Self(bytes))
    }

    /// Returns a reference to the underlying 64-byte seed.
    pub fn as_bytes(&self) -> &[u8; 64] {
        &self.0
    }
}

impl AsRef<[u8; 64]> for SeedBytes {
    fn as_ref(&self) -> &[u8; 64] {
        &self.0
    }
}

impl AsRef<[u8]> for SeedBytes {
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}