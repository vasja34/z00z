use z00z_crypto::{
    domains::HashToScalarDomain,
    hash::{hash_to_scalar_zk, poseidon2_hash},
    Z00ZRistrettoPoint, Z00ZScalar, ZkPackEncrypted,
};
use z00z_utils::codec::{BincodeCodec, Codec, CodecError};

use super::record::{RightLeaf, VoucherLeaf};
use super::TerminalId;

/// Storage-owned terminal leaf surface.
///
/// This leaf keeps the committed payload identical to the core asset-leaf
/// contract while preserving a storage-owned type boundary. Storage-facing code
/// must import this type instead of depending on
/// `z00z_core::assets::AssetLeaf` directly.
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct TerminalLeaf {
    pub asset_id: [u8; 32],
    pub serial_id: u32,
    pub r_pub: [u8; 32],
    pub owner_tag: [u8; 32],
    pub c_amount: [u8; 32],
    pub enc_pack: ZkPackEncrypted,
    pub range_proof: Vec<u8>,
    pub tag16: u16,
}

impl Default for TerminalLeaf {
    fn default() -> Self {
        Self {
            asset_id: [0u8; 32],
            serial_id: 0,
            r_pub: [0u8; 32],
            owner_tag: [0u8; 32],
            c_amount: [0u8; 32],
            enc_pack: ZkPackEncrypted {
                version: 1,
                ciphertext: Vec::new(),
                tag: [0u8; 16],
            },
            range_proof: Vec::new(),
            tag16: 0,
        }
    }
}

impl TerminalLeaf {
    #[must_use]
    pub fn into_core(self) -> z00z_core::assets::AssetLeaf {
        self.into()
    }

    #[must_use]
    pub fn dummy_for_scan(index: u32) -> Self {
        let idx = index.to_le_bytes();
        let asset_id = poseidon2_hash(b"Z00Z/PERF/ASSET", &[&idx]);
        let r_sk = hash_to_scalar_zk::<HashToScalarDomain>("Z00Z/PERF/RSK", &[&idx])
            .unwrap_or_else(|_| Z00ZScalar::one());
        let r_pub = Z00ZRistrettoPoint::from_secret_key(&r_sk).to_bytes();
        let owner_tag = poseidon2_hash(b"Z00Z/PERF/TAG", &[&idx]);
        let c_amount = poseidon2_hash(b"Z00Z/PERF/CAMT", &[&idx]);

        Self {
            asset_id,
            serial_id: index,
            r_pub,
            owner_tag,
            c_amount,
            enc_pack: ZkPackEncrypted {
                version: 1,
                ciphertext: vec![0u8; 64],
                tag: [0u8; 16],
            },
            range_proof: Vec::new(),
            tag16: 0,
        }
    }

    #[must_use]
    pub const fn terminal_id(&self) -> TerminalId {
        TerminalId::new(self.asset_id)
    }

    pub fn set_terminal_id(&mut self, terminal_id: TerminalId) {
        self.asset_id = terminal_id.into_bytes();
    }
}

impl From<z00z_core::assets::AssetLeaf> for TerminalLeaf {
    fn from(value: z00z_core::assets::AssetLeaf) -> Self {
        Self {
            asset_id: value.asset_id,
            serial_id: value.serial_id,
            r_pub: value.r_pub,
            owner_tag: value.owner_tag,
            c_amount: value.c_amount,
            enc_pack: value.enc_pack,
            range_proof: value.range_proof,
            tag16: value.tag16,
        }
    }
}

impl From<&z00z_core::assets::AssetLeaf> for TerminalLeaf {
    fn from(value: &z00z_core::assets::AssetLeaf) -> Self {
        Self::from(value.clone())
    }
}

impl From<&TerminalLeaf> for TerminalLeaf {
    fn from(value: &TerminalLeaf) -> Self {
        value.clone()
    }
}

impl From<TerminalLeaf> for z00z_core::assets::AssetLeaf {
    fn from(value: TerminalLeaf) -> Self {
        Self {
            asset_id: value.asset_id,
            serial_id: value.serial_id,
            r_pub: value.r_pub,
            owner_tag: value.owner_tag,
            c_amount: value.c_amount,
            enc_pack: value.enc_pack,
            range_proof: value.range_proof,
            tag16: value.tag16,
        }
    }
}

impl From<&TerminalLeaf> for z00z_core::assets::AssetLeaf {
    fn from(value: &TerminalLeaf) -> Self {
        Self {
            asset_id: value.asset_id,
            serial_id: value.serial_id,
            r_pub: value.r_pub,
            owner_tag: value.owner_tag,
            c_amount: value.c_amount,
            enc_pack: value.enc_pack.clone(),
            range_proof: value.range_proof.clone(),
            tag16: value.tag16,
        }
    }
}

pub const TERMINAL_LEAF_TAG: u8 = 1;
pub const RIGHT_LEAF_TAG: u8 = 2;
pub const VOUCHER_LEAF_TAG: u8 = 3;

pub fn encode_terminal_leaf(leaf: &TerminalLeaf) -> Result<Vec<u8>, CodecError> {
    let codec = BincodeCodec;
    let mut bytes = Vec::with_capacity(1 + 256);
    bytes.push(TERMINAL_LEAF_TAG);
    bytes.extend(codec.serialize(leaf)?);
    Ok(bytes)
}

pub fn encode_right_leaf(leaf: &RightLeaf) -> Result<Vec<u8>, CodecError> {
    let codec = BincodeCodec;
    let mut bytes = Vec::with_capacity(1 + 256);
    bytes.push(RIGHT_LEAF_TAG);
    bytes.extend(codec.serialize(leaf)?);
    Ok(bytes)
}

pub fn encode_voucher_leaf(leaf: &VoucherLeaf) -> Result<Vec<u8>, CodecError> {
    let codec = BincodeCodec;
    let mut bytes = Vec::with_capacity(1 + 256);
    bytes.push(VOUCHER_LEAF_TAG);
    bytes.extend(codec.serialize(leaf)?);
    Ok(bytes)
}

#[cfg(test)]
mod tests {
    use super::TerminalLeaf;
    use z00z_core::assets::{AssetLeaf, AssetPackPlain};
    use z00z_crypto::ZkPackEncrypted;
    use z00z_utils::codec::{BincodeCodec, Codec};

    #[test]
    fn test_storage_leaf_codec_same() {
        let core_leaf = AssetLeaf {
            asset_id: [7u8; 32],
            serial_id: 9,
            r_pub: [1u8; 32],
            owner_tag: [2u8; 32],
            c_amount: [3u8; 32],
            enc_pack: ZkPackEncrypted {
                version: 1,
                ciphertext: AssetPackPlain {
                    value: 55,
                    blinding: [4u8; 32],
                    s_out: [5u8; 32],
                }
                .to_bytes(),
                tag: [6u8; 16],
            },
            range_proof: vec![8u8; 12],
            tag16: 11,
        };
        let storage_leaf = TerminalLeaf::from(core_leaf.clone());
        let codec = BincodeCodec;

        let core_bytes = codec.serialize(&core_leaf).expect("core leaf bytes");
        let storage_bytes = codec.serialize(&storage_leaf).expect("storage leaf bytes");

        assert_eq!(storage_bytes, core_bytes);
    }
}
