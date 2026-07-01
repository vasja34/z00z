use std::time::{Duration, Instant};

use z00z_utils::rng::{RngCoreExt, SystemRngProvider};
use z00z_wallets::receiver::{
    DoSMitigation, ScanDecision, ScanResult, StealthOutputScanner, Tag16Cache, Tag16Context,
};
use z00z_wallets::{
    key::{ReceiverKeys, ReceiverSecret},
    stealth::ecdh::{compute_dh_receiver, decode_r_pub},
    stealth::kdf::{compute_owner_tag, compute_tag16, derive_k_dh},
    stealth::SenderWallet,
};

#[path = "test_inc/test_asset_scan_support.inc"]
mod asset_scan_support;

use asset_scan_support::{
    fill_cache, leaf_kdh, leaf_req_kdh, make_card, make_leaf, make_noise, make_own, make_req,
    req_leaf, scan_fast, scan_slow, scan_tag, scanner_assets, OWNED_COUNT,
};

#[test]
fn test_req_flow() {
    let receiver_secret = ReceiverSecret::generate().expect("receiver secret");
    let receiver_keys = ReceiverKeys::from_receiver_secret(receiver_secret).expect("keys");
    let card = make_card(&receiver_keys);
    let req_a = make_req(&receiver_keys, 707);
    let req_b = make_req(&receiver_keys, 707);
    assert_ne!(req_a.req_id, req_b.req_id);

    let mut asset_id = [0u8; 32];
    asset_id[0] = 0x71;
    let mut sender = SenderWallet::new([0x41u8; 32]);

    let leaf_a = req_leaf(&card, &mut sender, &req_a, &[0xA1; 32], 0, 707, asset_id);
    let leaf_b = req_leaf(&card, &mut sender, &req_b, &[0xB2; 32], 1, 707, asset_id);

    let mut scan = StealthOutputScanner::from_keys(&receiver_keys);
    scan.add_request(&req_a);

    assert!(matches!(scan.scan_leaf(&leaf_a), ScanResult::Mine { .. }));
    assert!(matches!(scan.scan_leaf(&leaf_b), ScanResult::NotMine));
    assert!(matches!(
        scan.scan_leaf_tag_only(&leaf_a),
        ScanResult::NotMine
    ));

    let mut strict = StealthOutputScanner::from_keys(&receiver_keys);
    strict.add_request(&req_a);
    strict.add_tag_context(
        leaf_a.tag16.expect("tag16").wrapping_add(1),
        Tag16Context {
            k_dh: [0x44; 32],
            req_id: None,
        },
    );
    assert!(matches!(strict.scan_leaf(&leaf_a), ScanResult::Mine { .. }));
    assert!(matches!(
        strict.scan_leaf_tag_only(&leaf_a),
        ScanResult::NotMine
    ));

    strict.add_tag_context(
        leaf_a.tag16.expect("tag16"),
        Tag16Context {
            k_dh: leaf_req_kdh(&receiver_keys, &leaf_a, &req_a.req_id),
            req_id: Some(req_a.req_id),
        },
    );
    strict.materialize_complete_tag_contexts(Vec::<(u16, Tag16Context)>::new());

    assert!(matches!(strict.scan_leaf(&leaf_a), ScanResult::Mine { .. }));
    assert!(matches!(
        strict.scan_leaf_tag_only(&leaf_a),
        ScanResult::Mine { .. }
    ));
    assert!(matches!(
        strict.scan_leaf_tag_only(&leaf_b),
        ScanResult::NotMine
    ));
}

#[test]
fn test_fast_reject() {
    let receiver_secret = ReceiverSecret::generate().expect("receiver secret");
    let receiver_keys = ReceiverKeys::from_receiver_secret(receiver_secret).expect("keys");
    let card = make_card(&receiver_keys);
    let mut sender = SenderWallet::new([55u8; 32]);

    let asset_id = [0x05; 32];
    let (my_leaves, my_tags) = make_own(&card, &mut sender, asset_id);
    let noise_leaves = make_noise(&mut sender, asset_id, &my_tags);

    let mut all_leaves = Vec::with_capacity(my_leaves.len() + noise_leaves.len());
    all_leaves.extend(my_leaves.iter().cloned());
    all_leaves.extend(noise_leaves);

    let cache = fill_cache(&receiver_keys, &my_leaves);
    let mut found = 0usize;
    let mut skipped = 0usize;
    let mut decrypt_try = 0usize;

    for leaf in &all_leaves {
        let tag16 = leaf.tag16.expect("leaf tag16");
        match cache.get_contexts(tag16) {
            Some(contexts) => {
                decrypt_try = decrypt_try.saturating_add(contexts.len());
                let mut local = StealthOutputScanner::from_keys(&receiver_keys);
                for context in contexts {
                    local.add_tag_context(tag16, context.clone());
                }
                local.materialize_complete_tag_contexts(Vec::<(u16, Tag16Context)>::new());
                if matches!(local.scan_leaf_tag_only(leaf), ScanResult::Mine { .. }) {
                    found = found.saturating_add(1);
                }
            }
            None => skipped = skipped.saturating_add(1),
        }
    }

    let total = all_leaves.len();
    let skip_rate = (skipped as f64 / total as f64) * 100.0;

    assert_eq!(found, 10, "must find all receiver outputs");
    assert!(skip_rate > 99.0, "tag16 prefilter must skip >99% noise");
    assert!(
        decrypt_try <= 20,
        "decrypt attempts should stay near receiver output count"
    );
}

#[test]
fn test_prefilter_collision() {
    let (alice_keys, legit_leaf) = make_leaf(777, true);
    let mut scanner = StealthOutputScanner::from_keys(&alice_keys);

    let legit_r_pub = legit_leaf.r_pub.expect("r_pub");
    let legit_decoded = decode_r_pub(&legit_r_pub).expect("decode");
    let legit_dh = compute_dh_receiver(alice_keys.reveal_view_sk(), &legit_decoded).expect("dh");
    let legit_k_dh = derive_k_dh(&legit_dh);
    let legit_tag = legit_leaf.tag16.expect("tag16");

    scanner.add_tag_context(
        legit_tag,
        Tag16Context {
            k_dh: legit_k_dh,
            req_id: None,
        },
    );

    let (_other_keys, mut collision_leaf) = make_leaf(555, true);
    collision_leaf.tag16 = Some(legit_tag);

    let ok = scanner.scan_leaf(&legit_leaf);
    assert!(matches!(ok, ScanResult::Mine { .. }));

    let bad = scanner.scan_leaf(&collision_leaf);
    assert!(matches!(
        bad,
        ScanResult::MaybeMine {
            tag16_match: true,
            m1_failed: true
        }
    ));

    let mut metric_cache = Tag16Cache::new();
    metric_cache.insert(
        legit_tag,
        Tag16Context {
            k_dh: legit_k_dh,
            req_id: None,
        },
    );
    metric_cache.insert(
        legit_tag,
        Tag16Context {
            k_dh: [0xAA; 32],
            req_id: None,
        },
    );
    let stats = metric_cache.stats();
    assert!(
        stats.collisions >= 1,
        "collision metrics must increase on same tag16 key"
    );
}

#[test]
fn test_m1_block() {
    let (alice_keys, mut leaf) = make_leaf(700, false);
    let bob_secret = ReceiverSecret::generate().expect("bob secret");
    let bob_keys = ReceiverKeys::from_receiver_secret(bob_secret).expect("bob keys");

    let r_pub = leaf.r_pub.expect("r_pub");
    let r_pub_decoded = decode_r_pub(&r_pub).expect("decode");
    let dh = compute_dh_receiver(alice_keys.reveal_view_sk(), &r_pub_decoded).expect("dh");
    let k_dh = derive_k_dh(&dh);

    let forged = compute_owner_tag(&bob_keys.owner_handle, &k_dh);
    leaf.owner_tag = Some(forged);

    let scan = StealthOutputScanner::from_keys(&alice_keys);
    assert!(matches!(scan.scan_leaf(&leaf), ScanResult::NotMine));
}

#[test]
fn test_dos_limit() {
    let (keys, leaf) = make_leaf(101, false);
    let scanner = StealthOutputScanner::from_keys(&keys);
    let policy = DoSMitigation::new(1_000, 10, 100);

    let legit_batch = vec![leaf.clone(); 100];
    let found = scanner.scan_with_dos_protection(&legit_batch, &policy);
    assert_eq!(
        found.len(),
        10,
        "decrypt limiter must cap processed outputs"
    );

    let mut spam_batch = vec![leaf; 100];
    for item in &mut spam_batch {
        item.owner_tag = Some([0u8; 32]);
    }

    let spam_found = scanner.scan_with_dos_protection(&spam_batch, &policy);
    assert!(
        spam_found.is_empty(),
        "spam outputs with invalid owner_tag must be rejected"
    );
}

#[test]
fn test_bad_rpub() {
    let (keys, mut leaf) = make_leaf(202, false);
    let scan = StealthOutputScanner::from_keys(&keys);

    leaf.r_pub = Some([0u8; 32]);
    let id_res = scan.scan_leaf(&leaf);
    assert!(matches!(id_res, ScanResult::NotMine));

    let mut malformed = None;
    for byte in 1u8..=u8::MAX {
        let candidate = [byte; 32];
        if decode_r_pub(&candidate).is_err() {
            malformed = Some(candidate);
            break;
        }
    }

    let malformed = malformed.expect("must produce invalid compressed point bytes");
    leaf.r_pub = Some(malformed);
    let malformed_res = scan.scan_leaf(&leaf);
    assert!(matches!(malformed_res, ScanResult::NotMine));
}

#[test]
fn test_tampered_tag16() {
    let (keys, mut leaf) = make_leaf(242, true);
    let scan = StealthOutputScanner::from_keys(&keys);

    leaf.tag16 = leaf.tag16.map(|tag16| tag16 ^ 1);
    let result = scan.scan_leaf(&leaf);
    assert!(matches!(
        result,
        ScanResult::MaybeMine {
            tag16_match: true,
            m1_failed: false
        }
    ));
}

#[test]
fn test_missing_fields() {
    let (keys, leaf) = make_leaf(303, false);
    let scan = StealthOutputScanner::from_keys(&keys);

    let mut no_rpub = leaf.clone();
    no_rpub.r_pub = None;
    assert!(matches!(scan.scan_leaf(&no_rpub), ScanResult::NotMine));

    let mut no_tag = leaf.clone();
    no_tag.owner_tag = None;
    assert!(matches!(scan.scan_leaf(&no_tag), ScanResult::NotMine));

    let mut no_pack = leaf;
    no_pack.enc_pack = None;
    assert!(matches!(scan.scan_leaf(&no_pack), ScanResult::NotMine));
}

#[test]
fn test_m1_check_prevents_attack() {
    let (keys, mut leaf) = make_leaf(777, false);
    let scanner = StealthOutputScanner::from_keys(&keys);
    let ok = scanner.scan_leaf(&leaf);
    assert!(matches!(ok, ScanResult::Mine { .. }));

    if let Some(owner_tag) = leaf.owner_tag.as_mut() {
        owner_tag[0] ^= 1;
    }

    let bad = scanner.scan_leaf(&leaf);
    assert!(matches!(bad, ScanResult::NotMine));
}

#[test]
fn test_dos_rate_limiting() {
    let (keys, leaf) = make_leaf(42, false);
    let scanner = StealthOutputScanner::from_keys(&keys);
    let leaves = vec![leaf; 5];

    let policy = DoSMitigation::new(10, 2, 10);
    let found = scanner.scan_with_dos_protection(&leaves, &policy);
    assert!(found.len() <= 2);
}

#[test]
fn test_dos_spam_detection() {
    let policy = DoSMitigation::new(2, 10, 1);
    let decision = policy.should_try_decrypt(3, 0);
    assert!(matches!(
        decision,
        ScanDecision::Defer {
            reason: "candidate_limit",
            ..
        }
    ));
}

#[test]
fn test_dos_defer_expensive_ops() {
    let policy = DoSMitigation::new(10, 1, 1);
    let decision = policy.should_try_decrypt(1, 1);
    assert!(matches!(
        decision,
        ScanDecision::Defer {
            reason: "decrypt_limit",
            ..
        }
    ));
}

#[test]
fn test_tag16_computation() {
    let tag = compute_tag16(&[1u8; 32], &[2u8; 32]);
    assert!((0..=u16::MAX).contains(&tag));
}

#[test]
fn test_tag16_false_positive_rate() {
    let provider = SystemRngProvider;
    let mut rng = provider.rng();
    let mut k_dh = [0u8; 32];
    let mut leaf_ad = [0u8; 32];

    let target = 0xABCDu16;
    let mut hits = 0usize;
    let trials = 10_000usize;

    for _ in 0..trials {
        rng.fill_bytes_ext(&mut k_dh);
        rng.fill_bytes_ext(&mut leaf_ad);
        if compute_tag16(&k_dh, &leaf_ad) == target {
            hits = hits.saturating_add(1);
        }
    }

    let fp_rate = hits as f64 / trials as f64;
    assert!(fp_rate < (1.0 / 256.0));
}

#[test]
fn test_e2e_tag16() {
    let (keys, mut leaf) = make_leaf(909, true);
    let mut scanner = StealthOutputScanner::from_keys(&keys);

    let r_pub = leaf.r_pub.expect("r_pub");
    let r_pub_decoded = decode_r_pub(&r_pub).expect("decode");
    let dh = compute_dh_receiver(keys.reveal_view_sk(), &r_pub_decoded).expect("dh");
    let k_dh = derive_k_dh(&dh);

    let tag16 = leaf.tag16.expect("tag16");
    scanner.add_tag_context(tag16, Tag16Context { k_dh, req_id: None });

    let result = scanner.scan_leaf(&leaf);
    assert!(matches!(result, ScanResult::Mine { .. }));

    leaf.owner_tag = Some([0u8; 32]);
    let bad = scanner.scan_leaf(&leaf);
    assert!(matches!(bad, ScanResult::MaybeMine { .. }));
}

#[test]
fn test_tag_safe() {
    let (keys, mut leaf) = make_leaf(909, true);
    let tag16 = leaf.tag16.expect("tag16");
    let good_ctx = Tag16Context {
        k_dh: leaf_kdh(&keys, &leaf),
        req_id: None,
    };

    let plain = StealthOutputScanner::from_keys(&keys);
    assert!(matches!(plain.scan_leaf(&leaf), ScanResult::Mine { .. }));
    assert!(matches!(
        plain.scan_leaf_tag_only(&leaf),
        ScanResult::NotMine
    ));

    let miss = scan_tag(tag16.wrapping_add(1), [0x11; 32], None, &keys);
    assert!(matches!(miss.scan_leaf(&leaf), ScanResult::Mine { .. }));
    assert!(matches!(
        miss.scan_leaf_tag_only(&leaf),
        ScanResult::NotMine
    ));

    let good = scan_tag(tag16, good_ctx.k_dh, good_ctx.req_id, &keys);
    assert!(matches!(good.scan_leaf(&leaf), ScanResult::Mine { .. }));
    assert!(matches!(
        good.scan_leaf_tag_only(&leaf),
        ScanResult::Mine { .. }
    ));

    let wrong = scan_tag(tag16, [0xAA; 32], None, &keys);
    assert!(matches!(
        wrong.scan_leaf_tag_only(&leaf),
        ScanResult::MaybeMine {
            tag16_match: true,
            m1_failed: true
        }
    ));

    let mut mixed = StealthOutputScanner::from_keys(&keys);
    mixed.add_tag_context(
        tag16,
        Tag16Context {
            k_dh: [0xAA; 32],
            req_id: None,
        },
    );
    mixed.add_tag_context(tag16, good_ctx);
    mixed.materialize_complete_tag_contexts(Vec::<(u16, Tag16Context)>::new());
    assert!(matches!(
        mixed.scan_leaf_tag_only(&leaf),
        ScanResult::Mine { .. }
    ));

    leaf.owner_tag = Some([0u8; 32]);
    assert!(!matches!(
        mixed.scan_leaf_tag_only(&leaf),
        ScanResult::Mine { .. }
    ));
}

#[test]
fn test_dos_resistance() {
    let (keys, leaf) = make_leaf(1234, true);
    let mut scanner = StealthOutputScanner::from_keys(&keys);
    let target = leaf.tag16.expect("tag16");
    let k_dh = [0xA5; 32];

    scanner.add_tag_context(target, Tag16Context { k_dh, req_id: None });

    let mut leaves = Vec::with_capacity(1000);
    for _ in 0..1000 {
        let mut clone = leaf.clone();
        clone.owner_tag = Some([0u8; 32]);
        leaves.push(clone);
    }

    let policy = DoSMitigation::new(1500, 200, 1000);
    let start = Instant::now();
    let _ = scanner.scan_with_dos_protection(&leaves, &policy);
    let elapsed = start.elapsed();
    assert!(elapsed < Duration::from_secs(5));
}

#[test]
#[ignore = "performance test"]
fn test_prefilter_perf() {
    let (keys, mut fast_assets, mut slow_assets, ctxs) = scanner_assets();
    let fast = scan_fast(&keys, &mut fast_assets, &ctxs);
    let slow = scan_slow(&keys, &mut slow_assets);

    assert_eq!(fast.0, OWNED_COUNT, "must find all receiver outputs");
    assert_eq!(
        slow.0, OWNED_COUNT,
        "must find all receiver outputs without tag16"
    );
    assert!(
        fast.1 < Duration::from_secs(5),
        "tag16 scan must complete in <5 seconds"
    );
    assert!(
        slow.1 < Duration::from_secs(5),
        "baseline scan must complete in <5 seconds"
    );
    assert!(
        fast.1 < slow.1,
        "tag16 path must be faster than baseline path"
    );
}
