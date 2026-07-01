use super::{FromStr, GenesisError};

// ============================================================================
// Network Type — Determines cryptographic domain for genesis operations
// ============================================================================

/// Network type for genesis operations — ensures cryptographic domain separation
///
/// Each network (devnet/testnet/mainnet) uses distinct hash domains to prevent
/// cross-network replay attacks and accidental mixing of test/production assets.
///
/// # Security Properties
/// - Domain separation: Different networks produce different outputs for same inputs
/// - No cross-network replay: Assets from devnet cannot be used on mainnet
/// - Type safety: Network type must be explicitly provided at genesis
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum ChainType {
    /// Development network — for local testing and debugging
    Devnet,
    /// Test network — for integration testing and staging
    Testnet,
    /// Main network — production environment
    Mainnet,
}

impl ChainType {
    /// Get human-readable name for the network
    pub fn as_str(&self) -> &'static str {
        match self {
            ChainType::Devnet => "devnet",
            ChainType::Testnet => "testnet",
            ChainType::Mainnet => "mainnet",
        }
    }
}

impl std::fmt::Display for ChainType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl FromStr for ChainType {
    type Err = GenesisError;

    /// Parse network type from string (case-insensitive)
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "devnet" => Ok(ChainType::Devnet),
            "testnet" => Ok(ChainType::Testnet),
            "mainnet" => Ok(ChainType::Mainnet),
            _ => Err(GenesisError::InvalidConfig(format!(
                "Unknown network type: '{}' (expected devnet/testnet/mainnet)",
                s
            ))),
        }
    }
}
