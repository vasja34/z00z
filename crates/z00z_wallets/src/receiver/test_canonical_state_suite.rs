use super::*;
use std::str::FromStr;

#[test]
fn test_path_encode_decode() {
    let path = Bip44Path::from_str("m/44'/1337'/0'/0/0").unwrap();
    let bytes = path_to_bytes(&path);
    let decoded = path_from_bytes(&bytes).unwrap();
    assert_eq!(path, decoded);
}

#[test]
fn test_canonical_roundtrip() {
    let path = Bip44Path::from_str("m/44'/1337'/0'/0/5").unwrap();
    let spend = vec![1u8; 32];
    let view = vec![2u8; 32];

    let entries = vec![(path, spend.clone(), view.clone())];
    let bytes = to_canonical(&entries).unwrap();
    let decoded = from_canonical(&bytes).unwrap();

    assert_eq!(decoded.len(), 1);
    assert_eq!(decoded[0].0, path);
    assert_eq!(decoded[0].1, spend);
    assert_eq!(decoded[0].2, view);
}

#[test]
fn test_version_mismatch() {
    let bytes = vec![99u8, 0, 0, 0, 0];
    let result = from_canonical(&bytes);
    assert!(matches!(
        result,
        Err(CanonicalStateError::InvalidVersion { .. })
    ));
}

#[test]
fn test_too_short() {
    let bytes = vec![1u8, 0, 0];
    let result = from_canonical(&bytes);
    assert!(matches!(
        result,
        Err(CanonicalStateError::TooShort { .. })
    ));
}

#[test]
fn test_length_mismatch() {
    let mut bytes = vec![STATE_VERSION];
    bytes.extend_from_slice(&1u32.to_le_bytes());
    bytes.extend_from_slice(&[0u8; 10]);

    let result = from_canonical(&bytes);
    assert!(matches!(
        result,
        Err(CanonicalStateError::InvalidLength { .. })
    ));
}

#[test]
fn test_invalid_spend_key_length() {
    let path = Bip44Path::from_str("m/44'/1337'/0'/0/0").unwrap();
    let bad_spend = vec![0u8; 31];
    let view = vec![0u8; 32];
    let result = to_canonical(&[(path, bad_spend, view)]);
    assert!(matches!(
        result,
        Err(CanonicalStateError::InvalidKeyLength {
            entry_index: 0,
            key_type: "spend_pk",
            actual: 31
        })
    ));
}

#[test]
fn test_invalid_view_key_length() {
    let path = Bip44Path::from_str("m/44'/1337'/0'/0/0").unwrap();
    let spend = vec![0u8; 32];
    let bad_view = vec![0u8; 33];
    let result = to_canonical(&[(path, spend, bad_view)]);
    assert!(matches!(
        result,
        Err(CanonicalStateError::InvalidKeyLength {
            entry_index: 0,
            key_type: "view_pk",
            actual: 33
        })
    ));
}

#[test]
fn test_multiple_entries_roundtrip() {
    let entries = vec![
        (
            Bip44Path::from_str("m/44'/1337'/0'/0/0").unwrap(),
            vec![1u8; 32],
            vec![2u8; 32],
        ),
        (
            Bip44Path::from_str("m/44'/1337'/0'/1/42").unwrap(),
            vec![3u8; 32],
            vec![4u8; 32],
        ),
    ];

    let bytes = to_canonical(&entries).unwrap();
    let decoded = from_canonical(&bytes).unwrap();
    assert_eq!(decoded, entries);
}