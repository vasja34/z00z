    #[test]
    fn test_asset_constant_matches_policy() {
        assert_eq!(
            Z00Z_BIP44_ASSET, 1337,
            "Coin type must not change without migration"
        );
    }

    #[test]
    fn test_parse_valid_bip44_path() {
        let path = Bip44Path::from_str(&format!("m/44'/{asset}'/0'/0/0", asset = Z00Z_BIP44_ASSET))
            .unwrap();
        assert!(path.purpose.is_hardened());
        assert!(path.asset_type.is_hardened());
        assert!(path.account.is_hardened());
        assert!(!path.change.is_hardened());
        assert!(!path.address_index.is_hardened());
    }

    #[test]
    fn test_parse_invalid_hardened_change() {
        let result =
            Bip44Path::from_str(&format!("m/44'/{asset}'/0'/0'/0", asset = Z00Z_BIP44_ASSET));
        assert!(result.is_err());
    }

    #[test]
    fn test_invalid_hardened_address_index() {
        let result =
            Bip44Path::from_str(&format!("m/44'/{asset}'/0'/0/0'", asset = Z00Z_BIP44_ASSET));
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_invalid_change_value() {
        let result =
            Bip44Path::from_str(&format!("m/44'/{asset}'/0'/2/0", asset = Z00Z_BIP44_ASSET));
        assert!(result.is_err());
    }

    #[test]
    fn test_path_roundtrip() {
        let original = format!("m/44'/{asset}'/0'/1/5", asset = Z00Z_BIP44_ASSET);
        let path = Bip44Path::from_str(&original).unwrap();
        let formatted = path.to_string();
        assert_eq!(original, formatted);
    }

    #[test]
    fn test_builder_pattern() {
        let path = Bip44PathBuilder::new()
            .asset_type(Z00Z_BIP44_ASSET)
            .account(0)
            .change(0)
            .address_index(5)
            .build()
            .unwrap();

        assert_eq!(
            path.to_string(),
            format!("m/44'/{asset}'/0'/0/5", asset = Z00Z_BIP44_ASSET)
        );
    }

    #[test]
    fn test_view_key_path_derivation() {
        let spend_path = Bip44Path::new_z00z(7, 1, 42).unwrap();
        let view_path = spend_path.to_view_key_path().unwrap();

        assert_eq!(view_path.account().index(), 7 + VIEW_KEY_ACCOUNT_OFFSET);
        assert_eq!(view_path.change().index(), spend_path.change().index());
        assert_eq!(
            view_path.address_index().index(),
            spend_path.address_index().index()
        );
        assert_eq!(view_path.asset_type().index(), Z00Z_BIP44_ASSET);
        assert!(Bip44Validator::validate(&view_path).is_ok());
    }

    #[test]
    fn test_path_rejects_view_account() {
        let view_path = Bip44PathBuilder::new()
            .asset_type(Z00Z_BIP44_ASSET)
            .account(VIEW_KEY_ACCOUNT_OFFSET)
            .change(0)
            .address_index(0)
            .build()
            .unwrap();
        let err = view_path.to_view_key_path().unwrap_err();
        assert!(matches!(err, Bip44Error::InvalidPath(_)));
    }

    #[test]
    fn test_key_path_boundary_cases() {
        let spend_path = Bip44Path::new_z00z(99_999, 0, 0).unwrap();
        let view_path = spend_path.to_view_key_path().unwrap();
        assert_eq!(view_path.account().index(), 199_999);

        let view_path_direct = Bip44PathBuilder::new()
            .asset_type(Z00Z_BIP44_ASSET)
            .account(VIEW_KEY_ACCOUNT_OFFSET)
            .change(0)
            .address_index(0)
            .build()
            .unwrap();
        let err = view_path_direct.to_view_key_path().unwrap_err();
        assert!(matches!(err, Bip44Error::InvalidPath(_)));
    }

    #[test]
    fn test_spend_path_reverse_function() {
        let view_path = Bip44PathBuilder::new()
            .asset_type(Z00Z_BIP44_ASSET)
            .account(VIEW_KEY_ACCOUNT_OFFSET + 42)
            .change(1)
            .address_index(99)
            .build()
            .unwrap();

        let spend_path = view_path.to_spend_key_path().unwrap();
        assert_eq!(spend_path.account().index(), 42);
        assert_eq!(spend_path.change().index(), 1);
        assert_eq!(spend_path.address_index().index(), 99);

        let spend_path_invalid = Bip44Path::new_z00z(50, 0, 0).unwrap();
        let err = spend_path_invalid.to_spend_key_path().unwrap_err();
        assert!(matches!(err, Bip44Error::InvalidPath(_)));
    }

    #[test]
    fn test_key_path_round_trip() {
        let spend_path = Bip44Path::new_z00z(42, 1, 99).unwrap();
        let view_path = spend_path.to_view_key_path().unwrap();
        let recovered = view_path.to_spend_key_path().unwrap();

        assert_eq!(spend_path.account().index(), recovered.account().index());
        assert_eq!(spend_path.change().index(), recovered.change().index());
        assert_eq!(
            spend_path.address_index().index(),
            recovered.address_index().index()
        );

        assert_eq!(view_path.account().index(), 42 + VIEW_KEY_ACCOUNT_OFFSET);
        assert_eq!(view_path.change().index(), spend_path.change().index());
        assert_eq!(
            view_path.address_index().index(),
            spend_path.address_index().index()
        );
    }

    #[test]
    fn test_path_classification() {
        let spend = Bip44Path::new_z00z(50_000, 0, 0).unwrap();
        assert!(spend.is_spend_key_path());
        assert!(!spend.is_view_key_path());

        let view = spend.to_view_key_path().unwrap();
        assert!(!view.is_spend_key_path());
        assert!(view.is_view_key_path());

        let high_account = Bip44PathBuilder::new()
            .asset_type(Z00Z_BIP44_ASSET)
            .account(200_000)
            .change(0)
            .address_index(0)
            .build()
            .unwrap();
        assert!(!high_account.is_view_key_path());
        assert!(!high_account.is_spend_key_path());
    }

    #[test]
    fn test_corresponding_path() {
        let spend = Bip44Path::new_z00z(7, 1, 42).unwrap();
        let view = spend.corresponding_path().unwrap();
        assert_eq!(view.account().index(), 7 + VIEW_KEY_ACCOUNT_OFFSET);
        assert!(view.is_view_key_path());

        let recovered_spend = view.corresponding_path().unwrap();
        assert_eq!(recovered_spend.account().index(), 7);
        assert!(recovered_spend.is_spend_key_path());

        let invalid = Bip44PathBuilder::new()
            .asset_type(Z00Z_BIP44_ASSET)
            .account(300_000)
            .change(0)
            .address_index(0)
            .build()
            .unwrap();
        let err = invalid.corresponding_path().unwrap_err();
        assert!(matches!(err, Bip44Error::InvalidPath(_)));
    }

    #[test]
    fn test_rejects_view_key_namespace() {
        let err = Bip44Path::new_z00z(VIEW_KEY_ACCOUNT_OFFSET, 0, 0).unwrap_err();
        assert!(matches!(err, Bip44Error::InvalidPath(_)));

        let err = Bip44Path::new_z00z(VIEW_KEY_ACCOUNT_OFFSET + 1000, 0, 0).unwrap_err();
        assert!(matches!(err, Bip44Error::InvalidPath(_)));

        let err = Bip44Path::new_z00z(2 * VIEW_KEY_ACCOUNT_OFFSET - 1, 0, 0).unwrap_err();
        assert!(matches!(err, Bip44Error::InvalidPath(_)));

        let _valid = Bip44Path::new_z00z(2 * VIEW_KEY_ACCOUNT_OFFSET, 0, 0).unwrap();
    }

    #[test]
    fn test_hardened_vs_nonhardened_collision() {
        let _path_a =
            Bip44Path::from_str(&format!("m/44'/{asset}'/0'/0/0", asset = Z00Z_BIP44_ASSET))
                .unwrap();
        let path_b_result =
            Bip44Path::from_str(&format!("m/44/{asset}/0/0/0", asset = Z00Z_BIP44_ASSET));

        assert!(path_b_result.is_err());
    }

    #[test]
    fn test_bip44_validation_rules() {
        assert!(
            Bip44Path::from_str(&format!("m/44/{asset}'/0'/0/0", asset = Z00Z_BIP44_ASSET))
                .is_err()
        );
        assert!(
            Bip44Path::from_str(&format!("m/44'/{asset}/0'/0/0", asset = Z00Z_BIP44_ASSET))
                .is_err()
        );
        assert!(
            Bip44Path::from_str(&format!("m/44'/{asset}'/0/0/0", asset = Z00Z_BIP44_ASSET))
                .is_err()
        );
        assert!(
            Bip44Path::from_str(&format!("m/44'/{asset}'/0'/0'/0", asset = Z00Z_BIP44_ASSET))
                .is_err()
        );
        assert!(
            Bip44Path::from_str(&format!("m/44'/{asset}'/0'/0/0'", asset = Z00Z_BIP44_ASSET))
                .is_err()
        );
        assert!(
            Bip44Path::from_str(&format!("m/44'/{asset}'/0'/2/0", asset = Z00Z_BIP44_ASSET))
                .is_err()
        );
    }

    #[test]
    fn test_master_key_generation() {
        let seed = [0u8; 32];
        let master = MasterKeyGenerator::from_seed(&seed).unwrap();
        assert!(!master.private_key().to_bytes().is_empty());
    }

    #[test]
    fn test_full_derivation_path() {
        let seed = [0u8; 32];
        let master = MasterKeyGenerator::from_seed(&seed).unwrap();
        let path = Bip44Path::from_str(&format!("m/44'/{asset}'/0'/0/0", asset = Z00Z_BIP44_ASSET))
            .unwrap();

        let child = Bip32KeyDeriver::derive_child(&master, &path).unwrap();

        assert_ne!(
            child.private_key().to_bytes(),
            master.private_key().to_bytes()
        );
    }

    #[test]
    fn test_bip44_key_manager() {
        let seed_bytes = test_seed_bytes();
        let seed = Bip39Seed64::new(seed_bytes);
        let manager = Bip44KeyManager::new(seed, Z00Z_BIP44_ASSET, ChainType::Devnet).unwrap();

        let account_key = manager.derive_account_key(0).unwrap();
        assert!(!account_key.private_key().to_bytes().is_empty());

        let address_key = manager.derive_address_key(0, 0, 0).unwrap();
        assert!(!address_key.private_key().to_bytes().is_empty());

        let ristretto_key = manager.derive_ristretto_key(0, 0, 0).unwrap();
        assert!(!ristretto_key.as_bytes().is_empty());
    }

    #[test]
    fn test_deterministic_derivation() {
        let seed_bytes = test_seed_bytes();
        let seed1 = Bip39Seed64::new(seed_bytes);
        let seed2 = Bip39Seed64::new(seed_bytes);

        let manager1 = Bip44KeyManager::new(seed1, Z00Z_BIP44_ASSET, ChainType::Devnet).unwrap();
        let manager2 = Bip44KeyManager::new(seed2, Z00Z_BIP44_ASSET, ChainType::Devnet).unwrap();

        let key1 = manager1.derive_address_key(0, 0, 0).unwrap();
        let key2 = manager2.derive_address_key(0, 0, 0).unwrap();

        assert_eq!(key1.private_key().to_bytes(), key2.private_key().to_bytes());
    }

    #[test]
    fn test_paths_produce_different_keys() {
        let seed_bytes = test_seed_bytes();
        let seed = Bip39Seed64::new(seed_bytes);
        let manager = Bip44KeyManager::new(seed, Z00Z_BIP44_ASSET, ChainType::Devnet).unwrap();

        let key1 = manager.derive_address_key(0, 0, 0).unwrap();
        let key2 = manager.derive_address_key(0, 0, 1).unwrap();
        let key3 = manager.derive_address_key(0, 1, 0).unwrap();

        assert_ne!(key1.private_key().to_bytes(), key2.private_key().to_bytes());
        assert_ne!(key1.private_key().to_bytes(), key3.private_key().to_bytes());
        assert_ne!(key2.private_key().to_bytes(), key3.private_key().to_bytes());
    }

    #[test]
    fn test_ristretto_bridge() {
        let seed_bytes = test_seed_bytes();
        let seed = Bip39Seed64::new(seed_bytes);
        let manager = Bip44KeyManager::new(seed, Z00Z_BIP44_ASSET, ChainType::Devnet).unwrap();

        let ristretto_key = manager.derive_ristretto_key(0, 0, 0).unwrap();

        assert_eq!(ristretto_key.as_bytes().len(), 32);
    }

    #[test]
    fn test_intermediate_derivation() {
        let seed = [42u8; 32];
        let master = MasterKeyGenerator::from_seed(&seed).unwrap();

        let account_partial = vec![
            ChildNumber::new(44, true).unwrap(),
            ChildNumber::new(Z00Z_BIP44_ASSET, true).unwrap(),
            ChildNumber::new(0, true).unwrap(),
        ];
        let account_key =
            Bip32KeyDeriver::derive_from_intermediate(&master, &account_partial).unwrap();

        let address_partial = vec![
            ChildNumber::new(0, false).unwrap(),
            ChildNumber::new(0, false).unwrap(),
        ];
        let address_key =
            Bip32KeyDeriver::derive_from_intermediate(&account_key, &address_partial).unwrap();

        let full_path =
            Bip44Path::from_str(&format!("m/44'/{asset}'/0'/0/0", asset = Z00Z_BIP44_ASSET))
                .unwrap();
        let full_key = Bip32KeyDeriver::derive_child(&master, &full_path).unwrap();

        assert_eq!(
            address_key.private_key().to_bytes(),
            full_key.private_key().to_bytes()
        );
    }

    #[test]
    fn test_edge_case_account_values() {
        let seed_bytes = test_seed_bytes();
        let seed = Bip39Seed64::new(seed_bytes);
        let manager = Bip44KeyManager::new(seed, Z00Z_BIP44_ASSET, ChainType::Devnet).unwrap();

        let key0 = manager.derive_account_key(0).unwrap();
        assert!(!key0.private_key().to_bytes().is_empty());

        let key999 = manager.derive_account_key(999).unwrap();
        assert!(!key999.private_key().to_bytes().is_empty());

        assert_ne!(
            key0.private_key().to_bytes(),
            key999.private_key().to_bytes()
        );
    }

    #[test]
    fn test_change_values() {
        let seed_bytes = test_seed_bytes();
        let seed = Bip39Seed64::new(seed_bytes);
        let manager = Bip44KeyManager::new(seed, Z00Z_BIP44_ASSET, ChainType::Devnet).unwrap();

        let external = manager.derive_address_key(0, 0, 0).unwrap();
        let internal = manager.derive_address_key(0, 1, 0).unwrap();

        assert_ne!(
            external.private_key().to_bytes(),
            internal.private_key().to_bytes()
        );
    }

    include!("test_bip44_manager_entropy_suite.rs");
