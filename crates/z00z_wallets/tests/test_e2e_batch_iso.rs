//! E2E Phase 3 section 6 mixed-batch ownership isolation.
//! Batch order is interleaved on purpose:
//! `bob-11, carol-12, dave-13, bob-14, dave-16, carol-15`.

use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;

use z00z_core::{genesis::asset_std::asset_from_dev_class, Asset};
use z00z_wallets::{
    build_tx_output_unchecked,
    key::{ReceiverKeys, ReceiverSecret},
    receiver::{ReceiveStatus, ReceiverCard, ScanResult, StealthOutputScanner},
    SenderWallet, TxStealthOutput,
};

struct OutRec {
    name: &'static str,
    serial: u32,
    asset: Asset,
}

fn mk_keys(seed: u8) -> ReceiverKeys {
    let mut buf = [seed; 32];
    buf[31] ^= 0x5A;
    let sec = ReceiverSecret::from_bytes(buf).expect("secret");
    ReceiverKeys::from_receiver_secret(sec).expect("keys")
}

fn mk_card(keys: &ReceiverKeys) -> ReceiverCard {
    let card = keys.export_receiver_card().expect("card");
    card.verify().expect("verify");
    card
}

fn mk_asset(amount: u64, aid: [u8; 32], out: &TxStealthOutput) -> Asset {
    let mut asset = asset_from_dev_class(z00z_core::AssetClass::Coin, 0, amount).expect("asset");
    let mut def = (*asset.definition).clone();
    def.id = aid;
    asset.definition = Arc::new(def);
    asset.commitment = z00z_crypto::Commitment::from_bytes(&out.c_amount)
        .expect("commitment")
        .0;
    asset.owner_pub = None;
    asset.owner_signature = None;
    asset.r_pub = Some(out.r_pub);
    asset.owner_tag = Some(out.owner_tag);
    asset.enc_pack = Some(out.enc_pack.clone());
    asset.tag16 = out.tag16;
    asset.leaf_ad_id = Some(asset.definition.id);
    asset
}

fn mk_out(name: &'static str, keys: &ReceiverKeys, amount: u64, serial: u32) -> OutRec {
    let card = mk_card(keys);
    let mut aid = [0u8; 32];
    aid[0] = serial as u8;
    let mut sender = SenderWallet::new([serial as u8; 32]);
    let out = build_tx_output_unchecked(
        &card,
        None,
        &mut sender,
        &[serial as u8; 32],
        serial,
        amount,
        &aid,
    )
    .expect("output");
    let asset = mk_asset(amount, aid, &out);
    OutRec {
        name,
        serial,
        asset,
    }
}

fn own_set(keys: &ReceiverKeys, batch: &[OutRec]) -> BTreeSet<u32> {
    let scan = StealthOutputScanner::from_keys(keys);
    let mut out = BTreeSet::new();

    for item in batch {
        let hit = scan.scan_leaf(&item.asset);
        match hit {
            ScanResult::Mine { .. } => {
                out.insert(item.serial);
            }
            _ => {
                assert_eq!(hit.recv_report().status, ReceiveStatus::NotMine);
            }
        }
    }

    out
}

fn want_set(list: &[u32]) -> BTreeSet<u32> {
    list.iter().copied().collect()
}

fn count_hits(sets: &[&BTreeSet<u32>]) -> BTreeMap<u32, usize> {
    let mut out = BTreeMap::new();
    for set in sets {
        for serial in *set {
            *out.entry(*serial).or_insert(0) += 1;
        }
    }
    out
}

#[test]
fn test_e2e_batch_iso() {
    let bob = mk_keys(0x11);
    let carol = mk_keys(0x22);
    let dave = mk_keys(0x33);
    let eve = mk_keys(0x44);

    let batch = vec![
        mk_out("bob", &bob, 501, 11),
        mk_out("carol", &carol, 502, 12),
        mk_out("dave", &dave, 503, 13),
        mk_out("bob", &bob, 504, 14),
        mk_out("dave", &dave, 506, 16),
        mk_out("carol", &carol, 505, 15),
    ];

    let bob_set = own_set(&bob, &batch);
    let carol_set = own_set(&carol, &batch);
    let dave_set = own_set(&dave, &batch);
    let eve_set = own_set(&eve, &batch);

    assert_eq!(bob_set, want_set(&[11, 14]));
    assert_eq!(carol_set, want_set(&[12, 15]));
    assert_eq!(dave_set, want_set(&[13, 16]));
    assert!(eve_set.is_empty());

    let all = count_hits(&[&bob_set, &carol_set, &dave_set]);
    assert_eq!(all.len(), batch.len());
    assert!(all.values().all(|count| *count == 1));

    let got: BTreeSet<u32> = all.keys().copied().collect();
    let want: BTreeSet<u32> = batch.iter().map(|item| item.serial).collect();
    assert_eq!(got, want);

    for item in &batch {
        match item.name {
            "bob" => assert!(bob_set.contains(&item.serial)),
            "carol" => assert!(carol_set.contains(&item.serial)),
            "dave" => assert!(dave_set.contains(&item.serial)),
            _ => panic!("unexpected owner"),
        }
    }
}
