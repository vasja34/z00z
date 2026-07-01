use z00z_crypto::{
    domains::{AssetIdDomain, LeafAdDomain, PackKeyDomain, PackNonceDomain},
    hash_zk::hash_zk,
};

fn to_hex(bytes: &[u8; 32]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

#[test]
fn test_hash_domain_snap() {
    let input = [0x01u8; 32];
    let serial = 7u32.to_le_bytes();

    let asset = hash_zk::<AssetIdDomain>("", &[&input]);
    let leaf = hash_zk::<LeafAdDomain>("", &[&input, &serial, &input, &input, &input]);
    let pkey = hash_zk::<PackKeyDomain>("", &[&input, &input, &serial]);
    let pnon = hash_zk::<PackNonceDomain>("", &[&input, &input, &input, &serial]);

    assert_eq!(
        to_hex(&asset),
        "74993eed5312e04fa5b5d9250d6eaa15570b4c6a6b67099cf3d9adcbe363fd4b"
    );
    assert_eq!(
        to_hex(&leaf),
        "05ef89df5319cb17d50875bef5112a94cd2ae8b7ea401659456e5919da9fc2d3"
    );
    assert_eq!(
        to_hex(&pkey),
        "ed37938a903ddb2336416ef2e576ad241e433b026df71f093295b807ed521a02"
    );
    assert_eq!(
        to_hex(&pnon),
        "5051338d918aa6db1f6f211af614bf9ad692be3d5181d5b5d9893691aac0d100"
    );
}
