impl CipherSeedContainer {
    fn derive_decryption_key(
        &self,
        password: &SafePassword,
    ) -> Result<zeroize::Zeroizing<[u8; 32]>, CipherSeedError> {
        let params: z00z_crypto::kdf::Argon2Params = self.kdf_params.into();
        Ok(zeroize::Zeroizing::new(
            z00z_crypto::kdf::derive_argon2id32_key(password.reveal(), &self.salt, &params)
                .map(|secret| secret.into_inner())
                .map_err(|_| Self::auth_err())?,
        ))
    }

    fn decrypt_payload(
        &self,
        key: &zeroize::Zeroizing<[u8; 32]>,
        wallet_id: &[u8],
        purpose: &[u8],
        chain: ChainType,
    ) -> Result<zeroize::Zeroizing<Vec<u8>>, CipherSeedError> {
        let aad_with_meta = Self::build_aad(wallet_id, purpose, self.birthday, chain)?;
        let envelope_nonce =
            Self::nonce_from_envelope(&self.ciphertext).map_err(|_| Self::auth_err())?;
        if envelope_nonce != self.nonce {
            return Err(Self::auth_err());
        }

        match self.aead {
            AeadId::XChaCha20Poly1305 => Ok(zeroize::Zeroizing::new(
                z00z_crypto::aead::open(key, &aad_with_meta, &self.ciphertext)
                    .map_err(|_| Self::auth_err())?,
            )),
        }
    }

    fn validate_payload_metadata(&self, payload: &[u8]) -> Result<(), CipherSeedError> {
        if payload.len() < 16 {
            return Err(Self::auth_err());
        }

        let birthday_bytes: [u8; 4] = payload[0..4].try_into().map_err(|_| Self::auth_err())?;
        let extracted_birthday = u32::from_le_bytes(birthday_bytes);
        if extracted_birthday != self.birthday {
            return Err(Self::auth_err());
        }

        let mem_kib_bytes: [u8; 4] = payload[4..8].try_into().map_err(|_| Self::auth_err())?;
        let time_bytes: [u8; 4] = payload[8..12].try_into().map_err(|_| Self::auth_err())?;
        let lanes_bytes: [u8; 4] = payload[12..16].try_into().map_err(|_| Self::auth_err())?;

        let extracted_mem_kib = u32::from_le_bytes(mem_kib_bytes);
        let extracted_time = u32::from_le_bytes(time_bytes);
        let extracted_lanes = u32::from_le_bytes(lanes_bytes);

        if extracted_mem_kib != self.kdf_params.mem_kib
            || extracted_time != self.kdf_params.time
            || extracted_lanes != self.kdf_params.lanes
        {
            return Err(Self::auth_err());
        }

        Ok(())
    }

    fn extract_seed(payload: &[u8]) -> Result<Hidden<SeedBytes>, CipherSeedError> {
        if payload.len() != 16 + SeedBytes::LEN {
            return Err(Self::auth_err());
        }

        let seed = SeedBytes::from_slice(&payload[16..]).map_err(|_| Self::auth_err())?;
        Ok(Hidden::hide(seed))
    }

    fn encrypt_wallet_inner(
        password: &SafePassword,
        meta: CipherSeedMeta<'_>,
        plaintext_seed: &[u8],
        kdf_params: Option<Argon2idParams>,
    ) -> Result<Self, CipherSeedError> {
        use z00z_crypto::aead::seal;

        if plaintext_seed.len() != SeedBytes::LEN {
            return Err(CipherSeedError::InvalidSeedLength {
                expected: SeedBytes::LEN,
                got: plaintext_seed.len(),
            });
        }

        let kdf_params = Self::selected_kdf_params(kdf_params)?;
        let salt = Self::generate_salt();
        let key = Self::derive_argon2_key(password, &salt, kdf_params)?;
        let aad_with_meta = Self::build_aad(meta.wallet_id, meta.purpose, meta.birthday, meta.chain)?;
        let payload = Self::build_encryption_payload(meta, kdf_params, plaintext_seed);

        let kdf = KdfId::Argon2id;
        let aead = AeadId::XChaCha20Poly1305;

        let ciphertext = match aead {
            AeadId::XChaCha20Poly1305 => seal(&key, &aad_with_meta, payload.as_slice())
                .map_err(|_| CipherSeedError::CryptoOperationFailed)?,
        };

        let nonce: [u8; 24] = ciphertext[1..25]
            .try_into()
            .map_err(|_| CipherSeedError::CryptoOperationFailed)?;

        Ok(Self {
            version: Self::VERSION,
            birthday: meta.birthday,
            kdf,
            kdf_params,
            aead,
            salt,
            nonce,
            ciphertext,
        })
    }

    /// Decrypt the seed container using typed AAD.
    ///
    /// Validates the stored KDF parameters and authenticated envelope metadata
    /// before reconstructing the hidden seed bytes.
    pub fn decrypt_wallet(
        &self,
        password: &SafePassword,
        wallet_id: &[u8],
        purpose: &[u8],
        chain: ChainType,
    ) -> Result<Hidden<SeedBytes>, CipherSeedError> {
        if self.version != Self::VERSION {
            return Err(CipherSeedError::UnsupportedVersion(self.version));
        }

        self.kdf_params.validate()?;

        let key = self.derive_decryption_key(password)?;
        let payload = self.decrypt_payload(&key, wallet_id, purpose, chain)?;
        self.validate_payload_metadata(&payload)?;
        Self::extract_seed(&payload)
    }

    /// Atomic password re-encryption (minimizes seed exposure).
    pub fn re_encrypt(
        &self,
        old_password: &SafePassword,
        new_password: &SafePassword,
        wallet_id: &[u8],
        purpose: &[u8],
        chain: ChainType,
    ) -> Result<Self, CipherSeedError> {
        let plaintext_seed = self.decrypt_wallet(old_password, wallet_id, purpose, chain)?;

        plaintext_seed.with_revealed(|seed_bytes| {
            Self::encrypt_wallet(
                new_password,
                wallet_id,
                purpose,
                self.birthday,
                chain,
                seed_bytes.as_bytes(),
                Some(self.kdf_params),
            )
        })
    }
}