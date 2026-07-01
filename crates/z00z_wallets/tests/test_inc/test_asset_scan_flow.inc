use z00z_core::Asset;
use z00z_wallets::{
    key::{ReceiverKeys, ReceiverSecret},
    receiver::ReceiverCard,
    receiver::{ScanResult, StealthOutputScanner},
    stealth::{build_tx_output_unchecked, SenderWallet, TxStealthOutput},
};

pub fn make_leaf(amount: u64, with_tag: bool) -> (ReceiverKeys, Asset) {
    let receiver_secret = ReceiverSecret::generate().expect("receiver secret");
    let receiver_keys = ReceiverKeys::from_receiver_secret(receiver_secret).expect("keys");
    let leaf = build_leaf_with_keys(&receiver_keys, amount, 17, with_tag);
    (receiver_keys, leaf)
}

pub fn set_output(leaf: &mut Asset, output: &TxStealthOutput) {
    leaf.r_pub = Some(output.r_pub);
    leaf.owner_tag = Some(output.owner_tag);
    leaf.enc_pack = Some(output.enc_pack.clone());
    leaf.commitment = z00z_crypto::Commitment::from_bytes(&output.c_amount)
        .expect("canonical commitment")
        .0;
}

pub fn build_outputs(
    card: &ReceiverCard,
    sender: &mut SenderWallet,
    digest: &[u8; 32],
    range: std::ops::Range<u32>,
    base_amount: u64,
    asset_id: [u8; 32],
) -> Vec<TxStealthOutput> {
    range
        .map(|index| {
            build_tx_output_unchecked(
                card,
                None,
                sender,
                digest,
                index,
                base_amount + u64::from(index),
                &asset_id,
            )
            .expect("rotation output")
        })
        .collect()
}

pub fn count_mine_hits(
    scan: &StealthOutputScanner,
    outputs: &[TxStealthOutput],
    base_amount: u64,
    asset_id: [u8; 32],
    base_index: u64,
) -> usize {
    outputs
        .iter()
        .enumerate()
        .filter(|(idx, output)| {
            let mut leaf = make_asset(base_amount + base_index + *idx as u64, asset_id);
            set_output(&mut leaf, output);
            matches!(scan.scan_leaf(&leaf), ScanResult::Mine { .. })
        })
        .count()
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

pub fn make_asset(amount: u64, asset_id: [u8; 32]) -> Asset {
    let mut asset =
        z00z_core::genesis::asset_std::asset_from_dev_cfg("z00z", 0, amount).expect("std asset");
    asset.leaf_ad_id = Some(asset_id);
    asset
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
