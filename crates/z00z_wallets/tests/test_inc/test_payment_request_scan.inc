use z00z_core::Asset;
use z00z_wallets::{
    key::ReceiverKeys,
    receiver::{PaymentRequest, ReceiverCard, RequestParams},
    stealth::{build_tx_output_unchecked, SenderWallet, TxStealthOutput},
    stealth::{
        ecdh::{compute_dh_receiver, decode_r_pub},
        kdf::derive_k_dh_with_req,
    },
};

pub fn leaf_req_kdh(keys: &ReceiverKeys, leaf: &Asset, req_id: &[u8; 32]) -> [u8; 32] {
    let point = decode_r_pub(&leaf.r_pub.expect("r_pub")).expect("decode");
    let dh = compute_dh_receiver(keys.reveal_view_sk(), &point).expect("dh");
    derive_k_dh_with_req(&dh, req_id)
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

fn set_output(leaf: &mut Asset, output: &TxStealthOutput) {
    leaf.r_pub = Some(output.r_pub);
    leaf.owner_tag = Some(output.owner_tag);
    leaf.enc_pack = Some(output.enc_pack.clone());
    leaf.commitment = z00z_crypto::Commitment::from_bytes(&output.c_amount)
        .expect("canonical commitment")
        .0;
}

fn make_asset(amount: u64, asset_id: [u8; 32]) -> Asset {
    let mut asset =
        z00z_core::genesis::asset_std::asset_from_dev_cfg("z00z", 0, amount).expect("std asset");
    asset.leaf_ad_id = Some(asset_id);
    asset
}
