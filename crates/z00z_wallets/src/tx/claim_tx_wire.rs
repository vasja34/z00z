use serde::{Deserialize, Serialize};

use z00z_core::assets::AssetPkgWire;

pub(crate) const CLAIM_TX_PACKAGE_TYPE: &str = "claim_tx";
pub(crate) const CLAIM_TX_TYPE: &str = "claim_tx";
// Outer transport tag only. The inner source-proof version stays governed by
// ClaimProofVer until a separate protocol migration changes that boundary.
pub(crate) const CLAIM_SOURCE_PROOF_TAG: &str = "claim_source";

fn default_claim_package_type() -> String {
    CLAIM_TX_PACKAGE_TYPE.to_string()
}

fn default_claim_chain_id() -> u32 {
    3
}

fn default_claim_chain_type() -> String {
    "devnet".to_string()
}

fn default_claim_chain_name() -> String {
    "z00z-devnet-1".to_string()
}

fn default_claim_tx_type() -> String {
    CLAIM_TX_TYPE.to_string()
}

/// Canonical claim package version.
pub const CLAIM_PKG: u32 = 1;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(deny_unknown_fields)]
/// One output produced by a claim transaction.
pub struct ClaimOutputWire {
    /// Asset identifier encoded as 64 lowercase hex chars.
    pub asset_id_hex: String,
    /// Output amount in base units.
    pub amount: u64,
    /// Asset class discriminator (Scenario-1 uses `coin`).
    pub asset_class: String,
    /// Owner binding encoded as 64 lowercase hex chars.
    pub owner_binding_hex: String,
    /// Output nonce encoded as 64 lowercase hex chars.
    pub nonce_hex: String,
    /// Full stealth asset wire for JMT publish and wallet import.
    /// Matches the regular tx output carrier semantics and is mandatory for canonical claim packages.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub asset_wire: Option<AssetPkgWire>,
    /// Recipient identity signature over the output asset-wire binding.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub owner_attest_hex: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(deny_unknown_fields)]
/// Scope key used to derive a deterministic scope hash for claims.
pub struct ClaimScopeKey {
    /// Chain id for domain separation.
    pub chain_id: u32,
    /// Scenario tag for domain separation.
    pub scenario_tag: String,
    /// Ruleset version included in the scope hash.
    pub ruleset_version: u32,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(deny_unknown_fields)]
/// Claim-source reference carried in canonical tx inputs.
pub struct ClaimInputWire {
    /// Claim identifier encoded as 64 lowercase hex chars.
    pub claim_id_hex: String,
    /// Claim source asset identifier encoded as 64 lowercase hex chars.
    pub claim_source_asset_id_hex: String,
    /// Claim source commitment encoded as 64 lowercase hex chars.
    pub claim_source_commitment_hex: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(deny_unknown_fields)]
/// Claim tx context fields.
pub struct ClaimContextWire {
    /// Recipient wallet identifier.
    pub recipient_wallet_id: String,
    /// Recipient owner key binding encoded as 64 lowercase hex chars.
    pub recipient_owner_hex: String,
    /// Scope hash encoded as 64 lowercase hex chars.
    pub claim_scope_hash_hex: String,
    /// Canonical signed receiver card encoded as lowercase hex.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub recipient_card_hex: Option<String>,
    /// Nullifier encoded as 64 lowercase hex chars.
    pub nullifier_hex: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(deny_unknown_fields)]
/// Claim tx proof fields.
pub struct ClaimProofWire {
    /// Proof type tag (`genesis_claim` for Scenario-1).
    pub proof_type: String,
    /// Proof blob encoded as lowercase hex.
    pub proof_hex: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(deny_unknown_fields)]
/// Claim tx auth fields.
pub struct ClaimAuthWire {
    /// Claim authority signature encoded as lowercase hex.
    pub claim_authority_sig_hex: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(deny_unknown_fields)]
/// Core wire payload for a claim transaction.
pub struct ClaimTxWire {
    /// Transaction subtype marker.
    #[serde(default = "default_claim_tx_type")]
    pub tx_type: String,
    /// Claim-source references.
    pub inputs: Vec<ClaimInputWire>,
    /// Created outputs of this claim.
    pub outputs: Vec<ClaimOutputWire>,
    /// Transaction fee (must be zero in Scenario-1).
    pub fee: u64,
    /// Sender nonce for sequencing.
    pub nonce: u64,
    /// Context fields.
    pub context: ClaimContextWire,
    /// Public proof object.
    pub proof: ClaimProofWire,
    /// Public auth object.
    pub auth: ClaimAuthWire,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(deny_unknown_fields)]
/// Top-level claim package envelope.
pub struct ClaimTxPackage {
    /// Package kind discriminator.
    pub kind: String,
    /// Package subtype discriminator.
    #[serde(default = "default_claim_package_type")]
    pub package_type: String,
    /// Package version.
    pub version: u32,
    /// Chain id for this package.
    #[serde(default = "default_claim_chain_id")]
    pub chain_id: u32,
    /// Chain classification.
    #[serde(default = "default_claim_chain_type")]
    pub chain_type: String,
    /// Human-readable chain name.
    #[serde(default = "default_claim_chain_name")]
    pub chain_name: String,
    /// Embedded claim transaction payload.
    pub tx: ClaimTxWire,
    /// Canonical digest over envelope context and `tx` payload.
    pub tx_digest_hex: String,
    /// Lifecycle status string.
    pub status: String,
}
