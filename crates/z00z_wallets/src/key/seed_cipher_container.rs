/// Encrypted seed container with birthday metadata.
///
/// # Example
///
/// ```
/// use z00z_wallets::key::{Argon2idParams, CipherSeedContainer, CipherSeedError};
/// use z00z_crypto::expert::encoding::SafePassword;
/// use z00z_core::genesis::ChainType;
///
/// let password = SafePassword::from("my-secret-password");
/// let wallet_id = b"wallet-uuid";
/// let purpose = b"seed-purpose";
/// let birthday = CipherSeedContainer::birthday_from_unix_seconds_checked(1700000000)?;
/// let seed = [42u8; 64]; // Your seed bytes
///
/// // Encrypt with default parameters (auto-adapted to hardware)
/// let container = CipherSeedContainer::encrypt_wallet(
///     &password,
///     wallet_id,
///     purpose,
///     birthday,
///     ChainType::Devnet,
///     &seed,
///     None,
/// )?;
///
/// // Or encrypt with custom parameters
/// let custom_params = Argon2idParams::MOBILE;
/// let container_mobile = CipherSeedContainer::encrypt_wallet(
///     &password,
///     wallet_id,
///     purpose,
///     birthday,
///     ChainType::Devnet,
///     &seed,
///     Some(custom_params),
/// )?;
///
/// // Decrypt
/// let decrypted = container.decrypt_wallet(&password, wallet_id, purpose, ChainType::Devnet)?;
/// decrypted.with_revealed(|seed_bytes| assert_eq!(seed_bytes.as_bytes(), &seed));
/// # Ok::<_, CipherSeedError>(())
/// ```
///
/// # Security Guarantees
/// - All metadata (birthday, kdf_params) is authenticated via AEAD
/// - No standalone checksum - relies on AEAD integrity only
/// - KDF parameters are validated with strict upper bounds to prevent DoS
/// - AAD is deterministic and versioned (includes chain, wallet_id, and purpose)
/// - wallet_id and purpose must be <= 255 bytes
impl CipherSeedContainer {
    /// Current (and only) supported container version.
    pub const VERSION: u8 = 1;
    /// AAD format version.
    pub const AAD_VERSION: u8 = 1;

    /// Returns authentication failure error with random timing jitter.
    ///
    /// # Timing Attack Resistance
    ///
    /// Uses random delay (80-120ms) to prevent distinguishing between:
    /// - Wrong password (KDF completes, AEAD fails)
    /// - Corrupted data (early failure before KDF)
    ///
    /// Without jitter, timing difference reveals information about container validity.
    ///
    /// # Note
    /// This delay is for timing attack resistance.
    /// For DoS protection, implement rate limiting at the service layer
    /// (e.g., HTTP middleware) instead of library code.
    fn auth_err() -> CipherSeedError {
        #[cfg(not(target_arch = "wasm32"))]
        {
            use rand::Rng;
            use std::time::Duration;
            use z00z_utils::rng::SystemRngProvider;

            // Random delay 80-120ms (prevents timing oracle)
            let mut rng = SystemRngProvider.rng();
            let jitter_ms = rng.gen_range(80..=120);
            std::thread::sleep(Duration::from_millis(jitter_ms));
        }
        CipherSeedError::AuthenticationFailed("Decryption failed".to_string())
    }

    /// Builds Additional Authenticated Data (AAD) for AEAD encryption.
    ///
    /// AAD format:
    /// - version (1)
    /// - birthday (4, little-endian)
    /// - chain_len (1) + chain
    /// - wallet_id_len (1) + wallet_id
    /// - purpose_len (1) + purpose
    /// - domain_tag (32)
    ///
    /// # Security
    ///
    /// The version byte is included in both the plaintext AAD and the domain hash input to prevent
    /// cross-version AAD collisions.
    fn build_aad(
        wallet_id: &[u8],
        purpose: &[u8],
        birthday: u32,
        chain: ChainType,
    ) -> Result<Vec<u8>, CipherSeedError> {
        let chain_bytes = chain.as_str().as_bytes();

        if wallet_id.len() > MAX_WALLET_ID_LEN {
            return Err(CipherSeedError::InputTooLong {
                field: "wallet_id",
                max: MAX_WALLET_ID_LEN,
            });
        }

        if purpose.len() > MAX_PURPOSE_LEN {
            return Err(CipherSeedError::InputTooLong {
                field: "purpose",
                max: MAX_PURPOSE_LEN,
            });
        }

        // Build base AAD: version + birthday + chain + wallet_id + purpose
        let mut aad_base = Vec::with_capacity(
            1 + 4 + 1 + chain_bytes.len() + 1 + wallet_id.len() + 1 + purpose.len(),
        );
        aad_base.push(Self::AAD_VERSION);
        aad_base.extend_from_slice(&birthday.to_le_bytes());
        aad_base.push(chain_bytes.len() as u8);
        aad_base.extend_from_slice(chain_bytes);
        aad_base.push(wallet_id.len() as u8);
        aad_base.extend_from_slice(wallet_id);
        aad_base.push(purpose.len() as u8);
        aad_base.extend_from_slice(purpose);

        // Include version explicitly in domain hash to prevent cross-version AAD collisions.
        // Even though `aad_base` contains version, explicit chaining ensures proper separation.
        let hash = DomainHasher::<CipherSeedAadTagDomain>::new_with_label("cipher_seed_aad")
            .chain([Self::AAD_VERSION])
            .chain(&aad_base)
            .finalize();

        // Combine base AAD with domain-separated tag
        let mut aad = Vec::with_capacity(aad_base.len() + 32);
        aad.extend_from_slice(&aad_base);
        aad.extend_from_slice(&hash.as_ref()[..32]);
        Ok(aad)
    }

    // NOTE: Additional versions are intentionally not supported in development.

    fn nonce_from_envelope(envelope: &[u8]) -> Result<[u8; 24], CipherSeedError> {
        const ALGO_ID_LEN: usize = 1;
        const NONCE_LEN: usize = 24;

        if envelope.len() < ALGO_ID_LEN + NONCE_LEN {
            // Return generic error - this is part of ciphertext structure validation
            // which happens during authentication, so use CryptoOperationFailed here
            // (caller will map to auth_err if in decrypt context)
            return Err(CipherSeedError::CryptoOperationFailed);
        }

        let mut nonce = [0u8; NONCE_LEN];
        nonce.copy_from_slice(&envelope[ALGO_ID_LEN..ALGO_ID_LEN + NONCE_LEN]);
        Ok(nonce)
    }

    /// Convert unix seconds to birthday (days).
    /// Returns None if the result would overflow u32 (year ~8056).
    pub fn birthday_from_unix_seconds(unix_seconds: u64) -> Option<u32> {
        const SECONDS_PER_DAY: u64 = 24 * 60 * 60;
        let days = unix_seconds / SECONDS_PER_DAY;
        if days > u32::MAX as u64 {
            return None;
        }
        Some(days as u32)
    }

    /// Convert unix seconds to birthday (days), returning an error on overflow.
    pub fn birthday_from_unix_seconds_checked(unix_seconds: u64) -> Result<u32, CipherSeedError> {
        Self::birthday_from_unix_seconds(unix_seconds).ok_or(CipherSeedError::BirthdayOverflow)
    }

    /// Encrypt a seed with birthday metadata using typed AAD.
    ///
    /// Uses Z00Z crypto utilities from core/crypto/utils.rs
    ///
    /// # Arguments
    /// * `password` - Password for encryption
    /// * `wallet_id` - Wallet identifier (included in AAD)
    /// * `purpose` - Purpose of the seed (included in AAD)
    /// * `birthday` - Days since epoch
    /// * `chain` - Network binding for this seed
    /// * `plaintext_seed` - Seed bytes to encrypt
    /// * `kdf_params` - Optional KDF parameters (uses DEFAULT if None)
    ///
    /// # Errors
    ///
    /// Returns `CipherSeedError::InvalidSeedLength` if `plaintext_seed` is not exactly 64 bytes.
    pub fn encrypt_wallet(
        password: &SafePassword,
        wallet_id: &[u8],
        purpose: &[u8],
        birthday: u32,
        chain: ChainType,
        plaintext_seed: &[u8],
        kdf_params: Option<Argon2idParams>,
    ) -> Result<Self, CipherSeedError> {
        let meta = CipherSeedMeta {
            wallet_id,
            purpose,
            birthday,
            chain,
        };
        Self::encrypt_wallet_inner(password, meta, plaintext_seed, kdf_params)
    }

    fn selected_kdf_params(
        kdf_params: Option<Argon2idParams>,
    ) -> Result<Argon2idParams, CipherSeedError> {
        let selected = match kdf_params {
            Some(kdf_params) => kdf_params,
            None => {
                #[cfg(feature = "test-params-fast")]
                {
                    Argon2idParams::TEST_FAST
                }

                #[cfg(not(feature = "test-params-fast"))]
                {
                    Argon2idParams::adapt_to_hardware()
                }
            }
        };

        selected.validate()?;
        Ok(selected)
    }

    fn generate_salt() -> [u8; 32] {
        use z00z_utils::rng::{RngCoreExt, SystemRngProvider};

        let mut salt = [0u8; 32];
        let provider = SystemRngProvider;
        let mut rng = provider.rng();
        rng.fill_bytes_ext(&mut salt);
        salt
    }

    fn derive_argon2_key(
        password: &SafePassword,
        salt: &[u8; 32],
        kdf_params: Argon2idParams,
    ) -> Result<zeroize::Zeroizing<[u8; 32]>, CipherSeedError> {
        use z00z_crypto::kdf::{Argon2Params, derive_argon2id32_key};

        let params: Argon2Params = kdf_params.into();
        Ok(zeroize::Zeroizing::new(
            derive_argon2id32_key(password.reveal(), salt, &params)
                .map(|secret| secret.into_inner())
                .map_err(|_| CipherSeedError::CryptoOperationFailed)?,
        ))
    }

    fn build_encryption_payload(
        meta: CipherSeedMeta<'_>,
        kdf_params: Argon2idParams,
        plaintext_seed: &[u8],
    ) -> zeroize::Zeroizing<Vec<u8>> {
        let mut payload =
            zeroize::Zeroizing::new(Vec::with_capacity(4 + 12 + plaintext_seed.len()));
        payload.extend_from_slice(&meta.birthday.to_le_bytes());
        payload.extend_from_slice(&kdf_params.mem_kib.to_le_bytes());
        payload.extend_from_slice(&kdf_params.time.to_le_bytes());
        payload.extend_from_slice(&kdf_params.lanes.to_le_bytes());
        payload.extend_from_slice(plaintext_seed);
        payload
    }
}

include!("seed_cipher_container_crypto.rs");
