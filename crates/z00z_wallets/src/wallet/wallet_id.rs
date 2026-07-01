/// Wallet identifier (core type).
///
/// This type represents the stable wallet identity inside the core wallet domain.
///
/// Note: The RPC layer has its own `WalletId` type for JSON-RPC serialization.
/// Keep conversions at the boundary.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(transparent)]
pub struct WalletId(#[serde(with = "serde_wallet_id_hex")] pub [u8; 32]);

mod serde_wallet_id_hex {
    use super::{from_hex, to_hex, Deserialize};
    use serde::de::Error as _;

    pub fn serialize<S>(value: &[u8; 32], serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&to_hex(value))
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<[u8; 32], D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        let bytes = from_hex(&s).map_err(|e| D::Error::custom(e.to_string()))?;
        if bytes.len() != 32 {
            return Err(D::Error::custom("wallet_id hex must decode to 32 bytes"));
        }

        let mut out = [0u8; 32];
        out.copy_from_slice(&bytes);
        Ok(out)
    }
}

impl WalletId {
    /// Returns the raw 32-byte identifier.
    pub fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }

    /// Returns a lowercase hex encoding of this wallet id.
    pub fn to_hex(&self) -> String {
        to_hex(&self.0)
    }

    /// Converts this core wallet id to the persistent RPC wallet id string format.
    ///
    /// The RPC layer uses a string identifier. The current convention is:
    /// `wallet_<64 hex chars>`.
    pub fn to_persist_wallet_id(&self) -> PersistWalletId {
        PersistWalletId(format!("wallet_{}", self.to_hex()))
    }

    /// Computes a new core wallet id from wallet creation inputs.
    ///
    /// This uses domain-separated hashing to avoid cross-protocol collisions.
    pub fn from_create_wallet_inputs(chain_id: ChainId, created_at: u64, nonce: [u8; 16]) -> Self {
        // NOTE: This is intentionally name-independent.
        // Wallet name is user-controlled metadata and must not affect identity.
        let hash = DomainHasher::<WalletIdDomain>::new_with_label("wallet_id")
            .chain(nonce)
            .chain([chain_id.as_u8()])
            .chain(created_at.to_le_bytes())
            .finalize();

        let mut id = [0u8; 32];
        id.copy_from_slice(&hash.as_ref()[..32]);
        Self(id)
    }
}

impl From<[u8; 32]> for WalletId {
    fn from(value: [u8; 32]) -> Self {
        Self(value)
    }
}

impl core::fmt::Display for WalletId {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str(&self.to_hex())
    }
}
