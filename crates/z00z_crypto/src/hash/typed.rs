#[derive(Clone, Copy, Eq, PartialEq, Hash)]
pub struct ConsensusHash32([u8; 32]);

impl ConsensusHash32 {
    pub const fn from_bytes(bytes: [u8; 32]) -> Self {
        Self(bytes)
    }

    pub const fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }

    pub const fn into_bytes(self) -> [u8; 32] {
        self.0
    }
}

impl core::fmt::Debug for ConsensusHash32 {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(formatter, "ConsensusHash32(..)")
    }
}

#[derive(Clone, Copy, Eq, PartialEq, Hash)]
pub struct WalletHash32([u8; 32]);

impl WalletHash32 {
    pub const fn from_bytes(bytes: [u8; 32]) -> Self {
        Self(bytes)
    }

    pub const fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }

    pub const fn into_bytes(self) -> [u8; 32] {
        self.0
    }
}

impl core::fmt::Debug for WalletHash32 {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(formatter, "WalletHash32(..)")
    }
}

#[cfg(test)]
mod tests {
    use super::{ConsensusHash32, WalletHash32};

    #[test]
    fn test_typed_hash_roundtrip() {
        let raw = [7u8; 32];
        let consensus = ConsensusHash32::from_bytes(raw);
        let wallet = WalletHash32::from_bytes(raw);

        assert_eq!(consensus.into_bytes(), raw);
        assert_eq!(wallet.into_bytes(), raw);
    }
}
