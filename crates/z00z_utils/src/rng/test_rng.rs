//! RNG provider trait tests

use crate::rng::{DeterministicRngSource, MockRngProvider, SecureRngProvider, SystemRngProvider};
use rand::RngCore;
use std::sync::Arc;
use std::thread;

#[test]
fn test_system_rng_provider_generates() {
    let provider = SystemRngProvider;
    let mut rng = provider.rng();

    let val1 = rng.next_u32();
    let mut rng2 = provider.rng();
    let val2 = rng2.next_u32();

    // Very unlikely to be equal
    assert_ne!(val1, val2);
}
#[test]
fn test_secure_rng_provider_trait() {
    fn test_with_provider<P: SecureRngProvider>(provider: P) {
        let mut rng = provider.rng();
        let _ = rng.next_u32(); // Should work
    }

    test_with_provider(SystemRngProvider);
}

#[test]
fn test_deterministic_rng_provider_trait() {
    fn test_with_provider<P: DeterministicRngSource>(provider: P) {
        let mut rng = provider.rng();
        let _ = rng.next_u32(); // Should work
    }

    test_with_provider(MockRngProvider::with_u64_seed(42));
}

#[test]
fn test_mock_rng_provider_thread() {
    let provider = Arc::new(MockRngProvider::with_u64_seed(42));
    let handles: Vec<_> = (0..5)
        .map(|_| {
            let provider = Arc::clone(&provider);
            thread::spawn(move || {
                let mut rng = provider.rng();
                rng.next_u32()
            })
        })
        .collect();

    let mut values = Vec::new();
    for handle in handles {
        values.push(handle.join().unwrap());
    }

    // All should be equal (same seed)
    assert!(values.iter().all(|&v| v == values[0]));
}

#[test]
fn test_rng_provider_send_sync() {
    fn test_assert_send_sync<T: Send + Sync>() {}
    test_assert_send_sync::<SystemRngProvider>();
    test_assert_send_sync::<MockRngProvider>();
}
