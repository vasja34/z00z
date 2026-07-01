use serde::{Deserialize, Serialize};
use z00z_storage::settlement::{
    chk_blob_settlement_inclusion, CheckRoot, ProofItem, SettlementPath, SettlementStateRoot,
    TerminalLeaf,
};

use super::state_errors::StateError;

/// Convert a checkpoint root into the storage proof root type.
#[must_use]
pub(crate) fn proof_root(prev_root: CheckRoot) -> SettlementStateRoot {
    SettlementStateRoot::settlement_v1(prev_root.into_bytes())
}

/// Canonical membership witness captured during pre-state resolution.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct MemberWit {
    /// Canonical witness bytes for membership under `prev_root`.
    pub(crate) proof: Vec<u8>,
    pub(crate) proof_item: ProofItem,
}

impl MemberWit {
    /// Build one thin wallet wrapper over canonical storage proof bytes.
    pub fn new(proof: Vec<u8>, proof_item: ProofItem) -> Result<Self, StateError> {
        let wit = Self { proof, proof_item };
        let path = wit.proof_item.path();
        let leaf = wit
            .proof_item
            .terminal_leaf()
            .map_err(|_| StateError::BadMember)?
            .clone();
        wit.check(wit.proof_root(), &path, &leaf)?;
        Ok(wit)
    }

    /// Return canonical proof bytes.
    #[must_use]
    pub fn proof(&self) -> &[u8] {
        &self.proof
    }

    /// Return the typed storage proof context bound to this witness.
    #[must_use]
    pub fn proof_item(&self) -> &ProofItem {
        &self.proof_item
    }

    /// Return the semantic settlement root embedded in the proof context.
    #[must_use]
    pub fn proof_root(&self) -> SettlementStateRoot {
        self.proof_item.settlement_root()
    }

    pub(crate) fn check(
        &self,
        root: SettlementStateRoot,
        path: &SettlementPath,
        leaf: &TerminalLeaf,
    ) -> Result<(), StateError> {
        let item = self.proof_item();
        chk_blob_settlement_inclusion(
            &self.proof,
            root,
            path,
            item.def_leaf(),
            item.ser_leaf(),
            leaf,
        )
        .map(|_| ())
        .map_err(|_| StateError::BadMember)
    }
}
