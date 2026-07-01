use thiserror::Error;

/// OWF circuit constraint: serial_id MUST be exactly 4 bytes LE — NOT in plaintext (in leaf_ad AAD)
pub const SERIAL_ID_BYTE_LEN: usize = 4;

#[derive(Debug, Error, Clone, Copy, PartialEq, Eq)]
pub enum SerialIdError {
    #[error("invalid serial_id length: expected {expected}, got {got}")]
    InvalidLength { expected: usize, got: usize },
    #[error("serial_id out of bounds: {serial_id} >= {max}")]
    OutOfBounds { serial_id: u32, max: u32 },
}

#[must_use]
pub fn serialize_serial_id(serial_id: u32) -> [u8; SERIAL_ID_BYTE_LEN] {
    serial_id.to_le_bytes()
}

pub fn deserialize_serial_id(bytes: &[u8]) -> Result<u32, SerialIdError> {
    if bytes.len() != SERIAL_ID_BYTE_LEN {
        return Err(SerialIdError::InvalidLength {
            expected: SERIAL_ID_BYTE_LEN,
            got: bytes.len(),
        });
    }

    let mut buf = [0u8; SERIAL_ID_BYTE_LEN];
    buf.copy_from_slice(bytes);
    Ok(u32::from_le_bytes(buf))
}

pub fn validate_serial_bounds(serial_id: u32, max: u32) -> Result<(), SerialIdError> {
    if serial_id >= max {
        return Err(SerialIdError::OutOfBounds { serial_id, max });
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{deserialize_serial_id, serialize_serial_id, SerialIdError, SERIAL_ID_BYTE_LEN};

    #[test]
    fn test_canonical_zero() {
        assert_eq!(serialize_serial_id(0), [0x00, 0x00, 0x00, 0x00]);
    }

    #[test]
    fn test_canonical_one() {
        assert_eq!(serialize_serial_id(1), [0x01, 0x00, 0x00, 0x00]);
    }

    #[test]
    fn test_canonical_256() {
        assert_eq!(serialize_serial_id(256), [0x00, 0x01, 0x00, 0x00]);
    }

    #[test]
    fn test_canonical_le() {
        assert_eq!(serialize_serial_id(0x1234_5678), [0x78, 0x56, 0x34, 0x12]);
    }

    #[test]
    fn test_reject_5_bytes() {
        let bytes = [0u8; 5];
        assert!(matches!(
            deserialize_serial_id(&bytes),
            Err(SerialIdError::InvalidLength {
                expected: SERIAL_ID_BYTE_LEN,
                got: 5
            })
        ));
    }

    #[test]
    fn test_reject_3_bytes() {
        let bytes = [0u8; 3];
        assert!(matches!(
            deserialize_serial_id(&bytes),
            Err(SerialIdError::InvalidLength {
                expected: SERIAL_ID_BYTE_LEN,
                got: 3
            })
        ));
    }

    #[test]
    fn test_roundtrip_max() {
        let bytes = serialize_serial_id(u32::MAX);
        assert_eq!(deserialize_serial_id(&bytes), Ok(u32::MAX));
    }
}
