use std::sync::Arc;

use rand::rngs::OsRng;
use z00z_core::assets::{Asset, AssetClass, AssetDefinition, BlindingFactor};
use z00z_crypto::ZkPackEncrypted;

fn mk_definition() -> Arc<AssetDefinition> {
    let definition = AssetDefinition::new(
        [0u8; 32],
        AssetClass::Coin,
        "Domain Test".into(),
        "DST".into(),
        8,
        1024,
        1_000_000,
        "sig.test".into(),
        1,
        1,
        0,
        None,
    )
    .expect("valid definition");
    Arc::new(definition)
}

fn mk_signed_asset() -> Asset {
    let mut rng = OsRng;
    let blinding = BlindingFactor::random(&mut rng);
    Asset::new(mk_definition(), 7, 500, &blinding, [42u8; 32], &mut rng).expect("signed asset")
}

fn mk_alt_asset() -> Asset {
    let mut rng = OsRng;
    let blinding = BlindingFactor::random(&mut rng);
    Asset::new(mk_definition(), 8, 500, &blinding, [43u8; 32], &mut rng).expect("alt asset")
}

fn mk_signed_stealth_asset() -> Asset {
    let mut rng = OsRng;
    let blinding = BlindingFactor::random(&mut rng);
    let mut asset = Asset::new(mk_definition(), 7, 500, &blinding, [44u8; 32], &mut rng)
        .expect("signed stealth asset");

    asset.r_pub = Some([0x11; 32]);
    asset.owner_tag = Some([0x22; 32]);
    asset.enc_pack = Some(ZkPackEncrypted {
        version: 1,
        ciphertext: vec![0x33; 72],
        tag: [0x44; 16],
    });
    asset.tag16 = Some(0x1234);
    asset.leaf_ad_id = Some([0x55; 32]);

    let signature = asset
        .sign_owner(&blinding, &mut rng)
        .expect("stealth signature");
    asset.owner_signature = Some(signature);
    asset
}

#[test]
fn test_sig_tamper_amount() {
    let signed = mk_signed_asset();
    assert!(signed.verify_owner_signature().is_ok());

    let mut tampered = signed.clone();
    tampered.amount = tampered.amount.saturating_add(1);

    let result = tampered.verify_owner_signature();
    assert!(result.is_err());
}

#[test]
fn test_sig_tamper_nonce() {
    let signed = mk_signed_asset();
    assert!(signed.verify_owner_signature().is_ok());

    let mut tampered = signed.clone();
    tampered.nonce[0] = tampered.nonce[0].wrapping_add(1);

    let result = tampered.verify_owner_signature();
    assert!(result.is_err());
}

#[test]
fn test_sig_tamper_commitment() {
    let signed = mk_signed_asset();
    assert!(signed.verify_owner_signature().is_ok());

    let mut tampered = signed.clone();
    let alt = mk_alt_asset();
    tampered.commitment = alt.commitment;

    let result = tampered.verify_owner_signature();
    assert!(result.is_err());
}

#[test]
fn test_sig_tamper_range_proof() {
    let signed = mk_signed_asset();
    assert!(signed.verify_owner_signature().is_ok());

    let mut tampered = signed.clone();
    let mut range_proof = tampered
        .range_proof
        .clone()
        .expect("range proof must exist for signed asset");
    if range_proof.is_empty() {
        range_proof.push(1);
    } else {
        range_proof[0] = range_proof[0].wrapping_add(1);
    }
    tampered.range_proof = Some(range_proof);

    let result = tampered.verify_owner_signature();
    assert!(result.is_err());
}

#[test]
fn test_sig_tamper_lock_height() {
    let signed = mk_signed_asset();
    assert!(signed.verify_owner_signature().is_ok());

    let mut tampered = signed.clone();
    tampered.lock_height = Some(123);

    let result = tampered.verify_owner_signature();
    assert!(result.is_err());
}

#[test]
fn test_sig_tamper_flags() {
    let signed = mk_signed_asset();
    assert!(signed.verify_owner_signature().is_ok());

    let mut tampered = signed.clone();
    tampered.is_frozen = !tampered.is_frozen;

    let result = tampered.verify_owner_signature();
    assert!(result.is_err());
}

#[test]
fn test_sig_tamper_owner_pub() {
    let signed = mk_signed_asset();
    assert!(signed.verify_owner_signature().is_ok());

    let mut tampered = signed.clone();
    tampered.owner_pub = mk_alt_asset().owner_pub;

    let result = tampered.verify_owner_signature();
    assert!(result.is_err());
}

#[test]
fn test_rejects_added_stealth_tuple() {
    let signed = mk_signed_asset();
    assert!(signed.verify_owner_signature().is_ok());

    let mut tampered = signed.clone();
    tampered.r_pub = Some([0x11; 32]);
    tampered.owner_tag = Some([0x22; 32]);
    tampered.enc_pack = Some(ZkPackEncrypted {
        version: 1,
        ciphertext: vec![0x33; 72],
        tag: [0x44; 16],
    });
    tampered.tag16 = Some(0x1234);
    tampered.leaf_ad_id = Some([0x55; 32]);

    assert!(tampered.verify_owner_signature().is_err());
}

#[test]
fn test_sig_tamper_stealth_fields() {
    let signed = mk_signed_stealth_asset();
    assert!(signed.verify_owner_signature().is_ok());

    let mut tampered = signed.clone();
    tampered.r_pub = Some([0x66; 32]);
    assert!(tampered.verify_owner_signature().is_err());

    let mut tampered = signed.clone();
    tampered.owner_tag = Some([0x77; 32]);
    assert!(tampered.verify_owner_signature().is_err());

    let mut tampered = signed.clone();
    tampered.enc_pack = Some(ZkPackEncrypted {
        version: 1,
        ciphertext: vec![0x88; 72],
        tag: [0x44; 16],
    });
    assert!(tampered.verify_owner_signature().is_err());

    let mut tampered = signed.clone();
    tampered.tag16 = Some(0x4321);
    assert!(tampered.verify_owner_signature().is_err());

    let mut tampered = signed.clone();
    tampered.leaf_ad_id = Some([0x99; 32]);
    assert!(tampered.verify_owner_signature().is_err());
}
