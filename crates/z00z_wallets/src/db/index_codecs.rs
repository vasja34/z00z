//! Canonical codecs for `.wlt` index tables.
//!
//! Index keys are structured, domain-separated, and length-prefixed to avoid
//! ambiguous concatenation and to enforce canonical formats.

use crate::db::{
    schema_codecs::{decode_object_id_be, encode_object_id_be},
    schema_keys::IndexTable,
};
use crate::{WalletError, WalletResult};

#[cfg(test)]
#[path = "test_index_codecs.rs"]
mod tests;

const INDEX_KEY_MAGIC: &[u8; 4] = b"zidx";

#[cfg(test)]
const TX_TIME_KEY_MAGIC: &[u8; 4] = b"ztim";
#[cfg(test)]
const TX_TIME_KEY_VERSION: u8 = 1;

const SEMANTIC_KEY_MAGIC: &[u8; 4] = b"zsem";

const SEMANTIC_KEY_VERSION: u8 = 1;

/// Recommended maximum size for index keys.
///
/// Keys should remain small and deterministic.
pub const MAX_INDEX_KEY_BYTES: usize = 256;

/// Recommended maximum size for index values.
///
/// Values should remain pointer-like; if larger, store as an object payload.
pub const MAX_INDEX_VALUE_BYTES: usize = 1024;

/// Maximum size of the semantic key portion inside an index key.
///
/// Since the full index key is capped at `MAX_INDEX_KEY_BYTES`, the semantic key
/// must fit after accounting for:
/// `magic(4) || table_tag(1) || semantic_len(2) || object_id_be(16)`.
pub const MAX_SEMANTIC_KEY_BYTES: usize = MAX_INDEX_KEY_BYTES - (4 + 1 + 2 + 16);

/// Typed wrapper for index key bytes.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct IndexKeyBytes(pub(crate) Vec<u8>);

impl IndexKeyBytes {
    pub fn new(table: IndexTable, key: Vec<u8>) -> WalletResult<Self> {
        let (decoded_table, semantic, object_id) = decode_index_key(&key)?;
        if decoded_table != table {
            return Err(WalletError::InvalidConfig(
                "index key table mismatch".to_string(),
            ));
        }

        let canonical = encode_index_key(decoded_table, &semantic, object_id)?;
        if canonical.as_slice() != key.as_slice() {
            return Err(WalletError::InvalidConfig(
                "index key is not canonical".to_string(),
            ));
        }

        Ok(Self(key))
    }
}

/// Typed wrapper for index value bytes.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct IndexValueBytes(pub(crate) Vec<u8>);

impl IndexValueBytes {
    pub(crate) fn new(value: Vec<u8>) -> WalletResult<Self> {
        if value.len() > MAX_INDEX_VALUE_BYTES {
            return Err(WalletError::InvalidConfig(
                "index value exceeds maximum size".to_string(),
            ));
        }
        Ok(Self(value))
    }

    pub(crate) fn from_object_id(object_id: u128) -> Self {
        Self(encode_object_id_be(object_id).to_vec())
    }
}

/// Encode a canonical index key.
///
/// Format:
/// `b"zidx" || table_tag(u8) || semantic_len(u16be) || semantic_key || object_id_be(16)`
pub fn encode_index_key(
    table: IndexTable,
    semantic_key: &[u8],
    object_id: u128,
) -> WalletResult<Vec<u8>> {
    if semantic_key.len() > (u16::MAX as usize) {
        return Err(WalletError::InvalidConfig(
            "index semantic key too large".to_string(),
        ));
    }

    let semantic_len = u16::try_from(semantic_key.len())
        .map_err(|_| WalletError::InvalidConfig("index semantic key too large".to_string()))?;

    let mut out = Vec::with_capacity(4 + 1 + 2 + semantic_key.len() + 16);
    out.extend_from_slice(INDEX_KEY_MAGIC);
    out.push(index_table_tag(table));
    out.extend_from_slice(&semantic_len.to_be_bytes());
    out.extend_from_slice(semantic_key);
    out.extend_from_slice(&encode_object_id_be(object_id));

    if out.len() > MAX_INDEX_KEY_BYTES {
        return Err(WalletError::InvalidConfig(
            "index key exceeds maximum size".to_string(),
        ));
    }

    Ok(out)
}

/// Encode a privacy-default index key where the semantic portion is keyed-hashed.
///
/// Format:
/// `b"zidx" || table_tag(u8) || semantic_len(u16be=32) || HMAC(INDEX_KEY, semantic_key || object_id_be) || object_id_be(16)`
///
/// This prevents plaintext semantic labels/statuses/timestamps from being inferable from index key bytes
/// without access to the derived `INDEX_KEY`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum IndexKeyMode {
    A,
}

pub fn encode_index_key_mode(
    index_key: &[u8; 32],
    mode: IndexKeyMode,
    table: IndexTable,
    semantic_key: &[u8],
    object_id: u128,
) -> WalletResult<Vec<u8>> {
    match mode {
        IndexKeyMode::A => encode_index_key_mode_a(index_key, table, semantic_key, object_id),
    }
}

pub fn encode_index_key_mode_a(
    index_key: &[u8; 32],
    table: IndexTable,
    semantic_key: &[u8],
    object_id: u128,
) -> WalletResult<Vec<u8>> {
    let object_id_be = encode_object_id_be(object_id);
    let mut msg = Vec::with_capacity(semantic_key.len() + object_id_be.len());
    msg.extend_from_slice(semantic_key);
    msg.extend_from_slice(&object_id_be);

    use crate::domains::hashing::compute_index_mac;
    let mac = compute_index_mac(index_key, &msg);
    encode_index_key(table, mac.as_slice(), object_id)
}

/// Validate index update payloads (key/value) before insertion.
pub fn validate_index_update(table: IndexTable, key: &[u8], value: &[u8]) -> WalletResult<()> {
    let (decoded_table, semantic, object_id) = decode_index_key(key)?;
    if decoded_table != table {
        return Err(WalletError::InvalidConfig(
            "index key table mismatch".to_string(),
        ));
    }

    let canonical = encode_index_key(decoded_table, &semantic, object_id)?;
    if canonical.as_slice() != key {
        return Err(WalletError::InvalidConfig(
            "index key is not canonical".to_string(),
        ));
    }

    if value.len() > MAX_INDEX_VALUE_BYTES {
        return Err(WalletError::InvalidConfig(
            "index value exceeds maximum size".to_string(),
        ));
    }

    Ok(())
}

/// Encode a canonical semantic index key as domain/field/value tuple.
///
/// Format:
/// `b"zsem" || version(u8) || domain_len(u16be) || domain || field_len(u16be) || field || value_len(u32be) || value`
pub fn encode_index_semantic_kv(domain: &str, field: &str, value: &[u8]) -> WalletResult<Vec<u8>> {
    let domain_bytes = domain.as_bytes();
    let field_bytes = field.as_bytes();

    if domain_bytes.is_empty() || field_bytes.is_empty() {
        return Err(WalletError::InvalidConfig(
            "index semantic key domain/field must be non-empty".to_string(),
        ));
    }

    let domain_len = u16::try_from(domain_bytes.len())
        .map_err(|_| WalletError::InvalidConfig("index semantic domain too large".to_string()))?;
    let field_len = u16::try_from(field_bytes.len())
        .map_err(|_| WalletError::InvalidConfig("index semantic field too large".to_string()))?;
    let value_len = u32::try_from(value.len())
        .map_err(|_| WalletError::InvalidConfig("index semantic value too large".to_string()))?;

    let total_len = 4 + 1 + 2 + domain_bytes.len() + 2 + field_bytes.len() + 4 + value.len();
    if total_len > MAX_SEMANTIC_KEY_BYTES {
        return Err(WalletError::InvalidConfig(
            "index semantic key exceeds maximum size".to_string(),
        ));
    }

    let mut out = Vec::with_capacity(total_len);
    out.extend_from_slice(SEMANTIC_KEY_MAGIC);
    out.push(SEMANTIC_KEY_VERSION);
    out.extend_from_slice(&domain_len.to_be_bytes());
    out.extend_from_slice(domain_bytes);
    out.extend_from_slice(&field_len.to_be_bytes());
    out.extend_from_slice(field_bytes);
    out.extend_from_slice(&value_len.to_be_bytes());
    out.extend_from_slice(value);
    Ok(out)
}

/// Validate that a semantic key is canonical according to `encode_index_semantic_kv`.
///
/// This is validated before applying keyed hashing, so callers cannot accidentally use
/// ad-hoc concatenation that would reduce collision resistance.
pub fn validate_index_semantic_key(semantic_key: &[u8]) -> WalletResult<()> {
    if semantic_key.len() < 4 + 1 + 2 + 2 + 4 {
        return Err(WalletError::InvalidConfig(
            "index semantic key is too short".to_string(),
        ));
    }
    if semantic_key.len() > MAX_SEMANTIC_KEY_BYTES {
        return Err(WalletError::InvalidConfig(
            "index semantic key exceeds maximum size".to_string(),
        ));
    }

    let (magic, rest) = semantic_key.split_at(4);
    if magic != SEMANTIC_KEY_MAGIC {
        return Err(WalletError::InvalidConfig(
            "index semantic key has invalid magic".to_string(),
        ));
    }
    if rest.is_empty() {
        return Err(WalletError::InvalidConfig(
            "index semantic key is too short".to_string(),
        ));
    }
    if rest[0] != SEMANTIC_KEY_VERSION {
        return Err(WalletError::InvalidConfig(
            "index semantic key has unsupported version".to_string(),
        ));
    }

    let mut offset = 1;
    if rest.len() < offset + 2 {
        return Err(WalletError::InvalidConfig(
            "index semantic key is too short".to_string(),
        ));
    }
    let domain_len = u16::from_be_bytes([rest[offset], rest[offset + 1]]) as usize;
    offset += 2;
    if rest.len() < offset + domain_len + 2 {
        return Err(WalletError::InvalidConfig(
            "index semantic key length mismatch".to_string(),
        ));
    }
    let domain = &rest[offset..offset + domain_len];
    offset += domain_len;

    let field_len = u16::from_be_bytes([rest[offset], rest[offset + 1]]) as usize;
    offset += 2;
    if rest.len() < offset + field_len + 4 {
        return Err(WalletError::InvalidConfig(
            "index semantic key length mismatch".to_string(),
        ));
    }
    let field = &rest[offset..offset + field_len];
    offset += field_len;

    let value_len = u32::from_be_bytes([
        rest[offset],
        rest[offset + 1],
        rest[offset + 2],
        rest[offset + 3],
    ]) as usize;
    offset += 4;
    if rest.len() != offset + value_len {
        return Err(WalletError::InvalidConfig(
            "index semantic key length mismatch".to_string(),
        ));
    }

    if domain.is_empty() || field.is_empty() {
        return Err(WalletError::InvalidConfig(
            "index semantic key domain/field must be non-empty".to_string(),
        ));
    }

    Ok(())
}

/// Parse and validate a canonical index key.
///
/// Returns `(table, semantic_key, object_id)`.
pub fn decode_index_key(key: &[u8]) -> WalletResult<(IndexTable, Vec<u8>, u128)> {
    if key.len() < 4 + 1 + 2 + 16 {
        return Err(WalletError::InvalidConfig(
            "index key is too short".to_string(),
        ));
    }
    if key.len() > MAX_INDEX_KEY_BYTES {
        return Err(WalletError::InvalidConfig(
            "index key exceeds maximum size".to_string(),
        ));
    }

    let (magic, rest) = key.split_at(4);
    if magic != INDEX_KEY_MAGIC {
        return Err(WalletError::InvalidConfig(
            "index key has invalid magic".to_string(),
        ));
    }

    let table_tag = rest[0];
    let table = index_table_from_tag(table_tag)?;

    let semantic_len = u16::from_be_bytes([rest[1], rest[2]]) as usize;
    let expected_len = 1 + 2 + semantic_len + 16;
    if rest.len() != expected_len {
        return Err(WalletError::InvalidConfig(
            "index key length mismatch".to_string(),
        ));
    }

    let semantic_start = 3;
    let semantic_end = semantic_start + semantic_len;
    let semantic = rest[semantic_start..semantic_end].to_vec();

    let object_id_bytes = &rest[semantic_end..];
    let object_id = decode_object_id_be(object_id_bytes)?;

    Ok((table, semantic, object_id))
}

#[inline]
fn index_table_tag(table: IndexTable) -> u8 {
    match table {
        IndexTable::AccountByLabel => 1,
        IndexTable::ReceiverByKind => 2,
        IndexTable::AssetDefBySymbol => 3,
        IndexTable::AssetOutByDef => 4,
        IndexTable::AssetOutBySpentFlag => 5,
        IndexTable::TrackedAssetBySpentFlag => 6,
        IndexTable::TxByStatus => 7,
        IndexTable::TxByTime => 8,
        IndexTable::PendingByStatusExpiry => 9,
        IndexTable::ReceiptByTxHash => 10,
        IndexTable::WalletByWalletId => 11,
        IndexTable::OwnedAssetById => 12,
        IndexTable::OwnedAssetByDefStatus => 13,
        IndexTable::OwnedAssetByStatus => 14,
        IndexTable::OwnedAssetByTx => 15,
        IndexTable::OwnedAssetByScan => 16,
        IndexTable::OwnedObjectByFamily => 17,
        IndexTable::OwnedObjectByStatus => 18,
        IndexTable::OwnedObjectByPolicy => 19,
        IndexTable::OwnedObjectByHolder => 20,
        IndexTable::OwnedVoucherById => 21,
        IndexTable::OwnedRightById => 22,
    }
}

fn index_table_from_tag(tag: u8) -> WalletResult<IndexTable> {
    match tag {
        1 => Ok(IndexTable::AccountByLabel),
        2 => Ok(IndexTable::ReceiverByKind),
        3 => Ok(IndexTable::AssetDefBySymbol),
        4 => Ok(IndexTable::AssetOutByDef),
        5 => Ok(IndexTable::AssetOutBySpentFlag),
        6 => Ok(IndexTable::TrackedAssetBySpentFlag),
        7 => Ok(IndexTable::TxByStatus),
        8 => Ok(IndexTable::TxByTime),
        9 => Ok(IndexTable::PendingByStatusExpiry),
        10 => Ok(IndexTable::ReceiptByTxHash),
        11 => Ok(IndexTable::WalletByWalletId),
        12 => Ok(IndexTable::OwnedAssetById),
        13 => Ok(IndexTable::OwnedAssetByDefStatus),
        14 => Ok(IndexTable::OwnedAssetByStatus),
        15 => Ok(IndexTable::OwnedAssetByTx),
        16 => Ok(IndexTable::OwnedAssetByScan),
        17 => Ok(IndexTable::OwnedObjectByFamily),
        18 => Ok(IndexTable::OwnedObjectByStatus),
        19 => Ok(IndexTable::OwnedObjectByPolicy),
        20 => Ok(IndexTable::OwnedObjectByHolder),
        21 => Ok(IndexTable::OwnedVoucherById),
        22 => Ok(IndexTable::OwnedRightById),
        _ => Err(WalletError::InvalidConfig(
            "index key has unknown table tag".to_string(),
        )),
    }
}
