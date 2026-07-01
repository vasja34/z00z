use jmt::KeyHash;
use z00z_crypto::{expert::hash_domain, hash_zk::hash_zk};
use z00z_utils::codec::{BincodeCodec, Codec};

use crate::{
    settlement::{
        keys::terminal_key,
        tree_id::{PathIndexRec, TreeId},
        DefinitionRootLeaf, SerialRootLeaf, SettlementLeaf, SettlementPath, TerminalId,
    },
    CheckpointError,
};

use super::types::SettlementStoreError;

hash_domain!(StorTerminalNsDom, "z00z.storage.ns.asset.v1", 1);
hash_domain!(StorSerNsDom, "z00z.storage.ns.serial.v1", 1);
hash_domain!(StorDefNsDom, "z00z.storage.ns.definition.v1", 1);

pub(crate) fn ns_key(tree_id: TreeId, key: KeyHash) -> KeyHash {
    let ns = tree_id.ns_bytes();
    match tree_id {
        TreeId::Definition => KeyHash(hash_zk::<StorDefNsDom>("", &[ns.as_slice(), &key.0])),
        TreeId::Serial(..) => KeyHash(hash_zk::<StorSerNsDom>("", &[ns.as_slice(), &key.0])),
        TreeId::Terminal(..) => KeyHash(hash_zk::<StorTerminalNsDom>("", &[ns.as_slice(), &key.0])),
    }
}

pub(crate) fn key_from_terminal_id(terminal_id: TerminalId) -> KeyHash {
    terminal_key(terminal_id)
}

pub(crate) fn leaf_payload(
    leaf: impl Into<SettlementLeaf>,
) -> Result<Vec<u8>, SettlementStoreError> {
    Ok(leaf.into().encode()?)
}

pub(crate) fn path_payload(
    path: impl Into<SettlementPath>,
) -> Result<Vec<u8>, SettlementStoreError> {
    let codec = BincodeCodec;
    let path: SettlementPath = path.into();
    Ok(codec.serialize(&PathIndexRec::new(path))?)
}

pub(crate) fn def_payload(leaf: DefinitionRootLeaf) -> Vec<u8> {
    leaf.encode()
}

pub(crate) fn ser_payload(leaf: SerialRootLeaf) -> Vec<u8> {
    leaf.encode()
}

pub(crate) fn map_jmt_err(err: impl std::fmt::Display) -> SettlementStoreError {
    SettlementStoreError::Jmt(err.to_string())
}

pub(crate) fn map_check_err(err: CheckpointError) -> SettlementStoreError {
    SettlementStoreError::Backend(err.to_string())
}
