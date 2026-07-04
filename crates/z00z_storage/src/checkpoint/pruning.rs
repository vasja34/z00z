use z00z_crypto::expert::hash_domain;
use z00z_crypto::hash_zk::hash_zk;
use z00z_utils::codec::{BincodeCodec, Codec, JsonCodec};

use crate::CheckpointError;

hash_domain!(
    StorCheckpointPruningDom,
    "z00z.storage.checkpoint.pruning",
    1
);

const PRUNING_DECISION_BIND_VER: u8 = 1;
const PRUNING_DECISION_BIND_LABEL: &str = "pruning_decision_v1";

#[derive(
    Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, serde::Serialize, serde::Deserialize,
)]
pub struct PruningDecisionVersion(u8);

impl PruningDecisionVersion {
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

#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PruningNodeClass {
    FullNode,
    ArchiveNode,
}

impl PruningNodeClass {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::FullNode => "full_node",
            Self::ArchiveNode => "archive_node",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct PruningDecisionV1 {
    version: PruningDecisionVersion,
    node_class: PruningNodeClass,
    prune_scope: String,
    target_epoch: u64,
    dispute_window_elapsed: bool,
    plonky3_epoch_finalized: bool,
    epoch_manifest_finalized: bool,
    archive_replication_threshold_met: bool,
    retrieval_audit_passed: bool,
    compact_metadata_retained: bool,
    epoch_manifest_retained: bool,
    state_snapshot_retained: bool,
    #[serde(default)]
    pruning_decision_bind_ver: u8,
    #[serde(default)]
    pruning_decision_bind: [u8; 32],
}

impl PruningDecisionV1 {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        version: PruningDecisionVersion,
        node_class: PruningNodeClass,
        prune_scope: impl Into<String>,
        target_epoch: u64,
        dispute_window_elapsed: bool,
        plonky3_epoch_finalized: bool,
        epoch_manifest_finalized: bool,
        archive_replication_threshold_met: bool,
        retrieval_audit_passed: bool,
        compact_metadata_retained: bool,
        epoch_manifest_retained: bool,
        state_snapshot_retained: bool,
    ) -> Result<Self, CheckpointError> {
        check_pruning_decision_ver(version)?;
        let prune_scope = prune_scope.into();
        if node_class != PruningNodeClass::FullNode
            || prune_scope != "local_full_node_only"
            || target_epoch == 0
            || !dispute_window_elapsed
            || !plonky3_epoch_finalized
            || !epoch_manifest_finalized
            || !archive_replication_threshold_met
            || !retrieval_audit_passed
            || !compact_metadata_retained
            || !epoch_manifest_retained
            || !state_snapshot_retained
        {
            return Err(CheckpointError::PruningMix);
        }
        let pruning_decision_bind = pruning_decision_bind(
            node_class,
            prune_scope.as_bytes(),
            target_epoch,
            dispute_window_elapsed,
            plonky3_epoch_finalized,
            epoch_manifest_finalized,
            archive_replication_threshold_met,
            retrieval_audit_passed,
            compact_metadata_retained,
            epoch_manifest_retained,
            state_snapshot_retained,
        );
        Ok(Self {
            version,
            node_class,
            prune_scope,
            target_epoch,
            dispute_window_elapsed,
            plonky3_epoch_finalized,
            epoch_manifest_finalized,
            archive_replication_threshold_met,
            retrieval_audit_passed,
            compact_metadata_retained,
            epoch_manifest_retained,
            state_snapshot_retained,
            pruning_decision_bind_ver: PRUNING_DECISION_BIND_VER,
            pruning_decision_bind,
        })
    }

    #[must_use]
    pub const fn version(&self) -> PruningDecisionVersion {
        self.version
    }

    #[must_use]
    pub const fn node_class(&self) -> PruningNodeClass {
        self.node_class
    }

    #[must_use]
    pub fn prune_scope(&self) -> &str {
        &self.prune_scope
    }

    #[must_use]
    pub const fn target_epoch(&self) -> u64 {
        self.target_epoch
    }

    fn check_bind(&self) -> Result<(), CheckpointError> {
        if self.pruning_decision_bind_ver != PRUNING_DECISION_BIND_VER {
            return Err(CheckpointError::PruningMix);
        }
        if self.pruning_decision_bind
            != pruning_decision_bind(
                self.node_class,
                self.prune_scope.as_bytes(),
                self.target_epoch,
                self.dispute_window_elapsed,
                self.plonky3_epoch_finalized,
                self.epoch_manifest_finalized,
                self.archive_replication_threshold_met,
                self.retrieval_audit_passed,
                self.compact_metadata_retained,
                self.epoch_manifest_retained,
                self.state_snapshot_retained,
            )
        {
            return Err(CheckpointError::PruningMix);
        }
        Ok(())
    }
}

pub(crate) fn check_pruning_decision_ver(
    version: PruningDecisionVersion,
) -> Result<(), CheckpointError> {
    if version == PruningDecisionVersion::CURRENT {
        return Ok(());
    }
    Err(CheckpointError::VersionMix)
}

pub(crate) fn encode_pruning_decision_bin_checked(
    decision: &PruningDecisionV1,
) -> Result<Vec<u8>, CheckpointError> {
    check_pruning_decision_ver(decision.version())?;
    decision.check_bind()?;
    Ok(BincodeCodec.serialize(decision)?)
}

pub(crate) fn decode_pruning_decision_bin_checked(
    bytes: &[u8],
) -> Result<PruningDecisionV1, CheckpointError> {
    let decision: PruningDecisionV1 = BincodeCodec.deserialize(bytes)?;
    check_pruning_decision_ver(decision.version())?;
    decision.check_bind()?;
    Ok(decision)
}

pub(crate) fn encode_pruning_decision_json_checked(
    decision: &PruningDecisionV1,
) -> Result<Vec<u8>, CheckpointError> {
    check_pruning_decision_ver(decision.version())?;
    decision.check_bind()?;
    Ok(JsonCodec.serialize_pretty(decision)?)
}

pub(crate) fn decode_pruning_decision_json_checked(
    bytes: &[u8],
) -> Result<PruningDecisionV1, CheckpointError> {
    let decision: PruningDecisionV1 = JsonCodec.deserialize(bytes)?;
    check_pruning_decision_ver(decision.version())?;
    decision.check_bind()?;
    Ok(decision)
}

#[allow(clippy::too_many_arguments)]
fn pruning_decision_bind(
    node_class: PruningNodeClass,
    prune_scope: &[u8],
    target_epoch: u64,
    dispute_window_elapsed: bool,
    plonky3_epoch_finalized: bool,
    epoch_manifest_finalized: bool,
    archive_replication_threshold_met: bool,
    retrieval_audit_passed: bool,
    compact_metadata_retained: bool,
    epoch_manifest_retained: bool,
    state_snapshot_retained: bool,
) -> [u8; 32] {
    let target_epoch = target_epoch.to_le_bytes();
    let flags = [
        u8::from(dispute_window_elapsed),
        u8::from(plonky3_epoch_finalized),
        u8::from(epoch_manifest_finalized),
        u8::from(archive_replication_threshold_met),
        u8::from(retrieval_audit_passed),
        u8::from(compact_metadata_retained),
        u8::from(epoch_manifest_retained),
        u8::from(state_snapshot_retained),
    ];
    hash_zk::<StorCheckpointPruningDom>(
        PRUNING_DECISION_BIND_LABEL,
        &[
            node_class.as_str().as_bytes(),
            prune_scope,
            &target_epoch,
            &flags,
        ],
    )
}

#[cfg(test)]
mod tests {
    use super::{
        check_pruning_decision_ver, PruningDecisionV1, PruningDecisionVersion, PruningNodeClass,
    };
    use crate::CheckpointError;

    fn good_decision() -> PruningDecisionV1 {
        PruningDecisionV1::new(
            PruningDecisionVersion::CURRENT,
            PruningNodeClass::FullNode,
            "local_full_node_only",
            10,
            true,
            true,
            true,
            true,
            true,
            true,
            true,
            true,
        )
        .expect("pruning decision")
    }

    #[test]
    fn test_full_node_pruning_builds() {
        let decision = good_decision();

        assert_eq!(decision.node_class(), PruningNodeClass::FullNode);
        assert_eq!(decision.prune_scope(), "local_full_node_only");
    }

    #[test]
    fn test_archive_node_pruning_rejects() {
        let err = PruningDecisionV1::new(
            PruningDecisionVersion::CURRENT,
            PruningNodeClass::ArchiveNode,
            "local_full_node_only",
            10,
            true,
            true,
            true,
            true,
            true,
            true,
            true,
            true,
        )
        .expect_err("archive node pruning rejects");

        assert!(matches!(err, CheckpointError::PruningMix));
    }

    #[test]
    fn test_pruning_audit_missing() {
        let err = PruningDecisionV1::new(
            PruningDecisionVersion::CURRENT,
            PruningNodeClass::FullNode,
            "local_full_node_only",
            10,
            true,
            true,
            true,
            true,
            false,
            true,
            true,
            true,
        )
        .expect_err("early pruning rejects");

        assert!(matches!(err, CheckpointError::PruningMix));
    }

    #[test]
    fn test_pruning_bad_version_rejects() {
        let err =
            check_pruning_decision_ver(PruningDecisionVersion::new(9)).expect_err("bad version");

        assert!(matches!(err, CheckpointError::VersionMix));
    }
}
