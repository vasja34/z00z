//! Card-related receiver exchange helpers.

/// NFC helpers for compact payment sharing.
#[path = "nfc_ndef.rs"]
pub mod nfc_ndef;
/// Stealth receiver card model, validation, and wire helpers.
#[path = "receiver_card.rs"]
pub mod receiver_card;
/// Wallet-local TOFU and pinning policy for receiver cards.
#[path = "receiver_card_trust.rs"]
pub mod receiver_card_trust;
