//! Phase 7 checkpoint state-update interface.

use std::collections::BTreeSet;

use z00z_core::assets::registry::AssetId;
use z00z_storage::checkpoint::{CheckpointDraft, CheckpointExecInput, CheckpointLink};
use z00z_storage::settlement::{CheckRoot, StoreItem, TerminalId, TerminalLeaf};
use z00z_storage::snapshot::{PrepReplayEntry, PrepSnapshot, PrepSnapshotId};

pub use super::state_checkpoint::{Checkpoint, CheckpointPubIn, CreatedEnt, SpentEnt};
pub use super::state_errors::{SpentIndexError, StateError, TxProofError};
pub use super::state_resolved_input::ResolvedInput;
pub use super::state_traits::{
    InputResolver, MemberIndex, SettlementState, SpentIndex, TxPkgSum, TxProofVerifier,
};
use super::state_witness::proof_root;
pub use super::state_witness::MemberWit;
use super::tx_verifier::TxInputWire;

/// Build one canonical checkpoint draft from validated snapshot replay and execution input.
pub fn build_cp_draft(
    height: u64,
    snap_id: PrepSnapshotId,
    snapshot: &PrepSnapshot,
    replay: &[PrepReplayEntry],
    link: &CheckpointLink,
    exec: &CheckpointExecInput,
    proof_chk: &impl TxProofVerifier,
    spent_idx: &impl SpentIndex,
) -> Result<CheckpointDraft, StateError> {
    fn map_tx_err(err: z00z_storage::checkpoint::TxProofError) -> TxProofError {
        match err {
            z00z_storage::checkpoint::TxProofError::Invalid => TxProofError::Invalid,
            z00z_storage::checkpoint::TxProofError::Version => TxProofError::Version,
        }
    }

    fn map_spent_err(err: z00z_storage::checkpoint::SpentIndexError) -> SpentIndexError {
        match err {
            z00z_storage::checkpoint::SpentIndexError::Lookup => SpentIndexError::Lookup,
        }
    }

    fn map_state_err(err: z00z_storage::checkpoint::StateError) -> StateError {
        match err {
            z00z_storage::checkpoint::StateError::EmptyBatch => StateError::EmptyBatch,
            z00z_storage::checkpoint::StateError::PrevRoot => StateError::PrevRoot,
            z00z_storage::checkpoint::StateError::EmptyInputs => StateError::EmptyInputs,
            z00z_storage::checkpoint::StateError::EmptyOutputs => StateError::EmptyOutputs,
            z00z_storage::checkpoint::StateError::DupInput => StateError::DupInput,
            z00z_storage::checkpoint::StateError::DupOut => StateError::DupOut,
            z00z_storage::checkpoint::StateError::MissingInput => StateError::MissingInput,
            z00z_storage::checkpoint::StateError::BadInputRef => StateError::BadInputRef,
            z00z_storage::checkpoint::StateError::LeafMatch => StateError::LeafMatch,
            z00z_storage::checkpoint::StateError::BadMember => StateError::BadMember,
            z00z_storage::checkpoint::StateError::BadResolve => StateError::BadResolve,
            z00z_storage::checkpoint::StateError::SpentAfter => StateError::SpentAfter,
            z00z_storage::checkpoint::StateError::SpentBatch => StateError::SpentBatch,
            z00z_storage::checkpoint::StateError::State(err) => StateError::State(err),
            z00z_storage::checkpoint::StateError::TxProof(err) => {
                StateError::TxProof(map_tx_err(err))
            }
            z00z_storage::checkpoint::StateError::SpentIndex(err) => {
                StateError::SpentIndex(map_spent_err(err))
            }
        }
    }

    fn map_resolved(
        item: &z00z_storage::checkpoint::ResolvedInput,
    ) -> Result<ResolvedInput, TxProofError> {
        let wit = MemberWit::new(
            item.member_wit().proof().to_vec(),
            item.member_wit().proof_item().clone(),
        )
        .map_err(|_| TxProofError::Invalid)?;
        ResolvedInput::new(item.path(), item.leaf().clone(), wit).map_err(|_| TxProofError::Invalid)
    }

    fn map_tx(tx: &z00z_storage::checkpoint::TxPkgSum) -> Result<TxPkgSum, TxProofError> {
        Ok(TxPkgSum {
            prev_root: tx.prev_root,
            resolved_inputs: tx
                .resolved_inputs
                .iter()
                .map(map_resolved)
                .collect::<Result<Vec<_>, _>>()?,
            outputs: tx.outputs.clone(),
            tx_proof: tx.tx_proof.clone(),
        })
    }

    struct ProofWrap<'a, T>(&'a T);

    impl<T: TxProofVerifier> z00z_storage::checkpoint::TxProofVerifier for ProofWrap<'_, T> {
        fn verify_tx(
            &self,
            tx: &z00z_storage::checkpoint::TxPkgSum,
        ) -> Result<(), z00z_storage::checkpoint::TxProofError> {
            let tx = map_tx(tx).map_err(|err| match err {
                TxProofError::Invalid => z00z_storage::checkpoint::TxProofError::Invalid,
                TxProofError::Version => z00z_storage::checkpoint::TxProofError::Version,
            })?;
            self.0.verify_tx(&tx).map_err(|err| match err {
                TxProofError::Invalid => z00z_storage::checkpoint::TxProofError::Invalid,
                TxProofError::Version => z00z_storage::checkpoint::TxProofError::Version,
            })
        }
    }

    struct SpentWrap<'a, T>(&'a T);

    impl<T: SpentIndex> z00z_storage::checkpoint::SpentIndex for SpentWrap<'_, T> {
        fn is_spent(
            &self,
            prev: CheckRoot,
            curr: CheckRoot,
            id: &TerminalId,
        ) -> Result<bool, z00z_storage::checkpoint::SpentIndexError> {
            self.0.is_spent(prev, curr, id).map_err(|err| match err {
                SpentIndexError::Lookup => z00z_storage::checkpoint::SpentIndexError::Lookup,
            })
        }
    }

    let proof_wrap = ProofWrap(proof_chk);
    let spent_wrap = SpentWrap(spent_idx);
    z00z_storage::checkpoint::build_cp_draft(
        height,
        snap_id,
        snapshot,
        replay,
        link,
        exec,
        &proof_wrap,
        &spent_wrap,
    )
    .map_err(map_state_err)
}

fn parse_terminal_id_hex(input: &TxInputWire) -> Result<TerminalId, StateError> {
    let raw = hex::decode(&input.asset_id_hex).map_err(|_| StateError::BadInputRef)?;
    let raw: AssetId = raw.try_into().map_err(|_| StateError::BadInputRef)?;
    Ok(TerminalId::new(raw))
}

fn resolve_inputs(
    prev_root: CheckRoot,
    inputs: &[TxInputWire],
    resolver: &impl InputResolver,
) -> Result<Vec<ResolvedInput>, StateError> {
    if inputs.is_empty() {
        return Err(StateError::EmptyInputs);
    }

    let mut seen = BTreeSet::new();
    let mut out = Vec::with_capacity(inputs.len());
    let want_root = proof_root(prev_root);
    for input in inputs {
        let terminal_id = parse_terminal_id_hex(input)?;
        if !seen.insert(terminal_id) {
            return Err(StateError::DupInput);
        }

        let resolved = resolver.resolve(prev_root, terminal_id, input.serial_id)?;
        if resolved.terminal_id() != terminal_id || resolved.serial_id() != input.serial_id {
            return Err(StateError::LeafMatch);
        }
        resolved
            .member_wit()
            .check(want_root, &resolved.path(), resolved.leaf())?;

        out.push(resolved);
    }

    Ok(out)
}

/// Prepare a checkpoint summary from public input refs plus pre-state data.
///
/// This path keeps `TxInputWire` compact and reference-only. It loads the full
/// public leaf bytes from state, validates `serial_id` against the resolved
/// leaf, and requires a membership witness under `prev_root` before the batch
/// summary is handed to checkpoint execution. The provided state view must
/// already be positioned at the same `prev_root`.
pub fn prepare_tx_sum(
    prev_root: CheckRoot,
    resolver: &impl InputResolver,
    inputs: &[TxInputWire],
    outputs: &[TerminalLeaf],
    tx_proof: &[u8],
) -> Result<TxPkgSum, StateError> {
    let resolved_inputs = resolve_inputs(prev_root, inputs, resolver)?;
    Ok(TxPkgSum {
        prev_root,
        resolved_inputs,
        outputs: outputs.to_vec(),
        tx_proof: tx_proof.to_vec(),
    })
}

/// Apply tx batch and return checkpoint material.
///
/// This helper is a deterministic state-application path for storage integration,
/// simulator wiring, tests, and debug comparison. Consensus validation must come
/// from proof verification over `CheckpointPubIn` and checkpoint proof bytes,
/// rather than from trusting local re-apply as the final oracle.
pub fn apply_batch_checkpoint(
    height: u64,
    state: &mut impl SettlementState,
    txs: &[TxPkgSum],
    proof_chk: &impl TxProofVerifier,
    spent_idx: &impl SpentIndex,
) -> Result<Checkpoint, StateError> {
    if txs.is_empty() {
        return Err(StateError::EmptyBatch);
    }

    let prev_root = state.root();
    let mut spent_delta = Vec::new();
    let mut created_delta = Vec::new();
    let mut out_seen = BTreeSet::new();
    let mut spent_seen = BTreeSet::new();

    for tx in txs {
        if tx.prev_root != prev_root {
            return Err(StateError::PrevRoot);
        }
        if tx.resolved_inputs.is_empty() {
            return Err(StateError::EmptyInputs);
        }
        if tx.outputs.is_empty() {
            return Err(StateError::EmptyOutputs);
        }

        let mut seen = BTreeSet::new();
        for resolved in &tx.resolved_inputs {
            if !seen.insert(resolved.terminal_id()) {
                return Err(StateError::DupInput);
            }
        }

        for resolved in &tx.resolved_inputs {
            let storage_leaf = resolved.leaf().clone();
            if StoreItem::new(resolved.path(), storage_leaf.clone()).is_err()
                || resolved.member_wit().proof().is_empty()
                || resolved.member_wit().proof_root() != proof_root(tx.prev_root)
                || resolved.member_wit().proof_item().path() != resolved.path()
                || resolved
                    .member_wit()
                    .proof_item()
                    .terminal_leaf()
                    .map_err(|_| StateError::BadResolve)?
                    != &storage_leaf
            {
                return Err(StateError::BadResolve);
            }
        }

        proof_chk.verify_tx(tx)?;

        for resolved in &tx.resolved_inputs {
            let id = resolved.terminal_id();
            if !spent_seen.insert(id) {
                return Err(StateError::SpentBatch);
            }
            let state_leaf = state.get_leaf(&id)?.ok_or(StateError::MissingInput)?;
            if state_leaf != *resolved.leaf() {
                return Err(StateError::BadResolve);
            }
            if spent_idx.is_spent(tx.prev_root, prev_root, &id)? {
                return Err(StateError::SpentAfter);
            }

            state.del_leaf(&id)?;
            spent_delta.push(SpentEnt { terminal_id: id });
        }

        for out in &tx.outputs {
            let out_id = out.terminal_id();
            if !out_seen.insert(out_id.into_bytes()) {
                return Err(StateError::DupOut);
            }
            if state.get_leaf(&out_id)?.is_some() {
                return Err(StateError::DupOut);
            }
            let h = state.leaf_hash(out)?;
            state.put_leaf(out.clone())?;
            created_delta.push(CreatedEnt {
                terminal_id: out_id,
                leaf_hash: h,
            });
        }
    }

    let new_root = state.root();
    Ok(Checkpoint {
        height,
        prev_root,
        new_root,
        spent_delta,
        created_delta,
        cp_proof: Vec::new(),
    })
}

#[cfg(test)]
#[path = "test_state_update.rs"]
mod tests;
