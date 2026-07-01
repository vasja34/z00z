//! Nullifier derivation and state types for claim anti-replay.
#![forbid(unsafe_code)]

use serde::{Deserialize, Serialize};
use z00z_utils::codec::{Codec, JsonCodec};

/// Raw nullifier bytes wrapper.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NullifierBytes(pub [u8; 32]);

impl NullifierBytes {
    /// Hex encoding of the wrapped nullifier bytes.
    pub fn to_hex(&self) -> String {
        hex::encode(self.0)
    }
}

/// Typed nullifier key container used by state layer.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NullifierKey {
    /// Wrapped nullifier bytes.
    pub nullifier: NullifierBytes,
}

impl NullifierKey {
    /// Compute domain-tagged key for nullifier state storage.
    pub fn state_key(&self) -> [u8; 32] {
        let mut h = blake3::Hasher::new();
        h.update(b"z00z.nullifier.state.v1");
        h.update(&self.nullifier.0);
        *h.finalize().as_bytes()
    }
}

/// Nullifier lifecycle status.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub enum NullifierStatus {
    /// Nullifier is reserved by an accepted claim path but not yet finalized.
    Reserved,
    /// Nullifier has already been spent.
    Spent,
}

/// Persisted nullifier state row.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct NullifierEntry {
    /// Nullifier encoded as lower-hex.
    pub nullifier_hex: String,
    /// Nullifier current status.
    pub status: NullifierStatus,
    /// Claim id encoded as lower-hex.
    pub claim_id_hex: String,
    /// Numeric chain id bound into nullifier derivation.
    pub chain_id: u32,
    /// Recipient binding encoded as lower-hex.
    pub owner_hex: String,
    /// Canonical tx digest that reserved or finalized this nullifier.
    pub tx_digest_hex: String,
    /// Monotonic sequence assigned by the active store.
    pub created_at_seq: u64,
}

impl NullifierEntry {
    /// Serialize entry to JSON bytes via `z00z_utils::codec::JsonCodec`.
    pub fn to_bytes(&self) -> Result<Vec<u8>, String> {
        let codec = JsonCodec;
        codec.serialize(self).map_err(|e| e.to_string())
    }

    /// Deserialize entry from JSON bytes via `z00z_utils::codec::JsonCodec`.
    pub fn from_bytes(data: &[u8]) -> Result<Self, String> {
        let codec = JsonCodec;
        codec.deserialize(data).map_err(|e| e.to_string())
    }
}

/// Derive deterministic claim nullifier with domain separation and chain binding.
pub fn derive_nullifier(claim_id: &[u8; 32], owner: &[u8; 32], chain_id: u32) -> NullifierBytes {
    let mut h = blake3::Hasher::new();
    h.update(b"z00z.nullifier.derive.v1");

    h.update(&chain_id.to_le_bytes());
    h.update(claim_id);
    h.update(owner);

    NullifierBytes(*h.finalize().as_bytes())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_nullifier_unique_per_claim() {
        let owner = [0x03u8; 32];
        let n1 = derive_nullifier(&[0x01u8; 32], &owner, 7);
        let n2 = derive_nullifier(&[0x02u8; 32], &owner, 7);
        assert_ne!(n1, n2);
    }

    #[test]
    fn test_nullifier_deterministic() {
        let claim_id = [0xabu8; 32];
        let owner = [0xcdu8; 32];
        let n1 = derive_nullifier(&claim_id, &owner, 7);
        let n2 = derive_nullifier(&claim_id, &owner, 7);
        assert_eq!(n1, n2);
    }

    #[test]
    fn test_nullifier_chain_sep() {
        let claim_id = [0xabu8; 32];
        let owner = [0xcdu8; 32];
        let n1 = derive_nullifier(&claim_id, &owner, 7);
        let n2 = derive_nullifier(&claim_id, &owner, 8);
        assert_ne!(n1, n2);
    }

    #[test]
    fn test_state_key_domain_sep() {
        let nullifier = NullifierBytes([0x11u8; 32]);
        let key = NullifierKey {
            nullifier: nullifier.clone(),
        };
        assert_ne!(key.state_key(), nullifier.0);
    }

    #[test]
    fn test_domain_prefix_sep() {
        let claim_id = [0x31u8; 32];
        let owner = [0x41u8; 32];
        let derived = derive_nullifier(&claim_id, &owner, 7);
        let key = NullifierKey {
            nullifier: derived.clone(),
        };
        assert_ne!(key.state_key(), derived.0);
    }

    #[test]
    fn test_nullifier_entry_roundtrip() {
        let entry = NullifierEntry {
            nullifier_hex: "ab".repeat(32),
            status: NullifierStatus::Reserved,
            claim_id_hex: "cd".repeat(32),
            chain_id: 3,
            owner_hex: "ef".repeat(32),
            tx_digest_hex: "12".repeat(32),
            created_at_seq: 42,
        };
        let data = entry.to_bytes().unwrap();
        let out = NullifierEntry::from_bytes(&data).unwrap();
        assert_eq!(entry, out);
    }
}
