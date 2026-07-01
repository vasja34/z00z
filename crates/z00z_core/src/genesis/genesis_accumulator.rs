use super::{Asset, AssetClass, GenesisRightRecord, GenesisVoucherRecord};

/// Type-safe accumulator for generated genesis Assets by class
///
/// Separates Assets into class-specific vectors for:
/// - Type safety (compile-time class checking)
/// - Easy serialization (separate files per class)
/// - Efficient iteration (no runtime class filtering)
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct GenesisSettlementCorpus {
    pub coins: Vec<Asset>,
    pub tokens: Vec<Asset>,
    pub nfts: Vec<Asset>,
    pub voids: Vec<Asset>,
    #[serde(default)]
    pub rights: Vec<GenesisRightRecord>,
    #[serde(default)]
    pub vouchers: Vec<GenesisVoucherRecord>,
}

pub type GenesisAssetAccumulator = GenesisSettlementCorpus;

impl GenesisSettlementCorpus {
    /// Create a new empty accumulator
    pub fn new() -> Self {
        Self {
            coins: Vec::new(),
            tokens: Vec::new(),
            nfts: Vec::new(),
            voids: Vec::new(),
            rights: Vec::new(),
            vouchers: Vec::new(),
        }
    }

    /// Push an asset into the appropriate class vector
    pub fn push(&mut self, asset: Asset, class: AssetClass) {
        match class {
            AssetClass::Coin => self.coins.push(asset),
            AssetClass::Token => self.tokens.push(asset),
            AssetClass::Nft => self.nfts.push(asset),
            AssetClass::Void => self.voids.push(asset),
        }
    }

    /// Get total count of all assets
    pub fn total_count(&self) -> usize {
        self.coins.len() + self.tokens.len() + self.nfts.len() + self.voids.len()
    }

    /// Get total count of generated rights.
    pub fn total_right_count(&self) -> usize {
        self.rights.len()
    }

    /// Get total count of generated vouchers.
    pub fn total_voucher_count(&self) -> usize {
        self.vouchers.len()
    }

    /// Get total count of settlement leaves.
    pub fn total_leaf_count(&self) -> usize {
        self.total_count() + self.total_right_count() + self.total_voucher_count()
    }

    /// Get assets by class
    pub fn get_by_class(&self, class: AssetClass) -> &[Asset] {
        match class {
            AssetClass::Coin => &self.coins,
            AssetClass::Token => &self.tokens,
            AssetClass::Nft => &self.nfts,
            AssetClass::Void => &self.voids,
        }
    }

    /// Flatten all assets into a single vector (for verification)
    pub fn flatten(&self) -> Vec<Asset> {
        let mut all = Vec::with_capacity(self.total_count());
        all.extend_from_slice(&self.coins);
        all.extend_from_slice(&self.tokens);
        all.extend_from_slice(&self.nfts);
        all.extend_from_slice(&self.voids);
        all
    }
}

impl Default for GenesisSettlementCorpus {
    fn default() -> Self {
        Self::new()
    }
}
