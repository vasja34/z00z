use std::collections::BTreeMap;

use crate::hashing::AssetIdHasher;

use super::{AssetClass, AssetDefinition, AssetError};

impl AssetDefinition {
    fn fill_id(hash_bytes: impl AsRef<[u8]>) -> [u8; 32] {
        let mut id = [0u8; 32];
        id.copy_from_slice(&hash_bytes.as_ref()[..32]);
        id
    }

    fn hash_bytes(hasher: &mut AssetIdHasher, tag: &[u8], value: &[u8]) {
        hasher.update(tag);
        hasher.update((value.len() as u32).to_le_bytes());
        hasher.update(value);
    }

    fn hash_u8(hasher: &mut AssetIdHasher, tag: &[u8], value: u8) {
        hasher.update(tag);
        hasher.update([value]);
    }

    fn hash_u32(hasher: &mut AssetIdHasher, tag: &[u8], value: u32) {
        hasher.update(tag);
        hasher.update(value.to_le_bytes());
    }

    fn hash_u64(hasher: &mut AssetIdHasher, tag: &[u8], value: u64) {
        hasher.update(tag);
        hasher.update(value.to_le_bytes());
    }

    pub(crate) fn derive_id(
        class: AssetClass,
        name: &str,
        symbol: &str,
        decimals: u8,
        serials: u32,
        nominal: u64,
        domain_name: &str,
        version: u8,
        crypto_version: u8,
        policy_flags: u8,
        metadata: Option<&BTreeMap<String, String>>,
    ) -> [u8; 32] {
        let mut hasher = AssetIdHasher::new_with_label("asset_definition");
        Self::hash_u8(&mut hasher, b"class", class.class_byte());
        Self::hash_bytes(&mut hasher, b"name", name.as_bytes());
        Self::hash_bytes(&mut hasher, b"symbol", symbol.as_bytes());
        Self::hash_u8(&mut hasher, b"decimals", decimals);
        Self::hash_u32(&mut hasher, b"serials", serials);
        Self::hash_u64(&mut hasher, b"nominal", nominal);
        Self::hash_bytes(&mut hasher, b"domain", domain_name.as_bytes());
        Self::hash_u8(&mut hasher, b"version", version);
        Self::hash_u8(&mut hasher, b"crypto", crypto_version);
        Self::hash_u8(&mut hasher, b"flags", policy_flags);

        if let Some(metadata_map) = metadata {
            Self::hash_u32(&mut hasher, b"meta_count", metadata_map.len() as u32);
            for (key, value) in metadata_map {
                Self::hash_bytes(&mut hasher, b"meta_key", key.as_bytes());
                Self::hash_bytes(&mut hasher, b"meta_val", value.as_bytes());
            }
        } else {
            Self::hash_u32(&mut hasher, b"meta_count", 0);
        }

        Self::fill_id(hasher.finalize())
    }

    pub(crate) fn validate_id(&self) -> Result<(), AssetError> {
        let expected_id = Self::derive_id(
            self.class,
            &self.name,
            &self.symbol,
            self.decimals,
            self.serials,
            self.nominal,
            &self.domain_name,
            self.version,
            self.crypto_version,
            self.policy_flags,
            self.metadata.as_ref(),
        );

        if self.id != expected_id {
            return Err(AssetError::Integrity(std::borrow::Cow::Owned(format!(
                "asset definition id mismatch: expected {:02x?}, got {:02x?}",
                expected_id, self.id
            ))));
        }

        Ok(())
    }
}
