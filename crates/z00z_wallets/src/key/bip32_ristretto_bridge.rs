/// Bridges derived BIP-32 material into chain-scoped Ristretto keys.
pub struct RistrettoBridge;

const MAINNET_KEYS_LABEL: &str = "z00z/mainnet/keys";
const DEVNET_KEYS_LABEL: &str = "z00z/devnet/keys";
const TESTNET_KEYS_LABEL: &str = "z00z/testnet/keys";

fn reject_zero_key(key: &RistrettoSecretKey) -> Result<(), Bip44Error> {
    if bool::from(key.ct_eq(&RistrettoSecretKey::default())) {
        return Err(Bip44Error::WeakEntropy("derived zero secret key".into()));
    }
    Ok(())
}

impl RistrettoBridge {
    /// Convert BIP-32 extended private key to Ristretto secret key
    ///
    /// # Arguments
    ///
    /// * `xprv` - BIP-32 extended private key
    /// * `chain` - Chain type: mainnet, testnet, or devnet
    ///
    /// # Returns
    ///
    /// `Result<RistrettoSecretKey, Bip44Error>` - Ristretto key for Z00Z protocol
    ///
    /// # Security
    ///
    /// This is NOT a secp256k1 → Ristretto conversion.
    /// This is a deterministic mapping using BIP-32 entropy.
    /// Keys are NOT compatible with Bitcoin/Ethereum wallets.
    /// Only HD derivation path structure is compatible.
    ///
    /// Network separation ensures different keys for different networks.
    ///
    /// # Domain Separation
    ///
    /// Hash input includes:
    /// - Private key bytes (32 bytes)
    /// - Chain code (32 bytes)
    /// - Derivation path bytes (`Bip44Path::to_bytes()`; stable little-endian encoding)
    /// - Chain type via the domain label
    ///
    /// This prevents cross-path key reuse even when two derivations share the same
    /// `XPrv` material.
    ///
    /// Rejects a derived zero secret key (probability ~$2^{-252}$) as defense in depth.
    /// While extremely unlikely, a zero key would be catastrophic for downstream
    /// cryptographic assumptions.
    ///
    /// # Example
    ///
    /// ```
    /// use z00z_wallets::key::{RistrettoBridge, Bip32KeyDeriver, Bip44Path, MasterKeyGenerator};
    /// use std::str::FromStr;
    /// use z00z_wallets::key::Z00Z_BIP44_ASSET;
    ///
    /// let mut seed = [1u8; 32];
    /// getrandom::getrandom(&mut seed)?;
    /// let master = MasterKeyGenerator::from_seed(&seed)?;
    /// let path_str = format!("m/44'/{Z00Z_BIP44_ASSET}'/0'/0/0");
    /// let path = Bip44Path::from_str(&path_str)?;
    /// let xprv = Bip32KeyDeriver::derive_child(&master, &path)?;
    /// use z00z_core::genesis::ChainType;
    /// let ristretto_key = RistrettoBridge::to_ristretto_key(&xprv, ChainType::Devnet, &path)?;
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn to_ristretto_key(
        xprv: &XPrv,
        chain: ChainType,
        path: &Bip44Path,
    ) -> Result<RistrettoSecretKey, Bip44Error> {
        let keys_label = match chain {
            ChainType::Mainnet => MAINNET_KEYS_LABEL,
            ChainType::Devnet => DEVNET_KEYS_LABEL,
            ChainType::Testnet => TESTNET_KEYS_LABEL,
        };

        // Extract private key bytes (32 bytes)
        let private_key_bytes = Zeroizing::new(xprv.private_key().to_bytes());

        // Extract chain code from attrs (32 bytes)
        let chain_code = Zeroizing::new(xprv.attrs().chain_code);

        // Concatenate: private_key || chain_code (64 bytes total)
        let mut data = Zeroizing::new([0u8; 64]);
        data[0..32].copy_from_slice(private_key_bytes.as_ref());
        data[32..64].copy_from_slice(chain_code.as_ref());

        // Hash with domain separation.
        // The label is FROZEN and part of the derivation contract.
        let hasher = DomainHasher::<RistrettoBridgeDomain>::new_with_label(keys_label);
        let hash = hasher
            .chain(data)
            // Include the derivation path explicitly to prevent cross-path key reuse.
            // Use stable bytes instead of string formatting to avoid encoding ambiguities.
            .chain(path.to_bytes())
            .finalize();

        // Convert to Ristretto key using uniform bytes (wide reduction).
        // Use Zeroizing to guarantee sensitive intermediate bytes are wiped even on early returns.
        let mut hash_bytes = Zeroizing::new([0u8; 64]);
        hash_bytes.copy_from_slice(hash.as_ref());

        let key = RistrettoSecretKey::from_uniform_bytes(hash_bytes.as_ref())
            .map_err(|e| Bip44Error::InvalidPath(format!("Ristretto conversion failed: {}", e)))?;

        // Defense in depth: reject a derived zero secret key (probability ~$2^{-252}$).
        // Must check before using key in any cryptographic operation.
        reject_zero_key(&key)?;

        Ok(key)
    }
}
