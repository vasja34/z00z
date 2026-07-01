use super::*;

fn encode_tx_time_index_key(timestamp_ms: u64, object_id: u128) -> Vec<u8> {
    let mut out = Vec::with_capacity(4 + 1 + 8 + 16);
    out.extend_from_slice(TX_TIME_KEY_MAGIC);
    out.push(TX_TIME_KEY_VERSION);
    out.extend_from_slice(&timestamp_ms.to_be_bytes());
    out.extend_from_slice(&encode_object_id_be(object_id));
    out
}

fn decode_tx_time_index_key(key: &[u8]) -> WalletResult<(u64, u128)> {
    if key.len() != 4 + 1 + 8 + 16 {
        return Err(WalletError::InvalidConfig(
            "tx time index key length mismatch".to_string(),
        ));
    }

    let (magic, rest) = key.split_at(4);
    if magic != TX_TIME_KEY_MAGIC {
        return Err(WalletError::InvalidConfig(
            "tx time index key has invalid magic".to_string(),
        ));
    }
    if rest[0] != TX_TIME_KEY_VERSION {
        return Err(WalletError::InvalidConfig(
            "tx time index key has unsupported version".to_string(),
        ));
    }

    let timestamp_ms = u64::from_be_bytes([
        rest[1], rest[2], rest[3], rest[4], rest[5], rest[6], rest[7], rest[8],
    ]);
    let object_id = decode_object_id_be(&rest[9..])?;
    Ok((timestamp_ms, object_id))
}

fn encode_view_pk(view_pk: &[u8; 32], object_id: u128) -> WalletResult<Vec<u8>> {
    let semantic = encode_index_semantic_kv("idx:view_pk", "view_pk", view_pk)?;
    encode_index_key(IndexTable::ReceiverByKind, &semantic, object_id)
}

fn decode_view_pk(key: &[u8]) -> WalletResult<(IndexTable, Vec<u8>, u128)> {
    decode_index_key(key)
}

#[test]
fn test_index_key_roundtrip() {
    let semantic = encode_index_semantic_kv("idx:label", "label", b"Main").unwrap();
    let object_id = 42u128;
    let key = encode_index_key(IndexTable::AccountByLabel, &semantic, object_id).unwrap();

    let (table, decoded_semantic, decoded_object_id) = decode_index_key(&key).unwrap();
    assert!(matches!(table, IndexTable::AccountByLabel));
    assert_eq!(decoded_semantic, semantic);
    assert_eq!(decoded_object_id, object_id);
}

#[test]
fn test_eq_view_pk_codec() {
    let view_pk = [0x11u8; 32];
    let object_id = 7u128;
    let semantic = encode_index_semantic_kv("idx:view_pk", "view_pk", &view_pk).unwrap();

    let direct = encode_index_key(IndexTable::ReceiverByKind, &semantic, object_id).unwrap();
    let via_spec = encode_view_pk(&view_pk, object_id).unwrap();
    assert_eq!(direct, via_spec);

    let direct_decoded = decode_index_key(&direct).unwrap();
    let via_spec_decoded = decode_view_pk(&via_spec).unwrap();
    assert_eq!(direct_decoded, via_spec_decoded);
}

#[test]
fn test_index_key_bad_magic() {
    let err = decode_index_key(b"nope").unwrap_err();
    assert!(matches!(err, WalletError::InvalidConfig(_)));
}

#[test]
fn test_index_key_length_mismatch() {
    let semantic = encode_index_semantic_kv("idx:label", "label", b"Main").unwrap();
    let object_id = 42u128;
    let mut key = encode_index_key(IndexTable::AccountByLabel, &semantic, object_id).unwrap();
    key[5] = 0;
    key[6] = 1;

    let err = decode_index_key(&key).unwrap_err();
    assert!(matches!(err, WalletError::InvalidConfig(_)));
}

#[test]
fn test_index_update_rejects_table() {
    let semantic = encode_index_semantic_kv("idx:label", "label", b"Main").unwrap();
    let object_id = 42u128;
    let key = encode_index_key(IndexTable::AccountByLabel, &semantic, object_id).unwrap();
    let err = validate_index_update(IndexTable::TxByStatus, &key, &[1]).unwrap_err();
    assert!(matches!(err, WalletError::InvalidConfig(_)));
}

#[test]
fn test_semantic_key_collision_resistant() {
    let naive_1 = [b"ab".as_slice(), b"c".as_slice()].concat();
    let naive_2 = [b"a".as_slice(), b"bc".as_slice()].concat();
    assert_eq!(naive_1, naive_2);

    let k1 = encode_index_semantic_kv("idx:test", "ab", b"c").unwrap();
    let k2 = encode_index_semantic_kv("idx:test", "a", b"bc").unwrap();
    assert_ne!(k1, k2);
}

#[test]
fn test_index_value_bounds_canon() {
    let semantic = encode_index_semantic_kv("idx:label", "label", b"Main").unwrap();
    let object_id = 42u128;
    let mut key = encode_index_key(IndexTable::AccountByLabel, &semantic, object_id).unwrap();
    key[0] = b'x';

    let err = IndexKeyBytes::new(IndexTable::AccountByLabel, key).unwrap_err();
    assert!(matches!(err, WalletError::InvalidConfig(_)));

    let value = vec![0u8; MAX_INDEX_VALUE_BYTES + 1];
    let err = IndexValueBytes::new(value).unwrap_err();
    assert!(matches!(err, WalletError::InvalidConfig(_)));
}

#[test]
fn test_index_hmac_hides_plaintext() {
    let index_key = [7u8; 32];
    let semantic = encode_index_semantic_kv("idx:label", "label", b"Main").unwrap();
    let object_id = 42u128;
    let key = encode_index_key_mode_a(&index_key, IndexTable::AccountByLabel, &semantic, object_id)
        .unwrap();

    let key_2 = encode_index_key_mode_a(
        &index_key,
        IndexTable::AccountByLabel,
        &semantic,
        object_id + 1,
    )
    .unwrap();
    assert_ne!(key, key_2);

    assert!(!key.windows(4).any(|w| w == b"Main"));
    assert!(!key.windows(4).any(|w| w == SEMANTIC_KEY_MAGIC));
    assert!(!key.windows(9).any(|w| w == b"idx:label"));
    assert!(!key.windows(5).any(|w| w == b"label"));
    assert!(!key
        .windows(semantic.len())
        .any(|w| w == semantic.as_slice()));
}

#[test]
fn test_validate_semantic_accepts_canonical() {
    let semantic = encode_index_semantic_kv("idx:test", "field", b"value").unwrap();
    validate_index_semantic_key(&semantic).unwrap();
}

#[test]
fn test_semantic_key_bad_magic() {
    let mut semantic = encode_index_semantic_kv("idx:test", "field", b"value").unwrap();
    semantic[0] = b'x';
    let err = validate_index_semantic_key(&semantic).unwrap_err();
    assert!(matches!(err, WalletError::InvalidConfig(_)));
}

#[test]
fn test_semantic_key_bad_version() {
    let mut semantic = encode_index_semantic_kv("idx:test", "field", b"value").unwrap();
    semantic[4] = 2;
    let err = validate_index_semantic_key(&semantic).unwrap_err();
    assert!(matches!(err, WalletError::InvalidConfig(_)));
}

#[test]
fn test_tx_time_key_roundtrip() {
    let ts = 1234u64;
    let object_id = 999u128;
    let key = encode_tx_time_index_key(ts, object_id);
    let (decoded_ts, decoded_object_id) = decode_tx_time_index_key(&key).unwrap();
    assert_eq!(decoded_ts, ts);
    assert_eq!(decoded_object_id, object_id);
}
