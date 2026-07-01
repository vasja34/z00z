use z00z_wallets::{
    key::{ReceiverKeys, ReceiverSecret},
    receiver::{PaymentRequest, RequestParams},
    receiver::{ScanResult, StealthOutputScanner},
    stealth::{build_tx_output_unchecked, SenderWallet},
};

#[path = "test_inc/test_asset_scan_flow.inc"]
mod asset_scan_support;

use asset_scan_support::{
    build_outputs, count_mine_hits, make_asset, make_card, make_leaf, set_output,
};

#[test]
fn test_pack_roundtrip() {
    let (keys, leaf) = make_leaf(5_000, false);
    let enc_len = leaf
        .enc_pack
        .as_ref()
        .map(|pack| pack.ciphertext.len() + pack.tag.len())
        .unwrap_or_default();
    assert!(
        enc_len > 40,
        "enc_pack must include ciphertext framing and AEAD data"
    );
    let scanner = StealthOutputScanner::from_keys(&keys);

    match scanner.scan_leaf(&leaf) {
        ScanResult::Mine { wallet_output } => {
            assert_eq!(wallet_output.amount, 5_000);
            assert_eq!(wallet_output.asset_id, leaf.asset_id());
        }
        _ => panic!("expected mine result"),
    }
}

#[test]
fn test_wrong_kdh() {
    let (alice_keys, leaf) = make_leaf(999, false);
    let bob_keys =
        ReceiverKeys::from_receiver_secret(ReceiverSecret::generate().expect("receiver secret"))
            .expect("receiver keys");

    let alice_scan = StealthOutputScanner::from_keys(&alice_keys);
    assert!(matches!(
        alice_scan.scan_leaf(&leaf),
        ScanResult::Mine { .. }
    ));

    let bob_scan = StealthOutputScanner::from_keys(&bob_keys);
    assert!(matches!(bob_scan.scan_leaf(&leaf), ScanResult::NotMine));
}

#[test]
fn test_req_binding() {
    let receiver_secret = ReceiverSecret::generate().expect("receiver secret");
    let receiver_keys = ReceiverKeys::from_receiver_secret(receiver_secret).expect("keys");
    let card = make_card(&receiver_keys);

    let request = PaymentRequest::generate(
        &receiver_keys,
        RequestParams {
            amount: Some(303),
            expiry_seconds: 3_600,
            memo: None,
            payment_id: None,
        },
        1,
    )
    .expect("request");

    let mut asset_id = [0u8; 32];
    asset_id[0] = 91;
    let mut sender_wallet = SenderWallet::new([77u8; 32]);
    let output = build_tx_output_unchecked(
        &card,
        Some(&request),
        &mut sender_wallet,
        &[55u8; 32],
        0,
        303,
        &asset_id,
    )
    .expect("output");

    let mut leaf = make_asset(303, asset_id);
    set_output(&mut leaf, &output);
    leaf.tag16 = output.tag16;

    let scanner_no_req = StealthOutputScanner::from_keys(&receiver_keys);
    assert!(matches!(
        scanner_no_req.scan_leaf(&leaf),
        ScanResult::NotMine
    ));

    let mut scanner = StealthOutputScanner::from_keys(&receiver_keys);
    scanner.add_request(&request);
    match scanner.scan_leaf(&leaf) {
        ScanResult::Mine { wallet_output } => assert_eq!(wallet_output.amount, 303),
        _ => panic!("expected mine result with request context"),
    }
}

#[test]
fn test_req_binding_rejects_context() {
    let receiver_secret = ReceiverSecret::generate().expect("receiver secret");
    let receiver_keys = ReceiverKeys::from_receiver_secret(receiver_secret).expect("keys");
    let card = make_card(&receiver_keys);

    let request = PaymentRequest::generate(
        &receiver_keys,
        RequestParams {
            amount: Some(404),
            expiry_seconds: 3_600,
            memo: None,
            payment_id: None,
        },
        1,
    )
    .expect("request");
    let unrelated = PaymentRequest::generate(
        &receiver_keys,
        RequestParams {
            amount: Some(404),
            expiry_seconds: 3_600,
            memo: Some("other".to_string()),
            payment_id: None,
        },
        1,
    )
    .expect("unrelated request");
    assert_ne!(request.req_id, unrelated.req_id);

    let mut sender_wallet = SenderWallet::new([78u8; 32]);
    let output = build_tx_output_unchecked(
        &card,
        Some(&request),
        &mut sender_wallet,
        &[56u8; 32],
        0,
        404,
        &[92u8; 32],
    )
    .expect("output");

    let mut leaf = make_asset(404, [92u8; 32]);
    set_output(&mut leaf, &output);
    leaf.tag16 = output.tag16;

    let mut scanner = StealthOutputScanner::from_keys(&receiver_keys);
    scanner.add_request(&unrelated);
    assert!(matches!(scanner.scan_leaf(&leaf), ScanResult::NotMine));
}

#[test]
fn test_view_rotation() {
    let password = b"phase4-rotation";
    let secret = ReceiverSecret::generate().expect("receiver secret");
    let encrypted = secret.to_encrypted(password).expect("encrypted secret");

    let old_secret = ReceiverSecret::from_encrypted(&encrypted, password).expect("old secret");
    let rotated_secret =
        ReceiverSecret::from_encrypted(&encrypted, password).expect("rotated secret");

    let old_keys = ReceiverKeys::from_receiver_secret(old_secret).expect("old keys");
    let mut rotated_keys = ReceiverKeys::from_receiver_secret(rotated_secret).expect("new keys");

    let card_0 = make_card(&old_keys);
    let card_1 = rotated_keys.rotate_view().expect("rotate view");

    assert_eq!(card_0.owner_handle, card_1.owner_handle);
    assert_ne!(card_0.view_pk, card_1.view_pk);

    let mut sender_wallet = SenderWallet::new([44u8; 32]);
    let asset_id = [0x11; 32];
    let outputs_0 = build_outputs(
        &card_0,
        &mut sender_wallet,
        &[0x0B; 32],
        0..5,
        1_000,
        asset_id,
    );
    let outputs_1 = build_outputs(
        &card_1,
        &mut sender_wallet,
        &[0x0C; 32],
        5..10,
        2_000,
        asset_id,
    );

    let old_scan = StealthOutputScanner::from_keys(&old_keys);
    let new_scan = StealthOutputScanner::from_keys(&rotated_keys);
    let old_hits = count_mine_hits(&old_scan, &outputs_0, 1_000, asset_id, 0);
    let new_hits = count_mine_hits(&new_scan, &outputs_1, 2_000, asset_id, 5);
    let new_on_old = count_mine_hits(&new_scan, &outputs_0, 1_000, asset_id, 0);
    let old_on_new = count_mine_hits(&old_scan, &outputs_1, 2_000, asset_id, 5);

    assert_eq!(old_hits, 5, "v0 key must scan all v0 outputs");
    assert_eq!(new_hits, 5, "v1 key must scan all v1 outputs");
    assert_eq!(
        new_on_old, 5,
        "rotated scanner must keep prior scan continuity"
    );
    assert_eq!(old_on_new, 0, "old key must not scan v1 outputs");
}

#[test]
fn test_scan_flow() {
    let recv_sec = ReceiverSecret::generate().expect("receiver secret");
    let recv_keys = ReceiverKeys::from_receiver_secret(recv_sec).expect("receiver keys");
    let recv_card = make_card(&recv_keys);

    let mut sender = SenderWallet::new([42u8; 32]);
    let tx_dig = [0xAA; 32];
    let asset_id = [0x01; 32];
    let output =
        build_tx_output_unchecked(&recv_card, None, &mut sender, &tx_dig, 0, 1_000, &asset_id)
            .expect("output");

    let mut leaf = make_asset(1_000, asset_id);
    set_output(&mut leaf, &output);
    leaf.tag16 = None;

    let scan = StealthOutputScanner::from_keys(&recv_keys);
    match scan.scan_leaf(&leaf) {
        ScanResult::Mine { wallet_output } => {
            assert_eq!(wallet_output.amount, 1_000);
            assert_eq!(wallet_output.asset_id, leaf.asset_id());
        }
        _ => panic!("expected mine result"),
    }
}

#[test]
fn test_scan_unlink() {
    let recv_sec = ReceiverSecret::generate().expect("receiver secret");
    let recv_keys = ReceiverKeys::from_receiver_secret(recv_sec).expect("receiver keys");
    let recv_card = make_card(&recv_keys);

    let mut sender = SenderWallet::new([99u8; 32]);
    let tx_dig = [0xBB; 32];
    let asset_id = [0x02; 32];

    let mut outputs = Vec::new();
    for index in 0..10 {
        let out = build_tx_output_unchecked(
            &recv_card,
            None,
            &mut sender,
            &tx_dig,
            index,
            100 + index as u64,
            &asset_id,
        )
        .expect("output");
        outputs.push(out);
    }

    for left in 0..10 {
        for right in (left + 1)..10 {
            assert_ne!(outputs[left].r_pub, outputs[right].r_pub);
            assert_ne!(outputs[left].owner_tag, outputs[right].owner_tag);
        }
    }

    let scan = StealthOutputScanner::from_keys(&recv_keys);
    let mut hits = 0usize;
    for (index, out) in outputs.iter().enumerate() {
        let mut leaf = make_asset(100 + index as u64, asset_id);
        set_output(&mut leaf, out);
        leaf.tag16 = None;

        if matches!(scan.scan_leaf(&leaf), ScanResult::Mine { .. }) {
            hits += 1;
        }
    }

    assert_eq!(hits, 10);
}

#[test]
fn test_wrong_scan() {
    let alice_sec = ReceiverSecret::generate().expect("alice secret");
    let alice_keys = ReceiverKeys::from_receiver_secret(alice_sec).expect("alice keys");
    let alice_card = make_card(&alice_keys);

    let bob_sec = ReceiverSecret::generate().expect("bob secret");
    let bob_keys = ReceiverKeys::from_receiver_secret(bob_sec).expect("bob keys");

    let mut sender = SenderWallet::new([7u8; 32]);
    let output = build_tx_output_unchecked(
        &alice_card,
        None,
        &mut sender,
        &[0xCC; 32],
        0,
        500,
        &[0x03; 32],
    )
    .expect("output");

    let mut leaf = make_asset(500, [0x03; 32]);
    set_output(&mut leaf, &output);
    leaf.tag16 = None;

    let bob_scan = StealthOutputScanner::from_keys(&bob_keys);
    assert!(matches!(bob_scan.scan_leaf(&leaf), ScanResult::NotMine));

    let alice_scan = StealthOutputScanner::from_keys(&alice_keys);
    assert!(matches!(
        alice_scan.scan_leaf(&leaf),
        ScanResult::Mine { .. }
    ));
}

#[test]
fn test_req_bind() {
    let recv_sec = ReceiverSecret::generate().expect("receiver secret");
    let recv_keys = ReceiverKeys::from_receiver_secret(recv_sec).expect("receiver keys");
    let recv_card = make_card(&recv_keys);

    let mut request = PaymentRequest {
        version: 1,
        owner_handle: recv_card.owner_handle,
        view_pk: recv_card.view_pk,
        identity_pk: recv_card.identity_pk,
        req_id: [0x61; 32],
        chain_id: 1,
        amount: Some(2_000),
        expiry: u64::MAX,
        metadata: None,
        signature: [0u8; 64],
    };
    request
        .sign(recv_keys.reveal_identity_sk())
        .expect("sign request");
    request.verify().expect("verify request");

    let mut sender = SenderWallet::new([33u8; 32]);
    let output = build_tx_output_unchecked(
        &recv_card,
        Some(&request),
        &mut sender,
        &[0xDD; 32],
        0,
        2_000,
        &[0x04; 32],
    )
    .expect("output");

    let mut leaf = make_asset(2_000, [0x04; 32]);
    set_output(&mut leaf, &output);
    leaf.tag16 = output.tag16;

    let scan_no_req = StealthOutputScanner::from_keys(&recv_keys);
    assert!(matches!(scan_no_req.scan_leaf(&leaf), ScanResult::NotMine));

    let mut scan = StealthOutputScanner::from_keys(&recv_keys);
    scan.add_request(&request);
    match scan.scan_leaf(&leaf) {
        ScanResult::Mine { wallet_output } => assert_eq!(wallet_output.amount, 2_000),
        _ => panic!("expected mine result with request context"),
    }
}
