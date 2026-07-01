use z00z_core::{assets::AssetLeaf, genesis::asset_std::asset_from_dev_class, Asset, AssetClass};
use z00z_crypto::{create_range_proof, Z00ZScalar};
use z00z_wallets::{
    build_tx_output_unchecked,
    key::{ReceiverKeys, ReceiverSecret},
    receiver::{receiver_scan_leaf, receiver_scan_report, ReceiverCard, Tag16Context},
    receiver::{ReceiveNext, ReceiveStatus, ScanResult, StealthOutputScanner},
    stealth::ecdh::{compute_dh_receiver, decode_r_pub},
    stealth::kdf::derive_k_dh,
    tx::wire_decrypt_leaf,
    validate_output_self, SenderValidationCtx, SenderWallet, TagMode,
};

#[path = "test_inc/test_payment_request_scan.inc"]
mod asset_scan_support;

use asset_scan_support::{leaf_req_kdh, make_req, req_leaf};

struct RecvBase {
    keys: ReceiverKeys,
    leaf: AssetLeaf,
    asset: Asset,
}

fn make_keys(seed: [u8; 32]) -> ReceiverKeys {
    let secret = ReceiverSecret::from_bytes(seed).expect("secret");
    ReceiverKeys::from_receiver_secret(secret).expect("keys")
}

fn make_card(keys: &ReceiverKeys) -> ReceiverCard {
    let card = keys.export_receiver_card().expect("card");
    card.verify().expect("verify");
    card
}

fn build_asset_from_leaf(leaf: &AssetLeaf, amount: u64) -> Asset {
    let mut asset = asset_from_dev_class(AssetClass::Coin, leaf.serial_id, amount).expect("asset");
    asset.leaf_ad_id = Some(leaf.asset_id);
    asset.commitment = z00z_crypto::Commitment::from_bytes(&leaf.c_amount)
        .expect("commitment")
        .0;
    asset.owner_pub = None;
    asset.owner_signature = None;
    asset.r_pub = Some(leaf.r_pub);
    asset.owner_tag = Some(leaf.owner_tag);
    asset.enc_pack = Some(leaf.enc_pack.clone());
    asset.tag16 = Some(leaf.tag16);
    asset.range_proof = Some(leaf.range_proof.clone());
    asset
}

fn make_base(seed: u8, amount: u64, serial_id: u32) -> RecvBase {
    let keys = make_keys([seed; 32]);
    let card = make_card(&keys);
    let aid = [seed; 32];
    let mut sender = SenderWallet::new([seed ^ 0x44; 32]);
    let out = build_tx_output_unchecked(
        &card,
        None,
        &mut sender,
        &[seed; 32],
        serial_id,
        amount,
        &aid,
    )
    .expect("output");
    let mut leaf = AssetLeaf {
        asset_id: aid,
        serial_id: 0,
        r_pub: out.r_pub,
        owner_tag: out.owner_tag,
        c_amount: out.c_amount,
        enc_pack: out.enc_pack,
        range_proof: Vec::new(),
        tag16: out.tag16.expect("tag16"),
    };
    let pack = receiver_scan_leaf(&keys, &leaf)
        .expect("scan")
        .expect("owned pack");
    let blind = Z00ZScalar::try_from_bytes(pack.blinding).expect("blinding");
    leaf.range_proof = create_range_proof(amount, &blind, 64, 0).expect("proof");
    let asset = build_asset_from_leaf(&leaf, amount);
    RecvBase { keys, leaf, asset }
}

#[test]
fn test_ex1_leaf_walk() {
    let base = make_base(0x91, 321, 7);
    let pack = receiver_scan_leaf(&base.keys, &base.leaf)
        .expect("scan")
        .expect("owned pack");
    let report = receiver_scan_report(&base.keys, &base.leaf).expect("report");
    let line = format!(
        "status={} next={:?} amount={} asset_id={} serial_id={}",
        report.status.rpc_code(),
        report.next,
        pack.value,
        hex::encode(base.leaf.asset_id),
        base.leaf.serial_id
    );

    assert_eq!(pack.value, 321);
    assert_eq!(report.status, ReceiveStatus::Detected);
    assert_eq!(report.next, ReceiveNext::ReportOnly);
    assert!(
        !report.next.should_persist(),
        "leaf example must stop before claim persistence"
    );
    assert!(line.contains("RECEIVE_DETECTED"));
    assert!(line.contains("amount=321"));
}

#[test]
fn test_ex2_runtime_scan() {
    let base = make_base(0x92, 654, 8);
    let pack = receiver_scan_leaf(&base.keys, &base.leaf)
        .expect("scan")
        .expect("owned pack");
    let canon = receiver_scan_report(&base.keys, &base.leaf).expect("canon");
    let runtime_leaf = wire_decrypt_leaf(&z00z_core::AssetWire::from_asset(&base.asset))
        .expect("runtime decrypt leaf");

    assert_eq!(runtime_leaf.asset_id, base.leaf.asset_id);
    assert_eq!(base.asset.serial_id, base.leaf.serial_id);
    assert_eq!(base.asset.r_pub, Some(base.leaf.r_pub));
    assert_eq!(base.asset.owner_tag, Some(base.leaf.owner_tag));
    assert_eq!(base.asset.enc_pack, Some(base.leaf.enc_pack.clone()));
    assert_eq!(base.asset.tag16, Some(base.leaf.tag16));
    assert_eq!(base.asset.commitment.as_bytes(), &base.leaf.c_amount);

    base.asset
        .validate_stealth_consistency()
        .expect("runtime stealth tuple");
    let scan = StealthOutputScanner::from_keys(&base.keys).scan_leaf(&base.asset);
    let report = scan.recv_report();
    let line = format!(
        "runtime={} canon={}",
        report.status.rpc_code(),
        canon.status.rpc_code()
    );

    assert_eq!(report, canon);
    assert_eq!(report.status, ReceiveStatus::Detected);
    assert_eq!(report.reject, None);
    assert_eq!(report.next, ReceiveNext::ReportOnly);
    assert!(line.contains("runtime=RECEIVE_DETECTED"));
    assert!(line.contains("canon=RECEIVE_DETECTED"));

    let ScanResult::Mine { wallet_output } = scan else {
        panic!("runtime asset must scan as Mine");
    };

    assert_eq!(wallet_output.amount, 654);
    assert_eq!(wallet_output.asset_id, base.asset.asset_id());
    assert_eq!(wallet_output.serial_id, base.leaf.serial_id);
    assert_eq!(wallet_output.asset_secret, Some(pack.s_out));
    assert_eq!(wallet_output.blinding, Some(pack.blinding));
    assert_eq!(wallet_output.r_pub, base.leaf.r_pub);
    assert_eq!(wallet_output.owner_tag, base.leaf.owner_tag);
}

struct BatchLeaf {
    label: &'static str,
    leaf: AssetLeaf,
    amount: u64,
}

fn batch_rows() -> Vec<BatchLeaf> {
    let keys = [
        make_keys([0xC1; 32]),
        make_keys([0xD1; 32]),
        make_keys([0xE1; 32]),
    ];
    let plan = [
        ("bob-1", 0usize, 1_100u64, [0xA1; 32]),
        ("carol-1", 1usize, 2_200u64, [0xB1; 32]),
        ("dave-1", 2usize, 3_300u64, [0xC1; 32]),
        ("bob-2", 0usize, 1_400u64, [0xA2; 32]),
        ("carol-2", 1usize, 2_500u64, [0xB2; 32]),
    ];
    let mut sender = SenderWallet::new([0xB1; 32]);
    let mut rows = Vec::with_capacity(plan.len());

    for (idx, (label, who, amount, aid)) in plan.into_iter().enumerate() {
        let card = make_card(&keys[who]);
        let out = build_tx_output_unchecked(
            &card,
            None,
            &mut sender,
            &[0xA5; 32],
            idx as u32,
            amount,
            &aid,
        )
        .expect("output");
        rows.push(BatchLeaf {
            label,
            amount,
            leaf: AssetLeaf {
                asset_id: aid,
                serial_id: 0,
                r_pub: out.r_pub,
                owner_tag: out.owner_tag,
                c_amount: out.c_amount,
                enc_pack: out.enc_pack,
                range_proof: Vec::new(),
                tag16: out.tag16.expect("tag16"),
            },
        });
    }

    rows
}

fn owned_rows(keys: &ReceiverKeys, rows: &[BatchLeaf]) -> Vec<&'static str> {
    let mut out = Vec::new();

    for row in rows {
        let report = receiver_scan_report(keys, &row.leaf).expect("report");
        match receiver_scan_leaf(keys, &row.leaf).expect("scan") {
            Some(pack) => {
                assert_eq!(report.status, ReceiveStatus::Detected);
                assert_eq!(pack.value, row.amount);
                out.push(row.label);
            }
            None => assert_eq!(report.status, ReceiveStatus::NotMine),
        }
    }

    out
}

#[test]
fn test_ex3_batch_scan() {
    let rows = batch_rows();
    let bob = make_keys([0xC1; 32]);
    let carol = make_keys([0xD1; 32]);
    let dave = make_keys([0xE1; 32]);
    let erin = make_keys([0xF1; 32]);

    let bob_rows = owned_rows(&bob, &rows);
    let carol_rows = owned_rows(&carol, &rows);
    let dave_rows = owned_rows(&dave, &rows);
    let erin_rows = owned_rows(&erin, &rows);

    assert_eq!(bob_rows, vec!["bob-1", "bob-2"]);
    assert_eq!(carol_rows, vec!["carol-1", "carol-2"]);
    assert_eq!(dave_rows, vec!["dave-1"]);
    assert!(erin_rows.is_empty());
    assert_eq!(
        bob_rows.len() + carol_rows.len() + dave_rows.len() + erin_rows.len(),
        rows.len()
    );
}

fn demo_ctx(
    keys: &ReceiverKeys,
    out: &z00z_wallets::TxStealthOutput,
    aid: [u8; 32],
) -> SenderValidationCtx {
    let r_pub = decode_r_pub(&out.r_pub).expect("r_pub");
    let dh = compute_dh_receiver(keys.reveal_view_sk(), &r_pub).expect("dh");
    SenderValidationCtx {
        k_dh: derive_k_dh(&dh),
        owner_handle: keys.owner_handle,
        asset_id: aid,
        serial_id: 0,
        tag_mode: TagMode::CardBound,
    }
}

#[test]
fn test_ex4_tamper_demo() {
    let keys = make_keys([0xF2; 32]);
    let card = make_card(&keys);
    let aid = [0xE2; 32];
    let mut sender = SenderWallet::new([0xB2; 32]);
    let output = build_tx_output_unchecked(&card, None, &mut sender, &[0xC4; 32], 0, 909, &aid)
        .expect("output");
    let ctx = demo_ctx(&keys, &output, aid);

    validate_output_self(&output, &ctx, 909).expect("baseline valid");
    let good = AssetLeaf {
        asset_id: aid,
        serial_id: 0,
        r_pub: output.r_pub,
        owner_tag: output.owner_tag,
        c_amount: output.c_amount,
        enc_pack: output.enc_pack.clone(),
        range_proof: Vec::new(),
        tag16: output.tag16.expect("tag16"),
    };
    let good_pack = receiver_scan_leaf(&keys, &good).expect("good scan");
    let good_report = receiver_scan_report(&keys, &good).expect("good report");

    let mut bad = output.clone();
    bad.enc_pack.ciphertext[0] ^= 1;
    let bad_leaf = AssetLeaf {
        asset_id: aid,
        serial_id: 0,
        r_pub: bad.r_pub,
        owner_tag: bad.owner_tag,
        c_amount: bad.c_amount,
        enc_pack: bad.enc_pack.clone(),
        range_proof: Vec::new(),
        tag16: bad.tag16.expect("tag16"),
    };
    let bad_err = validate_output_self(&bad, &ctx, 909).expect_err("tamper err");
    let bad_pack = receiver_scan_leaf(&keys, &bad_leaf).expect("bad scan");
    let bad_report = receiver_scan_report(&keys, &bad_leaf).expect("bad report");

    assert_eq!(good_report.status, ReceiveStatus::Detected);
    assert_eq!(good_pack.expect("good pack").value, 909);
    assert_eq!(bad_err, z00z_wallets::StealthError::InvalidStealthInput);
    assert!(bad_pack.is_none());
    assert_eq!(bad_report.status, ReceiveStatus::InvalidProof);
    assert_eq!(bad.tag16, output.tag16);
}

#[test]
fn test_ex5_req_merchant() {
    let keys = make_keys([0xA1; 32]);
    let card = make_card(&keys);
    let req_a = make_req(&keys, 707);
    let req_b = make_req(&keys, 707);
    assert_ne!(req_a.req_id, req_b.req_id);
    let mut sender = SenderWallet::new([0x41; 32]);
    let asset_id = [0x55; 32];
    let leaf_a = req_leaf(&card, &mut sender, &req_a, &[0xA1; 32], 0, 707, asset_id);
    let leaf_b = req_leaf(&card, &mut sender, &req_b, &[0xB2; 32], 1, 707, asset_id);

    let mut scan = StealthOutputScanner::from_keys(&keys);
    scan.add_request(&req_a);

    assert!(matches!(scan.scan_leaf(&leaf_a), ScanResult::Mine { .. }));
    assert!(matches!(scan.scan_leaf(&leaf_b), ScanResult::NotMine));
    assert!(matches!(
        scan.scan_leaf_tag_only(&leaf_a),
        ScanResult::NotMine
    ));

    let mut strict = StealthOutputScanner::from_keys(&keys);
    strict.add_request(&req_a);
    strict.add_tag_context(
        leaf_a.tag16.expect("tag16"),
        Tag16Context {
            k_dh: leaf_req_kdh(&keys, &leaf_a, &req_a.req_id),
            req_id: Some(req_a.req_id),
        },
    );
    strict.materialize_complete_tag_contexts(Vec::<(u16, Tag16Context)>::new());

    let owned = strict.scan_leaf(&leaf_a).recv_report();
    let other = strict.scan_leaf(&leaf_b).recv_report();

    assert!(matches!(
        strict.scan_leaf_tag_only(&leaf_a),
        ScanResult::Mine { .. }
    ));
    assert!(matches!(
        strict.scan_leaf_tag_only(&leaf_b),
        ScanResult::NotMine
    ));
    assert_eq!(owned.status.rpc_code(), "RECEIVE_DETECTED");
    assert_eq!(other.status, ReceiveStatus::NotMine);
}

#[test]
fn test_ex6_tamper_demo() {
    let base = make_base(0xB1, 777, 9);
    let good = receiver_scan_report(&base.keys, &base.leaf).expect("good report");
    let mut bad = base.leaf.clone();
    bad.enc_pack.ciphertext[0] ^= 1;

    let bad_pack = receiver_scan_leaf(&base.keys, &bad).expect("bad scan");
    let bad_report = receiver_scan_report(&base.keys, &bad).expect("bad report");
    let bad_asset = build_asset_from_leaf(&bad, 777);
    let run = StealthOutputScanner::from_keys(&base.keys).scan_leaf(&bad_asset);
    let line = format!(
        "good={} bad={}",
        good.status.rpc_code(),
        bad_report.status.rpc_code()
    );

    assert_eq!(good.status, ReceiveStatus::Detected);
    assert!(bad_pack.is_none());
    assert_eq!(bad_report.status, ReceiveStatus::InvalidProof);
    assert_eq!(bad_report.next, ReceiveNext::ReportOnly);
    assert!(!bad_report.next.should_persist());
    assert!(!matches!(run, ScanResult::Mine { .. }));
    assert_eq!(run.recv_report().status, bad_report.status);
    assert!(line.contains("good=RECEIVE_DETECTED"));
    assert!(line.contains("bad=RECEIVE_INVALID_PROOF"));
}
