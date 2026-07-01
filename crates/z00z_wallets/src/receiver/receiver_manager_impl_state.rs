/// Authenticated serialization of cached public address keys.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ReceiverCacheState {
    /// Exported `(path, spend_key, view_key)` tuples.
    pub entries: Vec<(Bip44Path, Vec<u8>, Vec<u8>)>,
    /// State format version for compatibility validation.
    pub version: u8,
    /// HMAC over the wallet-bound state payload.
    pub hmac: [u8; 32],
}

const RECEIVER_CACHE_MAC_ACCOUNT: u32 = 2_000_000;
const RECEIVER_CACHE_HMAC_LABEL: &str = "v3";

impl ReceiverCacheState {
    fn mac_key(key_manager: &dyn KeyManager, wallet_id: &[u8]) -> ReceiverManagerResult<[u8; 32]> {
        let path = Bip44Path::new_z00z(RECEIVER_CACHE_MAC_ACCOUNT, 0, 0)
            .map_err(|error| ReceiverManagerError::KeyDerivation(error.to_string()))?;
        let base_key = key_manager
            .derive_secret_transient(&path)
            .map_err(|error| ReceiverManagerError::KeyDerivation(error.to_string()))?;

        let mut info = Vec::with_capacity(22 + wallet_id.len());
        info.extend_from_slice(b"Z00Z/Wallet/Cache/MAC");
        info.extend_from_slice(wallet_id);

        hkdf_expand_32(base_key.as_bytes(), &[], &info)
            .map(|secret| secret.into_inner())
            .map_err(|_| ReceiverManagerError::MacKeyDerivationFailed)
    }

    fn mac(entries_bytes: &[u8], version: u8, wallet_id: &[u8], mac_key: &[u8; 32]) -> [u8; 32] {
        let mut msg = Vec::with_capacity(wallet_id.len() + 1 + entries_bytes.len());
        msg.extend_from_slice(wallet_id);
        msg.push(version);
        msg.extend_from_slice(entries_bytes);
        hmac_sha256(
            mac_key,
            ReceiverCacheHmacDomain::domain(),
            RECEIVER_CACHE_HMAC_LABEL,
            &msg,
        )
    }

    fn entries_bytes(entries: &[ReceiverCacheEntry]) -> ReceiverManagerResult<Vec<u8>> {
        to_canonical(entries).map_err(|error| {
            ReceiverManagerError::InvalidReceiverCacheState(error.to_string())
        })
    }

    /// Compute and store the authenticated state HMAC.
    pub fn sign(&mut self, wallet_id: &[u8], mac_key: &[u8; 32]) -> ReceiverManagerResult<()> {
        let entries_bytes = Self::entries_bytes(&self.entries)?;
        self.hmac = Self::mac(&entries_bytes, self.version, wallet_id, mac_key);
        Ok(())
    }

    /// Verify the authenticated state HMAC in constant time.
    pub fn verify(&self, wallet_id: &[u8], mac_key: &[u8; 32]) -> ReceiverManagerResult<()> {
        let entries_bytes = Self::entries_bytes(&self.entries)?;
        let computed = Self::mac(&entries_bytes, self.version, wallet_id, mac_key);
        if self.hmac.ct_eq(&computed).unwrap_u8() != 1 {
            return Err(ReceiverManagerError::CacheAuthenticationFailed);
        }
        Ok(())
    }
}