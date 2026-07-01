use super::*;
use z00z_utils::codec::{Codec, JsonCodec};

#[test]
fn test_pagination_serialization() {
    let params = RuntimePaginationParams {
        limit: Some(10),
        cursor: Some("cursor123".to_string()),
        include_total: Some(true),
    };
    let codec = JsonCodec;
    let bytes = codec.serialize(&params).unwrap();
    let json = String::from_utf8(bytes).unwrap();
    assert!(json.contains("limit"));
    assert!(json.contains("cursor"));
}

#[test]
fn test_operation_status_flatten() {
    let status = RuntimeOperationStatusWithTx {
        status: RuntimeOperationStatus {
            success: true,
            message: "OK".to_string(),
        },
        tx_id: PersistTxId("tx123".to_string()),
    };
    let codec = JsonCodec;
    let bytes = codec.serialize(&status).unwrap();
    let json = String::from_utf8(bytes).unwrap();
    assert!(json.contains("success"));
    assert!(json.contains("tx_id"));
}

#[test]
fn test_asset_ref_flatten() {
    let asset_amount = RuntimeAssetAmount {
        asset: RuntimeAssetRef {
            asset_id: [0u8; 32],
            serial_id: 1,
            symbol: "TEST".to_string(),
            class: AssetClass::Token,
        },
        amount: 1000,
    };
    let codec = JsonCodec;
    let bytes = codec.serialize(&asset_amount).unwrap();
    let json = String::from_utf8(bytes).unwrap();
    assert!(json.contains("asset_id"));
    assert!(json.contains("amount"));
}

#[test]
fn test_job_state_enum() {
    let state = JobState::Running;
    let codec = JsonCodec;
    let bytes = codec.serialize(&state).unwrap();
    let json = String::from_utf8(bytes).unwrap();
    assert_eq!(json, "\"Running\"");
}

#[test]
fn test_validation_result() {
    let valid =
        RuntimeValidationResult::valid_with_warnings(vec!["entropy heuristic warning".to_string()]);
    let codec = JsonCodec;
    let bytes = codec.serialize(&valid).unwrap();
    let json = String::from_utf8(bytes).unwrap();
    assert!(json.contains("valid"));
    assert!(json.contains("warnings"));

    let invalid = RuntimeValidationResult::invalid("Invalid receiver");
    let bytes = codec.serialize(&invalid).unwrap();
    let json = String::from_utf8(bytes).unwrap();
    assert!(json.contains("error"));
    assert!(!json.contains("warnings"));
}

#[test]
fn test_encrypted_response_serialization() {
    let encrypted = RuntimeEncryptedResponse {
        ciphertext: "0xabcdef123456".to_string(),
        metadata: RuntimeEncryptionMetadata {
            algorithm: "xchacha20poly1305".to_string(),
            nonce: "0x123456789abc".to_string(),
            key_derivation: "HKDF-SHA256".to_string(),
        },
    };

    let codec = JsonCodec;
    let bytes = codec.serialize(&encrypted).unwrap();
    let json = String::from_utf8(bytes).unwrap();

    assert!(json.contains("ciphertext"));
    assert!(json.contains("metadata"));

    let decoded: RuntimeEncryptedResponse = codec.deserialize(json.as_bytes()).unwrap();
    assert_eq!(decoded.ciphertext, "0xabcdef123456");
    assert_eq!(decoded.metadata.algorithm, "xchacha20poly1305");
    assert_eq!(decoded.metadata.nonce, "0x123456789abc");
}

#[test]
fn test_encrypted_response_stub() {
    let encrypted = RuntimeEncryptedResponse::stub("my_secret_seed_phrase");

    assert!(encrypted.ciphertext.contains("encrypted:"));
    assert_eq!(encrypted.metadata.algorithm, "xchacha20poly1305");
    assert!(!encrypted.metadata.nonce.is_empty());
}

#[test]
fn test_encrypted_response_is_encrypted() {
    let encrypted = RuntimeEncryptedResponse {
        ciphertext: "0xabcd".to_string(),
        metadata: RuntimeEncryptionMetadata {
            algorithm: "xchacha20poly1305".to_string(),
            nonce: "0x123".to_string(),
            key_derivation: "HKDF-SHA256".to_string(),
        },
    };

    assert!(encrypted.is_encrypted());
}
