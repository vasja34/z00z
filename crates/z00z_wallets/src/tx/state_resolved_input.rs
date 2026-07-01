use z00z_storage::settlement::{SettlementPath, StoreItem, TerminalId, TerminalLeaf};

use super::{state_errors::StateError, state_witness::MemberWit};

/// Path-bound pre-state input captured before checkpoint apply.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ResolvedInput {
    pub(crate) path: SettlementPath,
    pub(crate) leaf: TerminalLeaf,
    pub(crate) member_wit: MemberWit,
}

impl ResolvedInput {
    /// Build one path-bound resolved input.
    pub fn new(
        path: SettlementPath,
        leaf: TerminalLeaf,
        member_wit: MemberWit,
    ) -> Result<Self, StateError> {
        StoreItem::new(path, leaf.clone()).map_err(|_| StateError::LeafMatch)?;
        member_wit.check(member_wit.proof_root(), &path, &leaf)?;
        Ok(Self {
            path,
            leaf,
            member_wit,
        })
    }

    /// Return the canonical path bound to this resolved input.
    #[must_use]
    pub const fn path(&self) -> SettlementPath {
        self.path
    }

    /// Return the resolved pre-state leaf for this input.
    #[must_use]
    pub fn leaf(&self) -> &TerminalLeaf {
        &self.leaf
    }

    /// Return the membership witness paired with this input path.
    #[must_use]
    pub fn member_wit(&self) -> &MemberWit {
        &self.member_wit
    }

    /// Return the canonical terminal id carried by the retained path.
    #[must_use]
    pub const fn terminal_id(&self) -> TerminalId {
        self.path.terminal_id()
    }

    /// Return the canonical serial id carried by the retained path.
    #[must_use]
    pub const fn serial_id(&self) -> u32 {
        self.path.serial_id.get()
    }
}
