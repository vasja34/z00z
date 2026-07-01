use super::{
    Asset, AssetClass, AssetDefinition, AssetError, AssetWire, Commitment, DefinitionWire,
    KernelSignature, RangeProof, Z00ZRistrettoPoint,
};
use std::borrow::Cow;
use z00z_crypto::expert::encoding::ByteArray;
use z00z_crypto::Z00ZScalar;
use z00z_crypto::ZkPackEncrypted;
use z00z_utils::codec::{Codec, JsonCodec};

/// Maximum accepted byte size for human-readable `AssetPkgWire` JSON payloads.
/// This seam owns the public asset package contract and rejects oversized
/// payloads before secret probing or deserialization.
pub const ASSET_PKG_JSON_MAX_BYTES: usize = 64 * 1024;

/// Canonical frozen external DTO for asset transport.
///
/// This type defines the frozen human-readable JSON contract for tx packages,
/// claim packages, wallet verify and import payloads, and Scenario 1 artifacts.
/// Public JSON boundaries are expected to converge on this DTO rather than on
/// `AssetWire`.
///
/// `AssetWire` remains an internal mutable transport type and must not be used as
/// a public human-readable JSON contract.
///
/// # Security
///
/// This DTO is the explicit non-confidential public boundary. Plaintext `amount`
/// is intentionally preserved here, protocol-state flags are carried explicitly,
/// and trusted-only `secret` material is rejected. The owning seam also enforces
/// `ASSET_PKG_JSON_MAX_BYTES` as a fail-closed ceiling before JSON parsing.
///
/// # Examples
///
/// ```
/// use z00z_core::assets::{AssetClass, AssetPkgWire, AssetWire};
///
/// # #[cfg(feature = "deterministic-rng")]
/// # {
/// let asset = z00z_core::genesis::asset_std::asset_from_dev_class(AssetClass::Coin, 1, 10)
///     .expect("valid dev asset");
/// let dto = AssetPkgWire::from_wire(&AssetWire::from_asset(&asset));
///
/// assert_eq!(dto.serial_id, 1);
/// assert_eq!(dto.amount, 10);
/// # }
/// ```
#[derive(Clone, Debug, PartialEq)]
pub struct AssetPkgWire {
    pub definition: AssetDefinition,
    pub serial_id: u32,
    pub amount: u64,
    pub commitment: Commitment,
    pub range_proof: Option<RangeProof>,
    pub nonce: [u8; 32],
    pub lock_height: Option<u64>,
    pub is_burned: bool,
    pub is_frozen: bool,
    pub is_slashed: bool,
    pub owner_pub: Option<Z00ZRistrettoPoint>,
    pub owner_signature: Option<KernelSignature>,
    pub r_pub: Option<[u8; 32]>,
    pub owner_tag: Option<[u8; 32]>,
    pub enc_pack: Option<ZkPackEncrypted>,
    pub tag16: Option<u16>,
    /// Canonical decrypt/authentication namespace for accepted stealth flows.
    ///
    /// This field is not the canonical consumed-state key. It stays distinct
    /// from `asset_id` and only closes the shipped owned or spendable flow
    /// boundary, not every hypothetical crafted artifact surface.
    /// Existing identity-binding building blocks live here, but the uniform fail-closed policy still belongs to the higher-level receive/send lane.
    pub leaf_ad_id: Option<[u8; 32]>,
}

#[path = "wire_pkg_serde.rs"]
mod wire_pkg_serde;

pub use wire_pkg_serde::{decode_asset_pkg_json, encode_asset_pkg_json, payload_has_secret_field};
