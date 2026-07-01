use z00z_crypto::{
    compute_leaf_ad, compute_tag16, encode_leaf_preimage, range_ctx_hash, LEAF_PREIMAGE_SIZE,
};

#[test]
fn test_tag16_golden_vector() {
    assert_eq!(compute_tag16(&[0x44u8; 32], &[0x55u8; 32]), 0x5093);
}

#[test]
fn test_leaf_ad_golden_vectors() {
    let asset_id = [0x11u8; 32];
    let r_pub = [0x22u8; 32];
    let owner_tag = [0x33u8; 32];
    let c_amount = [0x44u8; 32];

    assert_eq!(
        compute_leaf_ad(&asset_id, 0, &r_pub, &owner_tag, &c_amount),
        [
            0x39, 0x50, 0x9a, 0x8b, 0x76, 0x24, 0x8d, 0xd1, 0xaa, 0xf2, 0x43, 0x29, 0x42, 0x67,
            0x8e, 0x29, 0x75, 0xe1, 0x52, 0x37, 0x51, 0xeb, 0xbd, 0x61, 0xa4, 0x25, 0xba, 0xb5,
            0xbf, 0x6e, 0x1b, 0x8c,
        ]
    );
    assert_eq!(
        compute_leaf_ad(&asset_id, 42, &r_pub, &owner_tag, &c_amount),
        [
            0x57, 0x41, 0x1d, 0x24, 0xab, 0x69, 0xc4, 0x82, 0x99, 0x87, 0x40, 0x17, 0xe6, 0x51,
            0x63, 0x6a, 0xf2, 0xcd, 0xba, 0xc3, 0x15, 0x5d, 0x2b, 0x1d, 0x95, 0x04, 0x07, 0x2b,
            0x70, 0x47, 0x8e, 0x9d,
        ]
    );
}

#[test]
fn test_leaf_ad_preimage_layout() {
    let asset_id = [0xA1u8; 32];
    let serial_id = 0x1234_5678u32;
    let r_pub = [0xB2u8; 32];
    let owner_tag = [0xC3u8; 32];
    let c_amount = [0xD4u8; 32];
    let encoded = encode_leaf_preimage(&asset_id, serial_id, &r_pub, &owner_tag, &c_amount);

    assert_eq!(encoded.len(), LEAF_PREIMAGE_SIZE);
    assert_eq!(&encoded[0..32], &asset_id);
    assert_eq!(&encoded[32..36], &serial_id.to_le_bytes());
    assert_eq!(&encoded[36..68], &r_pub);
    assert_eq!(&encoded[68..100], &owner_tag);
    assert_eq!(&encoded[100..132], &c_amount);
}

#[test]
fn test_context_changes_bound_field() {
    let asset_id = [0x11u8; 32];
    let commitment = [0x22u8; 32];
    let proof = [0x33u8; 48];

    let base = range_ctx_hash(&asset_id, 7, 1, 2, 3, &commitment, &proof);
    assert_ne!(
        base,
        range_ctx_hash(&[0x12u8; 32], 7, 1, 2, 3, &commitment, &proof)
    );
    assert_ne!(
        base,
        range_ctx_hash(&asset_id, 8, 1, 2, 3, &commitment, &proof)
    );
    assert_ne!(
        base,
        range_ctx_hash(&asset_id, 7, 9, 2, 3, &commitment, &proof)
    );
    assert_ne!(
        base,
        range_ctx_hash(&asset_id, 7, 1, 9, 3, &commitment, &proof)
    );
    assert_ne!(
        base,
        range_ctx_hash(&asset_id, 7, 1, 2, 4, &commitment, &proof)
    );
    assert_ne!(
        base,
        range_ctx_hash(&asset_id, 7, 1, 2, 3, &[0x23u8; 32], &proof)
    );
    assert_ne!(
        base,
        range_ctx_hash(&asset_id, 7, 1, 2, 3, &commitment, &[0x34u8; 48])
    );
}
