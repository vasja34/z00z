use sha2::{Digest, Sha256};
use z00z_utils::codec::{BincodeCodec, Codec};

use super::{PrepSnapshot, PrepSnapshotError, PrepSnapshotId, PrepSnapshotVersion};

pub(crate) fn check_ver(version: PrepSnapshotVersion) -> Result<(), PrepSnapshotError> {
    if version == PrepSnapshotVersion::CURRENT {
        return Ok(());
    }

    Err(PrepSnapshotError::VersionMix)
}

pub(crate) fn encode_snap(snapshot: &PrepSnapshot) -> Result<Vec<u8>, PrepSnapshotError> {
    check_ver(snapshot.version)?;
    Ok(BincodeCodec.serialize(snapshot)?)
}

pub(crate) fn decode_snap(bytes: &[u8]) -> Result<PrepSnapshot, PrepSnapshotError> {
    let snapshot: PrepSnapshot = BincodeCodec.deserialize(bytes)?;
    check_ver(snapshot.version)?;
    Ok(snapshot)
}

pub(crate) fn derive_id(snapshot: &PrepSnapshot) -> Result<PrepSnapshotId, PrepSnapshotError> {
    let bytes = encode_snap(snapshot)?;
    let hash: [u8; 32] = Sha256::digest(&bytes).into();
    Ok(PrepSnapshotId::new(hash))
}

#[cfg(test)]
mod tests {
    use super::{check_ver, decode_snap, derive_id, encode_snap};
    use crate::{
        settlement::{CheckRoot, SnapItem},
        snapshot::{PrepSnapshot, PrepSnapshotError, PrepSnapshotVersion},
    };

    #[test]
    fn test_codec_roundtrip() {
        let snapshot = PrepSnapshot::new(
            PrepSnapshotVersion::CURRENT,
            CheckRoot::new([5u8; 32]),
            Vec::<SnapItem>::new(),
        );

        let bytes = encode_snap(&snapshot).expect("encode snapshot");
        let decoded = decode_snap(&bytes).expect("decode snapshot");

        assert_eq!(decoded, snapshot);
    }

    #[test]
    fn test_id_stable_across_reencode() {
        let snapshot = PrepSnapshot::new(
            PrepSnapshotVersion::CURRENT,
            CheckRoot::new([7u8; 32]),
            Vec::<SnapItem>::new(),
        );

        let first = derive_id(&snapshot).expect("first id");
        let bytes = encode_snap(&snapshot).expect("encode snapshot");
        let decoded = decode_snap(&bytes).expect("decode snapshot");
        let second = derive_id(&decoded).expect("second id");

        assert_eq!(first, second);
    }

    #[test]
    fn test_bad_transport_fails_decode() {
        let err = decode_snap(&[1u8, 2, 3]).unwrap_err();

        assert!(matches!(err, PrepSnapshotError::Codec(_)));
    }

    #[test]
    fn test_unsupported_version_fails_gate() {
        let err = check_ver(PrepSnapshotVersion::new(9)).unwrap_err();

        assert!(matches!(err, PrepSnapshotError::VersionMix));
    }
}
