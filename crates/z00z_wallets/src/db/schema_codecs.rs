//! Canonical schema codecs for `.wlt` persistence.

use serde::{Deserialize, Serialize};
use z00z_utils::codec::{BincodeCodec, Codec};

use crate::{WalletError, WalletResult};

/// Encode an object id as a big-endian 16-byte key.
#[inline]
pub fn encode_object_id_be(object_id: u128) -> [u8; 16] {
    object_id.to_be_bytes()
}

/// Decode an object id from a big-endian 16-byte key.
pub fn decode_object_id_be(bytes: &[u8]) -> WalletResult<u128> {
    let bytes: [u8; 16] = bytes
        .try_into()
        .map_err(|_| WalletError::InvalidConfig("object id bytes must be 16 bytes".to_string()))?;
    Ok(u128::from_be_bytes(bytes))
}

/// Encode an objects-table value.
///
/// Values are stored as bincode-encoded records.
pub fn encode_encrypted_object_record<T: Serialize>(record: &T) -> WalletResult<Vec<u8>> {
    let codec = BincodeCodec;
    codec
        .serialize(record)
        .map_err(|_| WalletError::InvalidConfig("object record encode failed".to_string()))
}

/// Decode an objects-table value.
///
/// Values are stored as bincode-encoded records.
pub fn decode_encrypted_object_record<T: for<'de> Deserialize<'de>>(
    bytes: &[u8],
) -> WalletResult<T> {
    let codec = BincodeCodec;
    codec
        .deserialize(bytes)
        .map_err(|_| WalletError::InvalidConfig("object record decode failed".to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_object_id_roundtrip_r805() {
        let original = 123456789012345678901234567890123456u128;
        let bytes = encode_object_id_be(original);
        let decoded = decode_object_id_be(&bytes).unwrap();
        assert_eq!(decoded, original);
    }

    #[test]
    fn test_object_id_size_r861() {
        let err = decode_object_id_be(&[1, 2, 3]).unwrap_err();
        assert!(matches!(err, crate::WalletError::InvalidConfig(_)));
    }

    #[test]
    fn test_encrypted_bincode_roundtrip_r807() {
        #[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
        struct DummyRecord {
            a: u32,
            b: Vec<u8>,
        }

        let record = DummyRecord {
            a: 42,
            b: vec![1, 2, 3],
        };
        let bytes = encode_encrypted_object_record(&record).unwrap();
        let decoded: DummyRecord = decode_encrypted_object_record(&bytes).unwrap();
        assert_eq!(decoded, record);
    }
}
