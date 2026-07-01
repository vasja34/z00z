//! E2E Phase 3 section 5 taxonomy.
//! Taxonomy map:
//! - `tag16` -> self: invalid stealth input; scan: invalid proof.
//! - `enc_pack` -> self: invalid stealth input; scan: invalid proof.
//! - `c_amount` -> self: invalid stealth input; scan: invalid proof.
//! - `range_proof` -> asset: range proof failure before wallet ownership.
//! - `value_bind` -> self: invalid stealth input; open: commitment mismatch.
//! - `req_bind` -> wrong request must never return `Mine`.

use z00z_core::{
    assets::{AssetError, AssetLeaf, AssetPackPlain},
    genesis::asset_std::asset_from_dev_class,
    Asset, AssetClass,
};
use z00z_crypto::{create_commitment, create_range_proof, Z00ZScalar};
use z00z_wallets::{
    build_tx_output_unchecked,
    key::{ReceiverKeys, ReceiverSecret},
    receiver::{receiver_scan_leaf, receiver_scan_report, ScanResult},
    receiver::{
        PaymentRequest, ReceiveReject, ReceiveStatus, ReceiverCard, RequestMetadata,
        StealthOutputScanner, ValidityStatus,
    },
    stealth::ecdh::{compute_dh_receiver, decode_r_pub},
    stealth::kdf::{compute_leaf_ad, derive_k_dh, derive_k_dh_with_req},
    stealth::zkpack::ZkPack,
    validate_output_self, SenderValidationCtx, SenderWallet, StealthError, TagMode,
    TxStealthOutput,
};

struct CardCase {
    keys: ReceiverKeys,
    out: TxStealthOutput,
    ctx: SenderValidationCtx,
    leaf: AssetLeaf,
    asset: Asset,
    amount: u64,
    blind: [u8; 32],
}

struct ReqCase {
    keys: ReceiverKeys,
    req: PaymentRequest,
    bad: PaymentRequest,
    asset: Asset,
}

fn mk_keys(seed: u8) -> ReceiverKeys {
    let mut buf = [seed; 32];
    buf[31] ^= 0x3C;
    let sec = ReceiverSecret::from_bytes(buf).expect("secret");
    ReceiverKeys::from_receiver_secret(sec).expect("keys")
}

fn mk_card(keys: &ReceiverKeys) -> ReceiverCard {
    let card = keys.export_receiver_card().expect("card");
    card.verify().expect("verify");
    card
}

fn mk_req(keys: &ReceiverKeys, tail: u8) -> PaymentRequest {
    let mut req_id = [0x60; 32];
    req_id[31] = tail;
    let mut req = PaymentRequest {
        version: 1,
        owner_handle: keys.owner_handle,
        view_pk: keys.view_pk.as_bytes().try_into().expect("view_pk"),
        identity_pk: keys.identity_pk.as_bytes().try_into().expect("id_pk"),
        req_id,
        chain_id: 77,
        amount: Some(880),
        expiry: u64::MAX,
        metadata: Some(RequestMetadata {
            memo: Some("phase3 tax".to_string()),
            payment_id: Some([tail; 16]),
            min_confirmations: None,
            return_receiver: None,
            created_at: 1,
        }),
        signature: [0u8; 64],
    };
    req.sign(keys.reveal_identity_sk()).expect("sign");
    assert!(matches!(req.check_validity(), ValidityStatus::Valid(_)));
    req.verify().expect("verify");
    req
}

fn ctx_card(
    keys: &ReceiverKeys,
    out: &TxStealthOutput,
    aid: [u8; 32],
    sid: u32,
) -> SenderValidationCtx {
    let r_pub = decode_r_pub(&out.r_pub).expect("r_pub");
    let dh = compute_dh_receiver(keys.reveal_view_sk(), &r_pub).expect("dh");
    SenderValidationCtx {
        k_dh: derive_k_dh(&dh),
        owner_handle: keys.owner_handle,
        asset_id: aid,
        serial_id: sid,
        tag_mode: TagMode::CardBound,
    }
}

fn ctx_req(
    keys: &ReceiverKeys,
    out: &TxStealthOutput,
    aid: [u8; 32],
    sid: u32,
    req: &PaymentRequest,
) -> SenderValidationCtx {
    let r_pub = decode_r_pub(&out.r_pub).expect("r_pub");
    let dh = compute_dh_receiver(keys.reveal_view_sk(), &r_pub).expect("dh");
    SenderValidationCtx {
        k_dh: derive_k_dh_with_req(&dh, &req.req_id),
        owner_handle: keys.owner_handle,
        asset_id: aid,
        serial_id: sid,
        tag_mode: TagMode::RequestBound { req_id: req.req_id },
    }
}

fn open_pack(out: &TxStealthOutput, ctx: &SenderValidationCtx) -> AssetPackPlain {
    let leaf_ad = compute_leaf_ad(
        &ctx.asset_id,
        ctx.serial_id,
        &out.r_pub,
        &out.owner_tag,
        &out.c_amount,
    );
    let plain = ZkPack::decrypt(
        &ctx.k_dh,
        &leaf_ad,
        &out.r_pub,
        &ctx.asset_id,
        ctx.serial_id,
        &out.enc_pack,
    )
    .expect("decrypt");
    AssetPackPlain::decode_checked(&plain).expect("pack")
}

fn build_asset_leaf(out: &TxStealthOutput, aid: [u8; 32], sid: u32, proof: Vec<u8>) -> AssetLeaf {
    AssetLeaf {
        asset_id: aid,
        serial_id: sid,
        r_pub: out.r_pub,
        owner_tag: out.owner_tag,
        c_amount: out.c_amount,
        enc_pack: out.enc_pack.clone(),
        range_proof: proof,
        tag16: out.tag16.expect("tag16"),
    }
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

fn alt_com(base: &CardCase) -> [u8; 32] {
    let pack = receiver_scan_leaf(&base.keys, &base.leaf)
        .expect("scan")
        .expect("pack");

    for bit in 0u8..=u8::MAX {
        let mut bytes = pack.blinding;
        bytes[31] ^= bit;
        if bytes == pack.blinding {
            continue;
        }

        let Ok(blind) = Z00ZScalar::try_from_bytes(bytes) else {
            continue;
        };

        let out: [u8; 32] = create_commitment(pack.value, &blind)
            .expect("commitment")
            .as_bytes()
            .try_into()
            .expect("bytes");
        if out != base.leaf.c_amount {
            return out;
        }
    }

    panic!("alt commitment")
}

fn scanner_for_keys(keys: &ReceiverKeys) -> StealthOutputScanner {
    StealthOutputScanner::from_keys(keys)
}

fn req_scan(keys: &ReceiverKeys, req: &PaymentRequest) -> StealthOutputScanner {
    let mut scan = StealthOutputScanner::from_keys(keys);
    scan.add_request(req);
    scan
}

fn mk_card_case() -> CardCase {
    let keys = mk_keys(0x21);
    let card = mk_card(&keys);
    let amount = 777u64;
    let aid = [0x71; 32];
    let sid = 0;
    let mut sender = SenderWallet::new([0x31; 32]);
    let out = build_tx_output_unchecked(&card, None, &mut sender, &[0x51; 32], sid, amount, &aid)
        .expect("output");
    let ctx = ctx_card(&keys, &out, aid, sid);
    let pack = open_pack(&out, &ctx);
    let blind = Z00ZScalar::try_from_bytes(pack.blinding).expect("blinding");
    let proof = create_range_proof(amount, &blind, 64, 0).expect("proof");
    let leaf = build_asset_leaf(&out, aid, sid, proof);
    let asset = build_asset_from_leaf(&leaf, amount);

    CardCase {
        keys,
        out,
        ctx,
        leaf,
        asset,
        amount,
        blind: pack.blinding,
    }
}

fn mk_req_case() -> ReqCase {
    let keys = mk_keys(0x41);
    let req = mk_req(&keys, 0xA1);
    let bad = mk_req(&keys, 0xB2);
    let card = mk_card(&keys);
    let aid = [0x81; 32];
    let amount = req.amount.expect("amount");
    let mut sender = SenderWallet::new([0x71; 32]);
    let out =
        build_tx_output_unchecked(&card, Some(&req), &mut sender, &[0x91; 32], 0, amount, &aid)
            .expect("output");
    let ctx = ctx_req(&keys, &out, aid, 0, &req);
    let pack = open_pack(&out, &ctx);
    let blind = Z00ZScalar::try_from_bytes(pack.blinding).expect("blinding");
    let proof = create_range_proof(amount, &blind, 64, 0).expect("proof");
    let leaf = AssetLeaf {
        asset_id: aid,
        serial_id: 0,
        r_pub: out.r_pub,
        owner_tag: out.owner_tag,
        c_amount: out.c_amount,
        enc_pack: out.enc_pack.clone(),
        range_proof: proof.clone(),
        tag16: out.tag16.expect("tag16"),
    };
    let asset = build_asset_from_leaf(&leaf, amount);

    let scan = req_scan(&keys, &req);
    assert!(matches!(scan.scan_leaf(&asset), ScanResult::Mine { .. }));
    validate_output_self(&out, &ctx, amount).expect("self");

    ReqCase {
        keys,
        req,
        bad,
        asset,
    }
}

fn self_bad(out: &TxStealthOutput, ctx: &SenderValidationCtx, amount: u64) {
    let err = validate_output_self(out, ctx, amount).expect_err("self fail");
    assert_eq!(err, StealthError::InvalidStealthInput);
}

fn rep_bad(base: &CardCase, leaf: &AssetLeaf, asset: &Asset) {
    assert!(receiver_scan_leaf(&base.keys, leaf)
        .expect("scan")
        .is_none());
    let rep = receiver_scan_report(&base.keys, leaf).expect("report");
    assert_eq!(rep.status, ReceiveStatus::InvalidProof);
    assert_eq!(rep.reject, Some(ReceiveReject::InvalidProof));
    let scan = scanner_for_keys(&base.keys).scan_leaf(asset);
    assert!(!matches!(scan, ScanResult::Mine { .. }));
    assert_eq!(scan.recv_report(), rep);
}

fn tag_bad(base: &CardCase) {
    let mut out = base.out.clone();
    out.tag16 = out.tag16.map(|tag| tag ^ 1);
    self_bad(&out, &base.ctx, base.amount);

    let mut leaf = base.leaf.clone();
    leaf.tag16 ^= 1;
    let asset = build_asset_from_leaf(&leaf, base.amount);
    rep_bad(base, &leaf, &asset);
}

fn pack_bad(base: &CardCase) {
    let mut out = base.out.clone();
    out.enc_pack.ciphertext[0] ^= 1;
    self_bad(&out, &base.ctx, base.amount);

    let mut leaf = base.leaf.clone();
    leaf.enc_pack.ciphertext[0] ^= 1;
    let asset = build_asset_from_leaf(&leaf, base.amount);
    rep_bad(base, &leaf, &asset);
}

fn com_bad(base: &CardCase) {
    let mut out = base.out.clone();
    out.c_amount = alt_com(base);
    self_bad(&out, &base.ctx, base.amount);

    let mut leaf = base.leaf.clone();
    leaf.c_amount = out.c_amount;
    let asset = build_asset_from_leaf(&leaf, base.amount);
    let blind = Z00ZScalar::try_from_bytes(base.blind).expect("blind");
    assert!(matches!(
        asset.verify_commitment_opening(&blind),
        Err(AssetError::CommitmentMismatch { .. })
    ));
    rep_bad(base, &leaf, &asset);
}

fn proof_bad(base: &CardCase) {
    let mut asset = base.asset.clone();
    asset.range_proof.as_mut().expect("proof")[0] ^= 1;
    assert!(matches!(
        asset.validate_amount(),
        Err(AssetError::RangeProofVerification { .. })
    ));
    assert!(matches!(
        scanner_for_keys(&base.keys).scan_leaf(&asset),
        ScanResult::Mine { .. }
    ));
}

fn value_bad(base: &CardCase) {
    let mut asset = base.asset.clone();
    asset.amount = asset.amount.saturating_add(1);
    self_bad(&base.out, &base.ctx, asset.amount);
    let blind = Z00ZScalar::try_from_bytes(base.blind).expect("blind");
    assert!(matches!(
        asset.verify_commitment_opening(&blind),
        Err(AssetError::CommitmentMismatch { .. })
    ));
    assert!(matches!(
        scanner_for_keys(&base.keys).scan_leaf(&asset),
        ScanResult::Mine { .. }
    ));
}

fn req_bad(base: &ReqCase) {
    let good = req_scan(&base.keys, &base.req).scan_leaf(&base.asset);
    assert!(matches!(good, ScanResult::Mine { .. }));

    let bad = req_scan(&base.keys, &base.bad).scan_leaf(&base.asset);
    assert!(!matches!(bad, ScanResult::Mine { .. }));
    assert_ne!(bad.recv_report().status, ReceiveStatus::Detected);
}

#[test]
fn test_e2e_self_ok() {
    let base = mk_card_case();
    validate_output_self(&base.out, &base.ctx, base.amount).expect("self");
    base.asset.validate_stealth_consistency().expect("shape");
    base.asset.validate_amount().expect("proof");
    base.asset.validate().expect("asset");

    let pack = receiver_scan_leaf(&base.keys, &base.leaf)
        .expect("scan")
        .expect("pack");
    let rep = receiver_scan_report(&base.keys, &base.leaf).expect("report");
    let scan = scanner_for_keys(&base.keys).scan_leaf(&base.asset);

    assert_eq!(pack.value, base.amount);
    assert_eq!(rep.status, ReceiveStatus::Detected);
    assert_eq!(scan.recv_report().status, ReceiveStatus::Detected);
}

#[test]
fn test_e2e_self_tax_map() {
    let base = mk_card_case();
    tag_bad(&base);
    pack_bad(&base);
    com_bad(&base);
    proof_bad(&base);
    value_bad(&base);
    req_bad(&mk_req_case());
}
