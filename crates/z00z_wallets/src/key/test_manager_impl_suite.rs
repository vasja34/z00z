    #[test]
    fn test_secret_key_ct_eq() {
        let mut bytes1 = [1u8; 64];
        let key1 = RistrettoSecretKey::from_uniform_bytes(&bytes1).unwrap();
        let key2 = key1.clone();

        bytes1[0] = 2;
        let key3 = RistrettoSecretKey::from_uniform_bytes(&bytes1).unwrap();

        assert!(key1.ct_eq(&key2).unwrap_u8() != 0);
        assert!(key1.ct_eq(&key3).unwrap_u8() == 0);
    }

    #[test]
    fn test_identity_pubkey_rejection() {
        let km = KeyManagerImpl::new();
        let identity = RistrettoPublicKey::default();
        let err = km.verify_key(&identity).unwrap_err();
        assert!(matches!(err, KeyManagerError::InvalidPublicKey));
    }
    #[test]
    fn test_cache_corrupt() {
        let mut km = KeyManagerImpl::new();
        let seed_bytes = valid_seed_bytes();
        km.init_from_seed(seed_bytes.as_slice(), ChainType::Devnet)
            .unwrap();

        let path = Bip44Path::new_z00z(0, 0, 0).unwrap();
        let _ = km.derive_key(&path).unwrap();

        let wrong_sk = RistrettoSecretKey::from_uniform_bytes(&[3u8; 64]).unwrap();
        let wrong_pk = RistrettoPublicKey::from_secret_key(&wrong_sk);
        km.insert_cache(path, wrong_pk).unwrap();

        let err = km.validate_state().unwrap_err();
        assert!(matches!(err, KeyManagerError::CacheCorrupted));
    }
    #[test]
    fn test_cache_identity_pubkey() {
        let mut km = KeyManagerImpl::new();
        let seed_bytes = valid_seed_bytes();
        km.init_from_seed(seed_bytes.as_slice(), ChainType::Devnet)
            .unwrap();

        let path = Bip44Path::new_z00z(0, 0, 0).unwrap();
        let _ = km.derive_key(&path).unwrap();

        let identity = RistrettoPublicKey::default();
        km.insert_cache(path, identity).unwrap();

        let err = km.validate_state().unwrap_err();
        assert!(matches!(err, KeyManagerError::CacheCorrupted));
    }
    #[test]
    fn test_cache_spot_check_triggers() {
        let mut km = KeyManagerImpl::new();
        let seed_bytes = valid_seed_bytes();
        km.init_from_seed(seed_bytes.as_slice(), ChainType::Devnet)
            .unwrap();

        let path = Bip44Path::new_z00z(0, 0, 0).unwrap();
        let _ = km.derive_key(&path).unwrap();

        let wrong_sk = RistrettoSecretKey::from_uniform_bytes(&[3u8; 64]).unwrap();
        let wrong_pk = RistrettoPublicKey::from_secret_key(&wrong_sk);
        km.insert_cache(path, wrong_pk).unwrap();

        km.derive_count
            .store(CACHE_SPOT_CHECK_FREQUENCY - 1, Ordering::Relaxed);

        let err = km.derive_key(&path).unwrap_err();
        assert!(matches!(err, KeyManagerError::CacheCorrupted));

        let pk = km.derive_key(&path).unwrap();
        assert!(km.verify_key(&pk).is_ok());
    }
    #[test]
    fn test_cached_identity_evicted() {
        let mut km = KeyManagerImpl::new();
        let seed_bytes = valid_seed_bytes();
        km.init_from_seed(seed_bytes.as_slice(), ChainType::Devnet)
            .unwrap();

        let path = Bip44Path::new_z00z(0, 0, 0).unwrap();
        let identity = RistrettoPublicKey::default();
        km.insert_cache(path, identity).unwrap();

        let pk = km.derive_key(&path).unwrap();
        assert!(km.verify_key(&pk).is_ok());
    }
    #[test]
    fn test_state_chain_mismatch() {
        let mut km = KeyManagerImpl::new();
        let seed_bytes = valid_seed_bytes();
        km.init_from_seed(seed_bytes.as_slice(), ChainType::Devnet)
            .unwrap();

        km.chain = ChainType::Mainnet;

        let err = km.validate_state().unwrap_err();
        assert!(matches!(err, KeyManagerError::StateCorrupted));
    }
    #[test]
    fn test_state_gap_corrupt() {
        let mut km = KeyManagerImpl::new();
        let seed_bytes = valid_seed_bytes();
        km.init_from_seed(seed_bytes.as_slice(), ChainType::Devnet)
            .unwrap();

        km.gap_external.store(5, Ordering::Release);
        km.last_used_ext.store(10, Ordering::Release);

        let err = km.validate_state().unwrap_err();
        assert!(matches!(err, KeyManagerError::StateCorrupted));
    }
    #[test]
    fn test_external_fails_gap_corrupt() {
        let mut km = KeyManagerImpl::new();
        let seed_bytes = valid_seed_bytes();
        km.init_from_seed(seed_bytes.as_slice(), ChainType::Devnet)
            .unwrap();

        km.gap_external.store(5, Ordering::Release);
        km.last_used_ext.store(10, Ordering::Release);

        let err = km.next_external().unwrap_err();
        assert!(matches!(err, KeyManagerError::StateCorrupted));
    }

    #[test]
    fn test_clear_zeroizes_seed() {
        reset_seed_zeroized();

        let mut km = KeyManagerImpl::new();
        let seed_bytes = valid_seed_bytes();
        km.init_from_seed(seed_bytes.as_slice(), ChainType::Devnet)
            .unwrap();

        km.clear();

        assert!(seed_zeroized());
    }

    #[test]
    fn test_cache_ttl_expiration() {
        let time_provider = MockTimeProvider::system_now();
        let mut km = KeyManagerImpl::new_with_observability(
            Arc::new(NoopLogger),
            Arc::new(NoopMetrics),
            Arc::new(time_provider.clone()),
            SystemRngProvider,
        );

        let seed_bytes = valid_seed_bytes();
        km.init_from_seed(seed_bytes.as_slice(), ChainType::Devnet)
            .unwrap();

        let path = Bip44Path::new_z00z(0, 0, 0).unwrap();
        let key1 = km.derive_key(&path).unwrap();

        assert_eq!(km.cache_size(), 1);

        time_provider.advance_by(Duration::from_secs(DERIVED_KEY_TTL_SECONDS + 1));
        assert!(km.get_public_key(&path).is_none());
        let initial_count = km.derivation_count();

        let key2 = km.derive_key(&path).unwrap();
        assert_eq!(key1, key2);
        {
            assert_eq!(km.derivation_count(), initial_count + 1);
        }
    }

    #[test]
    fn test_cache_ttl_validation() {
        let time_provider = MockTimeProvider::system_now();
        let mut km = KeyManagerImpl::new_with_observability(
            Arc::new(NoopLogger),
            Arc::new(NoopMetrics),
            Arc::new(time_provider.clone()),
            SystemRngProvider,
        );

        let seed_bytes = valid_seed_bytes();
        km.init_from_seed(seed_bytes.as_slice(), ChainType::Devnet)
            .unwrap();

        let path = Bip44Path::new_z00z(0, 0, 0).unwrap();
        let key1 = km.derive_key(&path).unwrap();

        assert!(km.get_public_key(&path).is_some());
        assert_eq!(km.get_public_key(&path).unwrap(), key1);

        time_provider.advance_by(Duration::from_secs(DERIVED_KEY_TTL_SECONDS - 1));
        assert!(km.get_public_key(&path).is_some());
        assert_eq!(km.get_public_key(&path).unwrap(), key1);

        time_provider.advance_by(Duration::from_secs(DERIVED_KEY_TTL_SECONDS + 1));
        assert!(km.get_public_key(&path).is_none());
    }

    #[test]
    fn test_cache_ttl_redo() {
        let time_provider = MockTimeProvider::system_now();
        let mut km = KeyManagerImpl::new_with_observability(
            Arc::new(NoopLogger),
            Arc::new(NoopMetrics),
            Arc::new(time_provider.clone()),
            SystemRngProvider,
        );

        let seed_bytes = valid_seed_bytes();
        km.init_from_seed(seed_bytes.as_slice(), ChainType::Devnet)
            .unwrap();

        let path = Bip44Path::new_z00z(0, 0, 0).unwrap();
        let key1 = km.derive_key(&path).unwrap();

        time_provider.advance_by(Duration::from_secs(DERIVED_KEY_TTL_SECONDS + 1));
        let key2 = km.derive_key(&path).unwrap();

        assert_eq!(key1, key2);
        assert!(km.get_public_key(&path).is_some());
        assert_eq!(km.get_public_key(&path).unwrap(), key1);
    }

    #[test]
    fn test_cache_ttl_skew() {
        let time_provider = MockTimeProvider::system_now();
        let mut km = KeyManagerImpl::new_with_observability(
            Arc::new(NoopLogger),
            Arc::new(NoopMetrics),
            Arc::new(time_provider.clone()),
            SystemRngProvider,
        );

        let seed_bytes = valid_seed_bytes();
        km.init_from_seed(seed_bytes.as_slice(), ChainType::Devnet)
            .unwrap();

        let path = Bip44Path::new_z00z(0, 0, 0).unwrap();
        let key1 = km.derive_key(&path).unwrap();

        time_provider.advance_by(Duration::from_secs(3600));
        let _ = km.derive_key(&path).unwrap();
        time_provider.advance_by(Duration::from_secs(3600));
        time_provider.advance_by(Duration::from_secs(3600));

        assert!(km.get_public_key(&path).is_none());

        let key2 = km.derive_key(&path).unwrap();
        assert_eq!(key1, key2);
    }

    #[test]
    fn test_cache_ttl_concurrent_expiration() {
        use std::sync::Arc;
        use std::thread;

        let time_provider = MockTimeProvider::system_now();
        let mut km = KeyManagerImpl::new_with_observability(
            Arc::new(NoopLogger),
            Arc::new(NoopMetrics),
            Arc::new(time_provider.clone()),
            SystemRngProvider,
        );

        let seed_bytes = valid_seed_bytes();
        km.init_from_seed(seed_bytes.as_slice(), ChainType::Devnet)
            .unwrap();

        let path = Bip44Path::new_z00z(0, 0, 0).unwrap();
        let key1 = km.derive_key(&path).unwrap();

        time_provider.advance_by(Duration::from_secs(DERIVED_KEY_TTL_SECONDS + 1));

        let km = Arc::new(km);
        let handles: Vec<_> = (0..10)
            .map(|_| {
                let km = Arc::clone(&km);
                thread::spawn(move || km.derive_key(&path))
            })
            .collect();

        let results: Vec<_> = handles.into_iter().map(|h| h.join().unwrap()).collect();

        assert!(results
            .windows(2)
            .all(|w| w[0].as_ref().unwrap() == w[1].as_ref().unwrap()));
        assert_eq!(results[0].as_ref().unwrap(), &key1);
        {
            assert!(km.derivation_count() <= 2);
        }
    }

    #[test]
    fn test_cache_ttl_size_limit() {
        let time_provider = MockTimeProvider::system_now();
        let mut km = KeyManagerImpl::new_with_observability(
            Arc::new(NoopLogger),
            Arc::new(NoopMetrics),
            Arc::new(time_provider.clone()),
            SystemRngProvider,
        );

        let seed_bytes = valid_seed_bytes();
        km.init_from_seed(seed_bytes.as_slice(), ChainType::Devnet)
            .unwrap();

        for i in 0..(MAX_DERIVED_PUBKEY_CACHE + 10) {
            let path = Bip44Path::new_z00z(0, 0, i as u32).unwrap();
            km.derive_key(&path).unwrap();
        }

        assert_eq!(km.cache_size(), MAX_DERIVED_PUBKEY_CACHE);

        for i in 0..(MAX_DERIVED_PUBKEY_CACHE + 10) {
            let path = Bip44Path::new_z00z(0, 0, i as u32).unwrap();
            assert!(km.derive_key(&path).is_ok());
        }
    }

    #[test]
    fn test_gap_limit_external_enforced() {
        let mut km = KeyManagerImpl::new();
        let seed_bytes = valid_seed_bytes();
        km.init_from_seed(seed_bytes.as_slice(), ChainType::Devnet)
            .unwrap();

        for _ in 0..BIP44_GAP_LIMIT {
            let index = km.next_external().unwrap();
            let path = Bip44Path::new_z00z(0, 0, index).unwrap();
            assert!(km.derive_key(&path).is_ok());
        }

        let result = km.next_external();
        assert!(matches!(
            result,
            Err(KeyManagerError::GapLimitExceeded { gap }) if gap == BIP44_GAP_LIMIT
        ));
    }

    #[test]
    fn test_gap_limit_ext_usage() {
        let mut km = KeyManagerImpl::new();
        let seed_bytes = valid_seed_bytes();
        km.init_from_seed(seed_bytes.as_slice(), ChainType::Devnet)
            .unwrap();

        for _ in 0..BIP44_GAP_LIMIT {
            let index = km.next_external().unwrap();
            let path = Bip44Path::new_z00z(0, 0, index).unwrap();
            assert!(km.derive_key(&path).is_ok());
        }

        km.mark_external_used(10);

        for _ in 0..10 {
            let index = km.next_external().unwrap();
            let path = Bip44Path::new_z00z(0, 0, index).unwrap();
            assert!(km.derive_key(&path).is_ok());
        }
    }

    #[test]
    fn test_gap_limit_internal_enforced() {
        let mut km = KeyManagerImpl::new();
        let seed_bytes = valid_seed_bytes();
        km.init_from_seed(seed_bytes.as_slice(), ChainType::Devnet)
            .unwrap();

        for _ in 0..BIP44_GAP_LIMIT {
            let index = km.next_internal().unwrap();
            let path = Bip44Path::new_z00z(0, 1, index).unwrap();
            assert!(km.derive_key(&path).is_ok());
        }

        let result = km.next_internal();
        assert!(matches!(
            result,
            Err(KeyManagerError::GapLimitExceeded { gap }) if gap == BIP44_GAP_LIMIT
        ));
    }

    #[test]
    fn test_gap_limit_int_usage() {
        let mut km = KeyManagerImpl::new();
        let seed_bytes = valid_seed_bytes();
        km.init_from_seed(seed_bytes.as_slice(), ChainType::Devnet)
            .unwrap();

        for _ in 0..BIP44_GAP_LIMIT {
            let index = km.next_internal().unwrap();
            let path = Bip44Path::new_z00z(0, 1, index).unwrap();
            assert!(km.derive_key(&path).is_ok());
        }

        km.mark_internal_used(10);

        for _ in 0..10 {
            let index = km.next_internal().unwrap();
            let path = Bip44Path::new_z00z(0, 1, index).unwrap();
            assert!(km.derive_key(&path).is_ok());
        }
    }

    #[test]
    fn test_gap_limit_split() {
        let mut km = KeyManagerImpl::new();
        let seed_bytes = valid_seed_bytes();
        km.init_from_seed(seed_bytes.as_slice(), ChainType::Devnet)
            .unwrap();

        for _ in 0..BIP44_GAP_LIMIT {
            let index = km.next_external().unwrap();
            let path = Bip44Path::new_z00z(0, 0, index).unwrap();
            assert!(km.derive_key(&path).is_ok());
        }

        for _ in 0..BIP44_GAP_LIMIT {
            let index = km.next_internal().unwrap();
            let path = Bip44Path::new_z00z(0, 1, index).unwrap();
            assert!(km.derive_key(&path).is_ok());
        }

        assert!(km.next_external().is_err());
        assert!(km.next_internal().is_err());
    }

    #[test]
    fn test_sign_nonce_deterministic() {
        let mut km = KeyManagerImpl::new();
        let seed_bytes = valid_seed_bytes();
        km.init_from_seed(seed_bytes.as_slice(), ChainType::Devnet)
            .unwrap();

        let path = Bip44Path::new_z00z(0, 0, 0).unwrap();
        let msg = b"hello";

        let sig1 = km.sign(&path, msg).unwrap();
        let sig2 = km.sign(&path, msg).unwrap();

        assert_eq!(sig1.get_public_nonce(), sig2.get_public_nonce());
        assert_eq!(
            sig1.get_signature().as_bytes(),
            sig2.get_signature().as_bytes()
        );

        let sig3 = km.sign(&path, b"hello2").unwrap();
        assert_ne!(sig1.get_public_nonce(), sig3.get_public_nonce());
    }

    #[test]
    fn test_sign_zero_nonce_rejected() {
        let seed = [0u8; 64];
        let result = KeyManagerImpl::<SystemRngProvider>::nonce_from_seed(&seed);
        assert!(matches!(result, Err(KeyManagerError::SignatureFailed)));
    }

    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn test_gap_limit_concurrent_access() {
        use std::sync::Arc;
        use std::thread;

        let mut km = KeyManagerImpl::new();
        let seed_bytes = valid_seed_bytes();
        km.init_from_seed(seed_bytes.as_slice(), ChainType::Devnet)
            .unwrap();

        let km = Arc::new(km);
        let thread_count = 64usize;

        let handles: Vec<_> = (0..thread_count)
            .map(|_| {
                let km = Arc::clone(&km);
                thread::spawn(move || km.next_external())
            })
            .collect();

        let mut ok_indices = Vec::new();
        let mut err_count = 0usize;

        for handle in handles {
            match handle.join().unwrap() {
                Ok(idx) => ok_indices.push(idx),
                Err(KeyManagerError::GapLimitExceeded { .. }) => err_count += 1,
                Err(other) => panic!("unexpected error: {other:?}"),
            }
        }

        assert!(ok_indices.len() <= BIP44_GAP_LIMIT as usize);
        assert_eq!(ok_indices.len() + err_count, thread_count);

        for idx in &ok_indices {
            assert!(*idx < BIP44_GAP_LIMIT);
        }

        let uniq: std::collections::HashSet<u32> = ok_indices.into_iter().collect();
        assert!(uniq.len() <= BIP44_GAP_LIMIT as usize);
    }
    #[test]
    fn test_cache_ttl_no_race() {
        use std::sync::Arc;
        use std::thread;

        let time = Arc::new(MockTimeProvider::default());
        let time_provider: Arc<dyn TimeProvider> = time.clone();
        let mut km = KeyManagerImpl::new_with_observability(
            Arc::new(NoopLogger),
            Arc::new(NoopMetrics),
            time_provider,
            SystemRngProvider,
        );

        let seed_bytes = valid_seed_bytes();
        km.init_from_seed(seed_bytes.as_slice(), ChainType::Devnet)
            .unwrap();

        let km = Arc::new(km);
        let path = Bip44Path::new_z00z(0, 0, 0).unwrap();

        assert!(km.derive_key(&path).is_ok());
        let before = km.derivation_count();

        time.advance_by(Duration::from_secs(DERIVED_KEY_TTL_SECONDS + 1));

        let threads = 8usize;
        let handles: Vec<_> = (0..threads)
            .map(|_| {
                let km = Arc::clone(&km);
                thread::spawn(move || km.derive_key(&path))
            })
            .collect();

        for handle in handles {
            handle.join().unwrap().unwrap();
        }

        let after = km.derivation_count();
        assert_eq!(after, before + 1);
    }
    #[test]
    fn test_cache_expires_at_boundary() {
        let time = Arc::new(MockTimeProvider::default());
        let time_provider: Arc<dyn TimeProvider> = time.clone();
        let mut km = KeyManagerImpl::new_with_observability(
            Arc::new(NoopLogger),
            Arc::new(NoopMetrics),
            time_provider,
            SystemRngProvider,
        );

        let seed_bytes = valid_seed_bytes();
        km.init_from_seed(seed_bytes.as_slice(), ChainType::Devnet)
            .unwrap();

        let path = Bip44Path::new_z00z(0, 0, 0).unwrap();

        assert!(km.derive_key(&path).is_ok());
        let before = km.derivation_count();

        time.advance_by(Duration::from_secs(DERIVED_KEY_TTL_SECONDS));
        assert!(km.derive_key(&path).is_ok());

        let after = km.derivation_count();
        assert_eq!(after, before + 1);
    }