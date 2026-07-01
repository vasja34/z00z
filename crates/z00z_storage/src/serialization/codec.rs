use sha2::{Digest, Sha256};
use z00z_utils::codec::{BincodeCodec, Codec};

use crate::{
    error::SerializationError,
    serialization::{JmtSerArtifact, JmtSerArtifactId, JmtSerVersion},
};

pub(crate) fn check_ver(version: JmtSerVersion) -> Result<(), SerializationError> {
    if version == JmtSerVersion::CURRENT {
        return Ok(());
    }

    Err(SerializationError::VersionMix)
}

pub fn encode_artifact(artifact: &JmtSerArtifact) -> Result<Vec<u8>, SerializationError> {
    check_ver(artifact.version)?;
    Ok(BincodeCodec.serialize(artifact)?)
}

pub fn decode_artifact(bytes: &[u8]) -> Result<JmtSerArtifact, SerializationError> {
    let artifact: JmtSerArtifact = BincodeCodec.deserialize(bytes)?;
    check_ver(artifact.version)?;
    Ok(artifact)
}

pub fn derive_artifact_id(
    artifact: &JmtSerArtifact,
) -> Result<JmtSerArtifactId, SerializationError> {
    let bytes = encode_artifact(artifact)?;
    Ok(JmtSerArtifactId::new(Sha256::digest(&bytes).into()))
}

#[cfg(test)]
mod tests {
    use super::{check_ver, decode_artifact, derive_artifact_id, encode_artifact};
    use crate::{
        error::SerializationError,
        serialization::{
            JmtSerArtifact, JmtSerEdge, JmtSerMeta, JmtSerNode, JmtSerNodeKind, JmtSerRoots,
            JmtSerTreeId, JmtSerTreeRoot, JmtSerVersion,
        },
        settlement::{BucketId, DefinitionId, SerialId, SettlementStateRoot},
    };

    fn artifact() -> JmtSerArtifact {
        let tree_id = JmtSerTreeId::Terminal {
            definition_id: DefinitionId::new([2u8; 32]),
            serial_id: SerialId::new(7),
            bucket_id: BucketId::new([3u8; 32]),
        };

        JmtSerArtifact::new(
            JmtSerVersion::CURRENT,
            JmtSerRoots::new(
                SettlementStateRoot::settlement_v1([1u8; 32]),
                vec![JmtSerTreeRoot::new(tree_id, [3u8; 32], [13u8; 32])],
            ),
            JmtSerMeta::new(Vec::new(), 1, 1),
            vec![JmtSerNode::new(
                [4u8; 32],
                tree_id,
                JmtSerNodeKind::Leaf,
                vec![5],
                [6u8; 32],
                Some([7u8; 32]),
                Some([8u8; 32]),
                vec![9],
            )],
            vec![JmtSerEdge::new(tree_id, [10u8; 32], [11u8; 32], 1)],
        )
    }

    #[test]
    fn test_codec_roundtrip() {
        let artifact = artifact();

        let bytes = encode_artifact(&artifact).expect("encode artifact");
        let decoded = decode_artifact(&bytes).expect("decode artifact");

        assert_eq!(decoded, artifact);
    }

    #[test]
    fn test_id_stable_across_reencode() {
        let artifact = artifact();

        let first = derive_artifact_id(&artifact).expect("first id");
        let bytes = encode_artifact(&artifact).expect("encode artifact");
        let decoded = decode_artifact(&bytes).expect("decode artifact");
        let second = derive_artifact_id(&decoded).expect("second id");

        assert_eq!(first, second);
    }

    #[test]
    fn test_bad_transport_fails_decode() {
        let err = decode_artifact(&[1u8, 2, 3]).unwrap_err();

        assert!(matches!(err, SerializationError::Codec(_)));
    }

    #[test]
    fn test_unsupported_version_fails_gate() {
        let err = check_ver(JmtSerVersion::new(9)).unwrap_err();

        assert!(matches!(err, SerializationError::VersionMix));
    }
}
