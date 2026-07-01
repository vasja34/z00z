impl CipherSeedContainer {
    fn take_byte(bytes: &[u8], cursor: &mut usize) -> Result<u8, CipherSeedError> {
        let value = bytes
            .get(*cursor)
            .copied()
            .ok_or(CipherSeedError::InvalidFormat)?;
        *cursor += 1;
        Ok(value)
    }

    fn take_array<const N: usize>(
        bytes: &[u8],
        cursor: &mut usize,
    ) -> Result<[u8; N], CipherSeedError> {
        let value: [u8; N] = bytes
            .get(*cursor..*cursor + N)
            .ok_or(CipherSeedError::InvalidFormat)?
            .try_into()
            .map_err(|_| CipherSeedError::InvalidFormat)?;
        *cursor += N;
        Ok(value)
    }

    fn take_u32(bytes: &[u8], cursor: &mut usize) -> Result<u32, CipherSeedError> {
        Ok(u32::from_le_bytes(Self::take_array::<4>(bytes, cursor)?))
    }

    fn take_ciphertext(
        bytes: &[u8],
        cursor: usize,
        ct_len: usize,
    ) -> Result<Vec<u8>, CipherSeedError> {
        if bytes.len() != cursor + ct_len {
            return Err(CipherSeedError::InvalidFormat);
        }

        bytes
            .get(cursor..)
            .ok_or(CipherSeedError::InvalidFormat)
            .map(|slice| slice.to_vec())
    }

    /// Serialize the container to bytes for persistence.
    pub fn to_bytes(&self) -> Result<Vec<u8>, CipherSeedError> {
        const FIXED_LEN: usize = 1 + 4 + 1 + Argon2idParams::ENCODED_LEN + 1 + 32 + 24 + 4;
        const MAX_CIPHERTEXT_LEN: usize = 512;

        if self.version != Self::VERSION {
            return Err(CipherSeedError::InvalidVersion);
        }

        if self.ciphertext.len() > MAX_CIPHERTEXT_LEN {
            return Err(CipherSeedError::InputTooLong {
                field: "ciphertext",
                max: MAX_CIPHERTEXT_LEN,
            });
        }

        let ct_len_u32 =
            u32::try_from(self.ciphertext.len()).map_err(|_| CipherSeedError::InputTooLong {
                field: "ciphertext",
                max: u32::MAX as usize,
            })?;

        let mut out = Vec::with_capacity(FIXED_LEN + self.ciphertext.len());
        out.push(self.version);
        out.extend_from_slice(&self.birthday.to_le_bytes());
        out.push(u8::from(self.kdf));
        self.kdf_params.encode_into(&mut out);
        out.push(u8::from(self.aead));
        out.extend_from_slice(&self.salt);
        out.extend_from_slice(&self.nonce);
        out.extend_from_slice(&ct_len_u32.to_le_bytes());
        out.extend_from_slice(&self.ciphertext);
        Ok(out)
    }

    /// Deserialize the container from bytes for persistence.
    ///
    /// Rejects any version other than `VERSION`.
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, CipherSeedError> {
        const FIXED_LEN: usize = 1 + 4 + 1 + Argon2idParams::ENCODED_LEN + 1 + 32 + 24 + 4;
        const MAX_CIPHERTEXT_LEN: usize = 512;

        if bytes.len() < FIXED_LEN {
            return Err(CipherSeedError::InvalidFormat);
        }

        let mut cursor = 0usize;

        let version = Self::take_byte(bytes, &mut cursor)?;

        if version != Self::VERSION {
            return Err(CipherSeedError::UnsupportedVersion(version));
        }

        let birthday = Self::take_u32(bytes, &mut cursor)?;

        let kdf_byte = Self::take_byte(bytes, &mut cursor)?;
        let kdf = KdfId::try_from(kdf_byte)?;

        let (kdf_params, consumed) = Argon2idParams::decode_from(
            bytes.get(cursor..).ok_or(CipherSeedError::InvalidFormat)?,
        )?;
        cursor += consumed;

        let aead_byte = Self::take_byte(bytes, &mut cursor)?;
        let aead = AeadId::try_from(aead_byte)?;

        let salt = Self::take_array::<32>(bytes, &mut cursor)?;

        let nonce = Self::take_array::<24>(bytes, &mut cursor)?;

        let ct_len = Self::take_u32(bytes, &mut cursor)? as usize;

        if ct_len > MAX_CIPHERTEXT_LEN {
            return Err(CipherSeedError::InputTooLong {
                field: "ciphertext",
                max: MAX_CIPHERTEXT_LEN,
            });
        }

        let ciphertext = Self::take_ciphertext(bytes, cursor, ct_len)?;

        Ok(Self {
            version,
            birthday,
            kdf,
            kdf_params,
            aead,
            salt,
            nonce,
            ciphertext,
        })
    }
}