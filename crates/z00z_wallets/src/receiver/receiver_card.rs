use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine as _};
use bech32::{ToBase32, Variant};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use z00z_utils::time::{SystemTimeProvider, TimeProvider};

use z00z_crypto::expert::encoding::ByteArray;
use z00z_crypto::{
    validation::validate_scalar_nonzero, KernelSignature as Z00ZSchnorrSignature,
    Z00ZRistrettoPoint, Z00ZScalar,
};

use crate::key::{sign_identity, sign_identity_with_rng, verify_identity};

const CARD_VER_1: u8 = 1;
const CARD_SIGN_CTX: &[u8] = b"z00z.receiver.card.v1";
const STR_LEN_MAX: u32 = 1024;
/// Minimal accepted receiver card byte size.
pub const MIN_CARD_SIZE: usize = 1 + 32 + 32 + 32 + 1 + 1 + 64;
/// Max accepted receiver card byte size for untrusted parsing.
pub const MAX_CARD_SIZE: usize = 4096;

/// Errors for receiver card encoding, parsing, and verification.
#[derive(Debug, Error)]
pub enum ReceiverCardError {
    /// Card version is unsupported.
    #[error("unsupported card version")]
    UnsupportedVersion,
    /// Card byte payload is malformed.
    #[error("invalid card bytes")]
    InvalidCardBytes,
    /// Card size is outside accepted range.
    #[error("invalid card size")]
    InvalidCardSize,
    /// Optional-field flag contains invalid value.
    #[error("invalid card flag")]
    InvalidCardFlag,
    /// UTF-8 or string field is invalid.
    #[error("invalid card string")]
    InvalidCardString,
    /// Signature does not have 64 bytes.
    #[error("invalid signature length")]
    InvalidSignatureLen,
    /// Signature bytes are malformed.
    #[error("invalid signature")]
    InvalidSignature,
    /// Signature verification failed.
    #[error("signature verify failed")]
    VerifyFailed,
    /// Card is expired based on metadata validity window.
    #[error("card expired")]
    CardExpired,
    /// Owner handle pin is explicitly revoked.
    #[error("pin revoked")]
    PinRevoked,
    /// Public key bytes are malformed.
    #[error("invalid public key")]
    InvalidPublicKey,
    /// Provided identity secret does not match card identity key.
    #[error("identity key mismatch")]
    KeyMismatch,
    /// Identity point is rejected.
    #[error("identity point rejected")]
    IdentityPoint,
    /// Underlying cryptographic operation failed.
    #[error("crypto operation failed")]
    CryptoFailed,
}

/// Optional metadata embedded in receiver card.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct CardMetadata {
    /// Unix timestamp of card creation.
    pub created_at: u64,
    /// Optional display name shown to payer.
    pub display_name: Option<String>,
    /// Optional expiration timestamp.
    pub valid_until: Option<u64>,
    /// Optional contact string.
    pub contact: Option<String>,
}

impl CardMetadata {
    /// Returns canonical byte encoding of metadata.
    pub fn canonical_encoding(&self) -> Vec<u8> {
        let mut out = Vec::new();

        encode_opt_string(&mut out, &self.display_name);

        match self.valid_until {
            Some(value) => {
                out.push(1);
                out.extend_from_slice(&value.to_le_bytes());
            }
            None => out.push(0),
        }

        encode_opt_string(&mut out, &self.contact);
        out.extend_from_slice(&self.created_at.to_le_bytes());
        out
    }
}

/// Signed receiver card used for stealth payment routing.
/// It authenticates the receiver routing surface, but by itself it does not
/// prove final spend authority or replace request approval policy.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct ReceiverCard {
    /// Protocol version.
    pub version: u8,
    /// Stable owner handle.
    pub owner_handle: [u8; 32],
    /// Compressed Ristretto view key bytes.
    pub view_pk: [u8; 32],
    /// Compressed Ristretto identity key bytes.
    pub identity_pk: [u8; 32],
    /// Optional card identifier.
    pub card_id: Option<[u8; 16]>,
    /// Optional card metadata.
    pub metadata: Option<CardMetadata>,
    /// Signature bytes (`nonce\[32\] || s\[32\]`).
    #[serde(with = "sig_serde")]
    pub signature: [u8; 64],
}

mod sig_serde {
    use serde::{Deserialize, Deserializer, Serialize, Serializer};

    pub fn serialize<S>(sig: &[u8; 64], ser: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        sig.as_slice().serialize(ser)
    }

    pub fn deserialize<'de, D>(de: D) -> Result<[u8; 64], D::Error>
    where
        D: Deserializer<'de>,
    {
        let bytes = Vec::<u8>::deserialize(de)?;
        bytes
            .as_slice()
            .try_into()
            .map_err(|_| serde::de::Error::custom("invalid signature length"))
    }
}

impl ReceiverCard {
    /// Returns canonical encoding for unsigned fields.
    pub fn canonical_encoding_unsigned(&self) -> Vec<u8> {
        let mut out = Vec::new();

        out.push(self.version);
        out.extend_from_slice(&self.owner_handle);
        out.extend_from_slice(&self.view_pk);
        out.extend_from_slice(&self.identity_pk);

        match self.card_id {
            Some(card_id) => {
                out.push(1);
                out.extend_from_slice(&card_id);
            }
            None => out.push(0),
        }

        match &self.metadata {
            Some(meta) => {
                out.push(1);
                out.extend_from_slice(&meta.canonical_encoding());
            }
            None => out.push(0),
        }

        out
    }

    /// Returns full canonical encoding including signature.
    pub fn canonical_encoding(&self) -> Vec<u8> {
        let mut out = self.canonical_encoding_unsigned();
        out.extend_from_slice(&self.signature);
        out
    }

    /// Parses card from canonical bytes.
    pub fn from_canonical_encoding(bytes: &[u8]) -> Result<Self, ReceiverCardError> {
        let mut pos = 0usize;

        let version = read_u8(bytes, &mut pos)?;
        let owner_handle = read_arr::<32>(bytes, &mut pos)?;
        let view_pk = read_arr::<32>(bytes, &mut pos)?;
        let identity_pk = read_arr::<32>(bytes, &mut pos)?;

        let card_id = parse_card_id(bytes, &mut pos)?;
        let metadata = parse_card_meta(bytes, &mut pos)?;

        let signature = read_arr::<64>(bytes, &mut pos)?;

        if pos != bytes.len() {
            return Err(ReceiverCardError::InvalidCardBytes);
        }

        Ok(Self {
            version,
            owner_handle,
            view_pk,
            identity_pk,
            card_id,
            metadata,
            signature,
        })
    }

    /// Signs card with identity key.
    pub fn sign_with_rng<R>(
        &mut self,
        identity_sk: &Z00ZScalar,
        rng: &mut R,
    ) -> Result<(), ReceiverCardError>
    where
        R: rand::CryptoRng + rand::RngCore,
    {
        self.validate_structure()?;
        self.validate_ecc_points()?;

        let expected_pk = Z00ZRistrettoPoint::from_secret_key(identity_sk);
        if expected_pk.as_bytes() != self.identity_pk {
            return Err(ReceiverCardError::KeyMismatch);
        }

        let msg = self.canonical_encoding_unsigned();
        let sig = sign_identity_with_rng(identity_sk, &msg, CARD_SIGN_CTX, rng)
            .map_err(|_| ReceiverCardError::CryptoFailed)?;
        self.signature = sig_to_bytes(&sig);
        Ok(())
    }

    /// Signs card with identity key.
    pub fn sign(&mut self, identity_sk: &Z00ZScalar) -> Result<(), ReceiverCardError> {
        let mut rng = z00z_utils::rng::SystemRngProvider.rng();
        self.sign_with_rng(identity_sk, &mut rng)
    }

    /// Verifies card signature.
    pub fn verify(&self) -> Result<(), ReceiverCardError> {
        self.validate_structure()?;
        self.validate_ecc_points()?;

        let msg = self.canonical_encoding_unsigned();
        let identity_pk = decode_card_public_key(&self.identity_pk)?;
        let sig = sig_from_bytes(&self.signature)?;
        verify_identity(&identity_pk, &msg, CARD_SIGN_CTX, &sig)
            .map_err(|_| ReceiverCardError::VerifyFailed)?;

        if self.is_expired() {
            return Err(ReceiverCardError::CardExpired);
        }

        Ok(())
    }

    /// Parses card from untrusted bytes with strict bounds.
    pub fn from_untrusted_bytes(bytes: &[u8]) -> Result<Self, ReceiverCardError> {
        if bytes.len() < MIN_CARD_SIZE || bytes.len() > MAX_CARD_SIZE {
            return Err(ReceiverCardError::InvalidCardSize);
        }

        let card = Self::from_canonical_encoding(bytes)?;
        card.validate_structure()?;
        card.validate_ecc_points()?;
        Ok(card)
    }
}

/// Validation trait for receiver card checks.
pub trait ValidateReceiverCard {
    /// Validates structural invariants and version.
    fn validate_structure(&self) -> Result<(), ReceiverCardError>;
    /// Validates signature correctness.
    fn validate_signature(&self) -> Result<(), ReceiverCardError>;
    /// Validates encoded elliptic-curve points.
    fn validate_ecc_points(&self) -> Result<(), ReceiverCardError>;
}

impl ValidateReceiverCard for ReceiverCard {
    fn validate_structure(&self) -> Result<(), ReceiverCardError> {
        if self.version != CARD_VER_1 {
            return Err(ReceiverCardError::UnsupportedVersion);
        }

        if self.signature.len() != 64 {
            return Err(ReceiverCardError::InvalidSignatureLen);
        }

        Ok(())
    }

    fn validate_signature(&self) -> Result<(), ReceiverCardError> {
        self.verify()
    }

    fn validate_ecc_points(&self) -> Result<(), ReceiverCardError> {
        decode_card_public_key(&self.view_pk)?;
        decode_card_public_key(&self.identity_pk)?;

        Ok(())
    }
}

include!("receiver_card_codec.rs");

impl ReceiverCard {
    fn is_expired(&self) -> bool {
        self.metadata
            .as_ref()
            .and_then(|meta| meta.valid_until)
            .is_some_and(|valid_until| current_unix_timestamp_fail_closed() >= valid_until)
    }
}

#[cfg(test)]
#[path = "test_receiver_card.rs"]
mod test_receiver_card;
