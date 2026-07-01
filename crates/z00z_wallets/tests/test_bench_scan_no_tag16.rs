#[cfg(feature = "test-params-fast")]
use z00z_core::genesis::asset_std::{asset_from_dev_cfg, def_from_dev_cfg};
#[cfg(feature = "test-params-fast")]
use z00z_core::Asset;
#[cfg(feature = "test-params-fast")]
use z00z_crypto::ZkPackEncrypted;
#[cfg(feature = "test-params-fast")]
use z00z_utils::time::Instant;
#[cfg(feature = "test-params-fast")]
use z00z_wallets::receiver::Tag16Context;
#[cfg(feature = "test-params-fast")]
use z00z_wallets::{
    key::{ReceiverKeys, ReceiverSecret},
    receiver::{ScanResult, StealthOutputScanner},
    stealth::ecdh::{compute_dh_receiver, decode_r_pub},
    stealth::kdf::derive_k_dh,
    stealth::{build_tx_output_unchecked, SenderWallet, TxStealthOutput},
};

#[cfg(feature = "test-params-fast")]
#[test]
fn test_bench_scan_no_tag16() {
    let alice =
        ReceiverKeys::from_receiver_secret(ReceiverSecret::generate().expect("alice secret"))
            .expect("alice keys");

    let card_a = alice.export_receiver_card().expect("alice card");
    let mut sender = SenderWallet::new([9u8; 32]);
    let mut scanner = StealthOutputScanner::from_keys(&alice);

    let aid = def_from_dev_cfg("z00z").expect("std def").id;
    let mine_n = 10usize;
    let total_n = 10_000usize;
    let noise_n = total_n - mine_n;
    let noise_tpl = make_asset(aid, 2_000);

    let mut leaves_tag = Vec::with_capacity(total_n);
    for idx in 0..mine_n {
        let out = build_tx_output_unchecked(
            &card_a,
            None,
            &mut sender,
            &[0x21; 32],
            idx as u32,
            1_000 + idx as u64,
            &aid,
        )
        .expect("mine output");

        let leaf = bind_output(&out, aid, 1_000 + idx as u64);
        let rp = decode_r_pub(&out.r_pub).expect("decode r_pub");
        let dh = compute_dh_receiver(alice.reveal_view_sk(), &rp).expect("ecdh");
        let k_dh = derive_k_dh(&dh);
        scanner.add_tag_context(
            out.tag16.expect("mine tag16"),
            Tag16Context { k_dh, req_id: None },
        );
        leaves_tag.push(leaf);
    }

    for idx in 0..noise_n {
        leaves_tag.push(make_noise_leaf(&noise_tpl, idx as u64));
    }

    scanner.materialize_complete_tag_contexts(Vec::<(u16, Tag16Context)>::new());

    let mut leaves_no = leaves_tag.clone();
    for leaf in &mut leaves_no {
        leaf.tag16 = None;
    }

    let t_fast = Instant::now();
    let mut fast_found = 0usize;
    for leaf in &leaves_tag {
        if matches!(scanner.scan_leaf_tag_only(leaf), ScanResult::Mine { .. }) {
            fast_found += 1;
        }
    }
    let fast_elapsed = t_fast.elapsed();

    let t_slow = Instant::now();
    let mut slow_found = 0usize;
    for leaf in &leaves_no {
        if matches!(scanner.scan_leaf(leaf), ScanResult::Mine { .. }) {
            slow_found += 1;
        }
    }
    let slow_elapsed = t_slow.elapsed();

    let fast_secs = fast_elapsed.as_secs_f64();
    let slow_secs = slow_elapsed.as_secs_f64();
    let ratio = slow_secs / fast_secs.max(f64::MIN_POSITIVE);
    println!(
        "bench_scan_no_tag16 fast={fast_elapsed:?} slow={slow_elapsed:?} ratio={ratio:.2} found={slow_found}"
    );

    assert_eq!(fast_found, mine_n, "unexpected fast matches");
    assert_eq!(slow_found, mine_n, "unexpected slow matches");
    assert!(
        slow_elapsed > fast_elapsed,
        "fallback scan must be slower than tag16 scan"
    );
    assert!(
        ratio >= 1.1,
        "without tag16 must stay measurably slower, got {ratio:.2}x"
    );
}

#[cfg(feature = "test-params-fast")]
fn bind_output(out: &TxStealthOutput, aid: [u8; 32], amount: u64) -> Asset {
    let mut asset = make_asset(aid, amount);
    asset.commitment = z00z_crypto::Commitment::from_bytes(&out.c_amount)
        .expect("commitment")
        .0;
    asset.r_pub = Some(out.r_pub);
    asset.owner_tag = Some(out.owner_tag);
    asset.enc_pack = Some(out.enc_pack.clone());
    asset.tag16 = out.tag16;
    asset.leaf_ad_id = Some(asset.definition.id);
    asset
}

#[cfg(feature = "test-params-fast")]
fn make_asset(aid: [u8; 32], amount: u64) -> Asset {
    let asset = asset_from_dev_cfg("z00z", 0, amount).expect("std asset");
    assert_eq!(
        asset.definition.id, aid,
        "unexpected asset id from standard generator"
    );
    asset
}

#[cfg(feature = "test-params-fast")]
fn make_noise_leaf(base: &Asset, nonce: u64) -> Asset {
    let mut leaf = base.clone();
    let mut r_pub = [0u8; 32];
    r_pub[0..8].copy_from_slice(&nonce.to_le_bytes());
    let mut owner_tag = [0u8; 32];
    owner_tag[8..16].copy_from_slice(&nonce.to_le_bytes());

    leaf.r_pub = Some(r_pub);
    leaf.owner_tag = Some(owner_tag);
    leaf.enc_pack = Some(ZkPackEncrypted {
        version: 1,
        ciphertext: vec![0u8; 24],
        tag: [0u8; 16],
    });
    leaf.tag16 = Some(0xFFFF);
    leaf
}
