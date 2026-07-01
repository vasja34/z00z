//! Z00Z Protocol Type Definitions and Constants
//!
//! This module centralizes protocol-level constants and type aliases used throughout
//! the Z00Z ecosystem. These definitions are independent of specific implementations
//! and provide a stable foundation for protocol evolution.
//!
//! **Last Reviewed:** 2026-02-06  
//! **Reviewer:** Vadim M  
//! **Scope:** Type definitions, validation helpers, protocol constants  
//! **Findings:** Phase 1-7 security issues addressed, 0 critical remaining
//!
//! 📌 # Zero-Amount Outputs
//!
//! 📌 Z00Z may support zero-amount outputs for specific protocol use cases:
//! - Marker outputs (metadata-only)
//! - Burn outputs (provable destruction)
//! - Null outputs (protocol placeholders)
//!
//! ⚠️ Security: allowing zero-value outputs can enable spam/DoS if unbounded.
//! Implementations MUST enforce size limits and validation rules per output type.

use crate::CryptoError;
use core::convert::{TryFrom, TryInto};
use tari_crypto::ristretto::pedersen::PedersenCommitment;
use tari_crypto::ristretto::{RistrettoPublicKey, RistrettoSecretKey};
use tari_crypto::tari_utilities::ByteArray;

mod crypto_constants;
mod protocol_constants;
mod scalar_type;
mod types_validation;

pub use crypto_constants::{
    AGGREGATION_FACTOR, MAX_BATCH_MEMORY, MAX_BATCH_PROOF_COUNT, MAX_PROOF_SIZE,
    MAX_PROOF_SIZE_EXTENDED, MIN_VALUE_PROMISE, RANGE_PROOF_BITS, RANGE_PROOF_BITS_EXTENDED,
};
pub use protocol_constants::{CHECKSUM_BYTES, LENGTH_BYTES, VERSION, VERSION_BYTES};
pub use scalar_type::Z00ZScalar;
pub use types_validation::{
    validate_amount, validate_amount_relaxed, validate_asset_amount, validate_commitment_non_zero,
    validate_proof_size, validate_transfer_amount,
};

/// Z00Z Ristretto point wrapper (SPEC Phase 0).
#[derive(Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[repr(transparent)]
pub struct Z00ZRistrettoPoint(pub(crate) RistrettoPublicKey);

impl Z00ZRistrettoPoint {
    pub fn generator() -> Self {
        use tari_crypto::keys::PublicKey as _;
        let one = RistrettoSecretKey::from(1u64);
        Self(RistrettoPublicKey::from_secret_key(&one))
    }

    pub fn identity() -> Self {
        Self(RistrettoPublicKey::default())
    }

    /// Wrap a backend public key.
    pub fn from_ristretto_public_key(pk: RistrettoPublicKey) -> Self {
        Self(pk)
    }

    /// Parse a canonical compressed point.
    pub fn try_from_bytes(bytes: [u8; 32]) -> Result<Self, CryptoError> {
        if bytes == [0u8; 32] {
            return Err(CryptoError::IdentityPoint);
        }

        let pk = RistrettoPublicKey::from_canonical_bytes(&bytes)
            .map_err(|_| CryptoError::InvalidPoint)?;
        Ok(Self(pk))
    }

    /// Backward-compatible constructor from byte slice.
    #[deprecated(note = "use Z00ZRistrettoPoint::try_from_bytes([u8; 32])")]
    pub fn from_canonical_bytes(bytes: &[u8]) -> Result<Self, CryptoError> {
        let bytes: [u8; 32] = bytes.try_into().map_err(|_| CryptoError::InvalidPoint)?;
        Self::try_from_bytes(bytes)
    }

    /// Compute `P = s * G`.
    pub fn from_scalar(scalar: &Z00ZScalar) -> Result<Self, CryptoError> {
        if scalar.is_zero() {
            return Err(CryptoError::ZeroScalar);
        }

        // `from_secret_key` is provided via the `PublicKey` trait.
        use tari_crypto::keys::PublicKey as _;
        Ok(Self(RistrettoPublicKey::from_secret_key(scalar.inner())))
    }

    /// Public key derivation helper.
    pub fn from_secret_key(scalar: &Z00ZScalar) -> Self {
        use tari_crypto::keys::PublicKey as _;
        Self(RistrettoPublicKey::from_secret_key(scalar.inner()))
    }

    pub fn as_bytes(&self) -> &[u8] {
        self.0.as_bytes()
    }

    pub fn to_bytes(&self) -> [u8; 32] {
        let mut out = [0u8; 32];
        out.copy_from_slice(self.as_bytes());
        out
    }

    pub fn ct_eq(&self, other: &Self) -> bool {
        use subtle::ConstantTimeEq;
        self.as_bytes().ct_eq(other.as_bytes()).into()
    }

    pub fn is_identity(&self) -> bool {
        self.ct_eq(&Self::identity())
    }

    pub fn compress(&self) -> Z00ZCompressedRistretto {
        Z00ZCompressedRistretto::from_point(self)
    }

    /// Reveal inner point for interoperability call sites.
    pub fn reveal(&self) -> &RistrettoPublicKey {
        &self.0
    }

    pub(crate) fn inner(&self) -> &RistrettoPublicKey {
        &self.0
    }
}

impl core::fmt::Debug for Z00ZRistrettoPoint {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "Z00ZRistrettoPoint({})", self.0)
    }
}

impl core::ops::Add for &Z00ZRistrettoPoint {
    type Output = Z00ZRistrettoPoint;

    fn add(self, rhs: Self) -> Self::Output {
        Z00ZRistrettoPoint(&self.0 + &rhs.0)
    }
}

impl core::ops::Sub for &Z00ZRistrettoPoint {
    type Output = Z00ZRistrettoPoint;

    fn sub(self, rhs: Self) -> Self::Output {
        Z00ZRistrettoPoint(&self.0 - &rhs.0)
    }
}

impl core::ops::Mul<&Z00ZScalar> for &Z00ZRistrettoPoint {
    type Output = Z00ZRistrettoPoint;

    fn mul(self, rhs: &Z00ZScalar) -> Self::Output {
        Z00ZRistrettoPoint(&self.0 * rhs.inner())
    }
}

impl core::ops::Neg for &Z00ZRistrettoPoint {
    type Output = Z00ZRistrettoPoint;

    fn neg(self) -> Self::Output {
        let identity = Z00ZRistrettoPoint::identity();
        &identity - self
    }
}

impl TryFrom<RistrettoPublicKey> for Z00ZRistrettoPoint {
    type Error = CryptoError;

    fn try_from(value: RistrettoPublicKey) -> Result<Self, Self::Error> {
        Ok(Self(value))
    }
}

impl TryFrom<Z00ZRistrettoPoint> for RistrettoPublicKey {
    type Error = CryptoError;

    fn try_from(value: Z00ZRistrettoPoint) -> Result<Self, Self::Error> {
        Ok(value.0)
    }
}

/// Canonical 32-byte compressed Ristretto encoding (SPEC Phase 0).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[repr(transparent)]
pub struct Z00ZCompressedRistretto([u8; 32]);

impl Z00ZCompressedRistretto {
    pub fn from_bytes(bytes: &[u8; 32]) -> Self {
        Self(*bytes)
    }

    pub fn to_bytes(&self) -> [u8; 32] {
        self.0
    }

    pub fn try_from_bytes(bytes: [u8; 32]) -> Result<Self, CryptoError> {
        // Validate encoding by attempting to parse as a point.
        let _ = Z00ZRistrettoPoint::try_from_bytes(bytes)?;
        Ok(Self(bytes))
    }

    pub fn from_point(point: &Z00ZRistrettoPoint) -> Self {
        Self(point.to_bytes())
    }

    pub fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }

    pub fn decompress(&self) -> Option<Z00ZRistrettoPoint> {
        Z00ZRistrettoPoint::try_from_bytes(self.0).ok()
    }

    pub fn ct_eq(&self, other: &Self) -> bool {
        use subtle::ConstantTimeEq;
        self.0.ct_eq(&other.0).into()
    }
}

impl ByteArray for Z00ZCompressedRistretto {
    fn from_canonical_bytes(
        bytes: &[u8],
    ) -> Result<Self, tari_crypto::tari_utilities::ByteArrayError> {
        if bytes.len() != 32 {
            return Err(tari_crypto::tari_utilities::ByteArrayError::IncorrectLength {});
        }

        let mut out = [0u8; 32];
        out.copy_from_slice(bytes);
        Self::try_from_bytes(out).map_err(|_| {
            tari_crypto::tari_utilities::ByteArrayError::ConversionError {
                reason: "invalid compressed ristretto encoding".to_string(),
            }
        })
    }

    fn as_bytes(&self) -> &[u8] {
        &self.0
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Z00ZBasepointTable;

impl Z00ZBasepointTable {
    pub const GENERATOR_TABLE: Self = Self;

    pub fn multiply(&self, scalar: &Z00ZScalar) -> Z00ZRistrettoPoint {
        let _ = self;
        Z00ZRistrettoPoint::from_secret_key(scalar)
    }
}

/// Z00Z wrapper for commitments.
///
/// # Purpose
///
/// Provides backend isolation by wrapping Tari types.
/// Allows future backend swaps without breaking protocol.
#[derive(Debug, Clone, PartialEq, Eq, Default, serde::Serialize, serde::Deserialize)]
#[repr(transparent)]
pub struct Z00ZCommitment(pub(crate) PedersenCommitment);

impl Z00ZCommitment {
    /// Create from inner type.
    pub fn from_commitment(commitment: PedersenCommitment) -> Self {
        Z00ZCommitment(commitment)
    }

    /// Backward-compatible constructor from canonical bytes.
    #[deprecated(
        note = "decode with PedersenCommitment::from_canonical_bytes(bytes) and wrap with Z00ZCommitment::from_commitment"
    )]
    pub fn from_canonical_bytes(bytes: &[u8]) -> Result<Self, CryptoError> {
        use tari_crypto::tari_utilities::ByteArray as _;
        let commitment = PedersenCommitment::from_canonical_bytes(bytes)
            .map_err(|_| CryptoError::InvalidCommitment)?;
        Ok(Self::from_commitment(commitment))
    }

    /// Reveal inner commitment for crypto operations.
    pub fn reveal(&self) -> &PedersenCommitment {
        &self.0
    }

    /// Access underlying public key view of the commitment.
    pub fn as_public_key(&self) -> &tari_crypto::ristretto::RistrettoPublicKey {
        self.0.as_public_key()
    }

    /// Borrow commitment bytes without allocation.
    pub fn as_bytes(&self) -> &[u8] {
        self.0.as_bytes()
    }

    /// Convert to bytes.
    pub fn to_bytes(&self) -> Vec<u8> {
        self.0.as_bytes().to_vec()
    }
}

impl core::ops::Add for &Z00ZCommitment {
    type Output = Z00ZCommitment;

    fn add(self, other: Self) -> Self::Output {
        Z00ZCommitment::from_commitment(&self.0 + &other.0)
    }
}

impl core::ops::Sub for &Z00ZCommitment {
    type Output = Z00ZCommitment;

    fn sub(self, other: Self) -> Self::Output {
        Z00ZCommitment::from_commitment(&self.0 - &other.0)
    }
}

#[cfg(all(test, not(target_arch = "wasm32")))]
mod test_types;
