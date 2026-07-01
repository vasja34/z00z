#![forbid(unsafe_code)]
//! OWF circuit contract: asset_pack_plain wire format is
//! value[0..8 LE] | blinding[8..40] | s_out[40..72] = 72 bytes total.
//! This format is consensus-critical (see spec §4.5.6).

use thiserror::Error;
use z00z_crypto::{
    domains::HashToScalarDomain,
    hash::{hash_to_scalar_zk, poseidon2_hash},
    Z00ZRistrettoPoint, Z00ZScalar, ZkPackEncrypted,
};

use super::version::AssetPackVersion;

/// Canonical public asset leaf used for scan and ownership checks.
///
/// `z00z_storage` re-exports a storage-facing compatibility surface as
/// `z00z_storage::settlement::TerminalLeaf`. Storage code should depend on that
/// storage-owned surface while Phase 2 preserves the existing wire payload.
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct AssetLeaf {
    /// Asset identifier.
    pub asset_id: [u8; 32],
    /// Base serial — assigned at asset creation and MUST NOT change.
    ///
    /// All fragments (outputs) of one banknote share the same `serial_id`.
    /// A composite payment may mix fragments from different `serial_id` values,
    /// but each fragment keeps its original base serial.
    ///
    /// INVARIANT: `serial_id ∈ [0, definition.serials)` and is immutable post-creation.
    /// Validated by `validate_serial_id_version()` and `validate_serial_bounds()`.
    pub serial_id: u32,
    /// Sender ephemeral public key bytes.
    pub r_pub: [u8; 32],
    /// Owner tag for fast ownership precheck.
    pub owner_tag: [u8; 32],
    /// Commitment bytes.
    pub c_amount: [u8; 32],
    /// Encrypted asset package.
    pub enc_pack: ZkPackEncrypted,
    /// Range proof bytes.
    pub range_proof: Vec<u8>,
    /// Short prefilter tag.
    pub tag16: u16,
}

impl Default for AssetLeaf {
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

impl AssetLeaf {
    /// Build synthetic leaf for scan throughput tests.
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
}

/// Plain asset payload encoded inside ZkPack.
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct AssetPackPlain {
    /// Clear-text value.
    pub value: u64,
    /// Commitment blinding scalar bytes.
    pub blinding: [u8; 32],
    /// Output secret bytes.
    pub s_out: [u8; 32],
}

/// Memo-capable asset payload encoded inside the memo lane of ZkPack.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AssetPackPlainMemo {
    /// Clear-text value.
    pub value: u64,
    /// Commitment blinding scalar bytes.
    pub blinding: [u8; 32],
    /// Output secret bytes.
    pub s_out: [u8; 32],
    /// Optional wallet-facing memo bytes kept inside the encrypted payload.
    pub memo: Vec<u8>,
}

/// Version-aware decoded asset pack payload.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum DecodedAssetPack {
    /// Base fixed-width payload.
    Basic(AssetPackPlain),
    /// Memo-capable payload.
    Memo(AssetPackPlainMemo),
}

/// Asset pack decode errors with deterministic reasons.
#[derive(Clone, Copy, Debug, Error, PartialEq, Eq)]
pub enum PackErr {
    /// Input length does not match the canonical payload contract.
    #[error("asset pack length is malformed")]
    BadLen,
    /// Blinding bytes are not a canonical scalar encoding.
    #[error("asset pack blinding bytes are malformed")]
    BadBlind,
    /// Memo length exceeds the bounded memo-lane decode contract.
    #[error("asset pack memo length exceeds 512 bytes")]
    BadMemo,
    /// Version-aware decode rejected an unsupported asset-pack lane.
    #[error("asset pack version is unsupported")]
    BadVer,
}

impl AssetPackPlain {
    /// Fixed canonical plaintext size (value + blinding + s_out).
    pub const SIZE: usize = 8 + 32 + 32;

    /// Validate canonical plaintext length.
    #[must_use]
    pub fn is_valid_length(bytes: &[u8]) -> bool {
        bytes.len() == Self::SIZE
    }

    /// Encode payload to canonical bytes.
    /// Wire format (OWF consensus-critical): value[0..8 LE] | blinding[8..40] | s_out[40..72].
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut out = Vec::with_capacity(Self::SIZE);
        out.extend_from_slice(&self.value.to_le_bytes());
        out.extend_from_slice(&self.blinding);
        out.extend_from_slice(&self.s_out);
        out
    }

    /// Decode payload from canonical bytes.
    pub fn from_bytes(bytes: &[u8]) -> Option<Self> {
        Self::decode_checked(bytes).ok()
    }

    /// Decode payload with strict length checks and deterministic error reason.
    pub fn decode_strict(bytes: &[u8]) -> Result<Self, PackErr> {
        if !Self::is_valid_length(bytes) {
            return Err(PackErr::BadLen);
        }

        let mut value_bytes = [0u8; 8];
        value_bytes.copy_from_slice(&bytes[0..8]);

        let mut blinding = [0u8; 32];
        blinding.copy_from_slice(&bytes[8..40]);

        let mut s_out = [0u8; 32];
        s_out.copy_from_slice(&bytes[40..72]);

        Ok(Self {
            value: u64::from_le_bytes(value_bytes),
            blinding,
            s_out,
        })
    }

    /// Decode payload and validate blinding encoding.
    pub fn decode_checked(bytes: &[u8]) -> Result<Self, PackErr> {
        let pack = Self::decode_strict(bytes)?;
        if Z00ZScalar::try_from_bytes(pack.blinding).is_err() {
            return Err(PackErr::BadBlind);
        }
        Ok(pack)
    }
}

impl AssetPackPlainMemo {
    /// Fixed header size shared by all memo payloads.
    pub const HEAD_SIZE: usize = AssetPackPlain::SIZE + 2;

    /// Largest accepted memo payload for the current memo contract.
    pub const MEMO_MAX: usize = 512;

    /// Encode payload to canonical bytes.
    pub fn encode_checked(&self) -> Result<Vec<u8>, PackErr> {
        if self.memo.len() > Self::MEMO_MAX {
            return Err(PackErr::BadMemo);
        }

        let memo_len = u16::try_from(self.memo.len()).map_err(|_| PackErr::BadMemo)?;
        let mut out = Vec::with_capacity(Self::HEAD_SIZE + self.memo.len());
        out.extend_from_slice(&self.value.to_le_bytes());
        out.extend_from_slice(&self.blinding);
        out.extend_from_slice(&self.s_out);
        out.extend_from_slice(&memo_len.to_le_bytes());
        out.extend_from_slice(&self.memo);
        Ok(out)
    }

    /// Decode payload from canonical bytes.
    pub fn from_bytes(bytes: &[u8]) -> Option<Self> {
        Self::decode_checked(bytes).ok()
    }

    /// Decode payload with strict bounded memo checks.
    pub fn decode_strict(bytes: &[u8]) -> Result<Self, PackErr> {
        if bytes.len() < Self::HEAD_SIZE {
            return Err(PackErr::BadLen);
        }

        let base = AssetPackPlain::decode_strict(&bytes[..AssetPackPlain::SIZE])?;
        let memo_len =
            u16::from_le_bytes([bytes[AssetPackPlain::SIZE], bytes[AssetPackPlain::SIZE + 1]])
                as usize;

        if memo_len > Self::MEMO_MAX {
            return Err(PackErr::BadMemo);
        }

        if bytes.len() != Self::HEAD_SIZE + memo_len {
            return Err(PackErr::BadLen);
        }

        Ok(Self {
            value: base.value,
            blinding: base.blinding,
            s_out: base.s_out,
            memo: bytes[Self::HEAD_SIZE..].to_vec(),
        })
    }

    /// Decode payload and validate blinding encoding.
    pub fn decode_checked(bytes: &[u8]) -> Result<Self, PackErr> {
        let pack = Self::decode_strict(bytes)?;
        if Z00ZScalar::try_from_bytes(pack.blinding).is_err() {
            return Err(PackErr::BadBlind);
        }
        Ok(pack)
    }
}

/// Decode one of the currently live asset-pack lanes.
///
/// `AssetPackVersion::Unknown` is not a migration hook on the live path; it
/// fails closed with `PackErr::BadVer`.
pub fn decode_asset_pack(
    bytes: &[u8],
    version: AssetPackVersion,
) -> Result<DecodedAssetPack, PackErr> {
    match version {
        AssetPackVersion::Basic => {
            AssetPackPlain::decode_checked(bytes).map(DecodedAssetPack::Basic)
        }
        AssetPackVersion::Memo => {
            AssetPackPlainMemo::decode_checked(bytes).map(DecodedAssetPack::Memo)
        }
        AssetPackVersion::Unknown => Err(PackErr::BadVer),
    }
}

#[must_use]
pub fn serialize_asset_pack(pack: &AssetPackPlain) -> Vec<u8> {
    pack.to_bytes()
}

#[must_use]
pub fn deserialize_asset_pack(bytes: &[u8]) -> Option<AssetPackPlain> {
    AssetPackPlain::from_bytes(bytes)
}

#[must_use]
pub fn is_valid_asset_pack_length(bytes: &[u8]) -> bool {
    AssetPackPlain::is_valid_length(bytes)
}

#[cfg(test)]
#[path = "test_leaf.rs"]
mod tests;
