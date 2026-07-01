use jmt::ValueHash;
use sha2::Sha256;
use thiserror::Error;
use z00z_crypto::expert::encoding::{from_hex, to_hex};
use z00z_utils::codec::CodecError;

use crate::settlement::{FeeErr, ModelErr, ProofChkErr, SettlementLeaf, SettlementPath, StoreItem};

#[derive(Debug, Error)]
pub enum SettlementStoreError {
    #[error("codec error: {0}")]
    Codec(#[from] CodecError),
    #[error("model error: {0}")]
    Model(#[from] ModelErr),
    #[error("jmt error: {0}")]
    Jmt(String),
    #[error("empty tree")]
    EmptyTree,
    #[error("terminal key does not match leaf terminal id")]
    KeyLeafMix,
    #[error("terminal id is already bound to a different canonical path")]
    PathTerminalMix,
    #[error("settlement path is missing")]
    PathMiss,
    #[error("duplicate canonical path in one apply_ops call")]
    OpPathDup,
    #[error("state version is missing")]
    HistMiss,
    #[error("unsupported settlement generation: {0}")]
    UnsupportedGeneration(String),
    #[error("proof validation failed: {0}")]
    Proof(#[from] ProofChkErr),
    #[error("fee support rejected: {0}")]
    Fee(#[from] FeeErr),
    #[error("backend failure: {0}")]
    Backend(String),
    #[error("claim replay rejected: {0}")]
    ClaimReplay(String),
    #[error("scheduler backpressure at {stage}: queued {queued} exceeds limit {limit}")]
    SchedBackpressure {
        stage: &'static str,
        queued: usize,
        limit: usize,
    },
    #[error("scheduler cancelled at {stage}")]
    SchedCancel { stage: &'static str },
    #[error("scheduler failure at {stage}: {reason}")]
    Sched { stage: &'static str, reason: String },
    #[error("typed object delta rejected: {0}")]
    ObjectDelta(String),
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum StoreOp {
    Put(Box<StoreItem>),
    Delete(SettlementPath),
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum ClaimNullStatus {
    Spent,
}

#[derive(
    Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, serde::Serialize, serde::Deserialize,
)]
pub struct ClaimNullifier([u8; 32]);

impl ClaimNullifier {
    pub const fn new(bytes: [u8; 32]) -> Self {
        Self(bytes)
    }

    pub const fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }

    pub const fn into_bytes(self) -> [u8; 32] {
        self.0
    }

    pub fn to_hex(&self) -> String {
        to_hex(&self.0)
    }

    pub fn from_hex(raw: &str) -> Result<Self, String> {
        let value = from_hex(raw).map_err(|err| format!("invalid claim nullifier hex: {err}"))?;
        if value.len() != 32 {
            return Err(format!(
                "invalid claim nullifier hex: expected 32 bytes, got {}",
                value.len()
            ));
        }
        let mut bytes = [0u8; 32];
        bytes.copy_from_slice(&value);
        Ok(Self(bytes))
    }
}

impl std::fmt::Display for ClaimNullifier {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.to_hex())
    }
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ClaimNullRec {
    pub nullifier: ClaimNullifier,
    pub status: ClaimNullStatus,
    pub claim_id_hex: String,
    pub chain_id: u32,
    pub tx_digest_hex: String,
    pub created_at_seq: u64,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ClaimNullTx {
    pub nullifier: ClaimNullifier,
    pub claim_id_hex: String,
    // Wallet-side derivation already binds chain_id into the nullifier bytes.
    pub chain_id: u32,
    pub tx_digest_hex: String,
}

pub fn terminal_value_hash(
    leaf: impl Into<SettlementLeaf>,
) -> Result<ValueHash, SettlementStoreError> {
    let payload = leaf.into().encode()?;
    Ok(ValueHash::with::<Sha256>(&payload))
}
