use serde::{Deserialize, Serialize};

use crate::db::wallet_store_crypto::AeadEnvelope;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncryptedObjectPayload {
    pub payload_version: u16,
    pub kind_id: u8,
    pub data: Vec<u8>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncryptedObjectRecord {
    pub envelope: AeadEnvelope,
    pub payload_version: u16,
}
