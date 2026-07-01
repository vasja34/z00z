/// Generates master material from a validated BIP-39 seed.
pub struct MasterKeyGenerator;

impl MasterKeyGenerator {
    /// Generate master extended private key from seed
    ///
    /// # Arguments
    ///
    /// * `seed` - Seed bytes (32-64 bytes)
    ///
    /// # Returns
    ///
    /// `Result<XPrv, Bip44Error>` - Master extended private key or error
    ///
    /// # Errors
    ///
    /// Returns `Bip44Error::InvalidSeed` if seed length is not 32-64 bytes
    ///
    /// # Example
    ///
    /// ```
    /// use z00z_wallets::key::MasterKeyGenerator;
    ///
    /// let mut seed = [1u8; 32];
    /// getrandom::getrandom(&mut seed)?;
    /// let master = MasterKeyGenerator::from_seed(&seed)?;
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn from_seed(seed: &[u8]) -> Result<XPrv, Bip44Error> {
        XPrv::new(seed).map_err(|e| {
            if matches!(e, bip32::Error::SeedLength) {
                Bip44Error::InvalidSeed(seed.len())
            } else {
                Bip44Error::from(e)
            }
        })
    }
}

/// BIP-32 key deriver
///
/// Performs hierarchical deterministic key derivation using BIP-32 CKD algorithm.
///
/// # Security
///
/// - Uses proper BIP-32 Child Key Derivation with chain codes
/// - Supports both hardened and non-hardened derivation
/// - Maintains extended keys (private key + chain code)
///
/// # Example
///
/// ```
/// use z00z_wallets::key::{Bip32KeyDeriver, Bip44Path, MasterKeyGenerator};
/// use std::str::FromStr;
/// use z00z_wallets::key::Z00Z_BIP44_ASSET;
///
/// let mut seed = [1u8; 32];
/// getrandom::getrandom(&mut seed)?;
/// let master = MasterKeyGenerator::from_seed(&seed)?;
/// let path_str = format!("m/44'/{Z00Z_BIP44_ASSET}'/0'/0/0");
/// let path = Bip44Path::from_str(&path_str)?;
/// let child = Bip32KeyDeriver::derive_child(&master, &path)?;
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
pub struct Bip32KeyDeriver;

impl Bip32KeyDeriver {
    /// Derive child key from parent using full BIP-32 CKD
    ///
    /// # Arguments
    ///
    /// * `parent` - Parent extended private key
    /// * `path` - Complete derivation path
    ///
    /// # Returns
    ///
    /// `Result<XPrv, Bip44Error>` - Child extended private key
    ///
    /// # Example
    ///
    /// ```
    /// use z00z_wallets::key::{Bip32KeyDeriver, Bip44Path, MasterKeyGenerator};
    /// use std::str::FromStr;
    /// use z00z_wallets::key::Z00Z_BIP44_ASSET;
    ///
    /// let mut seed = [1u8; 32];
    /// getrandom::getrandom(&mut seed)?;
    /// let master = MasterKeyGenerator::from_seed(&seed)?;
    /// let path_str = format!("m/44'/{Z00Z_BIP44_ASSET}'/0'/0/0");
    /// let path = Bip44Path::from_str(&path_str)?;
    /// let child = Bip32KeyDeriver::derive_child(&master, &path)?;
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn derive_child(parent: &XPrv, path: &Bip44Path) -> Result<XPrv, Bip44Error> {
        let derivation_path = path.to_derivation_path();
        // Derive from parent by iterating through path.
        // Note: XPrv implements Zeroize (bip32 v0.4+), so clones are wiped on drop.
        let mut result = parent.clone();
        for child_num in derivation_path.iter() {
            result = result.derive_child(child_num)?;
        }
        Ok(result)
    }

    /// Derive from any intermediate level (account, change, or partial path)
    ///
    /// # Arguments
    ///
    /// * `parent` - Parent extended private key
    /// * `partial_path` - Partial derivation path as slice of ChildNumber.
    ///   If empty, this returns a clone of `parent` unchanged (zero-step derivation).
    ///
    /// # Returns
    ///
    /// `Result<XPrv, Bip44Error>` - Derived extended private key
    ///
    /// # Example
    ///
    /// ```
    /// use bip32::ChildNumber;
    /// use z00z_wallets::key::{Bip32KeyDeriver, MasterKeyGenerator, Z00Z_BIP44_ASSET};
    ///
    /// let mut seed = [1u8; 32];
    /// getrandom::getrandom(&mut seed)?;
    /// let master = MasterKeyGenerator::from_seed(&seed)?;
    /// let purpose = ChildNumber::new(44, true)?;
    /// let asset = ChildNumber::new(Z00Z_BIP44_ASSET, true)?;
    /// let account = ChildNumber::new(0, true)?;
    /// let partial = vec![purpose, asset, account];
    /// let account_key = Bip32KeyDeriver::derive_from_intermediate(&master, &partial)?;
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn derive_from_intermediate(
        parent: &XPrv,
        partial_path: &[ChildNumber],
    ) -> Result<XPrv, Bip44Error> {
        // Note: XPrv implements Zeroize (bip32 v0.4+), so clones are wiped on drop.
        let mut result = parent.clone();
        for child_num in partial_path {
            result = result.derive_child(*child_num)?;
        }
        Ok(result)
    }
}
