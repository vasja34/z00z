use super::*;

mod tests {
    use super::*;

    #[test]
    fn test_validation_report_new() {
        let report = ValidationReport::new();
        assert!(report.is_valid());
        assert_eq!(report.errors.len(), 0);
        assert_eq!(report.warnings.len(), 0);
        assert_eq!(report.total_validated, 0);
    }

    #[test]
    fn test_validation_report_add_error() {
        let mut report = ValidationReport::new();
        report.add_error("Test error".to_string());

        assert!(!report.is_valid());
        assert_eq!(report.errors.len(), 1);
        assert_eq!(report.errors[0], "Test error");
    }

    #[test]
    fn test_validation_report_add_warning() {
        let mut report = ValidationReport::new();
        report.add_warning("Test warning".to_string());

        assert!(report.is_valid());
        assert_eq!(report.warnings.len(), 1);
        assert_eq!(report.warnings[0], "Test warning");
    }

    #[test]
    fn test_validation_report_merge() {
        let mut report1 = ValidationReport::new();
        report1.add_error("Error 1".to_string());
        report1.add_warning("Warning 1".to_string());
        report1.total_validated = 10;

        let mut report2 = ValidationReport::new();
        report2.add_error("Error 2".to_string());
        report2.add_warning("Warning 2".to_string());
        report2.total_validated = 20;

        report1.merge(report2);

        assert_eq!(report1.errors.len(), 2);
        assert_eq!(report1.warnings.len(), 2);
        assert_eq!(report1.total_validated, 30);
    }

    #[test]
    fn test_validation_report_is_valid() {
        let mut report = ValidationReport::new();
        assert!(report.is_valid());

        report.add_warning("Warning".to_string());
        assert!(report.is_valid());

        report.add_error("Error".to_string());
        assert!(!report.is_valid());
    }

    #[test]
    fn test_validate_assets_schema_overflow() {
        use crate::assets::AssetClass;
        use crate::genesis::genesis_config::{AssetConfigEntry, PolicyConfig};

        let valid_asset = AssetConfigEntry {
            id: "test_coin".to_string(),
            class: AssetClass::Coin,
            name: "Test Coin".to_string(),
            symbol: "TST".to_string(),
            description: None,
            domain_name: "test.io".to_string(),
            policy: PolicyConfig {
                decimals: 8,
                serials: 1000,
                nominal: 100_000_000,
                ..PolicyConfig::default()
            },
            metadata: None,
        };
        assert!(validate_assets_schema(&[valid_asset]).is_ok());

        let overflow_asset = AssetConfigEntry {
            id: "overflow_coin".to_string(),
            class: AssetClass::Coin,
            name: "Overflow Coin".to_string(),
            symbol: "OVFL".to_string(),
            description: None,
            domain_name: "test.io".to_string(),
            policy: PolicyConfig {
                decimals: 8,
                serials: 1_000_000_000,
                nominal: 1_000_000_000_000,
                ..PolicyConfig::default()
            },
            metadata: None,
        };
        let result = validate_assets_schema(&[overflow_asset]);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("overflow"));

        let edge_asset = AssetConfigEntry {
            id: "edge_coin".to_string(),
            class: AssetClass::Coin,
            name: "Edge Coin".to_string(),
            symbol: "EDGE".to_string(),
            description: None,
            domain_name: "test.io".to_string(),
            policy: PolicyConfig {
                decimals: 8,
                serials: 1_000_000,
                nominal: 1_000_000_000_000,
                ..PolicyConfig::default()
            },
            metadata: None,
        };
        assert!(validate_assets_schema(&[edge_asset]).is_ok());

        let edge_asset2 = AssetConfigEntry {
            id: "edge_coin2".to_string(),
            class: AssetClass::Coin,
            name: "Edge Coin 2".to_string(),
            symbol: "EDG2".to_string(),
            description: None,
            domain_name: "test.io".to_string(),
            policy: PolicyConfig {
                decimals: 8,
                serials: 1_000_000_000,
                nominal: 1_000_000,
                ..PolicyConfig::default()
            },
            metadata: None,
        };
        assert!(validate_assets_schema(&[edge_asset2]).is_ok());

        let zero_serials_asset = AssetConfigEntry {
            id: "zero_serials".to_string(),
            class: AssetClass::Coin,
            name: "Zero Serials".to_string(),
            symbol: "ZERO".to_string(),
            description: None,
            domain_name: "test.io".to_string(),
            policy: PolicyConfig {
                decimals: 8,
                serials: 0,
                nominal: 1_000_000,
                ..PolicyConfig::default()
            },
            metadata: None,
        };
        let result = validate_assets_schema(&[zero_serials_asset]);
        assert!(result.is_err());

        let zero_nominal_asset = AssetConfigEntry {
            id: "zero_nominal".to_string(),
            class: AssetClass::Coin,
            name: "Zero Nominal".to_string(),
            symbol: "ZNOM".to_string(),
            description: None,
            domain_name: "test.io".to_string(),
            policy: PolicyConfig {
                decimals: 8,
                serials: 1000,
                nominal: 0,
                ..PolicyConfig::default()
            },
            metadata: None,
        };
        let result = validate_assets_schema(&[zero_nominal_asset]);
        assert!(result.is_err());
    }
}

mod c2_tests {
    use super::*;
    use crate::genesis::genesis_config::{
        ChainConfig, DomainsConfig, OutputsConfig, PerformanceConfig, ThreadCountConfig,
        ThreadCountMode,
    };
    use crate::rights::{RightClassConfig, RightsConfigEntry};
    use std::collections::BTreeMap;

    #[test]
    fn test_compute_genesis_state_hash() {
        let acc = GenesisAssetAccumulator::new();
        let hash1 = compute_genesis_state_hash(&acc);
        let hash2 = compute_genesis_state_hash(&acc);
        assert_eq!(hash1, hash2, "Empty accumulator should be deterministic");
        assert_ne!(hash1, [0u8; 32], "Hash should not be all zeros");
    }

    #[test]
    fn test_detect_network_type() {
        let mut config = GenesisConfig {
            chain: ChainConfig {
                id: 1,
                chain_type: "mainnet".to_string(),
                name: "z00z-mainnet-1".to_string(),
                magic_bytes: [0x7A, 0x30, 0x30, 0x7A],
                domains: DomainsConfig {
                    genesis_seed: [1u8; 32],
                },
            },
            assets: vec![],
            rights: vec![RightsConfigEntry {
                id: "test_right".to_string(),
                right_class: RightClassConfig::ServiceEntitlement,
                issuer_scope: "issuer_test".to_string(),
                provider_scope: "provider_test".to_string(),
                holder_fixture: "wallet_alice".to_string(),
                control_fixture: "wallet_alice".to_string(),
                beneficiary_fixture: Some("wallet_alice".to_string()),
                count: 1,
                domain_name: "rights.test.v1".to_string(),
                valid_from: 0,
                valid_until: 100,
                challenge_from: 0,
                challenge_until: 0,
                revocation_policy_id: "policy_revoke".to_string(),
                transition_policy_id: "policy_transition".to_string(),
                challenge_policy_id: "policy_challenge".to_string(),
                disclosure_policy_id: "policy_disclosure".to_string(),
                retention_policy_id: "policy_retention".to_string(),
                payload_commitment_seed: "payload_seed".to_string(),
                metadata: Some(BTreeMap::from([(
                    "purpose".to_string(),
                    "create, transfer, revoke".to_string(),
                )])),
            }],
            policies: vec![],
            vouchers: vec![],
            wallet_profiles: vec![],
            policy_profiles: vec![],
            outputs: OutputsConfig {
                assets_export_path: "outputs/".to_string(),
                snapshot_export_path: "outputs/snapshots/".to_string(),
                logging_path: "crates/z00z_core/outputs/log/".to_string(),
            },
            performance: PerformanceConfig {
                num_threads: ThreadCountConfig::Named(ThreadCountMode::Auto),
            },
        };

        assert_eq!(detect_chain_type(&config), Some(ChainType::Mainnet));

        config.chain.chain_type = "testnet".to_string();
        assert_eq!(detect_chain_type(&config), Some(ChainType::Testnet));

        config.chain.chain_type = "devnet".to_string();
        assert_eq!(detect_chain_type(&config), Some(ChainType::Devnet));

        config.chain.chain_type = "unknown".to_string();
        assert_eq!(detect_chain_type(&config), None);
    }

    #[test]
    fn test_network_type_equality() {
        assert_eq!(ChainType::Mainnet, ChainType::Mainnet);
        assert_ne!(ChainType::Mainnet, ChainType::Testnet);
        assert_ne!(ChainType::Testnet, ChainType::Devnet);
    }

    #[test]
    fn test_verify_genesis_consensus_devnet() {
        let hash = [42u8; 32];
        let result = verify_genesis_consensus(ChainType::Devnet, &hash);
        assert!(result.is_ok(), "Devnet should skip verification");
    }

    #[test]
    fn test_verify_mainnet_requires_anchor() {
        let hash = [7u8; 32];
        let result = verify_genesis_consensus(ChainType::Mainnet, &hash);
        assert!(matches!(
            result,
            Err(GenesisError::MissingGenesisAnchor { network }) if network == "mainnet"
        ));
    }

    #[test]
    fn test_verify_testnet_requires_anchor() {
        let hash = [9u8; 32];
        let result = verify_genesis_consensus(ChainType::Testnet, &hash);
        assert!(matches!(
            result,
            Err(GenesisError::MissingGenesisAnchor { network }) if network == "testnet"
        ));
    }

    #[test]
    fn test_str_rejects_unknown_values() {
        let result = "staging".parse::<ChainType>();
        assert!(matches!(result, Err(GenesisError::InvalidConfig(_))));
    }
}
