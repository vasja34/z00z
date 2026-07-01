use crate::settlement::{CheckRoot, SnapItem};

/// Canonical snapshot schema version.
///
/// # Examples
///
/// ```
/// use z00z_storage::snapshot::PrepSnapshotVersion;
///
/// assert_eq!(PrepSnapshotVersion::CURRENT.as_u8(), 1);
/// ```
#[derive(
    Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, serde::Serialize, serde::Deserialize,
)]
pub struct PrepSnapshotVersion(u8);

impl PrepSnapshotVersion {
    pub const CURRENT: Self = Self(1);

    /// Build one schema-version tag from its stable numeric value.
    #[must_use]
    pub const fn new(value: u8) -> Self {
        Self(value)
    }

    /// Return the stable numeric tag for this schema version.
    #[must_use]
    pub const fn as_u8(self) -> u8 {
        self.0
    }
}

/// External content-addressed identifier for one canonical snapshot artifact.
///
/// # Examples
///
/// ```
/// use z00z_storage::snapshot::PrepSnapshotId;
///
/// let snap_id = PrepSnapshotId::new([7u8; 32]);
/// assert_eq!(snap_id.as_bytes(), &[7u8; 32]);
/// ```
#[derive(
    Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, serde::Serialize, serde::Deserialize,
)]
pub struct PrepSnapshotId([u8; 32]);

impl PrepSnapshotId {
    /// Build one external snapshot identifier from canonical bytes.
    #[must_use]
    pub const fn new(bytes: [u8; 32]) -> Self {
        Self(bytes)
    }

    /// Consume the identifier and return its raw bytes.
    #[must_use]
    pub const fn into_bytes(self) -> [u8; 32] {
        self.0
    }

    /// Borrow the raw identifier bytes.
    #[must_use]
    pub const fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }
}

impl From<[u8; 32]> for PrepSnapshotId {
    fn from(bytes: [u8; 32]) -> Self {
        Self::new(bytes)
    }
}

/// Canonical pre-state snapshot artifact.
///
/// # Examples
///
/// ```
/// use z00z_storage::{
///     snapshot::{PrepSnapshot, PrepSnapshotVersion},
///     settlement::{CheckRoot, SnapItem},
/// };
///
/// let snap = PrepSnapshot::new(PrepSnapshotVersion::CURRENT, CheckRoot::new([0u8; 32]), Vec::<SnapItem>::new());
/// assert_eq!(snap.version, PrepSnapshotVersion::CURRENT);
/// ```
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct PrepSnapshot {
    pub version: PrepSnapshotVersion,
    pub prev_root: CheckRoot,
    pub entries: Vec<SnapItem>,
}

impl PrepSnapshot {
    /// Build one canonical snapshot shell from version, root, and ordered entries.
    #[must_use]
    pub fn new(version: PrepSnapshotVersion, prev_root: CheckRoot, entries: Vec<SnapItem>) -> Self {
        Self {
            version,
            prev_root,
            entries,
        }
    }
}
