use serde::{Deserialize, Serialize};

use crate::{
    settlement::{
        BucketId, ClaimNullRec, FeeReplayRec, ObjectDeltaSetV1, SettlementPath, SettlementStateRoot,
    },
    snapshot::PrepSnapshotId,
};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct StateMeta {
    pub(crate) version: u64,
    pub(crate) state_root: [u8; 32],
    pub(crate) flat_root: [u8; 32],
    pub(crate) snap_id: [u8; 32],
    pub(crate) draft_id: [u8; 32],
    pub(crate) check_id: [u8; 32],
    pub(crate) exec_id: [u8; 32],
    pub(crate) def_root: Option<[u8; 32]>,
    #[serde(default)]
    pub(crate) fee_replay_count: u64,
    #[serde(default)]
    pub(crate) fee_replay_digest: [u8; 32],
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct CanonExec {
    pub(crate) exec_id: crate::checkpoint::CheckpointExecInputId,
    pub(crate) exec_bytes: Vec<u8>,
}

impl CanonExec {
    pub(crate) fn new(
        exec_id: crate::checkpoint::CheckpointExecInputId,
        exec_bytes: Vec<u8>,
    ) -> Self {
        Self {
            exec_id,
            exec_bytes,
        }
    }
}

pub(crate) struct WriteArts {
    pub(crate) version: u64,
    pub(crate) snap_id: PrepSnapshotId,
    pub(crate) snap_bytes: Vec<u8>,
    pub(crate) canon_exec: Option<CanonExec>,
    pub(crate) spent: Vec<crate::checkpoint::SpentEnt>,
    pub(crate) created: Vec<crate::checkpoint::CreatedEnt>,
}

impl WriteArts {
    pub(crate) fn new(
        version: u64,
        snap_id: PrepSnapshotId,
        snap_bytes: Vec<u8>,
        canon_exec: Option<CanonExec>,
        spent: Vec<crate::checkpoint::SpentEnt>,
        created: Vec<crate::checkpoint::CreatedEnt>,
    ) -> Self {
        Self {
            version,
            snap_id,
            snap_bytes,
            canon_exec,
            spent,
            created,
        }
    }
}

#[derive(Clone)]
pub(crate) struct LoadState {
    pub(crate) version: u64,
    pub(crate) state_root: SettlementStateRoot,
    pub(crate) flat_root: [u8; 32],
    pub(crate) hjmt_terminal_rows: Vec<(SettlementPath, BucketId, Vec<u8>)>,
    pub(crate) hjmt_settlement_path_rows: Vec<(SettlementPath, Vec<u8>)>,
    pub(crate) claim_null_rows: Vec<ClaimNullRec>,
    pub(crate) fee_replay_rows: Vec<FeeReplayRec>,
    pub(crate) object_delta: Option<ObjectDeltaSetV1>,
    pub(crate) hjmt_journal: Option<crate::settlement::hjmt_journal::HjmtCommitJournalEntry>,
}
