use std::cmp::Ordering;
use std::collections::BTreeMap;

use super::{PaymentRequest, PaymentRequestError, ScanChunk, ValidationOutcome};

/// Optional local hint about the scan range associated with one request.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RequestRangeHint {
    /// First checkpoint height associated with this request hint.
    pub start_height: u64,
    /// Last checkpoint height associated with this request hint.
    pub end_height: Option<u64>,
}

impl RequestRangeHint {
    /// Builds a range hint from a concrete scan batch.
    pub fn from_chunks(chunks: &[ScanChunk]) -> Option<Self> {
        let start_height = chunks.iter().map(|chunk| chunk.height).min()?;
        let end_height = chunks.iter().map(|chunk| chunk.height).max();
        Some(Self {
            start_height,
            end_height,
        })
    }
}

/// Wallet-local request recipient binding recorded by the advisory inbox.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RequestRecipientBinding {
    /// Stable owner handle bound to the request.
    pub owner_handle: [u8; 32],
    /// Receiver view key bytes bound to the request.
    pub view_pk: [u8; 32],
    /// Receiver identity key bytes bound to the request.
    pub identity_pk: [u8; 32],
}

/// Stable reject classes recorded by the advisory inbox.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RequestInboxReject {
    /// Request version is not supported.
    UnsupportedVersion,
    /// Request size is outside the accepted bounds.
    InvalidRequestSize,
    /// Request bytes are malformed.
    InvalidRequestBytes,
    /// Optional request flag is malformed.
    InvalidRequestFlag,
    /// One string field failed strict decoding.
    InvalidRequestString,
    /// Request chain id does not match the wallet chain.
    WrongChainId,
    /// Request expiry boundary is already elapsed.
    RequestExpired,
    /// Request owner was explicitly revoked in local TOFU state.
    PinRevoked,
    /// Request signature bytes are malformed.
    InvalidSignature,
    /// Request signature verification failed.
    VerifyFailed,
    /// Request public key bytes are malformed.
    InvalidPublicKey,
    /// Request identity point failed strict validation.
    IdentityPoint,
    /// Request generation failed because RNG was unavailable.
    RngFailure,
    /// Request validation failed because the wall clock was unavailable.
    ClockUnavailable,
    /// Compact request encoding is malformed.
    InvalidCompact,
}

impl RequestInboxReject {
    fn from_error(error: &PaymentRequestError) -> Self {
        match error {
            PaymentRequestError::UnsupportedVersion => Self::UnsupportedVersion,
            PaymentRequestError::InvalidRequestSize => Self::InvalidRequestSize,
            PaymentRequestError::InvalidRequestBytes => Self::InvalidRequestBytes,
            PaymentRequestError::InvalidRequestFlag => Self::InvalidRequestFlag,
            PaymentRequestError::InvalidRequestString => Self::InvalidRequestString,
            PaymentRequestError::WrongChainId => Self::WrongChainId,
            PaymentRequestError::RequestExpired => Self::RequestExpired,
            PaymentRequestError::PinRevoked => Self::PinRevoked,
            PaymentRequestError::InvalidSignature => Self::InvalidSignature,
            PaymentRequestError::VerifyFailed => Self::VerifyFailed,
            PaymentRequestError::InvalidPublicKey => Self::InvalidPublicKey,
            PaymentRequestError::IdentityPoint => Self::IdentityPoint,
            PaymentRequestError::RngFailure => Self::RngFailure,
            PaymentRequestError::ClockUnavailable => Self::ClockUnavailable,
            PaymentRequestError::InvalidCompact => Self::InvalidCompact,
            #[cfg(feature = "qr-codes")]
            PaymentRequestError::Qr => Self::InvalidCompact,
        }
    }

    /// Returns a stable, redacted summary string.
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::UnsupportedVersion => "unsupported request version",
            Self::InvalidRequestSize => "invalid request size",
            Self::InvalidRequestBytes => "invalid request bytes",
            Self::InvalidRequestFlag => "invalid request flag",
            Self::InvalidRequestString => "invalid request string",
            Self::WrongChainId => "wrong chain id",
            Self::RequestExpired => "request expired",
            Self::PinRevoked => "request identity pin revoked",
            Self::InvalidSignature => "invalid signature",
            Self::VerifyFailed => "signature verify failed",
            Self::InvalidPublicKey => "invalid public key",
            Self::IdentityPoint => "identity point rejected",
            Self::RngFailure => "rng failure",
            Self::ClockUnavailable => "request clock unavailable",
            Self::InvalidCompact => "invalid compact encoding",
        }
    }
}

/// Wallet-local validation result recorded by the advisory inbox.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RequestInboxValidation {
    /// Request is approved for canonical receive processing.
    Approved,
    /// Request is structurally valid but still needs explicit user confirmation.
    RequiresUserConfirmation,
    /// Request owner handle maps to a different pinned identity.
    IdentityMismatch,
    /// Request failed validation with one stable reject class.
    Rejected(RequestInboxReject),
}

impl RequestInboxValidation {
    /// Maps payment-request validation into stable inbox metadata.
    pub fn from_result(
        result: &Result<ValidationOutcome, PaymentRequestError>,
    ) -> RequestInboxValidation {
        match result {
            Ok(ValidationOutcome::Approved) => Self::Approved,
            Ok(ValidationOutcome::RequiresUserConfirmation) => Self::RequiresUserConfirmation,
            Ok(ValidationOutcome::IdentityMismatch) => Self::IdentityMismatch,
            Err(error) => Self::Rejected(RequestInboxReject::from_error(error)),
        }
    }

    /// Returns true when the request may enter the canonical receive lane.
    pub fn is_approved(&self) -> bool {
        matches!(self, Self::Approved)
    }

    /// Returns a stable, redacted summary string.
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Approved => "approved",
            Self::RequiresUserConfirmation => "request requires user confirmation",
            Self::IdentityMismatch => "request identity mismatch",
            Self::Rejected(reject) => reject.as_str(),
        }
    }
}

/// Advisory request record stored by the wallet-local inbox.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RequestInboxRecord {
    /// Stable request identifier.
    pub request_id: [u8; 32],
    /// Runtime chain identifier bound to the request.
    pub chain_id: u32,
    /// Recipient binding copied from the request.
    pub recipient: RequestRecipientBinding,
    /// Request expiry timestamp.
    pub expiry: u64,
    /// Optional local scan-range hint.
    pub range_hint: Option<RequestRangeHint>,
    /// Wallet-local validation result.
    pub validation: RequestInboxValidation,
    /// Wallet-local creation timestamp.
    pub created_at: u64,
}

impl RequestInboxRecord {
    /// Builds one advisory record from a payment request.
    pub fn from_request(
        request: &PaymentRequest,
        validation: RequestInboxValidation,
        range_hint: Option<RequestRangeHint>,
        created_at: u64,
    ) -> Self {
        Self {
            request_id: request.req_id,
            chain_id: request.chain_id,
            recipient: RequestRecipientBinding {
                owner_handle: request.owner_handle,
                view_pk: request.view_pk,
                identity_pk: request.identity_pk,
            },
            expiry: request.expiry,
            range_hint,
            validation,
            created_at,
        }
    }
}

/// Wallet-local request-bound inbox.
///
/// This helper is advisory and off-consensus. It stores validation and range
/// hints only, and it never becomes a second receive persistence authority.
#[derive(Clone, Debug, Default)]
pub struct RequestInbox {
    records: BTreeMap<[u8; 32], RequestInboxRecord>,
}

impl RequestInbox {
    /// Creates an empty advisory inbox.
    pub fn new() -> Self {
        Self::default()
    }

    /// Inserts or replaces one request record.
    pub fn upsert(
        &mut self,
        request: &PaymentRequest,
        validation: RequestInboxValidation,
        range_hint: Option<RequestRangeHint>,
        created_at: u64,
    ) -> RequestInboxRecord {
        let record = RequestInboxRecord::from_request(request, validation, range_hint, created_at);
        self.records.insert(request.req_id, record.clone());
        record
    }

    /// Inserts or replaces one record from a validation result.
    pub fn record_result(
        &mut self,
        request: &PaymentRequest,
        result: &Result<ValidationOutcome, PaymentRequestError>,
        range_hint: Option<RequestRangeHint>,
        created_at: u64,
    ) -> RequestInboxRecord {
        let validation = RequestInboxValidation::from_result(result);
        self.upsert(request, validation, range_hint, created_at)
    }

    /// Returns one record by request id.
    pub fn get(&self, request_id: &[u8; 32]) -> Option<&RequestInboxRecord> {
        self.records.get(request_id)
    }

    /// Returns all records in deterministic inbox order.
    pub fn list(&self) -> Vec<RequestInboxRecord> {
        let mut records = self.records.values().cloned().collect::<Vec<_>>();
        records.sort_by(Self::cmp_record);
        records
    }

    /// Removes one record by request id.
    pub fn remove(&mut self, request_id: &[u8; 32]) -> Option<RequestInboxRecord> {
        self.records.remove(request_id)
    }

    /// Returns the number of advisory records.
    pub fn len(&self) -> usize {
        self.records.len()
    }

    /// Returns true when the inbox is empty.
    pub fn is_empty(&self) -> bool {
        self.records.is_empty()
    }

    /// Orders approved requests using inbox metadata only.
    pub fn ordered_requests<'a>(&self, requests: &'a [PaymentRequest]) -> Vec<&'a PaymentRequest> {
        let mut ordered = requests
            .iter()
            .filter_map(|request| {
                self.records.get(&request.req_id).and_then(|record| {
                    if record.validation.is_approved() {
                        Some((record, request))
                    } else {
                        None
                    }
                })
            })
            .collect::<Vec<_>>();
        ordered.sort_by(|(left, _), (right, _)| Self::cmp_record(left, right));
        ordered
            .into_iter()
            .map(|(_, request)| request)
            .collect::<Vec<_>>()
    }

    fn cmp_record(left: &RequestInboxRecord, right: &RequestInboxRecord) -> Ordering {
        let left_hint = left.range_hint.as_ref();
        let right_hint = right.range_hint.as_ref();

        match (left_hint, right_hint) {
            (Some(left_hint), Some(right_hint)) => left_hint
                .start_height
                .cmp(&right_hint.start_height)
                .then_with(|| {
                    left_hint
                        .end_height
                        .unwrap_or(u64::MAX)
                        .cmp(&right_hint.end_height.unwrap_or(u64::MAX))
                })
                .then_with(|| left.created_at.cmp(&right.created_at))
                .then_with(|| left.request_id.cmp(&right.request_id)),
            (Some(_), None) => Ordering::Less,
            (None, Some(_)) => Ordering::Greater,
            (None, None) => left
                .created_at
                .cmp(&right.created_at)
                .then_with(|| left.request_id.cmp(&right.request_id)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_request(mark: u8) -> PaymentRequest {
        PaymentRequest {
            version: 1,
            owner_handle: [mark; 32],
            view_pk: [mark.wrapping_add(1); 32],
            identity_pk: [mark.wrapping_add(2); 32],
            req_id: [mark.wrapping_add(3); 32],
            chain_id: 3,
            amount: Some(77),
            expiry: 999,
            metadata: None,
            signature: [0u8; 64],
        }
    }

    #[test]
    fn test_orders_hints_first() {
        let mut inbox = RequestInbox::new();
        let req_a = sample_request(0x10);
        let req_b = sample_request(0x20);
        let req_c = sample_request(0x30);

        inbox.upsert(&req_a, RequestInboxValidation::Approved, None, 30);
        inbox.upsert(
            &req_b,
            RequestInboxValidation::Approved,
            Some(RequestRangeHint {
                start_height: 7,
                end_height: Some(8),
            }),
            20,
        );
        inbox.upsert(
            &req_c,
            RequestInboxValidation::Approved,
            Some(RequestRangeHint {
                start_height: 7,
                end_height: Some(7),
            }),
            10,
        );

        let ids = inbox
            .list()
            .into_iter()
            .map(|record| record.request_id)
            .collect::<Vec<_>>();

        assert_eq!(ids, vec![req_c.req_id, req_b.req_id, req_a.req_id]);
    }

    #[test]
    fn test_orders_approved_into_receive() {
        let mut inbox = RequestInbox::new();
        let req_a = sample_request(0x41);
        let req_b = sample_request(0x42);

        inbox.upsert(
            &req_a,
            RequestInboxValidation::Rejected(RequestInboxReject::WrongChainId),
            Some(RequestRangeHint {
                start_height: 8,
                end_height: Some(8),
            }),
            1,
        );
        inbox.upsert(
            &req_b,
            RequestInboxValidation::Approved,
            Some(RequestRangeHint {
                start_height: 7,
                end_height: Some(7),
            }),
            2,
        );

        let requests = [req_a.clone(), req_b.clone()];
        let ordered = inbox.ordered_requests(&requests);
        assert_eq!(ordered.len(), 1);
        assert_eq!(ordered[0].req_id, req_b.req_id);
    }
}
