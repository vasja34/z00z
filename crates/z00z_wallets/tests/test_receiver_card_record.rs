use z00z_wallets::{
    chain::{verify_receiver_card_record, ReceiverCardRecord},
    key::{ReceiverKeys, ReceiverSecret},
    receiver::{receiver_card::encode_card_compact, ReceiverCard},
};

fn make_card() -> ReceiverCard {
    let secret = ReceiverSecret::generate().expect("secret");
    let keys = ReceiverKeys::from_receiver_secret(secret).expect("keys");
    keys.export_receiver_card().expect("card")
}

#[test]
fn test_record_gate_only() {
    let card = make_card();
    let record = ReceiverCardRecord::new(&card, card.canonical_encoding(), 1).expect("record");
    let compact = record.to_compact().expect("compact");

    let verified = verify_receiver_card_record(&record, None).expect("verified");
    assert_eq!(verified.owner_handle, card.owner_handle);

    let decoded = ReceiverCardRecord::from_compact(&compact, None)
        .expect("record compact")
        .decode_card()
        .expect("record card");
    assert_eq!(decoded.owner_handle, card.owner_handle);

    let compact_card_bytes = encode_card_compact(&card);
    assert!(ReceiverCardRecord::from_compact(&compact_card_bytes, None).is_err());

    let mut bad = record.clone();
    bad.registry_entry_id[0] ^= 0x01;
    assert!(verify_receiver_card_record(&bad, None).is_err());
}

#[test]
fn test_record_canonical_live_contract() {
    let card = make_card();
    let record = ReceiverCardRecord::new(&card, card.canonical_encoding(), 7).expect("record");
    let compact = record.to_compact().expect("compact");

    let verified = ReceiverCardRecord::from_compact(&compact, None).expect("roundtrip");
    assert_eq!(verified.version, 1);
    assert_eq!(verified.receiver_card_bytes, card.canonical_encoding());
}
