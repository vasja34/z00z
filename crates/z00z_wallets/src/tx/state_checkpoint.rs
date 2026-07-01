use serde::{Deserialize, Serialize};
use z00z_storage::checkpoint::WalletDraft;
use z00z_storage::settlement::{CheckRoot, TerminalId};

/// Consumed leaf entry recorded in checkpoint delta.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SpentEnt {
    /// Consumed canonical terminal id.
    pub terminal_id: TerminalId,
}

/// Created leaf entry recorded in checkpoint delta.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct CreatedEnt {
    /// Created terminal id.
    pub terminal_id: TerminalId,
    /// State leaf hash.
    pub leaf_hash: [u8; 32],
}

/// Public checkpoint output.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Checkpoint {
    /// Checkpoint height.
    pub height: u64,
    /// Root before batch apply.
    pub prev_root: CheckRoot,
    /// Root after batch apply.
    pub new_root: CheckRoot,
    /// Spent terminal ids in apply order.
    pub spent_delta: Vec<SpentEnt>,
    /// Created leaves in apply order.
    pub created_delta: Vec<CreatedEnt>,
    /// Opaque checkpoint proof bytes when later storage-backed finalization
    /// attaches them. Local proofless wallet helpers may legitimately leave this
    /// empty and must not be treated as finalized checkpoint artifacts.
    pub cp_proof: Vec<u8>,
}

/// Public inputs consumed by validator-side checkpoint verification.
///
/// Validator-facing verification is real, and the remaining gap is final cryptographic closure, not total absence of validator-facing verification.
/// Consensus verification must treat this typed
/// record plus the backend-defined package-coupled checkpoint acceptance
/// contract as the source of truth, not standalone checkpoint-authorization
/// carriers. Local state re-apply helpers may compare against the same values
/// in tests and debug flows, but they are not the consensus oracle.
/// Package-coupled checkpoint integrity exists on the accepted finalize or load
/// path: proof-system typing, statement shape, exec identity, post-state root,
/// and the persisted snapshot or link tuple must stay aligned before
/// acceptance holds. Detached proof bytes remain non-authoritative
/// fallback inputs rather than the live closure story. This shipped boundary
/// still does not create a generic standalone proof backend or finished
/// trustless publish theorem.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct CheckpointPubIn {
    /// Root before batch apply.
    pub prev_root: CheckRoot,
    /// Root after batch apply.
    pub new_root: CheckRoot,
    /// Typed spent entries keyed by canonical terminal id only.
    pub spent_delta: Vec<SpentEnt>,
    /// Typed created entries keyed by terminal id and leaf hash.
    pub created_delta: Vec<CreatedEnt>,
}

impl Checkpoint {
    /// Return typed public inputs for validator-side checkpoint verification.
    pub fn as_pub_in(&self) -> CheckpointPubIn {
        CheckpointPubIn {
            prev_root: self.prev_root,
            new_root: self.new_root,
            spent_delta: self.spent_delta.clone(),
            created_delta: self.created_delta.clone(),
        }
    }
}

impl WalletDraft for Checkpoint {
    fn draft_height(&self) -> u64 {
        self.height
    }

    fn draft_prev_root(&self) -> CheckRoot {
        self.prev_root
    }

    fn draft_new_root(&self) -> CheckRoot {
        self.new_root
    }

    fn draft_spent(&self) -> Vec<z00z_storage::checkpoint::SpentEnt> {
        self.spent_delta
            .iter()
            .map(|item| z00z_storage::checkpoint::SpentEnt::new(item.terminal_id))
            .collect()
    }

    fn draft_created(&self) -> Vec<z00z_storage::checkpoint::CreatedEnt> {
        self.created_delta
            .iter()
            .map(|item| z00z_storage::checkpoint::CreatedEnt::new(item.terminal_id, item.leaf_hash))
            .collect()
    }
}
