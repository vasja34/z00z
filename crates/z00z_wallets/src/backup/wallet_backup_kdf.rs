use serde::{Deserialize, Serialize};

/// Persisted salt-normalization rule for backup KDF records.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SaltPad {
    /// Canonical zero-padding up to 32 bytes.
    ZeroPadTo32,
}

impl SaltPad {
    fn from_ver(version: u16) -> Result<Self, CryptoError> {
        match version {
            KdfParams::VERSION => Ok(Self::ZeroPadTo32),
            _ => Err(CryptoError::InvalidParameters {
                param: "backup_kdf_version",
            }),
        }
    }

    fn matches_ver(self, version: u16) -> bool {
        matches!((self, version), (Self::ZeroPadTo32, KdfParams::VERSION))
    }
}

/// Self-describing backup KDF contract aligned with wallet `KdfParams`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BackupKdf {
    /// Persisted KDF schema version.
    pub version: u16,
    /// KDF algorithm identifier.
    pub algorithm: KdfAlgo,
    /// Raw persisted salt bytes.
    pub salt: Vec<u8>,
    /// Memory cost in bytes.
    pub mem_limit: u64,
    /// Time or iteration cost.
    pub ops_limit: u32,
    /// Parallel worker count.
    pub parallelism: u32,
    /// Explicit salt normalization rule tied to the version.
    pub salt_pad: SaltPad,
}

impl BackupKdf {
    /// Build the canonical backup KDF contract for new backups.
    pub fn default(salt: [u8; 16]) -> Self {
        Self::from_params(KdfParams::default_argon2id_with_salt(salt.to_vec()))
            .expect("default backup kdf params must stay valid")
    }

    /// Convert wallet `KdfParams` into the backup KDF wire shape.
    pub fn from_params(params: KdfParams) -> Result<Self, CryptoError> {
        let salt_pad = SaltPad::from_ver(params.version)?;
        Ok(Self {
            version: params.version,
            algorithm: params.algo,
            salt: params.salt,
            mem_limit: params.mem_limit,
            ops_limit: params.ops_limit,
            parallelism: params.parallelism,
            salt_pad,
        })
    }

    /// Convert the backup KDF wire shape back into validated wallet `KdfParams`.
    pub fn to_params(&self) -> Result<KdfParams, CryptoError> {
        if !self.salt_pad.matches_ver(self.version) {
            return Err(CryptoError::InvalidParameters {
                param: "backup_kdf_salt_pad",
            });
        }

        let params = KdfParams {
            algo: self.algorithm,
            salt: self.salt.clone(),
            mem_limit: self.mem_limit,
            ops_limit: self.ops_limit,
            parallelism: self.parallelism,
            version: self.version,
        };
        params.validate_untrusted_persisted()?
        ;
        Ok(params)
    }
}