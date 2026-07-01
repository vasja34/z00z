    fn build_aad_ver(
        version: u8,
        wallet_id: &[u8],
        purpose: &[u8],
        birthday: u32,
        chain: ChainType,
    ) -> Vec<u8> {
        let chain_bytes = chain.as_str().as_bytes();

        let mut aad_base = Vec::with_capacity(
            1 + 4 + 1 + chain_bytes.len() + 1 + wallet_id.len() + 1 + purpose.len(),
        );
        aad_base.push(version);
        aad_base.extend_from_slice(&birthday.to_le_bytes());
        aad_base.push(chain_bytes.len() as u8);
        aad_base.extend_from_slice(chain_bytes);
        aad_base.push(wallet_id.len() as u8);
        aad_base.extend_from_slice(wallet_id);
        aad_base.push(purpose.len() as u8);
        aad_base.extend_from_slice(purpose);

        let hash = DomainHasher::<CipherSeedAadTagDomain>::new_with_label("cipher_seed_aad")
            .chain([version])
            .chain(&aad_base)
            .finalize();

        let mut aad = Vec::with_capacity(aad_base.len() + 32);
        aad.extend_from_slice(&aad_base);
        aad.extend_from_slice(&hash.as_ref()[..32]);
        aad
    }

    fn assert_auth_fail(err: &CipherSeedError) {
        assert!(
            matches!(err, CipherSeedError::AuthenticationFailed(_)),
            "Expected AuthenticationFailed but got: {:?}",
            err
        );
        assert_eq!(err.to_string(), "Decryption failed");
    }

    #[test]
    fn test_cipher_seed_bytes_roundtrip() {
        let password = SafePassword::from("test-password");
        let wallet_id = b"wallet-uuid";
        let purpose = b"purpose";
        let birthday = 12345u32;
        let chain = ChainType::Devnet;
        let seed = [42u8; 64];

        let mut container = CipherSeedContainer::encrypt_wallet(
            &password, wallet_id, purpose, birthday, chain, &seed, None,
        )
        .unwrap();

        container.version = 2;
        let err = container
            .decrypt_wallet(&password, wallet_id, purpose, chain)
            .unwrap_err();
        assert!(matches!(err, CipherSeedError::UnsupportedVersion(2)));

        let container = CipherSeedContainer::encrypt_wallet(
            &password, wallet_id, purpose, birthday, chain, &seed, None,
        )
        .unwrap();

        let bytes = container.to_bytes().unwrap();
        let decoded = CipherSeedContainer::from_bytes(&bytes).unwrap();
        assert_eq!(decoded, container);
    }

    #[test]
    fn test_ct_eq_ciphertext_len() {
        let mut a = CipherSeedContainer {
            version: CipherSeedContainer::VERSION,
            birthday: 12345,
            kdf: KdfId::Argon2id,
            kdf_params: Argon2idParams::DEFAULT,
            aead: AeadId::XChaCha20Poly1305,
            salt: [7u8; 32],
            nonce: [9u8; 24],
            ciphertext: vec![1u8; 80],
        };

        let mut b = a.clone();
        b.ciphertext.push(0u8);
        assert_eq!(a.ct_eq(&b).unwrap_u8(), 0);

        a.ciphertext.push(0u8);
        assert_eq!(a.ct_eq(&b).unwrap_u8(), 1);
    }

    #[test]
    fn test_unsupported_version_rejected() {
        let password = SafePassword::from("test-password");
        let wallet_id = b"wallet-uuid";
        let purpose = b"purpose";
        let birthday = 12345u32;
        let chain = ChainType::Devnet;
        let seed = [42u8; 64];

        let container = CipherSeedContainer::encrypt_wallet(
            &password, wallet_id, purpose, birthday, chain, &seed, None,
        )
        .unwrap();

        let mut bytes = container.to_bytes().unwrap();
        bytes[0] = 2;

        let err = CipherSeedContainer::from_bytes(&bytes).unwrap_err();
        assert!(matches!(err, CipherSeedError::UnsupportedVersion(2)));
    }

    #[test]
    fn test_cipher_seed_unknown_ids() {
        let container = CipherSeedContainer {
            version: CipherSeedContainer::VERSION,
            birthday: 12345,
            kdf: KdfId::Argon2id,
            kdf_params: Argon2idParams::DEFAULT,
            aead: AeadId::XChaCha20Poly1305,
            salt: [0u8; 32],
            nonce: [0u8; 24],
            ciphertext: vec![0u8; 64],
        };

        const KDF_OFF: usize = 1 + 4;
        const AEAD_OFF: usize = 1 + 4 + 1 + Argon2idParams::ENCODED_LEN;

        let mut bytes = container.to_bytes().unwrap();
        bytes[KDF_OFF] = 0xFF;
        let err = CipherSeedContainer::from_bytes(&bytes).unwrap_err();
        assert!(matches!(err, CipherSeedError::InvalidKdf));

        let mut bytes = container.to_bytes().unwrap();
        bytes[AEAD_OFF] = 0xFF;
        let err = CipherSeedContainer::from_bytes(&bytes).unwrap_err();
        assert!(matches!(err, CipherSeedError::InvalidAead));
    }

    #[test]
    fn test_cipher_seed_roundtrip() {
        let password = SafePassword::from("test-password");
        let wallet_id = b"wallet-uuid";
        let purpose = b"purpose";
        let birthday = 12345u32;
        let chain = ChainType::Devnet;
        let seed = [42u8; 64];

        assert_eq!(seed.len(), SeedBytes::LEN);

        let container = CipherSeedContainer::encrypt_wallet(
            &password, wallet_id, purpose, birthday, chain, &seed, None,
        )
        .unwrap();
        assert_eq!(container.version, CipherSeedContainer::VERSION);
        assert_eq!(container.birthday, birthday);

        let decrypted = container
            .decrypt_wallet(&password, wallet_id, purpose, chain)
            .unwrap();
        decrypted.with_revealed(|seed_bytes| assert_eq!(seed_bytes.as_bytes(), &seed));
    }

    #[test]
    fn test_encrypt_seed_len_bad() {
        let password = SafePassword::from("test-password");
        let wallet_id = b"wallet-uuid";
        let purpose = b"purpose";
        let birthday = 12345u32;
        let chain = ChainType::Devnet;

        for &len in [0usize, 32, 63, 65].iter() {
            let seed = vec![42u8; len];
            let err = CipherSeedContainer::encrypt_wallet(
                &password,
                wallet_id,
                purpose,
                birthday,
                chain,
                seed.as_slice(),
                None,
            )
            .unwrap_err();

            assert!(
                matches!(
                    err,
                    CipherSeedError::InvalidSeedLength {
                        expected: SeedBytes::LEN,
                        got,
                    } if got == len
                ),
                "unexpected error for len={}: {:?}",
                len,
                err
            );
        }
    }

    #[test]
    fn test_nonce_mismatch_rejected() {
        let password = SafePassword::from("test-password");
        let wallet_id = b"wallet-uuid";
        let purpose = b"purpose";
        let birthday = 12345u32;
        let chain = ChainType::Devnet;
        let seed = [42u8; 64];

        let container = CipherSeedContainer::encrypt_wallet(
            &password, wallet_id, purpose, birthday, chain, &seed, None,
        )
        .unwrap();

        let mut tampered = container.clone();
        tampered.nonce[0] ^= 0x01;

        let err = tampered
            .decrypt_wallet(&password, wallet_id, purpose, chain)
            .unwrap_err();
        assert_auth_fail(&err);
    }

    #[test]
    fn test_cipher_seed_wrong_password() {
        let password1 = SafePassword::from("pw1");
        let password2 = SafePassword::from("pw2");
        let wallet_id = b"test-wallet";
        let purpose = b"test-purpose";
        let birthday = 1u32;
        let chain = ChainType::Devnet;
        let seed = [7u8; 64];

        let container = CipherSeedContainer::encrypt_wallet(
            &password1, wallet_id, purpose, birthday, chain, &seed, None,
        )
        .unwrap();
        let err = container
            .decrypt_wallet(&password2, wallet_id, purpose, chain)
            .unwrap_err();
        assert_auth_fail(&err);
    }

    #[test]
    fn test_birthday_conversion() {
        let unix = 1700000000u64;
        let birthday = CipherSeedContainer::birthday_from_unix_seconds(unix);
        assert_eq!(birthday, Some((unix / (24 * 60 * 60)) as u32));

        let year_2106 = 0x1_0000_0000u64;
        assert!(CipherSeedContainer::birthday_from_unix_seconds(year_2106).is_some());

        let overflow = (u32::MAX as u64 + 1) * (24 * 60 * 60);
        let birthday_overflow = CipherSeedContainer::birthday_from_unix_seconds(overflow);
        assert!(birthday_overflow.is_none());
    }

    #[test]
    fn test_birthday_overflow() {
        let overflow = (u32::MAX as u64 + 1) * (24 * 60 * 60);
        let err = CipherSeedContainer::birthday_from_unix_seconds_checked(overflow).unwrap_err();
        assert!(matches!(err, CipherSeedError::BirthdayOverflow));
    }

    #[test]
    fn test_decrypt_fails_wrong_params() {
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
        bad_container.kdf_params.mem_kib = 1024;

        let err = bad_container
            .decrypt_wallet(&password, wallet_id, purpose, chain)
            .unwrap_err();
        assert!(matches!(err, CipherSeedError::InvalidKdfParams));
    }

    #[test]
    fn test_load_rejects_invalid_bounds() {
        let params = Argon2idParams {
            mem_kib: 1024,
            time: 5,
            lanes: 8,
        };
        assert!(params.validate().is_err());

        let params = Argon2idParams {
            mem_kib: 128 * 1024,
            time: 2,
            lanes: 8,
        };
        assert!(params.validate().is_err());

        let params = Argon2idParams {
            mem_kib: 128 * 1024,
            time: 5,
            lanes: 0,
        };
        assert!(params.validate().is_err());

        let params = Argon2idParams {
            mem_kib: 128 * 1024,
            time: 5,
            lanes: 17,
        };
        assert!(params.validate().is_err());

        let params = Argon2idParams {
            mem_kib: 1024 * 1024 + 1,
            time: 5,
            lanes: 8,
        };
        assert!(params.validate().is_err());
    }

    #[test]
    fn test_salt_entropy_length_enforced() {
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

        assert_eq!(container.salt.len(), 32);
        assert!(!container.salt.iter().all(|&b| b == 0));

        let first_half = &container.salt[..16];
        let second_half = &container.salt[16..];
        assert_ne!(first_half, second_half);
    }

    #[test]
    fn test_adaptive_kdf_params() {
        let params = Argon2idParams::adapt_to_hardware();
        assert!(params.validate().is_ok());
        assert_eq!(params, Argon2idParams::DEFAULT);
    }

    #[test]
    fn test_custom_kdf_params() {
        let password = SafePassword::from("test");
        let wallet_id = b"test-wallet";
        let purpose = b"test-purpose";
        let birthday = 12345u32;
        let chain = ChainType::Devnet;
        let seed = [42u8; 64];

        let container = CipherSeedContainer::encrypt_wallet(
            &password,
            wallet_id,
            purpose,
            birthday,
            chain,
            &seed,
            Some(Argon2idParams::MOBILE),
        )
        .unwrap();

        assert_eq!(container.kdf_params, Argon2idParams::MOBILE);

        let decrypted = container
            .decrypt_wallet(&password, wallet_id, purpose, chain)
            .unwrap();
        assert_eq!(decrypted.reveal().as_bytes(), &seed);
    }