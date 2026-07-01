use jmt::KeyHash;
use z00z_crypto::expert::hash_domain;
use z00z_crypto::hash_zk::hash_zk;

use super::{DefinitionId, SerialId, TerminalId};

hash_domain!(StorDefKeyDom, "z00z.storage.key.definition.v1", 1);
hash_domain!(StorSerKeyDom, "z00z.storage.key.serial.v1", 1);

#[must_use]
pub fn definition_key(def_id: DefinitionId) -> KeyHash {
    KeyHash(hash_zk::<StorDefKeyDom>("", &[def_id.as_bytes()]))
}

#[must_use]
pub fn serial_key(def_id: DefinitionId, serial_id: SerialId) -> KeyHash {
    let serial = serial_id.get().to_le_bytes();
    KeyHash(hash_zk::<StorSerKeyDom>("", &[def_id.as_bytes(), &serial]))
}

#[must_use]
pub fn terminal_key(terminal_id: TerminalId) -> KeyHash {
    KeyHash(terminal_id.into_bytes())
}
