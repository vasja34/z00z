use serde::{Deserialize, Serialize};
use z00z_crypto::{aead, CryptoError, Hidden};

use super::{KdfParams, RedbKey32};

/// AEAD envelope using canonical format: `algo_id || nonce || ciphertext_with_tag`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AeadEnvelope {
    /// Canonical envelope bytes.
    pub envelope: Vec<u8>,
}

impl AeadEnvelope {
    /// Return the algorithm identifier encoded in the envelope.
    pub fn algo_id(&self) -> Result<u8, CryptoError> {
        if self.envelope.is_empty() {
            return Err(CryptoError::InvalidParameters { param: "envelope" });
        }
        Ok(self.envelope[0])
    }

    /// Check whether the envelope uses XChaCha20-Poly1305.
    pub fn is_xchacha20poly1305(&self) -> bool {
        self.algo_id().unwrap_or(0) == aead::XCHACHA20_POLY1305_ID
    }
}

/// Master key record stored in the wallet database.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MasterKeyRecord {
    /// Encrypted master key envelope.
    pub envelope: AeadEnvelope,
    /// Persisted KDF parameters for the master key record.
    pub kdf_params: Option<KdfParams>,
}

/// Secret kinds stored in the wallet secret table.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SecretsKind {
    /// BIP-39 seed phrase.
    Seed,
    /// Extended private key.
    Xprv,
    /// View-only key.
    ViewKey,
    /// Device binding key.
    DeviceBindingKey,
    /// Transaction signing key.
    SigningKey,
    /// Custom secret type.
    Custom,
}

/// Encrypted secret record stored in the wallet.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SecretsRecord {
    /// Secret type.
    pub kind: SecretsKind,
    /// Human-readable secret label.
    pub label: String,
    /// Secret format version.
    pub version: u16,
    /// Encrypted secret envelope.
    pub envelope: AeadEnvelope,
}

/// Derived encryption keys for wallet operations.
#[derive(Debug)]
pub struct WalletDerivedKeys {
    /// Key for encrypting wallet data.
    pub data_key: Hidden<RedbKey32>,
    /// Key for encrypting database indices.
    pub index_key: Hidden<RedbKey32>,
    /// Key for integrity verification.
    pub integrity_key: Hidden<RedbKey32>,
}

impl Clone for WalletDerivedKeys {
    fn clone(&self) -> Self {
        Self {
            data_key: self.data_key.dangerous_clone(),
            index_key: self.index_key.dangerous_clone(),
            integrity_key: self.integrity_key.dangerous_clone(),
        }
    }
}
