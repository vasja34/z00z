#[path = "test_inc/test_range_proof_env.inc"]
mod test_common;

use test_common::RangeProofEnvGuard;
use z00z_core::assets::AssetPackPlain;
use z00z_crypto::{
    domains::AssetIdDomain, hash_zk::hash_zk, verify_range_proof, AGGREGATION_FACTOR,
    MIN_VALUE_PROMISE, RANGE_PROOF_BITS,
};
use z00z_storage::settlement::TerminalLeaf;
use z00z_wallets::{
    key::{ReceiverKeys, ReceiverSecret},
    receiver::ReceiverCard,
    stealth::ecdh::{compute_dh_receiver, decode_r_pub},
    stealth::kdf::{compute_leaf_ad, compute_tag16, derive_k_dh, derive_s_out},
    stealth::zkpack::ZkPack,
    stealth::{build_stealth_leaf, build_tx_output_unchecked, SenderWallet, TxStealthOutput},
};

fn make_keys() -> ReceiverKeys {
    let sec = ReceiverSecret::generate().expect("secret");
    ReceiverKeys::from_receiver_secret(sec).expect("keys")
}

fn make_card(keys: &ReceiverKeys) -> ReceiverCard {
    let card = keys.export_receiver_card().expect("card");
    card.verify().expect("verify");
    card
}

fn make_light(card: &ReceiverCard, tx: &[u8; 32], aid: &[u8; 32], amount: u64) -> TxStealthOutput {
    let mut sender = SenderWallet::new([0x21u8; 32]);
    build_tx_output_unchecked(card, None, &mut sender, tx, 2, amount, aid).expect("light")
}

fn light_kdh(keys: &ReceiverKeys, out: &TxStealthOutput) -> [u8; 32] {
    let r_pub = decode_r_pub(&out.r_pub).expect("r_pub");
    let dh = compute_dh_receiver(keys.reveal_view_sk(), &r_pub).expect("dh");
    derive_k_dh(&dh)
}

fn open_light(out: &TxStealthOutput, k_dh: &[u8; 32], aid: &[u8; 32]) -> AssetPackPlain {
    let leaf_ad = compute_leaf_ad(aid, 0, &out.r_pub, &out.owner_tag, &out.c_amount);
    let bytes = ZkPack::decrypt(k_dh, &leaf_ad, &out.r_pub, aid, 0, &out.enc_pack).expect("open");
    AssetPackPlain::decode_checked(&bytes).expect("plain")
}

fn make_full(
    card: &ReceiverCard,
    k_dh: &[u8; 32],
    light: &TxStealthOutput,
    amount: u64,
) -> (TerminalLeaf, [u8; 32]) {
    let _guard = RangeProofEnvGuard::new();
    let s_out = derive_s_out(k_dh, &light.r_pub, 17);
    let leaf = build_stealth_leaf(k_dh, &light.r_pub, &card.owner_handle, amount, 17, s_out)
        .expect("full");

    (leaf, s_out)
}

fn open_leaf(leaf: &TerminalLeaf, k_dh: &[u8; 32]) -> AssetPackPlain {
    let leaf_ad = compute_leaf_ad(
        &leaf.asset_id,
        leaf.serial_id,
        &leaf.r_pub,
        &leaf.owner_tag,
        &leaf.c_amount,
    );
    let bytes = ZkPack::decrypt(
        k_dh,
        &leaf_ad,
        &leaf.r_pub,
        &leaf.asset_id,
        leaf.serial_id,
        &leaf.enc_pack,
    )
    .expect("open leaf");
    AssetPackPlain::decode_checked(&bytes).expect("leaf plain")
}

fn fake_leaf(light: &TxStealthOutput, aid: [u8; 32], serial: u32) -> TerminalLeaf {
    TerminalLeaf {
        asset_id: aid,
        serial_id: serial,
        r_pub: light.r_pub,
        owner_tag: light.owner_tag,
        c_amount: light.c_amount,
        enc_pack: light.enc_pack.clone(),
        range_proof: Vec::new(),
        tag16: light.tag16.expect("tag16"),
    }
}

fn check_full_sem(
    light: &TxStealthOutput,
    full: &TerminalLeaf,
    light_pack: &AssetPackPlain,
    full_pack: &AssetPackPlain,
    full_s_out: &[u8; 32],
    aid: &[u8; 32],
    k_dh: &[u8; 32],
) {
    let full_id = hash_zk::<AssetIdDomain>("", &[full_s_out]);
    let light_ad = compute_leaf_ad(aid, 0, &light.r_pub, &light.owner_tag, &light.c_amount);
    let full_ad = compute_leaf_ad(
        &full.asset_id,
        full.serial_id,
        &full.r_pub,
        &full.owner_tag,
        &full.c_amount,
    );

    assert_eq!(full.asset_id, full_id);
    assert_ne!(full.asset_id, *aid);
    assert_eq!(full.r_pub, light.r_pub);
    assert_eq!(full.owner_tag, light.owner_tag);
    assert_eq!(light_pack.value, full_pack.value);
    assert_eq!(full_pack.s_out, *full_s_out);
    assert_ne!(full_pack.s_out, light_pack.s_out);
    assert_eq!(light.tag16, Some(compute_tag16(k_dh, &light_ad)));
    assert_eq!(full.tag16, compute_tag16(k_dh, &full_ad));
    assert!(!full.range_proof.is_empty());
}

fn check_full_proof(full: &TerminalLeaf) {
    let commit = z00z_crypto::Commitment::from_bytes(&full.c_amount)
        .expect("commitment")
        .0;
    verify_range_proof(
        &full.range_proof,
        &commit,
        RANGE_PROOF_BITS,
        AGGREGATION_FACTOR,
        MIN_VALUE_PROMISE,
    )
    .expect("range proof");
}

fn check_fake_leaf(fake: &TerminalLeaf, k_dh: &[u8; 32]) {
    let leaf_ad = compute_leaf_ad(
        &fake.asset_id,
        fake.serial_id,
        &fake.r_pub,
        &fake.owner_tag,
        &fake.c_amount,
    );
    let commit = z00z_crypto::Commitment::from_bytes(&fake.c_amount)
        .expect("commitment")
        .0;

    assert!(ZkPack::decrypt(
        k_dh,
        &leaf_ad,
        &fake.r_pub,
        &fake.asset_id,
        fake.serial_id,
        &fake.enc_pack,
    )
    .is_none());
    assert!(verify_range_proof(
        &fake.range_proof,
        &commit,
        RANGE_PROOF_BITS,
        AGGREGATION_FACTOR,
        MIN_VALUE_PROMISE,
    )
    .is_err());
}

#[test]
fn test_s5_leaf_match() {
    let amount = 777u64;
    let aid = [0x31u8; 32];
    let tx = [0x41u8; 32];
    let keys = make_keys();
    let card = make_card(&keys);
    let light = make_light(&card, &tx, &aid, amount);
    let k_dh = light_kdh(&keys, &light);
    let light_pack = open_light(&light, &k_dh, &aid);
    let (full, full_s_out) = make_full(&card, &k_dh, &light, amount);
    let full_pack = open_leaf(&full, &k_dh);

    check_full_sem(
        &light,
        &full,
        &light_pack,
        &full_pack,
        &full_s_out,
        &aid,
        &k_dh,
    );
    check_full_proof(&full);
}

#[test]
fn test_s5_fake_leaf() {
    let amount = 777u64;
    let aid = [0x32u8; 32];
    let tx = [0x42u8; 32];
    let keys = make_keys();
    let card = make_card(&keys);
    let light = make_light(&card, &tx, &aid, amount);
    let k_dh = light_kdh(&keys, &light);
    let full_s_out = derive_s_out(&k_dh, &light.r_pub, 17);
    let full_id = hash_zk::<AssetIdDomain>("", &[&full_s_out]);
    let fake = fake_leaf(&light, full_id, 17);

    check_fake_leaf(&fake, &k_dh);
}
