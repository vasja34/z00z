use std::collections::BTreeMap;

use z00z_crypto::DomainHasher;

use crate::domains::MetadataHashDomain;

/// Off-chain metadata for assets, stored in treasury database or indexer.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AssetMetadata {
    pub custom_fields: BTreeMap<String, String>,
    pub metadata_hash: [u8; 32],
    pub timestamp: u64,
}

impl AssetMetadata {
    /// Compute canonical Blake2b-256 hash of metadata for on-chain commitment.
    pub fn compute_hash(&self) -> [u8; 32] {
        let mut hasher = DomainHasher::<MetadataHashDomain>::new_with_label("metadata");
        for (key, value) in &self.custom_fields {
            hasher.update(key.as_bytes());
            hasher.update(value.as_bytes());
        }
        hasher.update(self.timestamp.to_le_bytes());
        let hash = hasher.finalize();
        let mut result = [0u8; 32];
        result.copy_from_slice(&hash.as_ref()[..32]);
        result
    }

    /// Verify that metadata_hash matches the computed hash.
    pub fn verify_hash(&self) -> bool {
        self.metadata_hash == self.compute_hash()
    }
}
