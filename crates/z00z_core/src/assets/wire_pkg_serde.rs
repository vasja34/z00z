use super::{
    Asset, AssetClass, AssetDefinition, AssetError, AssetPkgWire, AssetWire, ByteArray, Codec,
    Commitment, Cow, DefinitionWire, JsonCodec, KernelSignature, RangeProof, Z00ZRistrettoPoint,
    Z00ZScalar, ZkPackEncrypted, ASSET_PKG_JSON_MAX_BYTES,
};

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct DefinitionPkg {
    id: String,
    class: AssetClass,
    name: String,
    symbol: String,
    decimals: u8,
    serials: u32,
    nominal: u64,
    domain_name: String,
    version: u8,
    crypto_version: u8,
    policy_flags: u8,
    metadata: Option<std::collections::BTreeMap<String, String>>,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct AssetPkgSerde {
    definition: DefinitionPkg,
    serial_id: u32,
    amount: u64,
    commitment: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    range_proof: Option<String>,
    nonce: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    lock_height: Option<u64>,
    is_burned: bool,
    #[serde(default, skip_serializing_if = "is_false")]
    is_frozen: bool,
    #[serde(default, skip_serializing_if = "is_false")]
    is_slashed: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    owner_pub: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    owner_signature: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    r_pub: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    owner_tag: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    enc_pack: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    tag16: Option<u16>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    leaf_ad_id: Option<String>,
}

struct CorePkgParsed {
    definition: AssetDefinition,
    commitment: Commitment,
    range_proof: Option<RangeProof>,
    nonce: [u8; 32],
}

struct OwnerPkgParsed {
    owner_pub: Option<Z00ZRistrettoPoint>,
    owner_signature: Option<KernelSignature>,
    r_pub: Option<[u8; 32]>,
    owner_tag: Option<[u8; 32]>,
    enc_pack: Option<ZkPackEncrypted>,
}

include!("wire_pkg_serde_impls.rs");
include!("wire_pkg_serde_parse.rs");
