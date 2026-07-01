//! Audit-only checkpoint records.
//!
//! This module is narrower than the main `checkpoint` surface on purpose: it carries replay and
//! wrapper-local fields such as `fragment_ids`, which must remain outside canonical checkpoint
//! artifact bytes.

use crate::CheckpointError;

pub use super::codec::{decode_audit_bin, decode_audit_json, encode_audit_bin, encode_audit_json};
use super::ids::CheckpointId;

/// Canonical checkpoint-audit schema version.
///
/// # Examples
///
/// ```
/// use z00z_storage::checkpoint::audit::CheckpointAuditVersion;
///
/// assert_eq!(CheckpointAuditVersion::CURRENT.as_u8(), 1);
/// ```
#[derive(
    Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, serde::Serialize, serde::Deserialize,
)]
pub struct CheckpointAuditVersion(u8);

impl CheckpointAuditVersion {
    pub const CURRENT: Self = Self(1);

    #[must_use]
    pub const fn new(value: u8) -> Self {
        Self(value)
    }

    #[must_use]
    pub const fn as_u8(self) -> u8 {
        self.0
    }
}

/// Storage-owned audit wrapper kept outside canonical artifact bytes.
///
/// # Examples
///
/// ```
/// use z00z_storage::checkpoint::{audit::{CheckpointAudit, CheckpointAuditVersion}, CheckpointId};
///
/// let audit = CheckpointAudit::new(
///     CheckpointAuditVersion::CURRENT,
///     CheckpointId::new([1u8; 32]),
///     vec![String::from("frag-1")],
/// )?;
/// assert_eq!(audit.fragment_ids(), &[String::from("frag-1")]);
/// # Ok::<(), z00z_storage::CheckpointError>(())
/// ```
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct CheckpointAudit {
    version: CheckpointAuditVersion,
    checkpoint_id: CheckpointId,
    fragment_ids: Vec<String>,
}

impl CheckpointAudit {
    pub fn new(
        version: CheckpointAuditVersion,
        checkpoint_id: CheckpointId,
        fragment_ids: Vec<String>,
    ) -> Result<Self, CheckpointError> {
        check_audit_ver(version)?;
        Ok(Self {
            version,
            checkpoint_id,
            fragment_ids,
        })
    }

    #[must_use]
    pub const fn version(&self) -> CheckpointAuditVersion {
        self.version
    }

    #[must_use]
    pub const fn checkpoint_id(&self) -> CheckpointId {
        self.checkpoint_id
    }

    #[must_use]
    pub fn fragment_ids(&self) -> &[String] {
        &self.fragment_ids
    }
}

pub(crate) fn check_audit_ver(version: CheckpointAuditVersion) -> Result<(), CheckpointError> {
    if version == CheckpointAuditVersion::CURRENT {
        return Ok(());
    }

    Err(CheckpointError::VersionMix)
}

#[cfg(test)]
mod tests {
    use super::{check_audit_ver, CheckpointAudit, CheckpointAuditVersion};
    use crate::{checkpoint::CheckpointId, CheckpointError};

    #[test]
    fn test_good_audit_builds() {
        let audit = CheckpointAudit::new(
            CheckpointAuditVersion::CURRENT,
            CheckpointId::new([1u8; 32]),
            vec![String::from("frag-1")],
        )
        .expect("audit");

        assert_eq!(audit.fragment_ids().len(), 1);
    }

    #[test]
    fn test_bad_audit_ver_rejects() {
        let err = check_audit_ver(CheckpointAuditVersion::new(9)).expect_err("bad audit version");

        assert!(matches!(err, CheckpointError::VersionMix));
    }
}
