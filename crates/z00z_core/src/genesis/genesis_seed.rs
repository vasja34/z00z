use super::{validate_genesis_seed, ChainType, FromStr, GenesisConfig, GenesisError};

/// Genesis seed for deterministic asset generation
///
/// A 32-byte cryptographically secure random seed that deterministically generates
/// all genesis assets. Same seed produces identical assets across all nodes.
///
/// # Security Requirements (M1)
///
/// MUST pass enhanced validation:
/// - NOT all zeros/ones
/// - NOT sequential patterns
/// - NOT repeating-byte patterns
/// - NOT known test seeds
#[derive(Clone, Debug)]
pub struct GenesisSeed([u8; 32]);

impl GenesisSeed {
    /// Load from config with enhanced validation (M1: Seed validation)
    pub fn from_config(config: &GenesisConfig) -> Result<Self, GenesisError> {
        let seed = config.chain.domains.genesis_seed;
        let network_type = ChainType::from_str(&config.chain.chain_type)?;

        validate_genesis_seed(&seed, network_type)?;

        Ok(GenesisSeed(seed))
    }

    /// Get seed reference (for derivation functions)
    pub fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }
}
