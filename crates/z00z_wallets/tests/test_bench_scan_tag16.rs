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
fn test_bench_scan_tag16() {
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
    let mut leaves = Vec::with_capacity(total_n);
    let noise_tpl = make_asset(aid, 2_000);

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

        let mut leaf = bind_output(&out, aid, 1_000 + idx as u64);
        let rp = decode_r_pub(&out.r_pub).expect("decode r_pub");
        let dh = compute_dh_receiver(alice.reveal_view_sk(), &rp).expect("ecdh");
        let k_dh = derive_k_dh(&dh);
        scanner.add_tag_context(
            out.tag16.expect("mine tag16"),
            Tag16Context { k_dh, req_id: None },
        );
        leaf.tag16 = out.tag16;
        leaves.push(leaf);
    }

    scanner.materialize_complete_tag_contexts(Vec::<(u16, Tag16Context)>::new());

    for idx in 0..noise_n {
        leaves.push(make_noise_leaf(&noise_tpl, idx as u64));
    }

    let start = Instant::now();
    let mut found = 0usize;
    for leaf in &leaves {
        if matches!(scanner.scan_leaf_tag_only(leaf), ScanResult::Mine { .. }) {
            found += 1;
        }
    }
    let elapsed = start.elapsed();

    println!("bench_scan_tag16 total={elapsed:?} found={found}");
    assert_eq!(found, mine_n, "unexpected number of matches");
    assert!(
        elapsed.as_millis() < 100,
        "scan 10k with tag16 exceeded 100ms: {:?}",
        elapsed
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
    let enc_pack = ZkPackEncrypted {
        version: 1,
        ciphertext: vec![0u8; 24],
        tag: [0u8; 16],
    };

    leaf.r_pub = Some(r_pub);
    leaf.owner_tag = Some(owner_tag);
    leaf.enc_pack = Some(enc_pack);
    leaf.tag16 = Some(0xFFFF);
    leaf
}
