use std::collections::HashMap;
use std::sync::{Condvar, Mutex};

struct DerivationFlight {
    is_deriving: Mutex<bool>,
    result: Mutex<Option<DerivationFlightResult>>,
    wait: Condvar,
}

#[derive(Clone)]
struct DerivationFlightResult {
    completed_at: u64,
    result: Result<RistrettoPublicKey>,
}

impl DerivationFlight {
    fn active() -> Self {
        Self {
            is_deriving: Mutex::new(true),
            result: Mutex::new(None),
            wait: Condvar::new(),
        }
    }
}

/// In-memory key manager implementation.
///
/// Security Note: This implementation caches only public keys, not secret keys.
/// Secret keys are derived on-demand and never stored in memory longer than necessary.
/// This mitigates memory exposure risks from memory dumps or swap files.
pub struct KeyManagerImpl<R: SecureRngProvider = SystemRngProvider> {
    /// Encrypted seed container (for persistence round-trip)
    encrypted_seed: Option<CipherSeedContainer>,
    /// New BIP-32/BIP-44 compliant key manager
    bip44_manager: Option<Bip44KeyManager>,
    /// Cache of public keys only (not secrets) for performance
    /// Protected by RwLock for thread-safe access
    /// Uses LRU eviction to prevent memory exhaustion attacks
    /// Keys have TTL and are evicted when expired
    derived_public_keys: RwLock<LruCache<Bip44Path, CachedKey>>,
    /// Tracks same-path derivations so cache misses can coalesce without cross-path wakeups.
    deriving_paths: Mutex<HashMap<Bip44Path, std::sync::Arc<DerivationFlight>>>,
    derivation_count: AtomicUsize,
    /// Chain identifier for Ristretto key separation (e.g. "devnet")
    chain: ChainType,
    /// Audit logger for key operations.
    logger: Arc<dyn Logger>,
    /// Metrics sink for key derivation observability.
    metrics: Arc<dyn MetricsSink>,
    /// Time provider for latency measurement.
    time_provider: Arc<dyn TimeProvider>,
    /// RNG provider used for operations that require randomness (e.g. cache spot-check selection).
    rng_provider: R,
    /// Next external address index (change = 0).
    gap_external: AtomicU32,
    /// Next internal address index (change = 1).
    gap_internal: AtomicU32,
    /// Last-used external address index + 1 (change = 0).
    ///
    /// Stored as `last_used + 1` to allow a sentinel `0` meaning "no used addresses".
    last_used_ext: AtomicU32,
    /// Last-used internal address index + 1 (change = 1).
    ///
    /// Stored as `last_used + 1` to allow a sentinel `0` meaning "no used addresses".
    last_used_int: AtomicU32,
    /// Total number of key derivations (for spot-check trigger).
    /// Incremented on each derive_key() call to trigger periodic cache validation.
    derive_count: AtomicU32,
}

impl<R: SecureRngProvider> std::fmt::Debug for KeyManagerImpl<R> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("KeyManagerImpl")
            .field("has_encrypted_seed", &self.encrypted_seed.is_some())
            .field("has_bip44_manager", &self.bip44_manager.is_some())
            .field("chain", &self.chain)
            .finish()
    }
}

impl Default for KeyManagerImpl<SystemRngProvider> {
    fn default() -> Self {
        Self::new()
    }
}

impl<R: SecureRngProvider> KeyManagerImpl<R> {
    fn derived_pubkey_cache_capacity() -> NonZeroUsize {
        NonZeroUsize::new(MAX_DERIVED_PUBKEY_CACHE).unwrap_or(NonZeroUsize::MIN)
    }

    /// Create a key manager with a custom RNG provider.
    ///
    /// This is primarily intended for deterministic tests.
    pub fn new_with_rng(rng_provider: R) -> Self {
        Self::new_with_observability(
            Arc::new(NoopLogger),
            Arc::new(NoopMetrics),
            Arc::new(SystemTimeProvider),
            rng_provider,
        )
    }

    /// Create a key manager with custom observability backends.
    pub fn new_with_observability(
        logger: Arc<dyn Logger>,
        metrics: Arc<dyn MetricsSink>,
        time_provider: Arc<dyn TimeProvider>,
        rng_provider: R,
    ) -> Self {
        Self {
            encrypted_seed: None,
            bip44_manager: None,
            derived_public_keys: RwLock::new(LruCache::new(Self::derived_pubkey_cache_capacity())),
            deriving_paths: Mutex::new(HashMap::new()),
            derivation_count: AtomicUsize::new(0),
            chain: ChainType::Devnet,
            logger,
            metrics,
            time_provider,
            rng_provider,
            gap_external: AtomicU32::new(0),
            gap_internal: AtomicU32::new(0),
            last_used_ext: AtomicU32::new(0),
            last_used_int: AtomicU32::new(0),
            derive_count: AtomicU32::new(0),
        }
    }
}

include!("manager_impl_system.rs");
include!("manager_impl_gap.rs");
include!("manager_impl_cache.rs");
include!("manager_impl_cache_validation.rs");
include!("manager_impl_state.rs");
include!("manager_impl_trait.rs");

#[cfg(test)]
mod tests {
    use super::*;

    use crate::key::seed::CipherSeedContainer;
    use crate::key::{reset_seed_zeroized, seed_zeroized};
    use std::time::Duration;
    use z00z_crypto::expert::encoding::SafePassword;
    use z00z_utils::rng::{MockRngProvider, RngCoreExt};
    use z00z_utils::time::MockTimeProvider;

    fn valid_seed_bytes() -> [u8; 64] {
        let provider = MockRngProvider::with_u64_seed(3_456_789);
        let mut rng = provider.rng();
        let mut seed = [0u8; 64];
        rng.fill_bytes_ext(&mut seed);
        seed
    }

    include!("test_manager_impl_suite.rs");
    include!("test_manager_password_suite.rs");
}
