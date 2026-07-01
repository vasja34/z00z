use z00z_wallets::receiver::{
    ScanChunk, ScanResult, ScanStrategy, StealthOutputScanner, Tag16Cache, Tag16Context,
};
use z00z_wallets::{
    key::{ReceiverKeys, ReceiverSecret},
    receiver::{PaymentRequest, RequestParams},
    stealth::{build_tx_output_unchecked, SenderWallet},
};

#[path = "test_inc/test_asset_scan_cache.inc"]
mod asset_scan_support;

use asset_scan_support::{build_leaf_with_keys, make_asset, make_card, make_leaf, set_output};

#[test]
fn test_tag16_cache_insert() {
    let mut cache = Tag16Cache::new();
    let tag16 = 7u16;

    cache.insert(
        tag16,
        Tag16Context {
            k_dh: [1u8; 32],
            req_id: None,
        },
    );

    assert!(cache.contains(tag16));
    let contexts = cache.get_contexts(tag16).expect("contexts");
    assert_eq!(contexts.len(), 1);
    assert_eq!(contexts[0].k_dh, [1u8; 32]);
}

#[test]
fn test_tag16_cache_collision_handling() {
    let mut cache = Tag16Cache::new();
    let tag16 = 11u16;

    cache.insert(
        tag16,
        Tag16Context {
            k_dh: [3u8; 32],
            req_id: None,
        },
    );
    cache.insert(
        tag16,
        Tag16Context {
            k_dh: [5u8; 32],
            req_id: None,
        },
    );

    let contexts = cache.get_contexts(tag16).expect("contexts");
    assert_eq!(contexts.len(), 2);
    assert_ne!(contexts[0].k_dh, contexts[1].k_dh);
}

#[test]
fn test_tag16_cache_active_requests() {
    let mut cache = Tag16Cache::new();
    let req_id = [9u8; 32];

    cache.add_active_request(req_id);
    assert!(cache.is_active_request(&req_id));

    cache.insert(
        19,
        Tag16Context {
            k_dh: [7u8; 32],
            req_id: Some(req_id),
        },
    );
    assert!(cache.is_active_request(&req_id));
}

#[test]
fn test_tag16_cache_stats() {
    let mut cache = Tag16Cache::new();
    cache.insert(
        77,
        Tag16Context {
            k_dh: [1u8; 32],
            req_id: None,
        },
    );
    cache.insert(
        77,
        Tag16Context {
            k_dh: [3u8; 32],
            req_id: None,
        },
    );

    assert!(cache.contains(77));
    assert!(!cache.contains(88));

    let stats = cache.stats();
    assert!(stats.hits >= 1);
    assert!(stats.misses >= 1);
    assert!(stats.collisions >= 1);
    assert_eq!(stats.size, 1);
}

#[test]
fn test_tag16_cache_clear() {
    let mut cache = Tag16Cache::new();
    let req_id = [99u8; 32];
    cache.insert(
        15,
        Tag16Context {
            k_dh: [5u8; 32],
            req_id: Some(req_id),
        },
    );
    assert!(cache.contains(15));
    assert!(cache.is_active_request(&req_id));

    cache.clear();

    assert!(!cache.contains(15));
    assert!(!cache.is_active_request(&req_id));
    assert_eq!(cache.stats().size, 0);
}

#[test]
fn test_scan_checkpoint_finds_owned() {
    let (keys, mut leaf) = make_leaf(500, false);
    let scanner = StealthOutputScanner::from_keys(&keys);
    let found = scanner.scan_checkpoint(&[leaf.clone()]);
    assert_eq!(found.len(), 1);
    assert_eq!(found[0].amount, 500);

    leaf.owner_tag = Some([0u8; 32]);
    let miss = scanner.scan_checkpoint(&[leaf]);
    assert!(miss.is_empty());
}

#[test]
fn test_scan_range_resume_ok() {
    let (keys, leaf_a) = make_leaf(500, true);
    let leaf_b = build_leaf_with_keys(&keys, 700, 18, true);
    let scanner = StealthOutputScanner::from_keys(&keys);
    let chunks = vec![
        ScanChunk {
            height: 7,
            hash: vec![7u8; 32],
            leaves: vec![leaf_a],
        },
        ScanChunk {
            height: 8,
            hash: vec![8u8; 32],
            leaves: vec![leaf_b],
        },
    ];

    let first = scanner
        .scan_range(&chunks, None, Some(1))
        .expect("first range");
    assert_eq!(first.outputs.len(), 1);
    assert_eq!(first.stat.done_ckpt, 1);
    assert_eq!(first.stat.cursor.last_scanned_height, 7);

    let next = scanner
        .scan_range(&chunks, Some(&first.stat.cursor), None)
        .expect("resume range");
    assert_eq!(next.outputs.len(), 1);
    assert_eq!(next.stat.done_ckpt, 1);
    assert_eq!(next.stat.cursor.last_scanned_height, 8);
}

#[test]
fn test_scan_skips_not_owned() {
    let (owner_keys, leaf) = make_leaf(700, true);
    let other_keys =
        ReceiverKeys::from_receiver_secret(ReceiverSecret::generate().expect("receiver secret"))
            .expect("receiver keys");

    let scanner = StealthOutputScanner::from_keys(&other_keys);
    let found = scanner.scan_checkpoint(&[leaf]);
    assert!(found.is_empty());

    let owner_scan = StealthOutputScanner::from_keys(&owner_keys);
    assert_eq!(owner_scan.scan_checkpoint(&[]).len(), 0);
}

#[test]
fn test_scan_with_tag16_filter() {
    let (keys, leaf) = make_leaf(321, true);

    let scanner = StealthOutputScanner::from_keys(&keys);
    let found = scanner.scan_checkpoint(&[leaf.clone()]);
    assert_eq!(found.len(), 1);
    assert_eq!(found[0].amount, 321);

    let mut tampered = leaf;
    tampered.tag16 = Some(0xA1B2);

    let mut scanner = StealthOutputScanner::from_keys(&keys);
    scanner.add_tag_context(
        0xA1B2,
        Tag16Context {
            k_dh: [1u8; 32],
            req_id: None,
        },
    );
    let result = scanner.scan_leaf(&tampered);
    assert!(matches!(result, ScanResult::MaybeMine { .. }));
}

#[test]
fn test_scan_without_tag16() {
    let (keys, mut leaf) = make_leaf(111, true);
    leaf.tag16 = None;

    let scanner = StealthOutputScanner::from_keys(&keys);
    let found = scanner.scan_checkpoint(&[leaf]);
    assert_eq!(found.len(), 1);
    assert_eq!(found[0].amount, 111);
}

#[test]
fn test_handle_tag16_match() {
    let (keys, leaf) = make_leaf(345, true);
    let mut scanner = StealthOutputScanner::from_keys(&keys);
    scanner.materialize_complete_tag_contexts(vec![(
        leaf.tag16.expect("tag16"),
        Tag16Context {
            k_dh: asset_scan_support::leaf_kdh(&keys, &leaf),
            req_id: None,
        },
    )]);

    let result = scanner.scan_leaf_tag_only(&leaf);
    assert!(matches!(result, ScanResult::Mine { .. }));
}

#[test]
fn test_background_scan_strategy() {
    let (keys, _) = make_leaf(123, false);
    let mut scanner = StealthOutputScanner::from_keys(&keys);

    assert_eq!(scanner.background_scan_strategy(), ScanStrategy::FullScan);

    scanner.materialize_complete_tag_contexts((0..1001u16).map(|tag| {
        (
            tag,
            Tag16Context {
                k_dh: [1u8; 32],
                req_id: None,
            },
        )
    }));
    assert_eq!(scanner.background_scan_strategy(), ScanStrategy::Balanced);

    scanner.materialize_complete_tag_contexts((1001u16..10002u16).map(|idx| {
        (
            idx,
            Tag16Context {
                k_dh: [3u8; 32],
                req_id: None,
            },
        )
    }));
    assert_eq!(
        scanner.background_scan_strategy(),
        ScanStrategy::TagFilterOnly
    );
}

#[test]
fn test_scan_with_request_key() {
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
    leaf.tag16 = None;

    let scanner_no_req = StealthOutputScanner::from_keys(&receiver_keys);
    assert!(matches!(
        scanner_no_req.scan_leaf(&leaf),
        ScanResult::NotMine
    ));

    let mut scanner = StealthOutputScanner::from_keys(&receiver_keys);
    scanner.add_request(&request);
    assert!(matches!(scanner.scan_leaf(&leaf), ScanResult::Mine { .. }));
}
