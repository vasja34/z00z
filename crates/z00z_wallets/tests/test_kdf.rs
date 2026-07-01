use z00z_crypto::{hash::poseidon2_hash, Z00ZRistrettoPoint, Z00ZScalar};
use z00z_wallets::stealth::kdf::{
    compute_owner_tag, compute_tag16, derive_k_dh, derive_pack_key, derive_pack_nonce,
};

fn sample_dh_point() -> Z00ZRistrettoPoint {
    let scalar = Z00ZScalar::from_uniform_bytes(&[0x42u8; 64]).expect("uniform scalar bytes");
    Z00ZRistrettoPoint::from_secret_key(&scalar)
}

#[test]
fn test_kdf_k_dh_determinism() {
    let dh_point = sample_dh_point();
    let k1 = derive_k_dh(&dh_point.to_bytes());
    let k2 = derive_k_dh(&dh_point.to_bytes());

    assert_eq!(k1, k2, "derive_k_dh must be deterministic");
    assert_eq!(k1.len(), 32, "k_dh must be 32 bytes");
    assert_ne!(k1, [0u8; 32], "k_dh must not be zero");
}

#[test]
fn test_kdf_chain_distinct() {
    let dh_point = sample_dh_point();
    let asset_id = [0xAAu8; 32];
    let serial_id = 42u32;
    let leaf_ad = [0xBBu8; 32];
    let r_pub_b = [0xCCu8; 32];

    let k_dh = derive_k_dh(&dh_point.to_bytes());
    let pack_key = derive_pack_key(&k_dh, &asset_id, serial_id);
    let nonce = derive_pack_nonce(&leaf_ad, &r_pub_b, &asset_id, serial_id);

    assert_ne!(k_dh, pack_key, "k_dh and pack_key must differ");
    assert_ne!(k_dh, nonce, "k_dh and nonce must differ");
    assert_ne!(pack_key, nonce, "pack_key and nonce must differ");
    assert_eq!(
        derive_k_dh(&dh_point.to_bytes()),
        k_dh,
        "derive_k_dh must be deterministic"
    );
    assert_eq!(
        derive_pack_key(&k_dh, &asset_id, serial_id),
        pack_key,
        "deterministic"
    );
}

#[test]
fn test_kdf_domain_sep() {
    let ikm = [0x42u8; 32];
    let k_dh_key = poseidon2_hash(b"z00z.consensus.dh_key.v1", &[&ikm[..]]);
    let pack_k = poseidon2_hash(b"z00z.consensus.pack_key.v1", &[&ikm[..]]);
    let tag_k = poseidon2_hash(b"z00z.consensus.owner_tag.v1", &[&ikm[..]]);

    assert_ne!(k_dh_key, pack_k, "dh_key vs pack_key must differ");
    assert_ne!(k_dh_key, tag_k, "dh_key vs owner_tag must differ");
    assert_ne!(pack_k, tag_k, "pack_key vs owner_tag must differ");
    assert_ne!(k_dh_key, [0u8; 32], "must not output zero");
}

#[test]
fn test_kdf_owner_tag_tag16() {
    let owner_handle = poseidon2_hash(b"z00z.consensus.receiver_id.v1", &[&[0x11u8; 32][..]]);
    let k_dh = poseidon2_hash(b"z00z.consensus.dh_key.v1", &[&[0x42u8; 32][..]]);
    let leaf_ad = poseidon2_hash(b"z00z.consensus.leaf_ad.v1", &[&[0xBBu8; 32][..]]);

    let owner_tag = compute_owner_tag(&owner_handle, &k_dh);
    assert_ne!(owner_tag, [0u8; 32], "owner_tag must not be zero");

    let tag16 = compute_tag16(&k_dh, &leaf_ad);
    assert_eq!(
        compute_tag16(&[0x44u8; 32], &[0x55u8; 32]),
        0x5093,
        "tag16 golden"
    );
    assert_ne!(tag16, compute_tag16(&leaf_ad, &k_dh), "order matters");
}
