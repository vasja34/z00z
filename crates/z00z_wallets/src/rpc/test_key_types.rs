use super::*;
use z00z_utils::codec::{Codec, JsonCodec};

#[test]
fn test_runtime_key_derive_response() {
    let response = RuntimeKeyDeriveResponse {
        public_key: "0xabc123".to_string(),
    };

    let codec = JsonCodec;
    let bytes = codec.serialize(&response).unwrap();
    let deserialized: RuntimeKeyDeriveResponse = codec.deserialize(&bytes).unwrap();

    assert_eq!(deserialized.public_key, "0xabc123");
}

#[test]
fn test_runtime_export_key_response() {
    let response = RuntimeExportPublicKeyResponse {
        public_key: "0xdef456".to_string(),
    };

    let codec = JsonCodec;
    let bytes = codec.serialize(&response).unwrap();
    let deserialized: RuntimeExportPublicKeyResponse = codec.deserialize(&bytes).unwrap();

    assert_eq!(deserialized.public_key, "0xdef456");
}

#[test]
fn test_runtime_derive_key_response() {
    let response = RuntimeDeriveReceiverResponse {
        public_key: "0x123".to_string(),
        path: "m/44'/0'/0'/0/0".to_string(),
    };

    let codec = JsonCodec;
    let bytes = codec.serialize(&response).unwrap();
    let deserialized: RuntimeDeriveReceiverResponse = codec.deserialize(&bytes).unwrap();

    assert_eq!(deserialized.public_key, "0x123");
    assert_eq!(deserialized.path, "m/44'/0'/0'/0/0");
}

#[test]
fn test_runtime_material_export_response() {
    let response = RuntimePubMaterialExportResponse {
        schema_version: 1,
        encrypted_pub_material: "base64data".to_string(),
        algorithm: "XChaCha20-Poly1305".to_string(),
        account: 0,
        fingerprint: "fingerprint123".to_string(),
    };

    let codec = JsonCodec;
    let bytes = codec.serialize(&response).unwrap();
    let deserialized: RuntimePubMaterialExportResponse = codec.deserialize(&bytes).unwrap();

    assert_eq!(deserialized.schema_version, 1);
    assert_eq!(deserialized.algorithm, "XChaCha20-Poly1305");
    assert_eq!(deserialized.account, 0);
}

#[test]
fn test_runtime_receiver_card_response() {
    let response = RuntimeGetReceiverCardResponse {
        owner_handle: "aa".repeat(32),
        view_key: "bb".repeat(32),
        identity_key: "cc".repeat(32),
        signature: "dd".repeat(64),
        card_compact: "card_compact_payload".to_string(),
        registry_entry_id: "ee".repeat(32),
        card_epoch: 0,
        owner_handle_display: "z00z1testaddress".to_string(),
    };

    let codec = JsonCodec;
    let bytes = codec.serialize(&response).unwrap();
    let decoded: RuntimeGetReceiverCardResponse = codec.deserialize(&bytes).unwrap();

    assert_eq!(decoded.owner_handle, "aa".repeat(32));
    assert_eq!(decoded.view_key, "bb".repeat(32));
    assert_eq!(decoded.identity_key, "cc".repeat(32));
    assert_eq!(decoded.signature, "dd".repeat(64));
    assert_eq!(decoded.card_compact, "card_compact_payload");
    assert_eq!(decoded.registry_entry_id, "ee".repeat(32));
    assert_eq!(decoded.card_epoch, 0);
    assert_eq!(decoded.owner_handle_display, "z00z1testaddress");
}

#[test]
fn test_create_payment_request_response() {
    let response = RuntimeCreatePaymentRequestResponse {
        owner_handle: "aa".repeat(32),
        view_key: "bb".repeat(32),
        identity_key: "cc".repeat(32),
        req_id: "dd".repeat(32),
        chain_id: 3,
        amount: Some(42),
        expiry: 1_700_000_000,
        signature: "ee".repeat(64),
        request_compact: "request_compact_payload".to_string(),
    };

    let codec = JsonCodec;
    let bytes = codec.serialize(&response).unwrap();
    let decoded: RuntimeCreatePaymentRequestResponse = codec.deserialize(&bytes).unwrap();

    assert_eq!(decoded.owner_handle, "aa".repeat(32));
    assert_eq!(decoded.view_key, "bb".repeat(32));
    assert_eq!(decoded.identity_key, "cc".repeat(32));
    assert_eq!(decoded.req_id, "dd".repeat(32));
    assert_eq!(decoded.chain_id, 3);
    assert_eq!(decoded.amount, Some(42));
    assert_eq!(decoded.expiry, 1_700_000_000);
    assert_eq!(decoded.signature, "ee".repeat(64));
    assert_eq!(decoded.request_compact, "request_compact_payload");
}

#[test]
fn test_validate_payment_request_response() {
    let response = RuntimeValidatePaymentRequestResponse {
        result: RuntimeValidationResult::valid(),
        outcome: Some("approved".to_string()),
        req_id: Some("aa".repeat(32)),
        owner_handle: Some("bb".repeat(32)),
        expiry: Some(1_700_000_000),
    };

    let codec = JsonCodec;
    let bytes = codec.serialize(&response).unwrap();
    let decoded: RuntimeValidatePaymentRequestResponse = codec.deserialize(&bytes).unwrap();

    assert!(decoded.result.valid);
    assert_eq!(decoded.outcome, Some("approved".to_string()));
    assert_eq!(decoded.req_id, Some("aa".repeat(32)));
    assert_eq!(decoded.owner_handle, Some("bb".repeat(32)));
    assert_eq!(decoded.expiry, Some(1_700_000_000));
}

#[test]
fn test_runtime_rotate_key_response() {
    let response = RuntimeRotateKeyResponse {
        new_fingerprint: "new_fp".to_string(),
        rotated_at: 1_700_000_000_000,
        records_rewrapped: 42,
    };

    let codec = JsonCodec;
    let bytes = codec.serialize(&response).unwrap();
    let deserialized: RuntimeRotateKeyResponse = codec.deserialize(&bytes).unwrap();

    assert_eq!(deserialized.new_fingerprint, "new_fp");
    assert_eq!(deserialized.rotated_at, 1_700_000_000_000);
    assert_eq!(deserialized.records_rewrapped, 42);
}

#[test]
fn test_runtime_receiver_filter() {
    let filter = RuntimeReceiverFilter {
        used: Some(true),
        change: Some(false),
    };

    let codec = JsonCodec;
    let bytes = codec.serialize(&filter).unwrap();
    let deserialized: RuntimeReceiverFilter = codec.deserialize(&bytes).unwrap();

    assert_eq!(deserialized.used, Some(true));
    assert_eq!(deserialized.change, Some(false));
}

#[test]
fn test_runtime_receiver_filter_none() {
    let filter = RuntimeReceiverFilter {
        used: None,
        change: None,
    };

    let codec = JsonCodec;
    let bytes = codec.serialize(&filter).unwrap();
    let deserialized: RuntimeReceiverFilter = codec.deserialize(&bytes).unwrap();

    assert!(deserialized.used.is_none());
    assert!(deserialized.change.is_none());
}

#[test]
fn test_runtime_list_receivers_response() {
    let response = RuntimeListReceiversResponse {
        items: vec![PersistReceiverInfo {
            receiver_id: "aa".repeat(32),
            path: "m/44'/1337'/0'/0/0".to_string(),
            public_key: "bb".repeat(32),
            balance: None,
            used: false,
            internal: false,
            label: Some("Primary".to_string()),
            index: 0,
        }],
        next_cursor: None,
        has_more: false,
        total_count: Some(1),
    };

    let codec = JsonCodec;
    let bytes = codec.serialize(&response).unwrap();
    let decoded: RuntimeListReceiversResponse = codec.deserialize(&bytes).unwrap();

    assert_eq!(decoded.items.len(), 1);
    assert_eq!(decoded.items[0].receiver_id, "aa".repeat(32));
    assert_eq!(decoded.items[0].public_key, "bb".repeat(32));
    assert_eq!(decoded.items[0].label.as_deref(), Some("Primary"));
}

#[test]
fn test_runtime_label_receiver_response() {
    let response = RuntimeLabelReceiverResponse {
        receiver_id: "aa".repeat(32),
        label: "Receiver Label".to_string(),
        status: RuntimeOperationStatus {
            success: true,
            message: "ok".to_string(),
        },
    };

    let codec = JsonCodec;
    let bytes = codec.serialize(&response).unwrap();
    let decoded: RuntimeLabelReceiverResponse = codec.deserialize(&bytes).unwrap();

    assert_eq!(decoded.receiver_id, "aa".repeat(32));
    assert_eq!(decoded.label, "Receiver Label");
    assert!(decoded.status.success);
}
