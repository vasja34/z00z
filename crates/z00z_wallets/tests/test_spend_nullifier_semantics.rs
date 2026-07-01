use z00z_crypto::{
    create_commitment, domains::AssetIdDomain, hash_zk::hash_zk, Z00ZRistrettoPoint,
};
use z00z_wallets::{
    claim::derive_nullifier as derive_claim_nullifier,
    key::{derive_owner_handle, derive_view_secret_key, ReceiverSecret},
    stealth::ecdh::compute_dh_sender,
    stealth::kdf::{compute_owner_tag, derive_k_dh},
    tx::{derive_spend_nullifier, verify_spend_rules, SpendIn, SpendRuleErr, SpendStmt},
};

fn test_scalar(seed: u64) -> z00z_crypto::Z00ZScalar {
    let mut bytes = [0u8; 32];
    bytes[..8].copy_from_slice(&seed.to_le_bytes());
    z00z_crypto::Z00ZScalar::try_from_bytes(bytes).expect("valid scalar")
}

fn test_secret(seed: u64) -> ReceiverSecret {
    let mut bytes = [0u8; 32];
    bytes[..8].copy_from_slice(&seed.to_le_bytes());
    if bytes == [0u8; 32] {
        bytes[0] = 1;
    }
    ReceiverSecret::from_bytes(bytes).expect("secret")
}

fn make_stmt() -> SpendStmt {
    let receiver_secret = test_secret(9);
    let view_sk = derive_view_secret_key(&receiver_secret).expect("view");
    let view_pk = Z00ZRistrettoPoint::from_secret_key(&view_sk);

    let r = test_scalar(77);
    let r_pub_pt = Z00ZRistrettoPoint::from_secret_key(&r);
    let r_pub_in = r_pub_pt.to_bytes();

    let dh = compute_dh_sender(&r, &view_pk).expect("dh");
    let k_in = derive_k_dh(&dh);
    let owner_handle = derive_owner_handle(&receiver_secret);
    let owner_tag_in = compute_owner_tag(&owner_handle, &k_in);

    let s_in = [5u8; 32];
    let leaf_ad_id_in = hash_zk::<AssetIdDomain>("", &[&s_in]);

    let in_blind = test_scalar(41);
    let c_in = create_commitment(12, &in_blind).expect("c_in");
    let c_out = create_commitment(12, &in_blind).expect("c_out");

    SpendStmt {
        receiver_secret,
        spend_ins: vec![SpendIn {
            chain_id: 3,
            r_pub_in,
            owner_tag_in,
            leaf_ad_id_in,
            nullifier_in: Some(derive_spend_nullifier(3, &s_in)),
            s_in,
            c_in,
        }],
        c_outs: vec![c_out],
        range_ok: true,
    }
}

#[test]
fn test_spend_nullifier_in_scope() {
    let secret = [0x55u8; 32];

    let left = derive_spend_nullifier(3, &secret);
    let right = derive_spend_nullifier(3, &secret);

    assert_eq!(left, right);
}

#[test]
fn test_spend_nullifier_chain_scope() {
    let secret = [0x55u8; 32];

    let left = derive_spend_nullifier(3, &secret);
    let right = derive_spend_nullifier(4, &secret);

    assert_ne!(left, right);
}

#[test]
fn test_spend_nullifier_claim_nullifier() {
    let receiver_secret = test_secret(9);
    let owner = derive_owner_handle(&receiver_secret);
    let spend_secret = [0x55u8; 32];
    let claim_id = [0x22u8; 32];

    let spend_nullifier = derive_spend_nullifier(3, &spend_secret);
    let claim_nullifier = derive_claim_nullifier(&claim_id, &owner, 3);

    assert_ne!(hex::encode(spend_nullifier), claim_nullifier.to_hex());
}

#[test]
fn test_spend_rules_one_contract() {
    let mut stmt = make_stmt();
    let duplicate = stmt.spend_ins[0].clone();
    stmt.spend_ins.push(duplicate);
    stmt.c_outs.push(stmt.c_outs[0].clone());

    let err = verify_spend_rules(&stmt).expect_err("duplicate spend nullifier must reject");

    assert_eq!(err, SpendRuleErr::DuplicateNullifier);
}

#[test]
fn test_spend_rules_same_input() {
    let mut stmt = make_stmt();
    let mut wrong = stmt.spend_ins[0].nullifier_in.expect("nullifier");
    wrong[0] ^= 1;
    stmt.spend_ins[0].nullifier_in = Some(wrong);

    let err = verify_spend_rules(&stmt).expect_err("nullifier drift must reject");

    assert_eq!(err, SpendRuleErr::BadNullifier { index: 0 });
}
