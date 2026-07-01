    #[test]
    fn test_init_weak_seed_rejected() {
        let mut km = KeyManagerImpl::new();
        let weak = [0x55u8; 64];

        let result = km.init_from_seed(&weak, ChainType::Devnet);

        assert!(matches!(result, Err(KeyManagerError::WeakEntropy(_))));
    }

    #[test]
    fn test_init_good_seed_accepted() {
        let mut km = KeyManagerImpl::new();
        let seed = valid_seed_bytes();

        km.init_from_seed(&seed, ChainType::Devnet).unwrap();

        assert!(km.bip44_manager.is_some());
        assert_eq!(km.chain, ChainType::Devnet);
    }

    #[test]
    fn test_init_all_zeros_rejected() {
        let mut km = KeyManagerImpl::new();
        let weak = [0u8; 64];

        let result = km.init_from_seed(&weak, ChainType::Devnet);

        assert!(matches!(result, Err(KeyManagerError::WeakEntropy(_))));
    }

    #[test]
    fn test_init_all_ff_rejected() {
        let mut km = KeyManagerImpl::new();
        let weak = [0xFFu8; 64];

        let result = km.init_from_seed(&weak, ChainType::Devnet);

        assert!(matches!(result, Err(KeyManagerError::WeakEntropy(_))));
    }

    #[test]
    fn test_long_zero_run_rejected() {
        let mut weak = valid_seed_bytes();
        weak[12..28].fill(0);
        let mut km = KeyManagerImpl::new();

        let result = km.init_from_seed(&weak, ChainType::Devnet);

        assert!(matches!(result, Err(KeyManagerError::WeakEntropy(_))));
    }

    #[test]
    #[cfg(not(target_arch = "wasm32"))]
    fn test_change_password_success() {
        let password_old = SafePassword::from("old-password");
        let password_new = SafePassword::from("new-password");
        let wallet_id = b"wallet-123";
        let purpose = b"main";
        let chain = ChainType::Devnet;
        let seed_bytes = valid_seed_bytes();

        let container = CipherSeedContainer::encrypt_wallet(
            &password_old,
            wallet_id,
            purpose,
            1u32,
            chain,
            &seed_bytes,
            None,
        )
        .expect("encrypt");

        let mut km = KeyManagerImpl::new();
        km.init_from_encrypted_seed(container, &password_old, wallet_id, purpose, chain)
            .expect("init");

        km.change_password(&password_old, &password_new, wallet_id, purpose)
            .expect("change password");

        let saved = km.encrypted_seed.clone().expect("encrypted seed saved");
        saved.decrypt_wallet(&password_new, wallet_id, purpose, chain)
            .expect("new password works");
        assert!(saved
            .decrypt_wallet(&password_old, wallet_id, purpose, chain)
            .is_err());
    }

    #[test]
    #[cfg(not(target_arch = "wasm32"))]
    fn test_password_wrong_old_pw() {
        let password_old = SafePassword::from("old-password");
        let password_wrong = SafePassword::from("wrong-password");
        let password_new = SafePassword::from("new-password");
        let wallet_id = b"wallet-123";
        let purpose = b"main";
        let chain = ChainType::Devnet;
        let seed_bytes = valid_seed_bytes();

        let container = CipherSeedContainer::encrypt_wallet(
            &password_old,
            wallet_id,
            purpose,
            1u32,
            chain,
            &seed_bytes,
            None,
        )
        .expect("encrypt");

        let mut km = KeyManagerImpl::new();
        km.init_from_encrypted_seed(container, &password_old, wallet_id, purpose, chain)
            .expect("init");

        let err = km
            .change_password(&password_wrong, &password_new, wallet_id, purpose)
            .expect_err("change should fail");

        assert!(matches!(err, KeyManagerError::AuthenticationFailed));
    }

    #[test]
    fn test_change_password_not_initialized() {
        let mut km = KeyManagerImpl::new();
        let password_old = SafePassword::from("old-password");
        let password_new = SafePassword::from("new-password");

        let err = km
            .change_password(&password_old, &password_new, b"wallet-123", b"main")
            .expect_err("must fail without encrypted seed");

        assert!(matches!(err, KeyManagerError::NotInitialized));
    }

    #[test]
    fn test_change_password_clears_cache() {
        let password_old = SafePassword::from("old-password");
        let password_new = SafePassword::from("new-password");
        let wallet_id = b"wallet-123";
        let purpose = b"main";
        let chain = ChainType::Devnet;
        let seed_bytes = valid_seed_bytes();

        let container = CipherSeedContainer::encrypt_wallet(
            &password_old,
            wallet_id,
            purpose,
            1u32,
            chain,
            &seed_bytes,
            None,
        )
        .expect("encrypt");

        let mut km = KeyManagerImpl::new();
        km.init_from_encrypted_seed(container, &password_old, wallet_id, purpose, chain)
            .expect("init");

        let path = Bip44Path::new_z00z(0, 0, 0).unwrap();
        km.derive_key(&path).expect("cache derive");
        assert_eq!(km.cache_size(), 1);

        km.change_password(&password_old, &password_new, wallet_id, purpose)
            .expect("change password");

        assert_eq!(km.cache_size(), 0);
    }

    #[test]
    #[cfg(not(target_arch = "wasm32"))]
    fn test_preserves_encrypted_seed_state() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("wallet.wlt");
        let password_old = SafePassword::from("old-password");
        let password_new = SafePassword::from("new-password");
        let wallet_id = PersistWalletId("wallet-storage-test".to_string());
        let identity = WalletIdentity {
            network: "persisted-local".to_string(),
            chain: ChainType::Devnet.to_string(),
        };

        crate::db::create_wallet_store(
            &path,
            &wallet_id,
            &password_old,
            "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon art",
            &identity,
            z00z_utils::rng::SystemRngProvider,
        )
        .expect("create storage-backed wallet");

        let mut km = KeyManagerImpl::unlock_from_storage(&path, &wallet_id, &password_old, &identity)
            .expect("unlock from storage");

        km.to_state(KeyManagerMetadata {
            wallet_id: wallet_id.0.clone(),
            created_at: 1,
            version: 1,
            label: Some("wallet-storage-test".to_string()),
        })
        .expect("persist encrypted state after unlock");

        km.change_password(
            &password_old,
            &password_new,
            wallet_id.0.as_bytes(),
            b"main",
        )
        .expect("change password after storage unlock");
    }