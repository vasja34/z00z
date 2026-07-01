/// Supported KDF identifiers for persisted encrypted seed containers.
///
/// Serialized form is a strict, stable lowercase token.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(into = "String", try_from = "String")]
pub enum KdfId {
    /// Argon2id KDF.
    Argon2id,
}

impl ConstantTimeEq for KdfId {
    fn ct_eq(&self, other: &Self) -> Choice {
        self.as_str().as_bytes().ct_eq(other.as_str().as_bytes())
    }
}

impl KdfId {
    fn as_str(self) -> &'static str {
        match self {
            Self::Argon2id => "argon2id",
        }
    }
}

impl From<KdfId> for String {
    fn from(value: KdfId) -> Self {
        value.as_str().to_string()
    }
}

impl TryFrom<String> for KdfId {
    type Error = CipherSeedError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        match value.as_str() {
            "argon2id" => Ok(Self::Argon2id),
            _ => Err(CipherSeedError::InvalidKdf),
        }
    }
}

impl From<KdfId> for u8 {
    fn from(value: KdfId) -> Self {
        match value {
            KdfId::Argon2id => 1,
        }
    }
}

impl TryFrom<u8> for KdfId {
    type Error = CipherSeedError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            1 => Ok(KdfId::Argon2id),
            _ => Err(CipherSeedError::InvalidKdf),
        }
    }
}

/// Supported AEAD identifiers for persisted encrypted seed containers.
///
/// Serialized form is a strict, stable lowercase token.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(into = "String", try_from = "String")]
pub enum AeadId {
    /// XChaCha20-Poly1305 AEAD.
    XChaCha20Poly1305,
}

impl ConstantTimeEq for AeadId {
    fn ct_eq(&self, other: &Self) -> Choice {
        self.as_str().as_bytes().ct_eq(other.as_str().as_bytes())
    }
}

impl AeadId {
    fn as_str(self) -> &'static str {
        match self {
            Self::XChaCha20Poly1305 => "xchacha20poly1305",
        }
    }
}

impl From<AeadId> for String {
    fn from(value: AeadId) -> Self {
        value.as_str().to_string()
    }
}

impl TryFrom<String> for AeadId {
    type Error = CipherSeedError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        match value.as_str() {
            "xchacha20poly1305" => Ok(Self::XChaCha20Poly1305),
            _ => Err(CipherSeedError::InvalidAead),
        }
    }
}

impl From<AeadId> for u8 {
    fn from(value: AeadId) -> Self {
        match value {
            AeadId::XChaCha20Poly1305 => 1,
        }
    }
}

impl TryFrom<u8> for AeadId {
    type Error = CipherSeedError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            1 => Ok(AeadId::XChaCha20Poly1305),
            _ => Err(CipherSeedError::InvalidAead),
        }
    }
}