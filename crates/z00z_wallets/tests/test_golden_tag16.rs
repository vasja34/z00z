use z00z_wallets::stealth::kdf::compute_tag16;

#[test]
fn test_golden_tag16() {
    let k_dh = [0x44u8; 32];
    let leaf_ad = [0x55u8; 32];
    let tag_a = compute_tag16(&k_dh, &leaf_ad);
    let tag_b = compute_tag16(&k_dh, &leaf_ad);

    assert_eq!(tag_a, 0x5093, "tag16 golden vector mismatch");
    assert_eq!(tag_a, tag_b, "tag16 must be deterministic");
}
