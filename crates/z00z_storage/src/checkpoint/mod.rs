//! Canonical checkpoint storage surface.
//!
//! Consensus-facing callers should use the top-level types re-exported from this module:
//! drafts, final artifacts, links, typed execution inputs, ids, and the storage facade.
//! Replay and audit-only data stays under the narrower [`audit`] submodule so wrapper-local
//! fields do not leak into the main checkpoint contract by default.

mod archive_manifest;
mod archive_receipt;
mod artifact_final;
mod artifact_proof_draft;
mod artifact_stmt;
mod artifact_types;
pub mod audit;
mod build;
mod build_prepare;
mod build_state;
mod codec;
mod contract_config;
mod exec_input;
mod ids;
mod link;
mod pruning;
mod retrieval_audit;
mod state_snapshot;
mod store;
mod store_fs;
#[cfg(test)]
mod test_checkpoint;
#[cfg(test)]
mod test_store;

pub use self::{
    archive_manifest::{ArchiveManifestVersion, CheckpointArchiveManifestV1},
    archive_receipt::{ArchiveBackend, ArchiveProviderReceiptV1, ArchiveProviderReceiptVersion},
    artifact_final::CheckpointArtifact,
    artifact_proof_draft::{CheckpointDraft, CheckpointProof},
    artifact_stmt::{CheckpointStatement, CheckpointStmt, WalletDraft},
    artifact_types::{
        CheckpointProofSystem, CheckpointPubIn, CheckpointVersion, CreatedEnt, SpentEnt,
    },
    build::{
        apply_batch_checkpoint, build_cp_draft, InputResolver, MemberIndex, MemberWit,
        ResolvedInput, SettlementState, SpentIndex, SpentIndexError, StateError, TxPkgSum,
        TxProofError, TxProofVerifier,
    },
    build_prepare::prepare_tx_sum,
    codec::{
        decode_archive_manifest_bin, decode_archive_manifest_json, decode_archive_receipt_bin,
        decode_archive_receipt_json, decode_art_bin, decode_art_json, decode_draft_bin,
        decode_draft_json, decode_exec_bin, decode_exec_json, decode_link_bin, decode_link_json,
        decode_pruning_decision_bin, decode_pruning_decision_json, decode_retrieval_audit_bin,
        decode_retrieval_audit_json, decode_state_snapshot_bin, decode_state_snapshot_json,
        encode_archive_manifest_bin, encode_archive_manifest_json, encode_archive_receipt_bin,
        encode_archive_receipt_json, encode_art_bin, encode_art_json, encode_draft_bin,
        encode_draft_json, encode_exec_bin, encode_exec_json, encode_link_bin, encode_link_json,
        encode_pruning_decision_bin, encode_pruning_decision_json, encode_retrieval_audit_bin,
        encode_retrieval_audit_json, encode_state_snapshot_bin, encode_state_snapshot_json,
    },
    contract_config::{
        ArchiveRetentionCfg, CheckpointContractConfigV1, CheckpointContractLimits,
        CheckpointContractPaths, PruningCfg, SnapshotsCfg, CHECKPOINT_CONTRACT_CONFIG_PATH,
    },
    exec_input::{
        CheckpointExecInput, CheckpointExecOut, CheckpointExecTx, CheckpointExecVersion,
        CheckpointInRef,
    },
    ids::{
        derive_checkpoint_id, derive_draft_id, derive_exec_id, reject_draft_for_checkpoint_id,
        CheckpointDraftId, CheckpointExecInputId, CheckpointId,
    },
    link::{CheckpointLink, CheckpointLinkVersion},
    pruning::{PruningDecisionV1, PruningDecisionVersion, PruningNodeClass},
    retrieval_audit::{RetrievalAuditV1, RetrievalAuditVersion},
    state_snapshot::{StateSnapshotV1, StateSnapshotVersion},
    store::{
        check_art_key, check_draft_key, check_exec_key, check_exec_root, check_link_ids,
        load_artifact, load_draft, CheckpointFsStore, CheckpointStore,
    },
};
