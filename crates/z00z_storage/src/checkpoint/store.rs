use std::path::PathBuf;

use z00z_utils::io::read_file;

use crate::error::CheckpointError;
use crate::snapshot::{PrepFsStore, PrepSnapshot, PrepSnapshotId, PrepSnapshotStore};

use super::{
    audit::CheckpointAudit,
    codec::{
        decode_art_bin, decode_audit_bin, decode_draft_bin, decode_exec_bin, decode_link_bin,
        encode_art_bin, encode_audit_bin, encode_draft_bin, encode_exec_bin, encode_link_bin,
    },
    exec_input::CheckpointExecInput,
    ids::{
        derive_checkpoint_id, derive_draft_id, derive_exec_id, CheckpointDraftId,
        CheckpointExecInputId, CheckpointId,
    },
    link::CheckpointLink,
    store_fs::CheckpointFinalLane,
    CheckpointArtifact, CheckpointDraft, CheckpointProof,
};

/// Load one canonical checkpoint draft from canonical storage bytes.
///
/// # Examples
///
/// ```
/// use z00z_storage::{
///     checkpoint::{load_draft, CheckpointDraft, CheckpointVersion, CreatedEnt, SpentEnt},
///     settlement::CheckRoot,
/// };
/// use z00z_utils::codec::{BincodeCodec, Codec};
///
/// let draft = CheckpointDraft::new(
///     CheckpointVersion::CURRENT,
///     9,
///     CheckRoot::new([1u8; 32]),
///     CheckRoot::new([2u8; 32]),
///     vec![SpentEnt::new([3u8; 32])],
///     vec![CreatedEnt::new([4u8; 32], [5u8; 32])],
/// );
/// let bytes = BincodeCodec.serialize(&draft)?;
/// let loaded = load_draft(&bytes)?;
/// assert_eq!(loaded, draft);
/// # Ok::<(), z00z_storage::CheckpointError>(())
/// ```
pub fn load_draft(bytes: &[u8]) -> Result<CheckpointDraft, CheckpointError> {
    match decode_draft_bin(bytes) {
        Ok(draft) => Ok(draft),
        Err(err) => {
            if decode_art_bin(bytes).is_ok() {
                return Err(CheckpointError::WrongClass);
            }
            Err(err)
        }
    }
}

/// Load one canonical final checkpoint artifact from canonical storage bytes.
///
/// # Examples
///
/// ```
/// use z00z_storage::{
///     checkpoint::{load_artifact, CheckpointArtifact, CheckpointDraft, CheckpointExecInputId, CheckpointVersion, CreatedEnt, SpentEnt},
///     settlement::CheckRoot,
///     snapshot::PrepSnapshotId,
/// };
/// use z00z_utils::codec::{BincodeCodec, Codec};
///
/// let draft = CheckpointDraft::new(
///     CheckpointVersion::CURRENT,
///     9,
///     CheckRoot::new([1u8; 32]),
///     CheckRoot::new([2u8; 32]),
///     vec![SpentEnt::new([3u8; 32])],
///     vec![CreatedEnt::new([4u8; 32], [5u8; 32])],
/// );
/// let proof = draft.attest_proof(
///     PrepSnapshotId::new([6u8; 32]),
///     CheckpointExecInputId::new([7u8; 32]),
/// )?;
/// let art = draft.finalize(proof)?;
/// let bytes = BincodeCodec.serialize(&art)?;
/// let loaded = load_artifact(&bytes)?;
/// assert_eq!(loaded, art);
/// # Ok::<(), z00z_storage::CheckpointError>(())
/// ```
pub fn load_artifact(bytes: &[u8]) -> Result<CheckpointArtifact, CheckpointError> {
    match decode_art_bin(bytes) {
        Ok(artifact) => Ok(artifact),
        Err(err) => {
            if decode_draft_bin(bytes).is_ok() {
                return Err(CheckpointError::WrongClass);
            }
            Err(err)
        }
    }
}

/// Check one backend draft key against its expected external id.
pub fn check_draft_key(
    want: CheckpointDraftId,
    got: CheckpointDraftId,
) -> Result<(), CheckpointError> {
    if want == got {
        return Ok(());
    }

    Err(CheckpointError::KeyMix)
}

/// Check one backend-final-artifact key against its expected external id.
///
/// # Examples
///
/// ```
/// use z00z_storage::checkpoint::{check_art_key, CheckpointId};
///
/// let art_id = CheckpointId::new([7u8; 32]);
/// check_art_key(art_id, art_id)?;
/// # Ok::<(), z00z_storage::CheckpointError>(())
/// ```
pub fn check_art_key(want: CheckpointId, got: CheckpointId) -> Result<(), CheckpointError> {
    if want == got {
        return Ok(());
    }

    Err(CheckpointError::KeyMix)
}

/// Check one backend execution-input key against its expected external id.
///
/// # Examples
///
/// ```
/// use z00z_storage::checkpoint::{check_exec_key, CheckpointExecInputId};
///
/// let exec_id = CheckpointExecInputId::new([7u8; 32]);
/// check_exec_key(exec_id, exec_id)?;
/// # Ok::<(), z00z_storage::CheckpointError>(())
/// ```
pub fn check_exec_key(
    want: CheckpointExecInputId,
    got: CheckpointExecInputId,
) -> Result<(), CheckpointError> {
    if want == got {
        return Ok(());
    }

    Err(CheckpointError::KeyMix)
}

/// Check one canonical replay link against snapshot and execution input ids.
pub fn check_link_ids(
    snap_id: PrepSnapshotId,
    link: &CheckpointLink,
    exec: &CheckpointExecInput,
) -> Result<CheckpointExecInputId, CheckpointError> {
    if link.prep_snapshot_id() != snap_id || exec.prep_snapshot_id() != snap_id {
        return Err(CheckpointError::LinkMix);
    }

    let exec_id = derive_exec_id(&encode_exec_bin(exec)?);
    if link.exec_input_id() != exec_id {
        return Err(CheckpointError::ReplayMix);
    }

    Ok(exec_id)
}

/// Check one execution input root against one validated snapshot root.
pub fn check_exec_root(
    snapshot: &PrepSnapshot,
    exec: &CheckpointExecInput,
) -> Result<(), CheckpointError> {
    if snapshot.prev_root != exec.prev_root() {
        return Err(CheckpointError::RootMix);
    }

    Ok(())
}

fn check_exec_replay(
    snap_id: PrepSnapshotId,
    exec_id: CheckpointExecInputId,
    exec: &CheckpointExecInput,
) -> Result<(), CheckpointError> {
    if exec.prep_snapshot_id() != snap_id {
        return Err(CheckpointError::LinkMix);
    }

    let got = derive_exec_id(&encode_exec_bin(exec)?);
    if got != exec_id {
        return Err(CheckpointError::ReplayMix);
    }

    Ok(())
}

/// Narrow storage-owned checkpoint facade.
pub trait CheckpointStore {
    fn save_draft(&mut self, draft: &CheckpointDraft)
        -> Result<CheckpointDraftId, CheckpointError>;

    fn load_draft(&self, draft_id: &CheckpointDraftId) -> Result<CheckpointDraft, CheckpointError>;

    fn load_artifact(
        &self,
        checkpoint_id: &CheckpointId,
    ) -> Result<CheckpointArtifact, CheckpointError>;

    /// Finalize and persist one canonical attested artifact.
    ///
    /// This path succeeds only when the attested statement matches persisted
    /// snapshot and execution-input rows that already exist as replay evidence.
    fn seal_artifact(
        &mut self,
        draft: &CheckpointDraft,
        proof: CheckpointProof,
        snap_id: PrepSnapshotId,
        exec_id: CheckpointExecInputId,
    ) -> Result<CheckpointLink, CheckpointError>;

    fn save_link(&mut self, link: &CheckpointLink) -> Result<(), CheckpointError>;

    fn load_link(&self, checkpoint_id: &CheckpointId) -> Result<CheckpointLink, CheckpointError>;

    fn save_exec_input(
        &mut self,
        exec: &CheckpointExecInput,
    ) -> Result<CheckpointExecInputId, CheckpointError>;

    fn load_exec_input(
        &self,
        exec_id: &CheckpointExecInputId,
    ) -> Result<CheckpointExecInput, CheckpointError>;

    fn save_audit(&mut self, audit: &CheckpointAudit) -> Result<(), CheckpointError>;

    fn load_audit(&self, checkpoint_id: &CheckpointId) -> Result<CheckpointAudit, CheckpointError>;
}

/// File-backed checkpoint store with separate persistence surfaces.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CheckpointFsStore {
    pub(super) root: PathBuf,
}

impl CheckpointFsStore {
    #[must_use]
    pub fn new(root: impl Into<PathBuf>) -> Self {
        Self { root: root.into() }
    }

    fn persist_artifact_bin(
        &mut self,
        artifact: &CheckpointArtifact,
    ) -> Result<CheckpointId, CheckpointError> {
        let checkpoint_id = derive_checkpoint_id(artifact)?;
        let bytes = encode_art_bin(artifact)?;
        Self::save_bin(&self.artifact_path(&checkpoint_id), &bytes)?;
        Ok(checkpoint_id)
    }

    pub(super) fn load_persisted_artifact(
        &self,
        checkpoint_id: &CheckpointId,
    ) -> Result<CheckpointArtifact, CheckpointError> {
        let bytes = read_file(self.artifact_path(checkpoint_id))
            .map_err(|err| CheckpointError::Backend(err.to_string()))?;
        let artifact = load_artifact(&bytes)?;
        let got = derive_checkpoint_id(&artifact)?;
        check_art_key(*checkpoint_id, got)?;
        Ok(artifact)
    }

    fn persist_audit_bin(&self, audit: &CheckpointAudit) -> Result<(), CheckpointError> {
        let bytes = encode_audit_bin(audit)?;
        Self::save_bin(&self.audit_path(&audit.checkpoint_id()), &bytes)
    }

    fn load_persisted_audit(
        &self,
        checkpoint_id: &CheckpointId,
    ) -> Result<CheckpointAudit, CheckpointError> {
        let bytes = read_file(self.audit_path(checkpoint_id))
            .map_err(|err| CheckpointError::Backend(err.to_string()))?;
        let audit = decode_audit_bin(&bytes)?;
        check_art_key(*checkpoint_id, audit.checkpoint_id())?;
        Ok(audit)
    }

    fn write_link_bin(&self, link: &CheckpointLink) -> Result<(), CheckpointError> {
        let bytes = encode_link_bin(link)?;
        Self::save_bin(&self.link_path(&link.checkpoint_id()), &bytes)
    }

    fn check_link_evidence(&self, link: &CheckpointLink) -> Result<(), CheckpointError> {
        let snapshot = PrepFsStore::new(&self.root)
            .load_snapshot(&link.prep_snapshot_id())
            .map_err(|_| CheckpointError::LinkMix)?;
        let exec = self
            .load_exec_input(&link.exec_input_id())
            .map_err(|_| CheckpointError::ReplayMix)?;
        check_link_ids(link.prep_snapshot_id(), link, &exec)?;
        check_exec_root(&snapshot, &exec)?;
        Ok(())
    }

    fn check_link_ready(&self, link: &CheckpointLink) -> Result<(), CheckpointError> {
        self.check_link_art(link)?;
        self.check_link_uniq(link)?;
        self.check_link_evidence(link)
    }

    fn load_link_validated(
        &self,
        checkpoint_id: &CheckpointId,
    ) -> Result<CheckpointLink, CheckpointError> {
        let bytes = read_file(self.link_path(checkpoint_id))
            .map_err(|err| CheckpointError::Backend(err.to_string()))?;
        let link = decode_link_bin(&bytes)?;
        check_art_key(*checkpoint_id, link.checkpoint_id())?;
        let artifact = self.load_persisted_artifact(checkpoint_id)?;
        Self::check_link_stmt(&link, &artifact)?;
        self.check_link_evidence(&link)?;
        Ok(link)
    }

    pub fn export_noncanonical_final_bundle(
        &mut self,
        artifact: &CheckpointArtifact,
        link: &CheckpointLink,
        audit: &CheckpointAudit,
    ) -> Result<CheckpointId, CheckpointError> {
        self.reject_canonical_final_lane()?;
        let checkpoint_id = derive_checkpoint_id(artifact)?;
        check_art_key(link.checkpoint_id(), checkpoint_id)?;
        check_art_key(audit.checkpoint_id(), checkpoint_id)?;
        Self::check_link_stmt(link, artifact)?;
        self.check_link_uniq(link)?;
        self.check_link_evidence(link)?;
        self.persist_artifact_bin(artifact)?;
        self.write_link_bin(link)?;
        self.persist_audit_bin(audit)?;
        self.persist_final_lane(CheckpointFinalLane::NoncanonicalExport)?;
        Ok(checkpoint_id)
    }

    pub fn load_noncanonical_artifact(
        &self,
        checkpoint_id: &CheckpointId,
    ) -> Result<CheckpointArtifact, CheckpointError> {
        self.require_noncanonical_final_lane()?;
        self.load_persisted_artifact(checkpoint_id)
    }

    pub fn load_noncanonical_link(
        &self,
        checkpoint_id: &CheckpointId,
    ) -> Result<CheckpointLink, CheckpointError> {
        self.require_noncanonical_final_lane()?;
        self.load_link_validated(checkpoint_id)
    }

    pub fn load_noncanonical_audit(
        &self,
        checkpoint_id: &CheckpointId,
    ) -> Result<CheckpointAudit, CheckpointError> {
        self.require_noncanonical_final_lane()?;
        self.load_persisted_audit(checkpoint_id)
    }
}

impl CheckpointStore for CheckpointFsStore {
    fn save_draft(
        &mut self,
        draft: &CheckpointDraft,
    ) -> Result<CheckpointDraftId, CheckpointError> {
        let draft_id = derive_draft_id(draft)?;
        let bytes = encode_draft_bin(draft)?;
        Self::save_bin(&self.draft_path(&draft_id), &bytes)?;
        Ok(draft_id)
    }

    fn load_draft(&self, draft_id: &CheckpointDraftId) -> Result<CheckpointDraft, CheckpointError> {
        let bytes = read_file(self.draft_path(draft_id))
            .map_err(|err| CheckpointError::Backend(err.to_string()))?;
        let draft = load_draft(&bytes)?;
        let got = derive_draft_id(&draft)?;
        check_draft_key(*draft_id, got)?;
        Ok(draft)
    }

    fn load_artifact(
        &self,
        checkpoint_id: &CheckpointId,
    ) -> Result<CheckpointArtifact, CheckpointError> {
        self.reject_noncanonical_final_lane()?;
        self.load_persisted_artifact(checkpoint_id)
    }

    fn seal_artifact(
        &mut self,
        draft: &CheckpointDraft,
        proof: CheckpointProof,
        snap_id: PrepSnapshotId,
        exec_id: CheckpointExecInputId,
    ) -> Result<CheckpointLink, CheckpointError> {
        self.reject_noncanonical_final_lane()?;
        let stmt = proof.statement();
        if stmt.prep_snapshot_id() != snap_id || stmt.exec_input_id() != exec_id {
            return Err(CheckpointError::LinkMix);
        }
        let snapshot = PrepFsStore::new(&self.root)
            .load_snapshot(&snap_id)
            .map_err(|_| CheckpointError::LinkMix)?;
        let exec = self
            .load_exec_input(&exec_id)
            .map_err(|_| CheckpointError::ReplayMix)?;
        check_exec_replay(snap_id, exec_id, &exec)?;
        check_exec_root(&snapshot, &exec)?;
        let artifact = draft.finalize(proof)?;
        let checkpoint_id = derive_checkpoint_id(&artifact)?;
        let link = CheckpointLink::new(
            super::link::CheckpointLinkVersion::CURRENT,
            checkpoint_id,
            snap_id,
            exec_id,
        )?;
        Self::check_link_stmt(&link, &artifact)?;
        self.check_link_uniq(&link)?;
        self.check_link_evidence(&link)?;
        self.persist_artifact_bin(&artifact)?;
        self.write_link_bin(&link)?;
        self.persist_final_lane(CheckpointFinalLane::CanonicalSeal)?;
        Ok(link)
    }

    fn save_link(&mut self, link: &CheckpointLink) -> Result<(), CheckpointError> {
        self.reject_noncanonical_final_lane()?;
        self.check_link_ready(link)?;
        self.write_link_bin(link)
    }

    fn load_link(&self, checkpoint_id: &CheckpointId) -> Result<CheckpointLink, CheckpointError> {
        self.reject_noncanonical_final_lane()?;
        self.load_link_validated(checkpoint_id)
    }

    fn save_exec_input(
        &mut self,
        exec: &CheckpointExecInput,
    ) -> Result<CheckpointExecInputId, CheckpointError> {
        let bytes = encode_exec_bin(exec)?;
        let exec_id = derive_exec_id(&bytes);
        Self::save_bin(&self.exec_path(&exec_id), &bytes)?;
        Ok(exec_id)
    }

    fn load_exec_input(
        &self,
        exec_id: &CheckpointExecInputId,
    ) -> Result<CheckpointExecInput, CheckpointError> {
        let bytes = read_file(self.exec_path(exec_id))
            .map_err(|err| CheckpointError::Backend(err.to_string()))?;
        let exec = decode_exec_bin(&bytes)?;
        let canon = encode_exec_bin(&exec)?;
        let got = derive_exec_id(&canon);
        check_exec_key(*exec_id, got)?;
        Ok(exec)
    }

    fn save_audit(&mut self, audit: &CheckpointAudit) -> Result<(), CheckpointError> {
        self.reject_noncanonical_final_lane()?;
        self.load_persisted_artifact(&audit.checkpoint_id())?;
        self.persist_audit_bin(audit)
    }

    fn load_audit(&self, checkpoint_id: &CheckpointId) -> Result<CheckpointAudit, CheckpointError> {
        self.reject_noncanonical_final_lane()?;
        self.load_persisted_audit(checkpoint_id)
    }
}
