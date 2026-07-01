use z00z_wallets::OwnerTag;

#[test]
fn test_golden_owner_tag() {
    let owner = [0x22u8; 32];
    let k_dh = [0x33u8; 32];
    let exp = "221b9ff0335fe5fc92c8c740d4fe8fe4b2afa1fb87260c75ea822bc8b2f5f594";
    let exp_bytes: [u8; 32] = hex::decode(exp)
        .expect("golden hex")
        .try_into()
        .expect("golden len");

    let tag_a = OwnerTag::compute(&owner, &k_dh);
    let tag_b = OwnerTag::compute(&owner, &k_dh);

    assert_eq!(
        tag_a.as_bytes(),
        &exp_bytes,
        "owner_tag golden vector mismatch"
    );
    assert_eq!(
        tag_a.as_bytes(),
        tag_b.as_bytes(),
        "owner_tag must be deterministic"
    );
}
