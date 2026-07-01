/// Chain identifier (immutable binding for a wallet).
///
/// This is a compact network discriminator used across the project:
/// - `1` = mainnet
/// - `2` = testnet
/// - `3` = devnet
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub struct ChainId(pub u8);

impl ChainId {
    /// Mainnet chain id.
    pub const MAINNET: ChainId = ChainId(1);
    /// Testnet chain id.
    pub const TESTNET: ChainId = ChainId(2);
    /// Devnet chain id.
    pub const DEVNET: ChainId = ChainId(3);

    /// Returns the raw numeric chain id.
    pub fn as_u8(&self) -> u8 {
        self.0
    }

    /// Returns the canonical wire numeric chain id (`u32`, zero-extended from `u8`).
    ///
    /// This is the Phase 2 wire-compatible representation for tx package fields.
    pub fn as_u32(&self) -> u32 {
        u32::from(self.0)
    }
}

impl TryFrom<u8> for ChainId {
    type Error = &'static str;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            1..=3 => Ok(Self(value)),
            _ => Err("invalid chain id"),
        }
    }
}

impl TryFrom<u32> for ChainId {
    type Error = &'static str;

    fn try_from(value: u32) -> Result<Self, Self::Error> {
        let narrowed = u8::try_from(value).map_err(|_| "invalid chain id")?;
        Self::try_from(narrowed)
    }
}

impl From<ChainId> for ChainType {
    fn from(chain_id: ChainId) -> Self {
        match chain_id {
            ChainId::MAINNET => ChainType::Mainnet,
            ChainId::TESTNET => ChainType::Testnet,
            ChainId::DEVNET => ChainType::Devnet,
            _ => ChainType::Devnet,
        }
    }
}

impl From<ChainId> for u32 {
    fn from(value: ChainId) -> Self {
        value.as_u32()
    }
}

impl From<&ChainId> for u32 {
    fn from(value: &ChainId) -> Self {
        value.as_u32()
    }
}

#[cfg(test)]
mod chain_id_tests {
    use super::ChainId;

    #[test]
    fn test_chain_id_to_u32() {
        assert_eq!(u32::from(ChainId::MAINNET), 1u32);
        assert_eq!(u32::from(ChainId::TESTNET), 2u32);
        assert_eq!(u32::from(ChainId::DEVNET), 3u32);
    }

    #[test]
    fn test_chain_id_le_wire() {
        let bytes = u32::from(ChainId::DEVNET).to_le_bytes();
        assert_eq!(bytes, [3u8, 0u8, 0u8, 0u8]);
    }

    #[test]
    fn test_chain_id_u32_ok() {
        assert_eq!(ChainId::try_from(1u32), Ok(ChainId::MAINNET));
        assert_eq!(ChainId::try_from(2u32), Ok(ChainId::TESTNET));
        assert_eq!(ChainId::try_from(3u32), Ok(ChainId::DEVNET));
    }

    #[test]
    fn test_chain_id_u32_bad() {
        assert!(ChainId::try_from(0u32).is_err());
        assert!(ChainId::try_from(4u32).is_err());
        assert!(ChainId::try_from(256u32).is_err());
    }
}
