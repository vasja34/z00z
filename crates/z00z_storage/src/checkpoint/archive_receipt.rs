use z00z_crypto::expert::hash_domain;
use z00z_crypto::hash_zk::hash_zk;
use z00z_utils::codec::{BincodeCodec, Codec, JsonCodec};

use crate::CheckpointError;

hash_domain!(
    StorCheckpointArchiveReceiptDom,
    "z00z.storage.checkpoint.archive_receipt",
    1
);

const ARCHIVE_RECEIPT_BIND_VER: u8 = 1;
const ARCHIVE_RECEIPT_BIND_LABEL: &str = "archive_provider_receipt_v1";

#[derive(
    Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, serde::Serialize, serde::Deserialize,
)]
pub struct ArchiveProviderReceiptVersion(u8);

impl ArchiveProviderReceiptVersion {
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

#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ArchiveBackend {
    Z00zArchiveNode,
    IpfsPinned,
    PaidArchivalProvider,
    FilecoinOrEquivalent,
    ColdObjectStore,
}

impl ArchiveBackend {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Z00zArchiveNode => "z00z_archive_node",
            Self::IpfsPinned => "ipfs_pinned",
            Self::PaidArchivalProvider => "paid_archival_provider",
            Self::FilecoinOrEquivalent => "filecoin_or_equivalent",
            Self::ColdObjectStore => "cold_object_store",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ArchiveProviderReceiptV1 {
    version: ArchiveProviderReceiptVersion,
    backend: ArchiveBackend,
    content_cid_or_digest: [u8; 32],
    byte_length: u64,
    provider_identity_digest: [u8; 32],
    receipt_digest: [u8; 32],
    pinned: bool,
    paid_or_operator_committed: bool,
    #[serde(default)]
    archive_receipt_bind_ver: u8,
    #[serde(default)]
    archive_receipt_bind: [u8; 32],
}

impl ArchiveProviderReceiptV1 {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        version: ArchiveProviderReceiptVersion,
        backend: ArchiveBackend,
        content_cid_or_digest: [u8; 32],
        byte_length: u64,
        provider_identity_digest: [u8; 32],
        receipt_digest: [u8; 32],
        pinned: bool,
        paid_or_operator_committed: bool,
    ) -> Result<Self, CheckpointError> {
        check_archive_receipt_ver(version)?;
        if byte_length == 0
            || is_zero_root(&content_cid_or_digest)
            || is_zero_root(&provider_identity_digest)
            || is_zero_root(&receipt_digest)
            || !paid_or_operator_committed
        {
            return Err(CheckpointError::ArchiveMix);
        }
        if backend == ArchiveBackend::IpfsPinned && !pinned {
            return Err(CheckpointError::ArchiveMix);
        }
        let archive_receipt_bind = archive_receipt_bind(
            backend,
            content_cid_or_digest,
            byte_length,
            provider_identity_digest,
            receipt_digest,
            pinned,
            paid_or_operator_committed,
        );
        Ok(Self {
            version,
            backend,
            content_cid_or_digest,
            byte_length,
            provider_identity_digest,
            receipt_digest,
            pinned,
            paid_or_operator_committed,
            archive_receipt_bind_ver: ARCHIVE_RECEIPT_BIND_VER,
            archive_receipt_bind,
        })
    }

    #[must_use]
    pub const fn version(&self) -> ArchiveProviderReceiptVersion {
        self.version
    }

    #[must_use]
    pub const fn backend(&self) -> ArchiveBackend {
        self.backend
    }

    #[must_use]
    pub const fn content_cid_or_digest(&self) -> [u8; 32] {
        self.content_cid_or_digest
    }

    #[must_use]
    pub const fn byte_length(&self) -> u64 {
        self.byte_length
    }

    #[must_use]
    pub const fn provider_identity_digest(&self) -> [u8; 32] {
        self.provider_identity_digest
    }

    #[must_use]
    pub const fn receipt_digest(&self) -> [u8; 32] {
        self.receipt_digest
    }

    #[must_use]
    pub const fn pinned(&self) -> bool {
        self.pinned
    }

    #[must_use]
    pub const fn paid_or_operator_committed(&self) -> bool {
        self.paid_or_operator_committed
    }

    fn check_bind(&self) -> Result<(), CheckpointError> {
        if self.archive_receipt_bind_ver != ARCHIVE_RECEIPT_BIND_VER {
            return Err(CheckpointError::ArchiveMix);
        }
        if self.archive_receipt_bind
            != archive_receipt_bind(
                self.backend,
                self.content_cid_or_digest,
                self.byte_length,
                self.provider_identity_digest,
                self.receipt_digest,
                self.pinned,
                self.paid_or_operator_committed,
            )
        {
            return Err(CheckpointError::ArchiveMix);
        }
        Ok(())
    }
}

pub(crate) fn check_archive_receipt_ver(
    version: ArchiveProviderReceiptVersion,
) -> Result<(), CheckpointError> {
    if version == ArchiveProviderReceiptVersion::CURRENT {
        return Ok(());
    }
    Err(CheckpointError::VersionMix)
}

pub(crate) fn encode_archive_receipt_bin_checked(
    receipt: &ArchiveProviderReceiptV1,
) -> Result<Vec<u8>, CheckpointError> {
    check_archive_receipt_ver(receipt.version())?;
    receipt.check_bind()?;
    Ok(BincodeCodec.serialize(receipt)?)
}

pub(crate) fn decode_archive_receipt_bin_checked(
    bytes: &[u8],
) -> Result<ArchiveProviderReceiptV1, CheckpointError> {
    let receipt: ArchiveProviderReceiptV1 = BincodeCodec.deserialize(bytes)?;
    check_archive_receipt_ver(receipt.version())?;
    receipt.check_bind()?;
    Ok(receipt)
}

pub(crate) fn encode_archive_receipt_json_checked(
    receipt: &ArchiveProviderReceiptV1,
) -> Result<Vec<u8>, CheckpointError> {
    check_archive_receipt_ver(receipt.version())?;
    receipt.check_bind()?;
    Ok(JsonCodec.serialize_pretty(receipt)?)
}

pub(crate) fn decode_archive_receipt_json_checked(
    bytes: &[u8],
) -> Result<ArchiveProviderReceiptV1, CheckpointError> {
    let receipt: ArchiveProviderReceiptV1 = JsonCodec.deserialize(bytes)?;
    check_archive_receipt_ver(receipt.version())?;
    receipt.check_bind()?;
    Ok(receipt)
}

fn archive_receipt_bind(
    backend: ArchiveBackend,
    content_cid_or_digest: [u8; 32],
    byte_length: u64,
    provider_identity_digest: [u8; 32],
    receipt_digest: [u8; 32],
    pinned: bool,
    paid_or_operator_committed: bool,
) -> [u8; 32] {
    let byte_length = byte_length.to_le_bytes();
    let pinned = [u8::from(pinned)];
    let committed = [u8::from(paid_or_operator_committed)];
    hash_zk::<StorCheckpointArchiveReceiptDom>(
        ARCHIVE_RECEIPT_BIND_LABEL,
        &[
            backend.as_str().as_bytes(),
            &content_cid_or_digest,
            &byte_length,
            &provider_identity_digest,
            &receipt_digest,
            &pinned,
            &committed,
        ],
    )
}

fn is_zero_root(root: &[u8; 32]) -> bool {
    root.iter().all(|byte| *byte == 0)
}

#[cfg(test)]
mod tests {
    use super::{
        check_archive_receipt_ver, ArchiveBackend, ArchiveProviderReceiptV1,
        ArchiveProviderReceiptVersion,
    };
    use crate::CheckpointError;

    fn root(byte: u8) -> [u8; 32] {
        [byte; 32]
    }

    #[test]
    fn test_ipfs_pinned_receipt_builds() {
        let receipt = ArchiveProviderReceiptV1::new(
            ArchiveProviderReceiptVersion::CURRENT,
            ArchiveBackend::IpfsPinned,
            root(1),
            1024,
            root(2),
            root(3),
            true,
            true,
        )
        .expect("ipfs receipt");

        assert_eq!(receipt.backend(), ArchiveBackend::IpfsPinned);
        assert!(receipt.pinned());
    }

    #[test]
    fn test_unpinned_ipfs_receipt_rejects() {
        let err = ArchiveProviderReceiptV1::new(
            ArchiveProviderReceiptVersion::CURRENT,
            ArchiveBackend::IpfsPinned,
            root(1),
            1024,
            root(2),
            root(3),
            false,
            true,
        )
        .expect_err("unpinned ipfs rejects");

        assert!(matches!(err, CheckpointError::ArchiveMix));
    }

    #[test]
    fn test_missing_commitment_rejects() {
        let err = ArchiveProviderReceiptV1::new(
            ArchiveProviderReceiptVersion::CURRENT,
            ArchiveBackend::ColdObjectStore,
            root(1),
            1024,
            root(2),
            root(3),
            false,
            false,
        )
        .expect_err("missing provider commitment rejects");

        assert!(matches!(err, CheckpointError::ArchiveMix));
    }

    #[test]
    fn test_receipt_bad_version() {
        let err = check_archive_receipt_ver(ArchiveProviderReceiptVersion::new(9))
            .expect_err("bad version");

        assert!(matches!(err, CheckpointError::VersionMix));
    }
}
