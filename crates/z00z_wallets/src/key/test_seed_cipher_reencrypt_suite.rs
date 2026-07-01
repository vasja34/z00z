    #[test]
    fn test_reencrypt_success() {
        let old_pw = SafePassword::from("old_password");
        let new_pw = SafePassword::from("new_password");
        let wallet_id = b"wallet-001";
        let purpose = b"z00z_wallet_seed";
        let birthday = 12345u32;
        let chain = ChainType::Devnet;
        let seed = [42u8; 64];

        let container = CipherSeedContainer::encrypt_wallet(
            &old_pw, wallet_id, purpose, birthday, chain, &seed, None,
        )
        .unwrap();

        let new_container = container
            .re_encrypt(&old_pw, &new_pw, wallet_id, purpose, chain)
            .unwrap();

        let err = new_container
            .decrypt_wallet(&old_pw, wallet_id, purpose, chain)
            .unwrap_err();
        assert_auth_fail(&err);

        let decrypted = new_container
            .decrypt_wallet(&new_pw, wallet_id, purpose, chain)
            .unwrap();
        assert_eq!(decrypted.reveal().as_bytes(), &seed);
    }

    #[test]
    fn test_reencrypt_wrong_old_pw() {
        let correct_pw = SafePassword::from("correct");
        let wrong_pw = SafePassword::from("wrong");
        let new_pw = SafePassword::from("new_password");
        let wallet_id = b"wallet-001";
        let purpose = b"z00z_wallet_seed";
        let birthday = 12345u32;
        let chain = ChainType::Devnet;
        let seed = [42u8; 64];

        let container = CipherSeedContainer::encrypt_wallet(
            &correct_pw,
            wallet_id,
            purpose,
            birthday,
            chain,
            &seed,
            None,
        )
        .unwrap();

        let err = container
            .re_encrypt(&wrong_pw, &new_pw, wallet_id, purpose, chain)
            .unwrap_err();
        assert_auth_fail(&err);

        let decrypted = container
            .decrypt_wallet(&correct_pw, wallet_id, purpose, chain)
            .unwrap();
        assert_eq!(decrypted.reveal().as_bytes(), &seed);
    }

    #[test]
    fn test_reencrypt_preserves_metadata() {
        let old_pw = SafePassword::from("old_password");
        let new_pw = SafePassword::from("new_password");
        let wallet_id = b"wallet-001";
        let purpose = b"z00z_wallet_seed";
        let birthday = 12345u32;
        let chain = ChainType::Devnet;
        let seed = [42u8; 64];

        let kdf_params = Some(Argon2idParams::MOBILE);

        let container = CipherSeedContainer::encrypt_wallet(
            &old_pw, wallet_id, purpose, birthday, chain, &seed, kdf_params,
        )
        .unwrap();

        let new_container = container
            .re_encrypt(&old_pw, &new_pw, wallet_id, purpose, chain)
            .unwrap();

        assert_eq!(new_container.birthday, container.birthday);
        assert_eq!(new_container.kdf_params, container.kdf_params);

        let decrypted = new_container
            .decrypt_wallet(&new_pw, wallet_id, purpose, chain)
            .unwrap();
        assert_eq!(decrypted.reveal().as_bytes(), &seed);
    }

    #[test]
    fn test_different_wallet_id_fails() {
        let old_pw = SafePassword::from("old_password");
        let new_pw = SafePassword::from("new_password");
        let wallet_id1 = b"wallet-001";
        let wallet_id2 = b"wallet-002";
        let purpose = b"z00z_wallet_seed";
        let birthday = 12345u32;
        let chain = ChainType::Devnet;
        let seed = [42u8; 64];

        let container = CipherSeedContainer::encrypt_wallet(
            &old_pw, wallet_id1, purpose, birthday, chain, &seed, None,
        )
        .unwrap();

        let err = container
            .re_encrypt(&old_pw, &new_pw, wallet_id2, purpose, chain)
            .unwrap_err();
        assert_auth_fail(&err);
    }

    #[test]
    fn test_reencrypt_different_chain_fails() {
        let old_pw = SafePassword::from("old_password");
        let new_pw = SafePassword::from("new_password");
        let wallet_id = b"wallet-001";
        let purpose = b"z00z_wallet_seed";
        let birthday = 12345u32;
        let seed = [42u8; 64];

        let container = CipherSeedContainer::encrypt_wallet(
            &old_pw,
            wallet_id,
            purpose,
            birthday,
            ChainType::Devnet,
            &seed,
            None,
        )
        .unwrap();

        let err = container
            .re_encrypt(&old_pw, &new_pw, wallet_id, purpose, ChainType::Mainnet)
            .unwrap_err();
        assert_auth_fail(&err);
    }