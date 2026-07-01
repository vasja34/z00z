use std::collections::BTreeSet;

use super::build_state::proof_root;
use super::{
    build::InputResolver, build::ResolvedInput, build::StateError, build::TxPkgSum, CheckpointInRef,
};
use crate::settlement::{CheckRoot, TerminalLeaf};

fn resolve_inputs(
    prev_root: CheckRoot,
    inputs: &[CheckpointInRef],
    resolver: &impl InputResolver,
) -> Result<Vec<ResolvedInput>, StateError> {
    if inputs.is_empty() {
        return Err(StateError::EmptyInputs);
    }

    let mut seen = BTreeSet::new();
    let mut out = Vec::with_capacity(inputs.len());
    let want_root = proof_root(prev_root);
    for input in inputs {
        let terminal_id = input.terminal_id();
        if !seen.insert(terminal_id) {
            return Err(StateError::DupInput);
        }

        let resolved = resolver.resolve(prev_root, terminal_id, input.serial_id().get())?;
        if resolved.terminal_id() != terminal_id || resolved.serial_id() != input.serial_id().get()
        {
            return Err(StateError::LeafMatch);
        }
        resolved
            .member_wit()
            .check(want_root, &resolved.path(), resolved.leaf())?;

        out.push(resolved);
    }

    Ok(out)
}

/// Prepare a checkpoint summary from canonical execution refs plus pre-state data.
pub fn prepare_tx_sum(
    prev_root: CheckRoot,
    resolver: &impl InputResolver,
    inputs: &[CheckpointInRef],
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
