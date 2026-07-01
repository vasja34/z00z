#![cfg(not(target_arch = "wasm32"))]
//! Phase 14 integration tests for address derivation rate limiting.
//!
//! These tests focus on DoS behavior:
//! - After burst is exhausted, further derivations are rejected cheaply.
//! - Expensive key derivation is not invoked on rejected requests.

use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc,
};
use std::time::{Duration, SystemTime};

use rand::RngCore;
use z00z_core::genesis::ChainType;
use z00z_crypto::expert::keys::{RistrettoPublicKey, RistrettoSecretKey};
use z00z_crypto::KernelSignature;
use z00z_utils::{rng::SystemRngProvider, time::MockTimeProvider};
use zeroize::Zeroizing;

use z00z_wallets::{
    key::{Bip44Path, KeyManager, KeyManagerImpl, Result},
    receiver::{ReceiverManager, ReceiverManagerError, ReceiverManagerImpl},
};

fn test_seed_bytes() -> [u8; 64] {
    let mut inner = SystemRngProvider.rng();

    let mut bytes = [0u8; 64];
    inner.fill_bytes(&mut bytes);
    bytes
}

#[derive(Debug)]
struct CountingKeyManager {
    inner: KeyManagerImpl<SystemRngProvider>,
    derive_calls: Arc<AtomicUsize>,
}

impl CountingKeyManager {
    fn new(inner: KeyManagerImpl<SystemRngProvider>, derive_calls: Arc<AtomicUsize>) -> Self {
        Self {
            inner,
            derive_calls,
        }
    }
}

impl KeyManager for CountingKeyManager {
    fn clear(&mut self) {
        self.inner.clear();
    }

    fn derive_key(&self, path: &Bip44Path) -> Result<RistrettoPublicKey> {
        self.derive_calls.fetch_add(1, Ordering::Relaxed);
        self.inner.derive_key(path)
    }

    fn get_public_key(&self, path: &Bip44Path) -> Option<RistrettoPublicKey> {
        self.inner.get_public_key(path)
    }

    fn derive_secret_transient(&self, path: &Bip44Path) -> Result<Zeroizing<RistrettoSecretKey>> {
        self.inner.derive_secret_transient(path)
    }

    fn sign(&self, path: &Bip44Path, msg: &[u8]) -> Result<KernelSignature> {
        self.inner.sign(path, msg)
    }
}

fn new_counting_mgr(
    time: MockTimeProvider,
    rate_per_sec: u32,
    burst: u32,
) -> (
    ReceiverManagerImpl<CountingKeyManager, MockTimeProvider>,
    Arc<AtomicUsize>,
) {
    let mut key_manager = KeyManagerImpl::new_with_rng(SystemRngProvider);
    let seed_bytes = test_seed_bytes();
    key_manager
        .init_from_seed(&seed_bytes, ChainType::Devnet)
        .expect("seed init must succeed");

    let derive_calls = Arc::new(AtomicUsize::new(0));
    let counting = CountingKeyManager::new(key_manager, Arc::clone(&derive_calls));

    let mgr = ReceiverManagerImpl::new(counting)
        .with_rate_limit(rate_per_sec, burst)
        .with_time_provider(time)
        .build()
        .expect("manager build must succeed");

    (mgr, derive_calls)
}

#[test]
fn test_rate_limit_expensive_work() {
    let time = MockTimeProvider::new(SystemTime::UNIX_EPOCH);
    let (mut mgr, derive_calls) = new_counting_mgr(time, 1, 5);

    const DOS_REQUESTS: u32 = 10_000;

    for i in 0..5u32 {
        mgr.derive_wallet_keys(Bip44Path::payment(i).unwrap())
            .expect("within burst must succeed");
    }

    let calls_after_burst = derive_calls.load(Ordering::Relaxed);
    assert_eq!(calls_after_burst, 10, "each success derives spend+view");

    for i in 5..DOS_REQUESTS {
        let err = mgr
            .derive_wallet_keys(Bip44Path::payment(i).unwrap())
            .unwrap_err();
        assert!(matches!(err, ReceiverManagerError::RateLimitExceeded));
    }

    let calls_after_rejects = derive_calls.load(Ordering::Relaxed);
    assert_eq!(
        calls_after_rejects, calls_after_burst,
        "rate-limited requests must not call derive_key"
    );
}

#[test]
fn test_allows_legitimate_sustained_usage() {
    let time = MockTimeProvider::new(SystemTime::UNIX_EPOCH);
    let (mut mgr, _derive_calls) = new_counting_mgr(time.clone(), 100, 100);

    for i in 0..300u32 {
        mgr.derive_wallet_keys(Bip44Path::payment(i).unwrap())
            .expect("sustained usage must succeed");
        time.advance_by(Duration::from_millis(10));
    }
}

#[test]
fn test_limit_burst_refill_succeeds() {
    let time = MockTimeProvider::new(SystemTime::UNIX_EPOCH);
    let (mut mgr, _derive_calls) = new_counting_mgr(time.clone(), 10, 20);

    for i in 0..20u32 {
        mgr.derive_wallet_keys(Bip44Path::payment(i).unwrap())
            .expect("burst must succeed");
    }

    let err = mgr
        .derive_wallet_keys(Bip44Path::payment(21).unwrap())
        .unwrap_err();
    assert!(matches!(err, ReceiverManagerError::RateLimitExceeded));

    time.advance_by(Duration::from_secs(2));

    for i in 30..40u32 {
        mgr.derive_wallet_keys(Bip44Path::payment(i).unwrap())
            .expect("refilled tokens must allow derivations");
    }
}
