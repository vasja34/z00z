use std::time::Instant;

#[path = "test_inc/test_mod.rs"]
mod test_common;

use test_common::managed_test_output_root;
use z00z_core::{
    genesis::asset_std::{asset_from_dev_cfg, def_from_dev_cfg},
    Asset,
};
use z00z_utils::io::{create_dir_all, write_file};
use z00z_wallets::{
    key::{ReceiverKeys, ReceiverSecret},
    stealth::ecdh::{compute_dh_receiver, decode_r_pub},
    stealth::kdf::derive_k_dh,
    stealth::{build_tx_output_unchecked, SenderWallet, TxStealthOutput},
};
use z00z_wallets::{
    receiver::Tag16Context,
    receiver::{ScanResult, StealthOutputScanner},
};

const OWN_N: usize = 20;
const FOR_N: usize = 200;
const BAD_N: usize = 20;

#[derive(Clone, Copy)]
enum Kind {
    Own,
    For,
    Bad,
}

struct Row {
    kind: Kind,
    leaf: Asset,
}

#[derive(Default)]
struct Cmx {
    tp: usize,
    fp: usize,
    tn: usize,
    fnn: usize,
}

#[derive(Default)]
struct PathCnt {
    pre_mine: usize,
    pre_not: usize,
    pre_maybe: usize,
    full_mine: usize,
    full_not: usize,
    full_maybe: usize,
    bad_early: usize,
}

fn is_mine(res: &ScanResult) -> bool {
    matches!(res, ScanResult::Mine { .. })
}

fn count_path(path: &mut PathCnt, pre: &ScanResult, full: &ScanResult, kind: Kind) {
    match pre {
        ScanResult::Mine { .. } => path.pre_mine += 1,
        ScanResult::NotMine => path.pre_not += 1,
        ScanResult::MaybeMine { .. } => path.pre_maybe += 1,
    }

    match full {
        ScanResult::Mine { .. } => path.full_mine += 1,
        ScanResult::NotMine => path.full_not += 1,
        ScanResult::MaybeMine { .. } => path.full_maybe += 1,
    }

    if matches!(kind, Kind::Bad) && matches!(pre, ScanResult::NotMine) {
        path.bad_early += 1;
    }
}

fn upd_cmx(cmx: &mut Cmx, kind: Kind, pre: &ScanResult) {
    let pred_pos = is_mine(pre);
    let actual_pos = matches!(kind, Kind::Own);

    match (actual_pos, pred_pos) {
        (true, true) => cmx.tp += 1,
        (true, false) => cmx.fnn += 1,
        (false, true) => cmx.fp += 1,
        (false, false) => cmx.tn += 1,
    }
}

fn pick_final<'a>(pre: &'a ScanResult, full: &'a ScanResult) -> &'a ScanResult {
    if matches!(pre, ScanResult::MaybeMine { .. }) {
        return full;
    }
    pre
}

fn mk_asset(aid: [u8; 32], amount: u64) -> Asset {
    let asset = asset_from_dev_cfg("z00z", 0, amount).expect("std asset");
    assert_eq!(asset.definition.id, aid, "asset id mismatch");
    asset
}

fn bind_out(out: &TxStealthOutput, aid: [u8; 32], amount: u64) -> Asset {
    let mut leaf = mk_asset(aid, amount);
    leaf.commitment = z00z_crypto::Commitment::from_bytes(&out.c_amount)
        .expect("commitment")
        .0;
    leaf.r_pub = Some(out.r_pub);
    leaf.owner_tag = Some(out.owner_tag);
    leaf.enc_pack = Some(out.enc_pack.clone());
    leaf.tag16 = out.tag16;
    leaf.leaf_ad_id = Some(aid);
    leaf
}

fn add_ctx(scanner: &mut StealthOutputScanner, keys: &ReceiverKeys, out: &TxStealthOutput) {
    let rp = decode_r_pub(&out.r_pub).expect("decode r_pub");
    let dh = compute_dh_receiver(keys.reveal_view_sk(), &rp).expect("ecdh");
    let k_dh = derive_k_dh(&dh);
    scanner.add_tag_context(
        out.tag16.expect("tag16 present"),
        Tag16Context { k_dh, req_id: None },
    );
}

fn mk_owned(
    scanner: &mut StealthOutputScanner,
    keys: &ReceiverKeys,
    card: &z00z_wallets::receiver::ReceiverCard,
    sender: &mut SenderWallet,
    aid: [u8; 32],
) -> Vec<Row> {
    let mut out = Vec::with_capacity(OWN_N);
    for idx in 0..OWN_N {
        let txd = [0x31u8; 32];
        let tx = build_tx_output_unchecked(
            card,
            None,
            sender,
            &txd,
            idx as u32,
            1_000 + idx as u64,
            &aid,
        )
        .expect("owned output");
        add_ctx(scanner, keys, &tx);
        out.push(Row {
            kind: Kind::Own,
            leaf: bind_out(&tx, aid, 1_000 + idx as u64),
        });
    }
    out
}

fn mk_foreign(
    card: &z00z_wallets::receiver::ReceiverCard,
    sender: &mut SenderWallet,
    aid: [u8; 32],
) -> Vec<Row> {
    let mut out = Vec::with_capacity(FOR_N);
    for idx in 0..FOR_N {
        let txd = [0x73u8; 32];
        let tx = build_tx_output_unchecked(
            card,
            None,
            sender,
            &txd,
            idx as u32,
            2_000 + idx as u64,
            &aid,
        )
        .expect("foreign output");
        out.push(Row {
            kind: Kind::For,
            leaf: bind_out(&tx, aid, 2_000 + idx as u64),
        });
    }
    out
}

fn mk_bad(base: &Asset) -> Vec<Row> {
    let mut out = Vec::with_capacity(BAD_N);
    for idx in 0..BAD_N {
        let mut leaf = base.clone();
        match idx % 4 {
            0 => {
                leaf.r_pub = None;
            }
            1 => {
                leaf.owner_tag = None;
            }
            2 => {
                leaf.enc_pack = None;
            }
            _ => {
                leaf.r_pub = None;
                leaf.tag16 = Some(0xAA55);
            }
        }
        out.push(Row {
            kind: Kind::Bad,
            leaf,
        });
    }
    out
}

fn write_sum(sum: &str) {
    let out = managed_test_output_root("e2e03");
    create_dir_all(&out).expect("mkdir outputs/tests/e2e03");
    write_file(out.join("e2e03_metrics.txt"), sum.as_bytes()).expect("write e2e03 metrics");
}

#[test]
fn test_stage4_prefilter() {
    let own_sec = ReceiverSecret::from_bytes([0x11u8; 32]).expect("own sec");
    let for_sec = ReceiverSecret::from_bytes([0x22u8; 32]).expect("for sec");
    let own_keys = ReceiverKeys::from_receiver_secret(own_sec).expect("own keys");
    let for_keys = ReceiverKeys::from_receiver_secret(for_sec).expect("for keys");

    let own_card = own_keys.export_receiver_card().expect("own card");
    let for_card = for_keys.export_receiver_card().expect("for card");
    let mut sender = SenderWallet::new([0x44u8; 32]);
    let aid = def_from_dev_cfg("z00z").expect("std def").id;

    let mut scanner = StealthOutputScanner::from_keys(&own_keys);

    let mut rows = mk_owned(&mut scanner, &own_keys, &own_card, &mut sender, aid);
    scanner.materialize_complete_tag_contexts(Vec::<(u16, Tag16Context)>::new());
    let mut foreign = mk_foreign(&for_card, &mut sender, aid);
    let base_bad = foreign.first().expect("foreign seed").leaf.clone();
    let mut bad = mk_bad(&base_bad);
    rows.append(&mut foreign);
    rows.append(&mut bad);

    let own_count = rows
        .iter()
        .filter(|row| matches!(row.kind, Kind::Own))
        .count();
    let for_count = rows
        .iter()
        .filter(|row| matches!(row.kind, Kind::For))
        .count();
    let bad_count = rows
        .iter()
        .filter(|row| matches!(row.kind, Kind::Bad))
        .count();
    assert!(own_count >= OWN_N, "owned count must be >= 20");
    assert!(for_count >= FOR_N, "foreign count must be >= 200");
    assert!(bad_count >= BAD_N, "malformed count must be >= 20");

    let mut cmx = Cmx::default();
    let mut path = PathCnt::default();

    let pre_t0 = Instant::now();
    let pre_res: Vec<ScanResult> = rows
        .iter()
        .map(|row| scanner.scan_leaf_tag_only(&row.leaf))
        .collect();
    let pre_dt = pre_t0.elapsed();

    let full_t0 = Instant::now();
    let full_res: Vec<ScanResult> = rows
        .iter()
        .map(|row| scanner.scan_leaf(&row.leaf))
        .collect();
    let full_dt = full_t0.elapsed();

    for (idx, row) in rows.iter().enumerate() {
        let pre = &pre_res[idx];
        let full = &full_res[idx];
        let fin = pick_final(pre, full);
        upd_cmx(&mut cmx, row.kind, fin);
        count_path(&mut path, pre, full, row.kind);

        if matches!(row.kind, Kind::Bad) {
            assert!(
                matches!(pre, ScanResult::NotMine),
                "bad must fail before decode"
            );
            assert!(
                matches!(full, ScanResult::NotMine),
                "bad must stay rejected in full"
            );
        }
    }

    assert_eq!(cmx.fp, 0, "FP must be zero");
    assert_eq!(cmx.fnn, 0, "FN must be zero");
    assert_eq!(path.bad_early, bad_count, "all malformed must early reject");

    let mut sum = String::new();
    sum.push_str("E2E-03 summary\n");
    sum.push_str(&format!(
        "counts own={own_count} foreign={for_count} bad={bad_count}\n"
    ));
    sum.push_str(&format!(
        "cmx tp={} fp={} tn={} fn={}\n",
        cmx.tp, cmx.fp, cmx.tn, cmx.fnn
    ));
    sum.push_str(&format!(
        "pre mine={} not={} maybe={}\n",
        path.pre_mine, path.pre_not, path.pre_maybe
    ));
    sum.push_str(&format!(
        "full mine={} not={} maybe={}\n",
        path.full_mine, path.full_not, path.full_maybe
    ));
    sum.push_str(&format!("bad_early={}\n", path.bad_early));
    sum.push_str(&format!("pre_ms={}\n", pre_dt.as_millis()));
    sum.push_str(&format!("full_ms={}\n", full_dt.as_millis()));
    write_sum(&sum);
}
