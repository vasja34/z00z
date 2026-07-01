use super::*;
use rand::SeedableRng;
use std::collections::BTreeSet;
use std::time::Duration;
use z00z_utils::time::SystemTimeProvider;

fn test_time() -> SystemTimeProvider {
    SystemTimeProvider
}

#[test]
fn test_nonce_counter_creation() {
    let counter = NonceCounter::new();
    assert_eq!(counter.value(), 0);
    assert_eq!(counter.last_updated(), 0);
}

#[test]
fn test_nonce_counter_increment() {
    let mut counter = NonceCounter::new();

    let val1 = counter.increment_unsafe(&test_time()).unwrap();
    assert_eq!(val1, 1);
    assert_eq!(counter.value(), 1);

    let val2 = counter.increment_unsafe(&test_time()).unwrap();
    assert_eq!(val2, 2);
    assert_eq!(counter.value(), 2);
}

#[test]
fn test_nonce_counter_overflow() {
    let mut counter = NonceCounter::new();
    counter.value = u64::MAX;

    let result = counter.increment_unsafe(&test_time());
    assert!(result.is_err());
}

#[test]
fn test_nonce_counter_recovery() {
    let mut counter = NonceCounter::new();

    counter
        .set_value_recovery(100, &test_time())
        .expect("recovery should succeed");
    assert_eq!(counter.value(), 100);

    // Can increment from recovered value
    let val = counter.increment_unsafe(&test_time()).unwrap();
    assert_eq!(val, 101);
}

#[test]
fn test_nonce_counter_recovery_prevents() {
    let mut counter = NonceCounter::new();
    counter.value = 100;

    // Should return error - cannot go backwards
    let result = counter.set_value_recovery(50, &test_time());
    assert!(
        result.is_err(),
        "Should error when trying to regress counter"
    );
    assert!(result
        .unwrap_err()
        .to_string()
        .contains("Cannot set counter to lower value"));
}

#[test]
fn test_try_get_before_epoch() {
    let mock = z00z_utils::time::MockTimeProvider::before_unix_secs(1);

    let result = try_get_timestamp_micros(&mock);
    assert!(result.is_err());
}

#[test]
fn test_try_derive_before_epoch() {
    let seed = [42u8; 32];
    let mock = z00z_utils::time::MockTimeProvider::before_unix_secs(1);

    let result = try_derive_nonce_simple(&seed, 1, &mock);
    assert!(result.is_err());
}

#[test]
fn test_try_nonce_minimal_epoch() {
    let mock = z00z_utils::time::MockTimeProvider::before_unix_secs(1);
    let mut rng = rand::rngs::StdRng::seed_from_u64(7);

    let result = try_derive_nonce_minimal(&mut rng, &mock);
    assert!(result.is_err());
}

#[test]
fn test_increment_unsafe_fails_closed() {
    let mut counter = NonceCounter::new();
    let mock = z00z_utils::time::MockTimeProvider::before_unix_secs(1);

    let result = counter.increment_unsafe(&mock);

    assert!(result.is_err());
    assert_eq!(counter.value(), 0);
    assert_eq!(counter.last_updated(), 0);
}

#[test]
fn test_set_value_fails_closed() {
    let mut counter = NonceCounter::new();
    let mock = z00z_utils::time::MockTimeProvider::before_unix_secs(1);

    let result = counter.set_value_recovery(10, &mock);

    assert!(result.is_err());
    assert_eq!(counter.value(), 0);
    assert_eq!(counter.last_updated(), 0);
}

#[test]
fn test_derive_nonce_deterministic() {
    let seed = [42u8; 32];
    let counter = 1;
    let timestamp = 1234567890;
    let prev_hash = [0u8; 32];

    let nonce1 = derive_nonce(&seed, counter, timestamp, &prev_hash);
    let nonce2 = derive_nonce(&seed, counter, timestamp, &prev_hash);

    assert_eq!(nonce1, nonce2);
}

#[test]
fn test_derive_nonce_unique_per() {
    let seed = [42u8; 32];
    let timestamp = 1234567890;
    let prev_hash = [0u8; 32];

    let nonce1 = derive_nonce(&seed, 1, timestamp, &prev_hash);
    let nonce2 = derive_nonce(&seed, 2, timestamp, &prev_hash);

    assert_ne!(nonce1, nonce2);
}

#[test]
fn test_derive_nonce_per_seed() {
    let seed1 = [42u8; 32];
    let seed2 = [43u8; 32];
    let counter = 1;
    let timestamp = 1234567890;
    let prev_hash = [0u8; 32];

    let nonce1 = derive_nonce(&seed1, counter, timestamp, &prev_hash);
    let nonce2 = derive_nonce(&seed2, counter, timestamp, &prev_hash);

    assert_ne!(nonce1, nonce2);
}

#[test]
fn test_derive_nonce_per_timestamp() {
    let seed = [42u8; 32];
    let counter = 1;
    let prev_hash = [0u8; 32];

    let nonce1 = derive_nonce(&seed, counter, 1000, &prev_hash);
    let nonce2 = derive_nonce(&seed, counter, 2000, &prev_hash);

    assert_ne!(nonce1, nonce2);
}

#[test]
fn test_derive_nonce_prev_hash() {
    let seed = [42u8; 32];
    let counter = 1;
    let timestamp = 1234567890;

    let prev_hash1 = [0u8; 32];
    let prev_hash2 = [1u8; 32];

    let nonce1 = derive_nonce(&seed, counter, timestamp, &prev_hash1);
    let nonce2 = derive_nonce(&seed, counter, timestamp, &prev_hash2);

    assert_ne!(nonce1, nonce2);
}

#[test]
fn test_nonce_uniqueness_large_scale() {
    let time = SystemTimeProvider;
    let seed = [42u8; 32];
    let prev_hash = [0u8; 32];
    let mut seen_nonces = BTreeSet::new();

    // Generate 10,000 nonces with different counters
    for counter in 0..10_000 {
        let timestamp = get_timestamp_micros(&time).expect("timestamp");
        let nonce = derive_nonce(&seed, counter, timestamp, &prev_hash);

        // Must be unique
        assert!(
            seen_nonces.insert(nonce),
            "Nonce collision at counter={}",
            counter
        );
    }

    assert_eq!(seen_nonces.len(), 10_000);
}

#[test]
fn test_wallet_recovery_reproduces_nonces() {
    let seed = [42u8; 32];
    let timestamp = 1234567890;
    let prev_hash = [0u8; 32];

    // Generate nonces first time
    let nonces_1: Vec<[u8; 32]> = (0..100)
        .map(|counter| derive_nonce(&seed, counter, timestamp, &prev_hash))
        .collect();

    // Simulate wallet recovery - generate again
    let nonces_2: Vec<[u8; 32]> = (0..100)
        .map(|counter| derive_nonce(&seed, counter, timestamp, &prev_hash))
        .collect();

    // Must be identical
    assert_eq!(nonces_1, nonces_2);
}

#[test]
fn test_get_timestamp_micros() {
    let time = SystemTimeProvider;
    let ts1 = get_timestamp_micros(&time).expect("timestamp");
    assert!(ts1 > 0);

    // Should be increasing - use longer sleep for reliability
    std::thread::sleep(Duration::from_millis(2));
    let ts2 = get_timestamp_micros(&time).expect("timestamp");
    assert!(ts2 > ts1);
}

#[test]
fn test_nonce_counter_serialization() {
    use z00z_utils::codec::{Codec, JsonCodec};

    let counter = NonceCounter {
        value: 12345,
        last_updated: 1234567890,
    };

    // Serialize
    let codec = JsonCodec;
    let json = codec.serialize(&counter).unwrap();

    // Deserialize
    let counter2: NonceCounter = codec.deserialize(&json).unwrap();

    assert_eq!(counter, counter2);
}

#[test]
fn test_derive_genesis_nonce_deterministic() {
    let genesis_seed = [0xABu8; 32];
    let definition_id = [0x01u8; 32];
    let serial_id = 100;

    // Same inputs must produce same nonce
    let nonce1 = derive_genesis_nonce(&genesis_seed, &definition_id, serial_id);
    let nonce2 = derive_genesis_nonce(&genesis_seed, &definition_id, serial_id);
    assert_eq!(nonce1, nonce2);
}

#[test]
fn test_derive_genesis_nonce_unique() {
    let genesis_seed = [0xABu8; 32];
    let definition_id = [0x01u8; 32];

    // Different serial_id must produce different nonces
    let nonce1 = derive_genesis_nonce(&genesis_seed, &definition_id, 100);
    let nonce2 = derive_genesis_nonce(&genesis_seed, &definition_id, 101);
    assert_ne!(nonce1, nonce2);
}

#[test]
fn test_derive_genesis_per_definition() {
    let genesis_seed = [0xABu8; 32];
    let definition_id1 = [0x01u8; 32];
    let definition_id2 = [0x02u8; 32];
    let serial_id = 100;

    // Different definition_id must produce different nonces
    let nonce1 = derive_genesis_nonce(&genesis_seed, &definition_id1, serial_id);
    let nonce2 = derive_genesis_nonce(&genesis_seed, &definition_id2, serial_id);
    assert_ne!(nonce1, nonce2);
}

#[test]
fn test_derive_genesis_per_seed() {
    let genesis_seed1 = [0xABu8; 32];
    let genesis_seed2 = [0xCDu8; 32];
    let definition_id = [0x01u8; 32];
    let serial_id = 100;

    // Different genesis_seed must produce different nonces
    let nonce1 = derive_genesis_nonce(&genesis_seed1, &definition_id, serial_id);
    let nonce2 = derive_genesis_nonce(&genesis_seed2, &definition_id, serial_id);
    assert_ne!(nonce1, nonce2);
}

#[test]
fn test_derive_genesis_nonce_coverage() {
    let genesis_seed = [0xABu8; 32];
    let definition_id = [0x01u8; 32];

    // Generate nonces for many serials - verify uniqueness
    let mut seen_nonces = BTreeSet::new();
    for serial_id in 0..1000 {
        let nonce = derive_genesis_nonce(&genesis_seed, &definition_id, serial_id);
        assert!(
            seen_nonces.insert(nonce),
            "Nonce collision at serial_id={}",
            serial_id
        );
    }
    assert_eq!(seen_nonces.len(), 1000);
}
