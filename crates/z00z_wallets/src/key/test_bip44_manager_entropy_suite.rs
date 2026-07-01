#[test]
    fn test_bip39_seed64_entropy_validation() {
        let zero_seed = Bip39Seed64::new([0u8; 64]);
        assert!(zero_seed.is_all_zeros());
        assert!(zero_seed.validate_entropy().is_err());
        match zero_seed.validate_entropy() {
            Err(Bip44Error::WeakEntropy(_)) => {}
            _ => panic!("Expected WeakEntropy error for all-zeros seed"),
        }

        let identical_seed = Bip39Seed64::new([42u8; 64]);
        assert!(identical_seed.is_all_identical());
        assert!(!validate_entropy_with_warnings(identical_seed.as_ref())
            .unwrap()
            .is_empty());
        assert!(identical_seed.validate_entropy().is_err());

        let valid_seed = Bip39Seed64::new(test_seed_bytes());
        assert!(!valid_seed.is_all_zeros());
        assert!(!valid_seed.is_all_identical());
        assert!(valid_seed.validate_entropy().is_ok());

        let short_slice = [0u8; 31];
        assert!(Bip39Seed64::from_slice(&short_slice).is_err());

        let long_slice = [0u8; 65];
        assert!(Bip39Seed64::from_slice(&long_slice).is_err());

        let valid_slice = [0u8; 64];
        assert!(Bip39Seed64::from_slice(&valid_slice).is_ok());
    }

    #[test]
    fn test_entropy_warn_ok() {
        let mut weak_seed = [0u8; 64];
        weak_seed[0] = 0x02;
        for idx in (8..64).step_by(9) {
            weak_seed[idx] = 0x01;
        }

        let warnings = validate_entropy_with_warnings(&weak_seed).unwrap();
        assert!(warnings
            .iter()
            .any(|w| matches!(w, EntropyWarning::UnusualBitCount { .. })));
        assert!(!warnings
            .iter()
            .any(|w| matches!(w, EntropyWarning::RepeatingPattern)));
        assert!(!warnings
            .iter()
            .any(|w| matches!(w, EntropyWarning::LongZeroRun)));

        let seed = Bip39Seed64::new(weak_seed);
        let err = Bip44KeyManager::new(seed, Z00Z_BIP44_ASSET, ChainType::Devnet).unwrap_err();
        assert!(matches!(err, Bip44Error::WeakEntropy(_)));
    }

    #[test]
    fn test_catastrophic_entropy_error() {
        let zero_seed = [0x00u8; 64];
        assert!(validate_entropy(&zero_seed).is_err());

        let ones_seed = [0xffu8; 64];
        assert!(validate_entropy(&ones_seed).is_err());
    }

    #[test]
    fn test_seed_repeating_blocks() {
        let mut seed_bytes = [0u8; 64];
        let block8: [u8; 8] = [0x11, 0x22, 0x33, 0x44, 0x55, 0x66, 0x77, 0x88];
        for i in 0..8 {
            let start = i * 8;
            seed_bytes[start..start + 8].copy_from_slice(&block8);
        }

        let seed = Bip39Seed64::new(seed_bytes);
        let warnings = validate_entropy_with_warnings(seed.as_ref()).unwrap();
        assert!(!warnings.is_empty());
        assert!(seed.validate_entropy().is_err());

        let mut seed_bytes = [0u8; 64];
        let block16: [u8; 16] = [
            0x00, 0x01, 0x02, 0x03, 0x10, 0x11, 0x12, 0x13, 0x20, 0x21, 0x22, 0x23, 0x30, 0x31,
            0x32, 0x33,
        ];
        for i in 0..4 {
            let start = i * 16;
            seed_bytes[start..start + 16].copy_from_slice(&block16);
        }

        let seed = Bip39Seed64::new(seed_bytes);
        let warnings = validate_entropy_with_warnings(seed.as_ref()).unwrap();
        assert!(!warnings.is_empty());
        assert!(seed.validate_entropy().is_err());
    }

    #[test]
    fn test_seed_extreme_bias() {
        let mut low = [0u8; 64];
        low[0] = 0x02;
        for idx in (8..64).step_by(9) {
            low[idx] = 0x01;
        }

        let seed = Bip39Seed64::new(low);
        let warnings = validate_entropy_with_warnings(seed.as_ref()).unwrap();
        assert!(!warnings.is_empty());
        assert!(seed.validate_entropy().is_err());

        let mut high = [0xffu8; 64];
        high[0] = 0xfe;
        let seed = Bip39Seed64::new(high);
        let warnings = validate_entropy_with_warnings(seed.as_ref()).unwrap();
        assert!(!warnings.is_empty());
        assert!(seed.validate_entropy().is_err());

        let seed = Bip39Seed64::new([0xffu8; 64]);
        assert!(seed.validate_entropy().is_err());
        assert!(matches!(
            seed.validate_entropy(),
            Err(Bip44Error::WeakEntropy(_))
        ));
    }

    #[test]
    fn test_long_zero_run_rejected() {
        let mut bytes = [0u8; 64];
        bytes[0] = 0x7f;
        bytes[1] = 0x3c;

        bytes[2..=10].fill(0x00);

        for (i, b) in bytes.iter_mut().enumerate().skip(11) {
            *b = (i as u8).wrapping_mul(13).wrapping_add(7);
        }

        let seed = Bip39Seed64::new(bytes);
        let warnings = validate_entropy_with_warnings(seed.as_ref()).unwrap();
        assert!(!warnings.is_empty());
        assert!(seed.validate_entropy().is_err());
    }

    #[test]
    fn test_seed_normal_passes() {
        let seed = Bip39Seed64::new([
            0x9c, 0x4f, 0x2a, 0x10, 0x77, 0xd1, 0x33, 0x8e, 0x0b, 0xa6, 0x51, 0xc2, 0x94, 0x6d,
            0xe8, 0x05, 0x3a, 0x7f, 0x28, 0x99, 0xb0, 0x1c, 0x46, 0xdd, 0x6a, 0x12, 0xf1, 0x83,
            0x58, 0x2e, 0x9d, 0x70, 0x4b, 0xc7, 0x0a, 0x3f, 0xe1, 0x55, 0x8c, 0x26, 0x91, 0x6f,
            0xd0, 0x1b, 0x47, 0xa2, 0x39, 0x8d, 0x62, 0x0c, 0xf7, 0x14, 0x9a, 0x53, 0xbe, 0x21,
            0x7a, 0xcd, 0x06, 0x44, 0x98, 0x30, 0xef, 0x11,
        ]);

        assert!(seed.validate_entropy().is_ok());
    }

    #[test]
    fn test_drop_zeroizes() {
        reset_seed_zeroized();

        let seed_bytes = test_seed_bytes();
        let seed = Bip39Seed64::new(seed_bytes);

        {
            let _manager = Bip44KeyManager::new(seed, Z00Z_BIP44_ASSET, ChainType::Devnet).unwrap();
        }

        assert!(seed_zeroized());
    }

    #[test]
    fn test_max_bip32_accepted() {
        let path = Bip44PathBuilder::new()
            .asset_type(Z00Z_BIP44_ASSET)
            .account(MAX_BIP32_INDEX)
            .change(0)
            .address_index(MAX_BIP32_INDEX)
            .build()
            .unwrap();

        assert_eq!(path.account().index(), MAX_BIP32_INDEX);
        assert_eq!(path.address_index().index(), MAX_BIP32_INDEX);
        assert!(path.account().is_hardened());
        assert!(!path.address_index().is_hardened());
    }

    #[test]
    fn test_account_overflow_rejected() {
        let result = Bip44PathBuilder::new()
            .asset_type(Z00Z_BIP44_ASSET)
            .account(MAX_BIP32_INDEX + 1)
            .change(0)
            .address_index(0)
            .build();

        assert!(result.is_err());
        assert!(matches!(
            result,
            Err(Bip44Error::IndexOutOfRange {
                field: "account",
                value: _,
                max: MAX_BIP32_INDEX
            })
        ));
    }

    #[test]
    fn test_address_overflow_rejected() {
        let result = Bip44PathBuilder::new()
            .asset_type(Z00Z_BIP44_ASSET)
            .account(0)
            .change(0)
            .address_index(MAX_BIP32_INDEX + 1)
            .build();

        assert!(result.is_err());
        assert!(matches!(
            result,
            Err(Bip44Error::IndexOutOfRange {
                field: "address_index",
                value: _,
                max: MAX_BIP32_INDEX
            })
        ));
    }

    #[test]
    fn test_seed_validation() {
        let short_seed = [0u8; 31];
        assert!(MasterKeyGenerator::from_seed(&short_seed).is_err());

        let long_seed = [0u8; 65];
        assert!(MasterKeyGenerator::from_seed(&long_seed).is_err());

        let bip32_min_seed = [0u8; 16];
        assert!(MasterKeyGenerator::from_seed(&bip32_min_seed).is_ok());

        let min_seed = [0u8; 32];
        assert!(MasterKeyGenerator::from_seed(&min_seed).is_ok());

        let max_seed = [0u8; 64];
        assert!(MasterKeyGenerator::from_seed(&max_seed).is_ok());
    }

    #[test]
    fn test_account_matches_bip32_path() {
        let seed_bytes = test_seed_bytes();
        let seed_for_expected = Bip39Seed64::new(seed_bytes);
        let seed_for_manager = Bip39Seed64::new(seed_bytes);
        let manager =
            Bip44KeyManager::new(seed_for_manager, Z00Z_BIP44_ASSET, ChainType::Devnet).unwrap();

        let derived = manager.derive_account_key(0).unwrap();
        let expected_path =
            DerivationPath::from_str(&format!("m/44'/{asset}'/0'", asset = Z00Z_BIP44_ASSET))
                .unwrap();
        let expected = XPrv::derive_from_path(seed_for_expected, &expected_path).unwrap();

        assert_eq!(
            derived.private_key().to_bytes(),
            expected.private_key().to_bytes()
        );
        assert_eq!(derived.attrs().chain_code, expected.attrs().chain_code);
    }

    #[test]
    fn test_address_matches_bip32_path() {
        let seed_bytes = test_seed_bytes();
        let seed_for_expected = Bip39Seed64::new(seed_bytes);
        let seed_for_manager = Bip39Seed64::new(seed_bytes);
        let manager =
            Bip44KeyManager::new(seed_for_manager, Z00Z_BIP44_ASSET, ChainType::Devnet).unwrap();

        let derived = manager.derive_address_key(0, 0, 5).unwrap();
        let expected_path =
            DerivationPath::from_str(&format!("m/44'/{asset}'/0'/0/5", asset = Z00Z_BIP44_ASSET))
                .unwrap();
        let expected = XPrv::derive_from_path(seed_for_expected, &expected_path).unwrap();

        assert_eq!(
            derived.private_key().to_bytes(),
            expected.private_key().to_bytes()
        );
        assert_eq!(derived.attrs().chain_code, expected.attrs().chain_code);
    }

    #[test]
    fn test_reject_nonz00z_asset_manager() {
        let seed_bytes: [u8; 64] = [
            0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0a, 0x0b, 0x0c, 0x0d,
            0x0e, 0x0f, 0x10, 0x11, 0x12, 0x13, 0x14, 0x15, 0x16, 0x17, 0x18, 0x19, 0x1a, 0x1b,
            0x1c, 0x1d, 0x1e, 0x1f, 0x20, 0x21, 0x22, 0x23, 0x24, 0x25, 0x26, 0x27, 0x28, 0x29,
            0x2a, 0x2b, 0x2c, 0x2d, 0x2e, 0x2f, 0x30, 0x31, 0x32, 0x33, 0x34, 0x35, 0x36, 0x37,
            0x38, 0x39, 0x3a, 0x3b, 0x3c, 0x3d, 0x3e, 0x3f,
        ];
        let seed = Bip39Seed64::new(seed_bytes);
        assert!(Bip44KeyManager::new(seed, 0, ChainType::Devnet).is_err());
    }

    #[test]
    fn test_network_separation() {
        let seed_bytes = test_seed_bytes();
        let mainnet_manager = Bip44KeyManager::new(
            Bip39Seed64::new(seed_bytes),
            Z00Z_BIP44_ASSET,
            ChainType::Mainnet,
        )
        .unwrap();
        let testnet_manager = Bip44KeyManager::new(
            Bip39Seed64::new(seed_bytes),
            Z00Z_BIP44_ASSET,
            ChainType::Testnet,
        )
        .unwrap();
        let devnet_manager = Bip44KeyManager::new(
            Bip39Seed64::new(seed_bytes),
            Z00Z_BIP44_ASSET,
            ChainType::Devnet,
        )
        .unwrap();

        let mainnet_key = mainnet_manager.derive_ristretto_key(0, 0, 0).unwrap();
        let testnet_key = testnet_manager.derive_ristretto_key(0, 0, 0).unwrap();
        let devnet_key = devnet_manager.derive_ristretto_key(0, 0, 0).unwrap();

        assert_ne!(mainnet_key.as_bytes(), testnet_key.as_bytes());
        assert_ne!(mainnet_key.as_bytes(), devnet_key.as_bytes());
        assert_ne!(testnet_key.as_bytes(), devnet_key.as_bytes());
    }

    #[test]
    fn test_keys_labels_are_frozen() {
        assert_eq!(MAINNET_KEYS_LABEL, "z00z/mainnet/keys");
        assert_eq!(DEVNET_KEYS_LABEL, "z00z/devnet/keys");
        assert_eq!(TESTNET_KEYS_LABEL, "z00z/testnet/keys");
    }

    #[test]
    fn test_zero_key_reject() {
        let err = reject_zero_key(&RistrettoSecretKey::default()).unwrap_err();
        assert!(matches!(err, Bip44Error::WeakEntropy(_)));
    }
