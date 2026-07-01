use crate::{
    checkpoint::CheckpointExecInputId,
    settlement::{CheckRoot, ClaimSourceRoot, SettlementStateRoot},
    snapshot::PrepSnapshotId,
};

use super::{CheckpointDraft, CheckpointPubIn, CheckpointVersion, CreatedEnt, SpentEnt};

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct CheckpointStmt {
    checkpoint_version: CheckpointVersion,
    height: u64,
    prev_root: CheckRoot,
    new_root: CheckRoot,
    prev_settlement_root: SettlementStateRoot,
    new_settlement_root: SettlementStateRoot,
    claim_root: Option<ClaimSourceRoot>,
    spent_delta: Vec<SpentEnt>,
    created_delta: Vec<CreatedEnt>,
    prep_snapshot_id: PrepSnapshotId,
    exec_input_id: CheckpointExecInputId,
}

impl CheckpointStmt {
    #[must_use]
    pub fn new(
        checkpoint_version: CheckpointVersion,
        height: u64,
        pub_in: CheckpointPubIn,
        prep_snapshot_id: PrepSnapshotId,
        exec_input_id: CheckpointExecInputId,
    ) -> Self {
        Self {
            checkpoint_version,
            height,
            prev_root: pub_in.prev_root(),
            new_root: pub_in.new_root(),
            prev_settlement_root: pub_in.prev_settlement_root(),
            new_settlement_root: pub_in.new_settlement_root(),
            claim_root: pub_in.claim_root(),
            spent_delta: pub_in.spent_delta().to_vec(),
            created_delta: pub_in.created_delta().to_vec(),
            prep_snapshot_id,
            exec_input_id,
        }
    }

    #[must_use]
    pub fn from_draft(
        draft: &CheckpointDraft,
        prep_snapshot_id: PrepSnapshotId,
        exec_input_id: CheckpointExecInputId,
    ) -> Self {
        Self::new(
            draft.version(),
            draft.height(),
            draft.pub_in(),
            prep_snapshot_id,
            exec_input_id,
        )
    }

    #[must_use]
    pub const fn checkpoint_version(&self) -> CheckpointVersion {
        self.checkpoint_version
    }

    #[must_use]
    pub const fn height(&self) -> u64 {
        self.height
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
    pub const fn claim_root(&self) -> Option<ClaimSourceRoot> {
        self.claim_root
    }

    #[must_use]
    pub fn spent_delta(&self) -> &[SpentEnt] {
        &self.spent_delta
    }

    #[must_use]
    pub fn created_delta(&self) -> &[CreatedEnt] {
        &self.created_delta
    }

    #[must_use]
    pub const fn prep_snapshot_id(&self) -> PrepSnapshotId {
        self.prep_snapshot_id
    }

    #[must_use]
    pub const fn exec_input_id(&self) -> CheckpointExecInputId {
        self.exec_input_id
    }

    #[must_use]
    pub fn backend_payload(&self) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(164);
        bytes.push(self.prev_settlement_root.generation_version());
        bytes.extend_from_slice(self.prev_settlement_root.as_bytes());
        bytes.push(self.new_settlement_root.generation_version());
        bytes.extend_from_slice(self.new_settlement_root.as_bytes());
        match self.claim_root {
            Some(claim_root) => {
                bytes.push(1);
                bytes.push(claim_root.root_version());
                bytes.extend_from_slice(claim_root.as_bytes());
            }
            None => {
                bytes.push(0);
                bytes.push(0);
                bytes.extend_from_slice(&[0u8; 32]);
            }
        }
        bytes.extend_from_slice(self.exec_input_id.as_bytes());
        bytes.extend_from_slice(self.new_root.as_bytes());
        bytes
    }

    #[must_use]
    pub fn pub_in(&self) -> CheckpointPubIn {
        let mut pub_in = CheckpointPubIn::new_settlement(
            self.prev_settlement_root,
            self.new_settlement_root,
            self.spent_delta.clone(),
            self.created_delta.clone(),
        );
        if let Some(claim_root) = self.claim_root {
            pub_in = pub_in.with_claim_root(claim_root);
        }
        pub_in
    }

    #[must_use]
    pub(super) fn matches_draft(&self, draft: &CheckpointDraft) -> bool {
        self.checkpoint_version == draft.version()
            && self.height == draft.height()
            && self.prev_root == draft.prev_root()
            && self.new_root == draft.new_root()
            && self.prev_settlement_root == draft.prev_settlement_root()
            && self.new_settlement_root == draft.new_settlement_root()
            && self.claim_root == draft.claim_root()
            && self.spent_delta == draft.spent_delta()
            && self.created_delta == draft.created_delta()
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CheckpointStatement {
    Detached,
    CURRENT(Box<CheckpointStmt>),
}

pub trait WalletDraft {
    fn draft_height(&self) -> u64;
    fn draft_prev_root(&self) -> CheckRoot;
    fn draft_new_root(&self) -> CheckRoot;
    fn draft_spent(&self) -> Vec<SpentEnt>;
    fn draft_created(&self) -> Vec<CreatedEnt>;

    fn draft_claim_root(&self) -> Option<ClaimSourceRoot> {
        None
    }

    fn draft_prev_settlement_root(&self) -> SettlementStateRoot {
        SettlementStateRoot::settlement_v1(self.draft_prev_root().into_bytes())
    }

    fn draft_new_settlement_root(&self) -> SettlementStateRoot {
        SettlementStateRoot::settlement_v1(self.draft_new_root().into_bytes())
    }
}
