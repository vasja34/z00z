//! TOFU and pinning policy for receiver cards and request identities.
//! This module owns wallet-local trust transitions such as first-seen,
//! confirmation, rotation, and revoke. These decisions are explicit policy, not
//! a substitute for external cryptographic proof of receiver authority.

use std::collections::HashMap;

use z00z_utils::time::{SystemTimeProvider, TimeProvider};

use crate::receiver::receiver_card::ValidateReceiverCard;

use super::receiver_card::{ReceiverCard, ReceiverCardError};

const UNKNOWN_VIEW_PK: [u8; 32] = [0u8; 32];

fn current_unix_timestamp_fail_closed() -> u64 {
    SystemTimeProvider.try_unix_timestamp().unwrap_or(u64::MAX)
}

/// Trust state for pinned receiver card.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TrustLevel {
    /// User confirmed and pinned entry.
    Pinned,
    /// First observed card, not yet explicitly confirmed.
    Tentative,
    /// Card is no longer valid.
    Expired,
    /// User revoked this identity.
    Revoked,
}

/// Stored pin entry for owner handle.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PinEntry {
    /// Pinned view key.
    pub view_pk: [u8; 32],
    /// Pinned identity key.
    pub identity_pk: [u8; 32],
    /// Optional directory identifier.
    pub directory_id: Option<String>,
    /// First observed timestamp.
    pub first_seen: u64,
    /// Current trust level.
    pub trust_level: TrustLevel,
}

/// Result of TOFU verify or pin flow.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum VerifyResult {
    /// New owner handle was pinned.
    NewPin,
    /// Existing pin matches incoming card.
    Verified,
    /// View key changed for existing owner handle.
    ViewKeyChanged {
        /// Previous pinned view key.
        old_pk: [u8; 32],
        /// New incoming view key.
        new_pk: [u8; 32],
        /// User confirmation is required.
        requires_confirmation: bool,
    },
    /// Identity key changed for existing owner handle.
    IdentityKeyChanged,
}

/// Result of pin check for payment request identity.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PinCheckResult {
    /// Identity matches existing pin.
    Verified,
    /// No pin existed and identity was pinned.
    NewIdentity,
    /// Pin existed but identity key changed.
    IdentityChanged,
    /// Pin exists but is revoked.
    Revoked,
}

/// TOFU store for receiver cards.
#[derive(Clone, Debug, Default)]
pub struct PinnedReceiverCards {
    pins: HashMap<[u8; 32], PinEntry>,
}

impl PinnedReceiverCards {
    /// Creates empty pin store.
    pub fn new() -> Self {
        Self {
            pins: HashMap::new(),
        }
    }

    /// Verifies incoming card against pin store or creates new tentative pin.
    pub fn verify_or_pin(
        &mut self,
        card: &ReceiverCard,
        directory_id: Option<&str>,
    ) -> Result<VerifyResult, ReceiverCardError> {
        card.validate_structure()?;
        card.validate_ecc_points()?;
        card.validate_signature()?;

        if self.is_card_expired(card) {
            return Err(ReceiverCardError::CardExpired);
        }

        if !self.pins.contains_key(&card.owner_handle) {
            self.insert_pin(card, directory_id);
            return Ok(VerifyResult::NewPin);
        }

        self.eval_pin_mut(card)
    }

    /// Verifies payment request identity against TOFU pins.
    pub fn verify_request_identity(
        &mut self,
        owner_handle: &[u8; 32],
        identity_pk: &[u8; 32],
    ) -> PinCheckResult {
        match self.pins.get_mut(owner_handle) {
            None => {
                self.pins.insert(
                    *owner_handle,
                    PinEntry {
                        view_pk: UNKNOWN_VIEW_PK,
                        identity_pk: *identity_pk,
                        directory_id: None,
                        first_seen: current_unix_timestamp_fail_closed(),
                        trust_level: TrustLevel::Tentative,
                    },
                );
                PinCheckResult::NewIdentity
            }
            Some(pin) => {
                if pin.trust_level == TrustLevel::Revoked {
                    return PinCheckResult::Revoked;
                }

                if pin.identity_pk == *identity_pk {
                    PinCheckResult::Verified
                } else {
                    PinCheckResult::IdentityChanged
                }
            }
        }
    }

    /// Confirms accepted view key rotation and marks entry as pinned.
    pub fn confirm_rotation(&mut self, owner: &[u8; 32], new_view: &[u8; 32]) {
        if let Some(pin) = self.pins.get_mut(owner) {
            pin.view_pk = *new_view;
            pin.trust_level = TrustLevel::Pinned;
        }
    }

    /// Marks existing pin as revoked.
    pub fn revoke(&mut self, owner: &[u8; 32]) {
        if let Some(pin) = self.pins.get_mut(owner) {
            pin.trust_level = TrustLevel::Revoked;
        }
    }

    /// Returns immutable pin entry by owner handle.
    pub fn get(&self, owner: &[u8; 32]) -> Option<&PinEntry> {
        self.pins.get(owner)
    }

    /// Number of pinned owners.
    pub fn len(&self) -> usize {
        self.pins.len()
    }

    /// Returns true when no pins are stored.
    pub fn is_empty(&self) -> bool {
        self.pins.is_empty()
    }

    /// Export pins as deterministic owner→entry pairs.
    pub fn to_pairs(&self) -> Vec<([u8; 32], PinEntry)> {
        let mut out: Vec<([u8; 32], PinEntry)> = self
            .pins
            .iter()
            .map(|(owner, entry)| (*owner, entry.clone()))
            .collect();
        out.sort_by_key(|(owner, _)| *owner);
        out
    }

    /// Build pin store from owner→entry pairs.
    pub fn from_pairs(pairs: Vec<([u8; 32], PinEntry)>) -> Self {
        let mut pins = HashMap::with_capacity(pairs.len());
        for (owner, entry) in pairs {
            pins.insert(owner, entry);
        }
        Self { pins }
    }

    fn is_card_expired(&self, card: &ReceiverCard) -> bool {
        card.metadata
            .as_ref()
            .and_then(|meta| meta.valid_until)
            .is_some_and(|valid_until| current_unix_timestamp_fail_closed() >= valid_until)
    }

    fn insert_pin(&mut self, card: &ReceiverCard, directory_id: Option<&str>) {
        let pin = PinEntry {
            view_pk: card.view_pk,
            identity_pk: card.identity_pk,
            directory_id: directory_id.map(ToString::to_string),
            first_seen: current_unix_timestamp_fail_closed(),
            trust_level: TrustLevel::Tentative,
        };
        self.pins.insert(card.owner_handle, pin);
    }

    fn eval_pin_mut(&mut self, card: &ReceiverCard) -> Result<VerifyResult, ReceiverCardError> {
        let pin = self
            .pins
            .get_mut(&card.owner_handle)
            .ok_or(ReceiverCardError::InvalidCardBytes)?;

        if pin.trust_level == TrustLevel::Revoked {
            return Err(ReceiverCardError::PinRevoked);
        }

        if pin.view_pk == UNKNOWN_VIEW_PK && pin.identity_pk == card.identity_pk {
            pin.view_pk = card.view_pk;
            return Ok(VerifyResult::Verified);
        }

        if pin.view_pk != card.view_pk {
            return Ok(VerifyResult::ViewKeyChanged {
                old_pk: pin.view_pk,
                new_pk: card.view_pk,
                requires_confirmation: true,
            });
        }

        if pin.identity_pk != card.identity_pk {
            return Ok(VerifyResult::IdentityKeyChanged);
        }

        Ok(VerifyResult::Verified)
    }
}

#[cfg(test)]
mod tests {
    include!("test_receiver_card_trust_suite.rs");
}
