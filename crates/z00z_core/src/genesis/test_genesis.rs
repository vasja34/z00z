use super::*;

mod tests {
    use super::*;
    use crate::config_paths::devnet_genesis_path;
    use crate::genesis::genesis_config::load_genesis_config;
    use crate::rights::{RightClassConfig, RightsConfigEntry};
    #[cfg(feature = "deterministic-rng")]
    use crate::vouchers::VoucherLifecycleV1;
    use std::collections::BTreeMap;
    use std::path::PathBuf;
    use std::str::FromStr;
    #[cfg(feature = "deterministic-rng")]
    use z00z_utils::prelude::{NoopLogger, NoopMetrics};

    fn sample_right_config() -> RightsConfigEntry {
        RightsConfigEntry {
            id: "service_entitlement".to_string(),
            right_class: RightClassConfig::ServiceEntitlement,
            issuer_scope: "issuer_test".to_string(),
            provider_scope: "provider_test".to_string(),
            holder_fixture: "wallet_alice".to_string(),
            control_fixture: "wallet_alice".to_string(),
            beneficiary_fixture: Some("wallet_alice".to_string()),
            count: 2,
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
        }
    }

    fn canonical_genesis_path() -> PathBuf {
        devnet_genesis_path()
    }

    #[test]
    fn test_accumulator_new() {
        let acc = GenesisAssetAccumulator::new();
        assert_eq!(acc.total_count(), 0);
        assert_eq!(acc.coins.len(), 0);
        assert_eq!(acc.tokens.len(), 0);
        assert_eq!(acc.nfts.len(), 0);
        assert_eq!(acc.voids.len(), 0);
    }

    #[test]
    fn test_accumulator_flatten_empty() {
        let acc = GenesisAssetAccumulator::new();
        let flat = acc.flatten();
        assert_eq!(flat.len(), 0);
    }

    #[test]
    fn test_genesis_seed_zeros_rejected() {
        use crate::genesis::validator::validate_genesis_seed;

        let all_zeros = [0u8; 32];
        let result = validate_genesis_seed(&all_zeros, ChainType::Mainnet);
        assert!(result.is_err());
    }

    #[test]
    fn test_known_test_seed_policy() {
        use crate::genesis::validator::validate_genesis_seed;

        let test_seed = [42u8; 32];
        let protected = validate_genesis_seed(&test_seed, ChainType::Mainnet);
        assert!(protected.is_err(), "Known test seed should fail on mainnet");

        let devnet = validate_genesis_seed(&test_seed, ChainType::Devnet);
        assert!(
            devnet.is_err(),
            "Repeating test seeds should fail closed on devnet too"
        );
    }

    #[test]
    fn test_genesis_seed_sequential_rejected() {
        use crate::genesis::validator::validate_genesis_seed;

        let sequential = [
            1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24,
            25, 26, 27, 28, 29, 30, 31, 32,
        ];

        let result = validate_genesis_seed(&sequential, ChainType::Mainnet);
        assert!(result.is_err());
    }

    #[test]
    fn test_genesis_seed_repeating_rejected() {
        use crate::genesis::validator::validate_genesis_seed;

        let repeating = [0x2Au8; 32];
        let result = validate_genesis_seed(&repeating, ChainType::Testnet);
        assert!(result.is_err());
    }

    #[test]
    fn test_type_str_rejects_unknown() {
        let result = ChainType::from_str("staging");
        assert!(result.is_err(), "unknown chain types must fail closed");
    }

    #[test]
    fn test_derive_deterministic_rng_seed() {
        let seed = [0x11u8; 32];
        let asset_id = [0x22u8; 32];
        let serial_id = 42u32;

        let rng_seed1 =
            derive_deterministic_rng_seed(&seed, &asset_id, serial_id, ChainType::Devnet);
        let rng_seed2 =
            derive_deterministic_rng_seed(&seed, &asset_id, serial_id, ChainType::Devnet);
        assert_eq!(rng_seed1, rng_seed2);

        let rng_seed3 =
            derive_deterministic_rng_seed(&seed, &asset_id, serial_id + 1, ChainType::Devnet);
        assert_ne!(rng_seed1, rng_seed3);
    }

    #[test]
    fn test_fixed_thread_count() {
        let config = crate::genesis::genesis_config::ThreadCountConfig::Fixed(3);
        assert_eq!(config.resolved_threads(), 3);
        assert_eq!(config.configured_label(), "3");
    }

    #[test]
    fn test_auto_thread_count() {
        let config = crate::genesis::genesis_config::ThreadCountConfig::Named(
            crate::genesis::genesis_config::ThreadCountMode::Auto,
        );
        assert!(config.resolved_threads() >= 1);
        assert_eq!(config.configured_label(), "auto");
    }

    #[test]
    fn test_genesis_pool_threads() {
        use rayon::prelude::*;

        let pool =
            build_genesis_thread_pool(&crate::genesis::genesis_config::ThreadCountConfig::Fixed(2))
                .expect("build genesis pool");
        let observed = pool.install(|| {
            (0..8)
                .into_par_iter()
                .map(|_| rayon::current_num_threads())
                .max()
                .expect("observed active thread count")
        });
        assert_eq!(observed, 2);
    }

    #[test]
    fn test_run_uses_thread_pool() {
        let source = include_str!("genesis_run.rs");
        assert!(source.contains("ThreadPoolBuilder::new()"));
        assert!(!source.contains("build_global()"));
    }

    #[test]
    fn test_genesis_output_module() {
        let source = include_str!("genesis_output.rs");
        let include_support = ["include", "!(\"", "genesis_output_support.rs", "\");"].concat();
        assert!(source.contains("genesis_output_support.rs"));
        assert!(source.contains("pub(crate) use self::genesis_output_support::{"));
        assert!(!source.contains(&include_support));
    }

    #[test]
    fn test_create_asset_definition() {
        use crate::genesis::genesis_config::PolicyConfig;

        let cfg = AssetConfigEntry {
            id: "z00z".to_string(),
            class: AssetClass::Coin,
            name: "Z00Z Coin".to_string(),
            symbol: "Z00Z".to_string(),
            description: None,
            domain_name: "z00z.local".to_string(),
            policy: PolicyConfig {
                decimals: 8,
                serials: 1000,
                nominal: 100_000,
                ..PolicyConfig::default()
            },
            metadata: None,
        };

        let genesis_seed = [0x42u8; 32];
        let definition = create_asset_definition(&cfg, &genesis_seed, ChainType::Devnet).unwrap();

        assert_eq!(definition.symbol, "Z00Z");
        assert_eq!(definition.name, "Z00Z Coin");
        assert_eq!(definition.class, AssetClass::Coin);
        assert_eq!(definition.decimals, 8);
        assert_eq!(definition.serials, 1000);
        assert_eq!(definition.nominal, 100_000);
        assert_eq!(definition.version, 1);
        assert_eq!(definition.crypto_version, 1);
        assert_eq!(definition.policy_flags, cfg.policy.asset_flags(cfg.class));
        assert_eq!(definition.id.len(), 32);
    }

    #[cfg(feature = "deterministic-rng")]
    #[test]
    fn test_generate_assets_single_definition() {
        let definition = AssetDefinition::new(
            [1u8; 32],
            AssetClass::Coin,
            "Test Coin".to_string(),
            "TST".to_string(),
            8,
            10,
            1000,
            "test.local".to_string(),
            1,
            1,
            0,
            None,
        )
        .unwrap();

        let genesis_seed = [0x42u8; 32];
        let definition_arc = Arc::new(definition.clone());
        let assets =
            generate_assets_checked(definition_arc, &genesis_seed, ChainType::Devnet).unwrap();

        assert_eq!(assets.len(), 10);
        for (i, asset) in assets.iter().enumerate() {
            assert_eq!(asset.serial_id, i as u32);
            assert_eq!(asset.amount, definition.nominal);
            assert_eq!(asset.definition.id, definition.id);
        }
    }

    #[cfg(feature = "deterministic-rng")]
    #[test]
    fn test_generate_assets_deterministic() {
        let definition = AssetDefinition::new(
            [2u8; 32],
            AssetClass::Token,
            "Test Token".to_string(),
            "TTK".to_string(),
            6,
            5,
            500_000,
            "test.local".to_string(),
            1,
            1,
            0,
            None,
        )
        .unwrap();

        let genesis_seed = [0x99u8; 32];
        let definition_arc = Arc::new(definition.clone());
        let assets1 = generate_assets_checked(
            Arc::clone(&definition_arc),
            &genesis_seed,
            ChainType::Devnet,
        )
        .unwrap();
        let assets2 = generate_assets_checked(
            Arc::clone(&definition_arc),
            &genesis_seed,
            ChainType::Devnet,
        )
        .unwrap();

        assert_eq!(assets1.len(), assets2.len());
        for (a1, a2) in assets1.iter().zip(assets2.iter()) {
            assert_eq!(a1.serial_id, a2.serial_id);
            assert_eq!(a1.amount, a2.amount);
            assert_eq!(a1.nonce, a2.nonce);
            assert_eq!(a1.commitment.as_bytes(), a2.commitment.as_bytes());
        }
    }

    #[cfg(feature = "deterministic-rng")]
    #[test]
    fn test_generate_all_genesis_assets() {
        let definitions = vec![
            AssetDefinition::new(
                [1u8; 32],
                AssetClass::Coin,
                "Z00Z Coin".to_string(),
                "Z00Z".to_string(),
                8,
                5,
                100_000_000,
                "z00z.local".to_string(),
                1,
                1,
                0,
                None,
            )
            .unwrap(),
            AssetDefinition::new(
                [2u8; 32],
                AssetClass::Token,
                "zUSD Token".to_string(),
                "zUSD".to_string(),
                6,
                3,
                1_000_000,
                "zusd.local".to_string(),
                1,
                1,
                0,
                None,
            )
            .unwrap(),
        ];
        let coin_id = definitions[0].id;
        let token_id = definitions[1].id;

        let genesis_seed = [0x77u8; 32];

        use z00z_utils::prelude::{NoopLogger, NoopMetrics};
        let logger = Arc::new(NoopLogger);
        let metrics = Arc::new(NoopMetrics);
        let accumulator = generate_all_genesis_assets(
            &definitions,
            &genesis_seed,
            ChainType::Devnet,
            logger,
            metrics,
        )
        .unwrap();

        assert_eq!(accumulator.total_count(), 8);
        assert_eq!(accumulator.coins.len(), 5);
        assert_eq!(accumulator.tokens.len(), 3);
        assert_eq!(accumulator.nfts.len(), 0);
        assert_eq!(accumulator.voids.len(), 0);

        for (i, coin) in accumulator.coins.iter().enumerate() {
            assert_eq!(coin.serial_id, i as u32);
            assert_eq!(coin.definition.id, coin_id);
        }
        for (i, token) in accumulator.tokens.iter().enumerate() {
            assert_eq!(token.serial_id, i as u32);
            assert_eq!(token.definition.id, token_id);
        }
    }

    #[cfg(not(feature = "deterministic-rng"))]
    #[test]
    fn test_assets_require_det_rng() {
        let definition = AssetDefinition::new(
            [1u8; 32],
            AssetClass::Coin,
            "Test Coin".to_string(),
            "TST".to_string(),
            8,
            10,
            1000,
            "test.local".to_string(),
            1,
            1,
            0,
            None,
        )
        .unwrap();

        let definition_arc = Arc::new(definition);
        let err = generate_assets_checked(definition_arc, &[0x42u8; 32], ChainType::Devnet)
            .expect_err("missing deterministic-rng feature must fail closed");

        assert!(matches!(err, GenesisError::InvalidConfig(_)));
        assert!(err
            .to_string()
            .contains("deterministic-rng feature is required"));
    }

    #[cfg(not(feature = "deterministic-rng"))]
    #[test]
    fn test_all_require_det_rng() {
        let definitions = vec![AssetDefinition::new(
            [1u8; 32],
            AssetClass::Coin,
            "Z00Z Coin".to_string(),
            "Z00Z".to_string(),
            8,
            5,
            100_000_000,
            "z00z.local".to_string(),
            1,
            1,
            0,
            None,
        )
        .unwrap()];

        use z00z_utils::prelude::{NoopLogger, NoopMetrics};

        let logger = Arc::new(NoopLogger);
        let metrics = Arc::new(NoopMetrics);
        let err = generate_all_genesis_assets(
            &definitions,
            &[0x77u8; 32],
            ChainType::Devnet,
            logger,
            metrics,
        )
        .expect_err("missing deterministic-rng feature must fail closed");

        assert!(matches!(err, GenesisError::InvalidConfig(_)));
        assert!(err
            .to_string()
            .contains("deterministic-rng feature is required"));
    }

    #[test]
    fn test_generate_genesis_assets() {
        let definitions = vec![];
        let genesis_seed = [0x88u8; 32];

        use z00z_utils::prelude::{NoopLogger, NoopMetrics};
        let logger = Arc::new(NoopLogger);
        let metrics = Arc::new(NoopMetrics);
        let accumulator = generate_all_genesis_assets(
            &definitions,
            &genesis_seed,
            ChainType::Devnet,
            logger,
            metrics,
        )
        .unwrap();

        assert_eq!(accumulator.total_count(), 0);
        assert_eq!(accumulator.flatten().len(), 0);
    }

    #[test]
    fn test_generate_genesis_rights_deterministic() {
        let rights = vec![sample_right_config()];
        let genesis_seed = [0x42u8; 32];

        let first = generate_genesis_rights(
            &rights,
            &genesis_seed,
            3,
            ChainType::Devnet,
            GENESIS_ROOT_GENERATION,
        )
        .unwrap();
        let second = generate_genesis_rights(
            &rights,
            &genesis_seed,
            3,
            ChainType::Devnet,
            GENESIS_ROOT_GENERATION,
        )
        .unwrap();

        assert_eq!(first, second);
        assert_eq!(first.len(), 2);
    }

    #[cfg(feature = "deterministic-rng")]
    #[test]
    fn test_rejects_asset_right_collision() {
        let definition = AssetDefinition::new(
            [7u8; 32],
            AssetClass::Coin,
            "Test Coin".to_string(),
            "TST".to_string(),
            8,
            1,
            1000,
            "test.local".to_string(),
            1,
            1,
            0,
            None,
        )
        .unwrap();
        let mut rights = generate_genesis_rights(
            &[sample_right_config()],
            &[0x42u8; 32],
            3,
            ChainType::Devnet,
            GENESIS_ROOT_GENERATION,
        )
        .unwrap();
        use z00z_utils::prelude::{NoopLogger, NoopMetrics};
        let logger = Arc::new(NoopLogger);
        let metrics = Arc::new(NoopMetrics);
        let mut corpus = generate_all_genesis_assets(
            &[definition],
            &[0x42u8; 32],
            ChainType::Devnet,
            logger,
            metrics,
        )
        .unwrap();
        rights[0].leaf.terminal_id = corpus.coins[0].asset_id();
        corpus.rights = rights;

        let err = ensure_terminal_collision_free(&corpus)
            .expect_err("asset/right terminal collisions must fail closed");

        assert!(matches!(err, GenesisError::TerminalCollision { .. }));
    }

    #[test]
    fn test_changes_when_rights_change() {
        let mut corpus = GenesisSettlementCorpus::new();
        corpus.rights = generate_genesis_rights(
            &[sample_right_config()],
            &[0x42u8; 32],
            3,
            ChainType::Devnet,
            GENESIS_ROOT_GENERATION,
        )
        .unwrap();

        let original_hash = compute_genesis_state_hash(&corpus);
        corpus.rights[0].leaf.payload_commitment = [9u8; 32];
        let changed_hash = compute_genesis_state_hash(&corpus);

        assert_ne!(original_hash, changed_hash);
    }

    #[test]
    fn test_export_tracks_right_changes() {
        let temp_dir = tempfile::TempDir::new().unwrap();
        let mut corpus = GenesisSettlementCorpus::new();
        corpus.rights = generate_genesis_rights(
            &[sample_right_config()],
            &[0x42u8; 32],
            3,
            ChainType::Devnet,
            GENESIS_ROOT_GENERATION,
        )
        .unwrap();

        let state_hash = compute_genesis_state_hash(&corpus);
        let (rights_path, manifest_path) = export_genesis_settlement_artifacts(
            temp_dir.path(),
            &[],
            &[],
            &corpus,
            ChainType::Devnet,
            GENESIS_ROOT_GENERATION,
            &state_hash,
            &[0x42u8; 32],
        )
        .unwrap();
        assert!(rights_path.exists());
        assert!(manifest_path.exists());

        let manifest: GenesisSettlementManifest =
            z00z_utils::io::load_json(&manifest_path).unwrap();
        assert_eq!(manifest.right_count, corpus.total_right_count());
        assert_eq!(manifest.corpus_digest, hex::encode(state_hash));
        assert_eq!(
            manifest.generation_seed_hash,
            hex::encode(compute_genesis_seed_hash(&[0x42u8; 32]))
        );

        corpus.rights[0].leaf.payload_commitment = [3u8; 32];
        let changed_state_hash = compute_genesis_state_hash(&corpus);
        let (_, changed_manifest_path) = export_genesis_settlement_artifacts(
            temp_dir.path(),
            &[],
            &[],
            &corpus,
            ChainType::Devnet,
            GENESIS_ROOT_GENERATION,
            &changed_state_hash,
            &[0x42u8; 32],
        )
        .unwrap();
        let changed_manifest: GenesisSettlementManifest =
            z00z_utils::io::load_json(&changed_manifest_path).unwrap();

        assert_ne!(manifest.manifest_hash, changed_manifest.manifest_hash);
    }

    #[test]
    fn test_when_holder_control_changes() {
        let genesis_seed = [0x42u8; 32];
        let base = sample_right_config();

        let base_right = generate_genesis_rights(
            &[base.clone()],
            &genesis_seed,
            3,
            ChainType::Devnet,
            GENESIS_ROOT_GENERATION,
        )
        .unwrap();

        let mut holder_changed = base.clone();
        holder_changed.holder_fixture = "wallet_bob".to_string();
        let holder_right = generate_genesis_rights(
            &[holder_changed],
            &genesis_seed,
            3,
            ChainType::Devnet,
            GENESIS_ROOT_GENERATION,
        )
        .unwrap();

        let mut control_changed = base;
        control_changed.control_fixture = "service_orchestrator".to_string();
        let control_right = generate_genesis_rights(
            &[control_changed],
            &genesis_seed,
            3,
            ChainType::Devnet,
            GENESIS_ROOT_GENERATION,
        )
        .unwrap();

        assert_ne!(
            base_right[0].leaf.payload_commitment,
            holder_right[0].leaf.payload_commitment
        );
        assert_ne!(
            base_right[0].leaf.payload_commitment,
            control_right[0].leaf.payload_commitment
        );
    }

    #[test]
    fn test_genesis_policies_canonical_fixture() {
        let config =
            load_genesis_config(canonical_genesis_path().to_str().expect("utf8 path")).unwrap();
        let first = generate_genesis_policies(&config.assets, &config.policies).unwrap();
        let second = generate_genesis_policies(&config.assets, &config.policies).unwrap();

        assert_eq!(first, second);
        assert!(
            first
                .iter()
                .any(|record| record.descriptor.label == "cash_policy_v1"),
            "native cash policy must stay built in",
        );
        assert!(
            first
                .iter()
                .any(|record| record.descriptor.label == "voucher_transferable_policy_v1"),
            "voucher policy fixture must export",
        );
        assert!(
            first
                .iter()
                .any(|record| record.descriptor.label == "right_delegate_policy_v1"),
            "right policy fixture must export",
        );
    }

    #[cfg(feature = "deterministic-rng")]
    #[test]
    fn test_genesis_vouchers_canonical_fixture() {
        let config =
            load_genesis_config(canonical_genesis_path().to_str().expect("utf8 path")).unwrap();
        let genesis_seed = GenesisSeed::from_config(&config).unwrap();
        let network = ChainType::from_str(&config.chain.chain_type).unwrap();
        let definitions = config
            .assets
            .iter()
            .map(|asset| create_asset_definition(asset, genesis_seed.as_bytes(), network))
            .collect::<Result<Vec<_>, _>>()
            .unwrap();
        let policies = generate_genesis_policies(&config.assets, &config.policies).unwrap();
        let corpus = generate_genesis_settlement_corpus(
            &definitions,
            &config.rights,
            &config.vouchers,
            &policies,
            genesis_seed.as_bytes(),
            config.chain.id,
            network,
            Arc::new(NoopLogger),
            Arc::new(NoopMetrics),
        )
        .unwrap();

        assert_eq!(corpus.vouchers.len(), 3);
        assert!(
            corpus
                .vouchers
                .iter()
                .any(|record| record.config.lifecycle == VoucherLifecycleV1::Active),
            "active transferable voucher missing",
        );
        assert!(
            corpus
                .vouchers
                .iter()
                .any(|record| record.config.lifecycle == VoucherLifecycleV1::PendingAcceptance),
            "pending non-transferable voucher missing",
        );
        assert!(
            corpus
                .vouchers
                .iter()
                .any(|record| record.config.lifecycle == VoucherLifecycleV1::Expired),
            "expired negative voucher missing",
        );
        assert!(
            corpus
                .vouchers
                .iter()
                .all(|record| record.config.audit_commitment.is_some()),
            "genesis vouchers must keep audit commitments",
        );
    }

    #[cfg(feature = "deterministic-rng")]
    #[test]
    fn test_genesis_manifest_canonical_fixture() {
        let config =
            load_genesis_config(canonical_genesis_path().to_str().expect("utf8 path")).unwrap();
        let genesis_seed = GenesisSeed::from_config(&config).unwrap();
        let network = ChainType::from_str(&config.chain.chain_type).unwrap();
        let definitions = config
            .assets
            .iter()
            .map(|asset| create_asset_definition(asset, genesis_seed.as_bytes(), network))
            .collect::<Result<Vec<_>, _>>()
            .unwrap();
        let policies = generate_genesis_policies(&config.assets, &config.policies).unwrap();
        let corpus = generate_genesis_settlement_corpus(
            &definitions,
            &config.rights,
            &config.vouchers,
            &policies,
            genesis_seed.as_bytes(),
            config.chain.id,
            network,
            Arc::new(NoopLogger),
            Arc::new(NoopMetrics),
        )
        .unwrap();
        let temp_dir = tempfile::TempDir::new().unwrap();
        let (_, manifest_path) = export_genesis_settlement_artifacts(
            temp_dir.path(),
            &definitions,
            &policies,
            &corpus,
            network,
            GENESIS_ROOT_GENERATION,
            &compute_genesis_state_hash(&corpus),
            genesis_seed.as_bytes(),
        )
        .unwrap();
        let manifest: GenesisSettlementManifest =
            z00z_utils::io::load_json(&manifest_path).unwrap();

        assert_eq!(manifest.policy_count, policies.len());
        assert_eq!(manifest.voucher_count, corpus.total_voucher_count());
        assert_eq!(manifest.policies_artifact, GENESIS_POLICIES_FILE);
        assert_eq!(manifest.vouchers_artifact, GENESIS_VOUCHERS_FILE);
        assert_eq!(manifest.version, 2);
    }
}
