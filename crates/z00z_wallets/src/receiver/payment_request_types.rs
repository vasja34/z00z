use serde::{Deserialize, Serialize};

/// Optional metadata included in payment request.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct RequestMetadata {
    /// Human-readable payment description.
    pub memo: Option<String>,
    /// Merchant order identifier.
    pub payment_id: Option<[u8; 16]>,
    /// Minimal confirmations expected by receiver.
    pub min_confirmations: Option<u32>,
    /// Optional return receiver route for failed payment.
    pub return_receiver: Option<String>,
    /// Creation timestamp.
    pub created_at: u64,
}

impl RequestMetadata {
    /// Returns canonical encoding of metadata.
    pub fn canonical_encoding(&self) -> Vec<u8> {
        let mut out = Vec::new();

        encode_opt_string(&mut out, &self.memo);

        match self.payment_id {
            Some(id) => {
                out.push(1);
                out.extend_from_slice(&id);
            }
            None => out.push(0),
        }

        match self.min_confirmations {
            Some(value) => {
                out.push(1);
                out.extend_from_slice(&value.to_le_bytes());
            }
            None => out.push(0),
        }

        encode_opt_string(&mut out, &self.return_receiver);
        out.extend_from_slice(&self.created_at.to_le_bytes());
        out
    }
}

/// Signed one-time payment request.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct PaymentRequest {
    /// Protocol version.
    pub version: u8,
    /// Stable owner handle.
    pub owner_handle: [u8; 32],
    /// Compressed view key bytes.
    pub view_pk: [u8; 32],
    /// Compressed identity key bytes.
    pub identity_pk: [u8; 32],
    /// Anti-DoS unique request identifier.
    pub req_id: [u8; 32],
    /// Chain identifier.
    pub chain_id: u32,
    /// Optional fixed amount.
    pub amount: Option<u64>,
    /// Expiry timestamp.
    pub expiry: u64,
    /// Optional request metadata.
    pub metadata: Option<RequestMetadata>,
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

/// Final request validation outcome.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ValidationOutcome {
    /// Request can be paid immediately.
    Approved,
    /// New receiver identity was observed (first contact).
    RequiresUserConfirmation,
    /// Receiver identity changed from pinned value.
    IdentityMismatch,
}

/// Request time validity status.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ValidityStatus {
    /// Request remains valid for given seconds.
    Valid(u64),
    /// Request is valid but expiring soon.
    ExpiringSoon(u64),
    /// Request has expired.
    Expired,
}

/// Input parameters for request generation.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct RequestParams {
    /// Optional fixed amount.
    pub amount: Option<u64>,
    /// Validity duration in seconds.
    pub expiry_seconds: u64,
    /// Optional memo text.
    pub memo: Option<String>,
    /// Optional merchant payment identifier.
    pub payment_id: Option<[u8; 16]>,
}

/// Validated request view used by payment execution.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ValidatedRequest {
    /// Request id to bind payment output.
    pub req_id: [u8; 32],
    /// Optional fixed amount from request.
    pub amount: Option<u64>,
    /// Owner handle for receiver binding.
    pub owner_handle: [u8; 32],
}

/// NFC NDEF-like record used for payment sharing.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NdefRecord {
    uri: String,
}

impl NdefRecord {
    /// Creates URI NDEF record.
    pub fn new_uri(uri: String) -> Self {
        Self { uri }
    }

    /// Returns URI payload.
    pub fn uri(&self) -> &str {
        &self.uri
    }
}

/// Validation trait for payment requests.
pub trait ValidatePaymentRequest {
    /// Validates request in strict sequence.
    fn validate_all(
        &self,
        pins: &mut PinnedReceiverCards,
        current_chain_id: u32,
    ) -> Result<ValidationOutcome, PaymentRequestError>;
}
