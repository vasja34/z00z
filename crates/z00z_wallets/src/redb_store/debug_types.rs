#[cfg(feature = "wallet_debug_tools")]
use super::super::{
    decode_bincode, decode_object_id_be, AccountPayload, AppPayload, BackupManifestPayload,
    ChainPayload, DerivationStatePayload, KdfParams, KeysPayload, ObjectKindId, OwnedAssetPayload,
    OwnedRightPayload, OwnedVoucherPayload, PersistWalletId, ScanStatePayload, StealthMetaPayload,
    TofuPinsPayload, WalletProfilePayload, WalletRootPayload, WalletTxEventPayload,
    WalletTxPayload, META_CHAIN_OBJECT_ID, META_DERIVATION_STATE_OBJECT_ID, META_KEYS_OBJECT_ID,
    META_SCAN_STATE_OBJECT_ID, META_SCHEMA_VERSION, META_STEALTH_META_OBJECT_ID,
    META_TOFU_PINS_OBJECT_ID, META_WALLET_CHAIN, META_WALLET_CREATED_AT, META_WALLET_ID,
    META_WALLET_INITIALIZED, META_WALLET_KDF, META_WALLET_UPDATED_AT, PAYLOAD_VERSION_ACCOUNT,
    PAYLOAD_VERSION_APP, PAYLOAD_VERSION_BACKUP_MANIFEST, PAYLOAD_VERSION_CHAIN,
    PAYLOAD_VERSION_DERIVATION_STATE, PAYLOAD_VERSION_KEYS, PAYLOAD_VERSION_OWNED_ASSET,
    PAYLOAD_VERSION_OWNED_RIGHT, PAYLOAD_VERSION_OWNED_VOUCHER, PAYLOAD_VERSION_SCAN_STATE,
    PAYLOAD_VERSION_STEALTH_META, PAYLOAD_VERSION_TOFU_PINS, PAYLOAD_VERSION_WALLET_PROFILE,
    PAYLOAD_VERSION_WALLET_ROOT, PAYLOAD_VERSION_WALLET_TX, PAYLOAD_VERSION_WALLET_TX_EVENT,
};
#[cfg(feature = "wallet_debug_tools")]
use base64::Engine as _;
#[cfg(feature = "wallet_debug_tools")]
use serde::Serialize;
#[cfg(feature = "wallet_debug_tools")]
use std::collections::BTreeMap;
#[cfg(feature = "wallet_debug_tools")]
use z00z_utils::codec::{Codec, JsonCodec, Value};

#[cfg(feature = "wallet_debug_tools")]
#[derive(Debug, Clone, Serialize)]
pub struct DebugTableRow {
    pub key_b64: String,
    pub value_b64: String,
}

#[cfg(feature = "wallet_debug_tools")]
#[derive(Debug, Clone, Serialize)]
pub struct DebugMetaEntry {
    pub key: String,
    pub raw_b64: String,
    pub decoded: Option<Value>,
}

#[cfg(feature = "wallet_debug_tools")]
#[derive(Debug, Clone, Serialize)]
pub struct DebugSecretEntry {
    pub name: String,
    pub kind: String,
    pub label: String,
    pub version: u16,
    pub record_raw_b64: String,
    pub record_error: Option<String>,
    pub plaintext_b64: String,
    pub plaintext_utf8: Option<String>,
    pub seed_phrase: Option<String>,
}

#[cfg(feature = "wallet_debug_tools")]
#[derive(Debug, Clone, Serialize)]
pub struct DebugIndexKey {
    pub table_name: String,
    pub table: String,
    pub object_id_hex: String,
    pub semantic_b64: String,
}

#[cfg(feature = "wallet_debug_tools")]
#[derive(Debug, Clone, Serialize)]
pub struct DebugObjectEntry {
    pub object_id_hex: String,
    pub kind_id: u8,
    pub payload_version: u16,
    pub payload_len: usize,
    pub payload_data_b64: String,
    pub decoded: Option<Value>,
}

#[cfg(feature = "wallet_debug_tools")]
#[derive(Debug, Clone, Serialize)]
pub struct DebugWalletDump {
    pub wlt_path: String,
    pub wallet_id: String,
    pub schema_version: u32,
    pub meta: Vec<DebugMetaEntry>,
    pub secrets: Vec<DebugSecretEntry>,
    pub secrets_redacted: bool,
    pub objects: Vec<DebugObjectEntry>,
    pub tables: BTreeMap<String, Vec<DebugTableRow>>,
    pub index_keys: Vec<DebugIndexKey>,
    pub table_errors: Vec<DebugTableError>,
}

#[cfg(feature = "wallet_debug_tools")]
#[derive(Debug, Clone, Serialize)]
pub struct DebugTableError {
    pub table_name: String,
    pub error: String,
}

#[cfg(feature = "wallet_debug_tools")]
pub(super) fn b64(bytes: &[u8]) -> String {
    base64::engine::general_purpose::STANDARD.encode(bytes)
}

#[cfg(feature = "wallet_debug_tools")]
fn json_value<T: Serialize>(value: &T) -> Option<Value> {
    let json = JsonCodec.serialize(value).ok()?;
    JsonCodec.deserialize::<Value>(&json).ok()
}

#[cfg(feature = "wallet_debug_tools")]
pub(super) fn decode_meta_value(key: &str, raw: &[u8]) -> Option<Value> {
    match key {
        META_WALLET_ID => decode_bincode::<PersistWalletId>(raw)
            .ok()
            .and_then(|v| json_value(&v)),
        META_SCHEMA_VERSION => decode_bincode::<u32>(raw).ok().and_then(|v| json_value(&v)),
        META_WALLET_KDF => decode_bincode::<KdfParams>(raw)
            .ok()
            .and_then(|v| json_value(&v)),
        META_WALLET_INITIALIZED => decode_bincode::<u8>(raw).ok().and_then(|v| json_value(&v)),
        META_WALLET_CREATED_AT => decode_bincode::<u64>(raw).ok().and_then(|v| json_value(&v)),
        META_WALLET_UPDATED_AT => decode_bincode::<u64>(raw).ok().and_then(|v| json_value(&v)),
        META_WALLET_CHAIN => decode_bincode::<String>(raw)
            .ok()
            .and_then(|v| json_value(&v)),
        META_DERIVATION_STATE_OBJECT_ID => decode_object_id_be(raw)
            .ok()
            .and_then(|v| json_value(&format!("0x{v:032x}"))),
        META_SCAN_STATE_OBJECT_ID => decode_object_id_be(raw)
            .ok()
            .and_then(|v| json_value(&format!("0x{v:032x}"))),
        META_CHAIN_OBJECT_ID => decode_object_id_be(raw)
            .ok()
            .and_then(|v| json_value(&format!("0x{v:032x}"))),
        META_KEYS_OBJECT_ID => decode_object_id_be(raw)
            .ok()
            .and_then(|v| json_value(&format!("0x{v:032x}"))),
        META_STEALTH_META_OBJECT_ID => decode_object_id_be(raw)
            .ok()
            .and_then(|v| json_value(&format!("0x{v:032x}"))),
        META_TOFU_PINS_OBJECT_ID => decode_object_id_be(raw)
            .ok()
            .and_then(|v| json_value(&format!("0x{v:032x}"))),
        _ => None,
    }
}

#[cfg(feature = "wallet_debug_tools")]
pub(crate) fn decode_object_json(kind_id: u8, payload_version: u16, data: &[u8]) -> Option<Value> {
    match kind_id {
        x if x == ObjectKindId::WalletRoot as u8
            && payload_version == PAYLOAD_VERSION_WALLET_ROOT =>
        {
            decode_bincode::<WalletRootPayload>(data)
                .ok()
                .and_then(|v| json_value(&v))
        }
        x if x == ObjectKindId::Account as u8 && payload_version == PAYLOAD_VERSION_ACCOUNT => {
            decode_bincode::<AccountPayload>(data)
                .ok()
                .and_then(|v| json_value(&v))
        }
        x if x == ObjectKindId::DerivationState as u8
            && payload_version == PAYLOAD_VERSION_DERIVATION_STATE =>
        {
            decode_bincode::<DerivationStatePayload>(data)
                .ok()
                .and_then(|v| json_value(&v))
        }
        x if x == ObjectKindId::ScanState as u8
            && payload_version == PAYLOAD_VERSION_SCAN_STATE =>
        {
            decode_bincode::<ScanStatePayload>(data)
                .ok()
                .and_then(|v| json_value(&v))
        }
        x if x == ObjectKindId::App as u8 && payload_version == PAYLOAD_VERSION_APP => {
            decode_bincode::<AppPayload>(data)
                .ok()
                .and_then(|v| json_value(&v))
        }
        x if x == ObjectKindId::Chain as u8 && payload_version == PAYLOAD_VERSION_CHAIN => {
            decode_bincode::<ChainPayload>(data)
                .ok()
                .and_then(|v| json_value(&v))
        }
        x if x == ObjectKindId::Keys as u8 && payload_version == PAYLOAD_VERSION_KEYS => {
            decode_bincode::<KeysPayload>(data)
                .ok()
                .and_then(|v| json_value(&v))
        }
        x if x == ObjectKindId::StealthMeta as u8
            && payload_version == PAYLOAD_VERSION_STEALTH_META =>
        {
            decode_bincode::<StealthMetaPayload>(data)
                .ok()
                .and_then(|v| json_value(&v))
        }
        x if x == ObjectKindId::TofuPins as u8 && payload_version == PAYLOAD_VERSION_TOFU_PINS => {
            decode_bincode::<TofuPinsPayload>(data)
                .ok()
                .and_then(|v| json_value(&v))
        }
        x if x == ObjectKindId::WalletProfile as u8
            && payload_version == PAYLOAD_VERSION_WALLET_PROFILE =>
        {
            decode_bincode::<WalletProfilePayload>(data)
                .ok()
                .and_then(|v| json_value(&v))
        }
        x if x == ObjectKindId::OwnedAsset as u8
            && payload_version == PAYLOAD_VERSION_OWNED_ASSET =>
        {
            decode_bincode::<OwnedAssetPayload>(data)
                .ok()
                .and_then(|v| json_value(&v))
        }
        x if x == ObjectKindId::OwnedVoucher as u8
            && payload_version == PAYLOAD_VERSION_OWNED_VOUCHER =>
        {
            decode_bincode::<OwnedVoucherPayload>(data)
                .ok()
                .and_then(|v| json_value(&v))
        }
        x if x == ObjectKindId::OwnedRight as u8
            && payload_version == PAYLOAD_VERSION_OWNED_RIGHT =>
        {
            decode_bincode::<OwnedRightPayload>(data)
                .ok()
                .and_then(|v| json_value(&v))
        }
        x if x == ObjectKindId::WalletTx as u8 && payload_version == PAYLOAD_VERSION_WALLET_TX => {
            decode_bincode::<WalletTxPayload>(data)
                .ok()
                .and_then(|v| json_value(&v))
        }
        x if x == ObjectKindId::WalletTxEvent as u8
            && payload_version == PAYLOAD_VERSION_WALLET_TX_EVENT =>
        {
            decode_bincode::<WalletTxEventPayload>(data)
                .ok()
                .and_then(|v| json_value(&v))
        }
        x if x == ObjectKindId::BackupManifest as u8
            && payload_version == PAYLOAD_VERSION_BACKUP_MANIFEST =>
        {
            decode_bincode::<BackupManifestPayload>(data)
                .ok()
                .and_then(|v| json_value(&v))
        }
        _ => None,
    }
}
