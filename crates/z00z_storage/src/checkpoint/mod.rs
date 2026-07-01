//! Canonical checkpoint storage surface.
//!
//! Consensus-facing callers should use the top-level types re-exported from this module:
//! drafts, final artifacts, links, typed execution inputs, ids, and the storage facade.
//! Replay and audit-only data stays under the narrower [`audit`] submodule so wrapper-local
//! fields do not leak into the main checkpoint contract by default.

mod artifact_final;
mod artifact_proof_draft;
mod artifact_stmt;
mod artifact_types;
pub mod audit;
mod build;
mod build_prepare;
mod build_state;
mod codec;
mod exec_input;
mod ids;
mod link;
mod store;
mod store_fs;
#[cfg(test)]
mod test_checkpoint;
#[cfg(test)]
mod test_store;

pub use self::{
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
        decode_art_bin, decode_art_json, decode_draft_bin, decode_draft_json, decode_exec_bin,
        decode_exec_json, decode_link_bin, decode_link_json, encode_art_bin, encode_art_json,
        encode_draft_bin, encode_draft_json, encode_exec_bin, encode_exec_json, encode_link_bin,
        encode_link_json,
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
    store::{
        check_art_key, check_draft_key, check_exec_key, check_exec_root, check_link_ids,
        load_artifact, load_draft, CheckpointFsStore, CheckpointStore,
    },
};
