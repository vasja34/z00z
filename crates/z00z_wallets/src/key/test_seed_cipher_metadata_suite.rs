    #[test]
    fn test_all_preset_params_valid() {
        assert!(Argon2idParams::DEFAULT.validate().is_ok());
        assert!(Argon2idParams::MOBILE.validate().is_ok());
        assert!(Argon2idParams::HIGH_SECURITY.validate().is_ok());
    }

    #[test]
    fn test_metadata_tampering_fails() {
        let password = SafePassword::from("test");
        let wallet_id = b"test-wallet";
        let purpose = b"test-purpose";
        let birthday = 12345u32;
        let chain = ChainType::Devnet;
        let seed = [42u8; 64];

        let container = CipherSeedContainer::encrypt_wallet(
            &password, wallet_id, purpose, birthday, chain, &seed, None,
        )
        .unwrap();

        let mut tampered_container = container.clone();
        tampered_container.birthday = birthday + 1;

        let err = tampered_container
            .decrypt_wallet(&password, wallet_id, purpose, chain)
            .unwrap_err();
        assert_auth_fail(&err);
    }

    #[test]
    fn test_kdf_cap_enforcement() {
        let params = Argon2idParams {
            mem_kib: 512 * 1024 + 1,
            time: 5,
            lanes: 8,
        };
        assert!(params.validate().is_err());

        let params = Argon2idParams {
            mem_kib: 128 * 1024,
            time: 11,
            lanes: 8,
        };
        assert!(params.validate().is_err());
    }

    #[test]
    fn test_aad_mismatch_error() {
        let password = SafePassword::from("test");
        let wallet_id1 = b"wallet-uuid";
        let wallet_id2 = b"different-wallet";
        let purpose = b"purpose";
        let birthday = 12345u32;
        let chain = ChainType::Devnet;
        let seed = [42u8; 64];

        let container = CipherSeedContainer::encrypt_wallet(
            &password, wallet_id1, purpose, birthday, chain, &seed, None,
        )
        .unwrap();

        let err = container
            .decrypt_wallet(&password, wallet_id2, purpose, chain)
            .unwrap_err();
        assert_auth_fail(&err);
    }

    #[test]
    fn test_chain_mismatch_rejected() {
        let password = SafePassword::from("test");
        let wallet_id = b"wallet-uuid";
        let purpose = b"purpose";
        let birthday = 12345u32;
        let seed = [42u8; 64];

        let container = CipherSeedContainer::encrypt_wallet(
            &password,
            wallet_id,
            purpose,
            birthday,
            ChainType::Devnet,
            &seed,
            None,
        )
        .unwrap();

        let err = container
            .decrypt_wallet(&password, wallet_id, purpose, ChainType::Mainnet)
            .unwrap_err();
        assert_auth_fail(&err);
    }

    #[test]
    fn test_aad_length_bounds() {
        let chain = ChainType::Devnet;
        let birthday = 1u32;
        let purpose = b"p";

        let wallet_id_0: [u8; 0] = [];
        assert!(CipherSeedContainer::build_aad(&wallet_id_0, purpose, birthday, chain).is_ok());

        let wallet_id_1 = [1u8; 1];
        assert!(CipherSeedContainer::build_aad(&wallet_id_1, purpose, birthday, chain).is_ok());

        let wallet_id_255 = vec![1u8; 255];
        assert!(CipherSeedContainer::build_aad(&wallet_id_255, purpose, birthday, chain).is_ok());
    }

    #[test]
    fn test_wallet_id_too_long() {
        let chain = ChainType::Devnet;
        let birthday = 1u32;
        let wallet_id = vec![1u8; 256];
        let purpose = b"p";

        let err = CipherSeedContainer::build_aad(&wallet_id, purpose, birthday, chain).unwrap_err();
        assert!(matches!(
            err,
            CipherSeedError::InputTooLong {
                field: "wallet_id",
                ..
            }
        ));
    }

    #[test]
    fn test_purpose_too_long() {
        let chain = ChainType::Devnet;
        let birthday = 1u32;
        let wallet_id = b"wallet";
        let purpose = vec![1u8; 256];

        let err = CipherSeedContainer::build_aad(wallet_id, &purpose, birthday, chain).unwrap_err();
        assert!(matches!(
            err,
            CipherSeedError::InputTooLong {
                field: "purpose",
                ..
            }
        ));
    }

    #[test]
    fn test_payload_tampering_fails() {
        let password = SafePassword::from("test");
        let wallet_id = b"test-wallet";
        let purpose = b"test-purpose";
        let birthday = 12345u32;
        let chain = ChainType::Devnet;
        let seed = [42u8; 64];

        let container = CipherSeedContainer::encrypt_wallet(
            &password, wallet_id, purpose, birthday, chain, &seed, None,
        )
        .unwrap();

        let mut tampered_container = container.clone();
        tampered_container.ciphertext[0] ^= 0x01;

        let err = tampered_container
            .decrypt_wallet(&password, wallet_id, purpose, chain)
            .unwrap_err();
        assert_auth_fail(&err);
    }

    #[test]
    fn test_auth_fail_uniform() {
        let password = SafePassword::from("pw");
        let wrong_password = SafePassword::from("wrong");
        let wallet_id = b"wallet-uuid";
        let wallet_id2 = b"wallet-uuid-2";
        let purpose = b"purpose";
        let birthday = 12345u32;
        let chain = ChainType::Devnet;
        let seed = [42u8; 64];

        let container = CipherSeedContainer::encrypt_wallet(
            &password, wallet_id, purpose, birthday, chain, &seed, None,
        )
        .unwrap();

        let err_pw = container
            .decrypt_wallet(&wrong_password, wallet_id, purpose, chain)
            .unwrap_err();

        let mut tampered = container.clone();
        tampered.ciphertext[0] ^= 0x01;
        let err_ct = tampered
            .decrypt_wallet(&password, wallet_id, purpose, chain)
            .unwrap_err();

        let err_aad = container
            .decrypt_wallet(&password, wallet_id2, purpose, chain)
            .unwrap_err();

        assert_auth_fail(&err_pw);
        assert_auth_fail(&err_ct);
        assert_auth_fail(&err_aad);
        assert_eq!(err_pw.to_string(), err_ct.to_string());
        assert_eq!(err_ct.to_string(), err_aad.to_string());
    }

    #[test]
    fn test_payload_metadata_extraction() {
        let password = SafePassword::from("test");
        let wallet_id = b"test-wallet";
        let purpose = b"test-purpose";
        let birthday = 12345u32;
        let chain = ChainType::Devnet;
        let seed = [42u8; 64];

        let container = CipherSeedContainer::encrypt_wallet(
            &password, wallet_id, purpose, birthday, chain, &seed, None,
        )
        .unwrap();

        assert_eq!(container.birthday, birthday);

        #[cfg(feature = "test-params-fast")]
        let expected_kdf_params = Argon2idParams::TEST_FAST;
        #[cfg(not(feature = "test-params-fast"))]
        let expected_kdf_params = Argon2idParams::DEFAULT;

        assert_eq!(container.kdf_params, expected_kdf_params);

        let decrypted = container
            .decrypt_wallet(&password, wallet_id, purpose, chain)
            .unwrap();
        assert_eq!(decrypted.reveal().as_bytes(), &seed);
    }

    #[test]
    fn test_kdf_params_exceeding_caps() {
        let password = SafePassword::from("test");
        let wallet_id = b"test-wallet";
        let purpose = b"test-purpose";
        let birthday = 1u32;
        let chain = ChainType::Devnet;
        let seed = [42u8; 64];

        let container = CipherSeedContainer::encrypt_wallet(
            &password, wallet_id, purpose, birthday, chain, &seed, None,
        )
        .unwrap();

        let mut bad_container = container.clone();
        bad_container.kdf_params.mem_kib = 512 * 1024 + 1;

        let err = bad_container
            .decrypt_wallet(&password, wallet_id, purpose, chain)
            .unwrap_err();
        assert!(matches!(err, CipherSeedError::InvalidKdfParams));

        let mut bad_container = container.clone();
        bad_container.kdf_params.time = 11;

        let err = bad_container
            .decrypt_wallet(&password, wallet_id, purpose, chain)
            .unwrap_err();
        assert!(matches!(err, CipherSeedError::InvalidKdfParams));
    }

    #[test]
    fn test_aad_deterministic_across_platforms() {
        let wallet_id = b"test-wallet-uuid";
        let purpose = b"seed-purpose";
        let birthday = 12345u32;
        let chain = ChainType::Devnet;

        let aad1 = CipherSeedContainer::build_aad(wallet_id, purpose, birthday, chain).unwrap();
        let aad2 = CipherSeedContainer::build_aad(wallet_id, purpose, birthday, chain).unwrap();
        let aad3 = CipherSeedContainer::build_aad(wallet_id, purpose, birthday, chain).unwrap();

        assert_eq!(aad1, aad2, "AAD must be deterministic");
        assert_eq!(aad2, aad3, "AAD must be deterministic");

        assert_eq!(
            aad1.len(),
            1 + 4 + 1 + chain.as_str().len() + 1 + wallet_id.len() + 1 + purpose.len() + 32
        );

        assert_eq!(aad1[0], CipherSeedContainer::AAD_VERSION);

        let birthday_bytes: [u8; 4] = aad1[1..5].try_into().unwrap();
        assert_eq!(u32::from_le_bytes(birthday_bytes), birthday);

        let chain_len = aad1[5] as usize;
        assert_eq!(chain_len, chain.as_str().len());
        let chain_start = 6;
        let chain_end = chain_start + chain_len;
        assert_eq!(&aad1[chain_start..chain_end], chain.as_str().as_bytes());

        let wallet_id_len = aad1[chain_end] as usize;
        assert_eq!(wallet_id_len, wallet_id.len());
        let wallet_id_start = chain_end + 1;
        let wallet_id_end = wallet_id_start + wallet_id_len;
        assert_eq!(&aad1[wallet_id_start..wallet_id_end], wallet_id);

        let purpose_len = aad1[wallet_id_end] as usize;
        assert_eq!(purpose_len, purpose.len());
        let purpose_start = wallet_id_end + 1;
        let purpose_end = purpose_start + purpose_len;
        assert_eq!(&aad1[purpose_start..purpose_end], purpose);

        assert_eq!(aad1.len(), purpose_end + 32);
    }

    #[test]
    fn test_aad_version_uniqueness() {
        let wallet_id = b"test-wallet-uuid";
        let purpose = b"seed-purpose";
        let birthday = 12345u32;
        let chain = ChainType::Devnet;

        let aad_1 = CipherSeedContainer::build_aad(wallet_id, purpose, birthday, chain).unwrap();
        let aad_2 = build_aad_ver(2u8, wallet_id, purpose, birthday, chain);

        assert_ne!(aad_1, aad_2);

        let tag1 = &aad_1[aad_1.len() - 32..];
        let tag2 = &aad_2[aad_2.len() - 32..];
        assert_ne!(tag1, tag2);
    }

    #[test]
    fn test_aad_fixed_test_vectors() {
        let wallet_id = b"wallet-001";
        let purpose = b"payment-key";
        let birthday = 1000u32;
        let chain = ChainType::Devnet;

        let aad = CipherSeedContainer::build_aad(wallet_id, purpose, birthday, chain).unwrap();

        assert_eq!(aad.len(), 1 + 4 + 1 + 6 + 1 + 10 + 1 + 11 + 32);
        assert_eq!(aad[0], CipherSeedContainer::AAD_VERSION);

        let birthday_bytes: [u8; 4] = aad[1..5].try_into().unwrap();
        assert_eq!(u32::from_le_bytes(birthday_bytes), 1000);

        assert_eq!(aad[5], 6);
        assert_eq!(&aad[6..12], b"devnet");
        assert_eq!(aad[12], 10);
        assert_eq!(&aad[13..23], b"wallet-001");
        assert_eq!(aad[23], 11);
        assert_eq!(&aad[24..35], b"payment-key");
        assert_eq!(aad.len(), 35 + 32);

        let domain_tag = &aad[35..];
        assert!(!domain_tag.iter().all(|&b| b == 0));
    }