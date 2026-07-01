use crate::settlement::TerminalId;
use crate::settlement::{CheckRoot, ClaimSourceRoot, SettlementStateRoot};

/// Canonical checkpoint schema version.
#[derive(
    Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, serde::Serialize, serde::Deserialize,
)]
pub struct CheckpointVersion(u8);

impl CheckpointVersion {
    pub const CURRENT: Self = Self(1);

    #[must_use]
    pub const fn new(value: u8) -> Self {
        Self(value)
    }

    #[must_use]
    pub const fn as_u8(self) -> u8 {
        self.0
    }
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct SpentEnt {
    terminal_id: TerminalId,
}

impl SpentEnt {
    #[must_use]
    pub fn new(terminal_id: impl Into<TerminalId>) -> Self {
        Self {
            terminal_id: terminal_id.into(),
        }
    }

    #[must_use]
    pub const fn terminal_id(&self) -> TerminalId {
        self.terminal_id
    }
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct CreatedEnt {
    terminal_id: TerminalId,
    leaf_hash: [u8; 32],
}

impl CreatedEnt {
    #[must_use]
    pub fn new(terminal_id: impl Into<TerminalId>, leaf_hash: [u8; 32]) -> Self {
        Self {
            terminal_id: terminal_id.into(),
            leaf_hash,
        }
    }

    #[must_use]
    pub const fn terminal_id(&self) -> TerminalId {
        self.terminal_id
    }

    #[must_use]
    pub const fn leaf_hash(&self) -> &[u8; 32] {
        &self.leaf_hash
    }
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct CheckpointPubIn {
    prev_root: CheckRoot,
    new_root: CheckRoot,
    prev_settlement_root: SettlementStateRoot,
    new_settlement_root: SettlementStateRoot,
    #[serde(default)]
    claim_root: Option<ClaimSourceRoot>,
    spent_delta: Vec<SpentEnt>,
    created_delta: Vec<CreatedEnt>,
}

impl CheckpointPubIn {
    #[must_use]
    pub fn new(
        prev_root: CheckRoot,
        new_root: CheckRoot,
        spent_delta: Vec<SpentEnt>,
        created_delta: Vec<CreatedEnt>,
    ) -> Self {
        Self::new_settlement(
            SettlementStateRoot::settlement_v1(prev_root.into_bytes()),
            SettlementStateRoot::settlement_v1(new_root.into_bytes()),
            spent_delta,
            created_delta,
        )
    }

    #[must_use]
    pub fn new_settlement(
        prev_settlement_root: SettlementStateRoot,
        new_settlement_root: SettlementStateRoot,
        spent_delta: Vec<SpentEnt>,
        created_delta: Vec<CreatedEnt>,
    ) -> Self {
        Self {
            prev_root: CheckRoot::from(prev_settlement_root),
            new_root: CheckRoot::from(new_settlement_root),
            prev_settlement_root,
            new_settlement_root,
            claim_root: None,
            spent_delta,
            created_delta,
        }
    }

    #[must_use]
    pub fn with_claim_root(mut self, claim_root: ClaimSourceRoot) -> Self {
        self.claim_root = Some(claim_root);
        self
    }

    #[must_use]
    pub const fn prev_root(&self) -> CheckRoot {
        self.prev_root
    }

    #[must_use]
    pub const fn new_root(&self) -> CheckRoot {
        self.new_root
    }

    #[must_use]
    pub const fn prev_settlement_root(&self) -> SettlementStateRoot {
        self.prev_settlement_root
    }

    #[must_use]
    pub const fn new_settlement_root(&self) -> SettlementStateRoot {
        self.new_settlement_root
    }

    #[must_use]
    pub fn spent_delta(&self) -> &[SpentEnt] {
        &self.spent_delta
    }

    #[must_use]
    pub const fn claim_root(&self) -> Option<ClaimSourceRoot> {
        self.claim_root
    }

    #[must_use]
    pub fn created_delta(&self) -> &[CreatedEnt] {
        &self.created_delta
    }
}

#[derive(
    Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, serde::Serialize, serde::Deserialize,
)]
pub struct CheckpointProofSystem(u8);

impl CheckpointProofSystem {
    pub const OPAQUE_ATTEST: Self = Self(2);
    pub const VERIFIED: Self = Self(3);

    #[must_use]
    pub const fn new(value: u8) -> Self {
        Self(value)
    }

    #[must_use]
    pub const fn as_u8(self) -> u8 {
        self.0
    }

    #[must_use]
    pub const fn is_opaque_attest(self) -> bool {
        self.0 == Self::OPAQUE_ATTEST.0
    }

    #[must_use]
    pub const fn claims_verified(self) -> bool {
        self.0 == Self::VERIFIED.0
    }
}
