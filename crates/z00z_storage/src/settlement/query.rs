use super::{DefinitionId, RightClass, SerialId, SettlementPath, StoreItem, TerminalId};

/// Typed exact lookup request for storage-owned generalized settlement search helpers.
#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum SettlementLookup {
    Path(SettlementPath),
    Terminal(TerminalId),
}

/// Scoped canonical listing surface for generalized settlement callers.
#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum SettlementScope {
    All,
    Def(DefinitionId),
    Ser(DefinitionId, SerialId),
    RightClass(RightClass),
}

/// Canonical-order pagination token for generalized settlement callers.
#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct SettlementPageTok(SettlementPath);

impl SettlementPageTok {
    #[must_use]
    pub fn new(path: impl Into<SettlementPath>) -> Self {
        Self(path.into())
    }

    #[must_use]
    pub const fn path(self) -> SettlementPath {
        self.0
    }
}

/// Typed deterministic list request over generalized settlement path order.
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct SettlementListReq {
    scope: SettlementScope,
    start: Option<SettlementPath>,
    end: Option<SettlementPath>,
    after: Option<SettlementPageTok>,
    limit: usize,
}

impl SettlementListReq {
    #[must_use]
    pub const fn new(
        scope: SettlementScope,
        start: Option<SettlementPath>,
        end: Option<SettlementPath>,
        after: Option<SettlementPageTok>,
        limit: usize,
    ) -> Self {
        Self {
            scope,
            start,
            end,
            after,
            limit,
        }
    }

    #[must_use]
    pub const fn all(limit: usize) -> Self {
        Self::new(SettlementScope::All, None, None, None, limit)
    }

    #[must_use]
    pub const fn for_def(definition_id: DefinitionId, limit: usize) -> Self {
        Self::new(SettlementScope::Def(definition_id), None, None, None, limit)
    }

    #[must_use]
    pub const fn for_ser(definition_id: DefinitionId, serial_id: SerialId, limit: usize) -> Self {
        Self::new(
            SettlementScope::Ser(definition_id, serial_id),
            None,
            None,
            None,
            limit,
        )
    }

    #[must_use]
    pub const fn for_right_class(right_class: RightClass, limit: usize) -> Self {
        Self::new(
            SettlementScope::RightClass(right_class),
            None,
            None,
            None,
            limit,
        )
    }

    #[must_use]
    pub const fn with_range(
        mut self,
        start: Option<SettlementPath>,
        end: Option<SettlementPath>,
    ) -> Self {
        self.start = start;
        self.end = end;
        self
    }

    #[must_use]
    pub const fn with_after(mut self, after: Option<SettlementPageTok>) -> Self {
        self.after = after;
        self
    }

    #[must_use]
    pub const fn scope(&self) -> SettlementScope {
        self.scope
    }

    #[must_use]
    pub const fn start(&self) -> Option<SettlementPath> {
        self.start
    }

    #[must_use]
    pub const fn end(&self) -> Option<SettlementPath> {
        self.end
    }

    #[must_use]
    pub const fn after(&self) -> Option<SettlementPageTok> {
        self.after
    }

    #[must_use]
    pub const fn limit(&self) -> usize {
        self.limit
    }
}

/// Deterministic page of generalized settlement items.
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct SettlementPage {
    items: Vec<StoreItem>,
    next: Option<SettlementPageTok>,
}

impl SettlementPage {
    #[must_use]
    pub fn new(items: Vec<StoreItem>, next: Option<SettlementPageTok>) -> Self {
        Self { items, next }
    }

    #[must_use]
    pub fn items(&self) -> &[StoreItem] {
        &self.items
    }

    #[must_use]
    pub const fn next(&self) -> Option<SettlementPageTok> {
        self.next
    }
}
