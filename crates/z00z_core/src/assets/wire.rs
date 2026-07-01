// crates/z00z_core/src/assets/wire.rs
//
// Wire Format for Serialization - Validator ↔ Wallet Communication

use std::borrow::Cow;

use super::assets::{Asset, AssetClass, AssetError};
use super::definition::AssetDefinition;
#[allow(unused_imports)]
use super::policy_flags::BURNABLE;
use super::registry::GLOBAL_ASSET_REGISTRY;
use super::serial_id::{validate_serial_bounds, SerialIdError};
use thiserror::Error;
use z00z_crypto::{
    KernelSignature, RangeProof, Z00ZCommitment as Commitment, Z00ZRistrettoPoint, ZkPackEncrypted,
};

#[path = "wire_pkg.rs"]
mod wire_pkg;

pub use wire_pkg::{
    decode_asset_pkg_json, encode_asset_pkg_json, payload_has_secret_field, AssetPkgWire,
    ASSET_PKG_JSON_MAX_BYTES,
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AssetPackFields {
    pub value: u64,
    pub blinding: [u8; 32],
    pub s_out: [u8; 32],
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Error)]
pub enum AssetPackError {
    #[error("asset pack is truncated")]
    TruncatedAssetPack,
}

pub fn verify_asset_pack_encoding(bytes: &[u8]) -> Result<AssetPackFields, AssetPackError> {
    if bytes.len() != super::leaf::AssetPackPlain::SIZE {
        return Err(AssetPackError::TruncatedAssetPack);
    }

    let mut value_bytes = [0u8; 8];
    value_bytes.copy_from_slice(&bytes[0..8]);

    let mut blinding = [0u8; 32];
    blinding.copy_from_slice(&bytes[8..40]);

    let mut s_out = [0u8; 32];
    s_out.copy_from_slice(&bytes[40..72]);

    Ok(AssetPackFields {
        value: u64::from_le_bytes(value_bytes),
        blinding,
        s_out,
    })
}

fn check_serial(serial_id: u32, max_allowed: u32) -> Result<(), AssetError> {
    validate_serial_bounds(serial_id, max_allowed).map_err(|error| match error {
        SerialIdError::OutOfBounds { serial_id, max } => AssetError::InvalidSerialId {
            serial_id,
            max_allowed: max,
        },
        SerialIdError::InvalidLength { .. } => AssetError::InvalidSerialId {
            serial_id,
            max_allowed,
        },
    })
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct DefinitionWire {
    pub id: [u8; 32],
    pub class: AssetClass,
    pub name: String,
    pub symbol: String,
    pub decimals: u8,
    pub serials: u32,
    pub nominal: u64,
    pub domain_name: String,
    pub version: u8,
    pub crypto_version: u8,
    pub policy_flags: u8,
    pub metadata: Option<std::collections::BTreeMap<String, String>>,
}

impl From<&AssetDefinition> for DefinitionWire {
    fn from(def: &AssetDefinition) -> Self {
        Self {
            id: def.id,
            class: def.class,
            name: def.name.clone(),
            symbol: def.symbol.clone(),
            decimals: def.decimals,
            serials: def.serials,
            nominal: def.nominal,
            domain_name: def.domain_name.clone(),
            version: def.version,
            crypto_version: def.crypto_version,
            policy_flags: def.policy_flags,
            metadata: def.metadata.clone(),
        }
    }
}

impl DefinitionWire {
    fn push_bytes(buffer: &mut Vec<u8>, tag: &[u8], value: &[u8]) {
        buffer.extend_from_slice(tag);
        buffer.extend_from_slice(&(value.len() as u32).to_le_bytes());
        buffer.extend_from_slice(value);
    }

    fn push_u8(buffer: &mut Vec<u8>, tag: &[u8], value: u8) {
        buffer.extend_from_slice(tag);
        buffer.push(value);
    }

    fn push_u32(buffer: &mut Vec<u8>, tag: &[u8], value: u32) {
        buffer.extend_from_slice(tag);
        buffer.extend_from_slice(&value.to_le_bytes());
    }

    fn push_u64(buffer: &mut Vec<u8>, tag: &[u8], value: u64) {
        buffer.extend_from_slice(tag);
        buffer.extend_from_slice(&value.to_le_bytes());
    }

    pub(crate) fn payload_bytes(&self) -> Vec<u8> {
        let mut buffer = Vec::new();
        Self::push_bytes(&mut buffer, b"id", &self.id);
        Self::push_u8(&mut buffer, b"class", self.class.class_byte());
        Self::push_bytes(&mut buffer, b"name", self.name.as_bytes());
        Self::push_bytes(&mut buffer, b"symbol", self.symbol.as_bytes());
        Self::push_u8(&mut buffer, b"decimals", self.decimals);
        Self::push_u32(&mut buffer, b"serials", self.serials);
        Self::push_u64(&mut buffer, b"nominal", self.nominal);
        Self::push_bytes(&mut buffer, b"domain", self.domain_name.as_bytes());
        Self::push_u8(&mut buffer, b"version", self.version);
        Self::push_u8(&mut buffer, b"crypto", self.crypto_version);
        Self::push_u8(&mut buffer, b"flags", self.policy_flags);

        if let Some(metadata) = &self.metadata {
            Self::push_u32(&mut buffer, b"meta_count", metadata.len() as u32);
            for (key, value) in metadata {
                Self::push_bytes(&mut buffer, b"meta_key", key.as_bytes());
                Self::push_bytes(&mut buffer, b"meta_val", value.as_bytes());
            }
        } else {
            Self::push_u32(&mut buffer, b"meta_count", 0);
        }

        buffer
    }
}

impl TryFrom<DefinitionWire> for AssetDefinition {
    type Error = AssetError;

    fn try_from(wire: DefinitionWire) -> Result<Self, Self::Error> {
        let expected_id = wire.id;
        let definition = AssetDefinition::new(
            [0u8; 32],
            wire.class,
            wire.name,
            wire.symbol,
            wire.decimals,
            wire.serials,
            wire.nominal,
            wire.domain_name,
            wire.version,
            wire.crypto_version,
            wire.policy_flags,
            wire.metadata,
        )?;

        if definition.id != expected_id {
            return Err(AssetError::Integrity(Cow::Owned(format!(
                "asset definition id mismatch: expected {:02x?}, got {:02x?}",
                definition.id, expected_id
            ))));
        }

        Ok(definition)
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq)]
pub struct AssetWire {
    pub definition: AssetDefinition,
    pub serial_id: u32,
    pub amount: u64,
    pub commitment: Commitment,
    pub range_proof: Option<RangeProof>,
    pub nonce: [u8; 32],
    pub lock_height: Option<u64>,
    pub is_burned: bool,
    pub owner_pub: Option<Z00ZRistrettoPoint>,
    pub owner_signature: Option<KernelSignature>,
    pub is_frozen: bool,
    pub is_slashed: bool,
    pub r_pub: Option<[u8; 32]>,
    pub owner_tag: Option<[u8; 32]>,
    pub enc_pack: Option<ZkPackEncrypted>,
    pub secret: Option<[u8; 32]>,
    pub tag16: Option<u16>,
    pub leaf_ad_id: Option<[u8; 32]>,
}

impl AssetWire {
    pub fn from_asset(asset: &Asset) -> Self {
        Self {
            definition: (*asset.definition).clone(),
            serial_id: asset.serial_id,
            amount: asset.amount,
            commitment: asset.commitment.clone(),
            range_proof: asset.range_proof.clone(),
            nonce: asset.nonce,
            lock_height: asset.lock_height,
            is_burned: asset.is_burned,
            owner_pub: asset.owner_pub.clone(),
            owner_signature: asset.owner_signature.clone(),
            is_frozen: asset.is_frozen,
            is_slashed: asset.is_slashed,
            r_pub: asset.r_pub,
            owner_tag: asset.owner_tag,
            enc_pack: asset.enc_pack.clone(),
            secret: None,
            tag16: asset.tag16,
            leaf_ad_id: asset.leaf_ad_id,
        }
    }

    pub fn leaf_ad_id(&self) -> Result<[u8; 32], AssetError> {
        self.leaf_ad_id
            .ok_or(AssetError::InvalidStealth(Cow::Borrowed(
                "full stealth fields require leaf_ad_id",
            )))
    }

    pub fn to_asset(self) -> Result<Asset, AssetError> {
        self.validate()?;

        let arc_def = GLOBAL_ASSET_REGISTRY.insert(self.definition)?;
        let asset = Asset {
            definition: arc_def,
            serial_id: self.serial_id,
            amount: self.amount,
            commitment: self.commitment,
            range_proof: self.range_proof,
            nonce: self.nonce,
            lock_height: self.lock_height,
            is_burned: self.is_burned,
            owner_pub: self.owner_pub,
            owner_signature: self.owner_signature,
            is_frozen: self.is_frozen,
            is_slashed: self.is_slashed,
            r_pub: self.r_pub,
            owner_tag: self.owner_tag,
            enc_pack: self.enc_pack,
            secret: self.secret,
            tag16: self.tag16,
            leaf_ad_id: self.leaf_ad_id,
        };

        asset.verify_complete()?;
        Ok(asset)
    }

    pub fn validate(&self) -> Result<(), AssetError> {
        self.definition.validate()?;
        let _pack_ver = super::version::validate_serial_id_version(self.serial_id);
        check_serial(self.serial_id, self.definition.serials)?;

        if self.amount == 0 && !matches!(self.definition.class, AssetClass::Nft | AssetClass::Void)
        {
            return Err(AssetError::InvalidAsset(Cow::Borrowed(
                "amount must be non-zero for non-NFT/Void assets",
            )));
        }

        if self.secret.is_some() {
            return Err(AssetError::InvalidAsset(Cow::Borrowed(
                "secret is forbidden in AssetWire import",
            )));
        }

        Ok(())
    }
}

#[cfg(test)]
#[path = "test_wire.rs"]
mod tests;

#[cfg(test)]
#[path = "test_wire_compat.rs"]
mod test_wire_compat;
