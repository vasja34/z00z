use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine as _};
use thiserror::Error;

use z00z_crypto::expert::encoding::ByteArray;
use z00z_crypto::{
    validation::validate_scalar_nonzero, KernelSignature as Z00ZSchnorrSignature,
    Z00ZRistrettoPoint, Z00ZScalar,
};
use z00z_utils::rng::{RngCoreExt, SystemRngProvider};
use z00z_utils::time::{SystemTimeProvider, TimeProvider};

use crate::key::{sign_identity, verify_identity, ReceiverKeys};

use super::card::receiver_card::ReceiverCard;
use super::card::receiver_card_trust::{PinCheckResult, PinnedReceiverCards};

#[path = "payment_request_parse.rs"]
mod payment_request_parse;
use payment_request_parse::ParsedRequest;

const REQ_VER_1: u8 = 1;
const REQUEST_SIGN_CTX: &[u8] = b"z00z.payment.request.v1";
const REQ_MIN_SIZE: usize = 1 + 32 + 32 + 32 + 32 + 4 + 1 + 8 + 1 + 64;
const REQ_MAX_SIZE: usize = 8192;

fn current_unix_timestamp() -> Result<u64, PaymentRequestError> {
    SystemTimeProvider
        .try_unix_timestamp()
        .map_err(|_| PaymentRequestError::ClockUnavailable)
}

fn current_unix_timestamp_fail_closed() -> u64 {
    current_unix_timestamp().unwrap_or(u64::MAX)
}

include!("payment_request_crypto.rs");
include!("payment_request_types.rs");

/// Errors for payment request encoding, validation, and signatures.
#[derive(Debug, Error)]
pub enum PaymentRequestError {
    /// Request version is not supported.
    #[error("unsupported request version")]
    UnsupportedVersion,
    /// Request size is outside accepted bounds.
    #[error("invalid request size")]
    InvalidRequestSize,
    /// Request bytes are malformed.
    #[error("invalid request bytes")]
    InvalidRequestBytes,
    /// Optional field flag is invalid.
    #[error("invalid request flag")]
    InvalidRequestFlag,
    /// UTF-8 string field is malformed.
    #[error("invalid request string")]
    InvalidRequestString,
    /// Request is bound to a different chain.
    #[error("wrong chain id")]
    WrongChainId,
    /// Request is expired.
    #[error("request expired")]
    RequestExpired,
    /// Receiver identity pin is revoked.
    #[error("request identity pin revoked")]
    PinRevoked,
    /// Request signature is malformed.
    #[error("invalid signature")]
    InvalidSignature,
    /// Request signature verification failed.
    #[error("signature verify failed")]
    VerifyFailed,
    /// Public key bytes are malformed.
    #[error("invalid public key")]
    InvalidPublicKey,
    /// Identity point is rejected.
    #[error("identity point rejected")]
    IdentityPoint,
    /// Random generator failed.
    #[error("rng failure")]
    RngFailure,
    /// System clock is unavailable for fail-closed request handling.
    #[error("request clock unavailable")]
    ClockUnavailable,
    /// Compact wire format is malformed.
    #[error("invalid compact encoding")]
    InvalidCompact,
    /// QR code generation failed.
    #[cfg(feature = "qr-codes")]
    #[error("qr generation failed")]
    Qr,
}

impl ValidatePaymentRequest for PaymentRequest {
    fn validate_all(
        &self,
        pins: &mut PinnedReceiverCards,
        current_chain_id: u32,
    ) -> Result<ValidationOutcome, PaymentRequestError> {
        // This is the accepted wallet policy gate for request use. It enforces
        // signature, chain, expiry, and TOFU or pinning rules, but it is still
        // a wallet-local approval boundary rather than a public verifier claim.
        if self.version != REQ_VER_1 {
            return Err(PaymentRequestError::UnsupportedVersion);
        }

        if self.chain_id != current_chain_id {
            return Err(PaymentRequestError::WrongChainId);
        }

        if self.is_expired() {
            return Err(PaymentRequestError::RequestExpired);
        }

        decode_pk(&self.view_pk)?;
        decode_pk(&self.identity_pk)?;

        self.verify()?;

        let pin = pins.verify_request_identity(&self.owner_handle, &self.identity_pk);
        match pin {
            PinCheckResult::Verified => Ok(ValidationOutcome::Approved),
            PinCheckResult::NewIdentity => Ok(ValidationOutcome::RequiresUserConfirmation),
            PinCheckResult::IdentityChanged => Ok(ValidationOutcome::IdentityMismatch),
            PinCheckResult::Revoked => Err(PaymentRequestError::PinRevoked),
        }
    }
}

impl PaymentRequest {
    /// Returns canonical encoding for unsigned fields.
    pub fn canonical_encoding_unsigned(&self) -> Vec<u8> {
        let mut out = Vec::new();

        out.push(self.version);
        out.extend_from_slice(&self.owner_handle);
        out.extend_from_slice(&self.view_pk);
        out.extend_from_slice(&self.identity_pk);
        out.extend_from_slice(&self.req_id);
        out.extend_from_slice(&self.chain_id.to_le_bytes());

        match self.amount {
            Some(value) => {
                out.push(1);
                out.extend_from_slice(&value.to_le_bytes());
            }
            None => out.push(0),
        }

        out.extend_from_slice(&self.expiry.to_le_bytes());

        match &self.metadata {
            Some(meta) => {
                out.push(1);
                out.extend_from_slice(&meta.canonical_encoding());
            }
            None => out.push(0),
        }

        out
    }

    /// Returns canonical encoding including signature.
    pub fn canonical_encoding(&self) -> Vec<u8> {
        let mut out = self.canonical_encoding_unsigned();
        out.extend_from_slice(&self.signature);
        out
    }

    /// Signs request with identity key.
    pub fn sign(&mut self, identity_sk: &Z00ZScalar) -> Result<(), PaymentRequestError> {
        self.validate_points()?;

        let expected_pk = Z00ZRistrettoPoint::from_secret_key(identity_sk);
        if expected_pk.as_bytes() != self.identity_pk {
            return Err(PaymentRequestError::InvalidSignature);
        }

        let msg = self.canonical_encoding_unsigned();
        let sig = sign_identity(identity_sk, &msg, REQUEST_SIGN_CTX)
            .map_err(|_| PaymentRequestError::InvalidSignature)?;
        self.signature = sig_to_bytes(&sig);
        Ok(())
    }

    /// Verifies request signature.
    pub fn verify(&self) -> Result<(), PaymentRequestError> {
        self.validate_points()?;
        let msg = self.canonical_encoding_unsigned();
        let identity_pk = decode_pk(&self.identity_pk)?;
        let sig = sig_from_bytes(&self.signature)?;

        verify_identity(&identity_pk, &msg, REQUEST_SIGN_CTX, &sig)
            .map_err(|_| PaymentRequestError::VerifyFailed)
    }

    /// Parses request from canonical bytes.
    pub fn from_canonical_encoding(bytes: &[u8]) -> Result<Self, PaymentRequestError> {
        let parsed = ParsedRequest::parse(bytes)?;
        Ok(Self {
            version: parsed.version,
            owner_handle: parsed.owner_handle,
            view_pk: parsed.view_pk,
            identity_pk: parsed.identity_pk,
            req_id: parsed.req_id,
            chain_id: parsed.chain_id,
            amount: parsed.amount,
            expiry: parsed.expiry,
            metadata: parsed.metadata,
            signature: parsed.signature,
        })
    }

    /// Parses request bytes with strict size bounds.
    pub fn from_untrusted_bytes(bytes: &[u8]) -> Result<Self, PaymentRequestError> {
        if bytes.len() < REQ_MIN_SIZE || bytes.len() > REQ_MAX_SIZE {
            return Err(PaymentRequestError::InvalidRequestSize);
        }

        let request = Self::from_canonical_encoding(bytes)?;
        request.validate_points()?;
        Ok(request)
    }

    /// Returns true when request is expired.
    pub fn is_expired(&self) -> bool {
        current_unix_timestamp_fail_closed() >= self.expiry
    }

    /// Returns signed seconds until expiry.
    pub fn remaining_seconds(&self) -> i64 {
        let now = current_unix_timestamp_fail_closed();
        let delta = (self.expiry as i128) - (now as i128);
        if delta > i64::MAX as i128 {
            i64::MAX
        } else if delta < i64::MIN as i128 {
            i64::MIN
        } else {
            delta as i64
        }
    }

    /// Returns compact request data string for QR/NFC payloads.
    pub fn to_qr_code_data(&self) -> String {
        encode_request_compact(self)
    }

    /// Returns current validity status.
    pub fn check_validity(&self) -> ValidityStatus {
        let now = current_unix_timestamp_fail_closed();
        if now >= self.expiry {
            return ValidityStatus::Expired;
        }
        let remain = self.expiry.saturating_sub(now);
        if remain < 60 {
            return ValidityStatus::ExpiringSoon(remain);
        }

        ValidityStatus::Valid(remain)
    }

    /// Generates and signs payment request.
    pub fn generate(
        keys: &ReceiverKeys,
        params: RequestParams,
        chain_id: u32,
    ) -> Result<Self, PaymentRequestError> {
        generate_request(keys, params, chain_id)
    }

    fn validate_points(&self) -> Result<(), PaymentRequestError> {
        decode_pk(&self.view_pk)?;
        decode_pk(&self.identity_pk)?;
        Ok(())
    }
}

include!("payment_request_transport.rs");

#[cfg(test)]
#[path = "test_payment_request.rs"]
mod test_payment_request;
