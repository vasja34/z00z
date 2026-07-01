use super::*;
use crate::key::{
    KeyManager, KeyManagerImpl, ReceiverKeys, ReceiverSecret, Z00Z_BIP44_ASSET,
};
use crate::stealth::{SenderWallet, build_tx_output_unchecked};
use std::io::{self, Write};
use std::sync::{Arc, Mutex};
use std::time::Duration;
use z00z_core::genesis::ChainType;
use z00z_crypto::expert::{
    encoding::ByteArray,
    keys::{RistrettoPublicKey, RistrettoSecretKey},
    traits::{PublicKeyTrait, SecretKeyTrait},
};
use z00z_utils::rng::{SecureRngProvider, SystemRngProvider};
use z00z_utils::time::MockTimeProvider;

const TEST_SEED_BYTES: [u8; 64] = [
    0x8f, 0x2b, 0x67, 0xd1, 0x0c, 0xa4, 0x39, 0xee, 0x51, 0x96, 0x73, 0x1d, 0xc8, 0x04, 0xfa, 0xb2,
    0x3e, 0x78, 0x9a, 0x2f, 0x10, 0xcd, 0x6b, 0x85, 0xf1, 0x4c, 0x22, 0x9d, 0x7e, 0x13, 0xa8, 0x5b,
    0x6e, 0x07, 0xbc, 0x44, 0xd8, 0x91, 0x2a, 0xf7, 0x30, 0x5d, 0x8c, 0x19, 0xe3, 0x62, 0xaf, 0x0b,
    0x95, 0x4f, 0x21, 0xda, 0x7c, 0x36, 0x88, 0xfe, 0x11, 0x6a, 0xc3, 0x55, 0x9e, 0x27, 0xb0, 0x4a,
];

// Helper to initialize key manager with BIP-39 seed (Phase 5: pure BIP-39 → BIP-32)
fn init_key_manager_with_seed<R: SecureRngProvider>(
    key_manager: &mut KeyManagerImpl<R>,
    seed_bytes: [u8; 64],
) -> std::result::Result<(), ReceiverManagerError> {
    // Phase 5: Pure BIP-39 → BIP-32 (no pre-KDF)
    // Pass the 64-byte BIP-39 seed output directly as BIP-32 seed input
    key_manager
        .init_from_seed(&seed_bytes, ChainType::Devnet)
        .map_err(|e| ReceiverManagerError::KeyDerivation(e.to_string()))
}

fn new_test_key_manager() -> KeyManagerImpl<SystemRngProvider> {
    KeyManagerImpl::new_with_rng(SystemRngProvider)
}

fn new_rate_limited_mgr(
    time: MockTimeProvider,
    rate_per_sec: u32,
    burst: u32,
) -> ReceiverManagerImpl<KeyManagerImpl<SystemRngProvider>, MockTimeProvider> {
    let mut key_manager = new_test_key_manager();
    init_key_manager_with_seed(&mut key_manager, TEST_SEED_BYTES).unwrap();

    ReceiverManagerImpl::new(key_manager)
        .with_rate_limit(rate_per_sec, burst)
        .with_time_provider(time)
        .build()
        .unwrap()
}

#[test]
fn test_rate_limit_burst_exhaustion() {
    let time = MockTimeProvider::default();
    let mut mgr = new_rate_limited_mgr(time.clone(), 1, 2);

    mgr.derive_wallet_keys(Bip44Path::payment(0).unwrap())
        .unwrap();
    mgr.derive_wallet_keys(Bip44Path::payment(1).unwrap())
        .unwrap();

    let err = mgr
        .derive_wallet_keys(Bip44Path::payment(2).unwrap())
        .unwrap_err();
    assert!(matches!(err, ReceiverManagerError::RateLimitExceeded));

    time.advance_by(Duration::from_secs(1));
    mgr.derive_wallet_keys(Bip44Path::payment(3).unwrap())
        .unwrap();
}

#[test]
fn test_limit_not_cache_hit() {
    let time = MockTimeProvider::default();
    let mut mgr = new_rate_limited_mgr(time, 1, 1);

    let path = Bip44Path::payment(0).unwrap();
    mgr.derive_wallet_keys(path).unwrap();
    mgr.derive_wallet_keys(path).unwrap();
}

#[test]
fn test_limit_batch_too_large() {
    let time = MockTimeProvider::default();
    let mut mgr = new_rate_limited_mgr(time, 100, 2);

    let paths = [
        Bip44Path::payment(0).unwrap(),
        Bip44Path::payment(1).unwrap(),
        Bip44Path::payment(2).unwrap(),
    ];

    let err = mgr.derive_batch(&paths).unwrap_err();
    assert!(matches!(
        err,
        ReceiverManagerError::BatchTooLarge {
            requested: 3,
            max_allowed: 2
        }
    ));
}

#[test]
fn test_limit_batch_is_atomic() {
    let time = MockTimeProvider::default();
    let mut mgr = new_rate_limited_mgr(time, 1, 2);

    let cached_path = Bip44Path::payment(0).unwrap();
    let miss_a = Bip44Path::payment(1).unwrap();
    let miss_b = Bip44Path::payment(2).unwrap();

    mgr.derive_wallet_keys(cached_path).unwrap();

    let err = mgr.derive_batch(&[miss_a, miss_b]).unwrap_err();
    assert!(matches!(err, ReceiverManagerError::RateLimitExceeded));
    assert_eq!(mgr.cache().unwrap().len(), 1);
    assert!(matches!(
        mgr.get_receiver_key(miss_a),
        Err(ReceiverManagerError::NotFound(_))
    ));
    assert!(matches!(
        mgr.get_receiver_key(miss_b),
        Err(ReceiverManagerError::NotFound(_))
    ));
}

#[test]
fn test_counts_timing_safe_paths() {
    let time = MockTimeProvider::default();
    let mut key_manager = new_test_key_manager();
    init_key_manager_with_seed(&mut key_manager, TEST_SEED_BYTES).unwrap();

    let mut mgr = ReceiverManagerImpl::new(key_manager)
        .with_rate_limit(1, 1)
        .with_timing_safe_mode(true)
        .with_time_provider(time.clone())
        .build()
        .unwrap();

    let cached_path = Bip44Path::payment(0).unwrap();
    mgr.derive_wallet_keys(cached_path).unwrap();

    time.advance_by(Duration::from_secs(1));

    let err = mgr.derive_batch(&[cached_path, cached_path]).unwrap_err();
    assert!(matches!(
        err,
        ReceiverManagerError::BatchTooLarge {
            requested: 2,
            max_allowed: 1,
        }
    ));
}

#[test]
fn test_limit_disabled_allows_many() {
    let time = MockTimeProvider::default();
    let mut key_manager = new_test_key_manager();
    init_key_manager_with_seed(&mut key_manager, TEST_SEED_BYTES).unwrap();

    let mut mgr = ReceiverManagerImpl::new(key_manager)
        .with_time_provider(time)
        .build()
        .unwrap();

    for i in 0..50u32 {
        mgr.derive_wallet_keys(Bip44Path::payment(i).unwrap())
            .unwrap();
    }
}

#[test]
fn test_seed_reproducible() {
    let mut key_manager1 = new_test_key_manager();
    let mut key_manager2 = new_test_key_manager();

    init_key_manager_with_seed(&mut key_manager1, TEST_SEED_BYTES).unwrap();
    init_key_manager_with_seed(&mut key_manager2, TEST_SEED_BYTES).unwrap();

    let path = Bip44Path::payment(0).unwrap();
    let pubkey1 = key_manager1.derive_key(&path).unwrap();
    let pubkey2 = key_manager2.derive_key(&path).unwrap();
    assert_eq!(pubkey1, pubkey2);
}

#[test]
fn test_seed_changes_keys() {
    let mut key_manager1 = new_test_key_manager();
    let mut key_manager2 = new_test_key_manager();

    let mut seed2 = TEST_SEED_BYTES;
    seed2[63] ^= 0x01;

    init_key_manager_with_seed(&mut key_manager1, TEST_SEED_BYTES).unwrap();
    init_key_manager_with_seed(&mut key_manager2, seed2).unwrap();

    let path = Bip44Path::payment(0).unwrap();
    let pubkey1 = key_manager1.derive_key(&path).unwrap();
    let pubkey2 = key_manager2.derive_key(&path).unwrap();
    assert_ne!(pubkey1, pubkey2);
}

#[test]
fn test_metrics_increment_works() {
    let metrics = CacheMetrics::default();
    metrics.inc_hits();
    metrics.inc_misses();
    metrics.inc_evictions();
    metrics.inc_total_derivations();
    metrics.inc_ttl_expirations();
    metrics.inc_total_lookups();
    metrics.inc_lookup_hits();
    metrics.inc_lookup_misses();
    metrics.inc_purge_runs();
    metrics.add_purge_entries(2);
    metrics.add_purge_bytes(128);

    assert_eq!(metrics.hits.load(Ordering::Relaxed), 1);
    assert_eq!(metrics.misses.load(Ordering::Relaxed), 1);
    assert_eq!(metrics.evictions.load(Ordering::Relaxed), 1);
    assert_eq!(metrics.total_derivations.load(Ordering::Relaxed), 1);
    assert_eq!(metrics.ttl_expirations.load(Ordering::Relaxed), 1);
    assert_eq!(metrics.total_lookups.load(Ordering::Relaxed), 1);
    assert_eq!(metrics.lookup_hits.load(Ordering::Relaxed), 1);
    assert_eq!(metrics.lookup_misses.load(Ordering::Relaxed), 1);
    assert_eq!(metrics.purge_runs.load(Ordering::Relaxed), 1);
    assert_eq!(metrics.purge_entries.load(Ordering::Relaxed), 2);
    assert_eq!(metrics.purge_bytes.load(Ordering::Relaxed), 128);
}

#[test]
fn test_metrics_saturates_at_max() {
    let metrics = CacheMetrics::default();
    metrics.hits.store(u64::MAX, Ordering::Relaxed);
    metrics.inc_hits();
    assert_eq!(metrics.hits.load(Ordering::Relaxed), u64::MAX);
}

#[test]
fn test_metrics_multiple_adds_saturate() {
    let metrics = CacheMetrics::default();
    metrics.hits.store(u64::MAX - 1, Ordering::Relaxed);
    metrics.inc_hits();
    metrics.inc_hits();
    assert_eq!(metrics.hits.load(Ordering::Relaxed), u64::MAX);
}

#[test]
fn test_metrics_warns_near_saturation() {
    struct BufWriter(Arc<Mutex<Vec<u8>>>);

    struct BufGuard(Arc<Mutex<Vec<u8>>>);

    impl Write for BufGuard {
        fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
            self.0.lock().unwrap().extend_from_slice(buf);
            Ok(buf.len())
        }

        fn flush(&mut self) -> io::Result<()> {
            Ok(())
        }
    }

    impl<'a> tracing_subscriber::fmt::MakeWriter<'a> for BufWriter {
        type Writer = BufGuard;

        fn make_writer(&'a self) -> Self::Writer {
            BufGuard(self.0.clone())
        }
    }

    let buf = Arc::new(Mutex::new(Vec::new()));
    let writer = BufWriter(buf.clone());

    let subscriber = tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::new("warn"))
        .with_writer(writer)
        .with_target(false)
        .without_time()
        .finish();

    let _guard = tracing::subscriber::set_default(subscriber);

    let metrics = CacheMetrics::default();
    metrics
        .hits
        .store(METRIC_SAT_THRESHOLD.saturating_sub(1), Ordering::Relaxed);
    metrics.inc_hits();
    metrics.inc_hits();

    let out = String::from_utf8(buf.lock().unwrap().clone()).unwrap();
    assert_eq!(out.matches("approaching saturation").count(), 1);
    assert!(out.contains("hits"));
}

#[test]
fn test_metrics_near_saturation() {
    let mut snap = CacheMetricsSnapshot::default();
    assert!(!snap.is_near_saturation());

    snap.hits = u64::MAX;
    assert!(snap.is_near_saturation());
}

#[test]
fn test_derive_address_creates_caches() {
    let mut key_manager = new_test_key_manager();

    // Initialize key manager with deterministic seed
    init_key_manager_with_seed(&mut key_manager, TEST_SEED_BYTES).unwrap();

    let mut receiver_manager = ReceiverManagerImpl::new(key_manager).build().unwrap();

    let path = Bip44Path::payment(0).unwrap();
    let pubkey1 = receiver_manager.derive_spend_key(path).unwrap();

    // Second call should return cached value
    let pubkey2 = receiver_manager.derive_spend_key(path).unwrap();
    assert_eq!(pubkey1, pubkey2);

    // Cache should contain the address
    assert_eq!(receiver_manager.cache().unwrap().len(), 1);
}

#[test]
fn test_derive_wallet_keys() {
    let mut key_manager = new_test_key_manager();

    init_key_manager_with_seed(&mut key_manager, TEST_SEED_BYTES).unwrap();

    let mut receiver_manager = ReceiverManagerImpl::new(key_manager).build().unwrap();

    let path = Bip44Path::payment(0).unwrap();
    let keys1 = receiver_manager.derive_wallet_keys(path).unwrap();
    assert_ne!(keys1.spend_key, keys1.view_key);

    let keys2 = receiver_manager.derive_wallet_keys(path).unwrap();
    assert_eq!(keys1.spend_key, keys2.spend_key);
    assert_eq!(keys1.view_key, keys2.view_key);

    let spend = receiver_manager.get_receiver_key(path).unwrap();
    assert_eq!(spend, keys1.spend_key);
    assert_eq!(receiver_manager.cache().unwrap().len(), 1);
}

#[test]
fn test_receiver_manager_create_card() {
    let key_manager = new_test_key_manager();
    let receiver_manager = ReceiverManagerImpl::new(key_manager).build().unwrap();

    let receiver_secret = ReceiverSecret::generate().expect("receiver secret");
    let receiver_keys = ReceiverKeys::from_receiver_secret(receiver_secret).expect("receiver keys");

    let card = receiver_manager
        .create_receiver_card(&receiver_keys)
        .unwrap();
    let mut view_pk = [0u8; 32];
    view_pk.copy_from_slice(receiver_keys.view_pk.as_bytes());
    let mut identity_pk = [0u8; 32];
    identity_pk.copy_from_slice(receiver_keys.identity_pk.as_bytes());

    assert_eq!(card.owner_handle, receiver_keys.owner_handle);
    assert_eq!(card.view_pk, view_pk);
    assert_eq!(card.identity_pk, identity_pk);
    card.verify().expect("card verify");
}

#[test]
fn test_receiver_manager_generate_request() {
    let key_manager = new_test_key_manager();
    let receiver_manager = ReceiverManagerImpl::new(key_manager).build().unwrap();

    let receiver_secret = ReceiverSecret::generate().expect("receiver secret");
    let receiver_keys = ReceiverKeys::from_receiver_secret(receiver_secret).expect("receiver keys");

    let params = RequestParams {
        amount: Some(123),
        expiry_seconds: 120,
        memo: Some("invoice".to_string()),
        payment_id: None,
    };

    let request = receiver_manager
        .generate_payment_request(&receiver_keys, params, 1)
        .unwrap();

    assert_eq!(request.owner_handle, receiver_keys.owner_handle);
    assert_eq!(request.chain_id, 1);
    assert_eq!(request.amount, Some(123));
}

#[test]
fn test_receiver_manager_scan_integration() {
    let key_manager = new_test_key_manager();
    let receiver_manager = ReceiverManagerImpl::new(key_manager).build().unwrap();

    let (receiver_keys, leaf) = make_scan_leaf(456);
    let found = receiver_manager
        .scan_checkpoint(&receiver_keys, &[leaf])
        .unwrap();

    assert_eq!(found.len(), 1);
    assert_eq!(found[0].amount, 456);
}

#[test]
fn test_receiver_manager_scan_requests() {
    let key_manager = new_test_key_manager();
    let receiver_manager = ReceiverManagerImpl::new(key_manager).build().unwrap();

    let receiver_secret = ReceiverSecret::generate().expect("receiver secret");
    let receiver_keys = ReceiverKeys::from_receiver_secret(receiver_secret).expect("receiver keys");

    let card = receiver_manager
        .create_receiver_card(&receiver_keys)
        .expect("card");

    let request = receiver_manager
        .generate_payment_request(
            &receiver_keys,
            RequestParams {
                amount: Some(222),
                expiry_seconds: u64::MAX,
                memo: None,
                payment_id: None,
            },
            1,
        )
        .expect("request");

    let mut asset_id = [0u8; 32];
    asset_id[0] = 4;
    let mut sender_wallet = SenderWallet::new([11u8; 32]);
    let output = build_tx_output_unchecked(
        &card,
        Some(&request),
        &mut sender_wallet,
        &[12u8; 32],
        1,
        222,
        &asset_id,
    )
    .expect("output");

    let mut leaf =
        z00z_core::genesis::asset_std::asset_from_dev_cfg("z00z", 0, 222).expect("std asset");
    leaf.commitment = z00z_crypto::Commitment::from_bytes(&output.c_amount)
        .expect("commitment")
        .0;

    leaf.r_pub = Some(output.r_pub);
    leaf.owner_tag = Some(output.owner_tag);
    leaf.enc_pack = Some(output.enc_pack);
    leaf.tag16 = output.tag16;
    leaf.leaf_ad_id = Some(asset_id);

    let without_req = receiver_manager
        .scan_checkpoint(&receiver_keys, &[leaf.clone()])
        .unwrap();
    assert!(without_req.is_empty());

    let with_req = receiver_manager
        .scan_checkpoint_with_requests(&receiver_keys, &[leaf], &[request])
        .unwrap();
    assert_eq!(with_req.len(), 1);
    assert_eq!(with_req[0].amount, 222);
}

fn make_scan_leaf(amount: u64) -> (ReceiverKeys, Asset) {
    let receiver_secret = ReceiverSecret::generate().expect("receiver secret");
    let receiver_keys = ReceiverKeys::from_receiver_secret(receiver_secret).expect("receiver keys");

    let card = ReceiverCard {
        version: 1,
        owner_handle: receiver_keys.owner_handle,
        view_pk: receiver_keys
            .view_pk
            .as_bytes()
            .try_into()
            .expect("view pk"),
        identity_pk: receiver_keys
            .identity_pk
            .as_bytes()
            .try_into()
            .expect("identity pk"),
        card_id: None,
        metadata: None,
        signature: [0u8; 64],
    };

    let mut asset_id = [0u8; 32];
    asset_id[0] = 3;
    let mut sender_wallet = SenderWallet::new([7u8; 32]);
    let output = build_tx_output_unchecked(
        &card,
        None,
        &mut sender_wallet,
        &[8u8; 32],
        0,
        amount,
        &asset_id,
    )
    .expect("output");

    let mut leaf =
        z00z_core::genesis::asset_std::asset_from_dev_cfg("z00z", 0, amount).expect("std asset");
    leaf.commitment = z00z_crypto::Commitment::from_bytes(&output.c_amount)
        .expect("commitment")
        .0;

    leaf.r_pub = Some(output.r_pub);
    leaf.owner_tag = Some(output.owner_tag);
    leaf.enc_pack = Some(output.enc_pack);
    leaf.tag16 = output.tag16;
    leaf.leaf_ad_id = Some(asset_id);

    (receiver_keys, leaf)
}

#[test]
fn test_get_address_returns_cached() {
    let mut key_manager = new_test_key_manager();

    // Initialize key manager
    init_key_manager_with_seed(&mut key_manager, TEST_SEED_BYTES).unwrap();

    let mut receiver_manager = ReceiverManagerImpl::new(key_manager).build().unwrap();

    let path = Bip44Path::payment(1).unwrap();
    let pubkey = receiver_manager.derive_spend_key(path).unwrap();

    let retrieved = receiver_manager.get_receiver_key(path).unwrap();
    assert_eq!(retrieved, pubkey);
}

#[test]
fn test_get_address_not_found() {
    let key_manager = new_test_key_manager();

    let mut receiver_manager = ReceiverManagerImpl::new(key_manager).build().unwrap();

    let path = Bip44Path::payment(99).unwrap();
    let result = receiver_manager.get_receiver_key(path);
    assert!(matches!(result, Err(ReceiverManagerError::NotFound(_))));
}

#[test]
fn test_list_receivers_returns_all() {
    let mut key_manager = new_test_key_manager();

    // Initialize key manager
    init_key_manager_with_seed(&mut key_manager, TEST_SEED_BYTES).unwrap();

    let mut receiver_manager = ReceiverManagerImpl::new(key_manager).build().unwrap();

    let path1 = Bip44Path::payment(0).unwrap();
    let path2 = Bip44Path::payment(1).unwrap();
    let path3 = Bip44Path::change_path(0).unwrap();

    receiver_manager.derive_spend_key(path1).unwrap();
    receiver_manager.derive_spend_key(path2).unwrap();
    receiver_manager.derive_spend_key(path3).unwrap();

    let receivers = receiver_manager.list_receivers().unwrap();
    assert_eq!(receivers.len(), 3);
}

#[test]
fn test_clear_cache_empties_cache() {
    let mut key_manager = new_test_key_manager();

    // Initialize key manager
    init_key_manager_with_seed(&mut key_manager, TEST_SEED_BYTES).unwrap();

    let mut receiver_manager = ReceiverManagerImpl::new(key_manager).build().unwrap();

    let path = Bip44Path::payment(0).unwrap();
    receiver_manager.derive_spend_key(path).unwrap();
    assert_eq!(receiver_manager.cache().unwrap().len(), 1);

    receiver_manager.clear_cache().unwrap();
    assert_eq!(receiver_manager.cache().unwrap().len(), 0);

    // Should be able to rederive after clear
    receiver_manager.derive_spend_key(path).unwrap();
    assert_eq!(receiver_manager.cache().unwrap().len(), 1);
}

#[test]
fn test_clear_all_empties_both() {
    let mut key_manager = new_test_key_manager();

    // Initialize key manager with deterministic seed
    init_key_manager_with_seed(&mut key_manager, TEST_SEED_BYTES).unwrap();

    let mut receiver_manager = ReceiverManagerImpl::new(key_manager).build().unwrap();

    let path = Bip44Path::payment(0).unwrap();
    receiver_manager.derive_spend_key(path).unwrap();
    assert_eq!(receiver_manager.cache().unwrap().len(), 1);

    receiver_manager.clear_all().unwrap();
    assert_eq!(receiver_manager.cache().unwrap().len(), 0);
}

#[test]
fn test_cache_size_limit_enforced() {
    let mut key_manager = new_test_key_manager();
    init_key_manager_with_seed(&mut key_manager, TEST_SEED_BYTES).unwrap();

    // Set small cache limit using builder
    let mut receiver_manager = ReceiverManagerImpl::new(key_manager)
        .with_limit(5)
        .build()
        .unwrap();

    // Derive 10 receivers
    for i in 0..10 {
        let path = Bip44Path::payment(i).unwrap();
        receiver_manager.derive_spend_key(path).unwrap();
    }

    // Cache should be at max size (5)
    assert_eq!(receiver_manager.cache().unwrap().len(), 5);

    // Verify LRU: oldest entries (0-4) should be evicted
    let cached_paths: Vec<_> = receiver_manager
        .cache()
        .unwrap()
        .iter()
        .map(|(path, _)| *path)
        .collect();
    assert!(!cached_paths.contains(&Bip44Path::payment(0).unwrap()));
    assert!(!cached_paths.contains(&Bip44Path::payment(4).unwrap()));
    assert!(cached_paths.contains(&Bip44Path::payment(5).unwrap()));
    assert!(cached_paths.contains(&Bip44Path::payment(9).unwrap()));
}

#[derive(Debug, Default)]
struct TestEvictListener {
    events: Mutex<Vec<(Bip44Path, Z00ZRistrettoPoint)>>,
}

impl CacheEvictionListener for TestEvictListener {
    fn on_evict(&self, path: Bip44Path, key: &Z00ZRistrettoPoint) {
        let mut guard = self.events.lock().unwrap();
        guard.push((path, key.clone()));
    }
}

#[test]
fn test_evict_listener_called() {
    let mut key_manager = new_test_key_manager();
    init_key_manager_with_seed(&mut key_manager, TEST_SEED_BYTES).unwrap();

    let listener = Arc::new(TestEvictListener::default());
    let mut receiver_manager = ReceiverManagerImpl::new(key_manager)
        .with_limit(2)
        .with_eviction_listener(listener.clone())
        .build()
        .unwrap();

    let p0 = Bip44Path::payment(0).unwrap();
    let k0 = receiver_manager.derive_spend_key(p0).unwrap();
    let _k1 = receiver_manager
        .derive_spend_key(Bip44Path::payment(1).unwrap())
        .unwrap();

    // Trigger LRU eviction of p0.
    let _k2 = receiver_manager
        .derive_spend_key(Bip44Path::payment(2).unwrap())
        .unwrap();

    let events = listener.events.lock().unwrap().clone();
    assert_eq!(events.len(), 1);
    assert_eq!(events[0].0, p0);
    assert_eq!(events[0].1, k0);
}

#[cfg(not(target_arch = "wasm32"))]
#[test]
fn test_persistence_listener_is_gated() {
    let mut key_manager = new_test_key_manager();
    init_key_manager_with_seed(&mut key_manager, TEST_SEED_BYTES).unwrap();

    let dir = tempfile::tempdir().unwrap();
    let log_path = dir.path().join("evictions.jsonl");

    let res = PersistenceEvictionListener::new(&log_path);
    if cfg!(all(debug_assertions, feature = "eviction-logs")) {
        let listener = Arc::new(res.unwrap());
        let mut receiver_manager = ReceiverManagerImpl::new(key_manager)
            .with_limit(2)
            .with_eviction_listener(listener)
            .build()
            .unwrap();

        // Trigger one eviction.
        let _k0 = receiver_manager
            .derive_spend_key(Bip44Path::payment(0).unwrap())
            .unwrap();
        let _k1 = receiver_manager
            .derive_spend_key(Bip44Path::payment(1).unwrap())
            .unwrap();
        let _k2 = receiver_manager
            .derive_spend_key(Bip44Path::payment(2).unwrap())
            .unwrap();

        let mut contents = String::new();
        for _ in 0..100 {
            if let Ok(s) = z00z_utils::io::read_to_string(&log_path) {
                if s.contains("\"path\"") && s.contains("\"spend_key\"") {
                    contents = s;
                    break;
                }
            }
            std::thread::sleep(Duration::from_millis(10));
        }

        assert!(contents.contains("\"path\""));
        assert!(contents.contains("\"spend_key\""));
    } else {
        let err = res.unwrap_err();
        assert_eq!(err.kind(), std::io::ErrorKind::Unsupported);
    }
}

#[derive(Debug, Default)]
struct PanicEvictListener;

impl CacheEvictionListener for PanicEvictListener {
    fn on_evict(&self, _path: Bip44Path, _key: &Z00ZRistrettoPoint) {
        panic!("intentional panic");
    }
}

#[test]
fn test_evict_listener_panic_ok() {
    let mut key_manager = new_test_key_manager();
    init_key_manager_with_seed(&mut key_manager, TEST_SEED_BYTES).unwrap();

    let listener = Arc::new(PanicEvictListener);
    let mut receiver_manager = ReceiverManagerImpl::new(key_manager)
        .with_limit(2)
        .with_eviction_listener(listener)
        .build()
        .unwrap();

    // Trigger an eviction; listener panics but eviction must not fail.
    let _k0 = receiver_manager
        .derive_spend_key(Bip44Path::payment(0).unwrap())
        .unwrap();
    let _k1 = receiver_manager
        .derive_spend_key(Bip44Path::payment(1).unwrap())
        .unwrap();
    let _k2 = receiver_manager
        .derive_spend_key(Bip44Path::payment(2).unwrap())
        .unwrap();

    assert_eq!(receiver_manager.cache().unwrap().len(), 2);
}

#[test]
fn test_metrics_tracking() {
    let mut key_manager = new_test_key_manager();
    init_key_manager_with_seed(&mut key_manager, TEST_SEED_BYTES).unwrap();

    let mut receiver_manager = ReceiverManagerImpl::new(key_manager).build().unwrap();

    // First derivation - miss
    let path1 = Bip44Path::payment(0).unwrap();
    receiver_manager.derive_spend_key(path1).unwrap();

    // Second call - hit
    receiver_manager.derive_spend_key(path1).unwrap();

    // Non-existent path - miss (via get_receiver_key)
    let path2 = Bip44Path::payment(99).unwrap();
    let _ = receiver_manager.get_receiver_key(path2);

    let metrics = receiver_manager.get_metrics();
    assert_eq!(metrics.total_derivations, 2); // Only counts derive_spend_key calls
    assert_eq!(metrics.hits, 1);
    assert_eq!(metrics.misses, 1); // Only from first derive

    // Test hit rate
    let hit_rate = metrics.hit_rate();
    assert!((hit_rate - 0.5).abs() < 0.01); // 1/2 = 0.5

    // Reset metrics
    receiver_manager.reset_metrics();
    assert_eq!(receiver_manager.get_metrics().hits, 0);
}

#[test]
fn test_batch_derivation() {
    let mut key_manager = new_test_key_manager();
    init_key_manager_with_seed(&mut key_manager, TEST_SEED_BYTES).unwrap();

    let mut receiver_manager = ReceiverManagerImpl::new(key_manager).build().unwrap();

    // Create batch of paths
    let paths: Vec<_> = (0..10).map(|i| Bip44Path::payment(i).unwrap()).collect();

    // Derive batch
    let results = receiver_manager.derive_batch(&paths).unwrap();

    // Verify results
    assert_eq!(results.len(), 10);

    // Verify all are cached
    for (i, path) in paths.iter().enumerate() {
        let cached = receiver_manager.get_receiver_key(*path).unwrap();
        assert_eq!(cached, results[i]);
    }

    // Verify metrics
    assert_eq!(receiver_manager.get_metrics().total_derivations, 10);
    assert_eq!(receiver_manager.get_metrics().misses, 10); // All first-time derivations
}

#[test]
fn test_batch_matches_sequential() {
    // Use same seed for both managers
    let mut key_manager1 = new_test_key_manager();
    init_key_manager_with_seed(&mut key_manager1, TEST_SEED_BYTES).unwrap();

    let mut key_manager2 = new_test_key_manager();
    init_key_manager_with_seed(&mut key_manager2, TEST_SEED_BYTES).unwrap();

    let mut receiver_manager_batch = ReceiverManagerImpl::new(key_manager1).build().unwrap();
    let mut receiver_manager_seq = ReceiverManagerImpl::new(key_manager2).build().unwrap();

    let paths: Vec<_> = (0..5).map(|i| Bip44Path::payment(i).unwrap()).collect();

    // Batch derivation
    let batch_results = receiver_manager_batch.derive_batch(&paths).unwrap();

    // Sequential derivation
    let mut seq_results = Vec::new();
    for path in &paths {
        seq_results.push(receiver_manager_seq.derive_spend_key(*path).unwrap());
    }

    // Results should be identical
    assert_eq!(batch_results, seq_results);
}

#[test]
fn test_ttl_expiration() {
    use z00z_utils::time::MockTimeProvider;

    let mut key_manager = new_test_key_manager();
    init_key_manager_with_seed(&mut key_manager, TEST_SEED_BYTES).unwrap();

    let time = MockTimeProvider::system_now();
    let mut receiver_manager = ReceiverManagerImpl::new_with_config(
        key_manager,
        100,
        Duration::from_secs(60), // 60 second TTL
        time.clone(),
    )
    .unwrap();

    // Derive an address
    let path = Bip44Path::payment(0).unwrap();
    receiver_manager.derive_spend_key(path).unwrap();
    assert_eq!(receiver_manager.cache().unwrap().len(), 1);

    // Advance time by 30 seconds - should still be valid
    time.advance_by(Duration::from_secs(30));
    assert!(receiver_manager.get_receiver_key(path).is_ok());

    // Advance time by another 31 seconds (total 61) - should be expired
    time.advance_by(Duration::from_secs(31));

    // get_receiver_key checks expiration and returns NotFound for expired entries
    assert!(matches!(
        receiver_manager.get_receiver_key(path),
        Err(ReceiverManagerError::NotFound(_))
    ));

    // Phase 16: expired entries are removed immediately on access.
    assert_eq!(receiver_manager.cache().unwrap().len(), 0);

    // list_receivers should also filter out expired
    let receivers = receiver_manager.list_receivers().unwrap();
    assert_eq!(receivers.len(), 0);

    // Now derive again - this will trigger evict_expired and re-derive
    receiver_manager.derive_spend_key(path).unwrap();

    // Metrics should show the expiration happened:
    // - ttl_expirations: 1 (from get_receiver_key removing expired)
    // - evictions: 1 (from get_receiver_key removing expired)
    assert_eq!(receiver_manager.get_metrics().ttl_expirations, 1);
    assert_eq!(receiver_manager.get_metrics().evictions, 1);
}

#[test]
fn test_purge_expired_removes_all() {
    use z00z_utils::time::MockTimeProvider;

    let mut key_manager = new_test_key_manager();
    init_key_manager_with_seed(&mut key_manager, TEST_SEED_BYTES).unwrap();

    let time = MockTimeProvider::system_now();
    let mut receiver_manager = ReceiverManagerImpl::new_with_config(
        key_manager,
        100,
        Duration::from_secs(60),
        time.clone(),
    )
    .unwrap();

    receiver_manager
        .derive_spend_key(Bip44Path::payment(0).unwrap())
        .unwrap();
    receiver_manager
        .derive_spend_key(Bip44Path::payment(1).unwrap())
        .unwrap();
    assert_eq!(receiver_manager.cache().unwrap().len(), 2);

    time.advance_by(Duration::from_secs(61));

    receiver_manager.purge_expired().unwrap();
    assert_eq!(receiver_manager.cache().unwrap().len(), 0);

    let metrics = receiver_manager.get_metrics();
    assert_eq!(metrics.ttl_expirations, 2);
    assert_eq!(metrics.evictions, 2);
}

#[test]
fn test_evict_expired_on_derive() {
    use z00z_utils::time::MockTimeProvider;

    let mut key_manager = new_test_key_manager();
    init_key_manager_with_seed(&mut key_manager, TEST_SEED_BYTES).unwrap();

    let time = MockTimeProvider::system_now();
    let mut receiver_manager = ReceiverManagerImpl::new_with_config(
        key_manager,
        100,
        Duration::from_secs(60),
        time.clone(),
    )
    .unwrap();

    // Derive multiple receiver keys
    for i in 0..5 {
        receiver_manager
            .derive_spend_key(Bip44Path::payment(i).unwrap())
            .unwrap();
    }

    // Make them all expire
    time.advance_by(Duration::from_secs(61));

    // Derive a new address - should trigger cleanup
    receiver_manager
        .derive_spend_key(Bip44Path::payment(5).unwrap())
        .unwrap();

    // Old entries should be gone
    assert_eq!(receiver_manager.cache().unwrap().len(), 1);
    assert_eq!(receiver_manager.get_metrics().ttl_expirations, 5);
    assert_eq!(receiver_manager.get_metrics().evictions, 5);
}

#[test]
fn test_purge_interval_gates_scans() {
    use z00z_utils::time::MockTimeProvider;

    let mut key_manager = new_test_key_manager();
    init_key_manager_with_seed(&mut key_manager, TEST_SEED_BYTES).unwrap();

    let time = MockTimeProvider::system_now();
    let mut receiver_manager = ReceiverManagerImpl::new_with_config(
        key_manager,
        100,
        Duration::from_secs(60),
        time.clone(),
    )
    .unwrap();

    receiver_manager.set_purge_interval(Duration::from_secs(30));
    receiver_manager.set_purge_min_size(1);

    receiver_manager
        .derive_spend_key(Bip44Path::payment(0).unwrap())
        .unwrap();
    receiver_manager
        .derive_spend_key(Bip44Path::payment(1).unwrap())
        .unwrap();

    // No purge should run yet (interval not elapsed).
    let metrics = receiver_manager.get_metrics();
    assert_eq!(metrics.purge_runs, 0);

    time.advance_by(Duration::from_secs(31));
    receiver_manager
        .derive_spend_key(Bip44Path::payment(2).unwrap())
        .unwrap();

    // A purge scan should run, but nothing is expired.
    let metrics = receiver_manager.get_metrics();
    assert_eq!(metrics.purge_runs, 1);
    assert_eq!(metrics.purge_entries, 0);

    time.advance_by(Duration::from_secs(31));
    receiver_manager
        .derive_spend_key(Bip44Path::payment(3).unwrap())
        .unwrap();

    // Two earliest entries are expired and should be removed.
    assert_eq!(receiver_manager.cache().unwrap().len(), 2);

    let metrics = receiver_manager.get_metrics();
    assert_eq!(metrics.purge_runs, 2);
    assert_eq!(metrics.purge_entries, 2);
    assert_eq!(metrics.purge_bytes, 2 * CACHE_ENTRY_EST_BYTES);
}

#[test]
fn test_cache_size_high_water() {
    use z00z_utils::time::MockTimeProvider;

    let mut key_manager = new_test_key_manager();
    init_key_manager_with_seed(&mut key_manager, TEST_SEED_BYTES).unwrap();

    let time = MockTimeProvider::system_now();
    let mut receiver_manager = ReceiverManagerImpl::new_with_config(
        key_manager,
        100,
        Duration::from_secs(5),
        time.clone(),
    )
    .unwrap();

    // Fill cache with a few entries.
    receiver_manager
        .derive_spend_key(Bip44Path::payment(0).unwrap())
        .unwrap();
    receiver_manager
        .derive_spend_key(Bip44Path::payment(1).unwrap())
        .unwrap();
    receiver_manager
        .derive_spend_key(Bip44Path::payment(2).unwrap())
        .unwrap();

    let before = receiver_manager.get_metrics();
    assert!(before.peak_cache_size >= 3);

    // Expire and purge everything.
    time.advance_by(Duration::from_secs(6));
    receiver_manager.purge_expired().unwrap();

    let after = receiver_manager.get_metrics();
    assert_eq!(after.current_cache_size, 0);
    assert_eq!(after.peak_cache_size, before.peak_cache_size);
}

#[test]
fn test_list_receivers_filters_expired() {
    use z00z_utils::time::MockTimeProvider;

    let mut key_manager = new_test_key_manager();
    init_key_manager_with_seed(&mut key_manager, TEST_SEED_BYTES).unwrap();

    let time = MockTimeProvider::system_now();
    let mut receiver_manager = ReceiverManagerImpl::new_with_config(
        key_manager,
        100,
        Duration::from_secs(60),
        time.clone(),
    )
    .unwrap();

    // Derive 3 receivers
    for i in 0..3 {
        receiver_manager
            .derive_spend_key(Bip44Path::payment(i).unwrap())
            .unwrap();
    }

    // Make all expire
    time.advance_by(Duration::from_secs(61));

    // List should return empty (all expired)
    let receivers = receiver_manager.list_receivers().unwrap();
    assert_eq!(receivers.len(), 0);
}

#[test]
fn test_cache_metrics_hit_rate() {
    let mut key_manager = new_test_key_manager();
    init_key_manager_with_seed(&mut key_manager, TEST_SEED_BYTES).unwrap();

    let mut receiver_manager = ReceiverManagerImpl::new(key_manager).build().unwrap();

    // Derive 5 receivers (5 misses)
    for i in 0..5 {
        receiver_manager
            .derive_spend_key(Bip44Path::payment(i).unwrap())
            .unwrap();
    }

    // Access them all again (5 hits)
    for i in 0..5 {
        receiver_manager
            .derive_spend_key(Bip44Path::payment(i).unwrap())
            .unwrap();
    }

    // get_receiver_key calls don't count in total_derivations
    // So total_derivations = 10 (5 first + 5 second)
    // Hits = 5 (second access)
    // Misses = 5 (first access)

    let metrics = receiver_manager.get_metrics();
    assert_eq!(metrics.total_derivations, 10);
    assert_eq!(metrics.hits, 5);
    assert_eq!(metrics.misses, 5);

    let hit_rate = metrics.hit_rate();
    assert!((hit_rate - 0.5).abs() < 0.01); // 5/10 = 0.5
}
#[test]
fn test_default_cache_size() {
    let key_manager = new_test_key_manager();

    let receiver_manager = ReceiverManagerImpl::new(key_manager).build().unwrap();

    assert_eq!(receiver_manager.max_cache_size(), DEFAULT_CACHE_SIZE);
}

#[test]
fn test_cache_size_zero() {
    let key_manager = new_test_key_manager();

    let result = ReceiverManagerImpl::new(key_manager).with_limit(0).build();

    assert!(matches!(
        result,
        Err(ReceiverManagerError::InvalidCacheSize(0))
    ));
}

#[test]
fn test_cache_size_max() {
    let key_manager = new_test_key_manager();

    let too_large = MAX_CACHE_SIZE + 1;
    let result = ReceiverManagerImpl::new(key_manager)
        .with_limit(too_large)
        .build();

    assert!(matches!(
        result,
        Err(ReceiverManagerError::InvalidCacheSize(size)) if size == too_large
    ));
}
#[test]
fn test_cache_size_getter() {
    let mut key_manager = new_test_key_manager();
    init_key_manager_with_seed(&mut key_manager, TEST_SEED_BYTES).unwrap();

    let mut receiver_manager = ReceiverManagerImpl::new(key_manager).build().unwrap();

    assert_eq!(receiver_manager.cache_size().unwrap(), 0);

    receiver_manager
        .derive_spend_key(Bip44Path::payment(0).unwrap())
        .unwrap();
    assert_eq!(receiver_manager.cache_size().unwrap(), 1);

    receiver_manager
        .derive_spend_key(Bip44Path::payment(1).unwrap())
        .unwrap();
    assert_eq!(receiver_manager.cache_size().unwrap(), 2);
}

// Async tests
#[test]
fn test_async_batch_threshold_validation() {
    assert!(validate_async_batch_threshold(1).is_ok());
    assert!(validate_async_batch_threshold(ASYNC_BATCH_THRESHOLD).is_ok());
    assert!(validate_async_batch_threshold(MAX_ASYNC_BATCH_THRESHOLD).is_ok());
    assert!(matches!(
        validate_async_batch_threshold(0),
        Err(ReceiverManagerError::InvalidAsyncBatchThreshold(0))
    ));
    assert!(matches!(
        validate_async_batch_threshold(MAX_ASYNC_BATCH_THRESHOLD + 1),
        Err(ReceiverManagerError::InvalidAsyncBatchThreshold(_))
    ));
}
#[test]
fn test_async_batch_threshold_respected() {
    let cfg = ReceiverManagerConfig {
        async_batch_threshold: Some(5),
    };
    cfg.validate().unwrap();
    let threshold = cfg.async_batch_threshold.unwrap();
    assert!(use_inline_batch(4, threshold));
    assert!(!use_inline_batch(5, threshold));
}

#[tokio::test]
async fn test_async_batch_selects_path() {
    let mut key_manager = new_test_key_manager();
    init_key_manager_with_seed(&mut key_manager, TEST_SEED_BYTES).unwrap();

    let sync_mgr = ReceiverManagerImpl::new(key_manager).build().unwrap();
    let cfg = ReceiverManagerConfig {
        async_batch_threshold: Some(3),
    };
    let async_mgr = AsyncReceiverManagerImpl::new_with_config(sync_mgr, cfg).unwrap();

    let inline_before = async_mgr.trace.inline_calls.load(Ordering::Relaxed);
    let spawn_before = async_mgr.trace.spawn_blocking_calls.load(Ordering::Relaxed);

    // batch_len < threshold => inline path
    let paths_inline: Vec<_> = (0..2).map(|i| Bip44Path::payment(i).unwrap()).collect();
    async_mgr.derive_batch(&paths_inline).await.unwrap();
    assert_eq!(
        async_mgr.trace.inline_calls.load(Ordering::Relaxed),
        inline_before + 1
    );
    assert_eq!(
        async_mgr.trace.spawn_blocking_calls.load(Ordering::Relaxed),
        spawn_before
    );

    // batch_len == threshold => spawn_blocking path
    let paths_async: Vec<_> = (0..3).map(|i| Bip44Path::payment(i).unwrap()).collect();
    async_mgr.derive_batch(&paths_async).await.unwrap();
    assert_eq!(
        async_mgr.trace.spawn_blocking_calls.load(Ordering::Relaxed),
        spawn_before + 1
    );
}

#[tokio::test]
async fn test_async_derive_spend_key() {
    let mut key_manager = new_test_key_manager();
    init_key_manager_with_seed(&mut key_manager, TEST_SEED_BYTES).unwrap();

    let sync_mgr = ReceiverManagerImpl::new(key_manager).build().unwrap();
    let async_mgr = AsyncReceiverManagerImpl::new(sync_mgr);

    let path = Bip44Path::payment(0).unwrap();
    let pubkey = async_mgr.derive_spend_key(path).await.unwrap();

    // Verify it's cached
    let cached = async_mgr.get_receiver_key(path).await.unwrap();
    assert_eq!(pubkey, cached);
}

#[tokio::test]
async fn test_async_batch_derivation() {
    let mut key_manager = new_test_key_manager();
    init_key_manager_with_seed(&mut key_manager, TEST_SEED_BYTES).unwrap();

    let sync_mgr = ReceiverManagerImpl::new(key_manager).build().unwrap();
    let async_mgr = AsyncReceiverManagerImpl::new(sync_mgr);

    let paths: Vec<_> = (0..10).map(|i| Bip44Path::payment(i).unwrap()).collect();

    // Batch derivation
    let results = async_mgr.derive_batch(&paths).await.unwrap();

    assert_eq!(results.len(), 10);

    // Verify all cached
    for (i, path) in paths.iter().enumerate() {
        let cached = async_mgr.get_receiver_key(*path).await.unwrap();
        assert_eq!(cached, results[i]);
    }
}

#[tokio::test]
async fn test_async_concurrent_access() {
    let mut key_manager = new_test_key_manager();
    init_key_manager_with_seed(&mut key_manager, TEST_SEED_BYTES).unwrap();

    let sync_mgr = ReceiverManagerImpl::new(key_manager).build().unwrap();
    let async_mgr = AsyncReceiverManagerImpl::new(sync_mgr);

    // Spawn multiple concurrent tasks
    let mut handles = Vec::new();

    for i in 0..10 {
        let mgr = async_mgr.clone();
        let path = Bip44Path::payment(i).unwrap();

        handles.push(tokio::spawn(
            async move { mgr.derive_spend_key(path).await },
        ));
    }

    // Wait for all to complete
    for handle in handles {
        handle.await.unwrap().unwrap();
    }

    // Verify metrics
    let metrics = async_mgr.metrics().await;
    assert_eq!(metrics.total_derivations, 10);
}

#[tokio::test]
async fn test_auto_tune_fast_derivation() {
    let mut key_manager = new_test_key_manager();
    init_key_manager_with_seed(&mut key_manager, TEST_SEED_BYTES).unwrap();

    let sync_mgr = ReceiverManagerImpl::new(key_manager).build().unwrap();

    // Simulate fast derivation (avg < 5ms)
    sync_mgr
        .metrics
        .total_derive_time_ms
        .store(40, Ordering::Relaxed); // 40ms total
    sync_mgr.metrics.derive_count.store(10, Ordering::Relaxed); // 10 derivations = 4ms avg

    let async_mgr = AsyncReceiverManagerImpl::new(sync_mgr);
    assert_eq!(
        async_mgr.batch_threshold.load(Ordering::Relaxed),
        ASYNC_BATCH_THRESHOLD
    );

    async_mgr.auto_tune_threshold().await;

    // Fast derivation → threshold should be 20
    assert_eq!(async_mgr.batch_threshold.load(Ordering::Relaxed), 20);
}

#[tokio::test]
async fn test_auto_tune_slow_derivation() {
    let mut key_manager = new_test_key_manager();
    init_key_manager_with_seed(&mut key_manager, TEST_SEED_BYTES).unwrap();

    let sync_mgr = ReceiverManagerImpl::new(key_manager).build().unwrap();

    // Simulate slow derivation (avg >= 5ms)
    sync_mgr
        .metrics
        .total_derive_time_ms
        .store(60, Ordering::Relaxed); // 60ms total
    sync_mgr.metrics.derive_count.store(10, Ordering::Relaxed); // 10 derivations = 6ms avg

    let async_mgr = AsyncReceiverManagerImpl::new(sync_mgr);
    assert_eq!(
        async_mgr.batch_threshold.load(Ordering::Relaxed),
        ASYNC_BATCH_THRESHOLD
    );

    async_mgr.auto_tune_threshold().await;

    // Slow derivation → threshold should remain 10
    assert_eq!(async_mgr.batch_threshold.load(Ordering::Relaxed), 10);
}

#[tokio::test]
async fn test_fixed_threshold_config() {
    let mut key_manager = new_test_key_manager();
    init_key_manager_with_seed(&mut key_manager, TEST_SEED_BYTES).unwrap();

    let sync_mgr = ReceiverManagerImpl::new(key_manager).build().unwrap();

    // Fixed threshold = 15 (Some variant)
    let config = ReceiverManagerConfig {
        async_batch_threshold: Some(15),
    };

    let async_mgr = AsyncReceiverManagerImpl::new_with_config(sync_mgr, config).unwrap();
    assert_eq!(async_mgr.batch_threshold.load(Ordering::Relaxed), 15);

    // Verify config returns Some(15)
    assert_eq!(async_mgr.config().async_batch_threshold, Some(15));

    // Auto-tune is disabled for fixed threshold.
    {
        let guard = async_mgr.inner.write().await;
        guard
            .metrics
            .total_derive_time_ms
            .store(20, Ordering::Relaxed);
        guard.metrics.derive_count.store(10, Ordering::Relaxed);
    }
    async_mgr.auto_tune_threshold().await;
    assert_eq!(async_mgr.batch_threshold.load(Ordering::Relaxed), 15);
}

#[tokio::test]
async fn test_default_config_autotune() {
    let mut key_manager = new_test_key_manager();
    init_key_manager_with_seed(&mut key_manager, TEST_SEED_BYTES).unwrap();

    let sync_mgr = ReceiverManagerImpl::new(key_manager).build().unwrap();

    // Default config has None → auto-tune enabled
    let config = ReceiverManagerConfig::default();
    assert_eq!(config.async_batch_threshold, None);

    let async_mgr = AsyncReceiverManagerImpl::new_with_config(sync_mgr, config).unwrap();

    // None → defaults to ASYNC_BATCH_THRESHOLD (10)
    assert_eq!(
        async_mgr.batch_threshold.load(Ordering::Relaxed),
        ASYNC_BATCH_THRESHOLD
    );
}

#[tokio::test]
async fn test_threshold_adapts() {
    let mut key_manager = new_test_key_manager();
    init_key_manager_with_seed(&mut key_manager, TEST_SEED_BYTES).unwrap();

    let sync_mgr = ReceiverManagerImpl::new(key_manager).build().unwrap();

    // Start with slow derivation
    sync_mgr
        .metrics
        .total_derive_time_ms
        .store(60, Ordering::Relaxed);
    sync_mgr.metrics.derive_count.store(10, Ordering::Relaxed); // 6ms avg

    let async_mgr = AsyncReceiverManagerImpl::new(sync_mgr);
    async_mgr.auto_tune_threshold().await;
    assert_eq!(async_mgr.batch_threshold.load(Ordering::Relaxed), 10); // Slow → 10

    // Performance improves
    let manager = async_mgr.inner.write().await;
    manager
        .metrics
        .total_derive_time_ms
        .store(80, Ordering::Relaxed); // +20ms
    manager.metrics.derive_count.store(30, Ordering::Relaxed); // +20 derivations = 2.67ms avg
    drop(manager);

    async_mgr.auto_tune_threshold().await;
    assert_eq!(async_mgr.batch_threshold.load(Ordering::Relaxed), 20); // Fast → 20

    // Performance degrades again
    let manager = async_mgr.inner.write().await;
    manager
        .metrics
        .total_derive_time_ms
        .store(280, Ordering::Relaxed); // +200ms
    manager.metrics.derive_count.store(50, Ordering::Relaxed); // +20 derivations = 5.6ms avg
    drop(manager);

    async_mgr.auto_tune_threshold().await;
    assert_eq!(async_mgr.batch_threshold.load(Ordering::Relaxed), 10); // Slow again → 10
}

#[tokio::test]
async fn test_async_expired_pop() {
    use z00z_utils::time::MockTimeProvider;

    let mut key_manager = new_test_key_manager();
    init_key_manager_with_seed(&mut key_manager, TEST_SEED_BYTES).unwrap();

    let time = MockTimeProvider::system_now();
    let sync_mgr = ReceiverManagerImpl::new_with_config(
        key_manager,
        100,
        Duration::from_secs(60),
        time.clone(),
    )
    .unwrap();
    let async_mgr = AsyncReceiverManagerImpl::new(sync_mgr);

    let path = Bip44Path::payment(0).unwrap();
    async_mgr.derive_spend_key(path).await.unwrap();
    time.advance_by(Duration::from_secs(61));

    let mut handles = Vec::new();
    for _ in 0..10 {
        let mgr = async_mgr.clone();
        handles.push(tokio::spawn(
            async move { mgr.get_receiver_key(path).await },
        ));
    }

    for handle in handles {
        assert!(matches!(
            handle.await.unwrap(),
            Err(ReceiverManagerError::NotFound(_))
        ));
    }

    let addrs = async_mgr.list_receivers().await.unwrap();
    assert!(addrs.is_empty());

    let metrics = async_mgr.metrics().await;
    assert_eq!(metrics.ttl_expirations, 1);
    assert_eq!(metrics.evictions, 1);
}

#[tokio::test]
async fn test_async_metrics() {
    let mut key_manager = new_test_key_manager();
    init_key_manager_with_seed(&mut key_manager, TEST_SEED_BYTES).unwrap();

    let sync_mgr = ReceiverManagerImpl::new(key_manager).build().unwrap();
    let async_mgr = AsyncReceiverManagerImpl::new(sync_mgr);

    let path = Bip44Path::payment(0).unwrap();

    // First derivation - miss
    async_mgr.derive_spend_key(path).await.unwrap();

    // Second call - hit
    async_mgr.derive_spend_key(path).await.unwrap();

    let metrics = async_mgr.metrics().await;
    assert_eq!(metrics.total_derivations, 2);
    assert_eq!(metrics.hits, 1);
    assert_eq!(metrics.misses, 1);
}

#[tokio::test]
async fn test_async_clear_cache() {
    let mut key_manager = new_test_key_manager();
    init_key_manager_with_seed(&mut key_manager, TEST_SEED_BYTES).unwrap();

    let sync_mgr = ReceiverManagerImpl::new(key_manager).build().unwrap();
    let async_mgr = AsyncReceiverManagerImpl::new(sync_mgr);

    // Derive some receivers
    for i in 0..5 {
        async_mgr
            .derive_spend_key(Bip44Path::payment(i).unwrap())
            .await
            .unwrap();
    }

    // Clear cache
    async_mgr.clear_cache().await.unwrap();

    // Verify empty
    let receivers = async_mgr.list_receivers().await.unwrap();
    assert_eq!(receivers.len(), 0);
}

#[tokio::test]
async fn test_async_batch_vs_sequential() {
    // Use same seed for both managers
    let mut key_manager1 = new_test_key_manager();
    init_key_manager_with_seed(&mut key_manager1, TEST_SEED_BYTES).unwrap();

    let mut key_manager2 = new_test_key_manager();
    init_key_manager_with_seed(&mut key_manager2, TEST_SEED_BYTES).unwrap();

    let sync_mgr1 = ReceiverManagerImpl::new(key_manager1).build().unwrap();
    let async_mgr1 = AsyncReceiverManagerImpl::new(sync_mgr1);

    let sync_mgr2 = ReceiverManagerImpl::new(key_manager2).build().unwrap();
    let async_mgr2 = AsyncReceiverManagerImpl::new(sync_mgr2);

    let paths: Vec<_> = (0..5).map(|i| Bip44Path::payment(i).unwrap()).collect();

    // Batch
    let batch_results = async_mgr1.derive_batch(&paths).await.unwrap();

    // Sequential
    let mut seq_results = Vec::new();
    for path in &paths {
        seq_results.push(async_mgr2.derive_spend_key(*path).await.unwrap());
    }

    assert_eq!(batch_results, seq_results);
}

#[tokio::test]
async fn test_async_ttl_expiration() {
    use z00z_utils::time::MockTimeProvider;

    let mut key_manager = new_test_key_manager();
    init_key_manager_with_seed(&mut key_manager, TEST_SEED_BYTES).unwrap();

    let time = MockTimeProvider::system_now();
    let sync_mgr = ReceiverManagerImpl::new_with_config(
        key_manager,
        100,
        Duration::from_secs(60),
        time.clone(),
    )
    .unwrap();
    let async_mgr = AsyncReceiverManagerImpl::new(sync_mgr);

    let path = Bip44Path::payment(0).unwrap();
    async_mgr.derive_spend_key(path).await.unwrap();

    // Expire
    time.advance_by(Duration::from_secs(61));

    // Should not find expired entry
    let result = async_mgr.get_receiver_key(path).await;
    assert!(matches!(result, Err(ReceiverManagerError::NotFound(_))));
}

#[test]
fn test_metrics_reset() {
    let mut key_manager = new_test_key_manager();
    init_key_manager_with_seed(&mut key_manager, TEST_SEED_BYTES).unwrap();

    let mut receiver_manager = ReceiverManagerImpl::new(key_manager).build().unwrap();

    // Generate some activity
    for i in 0..5 {
        receiver_manager
            .derive_spend_key(Bip44Path::payment(i).unwrap())
            .unwrap();
    }

    let metrics_before = receiver_manager.get_metrics().clone();
    assert!(metrics_before.total_derivations > 0);

    receiver_manager.reset_metrics();

    let metrics_after = receiver_manager.get_metrics();
    assert_eq!(metrics_after.total_derivations, 0);
    assert_eq!(metrics_after.hits, 0);
    assert_eq!(metrics_after.misses, 0);
}

#[test]
fn test_cache_persistence() {
    let mut key_manager = new_test_key_manager();
    init_key_manager_with_seed(&mut key_manager, TEST_SEED_BYTES).unwrap();

    let mut receiver_manager = ReceiverManagerImpl::new(key_manager).build().unwrap();
    let wallet_id = b"test_wallet_id";

    // Derive some receivers
    for i in 0..5 {
        receiver_manager
            .derive_spend_key(Bip44Path::payment(i).unwrap())
            .unwrap();
    }

    // Export cache
    let snapshot = receiver_manager.export_cache(wallet_id).unwrap();
    assert_eq!(snapshot.entries.len(), 5);
    assert_eq!(snapshot.version, 3);

    // Create new manager and import
    let mut key_manager2 = new_test_key_manager();
    init_key_manager_with_seed(&mut key_manager2, TEST_SEED_BYTES).unwrap();

    let mut receiver_manager2 = ReceiverManagerImpl::new(key_manager2).build().unwrap();
    receiver_manager2.import_cache(wallet_id, snapshot).unwrap();

    // Verify cache is restored
    assert_eq!(receiver_manager2.cache().unwrap().len(), 5);

    // Verify keys match
    for i in 0..5 {
        let path = Bip44Path::payment(i).unwrap();
        let key1 = receiver_manager.get_receiver_key(path).unwrap();
        let key2 = receiver_manager2.get_receiver_key(path).unwrap();
        assert_eq!(key1, key2);
    }

    // Verify atomic import: no partial accepts
    let metrics = receiver_manager2.get_metrics();
    assert_eq!(metrics.import_rejects, 0);
}

#[test]
fn test_snapshot_verify_ok() {
    let mut key_manager = new_test_key_manager();
    init_key_manager_with_seed(&mut key_manager, TEST_SEED_BYTES).unwrap();
    let mut receiver_manager = ReceiverManagerImpl::new(key_manager).build().unwrap();

    receiver_manager
        .derive_spend_key(Bip44Path::payment(0).unwrap())
        .unwrap();

    let wallet_id = b"test_wallet_id";
    let snapshot = receiver_manager.export_cache(wallet_id).unwrap();
    let mac_key = ReceiverCacheState::mac_key(&receiver_manager.key_manager, wallet_id).unwrap();
    snapshot.verify(wallet_id, &mac_key).unwrap();
}

#[test]
fn test_snapshot_entry_tamper() {
    let mut key_manager = new_test_key_manager();
    init_key_manager_with_seed(&mut key_manager, TEST_SEED_BYTES).unwrap();
    let mut receiver_manager = ReceiverManagerImpl::new(key_manager).build().unwrap();

    receiver_manager
        .derive_spend_key(Bip44Path::payment(0).unwrap())
        .unwrap();

    let wallet_id = b"test_wallet_id";
    let mut snapshot = receiver_manager.export_cache(wallet_id).unwrap();
    snapshot.entries[0].1[0] ^= 0x01;

    let err = receiver_manager
        .import_cache(wallet_id, snapshot)
        .unwrap_err();
    assert!(matches!(
        err,
        ReceiverManagerError::CacheAuthenticationFailed
    ));
}

#[test]
fn test_snapshot_hmac_tamper() {
    let mut key_manager = new_test_key_manager();
    init_key_manager_with_seed(&mut key_manager, TEST_SEED_BYTES).unwrap();
    let mut receiver_manager = ReceiverManagerImpl::new(key_manager).build().unwrap();

    receiver_manager
        .derive_spend_key(Bip44Path::payment(0).unwrap())
        .unwrap();

    let wallet_id = b"test_wallet_id";
    let mut snapshot = receiver_manager.export_cache(wallet_id).unwrap();
    snapshot.hmac[0] ^= 0x01;

    let err = receiver_manager
        .import_cache(wallet_id, snapshot)
        .unwrap_err();
    assert!(matches!(
        err,
        ReceiverManagerError::CacheAuthenticationFailed
    ));
}

#[test]
fn test_snapshot_wallet_id_mismatch() {
    let mut key_manager = new_test_key_manager();
    init_key_manager_with_seed(&mut key_manager, TEST_SEED_BYTES).unwrap();
    let mut receiver_manager = ReceiverManagerImpl::new(key_manager).build().unwrap();

    receiver_manager
        .derive_spend_key(Bip44Path::payment(0).unwrap())
        .unwrap();

    let wallet_id = b"test_wallet_id";
    let wrong_wallet_id = b"wrong_wallet_id";
    let snapshot = receiver_manager.export_cache(wallet_id).unwrap();

    let err = receiver_manager
        .import_cache(wrong_wallet_id, snapshot)
        .unwrap_err();
    assert!(matches!(
        err,
        ReceiverManagerError::CacheAuthenticationFailed
    ));
}

/// Test: Different wallet_ids produce different MAC keys (HKDF binds to wallet context)
#[test]
fn test_key_unique_per_wallet() {
    let mut key_manager = new_test_key_manager();
    init_key_manager_with_seed(&mut key_manager, TEST_SEED_BYTES).unwrap();

    let wallet_id_a = b"wallet_a";
    let wallet_id_b = b"wallet_b";

    let mac_key_a = ReceiverCacheState::mac_key(&key_manager, wallet_id_a).unwrap();
    let mac_key_b = ReceiverCacheState::mac_key(&key_manager, wallet_id_b).unwrap();

    // Different wallet_ids MUST produce different MAC keys
    assert_ne!(
        mac_key_a, mac_key_b,
        "MAC keys must be unique per wallet_id"
    );
}

/// Test: Same wallet_id with different seeds produces different MAC keys
#[test]
fn test_mac_depends_on_seed() {
    let mut key_manager_1 = new_test_key_manager();
    init_key_manager_with_seed(&mut key_manager_1, TEST_SEED_BYTES).unwrap();

    let mut key_manager_2 = new_test_key_manager();
    let mut seed_2 = TEST_SEED_BYTES;
    seed_2[0] ^= 0xFF; // Modify seed
    init_key_manager_with_seed(&mut key_manager_2, seed_2).unwrap();

    let wallet_id = b"test_wallet";

    let mac_key_1 = ReceiverCacheState::mac_key(&key_manager_1, wallet_id).unwrap();
    let mac_key_2 = ReceiverCacheState::mac_key(&key_manager_2, wallet_id).unwrap();

    // Different seeds MUST produce different MAC keys
    assert_ne!(mac_key_1, mac_key_2, "MAC keys must depend on seed");
}

/// Test: Snapshot with wrong wallet_id fails HMAC verification
#[test]
fn test_snapshot_wallet_forge_blocked() {
    let mut key_manager = new_test_key_manager();
    init_key_manager_with_seed(&mut key_manager, TEST_SEED_BYTES).unwrap();
    let mut receiver_manager = ReceiverManagerImpl::new(key_manager).build().unwrap();

    // Create snapshot for wallet_a
    receiver_manager
        .derive_spend_key(Bip44Path::payment(0).unwrap())
        .unwrap();

    let wallet_a = b"wallet_a";
    let wallet_b = b"wallet_b";
    let snapshot = receiver_manager.export_cache(wallet_a).unwrap();

    // Try to import snapshot for wallet_b - MUST FAIL
    let err = receiver_manager
        .import_cache(wallet_b, snapshot)
        .unwrap_err();
    assert!(
        matches!(err, ReceiverManagerError::CacheAuthenticationFailed),
        "Cross-wallet forgery must be blocked"
    );
}

/// Test: MAC key derivation is deterministic for same inputs
#[test]
fn test_mac_key_deterministic() {
    let mut key_manager = new_test_key_manager();
    init_key_manager_with_seed(&mut key_manager, TEST_SEED_BYTES).unwrap();

    let wallet_id = b"test_wallet";

    let mac_key_1 = ReceiverCacheState::mac_key(&key_manager, wallet_id).unwrap();
    let mac_key_2 = ReceiverCacheState::mac_key(&key_manager, wallet_id).unwrap();

    // Same inputs MUST produce same MAC key
    assert_eq!(
        mac_key_1, mac_key_2,
        "MAC key derivation must be deterministic"
    );
}

#[test]
fn test_snapshot_wrong_version_rejected() {
    let mut key_manager = new_test_key_manager();
    init_key_manager_with_seed(&mut key_manager, TEST_SEED_BYTES).unwrap();
    let mut receiver_manager = ReceiverManagerImpl::new(key_manager).build().unwrap();

    let wallet_id = b"test_wallet_id";
    let mut snapshot = ReceiverCacheState {
        version: 2,
        entries: vec![],
        hmac: [0u8; 32],
    };
    let mac_key = ReceiverCacheState::mac_key(&receiver_manager.key_manager, wallet_id).unwrap();
    snapshot.sign(wallet_id, &mac_key).unwrap();

    let err = receiver_manager
        .import_cache(wallet_id, snapshot)
        .unwrap_err();
    assert!(matches!(err, ReceiverManagerError::ImportEntryRejected(_)));
}

#[test]
fn test_cache_warming() {
    let mut key_manager = new_test_key_manager();
    init_key_manager_with_seed(&mut key_manager, TEST_SEED_BYTES).unwrap();

    let mut receiver_manager = ReceiverManagerImpl::new(key_manager).build().unwrap();

    // Warm cache for first 10 receivers
    receiver_manager.warm_cache(10).unwrap();

    // All should be cached
    assert_eq!(receiver_manager.cache().unwrap().len(), 10);

    // Verify metrics
    let metrics = receiver_manager.get_metrics();
    assert_eq!(metrics.total_derivations, 10);
    assert_eq!(metrics.misses, 10);
}

#[test]
fn test_cache_rejects_invalid_entries() {
    let mut key_manager = new_test_key_manager();
    init_key_manager_with_seed(&mut key_manager, TEST_SEED_BYTES).unwrap();

    let mut receiver_manager = ReceiverManagerImpl::new(key_manager).build().unwrap();
    let wallet_id = b"test_wallet_id";

    // Derive some receivers
    for i in 0..3 {
        receiver_manager
            .derive_spend_key(Bip44Path::payment(i).unwrap())
            .unwrap();
    }

    // Export cache
    let mut snapshot = receiver_manager.export_cache(wallet_id).unwrap();

    // Manually add an invalid entry (invalid key bytes length = 33)
    let invalid_path = Bip44Path::payment(0).unwrap();
    snapshot
        .entries
        .push((invalid_path, vec![0u8; 33], vec![0u8; 32]));
    let mac_key = ReceiverCacheState::mac_key(&receiver_manager.key_manager, wallet_id).unwrap();

    // Phase 7: sign() now validates canonical format - should fail when serializing
    let result = snapshot.sign(wallet_id, &mac_key);
    assert!(matches!(
        result,
        Err(ReceiverManagerError::InvalidReceiverCacheState(_))
    ));
}

#[test]
fn test_cache_filters_expired_entries() {
    use z00z_utils::time::MockTimeProvider;

    let mut key_manager = new_test_key_manager();
    init_key_manager_with_seed(&mut key_manager, TEST_SEED_BYTES).unwrap();

    let time = MockTimeProvider::system_now();
    let mut receiver_manager = ReceiverManagerImpl::new_with_config(
        key_manager,
        100,
        Duration::from_secs(60),
        time.clone(),
    )
    .unwrap();
    let wallet_id = b"test_wallet_id";

    // Derive some receivers
    for i in 0..5 {
        receiver_manager
            .derive_spend_key(Bip44Path::payment(i).unwrap())
            .unwrap();
    }

    // Make some entries expire
    time.advance_by(Duration::from_secs(61));

    // Export cache - should only include non-expired entries
    let snapshot = receiver_manager.export_cache(wallet_id).unwrap();
    assert_eq!(snapshot.entries.len(), 0);
}

#[test]
fn test_lookup_metrics() {
    let mut key_manager = new_test_key_manager();
    init_key_manager_with_seed(&mut key_manager, TEST_SEED_BYTES).unwrap();

    let mut receiver_manager = ReceiverManagerImpl::new(key_manager).build().unwrap();

    // Derive one address (1 miss)
    let path = Bip44Path::payment(0).unwrap();
    receiver_manager.derive_spend_key(path).unwrap();

    // Get it multiple times (3 hits)
    for _ in 0..3 {
        let _ = receiver_manager.get_receiver_key(path);
    }

    // Try to get non-existent (1 miss)
    let _ = receiver_manager.get_receiver_key(Bip44Path::payment(99).unwrap());

    let metrics = receiver_manager.get_metrics();
    assert_eq!(metrics.total_lookups, 4);
    assert_eq!(metrics.lookup_hits, 3);
    assert_eq!(metrics.lookup_misses, 1);
    // Overall hit rate: (0 deriv hits + 3 lookup hits) / (1 deriv + 4 lookups) = 3/5 = 0.6
    assert!((metrics.overall_hit_rate() - 0.6).abs() < 0.01);
}

#[test]
fn test_async_new_features() {
    use tokio::runtime::Runtime;

    let rt = Runtime::new().unwrap();
    rt.block_on(async {
        let mut key_manager = new_test_key_manager();
        init_key_manager_with_seed(&mut key_manager, TEST_SEED_BYTES).unwrap();

        let sync_mgr = ReceiverManagerImpl::new(key_manager).build().unwrap();
        let async_mgr = AsyncReceiverManagerImpl::new(sync_mgr);
        let wallet_id = b"test_wallet_id";

        // Test async cache warming
        async_mgr.warm_cache(5).await.unwrap();

        // Test async cache export
        let snapshot = async_mgr.export_cache(wallet_id).await.unwrap();
        assert_eq!(snapshot.entries.len(), 5);

        // Test async metrics
        let metrics = async_mgr.metrics().await;
        assert_eq!(metrics.total_derivations, 5);
    });
}

#[test]
fn test_persistence_invalid_entries_rejected() {
    let mut key_manager = new_test_key_manager();
    init_key_manager_with_seed(&mut key_manager, TEST_SEED_BYTES).unwrap();

    let mut receiver_manager = ReceiverManagerImpl::new(key_manager).build().unwrap();
    let wallet_id = b"test_wallet_id";

    // Create snapshot with invalid entries (identity keys)
    let invalid_path = Bip44Path::payment(0).unwrap();
    let invalid_key = [0u8; 32];

    let mut snap = ReceiverCacheState {
        version: 3,
        entries: vec![(invalid_path, invalid_key.to_vec(), invalid_key.to_vec())],
        hmac: [0u8; 32],
    };
    let mac_key = ReceiverCacheState::mac_key(&receiver_manager.key_manager, wallet_id).unwrap();
    snap.sign(wallet_id, &mac_key).unwrap();
    let snapshot = snap;

    // Import should fail (atomic: invalid entry rejects entire snapshot)
    let result = receiver_manager.import_cache(wallet_id, snapshot);
    assert!(matches!(
        result,
        Err(ReceiverManagerError::ImportEntryRejected(_))
    ));

    // Verify rejection metric counter was incremented
    let metrics = receiver_manager.get_metrics();
    assert_eq!(metrics.import_rejects, 1);
}

#[test]
fn test_failed_import_keeps_cache() {
    let mut key_manager = new_test_key_manager();
    init_key_manager_with_seed(&mut key_manager, TEST_SEED_BYTES).unwrap();

    let mut receiver_manager = ReceiverManagerImpl::new(key_manager).build().unwrap();
    let wallet_id = b"test_wallet_id";

    let keep_path = Bip44Path::payment(1).unwrap();
    let keep_keys = receiver_manager.derive_wallet_keys(keep_path).unwrap();
    assert!(receiver_manager.cache().unwrap().contains(&keep_path));

    let invalid_path = Bip44Path::payment(0).unwrap();
    let invalid_key = [0u8; 32];
    let mut snap = ReceiverCacheState {
        version: 3,
        entries: vec![(invalid_path, invalid_key.to_vec(), invalid_key.to_vec())],
        hmac: [0u8; 32],
    };
    let mac_key = ReceiverCacheState::mac_key(&receiver_manager.key_manager, wallet_id).unwrap();
    snap.sign(wallet_id, &mac_key).unwrap();
    let snapshot = snap;

    let result = receiver_manager.import_cache(wallet_id, snapshot);
    assert!(matches!(
        result,
        Err(ReceiverManagerError::ImportEntryRejected(_))
    ));

    assert!(receiver_manager.cache().unwrap().contains(&keep_path));
    assert_eq!(receiver_manager.cache().unwrap().len(), 1);

    let keep_keys2 = receiver_manager.derive_wallet_keys(keep_path).unwrap();
    assert_eq!(
        keep_keys.spend_key.as_bytes(),
        keep_keys2.spend_key.as_bytes()
    );
    assert_eq!(
        keep_keys.view_key.as_bytes(),
        keep_keys2.view_key.as_bytes()
    );
}

#[test]
fn test_persistence_invalid_bytes_rejected() {
    let mut key_manager = new_test_key_manager();
    init_key_manager_with_seed(&mut key_manager, TEST_SEED_BYTES).unwrap();

    let receiver_manager = ReceiverManagerImpl::new(key_manager).build().unwrap();
    let wallet_id = b"test_wallet_id";

    // Create snapshot with invalid key bytes (31 bytes instead of 32)
    let valid_path = Bip44Path::payment(0).unwrap();
    let invalid_key = [0u8; 31];

    let mut snap = ReceiverCacheState {
        version: 3,
        entries: vec![(valid_path, invalid_key.to_vec(), vec![0u8; 32])],
        hmac: [0u8; 32],
    };
    let mac_key = ReceiverCacheState::mac_key(&receiver_manager.key_manager, wallet_id).unwrap();

    // Phase 7: sign() now validates canonical format - should fail immediately
    let result = snap.sign(wallet_id, &mac_key);
    assert!(matches!(
        result,
        Err(ReceiverManagerError::InvalidReceiverCacheState(_))
    ));
}

#[test]
fn test_persistence_wrong_version_ignored() {
    let mut key_manager = new_test_key_manager();
    init_key_manager_with_seed(&mut key_manager, TEST_SEED_BYTES).unwrap();

    let mut receiver_manager = ReceiverManagerImpl::new(key_manager).build().unwrap();
    let wallet_id = b"test_wallet_id";

    // Create snapshot with wrong version
    let mut snap = ReceiverCacheState {
        version: 255,
        entries: vec![],
        hmac: [0u8; 32],
    };
    let mac_key = ReceiverCacheState::mac_key(&receiver_manager.key_manager, wallet_id).unwrap();
    snap.sign(wallet_id, &mac_key).unwrap();
    let snapshot = snap;

    // Import should fail due to wrong version
    let result = receiver_manager.import_cache(wallet_id, snapshot);
    assert!(matches!(
        result,
        Err(ReceiverManagerError::ImportEntryRejected(_))
    ));

    let metrics = receiver_manager.get_metrics();
    assert_eq!(metrics.import_rejects, 1);
}

#[test]
fn test_import_entry_count_limit() {
    let mut key_manager = new_test_key_manager();
    init_key_manager_with_seed(&mut key_manager, TEST_SEED_BYTES).unwrap();

    let mut receiver_manager = ReceiverManagerImpl::new(key_manager).build().unwrap();
    let wallet_id = b"test_wallet_id";

    let keys = receiver_manager
        .derive_wallet_keys(Bip44Path::payment(0).unwrap())
        .unwrap();
    let spend = keys.spend_key.as_bytes().to_vec();
    let view = keys.view_key.as_bytes().to_vec();

    let mut entries = Vec::with_capacity(MAX_IMPORT_ENTRIES + 1);
    for i in 0..(MAX_IMPORT_ENTRIES as u32 + 1) {
        entries.push((Bip44Path::payment(i).unwrap(), spend.clone(), view.clone()));
    }

    let mut snap = ReceiverCacheState {
        version: 3,
        entries,
        hmac: [0u8; 32],
    };
    let mac_key = ReceiverCacheState::mac_key(&receiver_manager.key_manager, wallet_id).unwrap();
    snap.sign(wallet_id, &mac_key).unwrap();
    let snapshot = snap;

    let err = receiver_manager
        .import_cache(wallet_id, snapshot)
        .unwrap_err();
    assert!(matches!(err, ReceiverManagerError::ImportTooLarge(_)));

    let metrics = receiver_manager.get_metrics();
    assert_eq!(metrics.import_rejects, 1);
}

#[test]
fn test_import_accepts_max_entries() {
    let mut key_manager = new_test_key_manager();
    init_key_manager_with_seed(&mut key_manager, TEST_SEED_BYTES).unwrap();

    let mut receiver_manager = ReceiverManagerImpl::new(key_manager)
        .with_limit(MAX_IMPORT_ENTRIES)
        .build()
        .unwrap();
    let wallet_id = b"test_wallet_id";

    let keys = receiver_manager
        .derive_wallet_keys(Bip44Path::payment(0).unwrap())
        .unwrap();
    let spend = keys.spend_key.as_bytes().to_vec();
    let view = keys.view_key.as_bytes().to_vec();

    let mut entries = Vec::with_capacity(MAX_IMPORT_ENTRIES);
    for i in 0..(MAX_IMPORT_ENTRIES as u32) {
        entries.push((Bip44Path::payment(i).unwrap(), spend.clone(), view.clone()));
    }

    let mut snap = ReceiverCacheState {
        version: 3,
        entries,
        hmac: [0u8; 32],
    };
    let mac_key = ReceiverCacheState::mac_key(&receiver_manager.key_manager, wallet_id).unwrap();
    snap.sign(wallet_id, &mac_key).unwrap();
    let snapshot = snap;
    receiver_manager.import_cache(wallet_id, snapshot).unwrap();

    assert_eq!(receiver_manager.cache().unwrap().len(), MAX_IMPORT_ENTRIES);

    let metrics = receiver_manager.get_metrics();
    assert_eq!(metrics.import_rejects, 0);
    assert_eq!(metrics.import_entries, MAX_IMPORT_ENTRIES as u64);
}

#[test]
/// Phase 7: With canonical format validation, keys must be exactly 32 bytes.
/// This means with MAX_IMPORT_ENTRIES = 10,000 and 64 bytes/entry (32+32),
/// max total_bytes = 640KB, which is far below MAX_IMPORT_SIZE_BYTES (10MB).
/// Therefore, this test actually triggers ImportTooLarge (entry count limit),
/// not ImportExceedsSizeLimit (byte limit). The byte limit is effectively
/// unreachable with valid keys under current constants.
fn test_cache_import_size_limit() {
    let mut key_manager = new_test_key_manager();
    init_key_manager_with_seed(&mut key_manager, TEST_SEED_BYTES).unwrap();

    let mut receiver_manager = ReceiverManagerImpl::new(key_manager).build().unwrap();
    let wallet_id = b"test_wallet_id";

    // Create enough valid entries to exceed entry count limit
    // With valid 32-byte keys, entry count limit is reached before byte limit
    let num_entries = MAX_IMPORT_ENTRIES + 100; // Exceeds MAX_IMPORT_ENTRIES (10,000)
    let entries: Vec<_> = (0..num_entries)
        .map(|i| {
            (
                Bip44Path::payment(i as u32).unwrap(),
                vec![i as u8; 32],
                vec![(i + 1) as u8; 32],
            )
        })
        .collect();

    let mut snap = ReceiverCacheState {
        version: 3,
        entries,
        hmac: [0u8; 32],
    };
    let mac_key = ReceiverCacheState::mac_key(&receiver_manager.key_manager, wallet_id).unwrap();
    snap.sign(wallet_id, &mac_key).unwrap();
    let snapshot = snap;

    let err = receiver_manager
        .import_cache(wallet_id, snapshot)
        .unwrap_err();
    // Phase 7: Entry count limit is reached first (10,000 < 163,840 needed for 10MB)
    assert!(matches!(err, ReceiverManagerError::ImportTooLarge(_)));

    let metrics = receiver_manager.get_metrics();
    assert_eq!(metrics.import_rejects, 1);
}

#[test]
fn test_cache_persistence_empty_snapshot() {
    let mut key_manager = new_test_key_manager();
    init_key_manager_with_seed(&mut key_manager, TEST_SEED_BYTES).unwrap();

    let mut receiver_manager = ReceiverManagerImpl::new(key_manager).build().unwrap();
    let wallet_id = b"test_wallet_id";

    // Create empty snapshot
    let mut snap = ReceiverCacheState {
        version: 3,
        entries: vec![],
        hmac: [0u8; 32],
    };
    let mac_key = ReceiverCacheState::mac_key(&receiver_manager.key_manager, wallet_id).unwrap();
    snap.sign(wallet_id, &mac_key).unwrap();
    let snapshot = snap;

    // Import should succeed
    let result = receiver_manager.import_cache(wallet_id, snapshot);
    assert!(result.is_ok());

    // Verify atomic import: all valid entries accepted
    let metrics = receiver_manager.get_metrics();
    assert_eq!(metrics.import_rejects, 0);
}

#[test]
fn test_cache_persistence_valid_entries() {
    let mut key_manager = new_test_key_manager();
    init_key_manager_with_seed(&mut key_manager, TEST_SEED_BYTES).unwrap();

    let mut receiver_manager = ReceiverManagerImpl::new(key_manager).build().unwrap();
    let wallet_id = b"test_wallet_id";

    // Derive some receivers first
    let path1 = Bip44Path::payment(0).unwrap();
    let path2 = Bip44Path::payment(1).unwrap();
    receiver_manager.derive_spend_key(path1).unwrap();
    receiver_manager.derive_spend_key(path2).unwrap();

    // Export cache
    let snapshot = receiver_manager.export_cache(wallet_id).unwrap();
    assert_eq!(snapshot.entries.len(), 2);

    // Clear cache
    receiver_manager.clear_cache().unwrap();
    assert_eq!(receiver_manager.cache().unwrap().len(), 0);

    // Import cache
    let result = receiver_manager.import_cache(wallet_id, snapshot);
    assert!(result.is_ok());

    // Verify entries were imported
    assert_eq!(receiver_manager.cache().unwrap().len(), 2);
    assert!(receiver_manager.get_receiver_key(path1).is_ok());
    assert!(receiver_manager.get_receiver_key(path2).is_ok());

    // Verify atomic import: all valid entries accepted
    let metrics = receiver_manager.get_metrics();
    assert_eq!(metrics.import_rejects, 0);
}

#[test]
fn test_persistence_mixed_entries_rejected() {
    let mut key_manager = new_test_key_manager();
    init_key_manager_with_seed(&mut key_manager, TEST_SEED_BYTES).unwrap();

    let mut receiver_manager = ReceiverManagerImpl::new(key_manager).build().unwrap();
    let wallet_id = b"test_wallet_id";

    // Create snapshot with mixed valid and invalid entries
    let valid_path = Bip44Path::payment(0).unwrap();
    let invalid_path = Bip44Path::payment(1).unwrap();
    let invalid_key = [0u8; 31];

    // Get valid spend+view keys
    let valid_keys = receiver_manager.derive_wallet_keys(valid_path).unwrap();
    let valid_spend = valid_keys.spend_key.as_bytes().to_vec();
    let valid_view = valid_keys.view_key.as_bytes().to_vec();

    let mut snapshot = ReceiverCacheState {
        version: 3,
        entries: vec![
            (valid_path, valid_spend, valid_view),
            (invalid_path, invalid_key.to_vec(), vec![0u8; 32]),
        ],
        hmac: [0u8; 32],
    };
    let mac_key = ReceiverCacheState::mac_key(&receiver_manager.key_manager, wallet_id).unwrap();

    // Phase 7: sign() now validates canonical format - should fail with mixed entries
    let result = snapshot.sign(wallet_id, &mac_key);
    assert!(matches!(
        result,
        Err(ReceiverManagerError::InvalidReceiverCacheState(_))
    ));
}

#[test]
fn test_persistence_cap_post_import() {
    let mut key_manager = new_test_key_manager();
    init_key_manager_with_seed(&mut key_manager, TEST_SEED_BYTES).unwrap();

    // Create manager with small cache limit
    let mut receiver_manager = ReceiverManagerImpl::new(key_manager)
        .with_limit(2)
        .build()
        .unwrap();
    let wallet_id = b"test_wallet_id";

    // Create snapshot with more entries than cache limit
    let keys0 = receiver_manager
        .derive_wallet_keys(Bip44Path::payment(0).unwrap())
        .unwrap();
    let keys1 = receiver_manager
        .derive_wallet_keys(Bip44Path::payment(1).unwrap())
        .unwrap();
    let keys2 = receiver_manager
        .derive_wallet_keys(Bip44Path::payment(2).unwrap())
        .unwrap();
    receiver_manager.clear_cache().unwrap();

    let mut snapshot = ReceiverCacheState {
        version: 3,
        entries: vec![
            (
                Bip44Path::payment(0).unwrap(),
                keys0.spend_key.as_bytes().to_vec(),
                keys0.view_key.as_bytes().to_vec(),
            ),
            (
                Bip44Path::payment(1).unwrap(),
                keys1.spend_key.as_bytes().to_vec(),
                keys1.view_key.as_bytes().to_vec(),
            ),
            (
                Bip44Path::payment(2).unwrap(),
                keys2.spend_key.as_bytes().to_vec(),
                keys2.view_key.as_bytes().to_vec(),
            ),
        ],
        hmac: [0u8; 32],
    };
    let mac_key = ReceiverCacheState::mac_key(&receiver_manager.key_manager, wallet_id).unwrap();
    snapshot.sign(wallet_id, &mac_key).unwrap();

    // Import should succeed
    let result = receiver_manager.import_cache(wallet_id, snapshot);
    assert!(result.is_ok());

    // Verify cache size is capped at limit (LRU eviction)
    assert!(receiver_manager.cache().unwrap().len() <= 2);
}

#[test]
fn test_persistence_warning_logs_emitted() {
    let mut key_manager = new_test_key_manager();
    init_key_manager_with_seed(&mut key_manager, TEST_SEED_BYTES).unwrap();

    let receiver_manager = ReceiverManagerImpl::new(key_manager).build().unwrap();
    let wallet_id = b"test_wallet_id";

    // Create snapshot with invalid entries
    let invalid_path = Bip44Path::payment(0).unwrap();
    let invalid_key = [0u8; 31];

    let mut snapshot = ReceiverCacheState {
        version: 3,
        entries: vec![(invalid_path, invalid_key.to_vec(), vec![0u8; 32])],
        hmac: [0u8; 32],
    };
    let mac_key = ReceiverCacheState::mac_key(&receiver_manager.key_manager, wallet_id).unwrap();

    // Phase 7: sign() now validates canonical format - should fail immediately
    let result = snapshot.sign(wallet_id, &mac_key);
    assert!(matches!(
        result,
        Err(ReceiverManagerError::InvalidReceiverCacheState(_))
    ));
}

#[derive(Debug, Default)]
struct CountingKeyManager {
    derive_calls: AtomicU64,
}

impl KeyManager for CountingKeyManager {
    fn clear(&mut self) {}

    fn derive_key(&self, _path: &Bip44Path) -> crate::key::Result<RistrettoPublicKey> {
        self.derive_calls.fetch_add(1, Ordering::Relaxed);
        Err(crate::key::KeyManagerError::InvalidParameters)
    }

    fn get_public_key(&self, _path: &Bip44Path) -> Option<RistrettoPublicKey> {
        None
    }

    fn derive_secret_transient(
        &self,
        _path: &Bip44Path,
    ) -> crate::key::Result<zeroize::Zeroizing<RistrettoSecretKey>> {
        Err(crate::key::KeyManagerError::InvalidParameters)
    }

    fn sign(
        &self,
        _path: &Bip44Path,
        _msg: &[u8],
    ) -> crate::key::Result<z00z_crypto::KernelSignature> {
        Err(crate::key::KeyManagerError::InvalidParameters)
    }
}

#[test]
fn test_derive_rejects_pre_derivation() {
    let key_manager = CountingKeyManager::default();
    let mut receiver_manager = ReceiverManagerImpl::new(key_manager).build().unwrap();

    let invalid_path = Bip44Path::new_unchecked_for_tests(
        (44, false),
        (Z00Z_BIP44_ASSET, true),
        (0, true),
        (0, false),
        (0, false),
    );

    let err = receiver_manager.derive_spend_key(invalid_path).unwrap_err();
    assert!(matches!(err, ReceiverManagerError::InvalidPath(_)));
    assert_eq!(
        receiver_manager
            .key_manager
            .derive_calls
            .load(Ordering::Relaxed),
        0
    );
}

#[test]
fn test_derive_rejects_coin_type() {
    let key_manager = CountingKeyManager::default();
    let mut receiver_manager = ReceiverManagerImpl::new(key_manager).build().unwrap();

    let wrong_coin_path = Bip44Path::new_unchecked_for_tests(
        (44, true),
        (9999, true),
        (0, true),
        (0, false),
        (0, false),
    );

    let err = receiver_manager
        .derive_spend_key(wrong_coin_path)
        .unwrap_err();
    assert!(matches!(err, ReceiverManagerError::InvalidCoinType(9999)));
    assert_eq!(
        receiver_manager
            .key_manager
            .derive_calls
            .load(Ordering::Relaxed),
        0
    );
}

#[derive(Debug, Default)]
struct CountingOkKeyManager {
    derive_calls: AtomicU64,
}

impl CountingOkKeyManager {
    fn derive_calls(&self) -> u64 {
        self.derive_calls.load(Ordering::Relaxed)
    }
}

impl KeyManager for CountingOkKeyManager {
    fn clear(&mut self) {}

    fn derive_key(&self, path: &Bip44Path) -> crate::key::Result<RistrettoPublicKey> {
        self.derive_calls.fetch_add(1, Ordering::Relaxed);

        let path_bytes = path.to_bytes();
        let parts: [&[u8]; 1] = [&path_bytes];

        let h0 = z00z_crypto::hash::blake2b_256(
            "z00z_wallets.receiver_manager",
            "counting_ok_km_seed0",
            &parts,
        );
        let h1 = z00z_crypto::hash::blake2b_256(
            "z00z_wallets.receiver_manager",
            "counting_ok_km_seed1",
            &parts,
        );

        let mut seed = [0u8; 64];
        seed[..32].copy_from_slice(&h0);
        seed[32..].copy_from_slice(&h1);

        let sk = RistrettoSecretKey::from_uniform_bytes(&seed)
            .map_err(|_| crate::key::KeyManagerError::InvalidParameters)?;
        Ok(RistrettoPublicKey::from_secret_key(&sk))
    }

    fn get_public_key(&self, _path: &Bip44Path) -> Option<RistrettoPublicKey> {
        None
    }

    fn derive_secret_transient(
        &self,
        _path: &Bip44Path,
    ) -> crate::key::Result<zeroize::Zeroizing<RistrettoSecretKey>> {
        Err(crate::key::KeyManagerError::InvalidParameters)
    }

    fn sign(
        &self,
        _path: &Bip44Path,
        _msg: &[u8],
    ) -> crate::key::Result<z00z_crypto::KernelSignature> {
        Err(crate::key::KeyManagerError::InvalidParameters)
    }
}

#[test]
fn test_derivation_runs_on_hit() {
    let key_manager = CountingOkKeyManager::default();
    let mut receiver_manager = ReceiverManagerImpl::new(key_manager)
        .with_timing_safe_mode(true)
        .build()
        .unwrap();

    let path = Bip44Path::payment(0).unwrap();

    let pk1 = receiver_manager.derive_spend_key(path).unwrap();
    assert_eq!(receiver_manager.key_manager.derive_calls(), 2);

    let pk2 = receiver_manager.derive_spend_key(path).unwrap();
    assert_eq!(pk1, pk2);

    assert_eq!(receiver_manager.key_manager.derive_calls(), 4);

    let metrics = receiver_manager.get_metrics();
    assert_eq!(metrics.total_derivations, 2);
    assert_eq!(metrics.hits, 1);
    assert_eq!(metrics.misses, 1);
}

#[test]
fn test_mode_skips_hit_derivation() {
    let key_manager = CountingOkKeyManager::default();
    let mut receiver_manager = ReceiverManagerImpl::new(key_manager)
        .with_timing_safe_mode(false)
        .build()
        .unwrap();

    let path = Bip44Path::payment(0).unwrap();

    let pk1 = receiver_manager.derive_spend_key(path).unwrap();
    assert_eq!(receiver_manager.key_manager.derive_calls(), 2);

    let pk2 = receiver_manager.derive_spend_key(path).unwrap();
    assert_eq!(pk1, pk2);

    // Second call should hit cache and not perform derivation in fast mode.
    assert_eq!(receiver_manager.key_manager.derive_calls(), 2);

    let metrics = receiver_manager.get_metrics();
    assert_eq!(metrics.total_derivations, 2);
    assert_eq!(metrics.hits, 1);
    assert_eq!(metrics.misses, 1);
}

// Phase 26: Purge interval zero handling
#[test]
fn test_purge_interval_zero_rejected() {
    let mut key_manager = new_test_key_manager();
    init_key_manager_with_seed(&mut key_manager, TEST_SEED_BYTES).unwrap();

    let result = ReceiverManagerImpl::new(key_manager)
        .with_purge_interval(Duration::ZERO)
        .build();

    assert!(matches!(
        result,
        Err(ReceiverManagerError::InvalidPurgeInterval)
    ));
}
#[test]
fn test_interval_below_min_rejected() {
    let mut key_manager = new_test_key_manager();
    init_key_manager_with_seed(&mut key_manager, TEST_SEED_BYTES).unwrap();

    let result = ReceiverManagerImpl::new(key_manager)
        .with_purge_interval(Duration::from_millis(500))
        .build();

    assert!(matches!(
        result,
        Err(ReceiverManagerError::InvalidPurgeInterval)
    ));
}

#[test]
#[should_panic(expected = "purge_interval must be >= 1 seconds")]
fn test_purge_interval_zero_panics() {
    let mut key_manager = new_test_key_manager();
    init_key_manager_with_seed(&mut key_manager, TEST_SEED_BYTES).unwrap();

    let mut mgr = ReceiverManagerImpl::new(key_manager).build().unwrap();
    mgr.set_purge_interval(Duration::ZERO);
}

#[test]
#[should_panic(expected = "purge_interval must be >= 1 seconds")]
fn test_interval_below_min_panics() {
    let mut key_manager = new_test_key_manager();
    init_key_manager_with_seed(&mut key_manager, TEST_SEED_BYTES).unwrap();

    let mut mgr = ReceiverManagerImpl::new(key_manager).build().unwrap();
    mgr.set_purge_interval(Duration::from_millis(999));
}
