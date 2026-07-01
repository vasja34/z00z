use super::*;
use z00z_utils::codec::{Codec, JsonCodec};

#[test]
fn test_event_history_cursor_string() {
    let response: EventHistoryResponse = RuntimePaginatedResponse {
        items: Vec::new(),
        next_cursor: Some(encode_event_cursor_ms(123)),
        has_more: true,
        total_count: None,
    };

    let codec = JsonCodec;
    let bytes = codec.serialize(&response).unwrap();
    let json = String::from_utf8(bytes).unwrap();

    assert!(json.contains("\"items\""));
    assert!(json.contains("\"next_cursor\""));
    assert_eq!(decode_event_cursor_ms("123"), Some(123));
}

#[test]
fn test_sync_progress_is_f32() {
    let event = SyncEvent::Progress {
        wallet_id: PersistWalletId("w".to_string()),
        current_height: 1,
        target_height: 10,
        progress: Some(0.5),
        eta_seconds: Some(2),
        timestamp: 1,
    };

    if let SyncEvent::Progress { progress, .. } = event {
        assert_eq!(progress, Some(0.5));
    }
}
