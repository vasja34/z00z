use std::collections::HashSet;
use std::time::Duration;

use z00z_core::Asset;
use z00z_wallets::{
    key::{ReceiverKeys, ReceiverSecret},
    receiver::{PaymentRequest, ReceiverCard, RequestParams},
    receiver::{ScanResult, StealthOutputScanner, Tag16Cache, Tag16Context},
    stealth::{build_tx_output_unchecked, SenderWallet, TxStealthOutput},
    stealth::{
        ecdh::{compute_dh_receiver, decode_r_pub},
        kdf::{derive_k_dh, derive_k_dh_with_req},
    },
};

pub const OWNED_COUNT: usize = 32;
pub const NOISE_COUNT: usize = 512;
pub const TOTAL_COUNT: usize = OWNED_COUNT + NOISE_COUNT;

pub fn make_leaf(amount: u64, with_tag: bool) -> (ReceiverKeys, Asset) {
    let receiver_secret = ReceiverSecret::generate().expect("receiver secret");
    let receiver_keys = ReceiverKeys::from_receiver_secret(receiver_secret).expect("keys");
    let leaf = build_leaf_with_keys(&receiver_keys, amount, 17, with_tag);
    (receiver_keys, leaf)
}

fn build_leaf_with_keys(keys: &ReceiverKeys, amount: u64, mark: u8, with_tag: bool) -> Asset {
    let card = make_card(keys);

    let mut asset_id = [0u8; 32];
    asset_id[0] = mark;
    let output = make_output(&card, amount, &asset_id);
    let mut leaf = make_asset(amount, asset_id);

    set_output(&mut leaf, &output);
    leaf.tag16 = if with_tag { output.tag16 } else { None };
    leaf
}

pub fn scanner_assets() -> (ReceiverKeys, Vec<Asset>, Vec<Asset>, Vec<Tag16Context>) {
    let mut fast_assets = Vec::with_capacity(TOTAL_COUNT);
    let mut slow_assets = Vec::with_capacity(TOTAL_COUNT);
    let mut ctxs = Vec::with_capacity(OWNED_COUNT);

    let seed = ReceiverSecret::generate().expect("receiver secret");
    let keys = ReceiverKeys::from_receiver_secret(seed).expect("receiver keys");

    for idx in 0..OWNED_COUNT {
        let (leaf, k_dh) = owned_leaf(&keys, idx as u32);
        let r_pub = leaf.r_pub.expect("r_pub");
        let point = decode_r_pub(&r_pub).expect("decode r_pub");
        let dh = compute_dh_receiver(keys.reveal_view_sk(), &point).expect("dh");
        assert_eq!(k_dh, derive_k_dh(&dh));
        ctxs.push(Tag16Context { k_dh, req_id: None });
        fast_assets.push(leaf.clone());
        let mut slow_leaf = leaf;
        slow_leaf.tag16 = None;
        slow_assets.push(slow_leaf);
    }

    for idx in 0..NOISE_COUNT {
        let noise = noise_leaf(idx as u32);
        fast_assets.push(noise.clone());
        let mut slow_noise = noise;
        slow_noise.tag16 = None;
        slow_assets.push(slow_noise);
    }

    (keys, fast_assets, slow_assets, ctxs)
}

pub fn scan_fast(
    keys: &ReceiverKeys,
    leaves: &mut [Asset],
    ctxs: &[Tag16Context],
) -> (usize, Duration) {
    let mut scan = StealthOutputScanner::from_keys(keys);
    let start = std::time::Instant::now();
    let mut found = 0usize;

    for (idx, context) in ctxs.iter().enumerate() {
        let tag = leaves[idx].tag16.expect("tag16");
        scan.add_tag_context(tag, context.clone());
    }

    for leaf in leaves {
        if matches!(scan.scan_leaf(leaf), ScanResult::Mine { .. }) {
            found = found.saturating_add(1);
        }
    }

    (found, start.elapsed())
}

pub fn scan_slow(keys: &ReceiverKeys, leaves: &mut [Asset]) -> (usize, Duration) {
    let scan = StealthOutputScanner::from_keys(keys);
    let start = std::time::Instant::now();
    let mut found = 0usize;

    for leaf in leaves {
        if matches!(scan.scan_leaf(leaf), ScanResult::Mine { .. }) {
            found = found.saturating_add(1);
        }
    }

    (found, start.elapsed())
}

fn set_output(leaf: &mut Asset, output: &TxStealthOutput) {
    leaf.r_pub = Some(output.r_pub);
    leaf.owner_tag = Some(output.owner_tag);
    leaf.enc_pack = Some(output.enc_pack.clone());
    leaf.commitment = z00z_crypto::Commitment::from_bytes(&output.c_amount)
        .expect("canonical commitment")
        .0;
}

pub fn leaf_kdh(keys: &ReceiverKeys, leaf: &Asset) -> [u8; 32] {
    let point = decode_r_pub(&leaf.r_pub.expect("r_pub")).expect("decode");
    let dh = compute_dh_receiver(keys.reveal_view_sk(), &point).expect("dh");
    derive_k_dh(&dh)
}

pub fn leaf_req_kdh(keys: &ReceiverKeys, leaf: &Asset, req_id: &[u8; 32]) -> [u8; 32] {
    let point = decode_r_pub(&leaf.r_pub.expect("r_pub")).expect("decode");
    let dh = compute_dh_receiver(keys.reveal_view_sk(), &point).expect("dh");
    derive_k_dh_with_req(&dh, req_id)
}

pub fn scan_tag(
    tag16: u16,
    k_dh: [u8; 32],
    req_id: Option<[u8; 32]>,
    keys: &ReceiverKeys,
) -> StealthOutputScanner {
    let mut scan = StealthOutputScanner::from_keys(keys);
    scan.add_tag_context(tag16, Tag16Context { k_dh, req_id });
    scan.materialize_complete_tag_contexts(Vec::<(u16, Tag16Context)>::new());
    scan
}

pub fn make_req(keys: &ReceiverKeys, amount: u64) -> PaymentRequest {
    PaymentRequest::generate(
        keys,
        RequestParams {
            amount: Some(amount),
            expiry_seconds: 3_600,
            memo: None,
            payment_id: None,
        },
        1,
    )
    .expect("request")
}

pub fn make_own(
    card: &ReceiverCard,
    sender: &mut SenderWallet,
    asset_id: [u8; 32],
) -> (Vec<Asset>, HashSet<u16>) {
    let mut leaves = Vec::new();
    let mut tags = HashSet::new();

    for index in 0..10u32 {
        let output =
            build_tx_output_unchecked(card, None, sender, &[0xEE; 32], index, 100, &asset_id)
                .expect("my output");
        let tag16 = output.tag16.expect("tag16");
        tags.insert(tag16);

        let mut leaf = make_asset(100, asset_id);
        set_output(&mut leaf, &output);
        leaf.tag16 = Some(tag16);
        leaves.push(leaf);
    }

    (leaves, tags)
}

pub fn make_noise(
    sender: &mut SenderWallet,
    asset_id: [u8; 32],
    tags: &HashSet<u16>,
) -> Vec<Asset> {
    let mut leaves = Vec::new();
    let mut next_idx = 10u32;

    while leaves.len() < 1000 {
        let noise_secret = ReceiverSecret::generate().expect("noise secret");
        let noise_keys = ReceiverKeys::from_receiver_secret(noise_secret).expect("noise keys");
        let noise_card = make_card(&noise_keys);
        let output = build_tx_output_unchecked(
            &noise_card,
            None,
            sender,
            &[0xEE; 32],
            next_idx,
            200,
            &asset_id,
        )
        .expect("noise output");
        next_idx = next_idx.saturating_add(1);

        let tag16 = output.tag16.expect("noise tag16");
        if tags.contains(&tag16) {
            continue;
        }

        let mut leaf = make_asset(200, asset_id);
        set_output(&mut leaf, &output);
        leaf.tag16 = Some(tag16);
        leaves.push(leaf);
    }

    leaves
}

pub fn fill_cache(keys: &ReceiverKeys, leaves: &[Asset]) -> Tag16Cache {
    let mut cache = Tag16Cache::new();
    for leaf in leaves {
        cache.insert(
            leaf.tag16.expect("tag16"),
            Tag16Context {
                k_dh: leaf_kdh(keys, leaf),
                req_id: None,
            },
        );
    }
    cache
}

pub fn req_leaf(
    card: &ReceiverCard,
    sender: &mut SenderWallet,
    request: &PaymentRequest,
    digest: &[u8; 32],
    out_idx: u32,
    amount: u64,
    asset_id: [u8; 32],
) -> Asset {
    let output = build_tx_output_unchecked(
        card,
        Some(request),
        sender,
        digest,
        out_idx,
        amount,
        &asset_id,
    )
    .expect("output");

    let mut leaf = make_asset(amount, asset_id);
    set_output(&mut leaf, &output);
    leaf.tag16 = output.tag16;
    leaf
}

pub fn scan_digest(seed: u32) -> [u8; 32] {
    let mut digest = [0u8; 32];
    digest[0..4].copy_from_slice(&seed.to_le_bytes());
    digest[8..12].copy_from_slice(&seed.wrapping_mul(0x9E37_79B1).to_le_bytes());
    digest
}

pub fn make_card(keys: &ReceiverKeys) -> ReceiverCard {
    ReceiverCard {
        version: 1,
        owner_handle: keys.owner_handle,
        view_pk: keys.view_pk.as_bytes().try_into().expect("view pk"),
        identity_pk: keys.identity_pk.as_bytes().try_into().expect("identity pk"),
        card_id: None,
        metadata: None,
        signature: [0u8; 64],
    }
}

fn make_output(card: &ReceiverCard, amount: u64, asset_id: &[u8; 32]) -> TxStealthOutput {
    let mut sender_wallet = SenderWallet::new([8u8; 32]);

    build_tx_output_unchecked(
        card,
        None,
        &mut sender_wallet,
        &[9u8; 32],
        1,
        amount,
        asset_id,
    )
    .expect("output")
}

fn make_asset(amount: u64, asset_id: [u8; 32]) -> Asset {
    let mut asset =
        z00z_core::genesis::asset_std::asset_from_dev_cfg("z00z", 0, amount).expect("std asset");
    asset.leaf_ad_id = Some(asset_id);
    asset
}

fn owned_leaf(keys: &ReceiverKeys, idx: u32) -> (Asset, [u8; 32]) {
    let card = make_card(keys);
    let mut sender = SenderWallet::new([8u8; 32]);
    let mut asset_id = [0u8; 32];
    asset_id[0] = 17;
    let digest = scan_digest(idx);
    let amount = 100u64 + u64::from(idx % 7);

    let output =
        build_tx_output_unchecked(&card, None, &mut sender, &digest, idx, amount, &asset_id)
            .expect("owned output");

    let mut leaf = make_asset(amount, asset_id);
    set_output(&mut leaf, &output);
    leaf.tag16 = output.tag16;

    let point = decode_r_pub(&leaf.r_pub.expect("r_pub")).expect("decode");
    let dh = compute_dh_receiver(keys.reveal_view_sk(), &point).expect("dh");
    (leaf, derive_k_dh(&dh))
}

fn noise_leaf(idx: u32) -> Asset {
    let secret = ReceiverSecret::generate().expect("noise secret");
    let keys = ReceiverKeys::from_receiver_secret(secret).expect("noise keys");
    let card = make_card(&keys);
    let mut sender = SenderWallet::new([8u8; 32]);

    let mut asset_id = [0u8; 32];
    asset_id[0] = 17;
    let digest = scan_digest(1_000u32.saturating_add(idx));

    let output = build_tx_output_unchecked(
        &card,
        None,
        &mut sender,
        &digest,
        1_000u32.saturating_add(idx),
        200,
        &asset_id,
    )
    .expect("noise output");

    let mut leaf = make_asset(200, asset_id);
    set_output(&mut leaf, &output);
    leaf.tag16 = output.tag16;
    leaf
}
