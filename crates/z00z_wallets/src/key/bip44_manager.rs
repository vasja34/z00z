/// High-level BIP-44 key manager built on the validated Z00Z derivation policy.
pub struct Bip44KeyManager {
    seed: Hidden<Bip39Seed64>,
    chain: ChainType,
}

impl fmt::Debug for Bip44KeyManager {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Bip44KeyManager")
            .field("seed", &"<redacted>")
            .field("chain", &self.chain)
            .finish()
    }
}

impl Bip44KeyManager {
    /// Create new key manager from seed.
    pub fn new(seed: Bip39Seed64, asset_type: u32, chain: ChainType) -> Result<Self, Bip44Error> {
        seed.validate_entropy()?;

        if asset_type != Z00Z_BIP44_ASSET {
            return Err(Bip44Error::NonStandardPath {
                reason: Bip44ViolationReason::AssetTypeValueMismatch,
                component: "asset_type".to_string(),
            });
        }

        let _ = MasterKeyGenerator::from_seed(seed.as_bytes())?;

        Ok(Self {
            seed: Hidden::hide(seed),
            chain,
        })
    }

    pub(crate) fn chain(&self) -> ChainType {
        self.chain
    }

    /// Explicitly zeroize the stored BIP-39 seed.
    pub fn zeroize_seed(&mut self) {
        self.zeroize_all();
    }

    /// Zeroize all sensitive in-memory state.
    pub fn zeroize_all(&mut self) {
        self.seed.zeroize();

        #[cfg(test)]
        {
            let is_zeroed = self.seed.with_revealed(|seed| seed.is_all_zeros());
            SEED_ZEROIZED.store(is_zeroed, Ordering::SeqCst);
        }
    }

    /// Derive account-level key.
    pub fn derive_account_key(&self, account: u32) -> Result<XPrv, Bip44Error> {
        let path = DerivationPath::from_str(&format!(
            "m/{}'/{}'/{}'",
            Z00Z_BIP44_PURPOSE, Z00Z_BIP44_ASSET, account
        ))
        .map_err(|e| Bip44Error::InvalidPath(format!("account derivation path: {}", e)))?;

        self.seed
            .with_revealed(|seed| XPrv::derive_from_path(&seed.as_bytes()[..], &path))
            .map_err(Bip44Error::from)
    }

    /// Derive an address extended private key for an already-constructed BIP-44 path.
    pub fn derive_address_key_for_path(&self, path: &Bip44Path) -> Result<XPrv, Bip44Error> {
        Bip44Validator::validate(path)?;

        self.seed
            .with_revealed(|seed| {
                XPrv::derive_from_path(&seed.as_bytes()[..], &path.to_derivation_path())
            })
            .map_err(Bip44Error::from)
    }

    /// Derive address key for the canonical Z00Z BIP-44 path.
    pub fn derive_address_key(
        &self,
        account: u32,
        change: u8,
        address_index: u32,
    ) -> Result<XPrv, Bip44Error> {
        let path = Bip44Path::new_z00z(account, change, address_index)?;
        self.derive_address_key_for_path(&path)
    }

    /// Derive a Ristretto key for Z00Z protocol use.
    pub fn derive_ristretto_key(
        &self,
        account: u32,
        change: u8,
        address_index: u32,
    ) -> Result<RistrettoSecretKey, Bip44Error> {
        let xprv = self.derive_address_key(account, change, address_index)?;
        let path = Bip44Path::new_z00z(account, change, address_index)?;
        RistrettoBridge::to_ristretto_key(&xprv, self.chain, &path)
    }
}

impl Drop for Bip44KeyManager {
    fn drop(&mut self) {
        self.zeroize_seed();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::key::{validate_entropy, validate_entropy_with_warnings, EntropyWarning};
    use std::str::FromStr;
    use z00z_crypto::expert::encoding::ByteArray;
    use z00z_utils::rng::{MockRngProvider, RngCoreExt};

    fn test_seed_bytes() -> [u8; 64] {
        let provider = MockRngProvider::with_u64_seed(2_345_678);
        let mut rng = provider.rng();
        let mut seed = [0u8; 64];
        rng.fill_bytes_ext(&mut seed);
        seed
    }

    include!("test_bip44_manager_suite.rs");
}
