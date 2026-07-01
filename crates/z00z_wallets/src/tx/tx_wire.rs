use z00z_core::assets::AssetPkgWire;

pub(crate) const TX_PACKAGE_KIND: &str = "TxPackage";
pub(crate) const REGULAR_TX_PACKAGE_TYPE: &str = "regular_tx";
pub(crate) const REGULAR_TX_TYPE: &str = "regular_tx";

pub(crate) fn default_regular_package_type() -> String {
    REGULAR_TX_PACKAGE_TYPE.to_string()
}

pub(crate) fn default_regular_tx_type() -> String {
    REGULAR_TX_TYPE.to_string()
}

pub(crate) const SPEND_PROOF_WIRE_VER: u8 = 2;
pub(crate) const SPEND_PROOF_SUITE: &str = "regular_spend_theorem_bpplus";
pub(crate) const SPEND_AUTH_WIRE_VER: u8 = 1;

/// Phase 1 transaction input reference.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TxInputWire {
    /// Canonical state_key encoded as 64 lowercase hex chars.
    ///
    /// This is the wire form of the JMT key for the consumed pre-state leaf.
    pub asset_id_hex: String,
    /// Leaf-match consistency field for the resolved pre-state leaf.
    ///
    /// This is not a second state key and must not be treated as checkpoint
    /// nullifier material.
    pub serial_id: u32,
}

/// Phase 1 transaction output wire.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TxOutRole {
    /// Ordinary transfer output.
    Recipient,
    /// Sender-owned return output.
    Change,
    /// Protocol or sequencer fee output.
    Fee,
}

/// Phase 1 transaction output wire.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TxOutputWire {
    /// Semantic output role inside the transaction package.
    pub role: TxOutRole,
    /// Output leaf as portable wire.
    pub asset_wire: AssetPkgWire,
}

/// Regular tx context object.
#[derive(Debug, Clone, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TxContextWire {}

/// Regular tx proof object.
///
/// This proof object carries local transaction-proof material only.
/// Membership of reference-only inputs stays in the checkpoint/pre-state path,
/// where validators resolve the consumed leaf under `asset_id_hex` and check
/// membership witnesses against `prev_root`.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SpendInputProofWire {
    /// Canonical state_key paired with `tx.inputs` by position.
    pub input_asset_id_hex: String,
    /// Serial id repeated into the signed spend statement for fail-closed pairing.
    pub serial_id: u32,
    /// Deterministic regular-spend nullifier encoded as 32-byte lowercase hex.
    pub nullifier_hex: String,
    /// Input ephemeral public point encoded as 32-byte lowercase hex.
    pub r_pub_hex: String,
    /// Input owner-tag encoded as 32-byte lowercase hex.
    pub owner_tag_hex: String,
    /// Input commitment encoded as 32-byte lowercase hex.
    pub commitment_hex: String,
    /// Canonical `leaf_ad_id` encoded as 32-byte lowercase hex.
    pub leaf_ad_id_hex: String,
    /// Associated-data transcript hash encoded as 32-byte lowercase hex.
    pub leaf_ad_hash_hex: String,
}

/// Public spend-proof object for canonical spend verification.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SpendProofWire {
    /// Spend-proof schema version.
    pub ver: u8,
    /// Proof-suite or parameter family identifier.
    pub proof_suite: String,
    /// Previous root encoded as 32-byte lowercase hex.
    pub prev_root_hex: String,
    /// Canonical public statement payload encoded as lowercase hex.
    pub statement_hex: String,
    /// Opaque proof payload encoded as lowercase hex.
    pub proof_hex: String,
    /// Input leaves paired with `tx.inputs` by position.
    pub inputs: Vec<SpendInputProofWire>,
}

/// Public spend-authorization object.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SpendAuthWire {
    /// Spend-auth schema version.
    pub ver: u8,
    /// Signed receiver card encoded in compact transport form.
    pub receiver_card_compact: String,
    /// Spend-authorization signature encoded as 64-byte lowercase hex.
    pub spend_sig_hex: String,
}

/// Regular tx proof container.
#[derive(Debug, Clone, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TxProofWire {
    /// Canonical public spend proof for accepted spend authorization paths.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub spend: Option<SpendProofWire>,
}

/// Regular tx auth object.
#[derive(Debug, Clone, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TxAuthWire {
    /// Canonical public spend authorization for accepted spend paths.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub spend: Option<SpendAuthWire>,
}

/// Phase 1 transaction wire.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TxWire {
    /// Transaction subtype marker.
    #[serde(default = "default_regular_tx_type")]
    pub tx_type: String,
    /// Input references.
    ///
    /// Each input is an input_ref to an existing pre-state leaf.
    ///
    /// The transaction package does not inline the consumed leaf bytes. Final
    /// spend semantics resolve the leaf by `asset_id_hex`, enforce a
    /// leaf_match on `serial_id`, and keep membership validation in the
    /// checkpoint/pre-state path rather than in the local tx proof.
    pub inputs: Vec<TxInputWire>,
    /// Output leaves.
    pub outputs: Vec<TxOutputWire>,
    /// Fee amount metadata.
    ///
    /// This scalar is metadata only and must equal the sum of `Fee` outputs.
    /// Economic value conservation is carried by the output set itself.
    pub fee: u64,
    /// Sender sequencing nonce.
    pub nonce: u64,
    /// Additional tx context.
    #[serde(default)]
    pub context: TxContextWire,
    /// Public proof object.
    #[serde(default)]
    pub proof: TxProofWire,
    /// Public authorization object.
    #[serde(default)]
    pub auth: TxAuthWire,
}

/// Phase 1 transaction package.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TxPackage {
    /// Package kind marker.
    pub kind: String,
    /// Package subtype marker.
    #[serde(default = "default_regular_package_type")]
    pub package_type: String,
    /// Package version.
    pub version: u8,
    /// Numeric chain identifier.
    pub chain_id: u32,
    /// Chain classification.
    pub chain_type: String,
    /// Human-readable chain name.
    pub chain_name: String,
    /// Transaction payload.
    pub tx: TxWire,
    /// Hex digest (32 bytes hex string).
    pub tx_digest_hex: String,
    /// Lifecycle status.
    pub status: String,
}

pub(crate) fn decode_tx_input_asset_id(value: &str) -> Result<[u8; 32], &'static str> {
    const ERR: &str = "tx input asset_id_hex must be 32-byte lowercase hex";

    let bytes = hex::decode(value).map_err(|_| ERR)?;
    let bytes: [u8; 32] = bytes.try_into().map_err(|_| ERR)?;
    if hex::encode(bytes) != value {
        return Err(ERR);
    }
    Ok(bytes)
}

pub(crate) fn canonicalize_tx_inputs(tx: &TxWire) -> Result<TxWire, &'static str> {
    let mut canonical = tx.clone();
    for input in &mut canonical.inputs {
        input.asset_id_hex = hex::encode(decode_tx_input_asset_id(&input.asset_id_hex)?);
    }
    Ok(canonical)
}
