/// Cached public key with timestamp for TTL management.
#[derive(Debug, Clone)]
pub struct CachedKey {
    /// The cached public key
    pub public_key: RistrettoPublicKey,
    /// Unix timestamp when the key was cached (in seconds)
    pub cached_at: u64,
}
