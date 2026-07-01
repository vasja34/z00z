use super::{
    decode_asset_pack, deserialize_asset_pack, is_valid_asset_pack_length, serialize_asset_pack,
    AssetLeaf, AssetPackPlain, AssetPackPlainMemo, DecodedAssetPack, PackErr,
};
use crate::assets::{validate_serial_id_version, AssetPackVersion};
use z00z_utils::rng::SystemRngProvider;

#[test]
fn test_len_72() {
    assert!(AssetPackPlain::is_valid_length(&[0u8; 72]));
}

#[test]
fn test_len_71() {
    assert!(!AssetPackPlain::is_valid_length(&[0u8; 71]));
}

#[test]
fn test_len_0() {
    assert!(!AssetPackPlain::is_valid_length(&[]));
}

#[test]
fn test_asset_pack_creation() {
    let pack = AssetPackPlain {
        value: 1000,
        blinding: [0x22; 32],
        s_out: [0x11; 32],
    };

    assert_eq!(pack.value, 1000);
    assert_eq!(pack.blinding, [0x22; 32]);
    assert_eq!(pack.s_out, [0x11; 32]);
}

#[test]
fn test_asset_pack_serialization() {
    let pack = AssetPackPlain {
        value: 1000,
        blinding: [0x22; 32],
        s_out: [0x11; 32],
    };

    let bytes = pack.to_bytes();
    assert_eq!(bytes.len(), AssetPackPlain::SIZE);
    assert_eq!(&bytes[0..8], &1000u64.to_le_bytes());
    assert_eq!(&bytes[8..40], &[0x22; 32]);
    assert_eq!(&bytes[40..72], &[0x11; 32]);
}

#[test]
fn test_asset_pack_bounded_parsing() {
    assert!(AssetPackPlain::from_bytes(&[0u8; 71]).is_none());
    assert!(AssetPackPlain::from_bytes(&[0u8; 73]).is_none());
    assert!(AssetPackPlain::from_bytes(&[]).is_none());
    assert!(AssetPackPlain::from_bytes(&[0u8; 200]).is_none());
    assert!(AssetPackPlain::from_bytes(&[0u8; 72]).is_some());
}

#[test]
fn test_asset_pack_roundtrip() {
    let mut blind = [0u8; 32];
    blind[0] = 1;
    let pack = AssetPackPlain {
        value: 5000,
        blinding: blind,
        s_out: [0x44; 32],
    };

    let bytes = serialize_asset_pack(&pack);
    let decoded = deserialize_asset_pack(&bytes);
    assert_eq!(decoded, Some(pack));
    assert!(is_valid_asset_pack_length(&bytes));
}

#[test]
fn test_from_bytes_fuzz_input() {
    use z00z_utils::rng::RngCoreExt;
    let mut rng = SystemRngProvider.rng();

    for _ in 0..1000 {
        let mut len_bytes = [0u8; 4];
        rng.fill_bytes_ext(&mut len_bytes);
        let len = (u32::from_le_bytes(len_bytes) % 201) as usize;
        let mut bytes = vec![0u8; len];
        rng.fill_bytes_ext(&mut bytes);
        let _ = AssetPackPlain::from_bytes(&bytes);
    }
}

#[test]
fn test_asset_pack_golden_72b() {
    let pack = AssetPackPlain {
        value: 1000,
        blinding: [0x22; 32],
        s_out: [0x11; 32],
    };

    let bytes = pack.to_bytes();
    let expected = hex::decode(concat!(
        "e803000000000000",
        "2222222222222222222222222222222222222222222222222222222222222222",
        "1111111111111111111111111111111111111111111111111111111111111111"
    ))
    .expect("golden hex");

    assert_eq!(bytes, expected);
}

#[test]
fn test_version_pre_decrypt() {
    let leaf = AssetLeaf {
        serial_id: 42,
        ..Default::default()
    };

    let version = validate_serial_id_version(leaf.serial_id);
    assert_eq!(version, AssetPackVersion::Basic);
    assert_ne!(leaf.enc_pack.ciphertext.len(), leaf.serial_id as usize);
}

#[test]
fn test_reject_trunc_all() {
    for len in 0..72 {
        let bytes = vec![0u8; len];
        assert!(AssetPackPlain::from_bytes(&bytes).is_none());
    }
}

#[test]
fn test_reject_overs_all() {
    for len in 73..=200 {
        let bytes = vec![0u8; len];
        assert!(AssetPackPlain::from_bytes(&bytes).is_none());
    }
}

#[test]
fn test_decode_strict_bad_len() {
    assert_eq!(
        AssetPackPlain::decode_strict(&[0u8; 71]),
        Err(PackErr::BadLen)
    );
    assert_eq!(
        AssetPackPlain::decode_strict(&[0u8; 73]),
        Err(PackErr::BadLen)
    );
}

#[test]
fn test_decode_checked_bad_blind() {
    let mut bytes = vec![0u8; AssetPackPlain::SIZE];
    bytes[0..8].copy_from_slice(&7u64.to_le_bytes());
    bytes[8..40].copy_from_slice(&[0xFFu8; 32]);
    bytes[40..72].copy_from_slice(&[0x11u8; 32]);

    assert_eq!(
        AssetPackPlain::decode_checked(&bytes),
        Err(PackErr::BadBlind)
    );
}

#[test]
fn test_value_endian() {
    let mut le = vec![0u8; AssetPackPlain::SIZE];
    let mut be = vec![0u8; AssetPackPlain::SIZE];
    let v = 0x0102_0304_0506_0708u64;

    le[0..8].copy_from_slice(&v.to_le_bytes());
    be[0..8].copy_from_slice(&v.to_be_bytes());

    let le_pack = AssetPackPlain::decode_strict(&le).expect("le");
    let be_pack = AssetPackPlain::decode_strict(&be).expect("be");

    assert_eq!(le_pack.value, v);
    assert_ne!(be_pack.value, v);
}

#[test]
fn test_offsets() {
    let pack = AssetPackPlain {
        value: 0x0807_0605_0403_0201,
        blinding: [0xA5u8; 32],
        s_out: [0x5Au8; 32],
    };
    let bytes = pack.to_bytes();

    assert_eq!(bytes.len(), AssetPackPlain::SIZE);
    assert_eq!(&bytes[0..8], &pack.value.to_le_bytes());
    assert_eq!(&bytes[8..40], &pack.blinding);
    assert_eq!(&bytes[40..72], &pack.s_out);
}

#[test]
fn test_memo_roundtrip() {
    let mut blind = [0u8; 32];
    blind[0] = 1;
    let pack = AssetPackPlainMemo {
        value: 77,
        blinding: blind,
        s_out: [0x55; 32],
        memo: b"phase035-memo".to_vec(),
    };

    let bytes = pack.encode_checked().expect("encode");
    let decoded = AssetPackPlainMemo::decode_checked(&bytes).expect("decode");

    assert_eq!(decoded, pack);
}

#[test]
fn test_memo_empty_roundtrip() {
    let mut blind = [0u8; 32];
    blind[0] = 1;
    let pack = AssetPackPlainMemo {
        value: 11,
        blinding: blind,
        s_out: [0x33; 32],
        memo: Vec::new(),
    };

    let bytes = pack.encode_checked().expect("encode");
    let decoded = AssetPackPlainMemo::from_bytes(&bytes);

    assert_eq!(decoded, Some(pack));
}

#[test]
fn test_memo_rejects_oversize() {
    let mut blind = [0u8; 32];
    blind[0] = 1;
    let pack = AssetPackPlainMemo {
        value: 1,
        blinding: blind,
        s_out: [0x22; 32],
        memo: vec![0x99; AssetPackPlainMemo::MEMO_MAX + 1],
    };

    assert_eq!(pack.encode_checked(), Err(PackErr::BadMemo));
}

#[test]
fn test_memo_rejects_bad_len() {
    let mut blind = [0u8; 32];
    blind[0] = 1;
    let pack = AssetPackPlainMemo {
        value: 9,
        blinding: blind,
        s_out: [0x10; 32],
        memo: b"abcd".to_vec(),
    };
    let mut bytes = pack.encode_checked().expect("encode");
    bytes[72..74].copy_from_slice(&10u16.to_le_bytes());

    assert_eq!(
        AssetPackPlainMemo::decode_checked(&bytes),
        Err(PackErr::BadLen)
    );
}

#[test]
fn test_memo_rejects_bad_blind() {
    let mut bytes = Vec::with_capacity(AssetPackPlainMemo::HEAD_SIZE + 4);
    bytes.extend_from_slice(&5u64.to_le_bytes());
    bytes.extend_from_slice(&[0xFF; 32]);
    bytes.extend_from_slice(&[0x11; 32]);
    bytes.extend_from_slice(&4u16.to_le_bytes());
    bytes.extend_from_slice(b"memo");

    assert_eq!(
        AssetPackPlainMemo::decode_checked(&bytes),
        Err(PackErr::BadBlind)
    );
}

#[test]
fn test_decode_asset_basic_lane() {
    let mut blind = [0u8; 32];
    blind[0] = 1;
    let pack = AssetPackPlain {
        value: 42,
        blinding: blind,
        s_out: [0x77; 32],
    };

    let decoded = decode_asset_pack(&pack.to_bytes(), AssetPackVersion::Basic).expect("basic");
    assert_eq!(decoded, DecodedAssetPack::Basic(pack));
}

#[test]
fn test_decode_asset_memo_lane() {
    let mut blind = [0u8; 32];
    blind[0] = 1;
    let pack = AssetPackPlainMemo {
        value: 64,
        blinding: blind,
        s_out: [0x12; 32],
        memo: b"wallet-note".to_vec(),
    };

    let bytes = pack.encode_checked().expect("encode");
    let decoded = decode_asset_pack(&bytes, validate_serial_id_version(1_000_000)).expect("memo");
    assert_eq!(decoded, DecodedAssetPack::Memo(pack));
}

#[test]
fn test_decode_asset_unknown_lane() {
    let bytes = [0u8; AssetPackPlain::SIZE];
    assert_eq!(
        decode_asset_pack(&bytes, validate_serial_id_version(2_000_000)),
        Err(PackErr::BadVer)
    );
}
