use z00z_core::assets::AssetPackPlain;
use z00z_crypto::{
    create_commitment,
    protocol::zkpack::{ZKPACK_CT_LEN, ZKPACK_TAG_LEN, ZKPACK_VER},
    Z00ZScalar, ZkPackEncrypted,
};

use crate::{
    key::{ReceiverKeys, ReceiverSecret},
    receiver::{DetectedAssetPack, PaymentRequest, ReceiverCard, RequestParams},
    stealth::{
        build_tx_output_unchecked, compute_dh_receiver, decode_r_pub, derive_k_dh,
        derive_k_dh_with_req, SenderWallet,
    },
};

use super::{
    ordered_request_candidates, scan_cached_keys, scan_owned, verify_pack_commitment, DetectFail,
    DetectState, ScanInput,
};

#[test]
fn test_verify_commitment_blinding_parse() {
    let input = dummy_input(valid_commitment_bytes());
    let pack = DetectedAssetPack::from_decoded(z00z_core::assets::DecodedAssetPack::Basic(
        AssetPackPlain {
            value: 7,
            blinding: [0xFF; 32],
            s_out: [3u8; 32],
        },
    ));

    let err = verify_pack_commitment(&input, &pack).expect_err("invalid blinding must fail");
    assert_eq!(err, DetectFail::Parse("invalid blinding"));
}

#[test]
fn test_verify_commitment_bytes_parse() {
    let input = dummy_input([0xFF; 32]);
    let pack = DetectedAssetPack::from_decoded(z00z_core::assets::DecodedAssetPack::Basic(
        AssetPackPlain {
            value: 7,
            blinding: [0u8; 32],
            s_out: [3u8; 32],
        },
    ));

    let err = verify_pack_commitment(&input, &pack).expect_err("invalid commitment must fail");
    assert_eq!(err, DetectFail::Parse("invalid commitment"));
}

#[test]
fn test_scan_cached_keeps_invalid() {
    let receiver_secret = ReceiverSecret::generate().expect("receiver secret");
    let receiver_keys = ReceiverKeys::from_receiver_secret(receiver_secret).expect("keys");
    let card = receiver_card(&receiver_keys);
    let mut sender_wallet = SenderWallet::new([7u8; 32]);
    let asset_id = [8u8; 32];
    let output = build_tx_output_unchecked(
        &card,
        None,
        &mut sender_wallet,
        &[9u8; 32],
        0,
        11,
        &asset_id,
    )
    .expect("output");

    let r_pub = decode_r_pub(&output.r_pub).expect("decode r_pub");
    let dh = compute_dh_receiver(receiver_keys.reveal_view_sk(), &r_pub).expect("dh");
    let k_dh = derive_k_dh(&dh);
    let mut enc_pack = output.enc_pack.clone();
    enc_pack.tag[0] ^= 1;

    let input = ScanInput {
        serial_id: 0,
        leaf_ad_id: &asset_id,
        r_pub: &output.r_pub,
        owner_tag: &output.owner_tag,
        c_amount: &output.c_amount,
        enc_pack: &enc_pack,
        tag16: output.tag16,
    };

    let cached = scan_cached_keys(&receiver_keys.owner_handle, &input, [(k_dh, None)]);
    assert!(cached.owner_hit);
    assert!(matches!(
        cached.state,
        DetectState::Invalid(DetectFail::Decrypt)
    ));
}

#[test]
fn test_ordered_request_fallback_last() {
    let dh = [9u8; 32];
    let req_a = [0x20; 32];
    let req_b = [0x10; 32];

    let candidates = ordered_request_candidates(&dh, [req_a, req_b]);

    assert_eq!(
        candidates,
        vec![
            (derive_k_dh_with_req(&dh, &req_a), Some(req_a)),
            (derive_k_dh_with_req(&dh, &req_b), Some(req_b)),
            (derive_k_dh(&dh), None),
        ]
    );
}

#[test]
fn test_scan_cached_first_win() {
    struct PanicAfterFirst {
        first: Option<([u8; 32], Option<[u8; 32]>)>,
    }

    impl Iterator for PanicAfterFirst {
        type Item = ([u8; 32], Option<[u8; 32]>);

        fn next(&mut self) -> Option<Self::Item> {
            if let Some(first) = self.first.take() {
                return Some(first);
            }

            panic!("scan_cached_keys read past the first winning candidate");
        }
    }

    let receiver_secret = ReceiverSecret::generate().expect("receiver secret");
    let receiver_keys = ReceiverKeys::from_receiver_secret(receiver_secret).expect("keys");
    let card = receiver_card(&receiver_keys);
    let mut sender_wallet = SenderWallet::new([0x31; 32]);
    let asset_id = [0x32; 32];
    let output = build_tx_output_unchecked(
        &card,
        None,
        &mut sender_wallet,
        &[0x33; 32],
        0,
        21,
        &asset_id,
    )
    .expect("output");

    let r_pub = decode_r_pub(&output.r_pub).expect("decode r_pub");
    let dh = compute_dh_receiver(receiver_keys.reveal_view_sk(), &r_pub).expect("dh");
    let k_dh = derive_k_dh(&dh);

    let input = ScanInput {
        serial_id: 0,
        leaf_ad_id: &asset_id,
        r_pub: &output.r_pub,
        owner_tag: &output.owner_tag,
        c_amount: &output.c_amount,
        enc_pack: &output.enc_pack,
        tag16: output.tag16,
    };

    let cached = scan_cached_keys(
        &receiver_keys.owner_handle,
        &input,
        PanicAfterFirst {
            first: Some((k_dh, None)),
        },
    );

    assert!(cached.owner_hit);
    assert!(matches!(cached.state, DetectState::Mine(_)));
}

#[test]
fn test_scan_owned_bound_output() {
    let receiver_secret = ReceiverSecret::generate().expect("receiver secret");
    let receiver_keys = ReceiverKeys::from_receiver_secret(receiver_secret).expect("keys");
    let card = receiver_card(&receiver_keys);
    let mut sender_wallet = SenderWallet::new([10u8; 32]);
    let request = PaymentRequest::generate(
        &receiver_keys,
        RequestParams {
            expiry_seconds: 3_600,
            ..Default::default()
        },
        7,
    )
    .expect("request");
    let asset_id = [11u8; 32];
    let output = build_tx_output_unchecked(
        &card,
        Some(&request),
        &mut sender_wallet,
        &[12u8; 32],
        0,
        11,
        &asset_id,
    )
    .expect("output");

    let input = ScanInput {
        serial_id: 0,
        leaf_ad_id: &asset_id,
        r_pub: &output.r_pub,
        owner_tag: &output.owner_tag,
        c_amount: &output.c_amount,
        enc_pack: &output.enc_pack,
        tag16: output.tag16,
    };

    let view_sks = receiver_keys.all_view_sks();
    let state = scan_owned(
        view_sks.iter(),
        &receiver_keys.owner_handle,
        &input,
        [request.req_id],
    );

    assert!(matches!(state, super::DetectState::Mine(_)));
}

fn dummy_input(c_amount: [u8; 32]) -> ScanInput<'static> {
    static ASSET_ID: [u8; 32] = [1u8; 32];
    static R_PUB: [u8; 32] = [2u8; 32];
    static OWNER_TAG: [u8; 32] = [4u8; 32];
    static ENC_PACK: std::sync::LazyLock<ZkPackEncrypted> =
        std::sync::LazyLock::new(|| ZkPackEncrypted {
            version: ZKPACK_VER,
            ciphertext: vec![0u8; ZKPACK_CT_LEN],
            tag: [0u8; ZKPACK_TAG_LEN],
        });

    ScanInput {
        serial_id: 0,
        leaf_ad_id: &ASSET_ID,
        r_pub: &R_PUB,
        owner_tag: &OWNER_TAG,
        c_amount: Box::leak(Box::new(c_amount)),
        enc_pack: &ENC_PACK,
        tag16: None,
    }
}

fn valid_commitment_bytes() -> [u8; 32] {
    let mut bytes = [0u8; 32];
    bytes[0] = 1;
    let blinding = Z00ZScalar::try_from_bytes(bytes).expect("valid scalar");
    let commitment = create_commitment(7, &blinding).expect("commitment");
    let mut out = [0u8; 32];
    out.copy_from_slice(commitment.as_bytes());
    out
}

fn receiver_card(keys: &ReceiverKeys) -> ReceiverCard {
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
