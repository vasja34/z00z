use z00z_core::assets::{
    deserialize_serial_id, serialize_serial_id, validate_serial_bounds, validate_serial_id_version,
    AssetPackVersion, SerialIdError,
};
use z00z_utils::codec::{BincodeCodec, Codec, JsonCodec};

#[test]
fn test_serial_canonical() {
    assert_eq!(serialize_serial_id(0), [0x00, 0x00, 0x00, 0x00]);
    assert_eq!(serialize_serial_id(1), [0x01, 0x00, 0x00, 0x00]);
    assert_eq!(serialize_serial_id(256), [0x00, 0x01, 0x00, 0x00]);
    assert_eq!(serialize_serial_id(0x1234_5678), [0x78, 0x56, 0x34, 0x12]);
}

#[test]
fn test_be_differs() {
    let value = 0x1234_5678u32;
    assert_ne!(value.to_be_bytes(), value.to_le_bytes());
}

#[test]
fn test_roundtrip_corners() {
    for value in [0u32, 1, 255, 256, 65_535, 65_536, u32::MAX] {
        assert_eq!(u32::from_le_bytes(u32::to_le_bytes(value)), value);
    }
}

#[test]
fn test_slice_valid() {
    assert_eq!(deserialize_serial_id(&[0x42, 0x00, 0x00, 0x00]), Ok(0x42));
}

#[test]
fn test_slice_reject_1() {
    assert!(matches!(
        deserialize_serial_id(&[0x42]),
        Err(SerialIdError::InvalidLength {
            expected: 4,
            got: 1
        })
    ));
}

#[test]
fn test_slice_reject_2() {
    assert!(matches!(
        deserialize_serial_id(&[0x42, 0x00]),
        Err(SerialIdError::InvalidLength {
            expected: 4,
            got: 2
        })
    ));
}

#[test]
fn test_slice_reject_5() {
    assert!(matches!(
        deserialize_serial_id(&[0x42, 0x00, 0x00, 0x00, 0x00]),
        Err(SerialIdError::InvalidLength {
            expected: 4,
            got: 5
        })
    ));
}

#[test]
fn test_bounds_valid() {
    assert!(validate_serial_bounds(50, 100).is_ok());
}

#[test]
fn test_bounds_equal() {
    assert!(matches!(
        validate_serial_bounds(1000, 1000),
        Err(SerialIdError::OutOfBounds {
            serial_id: 1000,
            max: 1000
        })
    ));
}

#[test]
fn test_bounds_exceed() {
    assert!(matches!(
        validate_serial_bounds(1001, 1000),
        Err(SerialIdError::OutOfBounds {
            serial_id: 1001,
            max: 1000
        })
    ));
}

#[test]
fn test_bounds_zero_zero() {
    assert!(matches!(
        validate_serial_bounds(0, 0),
        Err(SerialIdError::OutOfBounds {
            serial_id: 0,
            max: 0
        })
    ));
}

#[test]
fn test_bounds_zero_one() {
    assert!(validate_serial_bounds(0, 1).is_ok());
}

#[test]
fn test_bounds_max() {
    assert!(matches!(
        validate_serial_bounds(u32::MAX, u32::MAX),
        Err(SerialIdError::OutOfBounds {
            serial_id: u32::MAX,
            max: u32::MAX
        })
    ));
}

#[test]
fn test_basic_lane_low_bound() {
    assert_eq!(validate_serial_id_version(0), AssetPackVersion::Basic);
}

#[test]
fn test_basic_lane_high_bound() {
    assert_eq!(validate_serial_id_version(999_999), AssetPackVersion::Basic);
}

#[test]
fn test_memo_lane_low_bound() {
    assert_eq!(
        validate_serial_id_version(1_000_000),
        AssetPackVersion::Memo
    );
}

#[test]
fn test_memo_lane_high_bound() {
    assert_eq!(
        validate_serial_id_version(1_999_999),
        AssetPackVersion::Memo
    );
}

#[test]
fn test_unknown_lane_low_bound() {
    assert_eq!(
        validate_serial_id_version(2_000_000),
        AssetPackVersion::Unknown
    );
}

#[test]
fn test_unknown_lane_max_bound() {
    assert_eq!(
        validate_serial_id_version(u32::MAX),
        AssetPackVersion::Unknown
    );
}

#[test]
fn test_json_roundtrip() {
    let bytes = JsonCodec.serialize(&42u32).expect("json serialize");
    let got: u32 = JsonCodec.deserialize(&bytes).expect("json deserialize");
    assert_eq!(got, 42u32);
}

#[test]
fn test_bin_roundtrip() {
    let codec = BincodeCodec;
    let bytes = codec.serialize(&42u32).expect("bin serialize");
    let got: u32 = codec.deserialize(&bytes).expect("bin deserialize");
    assert_eq!(got, 42u32);
}
