use hex_literal::hex;
use tari_crypto::{
    keys::PublicKey,
    ristretto::{RistrettoPublicKey, RistrettoSecretKey},
    tari_utilities::ByteArray,
};

use z00z_crypto::{
    domains::{PackKeyDomain, PackNonceDomain},
    hash_zk::hash_zk,
    kdf_consensus::{
        compute_owner_tag, derive_asset_id, derive_leaf_ad, derive_owner_handle, derive_view_sk,
    },
    protocol::ecdh::{derive_dh_key, recover_stealth_dh_receiver},
    types::{Z00ZRistrettoPoint, Z00ZScalar},
};

fn seq_bytes(start: u8) -> [u8; 32] {
    let mut out = [0u8; 32];
    for (idx, item) in out.iter_mut().enumerate() {
        *item = start.wrapping_add(idx as u8);
    }
    out
}

fn sk_from_u8(value: u8) -> RistrettoSecretKey {
    let mut bytes = [0u8; 32];
    bytes[0] = value;
    RistrettoSecretKey::from_canonical_bytes(&bytes).expect("small scalar must be canonical")
}

fn dh_from_pair(a: u8, b: u8) -> [u8; 32] {
    let view_sk = Z00ZScalar::from_ristretto_secret_key(sk_from_u8(a));
    let r_sk = sk_from_u8(b);
    let r_pub =
        Z00ZRistrettoPoint::from_ristretto_public_key(RistrettoPublicKey::from_secret_key(&r_sk));
    let dh = recover_stealth_dh_receiver(&view_sk, &r_pub).expect("dh must derive");
    derive_dh_key(&dh)
}

fn rid_inputs() -> [[u8; 32]; 5] {
    [
        [0u8; 32],
        [0xFFu8; 32],
        [0x42u8; 32],
        seq_bytes(0x00),
        seq_bytes(0x80),
    ]
}

fn dh_inputs() -> [[u8; 32]; 5] {
    [
        dh_from_pair(1, 2),
        dh_from_pair(3, 5),
        dh_from_pair(7, 11),
        dh_from_pair(13, 17),
        dh_from_pair(19, 23),
    ]
}

fn asset_inputs() -> [[u8; 32]; 5] {
    [
        seq_bytes(0x10),
        seq_bytes(0x20),
        seq_bytes(0x30),
        seq_bytes(0x40),
        seq_bytes(0x50),
    ]
}

#[test]
fn test_rid_vectors() {
    let expected = [
        hex!("4583d0cc86b52a733514e0cdc5057df758b115c8d462ca2a671c333817a6fec0"),
        hex!("4d64bf09d86f309bc75a4129d37bef89d331b5e92d6925e5185eb4823cfe94cc"),
        hex!("0966199bbae1911543d593e3a846c0d8a7320344718167c1ac3e95fee8b462b6"),
        hex!("5b9717fed1190e951f67ee6523fd0dfb9ae6b2f0d90291bade6c488175684368"),
        hex!("aa79443e607cf96572493b9564a69d9c401f11557a9012424dd4c669a6a283e7"),
    ];

    for (idx, input) in rid_inputs().iter().enumerate() {
        let actual = derive_owner_handle(input);
        assert_eq!(actual, expected[idx], "RID vector {} mismatch", idx + 1);
    }
}

#[test]
fn test_view_vectors() {
    let expected = [
        hex!("8b84ed02824ea7bf988070e81c99dbc81d5b0f0f9f61730fcb77a35a43f0b305"),
        hex!("15c32de57518f7ab07a2b0c12983500fa6018a258deb0ef5fc9945d8b4a60c04"),
        hex!("f17acfcb768dc542c79cb46d7a303142dd66085eed847ecad9c3a4175c2b2c0f"),
        hex!("67be19aed2cb587032554b89635965427dff899755cf4f2b5a08aa981f4d6c05"),
        hex!("b267bb6b3847c16a323e775c1596851aae95b60d8561f6e52ac965d1cd64a90f"),
    ];

    for (idx, input) in rid_inputs().iter().enumerate() {
        let actual = derive_view_sk(input).expect("view key must derive");
        assert_eq!(
            actual.as_bytes(),
            &expected[idx],
            "VIEW vector {} mismatch",
            idx + 1
        );
    }
}

#[test]
fn test_dh_vectors() {
    let expected = [
        hex!("77d127efb5a6d3dd47561bad272eddf52ba535c7651954f821b2ea8aaf9b03f1"),
        hex!("ec47c3594aeeeb7a11c28dc2e30138c19bca4355455674bacc639c847cfb5823"),
        hex!("3f136035fd7a26a57527f3495bb8df7aaeffff97f74bb01730ea53d03a7ef6bf"),
        hex!("804b3fb28ef26b54fd130c189e59ae58646a4cd7f041299d8b73d0bf96480925"),
        hex!("e919c6e8e9d26d976de6f8b6154afbc5eff11a79ddc7a5a49d308fad50128d9a"),
    ];

    for (idx, actual) in dh_inputs().iter().enumerate() {
        assert_eq!(actual, &expected[idx], "DH vector {} mismatch", idx + 1);
    }
}

#[test]
fn test_tag_vectors() {
    let expected = [
        hex!("7e53e4d00135e043c310c7b8691027df7d7fed23da76bc830208c04fe6bfe9c1"),
        hex!("af6c3c391e35f9edd63bc64b55d49d1bdeb53c8f13f437927dee7ffa1320e024"),
        hex!("9053ff251e0b2e88fd5e9b94c9f41607da8ed03d6c6183b2bfbfa68724164c0d"),
        hex!("ead010e94b6aba1bb54ce59ae60c8eef69d8cf245304db8ce54da57676e32fbc"),
        hex!("59f23fde84c3c0f6d5f158e248199e36b3d0c47d66b8928c37319dd5f9dab87c"),
    ];

    let owners = rid_inputs();
    let dh_vals = dh_inputs();
    for idx in 0..5 {
        let owner = derive_owner_handle(&owners[idx]);
        let actual = compute_owner_tag(&owner, &dh_vals[idx]);
        assert_eq!(actual, expected[idx], "TAG vector {} mismatch", idx + 1);
    }
}

#[test]
fn test_asset_vectors() {
    let expected = [
        hex!("54e4e58213249a2a9dedf1647427cabb8136a3f6e758b501536b0fac6089f04d"),
        hex!("dc2f8a6195cfe67065515f59dbeae441457d651aa8b9c3394e275c81289039c6"),
        hex!("41d82c963886125e0115fee3c02509fcada2499fadc14588fbb900b75d905333"),
        hex!("d737b3fb7f59c6e04dc6c229962497016fe481e39a49d8d3bfcf15e81cb0c6e8"),
        hex!("4069c4685483aa3dde3cb0f50131c85ab6e90527cbe6dbe9d09b8656be59213a"),
    ];

    for (idx, input) in asset_inputs().iter().enumerate() {
        let actual = derive_asset_id(input);
        assert_eq!(actual, expected[idx], "ASSET vector {} mismatch", idx + 1);
    }
}

#[test]
fn test_packkey_vectors() {
    let expected = [
        hex!("407aa8cb34d854eb6f1ae60f6cfa3aa659a34ba29ab19b39d8032052c1390700"),
        hex!("6dda9801deef6f94d3879b28c61371f174aa8400d196c03b259f4aba8c8d81e8"),
        hex!("2f100a7aac6ce0a2082bf88e18f69f6d3baa06f6b3d99c0162d0f825fd618c52"),
        hex!("7bcfcfbf6fd7b215d682953ed47cdc40aa5a5bd471de65fbbd0fa1c7f92adfe7"),
        hex!("08ee219a07df59576f10e5293b3985295f6413ee9de11ed432593dd8d1e7e2bc"),
    ];

    let dh_vals = dh_inputs();
    let assets = asset_inputs();
    for idx in 0..5 {
        let serial = (idx as u32) + 1;
        let serial_le = serial.to_le_bytes();
        let asset = derive_asset_id(&assets[idx]);
        let actual = hash_zk::<PackKeyDomain>("", &[&dh_vals[idx], &asset, &serial_le]);
        assert_eq!(actual, expected[idx], "PACKKEY vector {} mismatch", idx + 1);
    }
}

#[test]
fn test_packnonce_vectors() {
    let expected = [
        hex!("eeccde17216d24c90cb4fe88367de5e2e341e2128b136cdf5693397cb6a460df"),
        hex!("9a3d7cebe5e6a9510fdecaed10572a6530603fc6ef4cb29f45d13135c8c20458"),
        hex!("b6b89b80f5b8227bf8348e5c06e5711735fcaf7ac3072cdea081ed5c20d9537c"),
        hex!("f2d2ad6b9c929e7981798d62438b1007300df643b2bd9d4085f2ee04b1261e57"),
        hex!("4af651a2a8f9825ee2058a48dc0b274fdd7bdce24b7abc5b05626556e0a6acb2"),
    ];

    let assets = asset_inputs();
    let owners = rid_inputs();
    let dh_vals = dh_inputs();
    for idx in 0..5 {
        let serial = (idx as u32) + 1;
        let serial_le = serial.to_le_bytes();
        let asset = derive_asset_id(&assets[idx]);
        let owner = derive_owner_handle(&owners[idx]);
        let owner_tag = compute_owner_tag(&owner, &dh_vals[idx]);

        let r_pub = seq_bytes(0x60u8.wrapping_add(idx as u8));
        let c_amount = seq_bytes(0xA0u8.wrapping_add(idx as u8));
        let leaf_ad = derive_leaf_ad(&asset, serial, &r_pub, &owner_tag, &c_amount);
        let actual = hash_zk::<PackNonceDomain>("", &[&leaf_ad, &r_pub, &asset, &serial_le]);

        assert_eq!(
            actual,
            expected[idx],
            "PACKNONCE vector {} mismatch",
            idx + 1
        );
    }
}
