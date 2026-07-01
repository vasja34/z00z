pub const ZKPACK_VER: u8 = 0x01;
pub const ZKPACK_CT_LEN: usize = 72;
pub const ZKPACK_TAG_LEN: usize = 16;
pub const ZKPACK_TOTAL_LEN: usize = 1 + ZKPACK_CT_LEN + ZKPACK_TAG_LEN;

#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum ZkPackWireError {
    #[error("Unsupported zkpack version")]
    UnsupportedVersion,
    #[error("Invalid zkpack ciphertext length")]
    InvalidCiphertextLength,
    #[error("Invalid zkpack wire length")]
    InvalidWireLength,
}

/// Encrypted ZkPack payload.
#[allow(unexpected_cfgs)]
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(
    not(z00z_formal_no_serde),
    derive(serde::Serialize, serde::Deserialize)
)]
pub struct ZkPackEncrypted {
    /// Protocol version.
    pub version: u8,
    /// Encrypted payload bytes.
    pub ciphertext: Vec<u8>,
    /// Authentication tag (16 bytes).
    pub tag: [u8; 16],
}

impl ZkPackEncrypted {
    /// Serialize the current fixed-size wire format.
    ///
    /// Layout: version\[0\] | ciphertext\[1..73\] | tag\[73..89\].
    pub fn to_bytes(&self) -> Result<[u8; ZKPACK_TOTAL_LEN], ZkPackWireError> {
        if self.version != ZKPACK_VER {
            return Err(ZkPackWireError::UnsupportedVersion);
        }
        if self.ciphertext.len() != ZKPACK_CT_LEN {
            return Err(ZkPackWireError::InvalidCiphertextLength);
        }

        let mut out = [0u8; ZKPACK_TOTAL_LEN];
        out[0] = self.version;
        out[1..(1 + ZKPACK_CT_LEN)].copy_from_slice(&self.ciphertext);
        out[(1 + ZKPACK_CT_LEN)..].copy_from_slice(&self.tag);
        Ok(out)
    }

    /// Parse the current fixed-size wire format.
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, ZkPackWireError> {
        if bytes.len() != ZKPACK_TOTAL_LEN {
            return Err(ZkPackWireError::InvalidWireLength);
        }
        if bytes[0] != ZKPACK_VER {
            return Err(ZkPackWireError::UnsupportedVersion);
        }

        let ciphertext = bytes[1..(1 + ZKPACK_CT_LEN)].to_vec();
        let mut tag = [0u8; ZKPACK_TAG_LEN];
        tag.copy_from_slice(&bytes[(1 + ZKPACK_CT_LEN)..]);

        Ok(Self {
            version: ZKPACK_VER,
            ciphertext,
            tag,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::{
        ZkPackEncrypted, ZkPackWireError, ZKPACK_CT_LEN, ZKPACK_TAG_LEN, ZKPACK_TOTAL_LEN,
    };

    #[test]
    fn test_to_bytes_len() {
        let enc = ZkPackEncrypted {
            version: 1,
            ciphertext: vec![0xAB; ZKPACK_CT_LEN],
            tag: [0xCD; ZKPACK_TAG_LEN],
        };
        let bytes = enc.to_bytes().expect("fixed v1 bytes");
        assert_eq!(bytes.len(), ZKPACK_TOTAL_LEN);
    }

    #[test]
    fn test_bytes_roundtrip() {
        let enc = ZkPackEncrypted {
            version: 1,
            ciphertext: vec![0x11; ZKPACK_CT_LEN],
            tag: [0x22; ZKPACK_TAG_LEN],
        };
        let bytes = enc.to_bytes().expect("fixed v1 bytes");
        let dec = ZkPackEncrypted::from_bytes(&bytes).expect("valid fixed v1 bytes");
        assert_eq!(enc, dec);
    }

    #[test]
    fn test_bytes_wrong_ver() {
        let mut bytes = [0u8; ZKPACK_TOTAL_LEN];
        bytes[0] = 2;
        assert_eq!(
            ZkPackEncrypted::from_bytes(&bytes),
            Err(ZkPackWireError::UnsupportedVersion)
        );
    }

    #[test]
    fn test_bytes_wrong_len() {
        let bytes88 = vec![0u8; ZKPACK_TOTAL_LEN - 1];
        let bytes90 = vec![0u8; ZKPACK_TOTAL_LEN + 1];
        assert_eq!(
            ZkPackEncrypted::from_bytes(&bytes88),
            Err(ZkPackWireError::InvalidWireLength)
        );
        assert_eq!(
            ZkPackEncrypted::from_bytes(&bytes90),
            Err(ZkPackWireError::InvalidWireLength)
        );
    }
}
