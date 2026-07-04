use z00z_crypto::expert::hash_domain;
use z00z_crypto::hash_zk::hash_zk;
use z00z_utils::codec::{BincodeCodec, Codec, JsonCodec};

use crate::CheckpointError;

hash_domain!(
    StorCheckpointRetrievalAuditDom,
    "z00z.storage.checkpoint.retrieval_audit",
    1
);

const RETRIEVAL_AUDIT_BIND_VER: u8 = 1;
const RETRIEVAL_AUDIT_BIND_LABEL: &str = "retrieval_audit_v1";

#[derive(
    Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, serde::Serialize, serde::Deserialize,
)]
pub struct RetrievalAuditVersion(u8);

impl RetrievalAuditVersion {
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

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct RetrievalAuditV1 {
    version: RetrievalAuditVersion,
    height: u64,
    interval_blocks: u64,
    archive_manifest_root: [u8; 32],
    requested_entries_root: [u8; 32],
    successful_receipts_root: [u8; 32],
    failed_receipts_root: [u8; 32],
    successful_replica_count: u32,
    passed: bool,
    #[serde(default)]
    retrieval_audit_bind_ver: u8,
    #[serde(default)]
    retrieval_audit_bind: [u8; 32],
}

impl RetrievalAuditV1 {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        version: RetrievalAuditVersion,
        height: u64,
        interval_blocks: u64,
        archive_manifest_root: [u8; 32],
        requested_entries_root: [u8; 32],
        successful_receipts_root: [u8; 32],
        failed_receipts_root: [u8; 32],
        successful_replica_count: u32,
        passed: bool,
    ) -> Result<Self, CheckpointError> {
        check_retrieval_audit_ver(version)?;
        if height == 0
            || interval_blocks == 0
            || !height.is_multiple_of(interval_blocks)
            || successful_replica_count < 3
            || !passed
            || is_zero_root(&archive_manifest_root)
            || is_zero_root(&requested_entries_root)
            || is_zero_root(&successful_receipts_root)
        {
            return Err(CheckpointError::ArchiveMix);
        }
        let retrieval_audit_bind = retrieval_audit_bind(
            height,
            interval_blocks,
            archive_manifest_root,
            requested_entries_root,
            successful_receipts_root,
            failed_receipts_root,
            successful_replica_count,
            passed,
        );
        Ok(Self {
            version,
            height,
            interval_blocks,
            archive_manifest_root,
            requested_entries_root,
            successful_receipts_root,
            failed_receipts_root,
            successful_replica_count,
            passed,
            retrieval_audit_bind_ver: RETRIEVAL_AUDIT_BIND_VER,
            retrieval_audit_bind,
        })
    }

    #[must_use]
    pub const fn version(&self) -> RetrievalAuditVersion {
        self.version
    }

    #[must_use]
    pub const fn height(&self) -> u64 {
        self.height
    }

    #[must_use]
    pub const fn interval_blocks(&self) -> u64 {
        self.interval_blocks
    }

    #[must_use]
    pub const fn archive_manifest_root(&self) -> [u8; 32] {
        self.archive_manifest_root
    }

    #[must_use]
    pub const fn successful_replica_count(&self) -> u32 {
        self.successful_replica_count
    }

    #[must_use]
    pub const fn passed(&self) -> bool {
        self.passed
    }

    fn check_bind(&self) -> Result<(), CheckpointError> {
        if self.retrieval_audit_bind_ver != RETRIEVAL_AUDIT_BIND_VER {
            return Err(CheckpointError::ArchiveMix);
        }
        if self.retrieval_audit_bind
            != retrieval_audit_bind(
                self.height,
                self.interval_blocks,
                self.archive_manifest_root,
                self.requested_entries_root,
                self.successful_receipts_root,
                self.failed_receipts_root,
                self.successful_replica_count,
                self.passed,
            )
        {
            return Err(CheckpointError::ArchiveMix);
        }
        Ok(())
    }
}

pub(crate) fn check_retrieval_audit_ver(
    version: RetrievalAuditVersion,
) -> Result<(), CheckpointError> {
    if version == RetrievalAuditVersion::CURRENT {
        return Ok(());
    }
    Err(CheckpointError::VersionMix)
}

pub(crate) fn encode_retrieval_audit_bin_checked(
    audit: &RetrievalAuditV1,
) -> Result<Vec<u8>, CheckpointError> {
    check_retrieval_audit_ver(audit.version())?;
    audit.check_bind()?;
    Ok(BincodeCodec.serialize(audit)?)
}

pub(crate) fn decode_retrieval_audit_bin_checked(
    bytes: &[u8],
) -> Result<RetrievalAuditV1, CheckpointError> {
    let audit: RetrievalAuditV1 = BincodeCodec.deserialize(bytes)?;
    check_retrieval_audit_ver(audit.version())?;
    audit.check_bind()?;
    Ok(audit)
}

pub(crate) fn encode_retrieval_audit_json_checked(
    audit: &RetrievalAuditV1,
) -> Result<Vec<u8>, CheckpointError> {
    check_retrieval_audit_ver(audit.version())?;
    audit.check_bind()?;
    Ok(JsonCodec.serialize_pretty(audit)?)
}

pub(crate) fn decode_retrieval_audit_json_checked(
    bytes: &[u8],
) -> Result<RetrievalAuditV1, CheckpointError> {
    let audit: RetrievalAuditV1 = JsonCodec.deserialize(bytes)?;
    check_retrieval_audit_ver(audit.version())?;
    audit.check_bind()?;
    Ok(audit)
}

#[allow(clippy::too_many_arguments)]
fn retrieval_audit_bind(
    height: u64,
    interval_blocks: u64,
    archive_manifest_root: [u8; 32],
    requested_entries_root: [u8; 32],
    successful_receipts_root: [u8; 32],
    failed_receipts_root: [u8; 32],
    successful_replica_count: u32,
    passed: bool,
) -> [u8; 32] {
    let height = height.to_le_bytes();
    let interval_blocks = interval_blocks.to_le_bytes();
    let successful_replica_count = successful_replica_count.to_le_bytes();
    let passed = [u8::from(passed)];
    hash_zk::<StorCheckpointRetrievalAuditDom>(
        RETRIEVAL_AUDIT_BIND_LABEL,
        &[
            &height,
            &interval_blocks,
            &archive_manifest_root,
            &requested_entries_root,
            &successful_receipts_root,
            &failed_receipts_root,
            &successful_replica_count,
            &passed,
        ],
    )
}

fn is_zero_root(root: &[u8; 32]) -> bool {
    root.iter().all(|byte| *byte == 0)
}

#[cfg(test)]
mod tests {
    use super::{check_retrieval_audit_ver, RetrievalAuditV1, RetrievalAuditVersion};
    use crate::CheckpointError;

    fn root(byte: u8) -> [u8; 32] {
        [byte; 32]
    }

    #[test]
    fn test_retrieval_audit_builds() {
        let audit = RetrievalAuditV1::new(
            RetrievalAuditVersion::CURRENT,
            1000,
            1000,
            root(1),
            root(2),
            root(3),
            [0u8; 32],
            3,
            true,
        )
        .expect("retrieval audit");

        assert_eq!(audit.height(), 1000);
        assert_eq!(audit.successful_replica_count(), 3);
    }

    #[test]
    fn test_retrieval_audit_requires_replicas() {
        let err = RetrievalAuditV1::new(
            RetrievalAuditVersion::CURRENT,
            1000,
            1000,
            root(1),
            root(2),
            root(3),
            [0u8; 32],
            2,
            true,
        )
        .expect_err("insufficient replicas reject");

        assert!(matches!(err, CheckpointError::ArchiveMix));
    }

    #[test]
    fn test_retrieval_audit_failed_rejects() {
        let err = RetrievalAuditV1::new(
            RetrievalAuditVersion::CURRENT,
            1000,
            1000,
            root(1),
            root(2),
            root(3),
            root(4),
            3,
            false,
        )
        .expect_err("failed audit rejects");

        assert!(matches!(err, CheckpointError::ArchiveMix));
    }

    #[test]
    fn test_audit_bad_version() {
        let err =
            check_retrieval_audit_ver(RetrievalAuditVersion::new(9)).expect_err("bad version");

        assert!(matches!(err, CheckpointError::VersionMix));
    }
}
