use z00z_storage::settlement::{CheckRoot, TerminalId, TerminalLeaf};

use super::{
    state_errors::{SpentIndexError, StateError, TxProofError},
    state_resolved_input::ResolvedInput,
    state_witness::MemberWit,
};

/// Minimal tx package summary for state update.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TxPkgSum {
    /// Root declared by tx package.
    pub prev_root: CheckRoot,
    /// Fully resolved pre-state inputs with canonical path semantics.
    pub resolved_inputs: Vec<ResolvedInput>,
    /// Output leaves.
    pub outputs: Vec<TerminalLeaf>,
    /// Opaque tx proof bytes.
    pub tx_proof: Vec<u8>,
}

impl TxPkgSum {
    /// Return canonical input terminal ids derived from path-bound resolved inputs.
    #[must_use]
    pub fn input_terminal_ids(&self) -> Vec<TerminalId> {
        self.resolved_inputs
            .iter()
            .map(ResolvedInput::terminal_id)
            .collect()
    }
}

/// Settlement state hook used by checkpoint apply.
///
/// Invariants:
/// - `get_leaf` reads from the current mutable state view.
/// - `del_leaf` removes exactly one leaf for the given id.
/// - `put_leaf` inserts one output leaf.
/// - `leaf_hash` is stable for identical leaf bytes.
pub trait SettlementState {
    /// Return current state root.
    fn root(&self) -> CheckRoot;

    /// Read input leaf by id.
    fn get_leaf(&self, id: &TerminalId) -> Result<Option<TerminalLeaf>, StateError>;

    /// Remove input leaf by id.
    fn del_leaf(&mut self, id: &TerminalId) -> Result<(), StateError>;

    /// Insert output leaf.
    fn put_leaf(&mut self, leaf: TerminalLeaf) -> Result<(), StateError>;

    /// Hash leaf in canonical state format.
    fn leaf_hash(&self, leaf: &TerminalLeaf) -> Result<[u8; 32], StateError>;
}

/// Tx proof verifier hook.
pub trait TxProofVerifier {
    /// Verify tx proof validity for the provided package summary.
    fn verify_tx(&self, tx: &TxPkgSum) -> Result<(), TxProofError>;
}

/// Spent-delta interval index hook.
pub trait SpentIndex {
    /// Return true when `id` is spent in `(prev, curr]` interval.
    fn is_spent(
        &self,
        prev: CheckRoot,
        curr: CheckRoot,
        id: &TerminalId,
    ) -> Result<bool, SpentIndexError>;
}

/// Membership witness source used during checkpoint preparation.
pub trait MemberIndex {
    /// Return canonical membership witness bytes for `id` under `prev_root`.
    fn get_wit(
        &self,
        prev_root: CheckRoot,
        id: &TerminalId,
    ) -> Result<Option<MemberWit>, StateError>;
}

/// Dedicated pre-state resolver for one compact tx input.
pub trait InputResolver {
    /// Resolve one compact input into a path-bound record under `prev_root`.
    fn resolve(
        &self,
        prev_root: CheckRoot,
        terminal_id: TerminalId,
        serial_id: u32,
    ) -> Result<ResolvedInput, StateError>;
}
