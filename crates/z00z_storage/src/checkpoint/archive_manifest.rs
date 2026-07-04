use z00z_crypto::expert::hash_domain;
use z00z_crypto::hash_zk::hash_zk;
use z00z_utils::codec::{BincodeCodec, Codec, JsonCodec};

use crate::CheckpointError;

hash_domain!(
    StorCheckpointArchiveManifestDom,
    "z00z.storage.checkpoint.archive_manifest",
    1
);

const ARCHIVE_MANIFEST_BIND_VER: u8 = 1;
const ARCHIVE_MANIFEST_BIND_LABEL: &str = "checkpoint_archive_manifest_v1";

#[derive(
    Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, serde::Serialize, serde::Deserialize,
)]
pub struct ArchiveManifestVersion(u8);

impl ArchiveManifestVersion {
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
pub struct CheckpointArchiveManifestV1 {
    version: ArchiveManifestVersion,
    statement_digest: [u8; 32],
    epoch_manifest_root: [u8; 32],
    raw_tx_package_root: [u8; 32],
    exact_tx_proof_bytes_root: [u8; 32],
    witness_archive_root: [u8; 32],
    delta_journal_root: [u8; 32],
    da_payload_commitment: [u8; 32],
    archive_provider_receipt_root: [u8; 32],
    retrieval_audit_root: [u8; 32],
    content_address_root: [u8; 32],
    min_archive_replicas: u32,
    #[serde(default)]
    archive_manifest_bind_ver: u8,
    #[serde(default)]
    archive_manifest_bind: [u8; 32],
}

impl CheckpointArchiveManifestV1 {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        version: ArchiveManifestVersion,
        statement_digest: [u8; 32],
        epoch_manifest_root: [u8; 32],
        raw_tx_package_root: [u8; 32],
        exact_tx_proof_bytes_root: [u8; 32],
        witness_archive_root: [u8; 32],
        delta_journal_root: [u8; 32],
        da_payload_commitment: [u8; 32],
        archive_provider_receipt_root: [u8; 32],
        retrieval_audit_root: [u8; 32],
        content_address_root: [u8; 32],
        min_archive_replicas: u32,
    ) -> Result<Self, CheckpointError> {
        check_archive_manifest_ver(version)?;
        if min_archive_replicas < 3 {
            return Err(CheckpointError::ArchiveMix);
        }
        let roots = [
            statement_digest,
            epoch_manifest_root,
            raw_tx_package_root,
            exact_tx_proof_bytes_root,
            witness_archive_root,
            delta_journal_root,
            da_payload_commitment,
            archive_provider_receipt_root,
            retrieval_audit_root,
            content_address_root,
        ];
        if roots.iter().any(is_zero_root) {
            return Err(CheckpointError::ArchiveMix);
        }
        let archive_manifest_bind = archive_manifest_bind(
            statement_digest,
            epoch_manifest_root,
            raw_tx_package_root,
            exact_tx_proof_bytes_root,
            witness_archive_root,
            delta_journal_root,
            da_payload_commitment,
            archive_provider_receipt_root,
            retrieval_audit_root,
            content_address_root,
            min_archive_replicas,
        );
        Ok(Self {
            version,
            statement_digest,
            epoch_manifest_root,
            raw_tx_package_root,
            exact_tx_proof_bytes_root,
            witness_archive_root,
            delta_journal_root,
            da_payload_commitment,
            archive_provider_receipt_root,
            retrieval_audit_root,
            content_address_root,
            min_archive_replicas,
            archive_manifest_bind_ver: ARCHIVE_MANIFEST_BIND_VER,
            archive_manifest_bind,
        })
    }

    #[must_use]
    pub const fn version(&self) -> ArchiveManifestVersion {
        self.version
    }

    #[must_use]
    pub const fn statement_digest(&self) -> [u8; 32] {
        self.statement_digest
    }

    #[must_use]
    pub const fn epoch_manifest_root(&self) -> [u8; 32] {
        self.epoch_manifest_root
    }

    #[must_use]
    pub const fn raw_tx_package_root(&self) -> [u8; 32] {
        self.raw_tx_package_root
    }

    #[must_use]
    pub const fn exact_tx_proof_bytes_root(&self) -> [u8; 32] {
        self.exact_tx_proof_bytes_root
    }

    #[must_use]
    pub const fn witness_archive_root(&self) -> [u8; 32] {
        self.witness_archive_root
    }

    #[must_use]
    pub const fn delta_journal_root(&self) -> [u8; 32] {
        self.delta_journal_root
    }

    #[must_use]
    pub const fn da_payload_commitment(&self) -> [u8; 32] {
        self.da_payload_commitment
    }

    #[must_use]
    pub const fn archive_provider_receipt_root(&self) -> [u8; 32] {
        self.archive_provider_receipt_root
    }

    #[must_use]
    pub const fn retrieval_audit_root(&self) -> [u8; 32] {
        self.retrieval_audit_root
    }

    #[must_use]
    pub const fn content_address_root(&self) -> [u8; 32] {
        self.content_address_root
    }

    #[must_use]
    pub const fn min_archive_replicas(&self) -> u32 {
        self.min_archive_replicas
    }

    fn check_bind(&self) -> Result<(), CheckpointError> {
        if self.archive_manifest_bind_ver != ARCHIVE_MANIFEST_BIND_VER {
            return Err(CheckpointError::ArchiveMix);
        }
        if self.archive_manifest_bind
            != archive_manifest_bind(
                self.statement_digest,
                self.epoch_manifest_root,
                self.raw_tx_package_root,
                self.exact_tx_proof_bytes_root,
                self.witness_archive_root,
                self.delta_journal_root,
                self.da_payload_commitment,
                self.archive_provider_receipt_root,
                self.retrieval_audit_root,
                self.content_address_root,
                self.min_archive_replicas,
            )
        {
            return Err(CheckpointError::ArchiveMix);
        }
        Ok(())
    }
}

pub(crate) fn check_archive_manifest_ver(
    version: ArchiveManifestVersion,
) -> Result<(), CheckpointError> {
    if version == ArchiveManifestVersion::CURRENT {
        return Ok(());
    }
    Err(CheckpointError::VersionMix)
}

pub(crate) fn encode_archive_manifest_bin_checked(
    manifest: &CheckpointArchiveManifestV1,
) -> Result<Vec<u8>, CheckpointError> {
    check_archive_manifest_ver(manifest.version())?;
    manifest.check_bind()?;
    Ok(BincodeCodec.serialize(manifest)?)
}

pub(crate) fn decode_archive_manifest_bin_checked(
    bytes: &[u8],
) -> Result<CheckpointArchiveManifestV1, CheckpointError> {
    let manifest: CheckpointArchiveManifestV1 = BincodeCodec.deserialize(bytes)?;
    check_archive_manifest_ver(manifest.version())?;
    manifest.check_bind()?;
    Ok(manifest)
}

pub(crate) fn encode_archive_manifest_json_checked(
    manifest: &CheckpointArchiveManifestV1,
) -> Result<Vec<u8>, CheckpointError> {
    check_archive_manifest_ver(manifest.version())?;
    manifest.check_bind()?;
    Ok(JsonCodec.serialize_pretty(manifest)?)
}

pub(crate) fn decode_archive_manifest_json_checked(
    bytes: &[u8],
) -> Result<CheckpointArchiveManifestV1, CheckpointError> {
    let manifest: CheckpointArchiveManifestV1 = JsonCodec.deserialize(bytes)?;
    check_archive_manifest_ver(manifest.version())?;
    manifest.check_bind()?;
    Ok(manifest)
}

#[allow(clippy::too_many_arguments)]
fn archive_manifest_bind(
    statement_digest: [u8; 32],
    epoch_manifest_root: [u8; 32],
    raw_tx_package_root: [u8; 32],
    exact_tx_proof_bytes_root: [u8; 32],
    witness_archive_root: [u8; 32],
    delta_journal_root: [u8; 32],
    da_payload_commitment: [u8; 32],
    archive_provider_receipt_root: [u8; 32],
    retrieval_audit_root: [u8; 32],
    content_address_root: [u8; 32],
    min_archive_replicas: u32,
) -> [u8; 32] {
    let replicas = min_archive_replicas.to_le_bytes();
    hash_zk::<StorCheckpointArchiveManifestDom>(
        ARCHIVE_MANIFEST_BIND_LABEL,
        &[
            &statement_digest,
            &epoch_manifest_root,
            &raw_tx_package_root,
            &exact_tx_proof_bytes_root,
            &witness_archive_root,
            &delta_journal_root,
            &da_payload_commitment,
            &archive_provider_receipt_root,
            &retrieval_audit_root,
            &content_address_root,
            &replicas,
        ],
    )
}

fn is_zero_root(root: &[u8; 32]) -> bool {
    root.iter().all(|byte| *byte == 0)
}

#[cfg(test)]
mod tests {
    use super::{check_archive_manifest_ver, ArchiveManifestVersion, CheckpointArchiveManifestV1};
    use crate::CheckpointError;

    fn root(byte: u8) -> [u8; 32] {
        [byte; 32]
    }

    fn manifest() -> CheckpointArchiveManifestV1 {
        CheckpointArchiveManifestV1::new(
            ArchiveManifestVersion::CURRENT,
            root(1),
            root(2),
            root(3),
            root(4),
            root(5),
            root(6),
            root(7),
            root(8),
            root(9),
            root(10),
            3,
        )
        .expect("archive manifest")
    }

    #[test]
    fn test_archive_manifest_builds() {
        let got = manifest();

        assert_eq!(got.min_archive_replicas(), 3);
        assert_eq!(got.exact_tx_proof_bytes_root(), root(4));
    }

    #[test]
    fn test_manifest_min_replicas() {
        let err = CheckpointArchiveManifestV1::new(
            ArchiveManifestVersion::CURRENT,
            root(1),
            root(2),
            root(3),
            root(4),
            root(5),
            root(6),
            root(7),
            root(8),
            root(9),
            root(10),
            2,
        )
        .expect_err("low replica count rejects");

        assert!(matches!(err, CheckpointError::ArchiveMix));
    }

    #[test]
    fn test_manifest_nonzero_roots() {
        let err = CheckpointArchiveManifestV1::new(
            ArchiveManifestVersion::CURRENT,
            [0u8; 32],
            root(2),
            root(3),
            root(4),
            root(5),
            root(6),
            root(7),
            root(8),
            root(9),
            root(10),
            3,
        )
        .expect_err("zero root rejects");

        assert!(matches!(err, CheckpointError::ArchiveMix));
    }

    #[test]
    fn test_manifest_bad_version() {
        let err =
            check_archive_manifest_ver(ArchiveManifestVersion::new(9)).expect_err("bad version");

        assert!(matches!(err, CheckpointError::VersionMix));
    }
}
