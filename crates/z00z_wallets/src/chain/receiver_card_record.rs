use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine as _};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use z00z_crypto::DomainHasher;
use z00z_utils::codec::{BincodeCodec, Codec};

use crate::{
    domains::CardEntryDomain,
    receiver::{ReceiverCard, ReceiverCardError},
};

const RECORD_TAG: &str = "z00zrc1:";
const RECORD_VER_1: u8 = 1;

/// Revocation state for a published receiver-card record.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RevocationState {
    /// Record is active.
    Active,
    /// Record is revoked.
    Revoked,
}

/// Canonical live publication record for a verified receiver card.
///
/// `ReceiverCardRecord` is the one supported published receiver-card contract used by
/// wallet RPC and persistence flows. New code must not introduce a parallel alternate live
/// record type without an explicit migration and retirement plan.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReceiverCardRecord {
    /// Record version.
    pub version: u8,
    /// Canonical receiver-card bytes.
    pub receiver_card_bytes: Vec<u8>,
    /// Monotonic publication epoch.
    pub card_epoch: u64,
    /// Stable entry id for this published record.
    pub registry_entry_id: [u8; 32],
    /// Revocation state.
    pub revocation_state: RevocationState,
}

/// Errors for Stage-2 record creation and verification.
#[derive(Debug, Error)]
pub enum CardRecordError {
    /// Record version is unsupported.
    #[error("unsupported receiver-card record version")]
    UnsupportedVersion,
    /// Embedded card bytes are malformed or non-canonical.
    #[error("invalid receiver-card record bytes")]
    InvalidCardBytes,
    /// Compact record text is malformed or unsupported.
    #[error("invalid receiver-card compact text")]
    InvalidCompact,
    /// Embedded receiver card is invalid.
    #[error(transparent)]
    InvalidCard(#[from] ReceiverCardError),
    /// Registry entry id does not match the canonical binding.
    #[error("receiver-card record id mismatch")]
    BadEntryId,
    /// Record is revoked.
    #[error("receiver-card record is revoked")]
    Revoked,
    /// Record epoch is stale.
    #[error("receiver-card record epoch is stale")]
    StaleEpoch,
    /// Stored record relabeling was detected.
    #[error("receiver-card record relabel detected")]
    Relabel,
}

impl ReceiverCardRecord {
    /// Build a new active record from a verified receiver card.
    pub fn new(
        card: &ReceiverCard,
        receiver_card_bytes: Vec<u8>,
        card_epoch: u64,
    ) -> Result<Self, CardRecordError> {
        card.verify()?;

        let expect = card.canonical_encoding();
        if receiver_card_bytes != expect {
            return Err(CardRecordError::InvalidCardBytes);
        }

        Ok(Self {
            version: RECORD_VER_1,
            registry_entry_id: entry_id(&receiver_card_bytes, card_epoch),
            receiver_card_bytes,
            card_epoch,
            revocation_state: RevocationState::Active,
        })
    }

    /// Return a revoked copy of the record.
    pub fn revoked(mut self) -> Self {
        self.revocation_state = RevocationState::Revoked;
        self
    }

    /// Decode the embedded receiver card.
    pub fn decode_card(&self) -> Result<ReceiverCard, CardRecordError> {
        ReceiverCard::from_untrusted_bytes(&self.receiver_card_bytes).map_err(Into::into)
    }

    /// Render the embedded card as compact publication text.
    pub fn to_compact(&self) -> Result<String, CardRecordError> {
        let _ = verify_receiver_card_record(self, None)?;
        let raw = BincodeCodec
            .serialize(self)
            .map_err(|_| CardRecordError::InvalidCompact)?;
        Ok(format!("{RECORD_TAG}{}", URL_SAFE_NO_PAD.encode(raw)))
    }

    /// Parse and verify a compact publication record.
    pub fn from_compact(compact: &str, last_epoch: Option<u64>) -> Result<Self, CardRecordError> {
        let payload = compact
            .strip_prefix(RECORD_TAG)
            .ok_or(CardRecordError::InvalidCompact)?;
        let raw = URL_SAFE_NO_PAD
            .decode(payload)
            .map_err(|_| CardRecordError::InvalidCompact)?;
        let record: Self = BincodeCodec
            .deserialize(&raw)
            .map_err(|_| CardRecordError::InvalidCompact)?;
        let _ = verify_receiver_card_record(&record, last_epoch)?;
        Ok(record)
    }
}

/// Verify a Stage-2 record before downstream consumption.
pub fn verify_receiver_card_record(
    record: &ReceiverCardRecord,
    last_epoch: Option<u64>,
) -> Result<ReceiverCard, CardRecordError> {
    if record.version != RECORD_VER_1 {
        return Err(CardRecordError::UnsupportedVersion);
    }

    let card = record.decode_card()?;
    card.verify()?;

    if record.registry_entry_id != entry_id(&record.receiver_card_bytes, record.card_epoch) {
        return Err(CardRecordError::BadEntryId);
    }

    if record.revocation_state == RevocationState::Revoked {
        return Err(CardRecordError::Revoked);
    }

    if last_epoch.is_some_and(|epoch| record.card_epoch < epoch) {
        return Err(CardRecordError::StaleEpoch);
    }

    Ok(card)
}

/// Reject relabeling of a stored record across ids or identities.
pub fn check_relabel(
    left: &ReceiverCardRecord,
    right: &ReceiverCardRecord,
) -> Result<(), CardRecordError> {
    let left_card = left.decode_card()?;
    left_card.verify()?;
    let right_card = right.decode_card()?;
    right_card.verify()?;

    let same_owner = left_card.owner_handle == right_card.owner_handle;
    let same_identity = left_card.identity_pk == right_card.identity_pk;
    if left.registry_entry_id == right.registry_entry_id && !(same_owner && same_identity) {
        return Err(CardRecordError::Relabel);
    }

    let _ = verify_receiver_card_record(left, None)?;
    let _ = verify_receiver_card_record(right, None)?;

    if left.receiver_card_bytes == right.receiver_card_bytes
        && left.registry_entry_id != right.registry_entry_id
    {
        return Err(CardRecordError::Relabel);
    }

    Ok(())
}

fn entry_id(receiver_card_bytes: &[u8], card_epoch: u64) -> [u8; 32] {
    let hash = DomainHasher::<CardEntryDomain>::new_with_label("card_entry")
        .chain([RECORD_VER_1])
        .chain(card_epoch.to_le_bytes())
        .chain(receiver_card_bytes)
        .finalize();

    let mut out = [0u8; 32];
    out.copy_from_slice(&hash.as_ref()[..32]);
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::key::{ReceiverKeys, ReceiverSecret};

    fn make_keys() -> ReceiverKeys {
        let secret = ReceiverSecret::generate().expect("secret");
        ReceiverKeys::from_receiver_secret(secret).expect("keys")
    }

    fn make_record(card_epoch: u64) -> ReceiverCardRecord {
        let keys = make_keys();
        let card = keys.export_receiver_card().expect("card");
        ReceiverCardRecord::new(&card, card.canonical_encoding(), card_epoch).expect("record")
    }

    #[test]
    fn test_record_ok() {
        let record = make_record(7);
        let card = verify_receiver_card_record(&record, None).expect("verified");
        assert_eq!(card.canonical_encoding(), record.receiver_card_bytes);
    }

    #[test]
    fn test_roundtrip_compact() {
        let record = make_record(7);
        let compact = record.to_compact().expect("compact");
        let roundtrip = ReceiverCardRecord::from_compact(&compact, None).expect("record");
        assert_eq!(roundtrip, record);
    }

    #[test]
    fn test_bad_owner() {
        let keys = make_keys();
        let card = keys.export_receiver_card().expect("card");
        let mut bytes = card.canonical_encoding();
        bytes[1] ^= 0x01;

        let record = ReceiverCardRecord {
            version: RECORD_VER_1,
            registry_entry_id: entry_id(&bytes, 1),
            receiver_card_bytes: bytes,
            card_epoch: 1,
            revocation_state: RevocationState::Active,
        };

        assert!(matches!(
            verify_receiver_card_record(&record, None),
            Err(CardRecordError::InvalidCard(
                ReceiverCardError::VerifyFailed
            ))
        ));
    }

    #[test]
    fn test_bad_view() {
        let keys = make_keys();
        let card = keys.export_receiver_card().expect("card");
        let mut bytes = card.canonical_encoding();
        bytes[33] = 0;
        bytes[34] = 0;

        let record = ReceiverCardRecord {
            version: RECORD_VER_1,
            registry_entry_id: entry_id(&bytes, 1),
            receiver_card_bytes: bytes,
            card_epoch: 1,
            revocation_state: RevocationState::Active,
        };

        assert!(matches!(
            verify_receiver_card_record(&record, None),
            Err(CardRecordError::InvalidCard(
                ReceiverCardError::IdentityPoint
                    | ReceiverCardError::InvalidPublicKey
                    | ReceiverCardError::VerifyFailed
            ))
        ));
    }

    #[test]
    fn test_bad_identity() {
        let keys = make_keys();
        let card = keys.export_receiver_card().expect("card");
        let mut bytes = card.canonical_encoding();
        bytes[65] = 0;
        bytes[66] = 0;

        let record = ReceiverCardRecord {
            version: RECORD_VER_1,
            registry_entry_id: entry_id(&bytes, 1),
            receiver_card_bytes: bytes,
            card_epoch: 1,
            revocation_state: RevocationState::Active,
        };

        assert!(matches!(
            verify_receiver_card_record(&record, None),
            Err(CardRecordError::InvalidCard(
                ReceiverCardError::IdentityPoint
                    | ReceiverCardError::InvalidPublicKey
                    | ReceiverCardError::VerifyFailed
            ))
        ));
    }

    #[test]
    fn test_bad_sig() {
        let keys = make_keys();
        let card = keys.export_receiver_card().expect("card");
        let mut bytes = card.canonical_encoding();
        let last = bytes.len() - 1;
        bytes[last] ^= 0x01;

        let record = ReceiverCardRecord {
            version: RECORD_VER_1,
            registry_entry_id: entry_id(&bytes, 1),
            receiver_card_bytes: bytes,
            card_epoch: 1,
            revocation_state: RevocationState::Active,
        };

        assert!(matches!(
            verify_receiver_card_record(&record, None),
            Err(CardRecordError::InvalidCard(
                ReceiverCardError::VerifyFailed
            ))
        ));
    }

    #[test]
    fn test_stale_epoch() {
        let record = make_record(2);
        assert!(matches!(
            verify_receiver_card_record(&record, Some(3)),
            Err(CardRecordError::StaleEpoch)
        ));
    }

    #[test]
    fn test_revoked() {
        let record = make_record(4).revoked();
        assert!(matches!(
            verify_receiver_card_record(&record, None),
            Err(CardRecordError::Revoked)
        ));
    }

    #[test]
    fn test_rotated_epoch() {
        let secret = ReceiverSecret::generate().expect("secret");
        let mut keys = ReceiverKeys::from_receiver_secret(secret).expect("keys");
        let next = keys.rotate_view().expect("next");
        let record = ReceiverCardRecord::new(&next, next.canonical_encoding(), 8).expect("record");

        let card = verify_receiver_card_record(&record, Some(7)).expect("verified");
        assert_eq!(card.owner_handle, next.owner_handle);
        assert_eq!(card.identity_pk, next.identity_pk);
    }

    #[test]
    fn test_relabel_id() {
        let mut left = make_record(9);
        let right = left.clone();
        left.registry_entry_id[0] ^= 0x01;

        assert!(matches!(
            check_relabel(&left, &right),
            Err(CardRecordError::BadEntryId)
        ));
    }

    #[test]
    fn test_relabel_card() {
        let left = make_record(5);
        let other_keys = make_keys();
        let other = other_keys.export_receiver_card().expect("other card");
        let mut right =
            ReceiverCardRecord::new(&other, other.canonical_encoding(), 6).expect("record");
        right.registry_entry_id = left.registry_entry_id;

        assert!(matches!(
            check_relabel(&left, &right),
            Err(CardRecordError::Relabel)
        ));
    }

    #[test]
    fn test_unframed_compact_rejected() {
        let keys = make_keys();
        let card = keys.export_receiver_card().expect("card");
        let unframed_compact = URL_SAFE_NO_PAD.encode(card.canonical_encoding());

        assert!(matches!(
            ReceiverCardRecord::from_compact(&unframed_compact, None),
            Err(CardRecordError::InvalidCompact)
        ));
    }
}
