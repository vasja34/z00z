use z00z_wallets::{
    chain::{
        receiver_card_record::{check_relabel, CardRecordError},
        verify_receiver_card_record, ReceiverCardRecord,
    },
    key::{ReceiverKeys, ReceiverSecret},
    receiver::{ReceiverCard, ReceiverCardError},
};

fn make_keys() -> ReceiverKeys {
    let secret = ReceiverSecret::generate().expect("secret");
    ReceiverKeys::from_receiver_secret(secret).expect("keys")
}

fn make_card() -> ReceiverCard {
    let keys = make_keys();
    let card = keys.export_receiver_card().expect("card");
    card.verify().expect("verified card");
    card
}

fn make_record(epoch: u64) -> ReceiverCardRecord {
    let card = make_card();
    ReceiverCardRecord::new(&card, card.canonical_encoding(), epoch).expect("record")
}

fn check_verify_ok(record: &ReceiverCardRecord) {
    let card = verify_receiver_card_record(record, None).expect("verified record");
    assert_eq!(card.canonical_encoding(), record.receiver_card_bytes);
}

fn check_compact_ok(record: &ReceiverCardRecord) {
    let _ = verify_receiver_card_record(record, None).expect("verified record");
    let compact = record.to_compact().expect("compact");
    let roundtrip = ReceiverCardRecord::from_compact(&compact, None).expect("roundtrip");
    assert_eq!(roundtrip, *record);
    assert_eq!(roundtrip.to_compact().expect("same compact"), compact);
}

#[test]
fn test_s5_record_flow() {
    let record = make_record(7);
    let card = record.decode_card().expect("card");
    let mut bad_payload = card.canonical_encoding();
    let last = bad_payload.len() - 1;
    bad_payload[last] ^= 0x01;

    check_verify_ok(&record);
    check_compact_ok(&record);
    assert!(matches!(
        ReceiverCardRecord::new(&card, bad_payload, 7),
        Err(CardRecordError::InvalidCardBytes)
    ));

    let rotated = ReceiverCardRecord::from_compact(&record.to_compact().expect("compact"), None)
        .expect("record");
    assert_eq!(rotated, record);
}

#[test]
fn test_s5_record_errs() {
    let record = make_record(9);

    let mut bad_ver = record.clone();
    bad_ver.version = 9;
    assert!(matches!(
        verify_receiver_card_record(&bad_ver, None),
        Err(CardRecordError::UnsupportedVersion)
    ));
    assert!(matches!(
        bad_ver.to_compact(),
        Err(CardRecordError::UnsupportedVersion)
    ));

    let mut bad_bytes = record.clone();
    let last = bad_bytes.receiver_card_bytes.len() - 1;
    bad_bytes.receiver_card_bytes[last] ^= 0x01;
    assert!(matches!(
        verify_receiver_card_record(&bad_bytes, None),
        Err(CardRecordError::InvalidCard(
            ReceiverCardError::VerifyFailed
        ))
    ));
    assert!(matches!(
        bad_bytes.to_compact(),
        Err(CardRecordError::InvalidCard(
            ReceiverCardError::VerifyFailed
        ))
    ));

    let mut bad_id = record.clone();
    bad_id.registry_entry_id[0] ^= 0x01;
    assert!(matches!(
        verify_receiver_card_record(&bad_id, None),
        Err(CardRecordError::BadEntryId)
    ));
    assert!(matches!(
        bad_id.to_compact(),
        Err(CardRecordError::BadEntryId)
    ));

    let revoked = record.clone().revoked();
    assert!(matches!(
        verify_receiver_card_record(&revoked, None),
        Err(CardRecordError::Revoked)
    ));
    assert!(matches!(
        revoked.to_compact(),
        Err(CardRecordError::Revoked)
    ));

    assert!(matches!(
        verify_receiver_card_record(&record, Some(10)),
        Err(CardRecordError::StaleEpoch)
    ));

    let newer = ReceiverCardRecord::new(
        &record.decode_card().expect("card"),
        record.receiver_card_bytes.clone(),
        10,
    )
    .expect("newer record");
    assert!(matches!(
        check_relabel(&record, &newer),
        Err(CardRecordError::Relabel)
    ));

    let mut rebound = make_record(11);
    rebound.registry_entry_id = record.registry_entry_id;
    assert!(matches!(
        check_relabel(&record, &rebound),
        Err(CardRecordError::Relabel)
    ));
}
